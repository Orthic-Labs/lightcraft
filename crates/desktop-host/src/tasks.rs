use std::collections::{BTreeMap, HashSet, VecDeque};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use serde_json::{Value, json};

use lightcraft_catalog::PhotoId;
use lightcraft_engine::import::{ImportJob, ImportOptions, Prepared};
use lightcraft_engine::merge::{MergeJob, MergeOutput};
use lightcraft_engine::{Selection, Session};

use crate::snapshot::{JobStatus, TerminalTask};

const EVENT_CAPACITY: usize = 64;
const MAX_TASKS: usize = 128;
const MAX_MERGE_IDS: usize = 256;
const MAX_COMPLETED_TASKS: usize = 64;

struct Task {
    id: String,
    kind: String,
    label: String,
    total: Arc<AtomicUsize>,
    completed: Arc<AtomicUsize>,
    cancel: Arc<AtomicBool>,
    worker: Option<std::thread::JoinHandle<()>>,
}

enum Event {
    Export {
        id: String,
        result: Result<Vec<Value>, String>,
        cancelled: bool,
    },
    Merge {
        id: String,
        job: MergeJob,
        result: Result<MergeOutput, String>,
        cancelled: bool,
    },
    Import {
        id: String,
        prepared: Prepared,
        opts: Box<ImportOptions>,
        now: String,
        album: Option<u64>,
        album_name: Option<String>,
        undo_before: usize,
        cancelled: bool,
        preserve_selection: bool,
    },
    LightroomPrepared {
        id: String,
        result: Result<lightcraft_engine::lightroom_job::PreparedLightroom, String>,
        cancelled: bool,
    },
    LightroomFinalized {
        id: String,
        report: Value,
        result: Result<(), String>,
        index_path: Option<String>,
        commit_error: Option<String>,
    },
    LightroomInspection {
        id: String,
        result: Result<Value, String>,
        cancelled: bool,
    },
}

pub(crate) struct Tasks {
    jobs: BTreeMap<String, Task>,
    tx: SyncSender<Event>,
    rx: Receiver<Event>,
    next: u64,
    notices: Vec<String>,
    completed: VecDeque<TerminalTask>,
}

impl Tasks {
    pub(crate) fn new() -> Self {
        let (tx, rx) = sync_channel(EVENT_CAPACITY);
        Self { jobs: BTreeMap::new(), tx, rx, next: 1, notices: Vec::new(), completed: VecDeque::new() }
    }

