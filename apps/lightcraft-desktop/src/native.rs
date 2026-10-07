#[path = "menu.rs"]
mod menu;
#[path = "preferences.rs"]
mod preferences;
#[path = "services.rs"]
mod services;

use std::path::PathBuf;

use lightcraft_desktop_host::{DesktopHandle, HostOptions, PreviewRequest};
use serde_json::{Value, json};
use tauri::http::{Request, Response};
#[cfg(feature = "qa-native")]
use tauri::plugin::TauriPlugin;
use tauri::{AppHandle, Manager, State, WebviewUrl, WebviewWindowBuilder, Wry};

use self::preferences::Preferences;

struct AppState {
    host: Option<DesktopHandle>,
    preferences: Preferences,
    startup_error: Option<String>,
}

fn blocking<T, F>(work: F) -> impl std::future::Future<Output = Result<T, String>>
where
    T: Send + 'static,
    F: FnOnce() -> Result<T, String> + Send + 'static,
{
    async move { tauri::async_runtime::spawn_blocking(work).await.map_err(|error| format!("native task failed: {error}"))? }
}

#[tauri::command]
async fn lc_run(state: State<'_, AppState>, id: String, params: Value) -> Result<Value, String> {
    let host = state.host.clone();
    let error = state.startup_error.clone();
    blocking(move || host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(|host| host.run(id, params)))
        .await
}

#[tauri::command]
async fn lc_snapshot(state: State<'_, AppState>) -> Result<Value, String> {
    let host = state.host.clone();
    let error = state.startup_error.clone();
    blocking(move || host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(DesktopHandle::snapshot))
        .await
}

#[tauri::command]
async fn lc_view_slice(state: State<'_, AppState>, generation: Option<u64>, offset: usize, limit: usize) -> Result<Value, String> {
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
async fn lc_preview(state: State<'_, AppState>, request: PreviewRequest) -> Result<lightcraft_desktop_host::PreviewDescriptor, String> {
    let host = state.host.clone();
    let error = state.startup_error.clone();
    blocking(move || {
        host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into())).and_then(|host| host.preview(request))
    })
    .await
}

