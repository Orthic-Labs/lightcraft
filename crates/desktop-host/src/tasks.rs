use std::collections::{BTreeMap, HashSet};
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::mpsc::{Receiver, SyncSender, sync_channel};

use serde_json::{Value, json};

use lightcraft_catalog::PhotoId;
use lightcraft_engine::import::{ImportJob, ImportOptions, Prepared};
use lightcraft_engine::{Selection, Session};

use crate::snapshot::JobStatus;

const EVENT_CAPACITY: usize = 64;
const MAX_TASKS: usize = 128;

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
    Import {
        id: String,
        prepared: Prepared,
        opts: Box<ImportOptions>,
        now: String,
        album: Option<u64>,
        album_name: Option<String>,
        undo_before: usize,
        cancelled: bool,
    },
}

pub(crate) struct Tasks {
    jobs: BTreeMap<String, Task>,
    tx: SyncSender<Event>,
    rx: Receiver<Event>,
    next: u64,
    notices: Vec<String>,
}

impl Tasks {
    pub(crate) fn new() -> Self {
        let (tx, rx) = sync_channel(EVENT_CAPACITY);
        Self { jobs: BTreeMap::new(), tx, rx, next: 1, notices: Vec::new() }
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
            dir: params
                .get("dir")
                .and_then(Value::as_str)
                .or_else(|| session.last_export.as_ref().and_then(|last| last.get("dir")).and_then(Value::as_str))
                .map(str::to_string)
                .filter(|dir| !dir.trim().is_empty())
                .unwrap_or_else(default_export_dir),
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
        if self.running_kind("import") {
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
                        });
                        log::error!("import task failed: {error}");
                    }
                }
            })
            .map_err(|error| format!("could not start import: {error}"))?;
        self.insert(Task { worker: Some(worker), ..task })?;
        Ok(json!({"taskId": task_id, "total": 0}))
    }

    pub(crate) fn poll(&mut self, session: &mut Session) {
        while let Ok(event) = self.rx.try_recv() {
            match event {
                Event::Export { id, result, cancelled } => self.finish(id, result.map(|files| json!({"files": files})), "export", cancelled),
                Event::Import { id, prepared, opts, now, album, album_name, undo_before, cancelled } => {
                    let result = if now.is_empty() {
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
                                if let Some(first) =
                                    report.get("imported").and_then(Value::as_array).and_then(|ids| ids.first()).and_then(Value::as_u64)
                                {
                                    session.selection = Selection::single(PhotoId(first));
                                }
                            })
                    };
                    self.finish(id, result, "import", cancelled);
                }
            }
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

    pub(crate) fn running_kind(&self, kind: &str) -> bool {
        self.jobs.values().any(|job| job.kind == kind)
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
            if cancelled {
                self.notices.push(format!("{} cancelled", job.label));
            } else if let Err(error) = result {
                self.notices.push(format!("{} failed: {error}", job.label));
            } else {
                self.notices.push(format!("{} finished", job.label));
            }
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

fn default_export_dir() -> String {
    std::env::var_os("HOME")
        .map(|home| std::path::PathBuf::from(home).join("Pictures/LightCraft Exports").to_string_lossy().to_string())
        .unwrap_or_else(|| ".".to_string())
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
