use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant};

use serde_json::{Map, Value, json};

use lightcraft_engine::Session;

use crate::tasks::Tasks;
use crate::{HostOptions, PreviewDescriptor, PreviewRequest, PreviewStore, Renderer, Request};

const TICK: Duration = Duration::from_millis(250);
const COMMAND_POLL: Duration = Duration::from_millis(20);

pub struct Controller {
    session: Session,
    renderer: Renderer,
    tasks: Tasks,
    snapshot_cache: crate::snapshot::CatalogSnapshotCache,
    preferences: Map<String, Value>,
    notices: Vec<String>,
    last_error: Option<String>,
}

impl Controller {
    pub(crate) fn new(options: HostOptions, store: PreviewStore) -> Result<Self, String> {
        let HostOptions { library_path, demo, demo_count } = options;
        let mut session = if let Some(path) = library_path {
            let mut session = Session::new();
            session.open_library(&path, demo).map_err(|error| error.to_string())?;
            session
        } else if demo {
            Session::with_demo()
        } else {
            Session::new()
        };
        if demo_count > 0 {
            let visible = session.visible_cloned();
            if demo_count < visible.len() {
                for id in visible.into_iter().skip(demo_count) {
                    let _ = session.catalog.apply(lightcraft_catalog::Op::RemovePhoto { id });
                }
            }
        }
        let notices = session.take_library_warnings();
        Ok(Self {
            session,
            renderer: Renderer::new(store),
            tasks: Tasks::new(),
            snapshot_cache: crate::snapshot::CatalogSnapshotCache::default(),
            preferences: Map::new(),
            notices,
            last_error: None,
        })
    }