#[tauri::command]
async fn lc_preview_ack(state: State<'_, AppState>, handle: String) -> Result<bool, String> {
    let host = state.host.clone();
    let error = state.startup_error.clone();
    blocking(move || {
        let host = host.as_ref().ok_or_else(|| error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
        Ok(host.preview_store().acknowledge(&handle))
    })
    .await
}

#[tauri::command]
async fn lc_preferences(state: State<'_, AppState>, patch: Option<Value>) -> Result<Value, String> {
    let preferences = state.preferences.clone();
    let host = state.host.clone();
    let startup_error = state.startup_error.clone();
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
async fn lc_native(action: String, params: Value, app: AppHandle<Wry>, state: State<'_, AppState>) -> Result<Value, String> {
    if matches!(action.as_str(), "backupLibrary" | "restoreLibrary") {
        let host = state.host.clone();
        let startup_error = state.startup_error.clone();
        return blocking(move || {
            let path = if let Some(path) = params.get("path").and_then(Value::as_str) {
                path.to_string()
            } else {
                let picker = if action == "backupLibrary" {
                    services::run("saveFile", &json!({"suggestedName": "LightCraft Library Backup.lclibrary", "extensions": ["lclibrary", "zip"]}))
                } else {
                    services::run("openFile", &json!({"multiple": false, "extensions": ["lclibrary", "zip"]}))
                }?;
                picker.get("path").and_then(Value::as_str).map(str::to_string).ok_or_else(|| "no library backup selected".to_string())?
            };
            if path.is_empty() || path.len() > 8_192 {
                return Err("invalid library backup path".into());
            }
            let host = host.as_ref().ok_or_else(|| startup_error.unwrap_or_else(|| "desktop host is unavailable".into()))?;
            host.run(if action == "backupLibrary" { "library.backup" } else { "library.restore" }.into(), json!({"path": path}))
        })
        .await;
    }
    if action == "preferences.patch" {
        let preferences = state.preferences.clone();
        let host = state.host.clone();
        return blocking(move || {
            let value = preferences.patch(Some(params))?;
            if let Some(host) = host { host.preferences(Some(value.clone())) } else { Ok(value) }
        })
        .await;
    }
    if matches!(action.as_str(), "secondWindow" | "fullscreen" | "toggleFullscreen" | "confirmClose" | "closeWindow") {
        let action = if action == "fullscreen" { "toggleFullscreen" } else { action.as_str() };
        return native_window_action(&app, &action, &params);
    }
    blocking(move || services::run(&action, &params)).await
}

fn native_window_action(app: &AppHandle<Wry>, action: &str, params: &Value) -> Result<Value, String> {
    let label = params.get("label").and_then(Value::as_str).unwrap_or("main");
    let window = app.get_webview_window(label).ok_or_else(|| format!("window {label} is unavailable"))?;
    match action {
        "secondWindow" => {
            if app.get_webview_window("second-main").is_none() {
                WebviewWindowBuilder::new(app, "second-main", WebviewUrl::App("index.html".into()))
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
            window.close().map_err(|error| format!("closing window: {error}"))?;
            Ok(Value::Null)
        }
        _ => Err(format!("unknown native window action {action}")),
    }
}

#[cfg(feature = "qa-native")]
fn control_args(args: &str) -> Result<Value, String> {
    if args.trim().is_empty() { Ok(Value::Null) } else { serde_json::from_str(args).or_else(|_| Ok(Value::String(args.to_string()))) }
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
fn control_dispatch(app: &AppHandle<Wry>, name: &str, raw: &str) -> Result<String, String> {
    let args = control_args(raw)?;
    let result = match name {
        "lc_run" => {
            let object = args.as_object().ok_or_else(|| "lc_run expects an object".to_string())?;
            let id = object.get("id").and_then(Value::as_str).ok_or_else(|| "lc_run requires id".to_string())?;
            control_host(app)?.run(id.to_string(), object.get("params").cloned().unwrap_or_else(|| json!({})))?
        }
        "lc_snapshot" => control_host(app)?.snapshot()?,
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
        "lc_native" => {
            let object = args.as_object().ok_or_else(|| "lc_native expects an object".to_string())?;
            let action = object.get("action").and_then(Value::as_str).ok_or_else(|| "lc_native requires action".to_string())?;
            let params = object.get("params").cloned().unwrap_or_else(|| json!({}));
            if matches!(action, "backupLibrary" | "restoreLibrary") {
                let path = params.get("path").and_then(Value::as_str).ok_or_else(|| format!("{action} requires path in QA control mode"))?;
                control_host(app)?.run(if action == "backupLibrary" { "library.backup" } else { "library.restore" }.into(), json!({"path": path}))?
            } else if action == "preferences.patch" {
                let state = app.try_state::<AppState>().ok_or_else(|| "desktop state is unavailable".to_string())?;
                let value = state.preferences.patch(Some(params))?;
                if let Some(host) = state.host.clone() {
                    host.preferences(Some(value.clone()))?;
                }
                value
            } else if matches!(action, "secondWindow" | "fullscreen" | "toggleFullscreen" | "confirmClose" | "closeWindow") {
                let mapped = if action == "fullscreen" { "toggleFullscreen" } else { action };
                native_window_action(app, mapped, &params)?
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
    response(200, bytes.bytes.to_vec(), &bytes.content_type)
}

fn response(status: u16, body: Vec<u8>, content_type: &str) -> Response<Vec<u8>> {
    Response::builder()
        .status(status)
        .header("Content-Type", content_type)
        .header("Cache-Control", "no-store")
        .body(body)
        .unwrap_or_else(|_| Response::new(Vec::new()))
}

fn startup_options(app: &AppHandle<Wry>) -> (HostOptions, PathBuf) {
    let args: Vec<String> = std::env::args().collect();
    let explicit = args
        .windows(2)
        .find(|pair| pair.first().is_some_and(|arg| arg == "--library"))
        .and_then(|pair| pair.get(1))
        .map(PathBuf::from)
        .or_else(|| args.iter().find_map(|arg| arg.strip_prefix("--library=")).map(PathBuf::from));
    let qa = args.iter().any(|arg| arg == "--qa")
        || std::env::var_os("LIGHTCRAFT_DESKTOP_QA").is_some()
        || std::env::var_os("RIGHTKIT_QA_HIDDEN").is_some();
    let demo = qa || args.iter().any(|arg| arg == "--demo") || std::env::var_os("LIGHTCRAFT_DESKTOP_DEMO").is_some();
    let app_dir = std::env::var_os("RIGHTKIT_QA_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| app.path().app_data_dir().ok())
        .unwrap_or_else(|| std::env::temp_dir().join("lightcraft-preview"));
    let library = explicit.unwrap_or_else(|| app_dir.join("Preview Library"));
    let demo_count = std::env::var("LIGHTCRAFT_DEMO_COUNT").ok().and_then(|value| value.parse::<usize>().ok()).unwrap_or(0);
    (HostOptions { library_path: Some(library.clone()), demo, demo_count }, library)
}

fn build_shell() -> rightkit_shell::Shell {
    let mut hardening = rightkit_shell::Hardening::default();
    // LightCraft owns fit/fill/100% & wheel zoom controls in its stage.
    hardening.block_zoom = false;
    let qa = std::env::args().any(|arg| arg == "--qa")
        || std::env::var_os("LIGHTCRAFT_DESKTOP_QA").is_some()
        || std::env::var_os("RIGHTKIT_QA_HIDDEN").is_some();
    rightkit_shell::Shell::builder("lightcraft-preview")
        .app_name("LightCraft")
        .main_label("main")
        .hardening(hardening)
        .show_on_ready(!qa)
        .menu(menu::spec())
        .on_menu(|app, id| {
            use tauri::Emitter;
            let _ = app.emit("lc://menu", json!({"id": id}));
        })
        .build()
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
        .invoke_handler(rightkit_shell::handler![lc_run, lc_snapshot, lc_view_slice, lc_preview, lc_preview_ack, lc_native, lc_preferences])
        .setup(|app| {
            let (options, _library) = startup_options(app.handle());
            let config_root = std::env::var_os("RIGHTKIT_QA_DATA_DIR")
                .map(PathBuf::from)
                .or_else(|| app.path().app_config_dir().ok())
                .or_else(|| app.path().app_data_dir().ok())
                .unwrap_or_else(|| std::env::temp_dir().join("lightcraft-preview"));
            let config = config_root.join("ui.json");
            let preferences = Preferences::load(config);
            let preferences_warning = preferences.warning();
            let startup_error = match DesktopHandle::spawn(options) {
                Ok(host) => {
                    if let Ok(value) = preferences.get() {
                        let _ = host.preferences(Some(value));
                    }
                    app.manage(AppState { host: Some(host), preferences, startup_error: None });
                    None
                }
                Err(error) => {
                    app.manage(AppState { host: None, preferences, startup_error: Some(error.clone()) });
                    Some(error)
                }
            };
            if let Some(error) = startup_error {
                use tauri::Emitter;
                let _ = app.emit("lc://error", json!({"message": error}));
            }
            if let Some(warning) = preferences_warning {
                use tauri::Emitter;
                let _ = app.emit("lc://notice", json!({"message": warning}));
            }
            Ok(())
        })
        .on_window_event(|window, event| {
            use tauri::Emitter;
            match event {
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    if let Some(state) = window.app_handle().try_state::<AppState>()
                        && let Some(host) = &state.host
                        && let Ok(snapshot) = host.snapshot()
                        && snapshot.pointer("/status/unsaved").and_then(Value::as_bool).unwrap_or(false)
                    {
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
                        let _ = window.app_handle().emit("lc://error", json!({"message": format!("saving library on exit: {error}")}));
                    }
                }
                _ => {}
            }
        });
    #[cfg(feature = "qa-native")]
    let builder = if let Some(control) = qa_control_plugin() { builder.plugin(control) } else { builder };
    if let Err(error) = builder.run(tauri::generate_context!()) {
        eprintln!("lightcraft desktop failed: {error}");
    }
}
