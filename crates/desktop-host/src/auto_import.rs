use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, TryRecvError, sync_channel};
use std::thread::JoinHandle;
use std::time::{Duration, Instant};

use serde_json::json;

use lightcraft_engine::Session;

use crate::tasks::Tasks;

const SCAN_INTERVAL: Duration = Duration::from_secs(3);
const STOP_TIMEOUT: Duration = Duration::from_secs(5);

struct ScanResult {
    generation: u64,
    folder: PathBuf,
    listing: Result<Vec<(String, u64)>, String>,
}

struct ScanWorker {
    cancel: Arc<AtomicBool>,
    result: Receiver<ScanResult>,
    join: JoinHandle<()>,
}

/// Bounded auto-import coordinator. Filesystem enumeration happens off the session owner thread;
/// engine scanning and import task start stay serialized on that owner.
pub(crate) struct AutoImport {
    stopped: bool,
    generation: u64,
    folder: Option<PathBuf>,
    next_scan: Instant,
    worker: Option<ScanWorker>,
    stopping: Option<ScanWorker>,
    stopping_since: Option<Instant>,
}

impl AutoImport {
    pub(crate) fn new() -> Self {
        Self {
            stopped: false,
            generation: 0,
            folder: None,
            next_scan: Instant::now() + SCAN_INTERVAL,
            worker: None,
            stopping: None,
            stopping_since: None,
        }
    }

    pub(crate) fn poll(&mut self, session: &mut Session, tasks: &mut Tasks) -> Result<(), String> {
        if self.stopped {
            self.drain_stopping()?;
            return Ok(());
        }
        self.drain_stopping()?;
        self.sync_config(session);

        if let Some(result) = self.take_result() {
            self.next_scan = Instant::now() + SCAN_INTERVAL;
            if generation_matches(self.generation, self.folder.as_ref(), result.generation, Some(&result.folder)) && !tasks.running_kind("import") {
                let listing = result.listing.map_err(|error| format!("auto import scan: {error}"))?;
                self.consume_listing(session, tasks, listing)?;
            }
        }

        if self.worker.is_none()
            && self.stopping.is_none()
            && self.folder.is_some()
            && !tasks.running_kind("import")
            && Instant::now() >= self.next_scan
        {
            self.start_worker()?;
            self.next_scan = Instant::now() + SCAN_INTERVAL;
        }
        Ok(())
    }

    pub(crate) fn stop(&mut self) -> Result<(), String> {
        self.stopped = true;
        self.cancel_worker();
        self.wait_for_stopping()
    }

    pub(crate) fn reset(&mut self) -> Result<(), String> {
        self.cancel_worker();
        self.wait_for_stopping()?;
        self.stopped = false;
        self.generation = self.generation.saturating_add(1);
        self.folder = None;
        self.next_scan = Instant::now() + SCAN_INTERVAL;
        Ok(())
    }

    fn sync_config(&mut self, session: &Session) {
        let configured = session.import_defaults.auto_folder.as_deref().map(PathBuf::from);
        if configured == self.folder {
            return;
        }
        self.cancel_worker();
        self.generation = self.generation.saturating_add(1);
        self.folder = configured;
        self.next_scan = Instant::now();
    }

    fn start_worker(&mut self) -> Result<(), String> {
        let Some(folder) = self.folder.clone() else { return Ok(()) };
        let generation = self.generation;
        let (tx, result) = sync_channel(1);
        let cancel = Arc::new(AtomicBool::new(false));
        let worker_cancel = cancel.clone();
        let folder_for_worker = folder.clone();
        let folder_string = folder.to_string_lossy().to_string();
        let join = std::thread::Builder::new()
            .name("lightcraft-auto-import-list".into())
            .spawn(move || {
                if worker_cancel.load(Ordering::Acquire) {
                    return;
                }
                let listing = lightcraft_engine::cmd::library::list_auto_import_folder(&folder_string);
                if !worker_cancel.load(Ordering::Acquire) {
                    let _ = tx.send(ScanResult { generation, folder: folder_for_worker, listing });
                }
            })
            .map_err(|error| format!("could not start auto import scan: {error}"))?;
        self.worker = Some(ScanWorker { cancel, result, join });
        Ok(())
    }