    pub(crate) fn serve(&mut self, rx: Receiver<Request>) {
        let mut next_tick = Instant::now() + TICK;
        loop {
            let timeout = next_tick.saturating_duration_since(Instant::now()).min(COMMAND_POLL);
            match rx.recv_timeout(timeout) {
                Ok(request) => {
                    let stop = self.handle(request);
                    self.poll();
                    if Instant::now() >= next_tick {
                        self.session.persist_if_dirty();
                        next_tick = Instant::now() + TICK;
                    }
                    if stop {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {
                    self.poll();
                    if Instant::now() >= next_tick {
                        self.session.persist_if_dirty();
                        next_tick = Instant::now() + TICK;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => {
                    let _ = self.tasks.cancel(None);
                    while self.tasks.running() {
                        self.poll();
                        std::thread::sleep(Duration::from_millis(10));
                    }
                    self.renderer.cancel_all();
                    let _ = self.session.close_library();
                    break;
                }
            }
        }
    }

    fn handle(&mut self, request: Request) -> bool {
        match request {
            Request::Run { id, params, reply } => {
                let result = self.run(&id, &params);
                let _ = reply.send(result);
                false
            }
            Request::Snapshot { reply } => {
                let result = self.snapshot();
                let _ = reply.send(result);
                false
            }
            Request::ViewSlice { generation, offset, limit, reply } => {
                let result = self.slice(generation, offset, limit);
                let _ = reply.send(result);
                false
            }
            Request::Preview { request, reply } => {
                let result = self.preview(request, reply);
                result
            }
            Request::Preferences { patch, reply } => {
                let result = self.preferences(patch);
                let _ = reply.send(result);
                false
            }
            Request::Persist { reply } => {
                let result = self.persist();
                let _ = reply.send(result);
                false
            }
            Request::Shutdown { reply } => {
                let result = self.shutdown();
                let stop = result.is_ok();
                let _ = reply.send(result);
                stop
            }
        }
    }

    fn run(&mut self, id: &str, params: &Value) -> Result<Value, String> {
        let result = lightcraft_engine::guard::catch("desktop command", || match id {
            "app.export" => self.tasks.start_export(&mut self.session, params),
            "app.exportPrevious" => {
                let previous = self.session.last_export.clone().ok_or_else(|| "nothing exported yet — use Export…".to_string())?;
                self.tasks.start_export(&mut self.session, &previous)
            }
            "library.import" => self.tasks.start_import(&mut self.session, params),
            "library.backup" => self.backup(params),
            "library.restore" => self.restore(params),
            "library.save" => self.persist().map(|()| Value::Null),
            "app.export.cancel" | "task.cancel" => self.tasks.cancel(params.get("taskId").or_else(|| params.get("id")).and_then(Value::as_str)),
            "ui.preferences" => self.preferences(Some(params.clone())),
            _ => self.session.execute(id, params).map_err(|error| error.to_string()),
        })
        .map_err(|error| {
            self.last_error = Some(error.clone());
            error
        })?;
        match result {
            Ok(value) => {
                self.last_error = None;
                Ok(value)
            }
            Err(error) => {
                self.last_error = Some(error.clone());
                Err(error)
            }
        }
    }

    fn snapshot(&mut self) -> Result<Value, String> {
        self.poll();
        let error = self.last_error.clone().or_else(|| self.session.unsaved().map(|(_, message)| message.to_string()));
        let value = crate::snapshot::snapshot(&mut self.session, &self.tasks, &self.preferences, &self.notices, error, &mut self.snapshot_cache)?;
        self.last_error = None;
        Ok(value)
    }

    fn slice(&mut self, generation: Option<u64>, offset: usize, limit: usize) -> Result<Value, String> {
        crate::snapshot::view_slice(&mut self.session, generation, offset, limit)
    }

    fn preview(&mut self, request: PreviewRequest, reply: Sender<Result<PreviewDescriptor, String>>) -> bool {
        let result = lightcraft_engine::guard::catch("desktop preview request", || self.renderer.request(&mut self.session, request, reply.clone()));
        match result {
            Ok(Ok(())) => {}
            Ok(Err(error)) | Err(error) => {
                log::error!("preview request: {error}");
                let _ = reply.send(Err(error));
            }
        }
        self.renderer.poll(&mut self.session);
        false
    }

    fn preferences(&mut self, patch: Option<Value>) -> Result<Value, String> {
        if let Some(Value::Object(values)) = patch {
            self.preferences.extend(values);
        } else if patch.is_some() {
            return Err("preferences patch must be an object".into());
        }
        Ok(Value::Object(self.preferences.clone()))
    }

    fn persist(&mut self) -> Result<(), String> {
        self.session.persist_if_dirty();
        if let Err(error) = self.session.save_prefs() {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            return Err(message);
        }
        if let Some((count, error)) = self.session.unsaved() {
            let message = format!("{count} library change(s) remain unsaved: {error}");
            self.last_error = Some(message.clone());
            return Err(message);
        }
        self.last_error = None;
        Ok(())
    }

    fn poll(&mut self) {
        if let Err(error) = lightcraft_engine::guard::catch("desktop task poll", || self.tasks.poll(&mut self.session)) {
            self.last_error = Some(error);
        }
        self.notices.extend(self.tasks.take_notices());
        if self.notices.len() > 64 {
            let drop_count = self.notices.len() - 64;
            self.notices.drain(..drop_count);
        }
        if let Err(error) = lightcraft_engine::guard::catch("desktop preview poll", || self.renderer.poll(&mut self.session)) {
            self.last_error = Some(error);
        }
    }

    fn shutdown(&mut self) -> Result<(), String> {
        self.tasks.cancel(None)?;
        self.poll();
        if self.tasks.running() {
            return Err("background tasks are still stopping; retry shutdown after progress completes".into());
        }
        self.renderer.cancel_all();
        self.session.persist().map_err(|error| {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            message
        })?;
        self.session.close_library().map_err(|error| {
            let message = error.to_string();
            self.last_error = Some(message.clone());
            message
        })
    }

    fn backup(&mut self, params: &Value) -> Result<Value, String> {
        let path = params.get("path").and_then(Value::as_str).ok_or_else(|| "library.backup requires path".to_string())?;
        if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
            return Err("invalid library backup path".into());
        }
        if self.tasks.running() {
            return Err("cannot back up while a background task is running; wait for it to finish".into());
        }
        let source = self
            .session
            .library
            .as_ref()
            .filter(|library| library.on_disk)
            .map(|library| library.dir.clone())
            .ok_or_else(|| "no on-disk library is open".to_string())?;
        self.session.close_library().map_err(|error| format!("library must be saved before backup: {error}"))?;
        let destination = backup_destination(Path::new(path), &source)?;
        if destination.exists() {
            return Err("backup destination already exists; refusing to overwrite it".into());
        }
        if let Err(error) = copy_tree(&source, &destination, 0) {
            let _ = std::fs::remove_dir_all(&destination);
            return Err(error);
        }
        Ok(json!({"path": destination.to_string_lossy(), "source": source.to_string_lossy()}))
    }

    fn restore(&mut self, params: &Value) -> Result<Value, String> {
        let path = params.get("path").and_then(Value::as_str).ok_or_else(|| "library.restore requires path".to_string())?;
        if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
            return Err("invalid library backup path".into());
        }
        if self.tasks.running() {
            return Err("cannot restore while a background task is running; wait for it to finish".into());
        }
        if !Path::new(path).is_dir() {
            return Err("library restore expects a backup directory".into());
        }
        let backup = std::fs::canonicalize(path).map_err(|error| format!("library restore path: {error}"))?;
        self.session.persist().map_err(|error| format!("library must be saved before restore: {error}"))?;
        self.renderer.cancel_all();
        self.session.close_library().map_err(|error| format!("could not close current library before restore: {error}"))?;
        if let Err(error) = self.session.open_library(&backup, false) {
            let message = format!("could not open restored library: {error}");
            self.last_error = Some(message.clone());
            return Err(message);
        }
        self.snapshot_cache = crate::snapshot::CatalogSnapshotCache::default();
        self.notices.extend(self.session.take_library_warnings());
        Ok(json!({"path": backup.to_string_lossy(), "libraryPath": backup.to_string_lossy()}))
    }
}

fn backup_destination(path: &Path, source: &Path) -> Result<PathBuf, String> {
    let parent = path.parent().filter(|parent| !parent.as_os_str().is_empty()).unwrap_or_else(|| Path::new("."));
    let parent = std::fs::canonicalize(parent).map_err(|error| format!("backup destination parent: {error}"))?;
    let name = path.file_name().ok_or_else(|| "backup path has no file name".to_string())?;
    let destination = parent.join(name);
    let source = std::fs::canonicalize(source).map_err(|error| format!("library path: {error}"))?;
    if destination == source || destination.starts_with(&source) || source.starts_with(&destination) {
        return Err("backup destination must be outside open library".into());
    }
    Ok(destination)
}

fn copy_tree(source: &Path, destination: &Path, depth: usize) -> Result<(), String> {
    if depth > 64 {
        return Err("library backup directory nesting is too deep".into());
    }
    let metadata = std::fs::symlink_metadata(source).map_err(|error| format!("backup read {}: {error}", source.display()))?;
    if metadata.file_type().is_symlink() {
        return Err(format!("library backup refuses symlink {}", source.display()));
    }
    if metadata.is_dir() {
        std::fs::create_dir(destination).map_err(|error| format!("backup create {}: {error}", destination.display()))?;
        for entry in std::fs::read_dir(source).map_err(|error| format!("backup list {}: {error}", source.display()))? {
            let entry = entry.map_err(|error| format!("backup entry {}: {error}", source.display()))?;
            copy_tree(&entry.path(), &destination.join(entry.file_name()), depth.saturating_add(1))?;
        }
        Ok(())
    } else if metadata.is_file() {
        std::fs::copy(source, destination).map(|_| ()).map_err(|error| format!("backup copy {}: {error}", source.display()))
    } else {
        Err(format!("unsupported library entry {}", source.display()))
    }
}

#[cfg(test)]
mod tests {
    use std::sync::atomic::Ordering;

    use super::*;

    #[test]
    fn failed_shutdown_keeps_owner_available_until_task_finishes() {
        let mut controller = Controller::new(HostOptions { demo: true, ..HostOptions::default() }, PreviewStore::default()).expect("demo controller");
        let (started, release) = controller.tasks.hold_for_test();
        while !started.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        assert!(controller.shutdown().is_err());
        assert!(controller.snapshot().is_ok());
        release.store(true, Ordering::Release);
        while controller.tasks.running() {
            controller.poll();
            std::thread::yield_now();
        }
        assert!(controller.shutdown().is_ok());
    }
}
