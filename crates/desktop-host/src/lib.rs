#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

mod controller;
mod preview;
mod render;
mod snapshot;
mod tasks;

use std::path::PathBuf;
use std::sync::mpsc::{Sender, SyncSender, channel, sync_channel};

use serde_json::Value;

pub use controller::Controller;
pub use preview::{PreviewBytes, PreviewStore};
pub use render::{PreviewDescriptor, PreviewQuality, PreviewRequest, Renderer};
pub use snapshot::{AlbumSummary, CommandInfo, ControlSpec, HostStatus, JobStatus, PhotoSummary, ViewSlice};

const IPC_CAPACITY: usize = 64;
const MAX_JS_SAFE_ID: u64 = 9_007_199_254_740_991;

#[derive(Clone, Debug, Default)]
pub struct HostOptions {
    pub library_path: Option<PathBuf>,
    pub demo: bool,
    pub demo_count: usize,
}

enum Request {
    Run { id: String, params: Value, reply: SyncSender<Result<Value, String>> },
    Snapshot { reply: SyncSender<Result<Value, String>> },
    ViewSlice { generation: Option<u64>, offset: usize, limit: usize, reply: SyncSender<Result<Value, String>> },
    Preview { request: PreviewRequest, reply: Sender<Result<PreviewDescriptor, String>> },
    Preferences { patch: Option<Value>, reply: SyncSender<Result<Value, String>> },
    Shutdown { reply: SyncSender<Result<(), String>> },
}

#[derive(Clone)]
pub struct DesktopHandle {
    tx: SyncSender<Request>,
    previews: PreviewStore,
}

impl DesktopHandle {
    pub fn spawn(options: HostOptions) -> Result<Self, String> {
        let (tx, rx) = sync_channel(IPC_CAPACITY);
        let (ready_tx, ready_rx) = sync_channel(1);
        let previews = PreviewStore::default();
        let owner_previews = previews.clone();
        std::thread::Builder::new()
            .name("lightcraft-session".into())
            .spawn(move || {
                let result = lightcraft_engine::guard::catch("desktop host startup", || Controller::new(options, owner_previews));
                match result {
                    Ok(Ok(mut controller)) => {
                        let _ = ready_tx.send(Ok(()));
                        if let Err(error) = lightcraft_engine::guard::catch("desktop host owner", || controller.serve(rx)) {
                            log::error!("desktop host owner stopped unexpectedly: {error}");
                            let _ = controller.shutdown();
                        }
                    }
                    Ok(Err(error)) | Err(error) => {
                        let _ = ready_tx.send(Err(error));
                    }
                }
            })
            .map_err(|error| format!("could not start session owner: {error}"))?;
        ready_rx.recv().map_err(|_| "session owner stopped during startup".to_string())??;
        Ok(Self { tx, previews })
    }

    pub fn run(&self, id: String, params: Value) -> Result<Value, String> {
        self.call(|reply| Request::Run { id, params, reply })
    }
    pub fn snapshot(&self) -> Result<Value, String> {
        self.call(|reply| Request::Snapshot { reply })
    }
    pub fn view_slice(&self, generation: Option<u64>, offset: usize, limit: usize) -> Result<Value, String> {
        self.call(|reply| Request::ViewSlice { generation, offset, limit, reply })
    }
    pub fn preview(&self, request: PreviewRequest) -> Result<PreviewDescriptor, String> {
        let (reply_tx, reply_rx) = channel();
        self.tx.send(Request::Preview { request, reply: reply_tx }).map_err(|_| "session owner is closed".to_string())?;
        reply_rx.recv().map_err(|_| "session owner stopped before replying".to_string())?
    }
    pub fn preferences(&self, patch: Option<Value>) -> Result<Value, String> {
        self.call(|reply| Request::Preferences { patch, reply })
    }
    pub fn shutdown(&self) -> Result<(), String> {
        self.call(|reply| Request::Shutdown { reply })
    }
    pub fn preview_store(&self) -> PreviewStore {
        self.previews.clone()
    }

    fn call<T>(&self, make: impl FnOnce(SyncSender<Result<T, String>>) -> Request) -> Result<T, String> {
        let (reply_tx, reply_rx) = sync_channel(1);
        self.tx.send(make(reply_tx)).map_err(|_| "session owner is closed".to_string())?;
        reply_rx.recv().map_err(|_| "session owner stopped before replying".to_string())?
    }
}