    pub(crate) fn start_export(&mut self, session: &mut Session, params: &Value) -> Result<Value, String> {
        if self.running_kind("export") {
            return Err("an export is already running".into());
        }
        if self.jobs.len() >= MAX_TASKS {
            return Err("too many background tasks".into());
        }
        let mut expanded = session.export_params(params)?;
        normalize_export_params(&mut expanded);
        let opts = lightcraft_engine::export::ExportOptions::from_json(&expanded);
        let ids = export_targets(session, params);
        if ids.is_empty() {
            return Err("no photos selected for export".into());
        }
        for id in &ids {
            crate::validate_id(id.0)?;
        }
        let destination = lightcraft_engine::export::Destination {
            dir: export_directory(session, params)?,
            exact: params.get("path").and_then(Value::as_str).map(str::to_string),
        };
        if destination.dir.contains('\0') || destination.exact.as_deref().is_some_and(|path| path.contains('\0')) {
            return Err("export path contains NUL".into());
        }
        let items = lightcraft_engine::export::prepare_batch(session, &ids, &opts)?;
        let total = items.len();
        let last_dir = destination.dir.clone();
        let worker_destination = destination.clone();
        let task_id = self.new_id("export");
        let completed = Arc::new(AtomicUsize::new(0));
        let total_counter = Arc::new(AtomicUsize::new(total));
        let cancel = Arc::new(AtomicBool::new(false));
        let task = Task {
            id: task_id.clone(),
            kind: "export".into(),
            label: "Export Photos".into(),
            total: total_counter.clone(),
            completed: completed.clone(),
            cancel: cancel.clone(),
            worker: None,
        };
        let tx = self.tx.clone();
        let id_for_worker = task_id.clone();
        let worker = std::thread::Builder::new()
            .name("lightcraft-export".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("export", || {
                    let mut write = |path: &str, bytes: &[u8]| lightcraft_engine::export::write_file(path, bytes);
                    let exists = |path: &str| std::path::Path::new(path).exists();
                    let mut progress = |done: usize, _name: &str| {
                        completed.store(done, Ordering::Relaxed);
                        !cancel.load(Ordering::Relaxed)
                    };
                    lightcraft_engine::export::run_batch(items, &opts, &worker_destination, &mut write, &exists, false, &mut progress)
                })
                .unwrap_or_else(Err);
                let cancelled = cancel.load(Ordering::Relaxed);
                let _ = tx.send(Event::Export { id: id_for_worker, result, cancelled });
            })
            .map_err(|error| format!("could not start export: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        let mut last = params.clone();
        if let Some(values) = last.as_object_mut() {
            values.remove("ids");
            values.remove("path");
            values.insert("dir".into(), Value::String(last_dir));
        }
        session.last_export = Some(last);
        if let Err(error) = session.save_prefs() {
            self.notices.push(format!("export preferences were not saved: {error}"));
        }
        Ok(json!({"taskId": task_id, "total": total}))
    }

    pub(crate) fn start_import(&mut self, session: &mut Session, params: &Value) -> Result<Value, String> {
        self.start_import_with_selection(session, params, false)
    }

    pub(crate) fn start_auto_import(&mut self, session: &mut Session, params: &Value) -> Result<Value, String> {
        self.start_import_with_selection(session, params, true)
    }

    fn start_import_with_selection(&mut self, session: &mut Session, params: &Value, preserve_selection: bool) -> Result<Value, String> {
        if self.running_import() {
            return Err("an import is already running".into());
        }
        if self.jobs.len() >= MAX_TASKS {
            return Err("too many background tasks".into());
        }
        let request = lightcraft_engine::cmd::library::import_params(session, params).map_err(|error| error.to_string())?;
        if request.paths.is_empty() {
            return Err("no paths to import".into());
        }
        let mut job = ImportJob::new(session, request.opts.clone()).map_err(|error| error.to_string())?;
        let total = job.total.clone();
        let completed = job.done.clone();
        let cancel = Arc::new(AtomicBool::new(false));
        let task_id = self.new_id("import");
        let task = Task {
            id: task_id.clone(),
            kind: "import".into(),
            label: "Import Photos".into(),
            total,
            completed,
            cancel: cancel.clone(),
            worker: None,
        };
        let tx = self.tx.clone();
        let id_for_worker = task_id.clone();
        let paths = request.paths;
        let opts = request.opts;
        let album = request.album;
        let album_name = request.album_name;
        let undo_before = session.undo.len();
        let worker_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("lightcraft-import".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("import", || job.prepare(&paths, &worker_cancel));
                match result {
                    Ok(prepared) => {
                        let cancelled = worker_cancel.load(Ordering::Relaxed);
                        let _ = tx.send(Event::Import {
                            id: id_for_worker,
                            prepared,
                            opts: Box::new(opts),
                            now: job.now().to_string(),
                            album,
                            album_name,
                            undo_before,
                            cancelled,
                            preserve_selection,
                        });
                    }
                    Err(error) => {
                        let empty = Prepared::default();
                        let _ = tx.send(Event::Import {
                            id: id_for_worker,
                            prepared: empty,
                            opts: Box::new(opts),
                            now: String::new(),
                            album,
                            album_name,
                            undo_before,
                            cancelled: worker_cancel.load(Ordering::Relaxed),
                            preserve_selection,
                        });
                        log::error!("import task failed: {error}");
                    }
                }
            })
            .map_err(|error| format!("could not start import: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        Ok(json!({"taskId": task_id, "total": 0}))
    }

    pub(crate) fn start_lightroom_import(&mut self, session: &mut Session, params: &Value) -> Result<Value, String> {
        if self.running_import() {
            return Err("an import is already running".into());
        }
        if self.jobs.len() >= MAX_TASKS {
            return Err("too many background tasks".into());
        }
        let path = lightroom_path(params, "library.importLightroom")?;
        let update_existing = params.get("updateExisting").and_then(Value::as_bool).unwrap_or(false);
        let mut job = lightcraft_engine::lightroom_job::LightroomJob::new(session, path, update_existing).map_err(|error| error.to_string())?;
        let total = job.total_atomic();
        let completed = job.done_atomic();
        let cancel = Arc::new(AtomicBool::new(false));
        let task_id = self.new_id("lightroomImport");
        let task = Task {
            id: task_id.clone(),
            kind: "lightroomImport".into(),
            label: "Import Lightroom Catalog".into(),
            total,
            completed,
            cancel: cancel.clone(),
            worker: None,
        };
        let tx = self.tx.clone();
        let id_for_worker = task_id.clone();
        let worker_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("lightcraft-lightroom-import".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("Lightroom import", || job.prepare(&worker_cancel))
                    .and_then(|result| result.map_err(|error| error.to_string()));
                let cancelled = worker_cancel.load(Ordering::Relaxed);
                let _ = tx.send(Event::LightroomPrepared { id: id_for_worker, result, cancelled });
            })
            .map_err(|error| format!("could not start Lightroom import: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        Ok(json!({"taskId": task_id, "kind": "lightroomImport", "total": 0}))
    }

    pub(crate) fn start_lightroom_inspection(&mut self, _session: &mut Session, params: &Value) -> Result<Value, String> {
        if self.jobs.len() >= MAX_TASKS {
            return Err("too many background tasks".into());
        }
        let path = lightroom_path(params, "library.inspectLightroom")?;
        let total = Arc::new(AtomicUsize::new(16));
        let completed = Arc::new(AtomicUsize::new(0));
        let cancel = Arc::new(AtomicBool::new(false));
        let task_id = self.new_id("lightroomInspect");
        let task = Task {
            id: task_id.clone(),
            kind: "lightroomInspect".into(),
            label: "Inspect Lightroom Catalog".into(),
            total: total.clone(),
            completed: completed.clone(),
            cancel: cancel.clone(),
            worker: None,
        };
        let tx = self.tx.clone();
        let id_for_worker = task_id.clone();
        let worker_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("lightcraft-lightroom-inspect".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("Lightroom inspection", || {
                    lightcraft_engine::lightroom_catalog::read_with_progress(&path, &worker_cancel, &total, &completed)
                        .map(|data| lightroom_inspection_report(&data))
                })
                .unwrap_or_else(Err);
                let cancelled = worker_cancel.load(Ordering::Relaxed);
                let _ = tx.send(Event::LightroomInspection { id: id_for_worker, result, cancelled });
            })
            .map_err(|error| format!("could not start Lightroom inspection: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        Ok(json!({"taskId": task_id, "kind": "lightroomInspect", "total": 16}))
    }

    pub(crate) fn start_merge(&mut self, session: &mut Session, command: &str, params: &Value) -> Result<Value, String> {
        if self.running_kind("merge") {
            return Err("a merge is already running".into());
        }
        if self.jobs.len() >= MAX_TASKS {
            return Err("too many background tasks".into());
        }
        if let Some(raw_ids) = params.get("ids") {
            let ids = raw_ids.as_array().ok_or_else(|| "merge ids must be an array".to_string())?;
            if ids.len() > MAX_MERGE_IDS {
                return Err(format!("merge accepts at most {MAX_MERGE_IDS} photos"));
            }
        }
        let (kind, finish) = lightcraft_engine::merge::parse(command, params).map_err(|error| error.to_string())?;
        let ids = session.targets(params);
        for id in &ids {
            crate::validate_id(id.0)?;
        }
        let job = session.plan_merge(kind, finish, &ids, false).map_err(|error| error.to_string())?;
        let task_id = self.new_id("merge");
        let completed = Arc::new(AtomicUsize::new(0));
        let total = Arc::new(AtomicUsize::new(100));
        let cancel = Arc::new(AtomicBool::new(false));
        let task = Task {
            id: task_id.clone(),
            kind: "merge".into(),
            label: merge_label(command),
            total: total.clone(),
            completed: completed.clone(),
            cancel: cancel.clone(),
            worker: None,
        };
        let tx = self.tx.clone();
        let id_for_worker = task_id.clone();
        let worker_cancel = cancel.clone();
        let worker = std::thread::Builder::new()
            .name("lightcraft-merge".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("merge", || {
                    job.run(&|fraction, _stage| {
                        let progress = (fraction.clamp(0.0, 1.0) * 100.0).round() as usize;
                        completed.store(progress, Ordering::Relaxed);
                        !worker_cancel.load(Ordering::Relaxed)
                    })
                })
                .unwrap_or_else(Err);
                let cancelled = worker_cancel.load(Ordering::Relaxed);
                let _ = tx.send(Event::Merge { id: id_for_worker, job, result, cancelled });
            })
            .map_err(|error| format!("could not start merge: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        Ok(json!({"taskId": task_id, "total": 100}))
    }

    pub(crate) fn poll(&mut self, session: &mut Session) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Export { id, result, cancelled } => self.finish(id, result.map(|files| json!({"files": files})), "export", cancelled),
                Event::Merge { id, job, result, cancelled } => {
                    let cancelled = merge_cancelled(cancelled, self.jobs.get(&id));
                    let result = if cancelled {
                        Err("cancelled".into())
                    } else {
                        result.and_then(|output| session.finish_merge(&job, output).map_err(|error| error.to_string()))
                    };
                    self.finish(id, result, "merge", cancelled);
                }
                Event::Import { id, prepared, opts, now, album, album_name, undo_before, cancelled, preserve_selection } => {
                    let cancelled = cancelled || self.jobs.get(&id).is_some_and(|task| task.cancel.load(Ordering::Relaxed));
                    let selection_before = preserve_selection.then(|| session.selection.clone());
                    let result = if cancelled || now.is_empty() {
                        Err("import worker failed".into())
                    } else {
                        lightcraft_engine::import::commit_prepared(session, &opts, &now, prepared)
                            .map_err(|error| error.to_string())
                            .and_then(|report| {
                                lightcraft_engine::cmd::library::import_batch_done(session, report, album, album_name.as_deref())
                                    .map_err(|error| error.to_string())
                            })
                            .inspect(|report| {
                                let n = session.undo.len().saturating_sub(undo_before);
                                session.merge_undo(n, "Import Photos");
                                if !preserve_selection
                                    && let Some(first) =
                                        report.get("imported").and_then(Value::as_array).and_then(|ids| ids.first()).and_then(Value::as_u64)
                                {
                                    session.selection = Selection::single(PhotoId(first));
                                }
                            })
                    };
                    if let Some(selection) = selection_before {
                        session.selection = selection;
                    }
                    self.finish(id, result, "import", cancelled);
                }
                Event::LightroomPrepared { id, result, cancelled } => self.poll_lightroom_prepared(session, id, result, cancelled),
                Event::LightroomFinalized { id, report, result, index_path, commit_error } => {
                    self.finish_lightroom(id, report, result, index_path, commit_error)
                }
                Event::LightroomInspection { id, result, cancelled } => self.finish(id, result, "lightroomInspect", cancelled),
            }
        }
    }

    fn poll_lightroom_prepared(
        &mut self,
        session: &mut Session,
        id: String,
        result: Result<lightcraft_engine::lightroom_job::PreparedLightroom, String>,
        worker_cancelled: bool,
    ) {
        if let Some(task) = self.jobs.get_mut(&id)
            && let Some(worker) = task.worker.take()
        {
            let _ = worker.join();
        }
        let cancelled = worker_cancelled || self.jobs.get(&id).is_some_and(|task| task.cancel.load(Ordering::Relaxed));
        let prepared = match result {
            Ok(prepared) if !cancelled => prepared,
            Ok(prepared) => {
                prepared.rollback();
                self.finish(id, Err("cancelled".into()), "lightroomImport", true);
                return;
            }
            Err(error) => {
                self.finish(id, Err(error), "lightroomImport", cancelled);
                return;
            }
        };
        let mut completion_out = None;
        let commit_result = session.execute_fn("library.importLightroom", |owner| {
            let completion = lightcraft_engine::lightroom_job::commit_prepared(owner, prepared)?;
            let report = completion.report.clone();
            completion_out = Some(completion);
            Ok(report)
        });
        let Some(completion) = completion_out else {
            self.finish(id, commit_result.map_err(|error| error.to_string()), "lightroomImport", false);
            return;
        };
        let commit_error = commit_result.err().map(|error| error.to_string());
        let report = completion.report;
        let index_path = completion.finalization.path().map(|path| path.to_string_lossy().to_string());
        let report_for_worker = report.clone();
        let index_path_for_worker = index_path.clone();
        let commit_error_for_worker = commit_error.clone();
        let tx = self.tx.clone();
        let id_for_worker = id.clone();
        let worker = std::thread::Builder::new().name("lightcraft-lightroom-index".into()).spawn(move || {
            let result = lightcraft_engine::guard::catch("Lightroom archive index", || completion.finalization.finish())
                .and_then(|result| result.map_err(|error| error.to_string()));
            let _ = tx.send(Event::LightroomFinalized {
                id: id_for_worker,
                report: report_for_worker,
                result,
                index_path: index_path_for_worker,
                commit_error: commit_error_for_worker,
            });
        });
        match worker {
            Ok(worker) => {
                if let Some(task) = self.jobs.get_mut(&id) {
                    task.completed.store(task.total.load(Ordering::Relaxed), Ordering::Relaxed);
                    task.worker = Some(worker);
                } else {
                    let _ = worker.join();
                }
            }
            Err(error) => {
                self.finish_lightroom(id, report, Err(format!("could not start Lightroom archive finalization: {error}")), index_path, commit_error);
            }
        }
    }

    fn finish_lightroom(
        &mut self,
        id: String,
        mut report: Value,
        result: Result<(), String>,
        index_path: Option<String>,
        commit_error: Option<String>,
    ) {
        let Some(mut job) = self.jobs.remove(&id) else {
            self.notices.push("unknown lightroomImport task completed".into());
            return;
        };
        if let Some(worker) = job.worker.take() {
            let _ = worker.join();
        }
        job.completed.store(job.total.load(Ordering::Relaxed), Ordering::Relaxed);
        let archive_error = match result {
            Ok(()) => None,
            Err(error) => {
                let warning = match &index_path {
                    Some(path) => format!("Lightroom import committed, but archive index could not be written at {path}: {error}"),
                    None => format!("Lightroom import committed, but archive index could not be finalized: {error}"),
                };
                append_warning(&mut report, warning.clone());
                append_index_warning(&mut report);
                Some(warning)
            }
        };
        let error =
            commit_error.map(|error| format!("Lightroom import was applied in memory, but library save failed: {error}; retry saving the library"));
        if let Some(commit_error) = &error {
            append_warning(&mut report, commit_error.clone());
        }
        let state = if error.is_some() { "failed" } else { "done" };
        let notice = error.clone().or(archive_error).unwrap_or_else(|| format!("{} finished", job.label));
        self.completed.push_back(TerminalTask { id: job.id, kind: job.kind, label: job.label, state: state.into(), result: Some(report), error });
        while self.completed.len() > MAX_COMPLETED_TASKS {
            self.completed.pop_front();
        }
        self.notices.push(notice);
        if self.notices.len() > 32 {
            let drop_count = self.notices.len() - 32;
            self.notices.drain(..drop_count);
        }
    }

    pub(crate) fn cancel(&mut self, id: Option<&str>) -> Result<Value, String> {
        let mut changed = 0usize;
        for job in self.jobs.values() {
            if id.is_none_or(|wanted| wanted == job.id) {
                job.cancel.store(true, Ordering::Relaxed);
                changed += 1;
            }
        }
        Ok(json!({"cancelled": changed}))
    }

    pub(crate) fn statuses(&self) -> Vec<JobStatus> {
        self.jobs
            .values()
            .map(|job| JobStatus {
                id: job.id.clone(),
                kind: job.kind.clone(),
                label: job.label.clone(),
                completed: job.completed.load(Ordering::Relaxed),
                total: job.total.load(Ordering::Relaxed),
                cancellable: true,
                error: None,
            })
            .collect()
    }

    pub(crate) fn completed_jobs(&self) -> Vec<TerminalTask> {
        self.completed.iter().cloned().collect()
    }

    pub(crate) fn running_kind(&self, kind: &str) -> bool {
        self.jobs.values().any(|job| if kind == "import" { job.kind == "import" || job.kind == "lightroomImport" } else { job.kind == kind })
    }

    fn running_import(&self) -> bool {
        self.running_kind("import")
    }
    pub(crate) fn running(&self) -> bool {
        !self.jobs.is_empty()
    }
    pub(crate) fn take_notices(&mut self) -> Vec<String> {
        std::mem::take(&mut self.notices)
    }

    fn new_id(&mut self, kind: &str) -> String {
        let id = format!("{kind}-{}", self.next);
        self.next = self.next.saturating_add(1);
        id
    }

    fn insert(&mut self, task: Task) -> Result<(), String> {
        if self.jobs.len() >= MAX_TASKS {
            task.cancel.store(true, Ordering::Relaxed);
            if let Some(worker) = task.worker {
                let _ = worker.join();
            }
            return Err("too many background tasks".into());
        }
        self.jobs.insert(task.id.clone(), task);
        Ok(())
    }

    fn finish(&mut self, id: String, result: Result<Value, String>, kind: &str, cancelled: bool) {
        if let Some(mut job) = self.jobs.remove(&id) {
            if let Some(worker) = job.worker.take() {
                let _ = worker.join();
            }
            job.completed.store(job.total.load(Ordering::Relaxed), Ordering::Relaxed);
            let (state, result, error, notice) = if cancelled {
                ("cancelled", None, None, format!("{} cancelled", job.label))
            } else {
                match result {
                    Ok(result) => ("done", Some(result), None, format!("{} finished", job.label)),
                    Err(error) => ("failed", None, Some(error.clone()), format!("{} failed: {error}", job.label)),
                }
            };
            self.completed.push_back(TerminalTask { id: job.id, kind: job.kind, label: job.label, state: state.into(), result, error });
            while self.completed.len() > MAX_COMPLETED_TASKS {
                self.completed.pop_front();
            }
            self.notices.push(notice);
        } else {
            self.notices.push(format!("unknown {kind} task completed"));
        }
        if self.notices.len() > 32 {
            let drop_count = self.notices.len() - 32;
            self.notices.drain(..drop_count);
        }
    }
}

fn export_targets(session: &mut Session, params: &Value) -> Vec<PhotoId> {
    if let Some(ids) = params.get("ids").and_then(Value::as_array) {
        return ids.iter().filter_map(Value::as_u64).map(PhotoId).collect();
    }
    if let Some(id) = params.get("id").and_then(Value::as_u64) {
        return vec![PhotoId(id)];
    }
    if session.selection.ids.is_empty() {
        return session.active().into_iter().collect();
    }
    let selected: HashSet<PhotoId> = session.selection.ids.iter().copied().collect();
    let visible = session.visible_cloned();
    let mut ids: Vec<PhotoId> = visible.iter().copied().filter(|id| selected.contains(id)).collect();
    let shown: HashSet<PhotoId> = ids.iter().copied().collect();
    ids.extend(session.selection.ids.iter().copied().filter(|id| !shown.contains(id)));
    ids
}

fn export_directory(session: &Session, params: &Value) -> Result<String, String> {
    if let Some(value) = params.get("dir") {
        let dir = value.as_str().ok_or_else(|| "Choose an export folder first.".to_string())?;
        if dir.trim().is_empty() {
            return Err("Choose an export folder first.".into());
        }
        return Ok(dir.to_string());
    }
    session
        .last_export
        .as_ref()
        .and_then(|last| last.get("dir"))
        .and_then(Value::as_str)
        .filter(|dir| !dir.trim().is_empty())
        .map(str::to_string)
        .ok_or_else(|| "Choose an export folder first.".into())
}

fn merge_label(command: &str) -> String {
    match command {
        "merge.hdr" => "HDR Merge".into(),
        "merge.panorama" => "Panorama Merge".into(),
        "merge.hdrPanorama" => "HDR Panorama Merge".into(),
        _ => "Photo Merge".into(),
    }
}

fn merge_cancelled(worker_cancelled: bool, task: Option<&Task>) -> bool {
    worker_cancelled || task.is_some_and(|task| task.cancel.load(Ordering::Relaxed))
}

fn lightroom_path(params: &Value, command: &str) -> Result<std::path::PathBuf, String> {
    let path = params.get("path").and_then(Value::as_str).ok_or_else(|| format!("{command} requires path"))?;
    if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
        return Err(format!("invalid {command} path"));
    }
    Ok(std::path::PathBuf::from(path))
}

fn lightroom_inspection_report(data: &lightcraft_engine::lightroom_catalog::CatalogImport) -> Value {
    let collections = data.collections.iter().filter(|row| row.get("systemOnly").and_then(Value::as_f64).unwrap_or(0.0) == 0.0).count();
    let virtual_copies = data.photos.iter().filter(|photo| photo.image.get("masterImage").and_then(Value::as_i64).unwrap_or(0) != 0).count();
    let missing = data
        .photos
        .iter()
        .filter(|photo| !std::path::Path::new(&photo.path).is_file())
        .map(|photo| Value::String(photo.path.clone()))
        .collect::<Vec<_>>();
    json!({
        "source": data.source,
        "photos": data.photos.len(),
        "collections": collections,
        "virtualCopies": virtual_copies,
        "missing": missing,
        "warnings": data.warnings,
    })
}

fn append_warning(report: &mut Value, warning: String) {
    let Some(object) = report.as_object_mut() else { return };
    match object.get_mut("warnings") {
        Some(Value::Array(warnings)) => warnings.push(Value::String(warning)),
        _ => {
            object.insert("warnings".into(), json!([warning]));
        }
    }
}

fn append_index_warning(report: &mut Value) {
    let Some(object) = report.as_object_mut() else { return };
    object.insert(
        "indexWarning".into(),
        Value::String("archive index finalization failed; retry import after checking file permissions and free space".into()),
    );
}

fn normalize_export_params(params: &mut Value) {
    let Some(values) = params.as_object_mut() else { return };
    if values.get("naming").is_none()
        && let Some(rename) = values.remove("rename")
    {
        values.insert("naming".into(), rename);
    }
    let full_size = values.get("fullSize").and_then(Value::as_bool).unwrap_or(false);
    let mode = values.get("resizeMode").and_then(Value::as_str).map(str::to_string);
    let size = values.get("resizeValue").cloned();
    for key in ["fullSize", "resizeMode", "resizeValue"] {
        values.remove(key);
    }
    if full_size {
        for key in ["resize", "longEdge", "shortEdge", "width", "height", "megapixels", "percent"] {
            values.remove(key);
        }
        values.insert("longEdge".into(), Value::from(0));
    } else if let (Some(mode), Some(size)) = (mode, size) {
        for key in ["resize", "longEdge", "shortEdge", "width", "height", "megapixels", "percent"] {
            values.remove(key);
        }
        let key = match mode.as_str() {
            "longEdge" => Some("longEdge"),
            "shortEdge" => Some("shortEdge"),
            "width" => Some("width"),
            "height" => Some("height"),
            "megapixels" => Some("megapixels"),
            "percent" => Some("percent"),
            _ => None,
        };
        if let Some(key) = key {
            values.insert(key.into(), size);
        }
    } else if !["resize", "longEdge", "shortEdge", "width", "height", "megapixels", "percent"].iter().any(|key| values.contains_key(*key)) {
        values.insert("longEdge".into(), Value::from(2048));
    }
    if values.get("format").is_none() {
        let extension = values.get("path").and_then(Value::as_str).and_then(|path| path.rsplit_once('.').map(|(_, ext)| ext.to_string()));
        if let Some(extension) = extension {
            values.insert("format".into(), Value::String(extension));
        }
    }
}

impl Default for Tasks {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
impl Tasks {
    pub(crate) fn hold_for_test(&mut self) -> (Arc<AtomicBool>, Arc<AtomicBool>) {
        let id = self.new_id("test");
        let cancel = Arc::new(AtomicBool::new(false));
        let started = Arc::new(AtomicBool::new(false));
        let release = Arc::new(AtomicBool::new(false));
        let worker_release = release.clone();
        let worker_started = started.clone();
        let tx = self.tx.clone();
        let worker_id = id.clone();
        let worker = std::thread::spawn(move || {
            worker_started.store(true, Ordering::Release);
            while !worker_release.load(Ordering::Acquire) {
                std::thread::yield_now();
            }
            let _ = tx.send(Event::Export { id: worker_id, result: Ok(Vec::new()), cancelled: true });
        });
        let task = Task {
            id: id.clone(),
            kind: "test".into(),
            label: "Test task".into(),
            total: Arc::new(AtomicUsize::new(1)),
            completed: Arc::new(AtomicUsize::new(0)),
            cancel,
            worker: Some(worker),
        };
        self.jobs.insert(id, task);
        (started, release)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    fn finished_task(tasks: &mut Tasks, id: &str, result: Result<Value, String>) {
        tasks.jobs.insert(
            id.into(),
            Task {
                id: id.into(),
                kind: "merge".into(),
                label: "Merge Photos".into(),
                total: Arc::new(AtomicUsize::new(1)),
                completed: Arc::new(AtomicUsize::new(0)),
                cancel: Arc::new(AtomicBool::new(false)),
                worker: None,
            },
        );
        tasks.finish(id.into(), result, "merge", false);
    }

    fn task_with_cancel(cancelled: bool) -> Task {
        Task {
            id: "merge-test-1".into(),
            kind: "merge".into(),
            label: "Merge test".into(),
            total: Arc::new(AtomicUsize::new(100)),
            completed: Arc::new(AtomicUsize::new(100)),
            cancel: Arc::new(AtomicBool::new(cancelled)),
            worker: None,
        }
    }

    #[test]
    fn merge_completion_cancel_wins_worker_success_race() {
        let task = task_with_cancel(true);
        assert!(merge_cancelled(false, Some(&task)));
        assert!(merge_cancelled(true, Some(&task_with_cancel(false))));
        assert!(!merge_cancelled(false, Some(&task_with_cancel(false))));
    }

    #[test]
    fn terminal_record_survives_completion_before_first_snapshot() {
        let mut tasks = Tasks::new();
        finished_task(&mut tasks, "merge-early", Ok(json!({"id": 41, "path": "merged.dng"})));
        let completed = tasks.completed_jobs();
        assert_eq!(completed.len(), 1);
        assert_eq!(completed[0].id, "merge-early");
        assert_eq!(completed[0].state, "done");
        assert_eq!(completed[0].result.as_ref().and_then(|value| value.get("id")).and_then(Value::as_u64), Some(41));
    }

    #[test]
    fn terminal_records_keep_bounded_recent_history() {
        let mut tasks = Tasks::new();
        for index in 0..(MAX_COMPLETED_TASKS + 3) {
            finished_task(&mut tasks, &format!("merge-{index}"), Ok(Value::Null));
        }
        let completed = tasks.completed_jobs();
        assert_eq!(completed.len(), MAX_COMPLETED_TASKS);
        assert_eq!(completed.first().map(|task| task.id.as_str()), Some("merge-3"));
        assert_eq!(completed.last().map(|task| task.id.as_str()), Some("merge-66"));
    }

    #[test]
    fn blank_export_directory_requires_user_choice() {
        let mut session = Session::new();
        let expected = "Choose an export folder first.";
        assert_eq!(export_directory(&session, &json!({})).expect_err("first export must require a folder"), expected);
        assert_eq!(export_directory(&session, &json!({"dir": "/chosen"})).expect("explicit export folder"), "/chosen");
        session.last_export = Some(json!({"dir": "/previous"}));
        assert_eq!(export_directory(&session, &json!({})).expect("previous export folder"), "/previous");
        assert_eq!(export_directory(&session, &json!({"dir": "  "})).expect_err("blank export destination must be rejected"), expected);
    }

    #[test]
    fn lightroom_inspection_report_counts_virtual_copies_and_preserves_schema() {
        let data = lightcraft_engine::lightroom_catalog::CatalogImport {
            source: "catalog.lrcat".into(),
            photos: vec![
                lightcraft_engine::lightroom_catalog::CatalogPhoto {
                    source_id: 1,
                    uuid: "master".into(),
                    path: "/missing/master.raw".into(),
                    image: serde_json::from_value(json!({"id_local": 1})).unwrap_or_default(),
                    settings: String::new(),
                    xmp: String::new(),
                    keywords: Vec::new(),
                    history: Vec::new(),
                    snapshots: Vec::new(),
                },
                lightcraft_engine::lightroom_catalog::CatalogPhoto {
                    source_id: 2,
                    uuid: "copy".into(),
                    path: "/missing/master.raw".into(),
                    image: serde_json::from_value(json!({"id_local": 2, "masterImage": 1})).unwrap_or_default(),
                    settings: String::new(),
                    xmp: String::new(),
                    keywords: Vec::new(),
                    history: Vec::new(),
                    snapshots: Vec::new(),
                },
            ],
            collections: vec![
                serde_json::from_value(json!({"name": "Visible"})).unwrap_or_default(),
                serde_json::from_value(json!({"systemOnly": 1})).unwrap_or_default(),
            ],
            members: Vec::new(),
            collection_content: Vec::new(),
            warnings: vec!["metadata warning".into()],
        };
        let report = lightroom_inspection_report(&data);
        assert_eq!(report["source"], "catalog.lrcat");
        assert_eq!(report["photos"], 2);
        assert_eq!(report["collections"], 1);
        assert_eq!(report["virtualCopies"], 1);
        assert_eq!(report["missing"].as_array().map(Vec::len), Some(2));
        assert_eq!(report["warnings"][0], "metadata warning");
    }

    #[test]
    fn lightroom_archive_failure_keeps_committed_report_and_actionable_warning() {
        let mut tasks = Tasks::new();
        tasks.jobs.insert(
            "lightroomImport-1".into(),
            Task {
                id: "lightroomImport-1".into(),
                kind: "lightroomImport".into(),
                label: "Import Lightroom Catalog".into(),
                total: Arc::new(AtomicUsize::new(1)),
                completed: Arc::new(AtomicUsize::new(0)),
                cancel: Arc::new(AtomicBool::new(false)),
                worker: None,
            },
        );
        tasks.finish_lightroom(
            "lightroomImport-1".into(),
            json!({"source": "catalog.lrcat", "photos": 2, "warnings": []}),
            Err("permission denied".into()),
            Some("/library/Interop/lightroom-index.json".into()),
            None,
        );
        let completed = tasks.completed_jobs();
        assert_eq!(completed[0].state, "done");
        assert_eq!(completed[0].result.as_ref().and_then(|value| value.get("photos")).and_then(Value::as_u64), Some(2));
        assert!(completed[0].result.as_ref().is_some_and(|value| value.get("indexWarning").is_some()));
        assert!(completed[0].error.as_deref().is_some_and(|error| error.contains("permission denied")));
    }
}