    fn take_result(&mut self) -> Option<ScanResult> {
        let worker = self.worker.take()?;
        match worker.result.try_recv() {
            Ok(result) => {
                self.release_worker(worker);
                Some(result)
            }
            Err(TryRecvError::Empty) => {
                self.worker = Some(worker);
                None
            }
            Err(TryRecvError::Disconnected) => {
                self.release_worker(worker);
                Some(ScanResult {
                    generation: self.generation,
                    folder: self.folder.clone().unwrap_or_default(),
                    listing: Err("scan worker stopped".into()),
                })
            }
        }
    }

    fn consume_listing(&mut self, session: &mut Session, tasks: &mut Tasks, listing: Vec<(String, u64)>) -> Result<(), String> {
        if tasks.running_kind("import") {
            return Ok(());
        }
        let result = session.execute("library.autoImportScan", &json!({"listing": listing, "start": false})).map_err(|error| error.to_string())?;
        let Some(params) = result.get("import").cloned() else { return Ok(()) };
        if !params.is_object() || tasks.running_kind("import") {
            return Ok(());
        }
        tasks.start_auto_import(session, &params).map(|_| ())
    }

    fn cancel_worker(&mut self) {
        let Some(worker) = self.worker.take() else { return };
        worker.cancel.store(true, Ordering::Release);
        self.queue_stopping(worker);
    }

    fn queue_stopping(&mut self, worker: ScanWorker) {
        if worker.join.is_finished() {
            let _ = worker.join.join();
            return;
        }
        self.stopping_since.get_or_insert_with(Instant::now);
        self.stopping = Some(worker);
    }

    fn release_worker(&mut self, worker: ScanWorker) {
        if worker.join.is_finished() {
            let _ = worker.join.join();
        } else {
            self.queue_stopping(worker);
        }
    }

    fn drain_stopping(&mut self) -> Result<(), String> {
        let Some(worker) = self.stopping.as_ref() else {
            self.stopping_since = None;
            return Ok(());
        };
        if worker.join.is_finished() {
            let worker = self.stopping.take();
            if let Some(worker) = worker {
                let _ = worker.join.join();
            }
            self.stopping_since = None;
            return Ok(());
        }
        if self.stopping_since.is_some_and(|started| started.elapsed() >= STOP_TIMEOUT) {
            return Err("auto import scan worker did not stop within 5s; disable auto import or restart library before retrying".into());
        }
        Ok(())
    }

    fn wait_for_stopping(&mut self) -> Result<(), String> {
        let deadline = Instant::now() + STOP_TIMEOUT;
        loop {
            self.drain_stopping()?;
            if self.stopping.is_none() {
                return Ok(());
            }
            if Instant::now() >= deadline {
                return Err("auto import scan worker did not stop within 5s; disable auto import or restart library before retrying".into());
            }
            std::thread::sleep(Duration::from_millis(10));
        }
    }
}

fn generation_matches(current: u64, current_folder: Option<&PathBuf>, result: u64, result_folder: Option<&PathBuf>) -> bool {
    current == result && current_folder == result_folder
}

impl Drop for AutoImport {
    fn drop(&mut self) {
        self.cancel_worker();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;

    use serde_json::{Value, json};

    fn temp_folder(label: &str) -> PathBuf {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_nanos()).unwrap_or_default();
        std::env::temp_dir().join(format!("lightcraft-auto-{label}-{}-{stamp}", std::process::id()))
    }

    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn arrival_listing_reads_real_file_and_size() {
        let folder = temp_folder("arrival");
        assert!(fs::create_dir_all(&folder).is_ok());
        let path = folder.join("arrived.png");
        assert!(fs::write(&path, [1_u8, 2, 3, 4]).is_ok());
        let listing = lightcraft_engine::cmd::library::list_auto_import_folder(&folder.to_string_lossy());
        assert!(listing.is_ok(), "listing should succeed: {listing:?}");
        let listing = match listing {
            Ok(value) => value,
            Err(_) => return,
        };
        let expected = path.to_string_lossy();
        assert!(listing.iter().any(|(candidate, size)| candidate == expected.as_ref() && *size == 4));
        let _ = fs::remove_dir_all(folder);
    }

