use std::path::{Path, PathBuf};
use std::sync::mpsc::{Receiver, Sender};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value, json};

use lightcraft_engine::Session;

use crate::tasks::Tasks;
use crate::{HostOptions, PreviewDescriptor, PreviewRequest, PreviewStore, Renderer, Request};

const TICK: Duration = Duration::from_millis(250);
const COMMAND_POLL: Duration = Duration::from_millis(20);
const PREVIEW_SHUTDOWN_WAIT: Duration = Duration::from_secs(5);

pub struct Controller {
    session: Session,
    renderer: Renderer,
    tasks: Tasks,
    snapshot_cache: crate::snapshot::CatalogSnapshotCache,
    preferences: Map<String, Value>,
    notices: Vec<String>,
    last_error: Option<String>,
    segmenter_dir: Option<PathBuf>,
    segmenter_mirrors_file: Option<PathBuf>,
}

impl Controller {
    pub(crate) fn new(options: HostOptions, store: PreviewStore) -> Result<Self, String> {
        let HostOptions { library_path, demo, demo_count, sam3_dir, sam3_mirrors_file } = options;
        let segmenter_dir = sam3_dir
            .or_else(|| std::env::var_os("LIGHTCRAFT_SAM3_DIR").map(PathBuf::from))
            .or_else(|| lightcraft_engine::camera_profiles::config_dir().map(|dir| dir.join("models").join("sam3")));
        let segmenter_mirrors_file =
            sam3_mirrors_file.or_else(|| lightcraft_engine::camera_profiles::config_dir().map(|dir| dir.join("models").join("sam3-mirrors.txt")));
        let mut session = if let Some(path) = library_path {
            let mut session = Session::new().with_fs().with_system_clock();
            session.open_library(&path, demo).map_err(|error| error.to_string())?;
            session
        } else if demo {
            Session::with_demo().with_fs().with_system_clock()
        } else {
            Session::new().with_fs().with_system_clock()
        };
        configure_segmenter(&mut session, &segmenter_dir, &segmenter_mirrors_file);
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
            segmenter_dir,
            segmenter_mirrors_file,
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
                    if let Err(error) = cancel_preview_build(&self.session, PREVIEW_SHUTDOWN_WAIT) {
                        log::error!("desktop owner disconnected before preview build stopped: {error}");
                        break;
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
            Request::Preview { request, reply } => self.preview(request, reply),
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
            "library.open" => self.open(params),
            "library.restore" => self.restore(params),
            "library.save" => self.persist().map(|()| Value::Null),
            "app.export.cancel" | "task.cancel" => self.tasks.cancel(params.get("taskId").or_else(|| params.get("id")).and_then(Value::as_str)),
            "ui.preferences" => self.preferences(Some(params.clone())),
            _ => self.session.execute(id, params).map_err(|error| error.to_string()),
        })
        .inspect_err(|error| {
            self.last_error = Some(error.clone());
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

    pub(crate) fn shutdown(&mut self) -> Result<(), String> {
        self.tasks.cancel(None)?;
        self.poll();
        if self.tasks.running() {
            return Err("background tasks are still stopping; retry shutdown after progress completes".into());
        }
        cancel_preview_build(&self.session, PREVIEW_SHUTDOWN_WAIT)?;
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

    fn open(&mut self, params: &Value) -> Result<Value, String> {
        let path = library_path(params, "library.open")?;
        let (path, photos) = self.switch_library(path)?;
        Ok(json!({"path": path.to_string_lossy(), "photos": photos}))
    }

    /// Replace the owner session only after the requested library has loaded successfully.
    /// Keeping the old session alive through the load preserves its lock and in-memory edits
    /// when opening a bad path or a library held by another process fails.
    fn switch_library(&mut self, path: PathBuf) -> Result<(PathBuf, usize), String> {
        if self.tasks.running() {
            return Err("cannot change libraries while a background task is running; wait for it to finish".into());
        }
        if preview_build_running(&self.session) {
            return Err("cannot change libraries while a preview build is running; run library.cancelPreviews, wait for library.previewProgress running=false, then retry".into());
        }
        if self.session.library.as_ref().is_some_and(|library| same_library_path(&library.dir, &path)) {
            return Ok((path, self.session.catalog.len()));
        }

        self.persist().map_err(|error| format!("current library must be saved before opening another library: {error}"))?;

        let mut fresh = Session::new().with_fs().with_system_clock();
        fresh.open_library(&path, false).map_err(|error| format!("could not open library: {error}"))?;
        configure_segmenter(&mut fresh, &self.segmenter_dir, &self.segmenter_mirrors_file);
        let notices = fresh.take_library_warnings();
        let photos = fresh.catalog.len();

        self.session.close_library().map_err(|error| format!("could not close current library: {error}"))?;
        self.renderer.cancel_all();
        let store = self.renderer.store();
        self.session = fresh;
        self.renderer = Renderer::new(store);
        self.snapshot_cache = crate::snapshot::CatalogSnapshotCache::default();
        self.notices = notices;
        self.last_error = None;
        Ok((path, photos))
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
        let path = library_path(params, "library.restore")?;
        if self.tasks.running() {
            return Err("cannot restore while a background task is running; wait for it to finish".into());
        }
        let path_metadata = std::fs::symlink_metadata(&path).map_err(|error| format!("library restore path: {error}"))?;
        if path_metadata.file_type().is_symlink() {
            return Err("library restore refuses symlink backup directory".into());
        }
        if !path_metadata.is_dir() {
            return Err("library restore expects a backup directory".into());
        }
        let backup = std::fs::canonicalize(&path).map_err(|error| format!("library restore path: {error}"))?;
        let destination = restore_destination(&backup)?;
        let metadata = std::fs::symlink_metadata(&backup).map_err(|error| format!("library restore read {}: {error}", backup.display()))?;
        if metadata.file_type().is_symlink() || !metadata.is_dir() {
            return Err("library restore expects a regular backup directory".into());
        }
        if let Err(error) = std::fs::create_dir(&destination) {
            return Err(format!("could not claim restore destination {}: {error}", destination.display()));
        }
        if let Err(error) = copy_tree_entries(&backup, &destination, 0) {
            let _ = std::fs::remove_dir_all(&destination);
            return Err(format!("could not stage restored library: {error}"));
        }
        if let Err(error) = self.switch_library(destination.clone()) {
            let _ = std::fs::remove_dir_all(&destination);
            return Err(error);
        }
        Ok(json!({
            "path": destination.to_string_lossy(),
            "libraryPath": destination.to_string_lossy(),
            "restoredPath": destination.to_string_lossy(),
            "sourceBackup": path.to_string_lossy(),
        }))
    }
}

fn library_path(params: &Value, command: &str) -> Result<PathBuf, String> {
    let path = params.get("path").and_then(Value::as_str).ok_or_else(|| format!("{command} requires path"))?;
    if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
        return Err(format!("invalid {command} path"));
    }
    Ok(PathBuf::from(path))
}

fn configure_segmenter(session: &mut Session, dir: &Option<PathBuf>, mirrors_file: &Option<PathBuf>) {
    session.segmenter.dir = dir.clone();
    session.segmenter.mirrors_file = mirrors_file.clone();
}

fn same_library_path(left: &Path, right: &Path) -> bool {
    match (std::fs::canonicalize(left), std::fs::canonicalize(right)) {
        (Ok(left), Ok(right)) => left == right,
        _ => left == right,
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

fn restore_destination(backup: &Path) -> Result<PathBuf, String> {
    let parent = backup.parent().ok_or_else(|| "backup path has no parent".to_string())?;
    let parent = std::fs::canonicalize(parent).map_err(|error| format!("restore destination parent: {error}"))?;
    let name = backup.file_name().ok_or_else(|| "backup path has no file name".to_string())?.to_string_lossy();
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or_default();
    let pid = std::process::id();
    for attempt in 0..32u32 {
        let destination = parent.join(format!("{name}.restored-{pid}-{stamp}-{attempt}"));
        if std::fs::symlink_metadata(&destination).is_err() {
            let destination = backup_destination(&destination, backup)?;
            if std::fs::symlink_metadata(&destination).is_err() {
                return Ok(destination);
            }
        }
    }
    Err("could not allocate a unique restore destination".into())
}

fn preview_build_running(session: &Session) -> bool {
    session.preview_build.as_ref().is_some_and(|build| !build.finished.load(std::sync::atomic::Ordering::Relaxed))
}

fn cancel_preview_build(session: &Session, timeout: Duration) -> Result<(), String> {
    let Some(build) = session.preview_build.as_ref().filter(|build| !build.finished.load(std::sync::atomic::Ordering::Relaxed)) else {
        return Ok(());
    };
    build.cancel.store(true, std::sync::atomic::Ordering::Relaxed);
    let deadline = Instant::now() + timeout;
    while !build.finished.load(std::sync::atomic::Ordering::Relaxed) && Instant::now() < deadline {
        std::thread::sleep(Duration::from_millis(5));
    }
    if build.finished.load(std::sync::atomic::Ordering::Relaxed) {
        Ok(())
    } else {
        Err("preview build is still stopping; retry shutdown after library.previewProgress reports running=false".into())
    }
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
        copy_tree_entries(source, destination, depth)?;
        Ok(())
    } else if metadata.is_file() {
        std::fs::copy(source, destination).map(|_| ()).map_err(|error| format!("backup copy {}: {error}", source.display()))
    } else {
        Err(format!("unsupported library entry {}", source.display()))
    }
}

fn copy_tree_entries(source: &Path, destination: &Path, depth: usize) -> Result<(), String> {
    for entry in std::fs::read_dir(source).map_err(|error| format!("backup list {}: {error}", source.display()))? {
        let entry = entry.map_err(|error| format!("backup entry {}: {error}", source.display()))?;
        if depth == 0 && matches!(entry.file_name().to_str(), Some("catalog.lock" | "catalog.lock.owner")) {
            continue;
        }
        copy_tree(&entry.path(), &destination.join(entry.file_name()), depth.saturating_add(1))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use std::fs;
    use std::sync::atomic::Ordering;

    use super::*;

    fn temp_library(label: &str) -> PathBuf {
        let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|duration| duration.as_nanos()).unwrap_or_default();
        std::env::temp_dir().join(format!("lightcraft-host-{label}-{}-{stamp}", std::process::id()))
    }

    fn seed_library(path: &Path) {
        let mut session = Session::new().with_system_clock();
        assert!(session.open_library(path, true).is_ok(), "demo library should open");
        assert!(session.close_library().is_ok(), "demo library should close");
    }

    #[test]
    fn failed_shutdown_keeps_owner_available_until_task_finishes() {
        let controller_result = Controller::new(HostOptions { demo: true, ..HostOptions::default() }, PreviewStore::default());
        assert!(controller_result.is_ok(), "demo controller should start");
        let Ok(mut controller) = controller_result else { return };
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

    #[test]
    fn failed_open_keeps_old_session_and_lock() {
        let old = temp_library("open-old");
        let held = temp_library("open-held");
        let controller_result = Controller::new(HostOptions { library_path: Some(old.clone()), ..HostOptions::default() }, PreviewStore::default());
        assert!(controller_result.is_ok());
        let Ok(mut controller) = controller_result else { return };
        let mut blocker = Session::new().with_system_clock();
        assert!(blocker.open_library(&held, false).is_ok());

        let result = controller.run("library.open", &json!({"path": held}));
        assert!(result.is_err());
        assert_eq!(controller.session.library.as_ref().map(|library| &library.dir), Some(&old));

        let mut second = Session::new().with_system_clock();
        assert!(second.open_library(&old, false).is_err(), "failed open must retain old lock");
        drop(blocker);
        assert!(controller.shutdown().is_ok());
        let _ = fs::remove_dir_all(old);
        let _ = fs::remove_dir_all(held);
    }

    #[test]
    fn open_refuses_running_background_task() {
        let controller_result = Controller::new(HostOptions { demo: true, ..HostOptions::default() }, PreviewStore::default());
        assert!(controller_result.is_ok());
        let Ok(mut controller) = controller_result else { return };
        let (started, release) = controller.tasks.hold_for_test();
        while !started.load(Ordering::Acquire) {
            std::thread::yield_now();
        }
        let result = controller.run("library.open", &json!({"path": temp_library("running")}));
        assert!(matches!(result, Err(ref error) if error.contains("background task")));
        release.store(true, Ordering::Release);
        while controller.tasks.running() {
            controller.poll();
            std::thread::yield_now();
        }
        assert!(controller.shutdown().is_ok());
    }

    #[test]
    fn filesystem_hooks_import_png_and_render_preview() {
        let library = temp_library("png-library");
        let source = temp_library("procedural-rgb-01").with_extension("png");
        let image = lightcraft_raster::Rgba8::from_fn(4, 3, |x, y| [(x * 40) as u8, (y * 60) as u8, 120, 255]);
        let encoded = lightcraft_codecs::encode_png(&lightcraft_codecs::EncodeImage::rgba8(&image), &lightcraft_codecs::EncodeMeta::default());
        assert!(encoded.is_ok(), "procedural PNG should encode: {:?}", encoded.as_ref().err());
        let Ok(encoded) = encoded else { return };
        assert!(fs::write(&source, encoded).is_ok(), "procedural PNG should be written");

        let controller_result =
            Controller::new(HostOptions { library_path: Some(library.clone()), ..HostOptions::default() }, PreviewStore::default());
        assert!(controller_result.is_ok(), "PNG host controller should start: {:?}", controller_result.as_ref().err());
        let Ok(mut controller) = controller_result else { return };
        let imported = controller.run("library.import", &json!({"paths": [source.to_string_lossy()]}));
        assert!(imported.is_ok(), "PNG import should start: {:?}", imported.err());
        for _ in 0..2_000 {
            controller.poll();
            if !controller.tasks.running() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!controller.tasks.running(), "PNG import worker should finish");
        assert!(controller.notices.iter().any(|notice| notice == "Import Photos finished"), "PNG import should finish: {:?}", controller.notices);

        let snapshot = controller.snapshot();
        assert!(snapshot.is_ok(), "PNG snapshot should succeed: {:?}", snapshot.as_ref().err());
        let Ok(snapshot) = snapshot else { return };
        let generation = snapshot.get("viewGeneration").and_then(Value::as_u64).expect("PNG snapshot must expose view generation");
        let slice = controller.slice(Some(generation), 0, 1);
        assert!(slice.is_ok(), "PNG slice should succeed: {:?}", slice.as_ref().err());
        let Ok(slice) = slice else { return };
        let Some(photo) = slice.get("photos").and_then(Value::as_array).and_then(|photos| photos.first()) else {
            assert!(slice.get("photos").and_then(Value::as_array).is_some_and(|photos| !photos.is_empty()), "PNG import should create a photo");
            return;
        };
        let photo_id = photo.get("id").and_then(Value::as_u64).expect("PNG import must expose photo identity");
        assert_eq!(photo.get("w").and_then(Value::as_u64), Some(4));
        assert_eq!(photo.get("h").and_then(Value::as_u64), Some(3));

        let (reply_tx, reply_rx) = std::sync::mpsc::channel();
        let request = PreviewRequest {
            photo_id,
            slot: "png-regression".into(),
            view_generation: generation,
            width: 64,
            height: 64,
            quality: crate::PreviewQuality::Draft,
            before: false,
            sequence: 1,
        };
        let _ = controller.preview(request, reply_tx);
        let mut preview = None;
        for _ in 0..2_000 {
            controller.poll();
            match reply_rx.try_recv() {
                Ok(result) => {
                    preview = Some(result);
                    break;
                }
                Err(std::sync::mpsc::TryRecvError::Empty) => std::thread::sleep(Duration::from_millis(1)),
                Err(std::sync::mpsc::TryRecvError::Disconnected) => break,
            }
        }
        assert!(preview.is_some(), "PNG preview should reply");
        let Some(preview) = preview else { return };
        assert!(preview.is_ok(), "PNG preview should render: {:?}", preview.as_ref().err());
        let Ok(preview) = preview else { return };
        assert_eq!(preview.photo_id, photo_id);
        assert_eq!(preview.encoding, "png");
        assert!(controller.renderer.store().get(&preview.handle).is_some(), "PNG preview bytes should be published");
        assert!(controller.shutdown().is_ok());
        let _ = fs::remove_dir_all(library);
        let _ = fs::remove_file(source);
    }

    #[test]
    fn equal_revision_catalog_switch_gets_fresh_generation() {
        let old = temp_library("same-old");
        let next = temp_library("same-next");
        let model_dir = temp_library("sam3-model");
        let mirrors_file = temp_library("sam3-mirrors");
        seed_library(&old);
        seed_library(&next);
        let controller_result = Controller::new(
            HostOptions {
                library_path: Some(old.clone()),
                sam3_dir: Some(model_dir.clone()),
                sam3_mirrors_file: Some(mirrors_file.clone()),
                ..HostOptions::default()
            },
            PreviewStore::default(),
        );
        assert!(controller_result.is_ok());
        let Ok(mut controller) = controller_result else { return };
        assert_eq!(controller.session.segmenter.dir.as_ref(), Some(&model_dir));
        assert_eq!(controller.session.segmenter.mirrors_file.as_ref(), Some(&mirrors_file));
        let before_result = controller.snapshot();
        assert!(before_result.is_ok());
        let Ok(before) = before_result else { return };
        let Some(before_generation) = before.get("viewGeneration").and_then(Value::as_u64) else { return };
        let before_revision = before.get("revision").and_then(Value::as_u64);
        let before_slice_result = controller.slice(Some(before_generation), 0, 1);
        assert!(before_slice_result.is_ok());
        let Ok(before_slice) = before_slice_result else { return };
        let before_id =
            before_slice.get("photos").and_then(Value::as_array).and_then(|photos| photos.first()).and_then(|photo| photo.get("id")).cloned();

        let opened_result = controller.run("library.open", &json!({"path": next}));
        assert!(opened_result.is_ok());
        let Ok(opened) = opened_result else { return };
        assert_eq!(controller.session.segmenter.dir.as_ref(), Some(&model_dir));
        assert_eq!(controller.session.segmenter.mirrors_file.as_ref(), Some(&mirrors_file));
        assert!(opened.get("photos").and_then(Value::as_u64).is_some_and(|count| count > 0));
        let after_result = controller.snapshot();
        assert!(after_result.is_ok());
        let Ok(after) = after_result else { return };
        assert_eq!(after.get("revision").and_then(Value::as_u64), before_revision);
        assert_ne!(after.get("viewGeneration").and_then(Value::as_u64), Some(before_generation));
        let Some(after_generation) = after.get("viewGeneration").and_then(Value::as_u64) else { return };
        let after_slice_result = controller.slice(Some(after_generation), 0, 1);
        assert!(after_slice_result.is_ok());
        let Ok(after_slice) = after_slice_result else { return };
        let after_id =
            after_slice.get("photos").and_then(Value::as_array).and_then(|photos| photos.first()).and_then(|photo| photo.get("id")).cloned();
        assert_eq!(after_id, before_id, "equal-revision catalogs should retain same photo ids");

        assert!(controller.shutdown().is_ok());
        let _ = fs::remove_dir_all(old);
        let _ = fs::remove_dir_all(next);
    }
}
