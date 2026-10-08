#[path = "menu.rs"]
mod menu;
#[path = "preferences.rs"]
mod preferences;
#[path = "services.rs"]
mod services;

use std::ffi::OsStr;
use std::path::PathBuf;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::{fs, path::Path};

use lightcraft_desktop_host::{DesktopHandle, HostOptions, PreviewRequest};
use serde_json::{Value, json};
use tauri::WebviewWindow;
use tauri::http::{Request, Response};
#[cfg(feature = "qa-native")]
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, Wry};

use self::preferences::Preferences;

struct AppState {
    host: Option<DesktopHandle>,
    preferences: Preferences,
    startup_error: Option<String>,
    startup_warnings: Vec<String>,
    /// One-shot close override set only by an explicit user-facing quit action.
    closing: Arc<AtomicBool>,
}

async fn blocking<T, F>(work: F) -> Result<T, String>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    tauri::async_runtime::spawn_blocking(work).await.map_err(|error| format!("native task failed: {error}"))?
}

// tauri-macros 2.7.1 emits a sibling `const _: () = if false { ... unreachable!() ... }`
// to typecheck async Result signatures. That branch cannot run. Scope this allowance
// to these generated bindings; runtime command bodies introduce no panic paths.
#[allow(clippy::unreachable)]
mod tauri_commands {
    use super::*;