pub(crate) fn validate_id(id: u64) -> Result<(), String> {
    (id <= MAX_JS_SAFE_ID).then_some(()).ok_or_else(|| "photo id exceeds JavaScript safe integer range".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{Value, json};

    fn demo_host() -> Option<DesktopHandle> {
        let result = DesktopHandle::spawn(HostOptions { demo: true, demo_count: 3, ..HostOptions::default() });
        assert!(result.is_ok(), "demo session should start: {:?}", result.as_ref().err());
        result.ok()
    }

    #[test]
    fn owner_lifecycle_snapshot_and_generation_slice() {
        let Some(host) = demo_host() else { return };
        let snapshot = host.snapshot();
        assert!(snapshot.is_ok(), "snapshot should reply: {:?}", snapshot.err());
        let snapshot = match snapshot {
            Ok(value) => value,
            Err(_) => return,
        };
        let generation = snapshot.get("viewGeneration").and_then(Value::as_u64);
        let total = snapshot.get("total").and_then(Value::as_u64);
        assert_eq!(total, Some(3));
        let Some(generation) = generation else { return };
        let slice = host.view_slice(Some(generation), 0, 2);
        assert!(slice.is_ok(), "slice should reply: {:?}", slice.err());
        let slice = match slice {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(slice.get("generation").and_then(Value::as_u64), Some(generation));
        assert_eq!(slice.get("photos").and_then(Value::as_array).map(Vec::len), Some(2));
        assert!(host.shutdown().is_ok());
        assert!(host.snapshot().is_err(), "closed owner must reject calls");
    }

    #[test]
    fn owner_serializes_cloned_handles_and_actions() {
        let Some(host) = demo_host() else { return };
        let peer = host.clone();
        let worker = std::thread::spawn(move || peer.snapshot());
        let selected = host.run("library.select".into(), json!({"ids": [1], "active": 1}));
        assert!(selected.is_ok(), "selection action should dispatch: {:?}", selected.err());
        let joined = worker.join();
        assert!(joined.is_ok(), "owner snapshot worker should complete");
        let rated = host.run("photo.rate".into(), json!({"rating": 5, "ids": [1]}));
        assert!(rated.is_ok(), "rating action should dispatch: {:?}", rated.err());
        let began = host.run("develop.beginInteraction".into(), json!({"label": "Host test"}));
        assert!(began.is_ok(), "interaction should begin: {:?}", began.err());
        let edited = host.run("develop.set".into(), json!({"control": "light.exposure", "value": 0.25}));
        assert!(edited.is_ok(), "develop action should dispatch: {:?}", edited.err());
        let ended = host.run("develop.endInteraction".into(), json!({}));
        assert!(ended.is_ok(), "interaction should end: {:?}", ended.err());
        let snapshot = host.snapshot();
        assert!(snapshot.is_ok(), "snapshot should expose action state: {:?}", snapshot.err());
        let snapshot = match snapshot {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(snapshot.get("active").and_then(Value::as_u64), Some(1));
        assert!(snapshot.get("undo").and_then(Value::as_u64).is_some_and(|count| count > 0));
        assert!(host.shutdown().is_ok());
    }

    #[test]
    fn task_adapter_rejects_invalid_io_without_starting_jobs() {
        let Some(host) = demo_host() else { return };
        let export = host.run("app.export".into(), json!({"ids": [1], "dir": "bad\u{0000}path"}));
        assert!(export.is_err(), "NUL export paths must be rejected");
        let import = host.run("library.import".into(), json!({"paths": []}));
        assert!(import.is_err(), "empty import must be rejected");
        let cancel = host.run("task.cancel".into(), json!({"id": "missing-1"}));
        assert_eq!(cancel.ok().and_then(|value| value.get("cancelled").and_then(Value::as_u64)), Some(0));
        let snapshot = host.snapshot();
        assert!(snapshot.is_ok(), "snapshot should expose empty task state: {:?}", snapshot.err());
        let snapshot = match snapshot {
            Ok(value) => value,
            Err(_) => return,
        };
        assert_eq!(snapshot.get("status").and_then(|status| status.get("jobs")).and_then(Value::as_array).map(Vec::len), Some(0));
        assert!(host.shutdown().is_ok());
    }
}
