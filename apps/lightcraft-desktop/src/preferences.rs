//! Crash-safe app preferences for the Tauri preview host.
//!
//! Preferences are JSON objects rather than a typed projection so fields introduced by another
//! LightCraft host survive a round trip through this app.

#![forbid(unsafe_code)]

use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::{Arc, Mutex};
use std::time::{SystemTime, UNIX_EPOCH};

use serde_json::{Map, Value};

const MAX_BYTES: usize = 4 * 1024 * 1024;

#[derive(Clone)]
pub struct Preferences {
    inner: Arc<Mutex<State>>,
}

struct State {
    path: PathBuf,
    value: Map<String, Value>,
    blocked: bool,
    warning: Option<String>,
}

impl Preferences {
    pub fn load(path: PathBuf) -> Self {
        let (value, blocked, warning) = match fs::read(&path) {
            Ok(bytes) if bytes.len() > MAX_BYTES => (Map::new(), true, Some(format!("preferences exceed {MAX_BYTES} bytes"))),
            Ok(bytes) => match serde_json::from_slice::<Value>(&bytes) {
                Ok(Value::Object(object)) => (object, false, None),
                Ok(_) => quarantine(&path, "preferences must be a JSON object"),
                Err(error) => quarantine(&path, &format!("preferences are invalid: {error}")),
            },
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => (Map::new(), false, None),
            Err(error) => (Map::new(), true, Some(format!("preferences could not be read: {error}"))),
        };
        Self { inner: Arc::new(Mutex::new(State { path, value, blocked, warning })) }
    }

    pub fn get(&self) -> Result<Value, String> {
        let state = self.inner.lock().map_err(|_| "preferences lock poisoned".to_string())?;
        Ok(Value::Object(state.value.clone()))
    }

    pub fn warning(&self) -> Option<String> {
        self.inner.lock().ok().and_then(|state| state.warning.clone())
    }

    /// Merge a bounded object patch, preserving unknown baseline keys, then persist atomically.
    pub fn patch(&self, patch: Option<Value>) -> Result<Value, String> {
        let Some(patch) = patch else { return self.get() };
        let Value::Object(values) = patch else { return Err("preferences patch must be an object".into()) };
        let mut state = self.inner.lock().map_err(|_| "preferences lock poisoned".to_string())?;
        if state.blocked {
            return Err(state.warning.clone().unwrap_or_else(|| "preferences file is blocked for this session".into()));
        }
        let mut next = state.value.clone();
        merge_object(&mut next, values);
        let bytes = serde_json::to_vec_pretty(&Value::Object(next.clone())).map_err(|error| format!("serializing preferences: {error}"))?;
        if bytes.len() > MAX_BYTES {
            return Err("preferences patch exceeds size limit".into());
        }
        write_atomic(&state.path, &bytes).map_err(|error| format!("saving preferences: {error}"))?;
        state.value = next;
        Ok(Value::Object(state.value.clone()))
    }
}

fn merge_object(target: &mut Map<String, Value>, patch: Map<String, Value>) {
    for (key, value) in patch {
        match value {
            Value::Object(incoming) => {
                if let Some(Value::Object(existing)) = target.get_mut(&key) {
                    merge_object(existing, incoming);
                } else {
                    target.insert(key, Value::Object(incoming));
                }
            }
            other => {
                target.insert(key, other);
            }
        }
    }
}

fn quarantine(path: &Path, reason: &str) -> (Map<String, Value>, bool, Option<String>) {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH).map(|duration| duration.as_secs()).unwrap_or(0);
    let quarantine = path.with_file_name(format!(
        "{}.corrupt-{stamp}-{}",
        path.file_name().and_then(|name| name.to_str()).unwrap_or("ui.json"),
        std::process::id()
    ));
    let (blocked, message) = match fs::rename(path, &quarantine) {
        Ok(()) => (false, format!("{reason}; damaged file quarantined at {}", quarantine.display())),
        Err(error) => (true, format!("{reason}; damaged file could not be quarantined: {error}")),
    };
    (Map::new(), blocked, Some(message))
}

fn write_atomic(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    let parent = path.parent().ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidInput, "preferences path has no parent"))?;
    fs::create_dir_all(parent)?;
    let name = path.file_name().and_then(|name| name.to_str()).unwrap_or("ui.json");
    let tmp = parent.join(format!(".{name}.tmp-{}", std::process::id()));
    let result = (|| {
        let mut file = OpenOptions::new().create(true).truncate(true).write(true).open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        fs::rename(&tmp, path)?;
        if let Ok(directory) = File::open(parent) {
            let _ = directory.sync_all();
        }
        Ok(())
    })();
    if result.is_err() {
        let _ = fs::remove_file(&tmp);
    }
    result
}