    #[tauri::command]
    pub(super) async fn lc_run(state: State<'_, AppState>, id: String, params: Value) -> Result<Value, String> {
        let host = state.host.clone();
        let error = state.startup_error.clone();
        let preferences = state.preferences.clone();
        let restore = id == "library.restore";
        blocking(move || {
            let result =
                host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(|host| host.run(id, params))?;
            if restore { persist_library_path(&preferences, host.as_ref(), result, None) } else { Ok(result) }
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_snapshot(state: State<'_, AppState>) -> Result<Value, String> {
        let host = state.host.clone();
        let error = state.startup_error.clone();
        let warnings = state.startup_warnings.clone();
        blocking(move || {
            let snapshot =
                host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(DesktopHandle::snapshot)?;
            Ok(merge_startup_warnings(snapshot, &warnings))
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_view_slice(state: State<'_, AppState>, generation: Option<u64>, offset: usize, limit: usize) -> Result<Value, String> {
        let host = state.host.clone();
        let error = state.startup_error.clone();
        blocking(move || {
            host.as_ref()
                .ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into()))
                .and_then(|host| host.view_slice(generation, offset, limit))
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_preview(
        state: State<'_, AppState>,
        request: PreviewRequest,
    ) -> Result<lightcraft_desktop_host::PreviewDescriptor, String> {
        let host = state.host.clone();
        let error = state.startup_error.clone();
        blocking(move || {
            host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(|host| host.preview(request))
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_preview_ack(state: State<'_, AppState>, handle: String) -> Result<bool, String> {
        let host = state.host.clone();
        let error = state.startup_error.clone();
        blocking(move || {
            let host = host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
            Ok(host.preview_store().acknowledge(&handle))
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_preferences(state: State<'_, AppState>, patch: Option<Value>, window: WebviewWindow<Wry>) -> Result<Value, String> {
        let preferences = state.preferences.clone();
        let host = state.host.clone();
        let startup_error = state.startup_error.clone();
        let patch = preference_patch_for_window(&window, patch);
        blocking(move || {
            let value = preferences.patch(patch)?;
            if let Some(host) = host {
                host.preferences(Some(value.clone()))
            } else if let Some(error) = startup_error {
                Err(error)
            } else {
                Ok(value.clone())
            }
        })
        .await
    }

    #[tauri::command]
    pub(super) async fn lc_native(
        action: String,
        params: Value,
        app: AppHandle<Wry>,
        window: WebviewWindow<Wry>,
        state: State<'_, AppState>,
    ) -> Result<Value, String> {
        let caller_label = window.label().to_string();
        if action == "openLibrary" {
            let host = state.host.clone();
            let startup_error = state.startup_error.clone();
            let preferences = state.preferences.clone();
            return blocking(move || open_library_action(host, startup_error, preferences, params)).await;
        }
        if matches!(action.as_str(), "backupLibrary" | "restoreLibrary") {
            let host = state.host.clone();
            let startup_error = state.startup_error.clone();
            let preferences = state.preferences.clone();
            return blocking(move || {
                let path = if let Some(path) = params.get("path").and_then(Value::as_str) {
                    path.to_string()
                } else {
                    let picker = if action == "backupLibrary" {
                        services::run("saveFile", &json!({"suggestedName": "LightCraft Library Backup.lclibrary", "extensions": ["lclibrary"]}))
                    } else {
                        services::run("chooseFolder", &json!({}))
                    }?;
                    picker.get("path").and_then(Value::as_str).map(str::to_string).ok_or_else(|| "no library backup selected".to_string())?
                };
                if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
                    return Err("invalid library backup path".into());
                }
                let host = host.as_ref().ok_or_else(|| startup_error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
                let result = host.run(if action == "backupLibrary" { "library.backup" } else { "library.restore" }.into(), json!({"path": path}))?;
                if action == "restoreLibrary" { persist_library_path(&preferences, Some(host), result, None) } else { Ok(result) }
            })
            .await;
        }
        if action == "preferences.patch" {
            let preferences = state.preferences.clone();
            let host = state.host.clone();
            let patch = preference_patch_for_window(&window, Some(params));
            return blocking(move || {
                let value = preferences.patch(patch)?;
                if let Some(host) = host { host.preferences(Some(value.clone())) } else { Ok(value) }
            })
            .await;
        }
        if action == "saveBeforeClose" {
            let host = state.host.clone();
            let startup_error = state.startup_error.clone();
            let label =
                if caller_label == "main" { params.get("label").and_then(Value::as_str).unwrap_or("main").to_string() } else { caller_label.clone() };
            let owns_session = caller_label == "main";
            let app_handle = app.clone();
            return blocking(move || {
                if owns_session {
                    let host = host.as_ref().ok_or_else(|| startup_error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
                    let snapshot = host.snapshot()?;
                    if let Some(reason) = active_close_reason(&snapshot) {
                        return Err(reason);
                    }
                    host.persist()?;
                }
                let window = app_handle.get_webview_window(&label).ok_or_else(|| format!("window {label} is unavailable"))?;
                window.close().map_err(|error| format!("closing window: {error}"))?;
                Ok(json!({"closed": true}))
            })
            .await;
        }
        if matches!(action.as_str(), "secondWindow" | "fullscreen" | "toggleFullscreen" | "confirmClose" | "closeWindow") {
            let action = if action == "fullscreen" { "toggleFullscreen" } else { action.as_str() };
            return native_window_action(&app, action, &params, Some(&caller_label));
        }
        blocking(move || services::run(&action, &params)).await
    }
}

fn merge_startup_warnings(mut snapshot: Value, warnings: &[String]) -> Value {
    if !snapshot.get("status").is_some_and(Value::is_object)
        && let Some(object) = snapshot.as_object_mut()
    {
        object.insert("status".into(), json!({}));
    }
    let Some(status) = snapshot.get_mut("status").and_then(Value::as_object_mut) else { return snapshot };
    let notices = status.entry("notices").or_insert_with(|| json!([]));
    if !notices.is_array() {
        *notices = json!([]);
    }
    if let Some(notices) = notices.as_array_mut() {
        for warning in warnings.iter().take(16) {
            if !notices.iter().any(|notice| notice.as_str() == Some(warning)) {
                notices.push(Value::String(warning.clone()));
            }
        }
        if notices.len() > 64 {
            let remove = notices.len() - 64;
            notices.drain(..remove);
        }
    }
    snapshot
}

fn preference_patch_for_window(window: &WebviewWindow<Wry>, patch: Option<Value>) -> Option<Value> {
    if window.label() == "main" {
        return patch;
    }
    let Some(Value::Object(mut values)) = patch else { return patch };
    values.remove("ui");
    values.remove("layout");
    Some(Value::Object(values))
}

fn open_library_action(host: Option<DesktopHandle>, startup_error: Option<String>, preferences: Preferences, params: Value) -> Result<Value, String> {
    let picked = services::run("openLibrary", &params)?;
    let Some(path) = picked.get("path").and_then(Value::as_str) else {
        return Ok(json!({"cancelled": true}));
    };
    if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
        return Err("invalid library path".into());
    }
    let host = host.ok_or_else(|| startup_error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
    let result = host.run("library.open".into(), json!({"path": path}))?;
    persist_library_path(&preferences, Some(&host), result, Some(path))
}

fn persist_library_path(preferences: &Preferences, host: Option<&DesktopHandle>, result: Value, fallback: Option<&str>) -> Result<Value, String> {
    let path = result
        .get("libraryPath")
        .and_then(Value::as_str)
        .or_else(|| result.get("restoredPath").and_then(Value::as_str))
        .or_else(|| result.get("path").and_then(Value::as_str))
        .or(fallback)
        .ok_or_else(|| "library operation returned no library path".to_string())?;
    if path.is_empty() || path.len() > 8_192 || path.contains('\0') {
        return Err("library operation returned an invalid library path".into());
    }
    let value = preferences.patch(Some(json!({"libraryPath": path})))?;
    if let Some(host) = host {
        host.preferences(Some(value))?;
    }
    Ok(result)
}

fn native_window_action(app: &AppHandle<Wry>, action: &str, params: &Value, caller_label: Option<&str>) -> Result<Value, String> {
    let label = match caller_label {
        Some(caller) if caller != "main" => caller,
        _ => params.get("label").and_then(Value::as_str).or(caller_label).unwrap_or("main"),
    };
    let window = app.get_webview_window(label).ok_or_else(|| format!("window {label} is unavailable"))?;
    match action {
        "secondWindow" => {
            if app.get_webview_window("second-main").is_none() {
                let mut route = String::from("index.html?window=second");
                if let Some(view) = params
                    .get("view")
                    .and_then(Value::as_str)
                    .filter(|view| matches!(*view, "photoGrid" | "squareGrid" | "detail" | "compare" | "survey" | "people" | "reference"))
                {
                    route.push_str("&view=");
                    route.push_str(view);
                }
                if let Some(active) = params.get("active").and_then(Value::as_u64) {
                    route.push_str("&active=");
                    route.push_str(&active.to_string());
                }
                WebviewWindowBuilder::new(app, "second-main", WebviewUrl::App(route.into()))
                    .title("LightCraft")
                    .inner_size(1200.0, 800.0)
                    .min_inner_size(800.0, 560.0)
                    .build()
                    .map_err(|error| format!("opening second window: {error}"))?;
            }
            Ok(json!({"label": "second-main"}))
        }
        "toggleFullscreen" => {
            let fullscreen = params.get("enabled").and_then(Value::as_bool).unwrap_or(!window.is_fullscreen().map_err(|error| error.to_string())?);
            window.set_fullscreen(fullscreen).map_err(|error| error.to_string())?;
            Ok(json!({"fullscreen": fullscreen}))
        }
        "confirmClose" | "closeWindow" => {
            let force = params.get("force").and_then(Value::as_bool).unwrap_or(false);
            if force
                && label == "main"
                && let Some(state) = app.try_state::<AppState>()
            {
                state.closing.store(true, Ordering::SeqCst);
            }
            if let Err(error) = window.close() {
                if force
                    && label == "main"
                    && let Some(state) = app.try_state::<AppState>()
                {
                    state.closing.store(false, Ordering::SeqCst);
                }
                return Err(format!("closing window: {error}"));
            }
            Ok(Value::Null)
        }
        _ => Err(format!("unknown native window action {action}")),
    }
}

fn active_close_reason(snapshot: &Value) -> Option<String> {
    let status = snapshot.get("status")?;
    let active = ["importing", "exporting", "previewBuild"].into_iter().any(|key| status.get(key).and_then(Value::as_bool).unwrap_or(false))
        || status.get("jobs").and_then(Value::as_array).is_some_and(|jobs| !jobs.is_empty());
    active.then(|| "cannot close while a native task is active; wait for cancellation to settle".to_string())
}

#[cfg(feature = "qa-native")]
fn control_args(args: &str) -> Result<Value, String> {
    if args.trim().is_empty() {
        return Ok(Value::Null);
    }
    let parsed = serde_json::from_str::<Value>(args).unwrap_or_else(|_| Value::String(args.to_string()));
    let Some(serialized) = parsed.get("args").filter(|_| parsed.get("name").is_some()).and_then(Value::as_str) else {
        return Ok(parsed);
    };
    if serialized.trim().is_empty() {
        Ok(Value::Null)
    } else {
        serde_json::from_str(serialized).or_else(|_| Ok(Value::String(serialized.to_string())))
    }
}

#[cfg(feature = "qa-native")]
fn control_host(app: &AppHandle<Wry>) -> Result<DesktopHandle, String> {
    let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
    state.host.clone().ok_or_else(|| state.startup_error.clone().unwrap_or_else(|| "desktop host is unavailable".into()))
}

#[cfg(feature = "qa-native")]
fn control_json(value: Value) -> Result<String, String> {
    serde_json::to_string(&value).map_err(|error| format!("serializing control result: {error}"))
}

#[cfg(feature = "qa-native")]
fn qa_viewport(app: &AppHandle<Wry>, args: &Value) -> Result<Value, String> {
    let object = args.as_object().ok_or_else(|| "lc_qa_viewport expects an object".to_string())?;
    let width = object.get("width").and_then(Value::as_u64).ok_or_else(|| "lc_qa_viewport requires width".to_string())?;
    let height = object.get("height").and_then(Value::as_u64).ok_or_else(|| "lc_qa_viewport requires height".to_string())?;
    if !matches!((width, height), (1280, 800) | (1600, 1000)) {
        return Err("lc_qa_viewport allows only 1280x800 or 1600x1000".into());
    }
    let window = app.get_webview_window("main").ok_or_else(|| "main window is unavailable".to_string())?;
    let geometry = |window: &WebviewWindow<Wry>| {
        let size = |result: Result<tauri::PhysicalSize<u32>, _>| result.ok().map(|size| json!({"width": size.width, "height": size.height}));
        let position = |result: Result<tauri::PhysicalPosition<i32>, _>| result.ok().map(|position| json!({"x": position.x, "y": position.y}));
        json!({
            "innerSize": size(window.inner_size()),
            "outerSize": size(window.outer_size()),
            "innerPosition": position(window.inner_position()),
            "outerPosition": position(window.outer_position()),
            "isDecorated": window.is_decorated().ok(),
            "isResizable": window.is_resizable().ok(),
            "isMaximized": window.is_maximized().ok(),
            "isFullscreen": window.is_fullscreen().ok(),
        })
    };
    let before = geometry(&window);
    let config = app.config().app.windows.iter().find(|candidate| candidate.label == "main").map(|candidate| {
        json!({
            "label": candidate.label,
            "width": candidate.width,
            "height": candidate.height,
            "minWidth": candidate.min_width,
            "minHeight": candidate.min_height,
            "decorations": candidate.decorations,
            "shadow": candidate.shadow,
            "resizable": candidate.resizable,
        })
    });
    window
        .set_size(tauri::Size::Logical(tauri::LogicalSize::new(width as f64, height as f64)))
        .map_err(|error| format!("setting QA viewport: {error}"))?;
    let observed = window.inner_size().map_err(|error| format!("reading QA viewport: {error}"))?;
    let after = geometry(&window);
    Ok(json!({
        "requested": {"width": width, "height": height},
        "observedInnerSize": {"width": observed.width, "height": observed.height},
        "config": config,
        "before": before,
        "after": after,
    }))
}

#[cfg(feature = "qa-native")]
fn control_dispatch(app: &AppHandle<Wry>, name: &str, raw: &str) -> Result<String, String> {
    let args = control_args(raw)?;
    let result = match name {
        "lc_run" => {
            let object = args.as_object().ok_or_else(|| "lc_run expects an object".to_string())?;
            let id = object.get("id").and_then(Value::as_str).ok_or_else(|| "lc_run requires id".to_string())?;
            let host = control_host(app)?;
            let result = host.run(id.to_string(), object.get("params").cloned().unwrap_or_else(|| json!({})))?;
            if id == "library.restore" {
                let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                persist_library_path(&state.preferences, Some(&host), result, None)?
            } else {
                result
            }
        }
        "lc_snapshot" => {
            let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
            merge_startup_warnings(control_host(app)?.snapshot()?, &state.startup_warnings)
        }
        "lc_view_slice" => {
            let object = args.as_object().ok_or_else(|| "lc_view_slice expects an object".to_string())?;
            let generation = object.get("generation").and_then(Value::as_u64);
            let offset = object.get("offset").and_then(Value::as_u64).unwrap_or(0) as usize;
            let limit = object.get("limit").and_then(Value::as_u64).unwrap_or(512).min(512) as usize;
            control_host(app)?.view_slice(generation, offset, limit)?
        }
        "lc_preview" => {
            let request = args.get("request").cloned().unwrap_or(args);
            control_host(app)?
                .preview(serde_json::from_value(request).map_err(|error| format!("lc_preview args are invalid: {error}"))?)
                .and_then(|value| serde_json::to_value(value).map_err(|error| error.to_string()))?
        }
        "lc_preview_ack" => {
            let handle =
                args.get("handle").and_then(Value::as_str).or_else(|| args.as_str()).ok_or_else(|| "lc_preview_ack requires handle".to_string())?;
            json!(control_host(app)?.preview_store().acknowledge(handle))
        }
        "lc_preferences" => {
            let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
            let patch = args.get("patch").cloned().or_else(|| (!args.is_null()).then_some(args));
            let value = state.preferences.patch(patch)?;
            if let Some(host) = state.host.clone() {
                host.preferences(Some(value.clone()))?;
            }
            value
        }
        "lc_qa_viewport" => qa_viewport(app, &args)?,
        "lc_native" => {
            let object = args.as_object().ok_or_else(|| "lc_native expects an object".to_string())?;
            let action = object.get("action").and_then(Value::as_str).ok_or_else(|| "lc_native requires action".to_string())?;
            let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
            if action == "openLibrary" {
                let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                open_library_action(state.host.clone(), state.startup_error.clone(), state.preferences.clone(), params)?
            } else if matches!(action, "backupLibrary" | "restoreLibrary") {
                let path = params.get("path").and_then(Value::as_str).ok_or_else(|| format!("{action} requires path in QA control mode"))?;
                let host = control_host(app)?;
                let result = host.run(if action == "backupLibrary" { "library.backup" } else { "library.restore" }.into(), json!({"path": path}))?;
                if action == "restoreLibrary" {
                    let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                    persist_library_path(&state.preferences, Some(&host), result, None)?
                } else {
                    result
                }
            } else if action == "preferences.patch" {
                let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                let value = state.preferences.patch(Some(params))?;
                if let Some(host) = state.host.clone() {
                    host.preferences(Some(value.clone()))?;
                }
                value
            } else if action == "saveBeforeClose" {
                let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                let host = state.host.clone().ok_or_else(|| state.startup_error.clone().unwrap_or_else(|| "desktop host is unavailable".into()))?;
                let snapshot = host.snapshot()?;
                if let Some(reason) = active_close_reason(&snapshot) {
                    return Err(reason);
                }
                host.persist()?;
                let label = params.get("label").and_then(Value::as_str).unwrap_or("main");
                let window = app.get_webview_window(label).ok_or_else(|| format!("window {label} is unavailable"))?;
                window.close().map_err(|error| format!("closing window: {error}"))?;
                json!({"closed": true})
            } else if matches!(action, "secondWindow" | "fullscreen" | "toggleFullscreen" | "confirmClose" | "closeWindow") {
                let mapped = if action == "fullscreen" { "toggleFullscreen" } else { action };
                native_window_action(app, mapped, &params, None)?
            } else {
                services::run(action, &params)?
            }
        }
        other => return Err(format!("unknown QA control command {other}")),
    };
    control_json(result)
}

#[cfg(feature = "qa-native")]
fn qa_control_plugin() -> Option<TauriPlugin<Wry>> {
    let control = rightkit_control::embedded::Control::<Wry>::new()
        .window("main")
        .command("lc_run", |app, args| control_dispatch(app, "lc_run", args))
        .command("lc_snapshot", |app, args| control_dispatch(app, "lc_snapshot", args))
        .command("lc_view_slice", |app, args| control_dispatch(app, "lc_view_slice", args))
        .command("lc_preview", |app, args| control_dispatch(app, "lc_preview", args))
        .command("lc_preview_ack", |app, args| control_dispatch(app, "lc_preview_ack", args))
        .command("lc_qa_viewport", |app, args| control_dispatch(app, "lc_qa_viewport", args))
        .command("lc_native", |app, args| control_dispatch(app, "lc_native", args))
        .command("lc_preferences", |app, args| control_dispatch(app, "lc_preferences", args));
    control.build_if_enabled()
}

fn preview_response(app: &AppHandle<Wry>, request: Request<Vec<u8>>) -> Response<Vec<u8>> {
    let handle = request.uri().path().trim_matches('/');
    let valid = handle.len() <= 64 && handle.starts_with("lc-preview-") && !handle.contains("..") && !handle.contains('/');
    if !valid {
        return response(400, "invalid preview handle".as_bytes().to_vec(), "text/plain; charset=utf-8");
    }
    let Some(state) = app.try_state::<AppState>() else {
        return response(503, "desktop host unavailable".as_bytes().to_vec(), "text/plain; charset=utf-8");
    };
    let Some(host) = state.host.as_ref() else {
        return response(503, "desktop host unavailable".as_bytes().to_vec(), "text/plain; charset=utf-8");
    };
    let Some(bytes) = host.preview_store().get(handle) else {
        return response(404, "preview not found".as_bytes().to_vec(), "text/plain; charset=utf-8");
    };
    response(200, bytes.bytes.to_vec(), bytes.content_type)
}

fn response(status: u16, body: Vec<u8>, content_type: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Cache-Control", "no-store")
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

const PRIMARY_IDENTIFIER: &str = "ai.storyteller.lightcraft";
const MAX_PERSISTED_LIBRARY_PATH: usize = 8_192;

fn qa_hidden_enabled(value: Option<&OsStr>) -> bool {
    value.and_then(|value| value.to_str().map(str::trim)).is_some_and(|value| value.eq_ignore_ascii_case("1") || value.eq_ignore_ascii_case("true"))
}

fn runtime_modes(args: &[String]) -> (bool, Option<PathBuf>) {
    #[cfg(feature = "qa-native")]
    let qa_env = std::env::var_os("LIGHTCRAFT_DESKTOP_QA").is_some() || qa_hidden_enabled(std::env::var_os("RIGHTKIT_QA_HIDDEN").as_deref());
    #[cfg(not(feature = "qa-native"))]
    let qa_env = false;
    let qa = args.iter().any(|arg| arg == "--qa") || qa_env;
    #[cfg(feature = "qa-native")]
    let qa_data_dir = std::env::var_os("RIGHTKIT_QA_DATA_DIR").map(PathBuf::from);
    #[cfg(not(feature = "qa-native"))]
    let qa_data_dir = None;
    (qa, qa_data_dir)
}

fn explicit_library(args: &[String]) -> Option<PathBuf> {
    args.windows(2)
        .find(|pair| pair.first().is_some_and(|arg| arg == "--library"))
        .and_then(|pair| pair.get(1))
        .map(PathBuf::from)
        .or_else(|| args.iter().find_map(|arg| arg.strip_prefix("--library=")).map(PathBuf::from))
}

fn isolated_root(app: &AppHandle<Wry>, qa: bool, qa_data_dir: Option<&Path>) -> PathBuf {
    qa_data_dir
        .map(Path::to_path_buf)
        .or_else(|| qa.then(|| std::env::temp_dir().join("lightcraft-preview-qa")))
        .or_else(|| app.path().app_data_dir().ok())
        .unwrap_or_else(|| std::env::temp_dir().join("lightcraft-preview"))
}

fn preferences_root(app: &AppHandle<Wry>, qa: bool, qa_data_dir: Option<&Path>) -> PathBuf {
    qa_data_dir
        .map(Path::to_path_buf)
        .or_else(|| qa.then(|| std::env::temp_dir().join("lightcraft-preview-qa")))
        .or_else(|| app.path().app_config_dir().ok())
        .or_else(|| app.path().app_data_dir().ok())
        .unwrap_or_else(|| std::env::temp_dir().join("lightcraft-preview"))
}

fn preferences_path(app: &AppHandle<Wry>) -> PathBuf {
    let args: Vec<String> = std::env::args().collect();
    let (qa, qa_data_dir) = runtime_modes(&args);
    preferences_root(app, qa, qa_data_dir.as_deref()).join("ui.json")
}

fn persisted_library_path(path: &Path) -> Option<PathBuf> {
    let bytes = fs::read(path).ok()?;
    if bytes.len() > 4 * 1024 * 1024 {
        return None;
    }
    let value: Value = serde_json::from_slice(&bytes).ok()?;
    let path = value
        .get("libraryPath")
        .and_then(Value::as_str)
        .or_else(|| value.pointer("/settings/libraryPath").and_then(Value::as_str))
        .or_else(|| value.pointer("/settings/library_path").and_then(Value::as_str))?;
    if path.is_empty() || path.len() > MAX_PERSISTED_LIBRARY_PATH || path.contains('\0') {
        return None;
    }
    Some(PathBuf::from(path))
}

fn model_paths(primary: bool, qa: bool, config: &Path) -> (Option<PathBuf>, Option<PathBuf>) {
    if primary && !qa {
        let dir = std::env::var_os("LIGHTCRAFT_SAM3_DIR")
            .map(PathBuf::from)
            .or_else(|| lightcraft_engine::camera_profiles::config_dir().map(|path| path.join("models").join("sam3")));
        let mirrors = lightcraft_engine::camera_profiles::config_dir().map(|path| path.join("models").join("sam3-mirrors.txt"));
        return (dir, mirrors);
    }
    let root = config.parent().unwrap_or_else(|| Path::new("."));
    (Some(root.join("models").join("sam3")), Some(root.join("models").join("sam3-mirrors.txt")))
}

fn gpu_preference(value: &Value) -> bool {
    value
        .get("gpu")
        .and_then(Value::as_bool)
        .or_else(|| value.pointer("/ui/gpu").and_then(Value::as_bool))
        .or_else(|| value.pointer("/layout/gpu").and_then(Value::as_bool))
        .or_else(|| value.pointer("/settings/gpu").and_then(Value::as_bool))
        .unwrap_or(true)
}

fn gpu_marker_path(primary: bool, qa: bool, config: &Path) -> Option<PathBuf> {
    if std::env::var_os("LIGHTCRAFT_NO_PREFS").is_some() {
        return None;
    }
    if primary && !qa {
        lightcraft_engine::camera_profiles::config_dir().map(|path| path.join("gpu-init.marker"))
    } else {
        Some(config.with_file_name("gpu-init.marker"))
    }
}

fn gpu_crash_check(marker: Option<PathBuf>) -> Option<String> {
    let left = marker.as_deref().and_then(lightcraft_engine::gpu::backend::take_init_marker);
    lightcraft_engine::gpu::backend::set_init_marker(marker);
    left.map(|what| {
        format!(
            "LightCraft closed unexpectedly while starting the GPU last time ({what}), so GPU rendering is now off and photos render on the CPU. To try the GPU again, turn on Settings ▸ Performance ▸ Use the GPU for rendering; to try another graphics backend, start LightCraft with LIGHTCRAFT_GPU_BACKEND=dx12, vulkan or off."
        )
    })
}

fn prepare_primary_preferences(path: &Path) -> Option<String> {
    if path.exists() {
        return None;
    }
    let legacy = services::legacy_preferences_path()?;
    match services::migrate_preferences(&legacy, path) {
        Ok(true) => log::info!("migrated legacy LightCraft preferences from {}", legacy.display()),
        Ok(false) => {}
        Err(error) => {
            let warning = format!("legacy LightCraft preferences were not migrated: {error}");
            log::warn!("{warning}");
            return Some(warning);
        }
    }
    None
}

fn startup_options(app: &AppHandle<Wry>) -> (HostOptions, PathBuf, Option<String>) {
    let args: Vec<String> = std::env::args().collect();
    let (qa, qa_data_dir) = runtime_modes(&args);
    let primary = app.config().identifier == PRIMARY_IDENTIFIER;
    let config = preferences_root(app, qa, qa_data_dir.as_deref()).join("ui.json");
    let migration_warning = (primary && !qa).then(|| prepare_primary_preferences(&config)).flatten();
    let app_dir = isolated_root(app, qa, qa_data_dir.as_deref());
    let explicit = explicit_library(&args);
    let library = explicit
        .or_else(|| (!qa && primary).then(|| persisted_library_path(&config)).flatten())
        .or_else(|| (!qa && primary).then(lightcraft_engine::library::default_dir).flatten())
        .unwrap_or_else(|| app_dir.join("Preview Library"));
    let (sam3_dir, sam3_mirrors_file) = model_paths(primary, qa, &config);
    let demo = args.iter().any(|arg| arg == "--demo") || std::env::var_os("LIGHTCRAFT_DESKTOP_DEMO").is_some();
    let demo_count = std::env::var("LIGHTCRAFT_DEMO_COUNT").ok().and_then(|value| value.parse::<usize>().ok()).unwrap_or(0);
    (HostOptions { library_path: Some(library.clone()), demo, demo_count, sam3_dir, sam3_mirrors_file }, library, migration_warning)
}

fn build_shell() -> rightkit_shell::Shell {
    // LightCraft owns fit/fill/100% & wheel zoom controls in its stage.
    let hardening = rightkit_shell::Hardening { block_zoom: false, ..Default::default() };
    let args: Vec<String> = std::env::args().collect();
    let qa = args.iter().any(|arg| arg == "--qa")
        || std::env::var_os("LIGHTCRAFT_DESKTOP_QA").is_some()
        || qa_hidden_enabled(std::env::var_os("RIGHTKIT_QA_HIDDEN").as_deref());
    let (_, qa_data_dir) = runtime_modes(&args);
    let mut builder =
        rightkit_shell::Shell::builder("lightcraft-preview").app_name("LightCraft").main_label("main").hardening(hardening).show_on_ready(!qa);
    if let Some(data_dir) = qa_data_dir {
        builder = builder.state_dir(data_dir.join("shell"));
    }
    builder.menu(menu::spec()).build()
}

pub fn run() {
    let shell = build_shell();
    let builder = tauri::Builder::default()
        // RightKit shell stays first so single-instance arbitration & webview hardening run before app plugins.
        .plugin(shell.plugin())
        .register_asynchronous_uri_scheme_protocol("lightcraft-preview", |ctx, request, responder| {
            let app = ctx.app_handle().clone();
            responder.respond(preview_response(&app, request));
        })
        .invoke_handler(rightkit_shell::handler![
            tauri_commands::lc_run,
            tauri_commands::lc_snapshot,
            tauri_commands::lc_view_slice,
            tauri_commands::lc_preview,
            tauri_commands::lc_preview_ack,
            tauri_commands::lc_native,
            tauri_commands::lc_preferences
        ])
        .setup(|app| {
            let (options, _library, migration_warning) = startup_options(app.handle());
            let config = preferences_path(app.handle());
            let preferences = Preferences::load(config);
            let args: Vec<String> = std::env::args().collect();
            let (qa, _) = runtime_modes(&args);
            let primary = app.config().identifier == PRIMARY_IDENTIFIER;
            let preference_value = preferences.get().unwrap_or_else(|error| {
                log::warn!("could not read startup preferences for GPU setup: {error}");
                Value::Object(serde_json::Map::new())
            });
            let gpu_crash = gpu_crash_check(gpu_marker_path(primary, qa, preferences_path(app.handle()).as_path()));
            lightcraft_engine::gpu::set_enabled(gpu_preference(&preference_value) && gpu_crash.is_none());
            let gpu_warning = gpu_crash.map(|notice| match preferences.patch(Some(json!({"ui": {"gpu": false}}))) {
                Ok(_) => notice,
                Err(error) => format!("{notice} Saving GPU preference failed: {error}"),
            });
            let preferences_warning = preferences.warning();
            let startup_warnings =
                [preferences_warning.clone(), migration_warning.clone(), gpu_warning.clone()].into_iter().flatten().collect::<Vec<_>>();
            let startup_error = match DesktopHandle::spawn(options) {
                Ok(host) => {
                    if let Ok(value) = preferences.get() {
                        let _ = host.preferences(Some(value));
                    }
                    app.manage(AppState {
                        host: Some(host),
                        preferences,
                        startup_error: None,
                        startup_warnings: startup_warnings.clone(),
                        closing: Arc::new(AtomicBool::new(false)),
                    });
                    None
                }
                Err(error) => {
                    app.manage(AppState {
                        host: None,
                        preferences,
                        startup_error: Some(error.clone()),
                        startup_warnings: startup_warnings.clone(),
                        closing: Arc::new(AtomicBool::new(false)),
                    });
                    Some(error)
                }
            };
            if let Some(error) = startup_error {
                use tauri::Emitter;
                let _ = app.emit("lc://error", json!({"message": error}));
            }
            for warning in startup_warnings {
                use tauri::Emitter;
                let _ = app.emit("lc://notice", json!({"message": warning}));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::Emitter;
            match event {
                tauri::WindowEvent::Focused(focused) => {
                    let _ = window.emit("lc://window-focus", json!({"label": window.label(), "owner": window.label() == "main", "focused": focused}));
                }
                tauri::WindowEvent::CloseRequested { .. } if window.label() != "main" => {}
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    let Some(state) = window.app_handle().try_state::<AppState>() else { return };
                    let force = state.closing.swap(false, Ordering::SeqCst);
                    let Some(host) = &state.host else { return };
                    let snapshot = match host.snapshot() {
                        Ok(snapshot) => snapshot,
                        Err(error) => {
                            api.prevent_close();
                            let _ = window.emit("lc://error", json!({"message": error}));
                            return;
                        }
                    };
                    if let Some(reason) = active_close_reason(&snapshot) {
                        api.prevent_close();
                        let _ = window.emit("lc://close-requested", json!({"active": true, "message": reason}));
                    } else if !force && snapshot.pointer("/status/unsaved").and_then(Value::as_bool).unwrap_or(false) {
                        api.prevent_close();
                        let _ = window.emit("lc://close-requested", json!({"unsaved": true}));
                    }
                }
                tauri::WindowEvent::DragDrop(tauri::DragDropEvent::Drop { paths, .. }) => {
                    let _ = window.emit("lc://native-drop", json!({"paths": paths}));
                }
                tauri::WindowEvent::Destroyed => {
                    if window.label() == "main"
                        && let Some(state) = window.app_handle().try_state::<AppState>()
                        && let Some(host) = state.host.clone()
                        && let Err(error) = host.shutdown()
                    {
                        log::error!("desktop host shutdown failed: {error}");
                    }
                }
                _ => {}
            }
        });
    #[cfg(feature = "qa-native")]
    let builder = if let Some(control) = qa_control_plugin() { builder.plugin(control) } else { builder };
    #[cfg(target_os = "macos")]
    let builder =
        if qa_hidden_enabled(std::env::var_os("RIGHTKIT_QA_BACKGROUND").as_deref()) { builder.activate_ignoring_other_apps(false) } else { builder };
    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("lightcraft desktop failed: {error}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn persisted_library_path_reads_legacy_nested_key() -> Result<(), Box<dyn std::error::Error>> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("lightcraft-native-path-{}-{stamp}.json", std::process::id()));
        fs::write(&path, br#"{"settings":{"libraryPath":"/legacy/catalog"}}"#)?;
        assert_eq!(persisted_library_path(&path), Some(PathBuf::from("/legacy/catalog")));
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn persisted_library_path_rejects_malformed_value() -> Result<(), Box<dyn std::error::Error>> {
        let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
        let path = std::env::temp_dir().join(format!("lightcraft-native-path-invalid-{}-{stamp}.json", std::process::id()));
        fs::write(&path, br#"{"libraryPath":""}"#)?;
        assert!(persisted_library_path(&path).is_none());
        fs::remove_file(path)?;
        Ok(())
    }

    #[test]
    fn qa_hidden_flag_requires_enabled_value() {
        assert!(!qa_hidden_enabled(Some(OsStr::new("0"))));
        assert!(!qa_hidden_enabled(Some(OsStr::new("false"))));
        assert!(!qa_hidden_enabled(Some(OsStr::new(""))));
        assert!(qa_hidden_enabled(Some(OsStr::new("1"))));
        assert!(qa_hidden_enabled(Some(OsStr::new("true"))));
        assert!(qa_hidden_enabled(Some(OsStr::new(" TRUE "))));
        assert!(!qa_hidden_enabled(None));
    }
}