    #[test]
    fn disabled_or_switched_folder_rejects_stale_result() {
        let old = temp_folder("old");
        let new = temp_folder("new");
        assert!(!generation_matches(2, None, 1, Some(&old)));
        assert!(!generation_matches(2, Some(&new), 2, Some(&old)));
    }

    #[test]
    fn unchanged_folder_generation_accepts_arrival_result() {
        let folder = temp_folder("stable");
        assert!(generation_matches(7, Some(&folder), 7, Some(&folder)));
    }

    #[test]
    fn stable_arrival_task_commits_without_replacing_user_selection() {
        let folder = temp_folder("task");
        assert!(fs::create_dir_all(&folder).is_ok());
        let seed = temp_folder("seed").with_extension("png");
        let arrival = folder.join("arrival.png");
        assert!(write_test_png(&seed, 11));
        assert!(write_test_png(&arrival, 19));

        let mut session = Session::new().with_fs().with_system_clock();
        let seeded = session.execute("library.import", &json!({"paths": [seed.to_string_lossy()]}));
        assert!(seeded.is_ok(), "seed import should succeed: {seeded:?}");
        assert!(!session.selection.ids.is_empty(), "seed import must select a photo");
        assert!(session.execute("library.autoImport", &json!({"folder": folder.to_string_lossy()})).is_ok());
        let listing = lightcraft_engine::cmd::library::list_auto_import_folder(&folder.to_string_lossy());
        assert!(listing.is_ok(), "arrival listing should succeed: {listing:?}");
        let listing = match listing {
            Ok(value) => value,
            Err(_) => return,
        };
        let first = session.execute("library.autoImportScan", &json!({"listing": listing, "start": false}));
        assert!(first.is_ok(), "first stability scan should succeed: {first:?}");
        let listing = lightcraft_engine::cmd::library::list_auto_import_folder(&folder.to_string_lossy());
        assert!(listing.is_ok(), "second arrival listing should succeed: {listing:?}");
        let listing = match listing {
            Ok(value) => value,
            Err(_) => return,
        };
        let second = session.execute("library.autoImportScan", &json!({"listing": listing, "start": false}));
        assert!(second.is_ok(), "second stability scan should succeed: {second:?}");
        let second = match second {
            Ok(value) => value,
            Err(_) => return,
        };
        let params = second.get("import").cloned();
        assert!(params.as_ref().is_some_and(Value::is_object), "stable arrival should return import params: {second:?}");
        let Some(params) = params else { return };

        let mut tasks = Tasks::new();
        assert!(tasks.start_auto_import(&mut session, &params).is_ok());
        // User selection can change while decoding runs; completion must preserve
        // current selection rather than restoring a stale start-of-task snapshot.
        assert!(session.execute("library.select", &json!({"ids": [], "mode": "replace"})).is_ok());
        let selected = session.selection.clone();
        for _ in 0..2_000 {
            tasks.poll(&mut session);
            if !tasks.running() {
                break;
            }
            std::thread::sleep(Duration::from_millis(1));
        }
        assert!(!tasks.running(), "auto import should finish");
        assert_eq!(session.catalog.len(), 2);
        assert_eq!(session.selection, selected);
        let _ = fs::remove_file(seed);
        let _ = fs::remove_dir_all(folder);
    }

    fn write_test_png(path: &std::path::Path, value: u8) -> bool {
        let image = lightcraft_raster::Rgba8::from_fn(4, 3, |x, y| [value.saturating_add(x as u8), value.saturating_add(y as u8), 120, 255]);
        let encoded = lightcraft_codecs::encode_png(&lightcraft_codecs::EncodeImage::rgba8(&image), &lightcraft_codecs::EncodeMeta::default());
        match encoded {
            Ok(bytes) => fs::write(path, bytes).is_ok(),
            Err(_) => false,
        }
    }
}
