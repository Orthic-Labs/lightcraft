#![cfg(any(target_os = "macos", target_os = "windows"))]

mod fixture_inputs;

use std::fs;
use std::panic::{catch_unwind, resume_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use rightkit_qa::control::{launch, LaunchSpec, Mode};
use rightkit_qa::harness::Harness;
use rightkit_qa::util::sha256_hex;
use rightkit_qa::workspace;
use serde_json::{json, Value};

fn required_env(name: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| panic!("{name} is required for native qualification"))
}

fn fixture_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../fixtures/desktop")
}

fn qa_harness(binary: &Path, evidence: &Path, revision: &str, platform: &str, architecture: &str) -> Harness {
    Harness::new("lightcraft-desktop", PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../.."))
        .expect("RightKit QA harness must initialize")
        .with_evidence_root(evidence)
        .with_identity(json!({
            "sourceRevision": revision,
            "platform": platform,
            "architecture": architecture,
            "webview": if platform == "macos" { "WKWebView" } else { "WebView2" },
            "hidden": true,
        }))
        .with_identity_file("ui", binary)
        .expect("QA binary identity must be hashable")
}

fn launch_hidden(binary: &Path, scenario: &rightkit_qa::harness::Scenario, catalog: &Path) -> (rightkit_qa::control::Control, PathBuf) {
    let cache = scenario.dir().join("control-workspace");
    let run_id = format!("lightcraft-{}", scenario.name());
    let ws = workspace::create(&cache, Some(&run_id), "lightcraft").expect("isolated RightKit workspace must initialize");
    let data = ws.data_dir.clone();
    let env = ws.env.into_iter().collect::<Vec<_>>();
    let spec = LaunchSpec {
        binary: binary.to_path_buf(),
        mode: Mode::Hidden,
        env: {
            let mut values = env;
            values.push(("RIGHTKIT_QA_CATALOG".into(), catalog.display().to_string()));
            values
        },
        startup_timeout: Duration::from_secs(90),
        label: "lightcraft-desktop-native".into(),
    };
    let control = launch(&spec, &ws, scenario.tracker()).expect("hidden native app must expose rightkit-control");
    (control, data)
}

fn run(control: &rightkit_qa::control::Control, id: &str, params: Value) -> Value {
    control.command("lc_run", &json!({"id": id, "params": params})).unwrap_or_else(|error| panic!("{id} failed: {error}"))
}

fn snapshot(control: &rightkit_qa::control::Control) -> Value {
    control.command("lc_snapshot", &Value::Null).expect("native snapshot must reply")
}

fn wait_task(control: &rightkit_qa::control::Control, task_id: &str) -> Value {
    for _ in 0..900 {
        let value = snapshot(control);
        let running = value["status"]["jobs"].as_array().map_or(false, |jobs| jobs.iter().any(|job| job["id"].as_str() == Some(task_id)));
        if !running {
            let notices = value["status"]["notices"].as_array().cloned().unwrap_or_default();
            assert!(
                !notices.iter().any(|notice| notice.as_str().is_some_and(|text| text.contains("failed"))),
                "native task {task_id} failed: {notices:?}"
            );
            return value;
        }
        sleep(Duration::from_millis(100));
    }
    panic!("native task {task_id} did not finish within 90 seconds");
}

fn import_file(control: &rightkit_qa::control::Control, path: &Path) -> Value {
    let preview = run(control, "library.importPreview", json!({"paths": [path]}));
    assert!(preview["candidates"].as_array().is_some_and(|items| !items.is_empty()), "real fixture must produce an import candidate: {preview}");
    let started = run(control, "library.import", json!({"paths": [path], "mode": "add"}));
    let task_id = started["taskId"].as_str().expect("import must return task id").to_string();
    let snapshot = wait_task(control, &task_id);
    assert!(snapshot["counts"]["catalog"].as_u64().is_some_and(|count| count > 0), "import must add catalog photo: {snapshot}");
    snapshot
}

fn preview_imported(control: &rightkit_qa::control::Control, snapshot: &Value) -> Value {
    let photo_id = snapshot["active"].as_u64().expect("import must select active photo");
    let request = json!({
        "photoId": photo_id,
        "slot": "qualification",
        "viewGeneration": snapshot["viewGeneration"],
        "width": 96,
        "height": 64,
        "quality": "draft",
        "before": false,
        "sequence": 1,
    });
    let descriptor = control.command("lc_preview", &json!({"request": request})).expect("real preview must render");
    assert!(descriptor["handle"].as_str().is_some_and(|handle| !handle.is_empty()), "preview must return stored handle: {descriptor}");
    assert_eq!(descriptor["viewGeneration"], snapshot["viewGeneration"]);
    descriptor
}

fn tree_fingerprint(path: &Path) -> String {
    fn walk(root: &Path, path: &Path, rows: &mut Vec<String>) {
        let mut entries =
            fs::read_dir(path).expect("backup tree must be readable").collect::<Result<Vec<_>, _>>().expect("backup entries must be readable");
        entries.sort_by_key(|entry| entry.file_name());
        for entry in entries {
            let child = entry.path();
            if child.is_dir() {
                walk(root, &child, rows);
            } else {
                let bytes = fs::read(&child).expect("backup file must be readable");
                rows.push(format!("{}:{}", child.strip_prefix(root).unwrap_or(&child).display(), sha256_hex(&bytes)));
            }
        }
    }
    let mut rows = Vec::new();
    walk(path, path, &mut rows);
    sha256_hex(rows.join("\n").as_bytes())
}

fn with_control<T>(
    binary: &Path,
    scenario: &rightkit_qa::harness::Scenario,
    catalog: &Path,
    body: impl FnOnce(&rightkit_qa::control::Control, &Path) -> T,
) -> T {
    let (mut control, data) = launch_hidden(binary, scenario, catalog);
    let result = catch_unwind(AssertUnwindSafe(|| body(&control, &data)));
    let stopped = control.stop().expect("native app must stop cleanly");
    assert!(stopped.endpoint_closed, "rightkit-control endpoint must close");
    assert!(stopped.process_gone, "native app process must exit");
    if cfg!(target_os = "macos") {
        assert!(stopped.owned_never_frontmost, "hidden qualification must never activate app");
    }
    match result {
        Ok(value) => value,
        Err(error) => resume_unwind(error),
    }
}

#[test]
#[ignore = "target-native candidate qualification only; invoke with cargo test --ignored on CI"]
fn native_hidden_control_journeys() {
    let binary = PathBuf::from(required_env("RIGHTKIT_QA_UI_BINARY"));
    assert!(binary.is_file(), "candidate executable must exist: {}", binary.display());
    let evidence = PathBuf::from(required_env("RIGHTKIT_QA_EVIDENCE"));
    let revision = required_env("RIGHTKIT_QA_SOURCE_REVISION");
    let architecture = required_env("RIGHTKIT_QA_ARCHITECTURE");
    let installed_hash = required_env("RIGHTKIT_QA_INSTALLED_ARTIFACT_SHA256");
    assert!(installed_hash.len() == 64 && installed_hash.bytes().all(|byte| byte.is_ascii_hexdigit()), "installed artifact hash must be SHA-256");
    let platform = if cfg!(target_os = "macos") { "macos" } else { "windows" };
    let baseline: Value =
        serde_json::from_str(&fs::read_to_string(fixture_root().join("installed-baseline.json")).expect("baseline fixture must be readable"))
            .expect("baseline fixture must be valid JSON");
    assert_eq!(baseline["rollback"]["required"].as_bool(), Some(true));
    let input_dir = evidence.join("fixture-inputs");
    fs::create_dir_all(&input_dir).expect("fixture input directory must exist");
    let inputs = fixture_inputs::write_fixture_inputs(&input_dir).expect("real ARW/PNG/Lightroom fixtures must be generated");
    let harness = qa_harness(&binary, &evidence, &revision, platform, &architecture);

    let scenario_names = ["ipc", "stalePreview", "cache", "preferences", "gesture", "arwImport", "lightroomImport", "cliExport", "rollback"];
    for name in scenario_names {
        let outcome = harness.scenario(name, "fast", &[], |scenario| {
            let baseline_capture = scenario.dir().join("baseline.png");
            if name == "preferences" {
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let updated = control
                        .command("lc_preferences", &json!({"qaSentinel": "preserved", "ui": {"theme": "dark"}}))
                        .expect("preferences patch must reply");
                    assert_eq!(updated["qaSentinel"].as_str(), Some("preserved"));
                });
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let reopened = control.command("lc_preferences", &Value::Null).expect("preferences reopen must reply after process restart");
                    assert_eq!(reopened["qaSentinel"].as_str(), Some("preserved"), "unknown preference fields must survive process restart");
                    let dark = scenario.dir().join("dark-theme.png");
                    control.screenshot_to(&dark).expect("dark theme screenshot must be captured");
                    assert!(dark.is_file());
                    control.command("lc_preferences", &json!({"ui": {"theme": "light"}})).expect("light theme preference must persist");
                });
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let light = scenario.dir().join("light-theme.png");
                    control.screenshot_to(&light).expect("light theme screenshot must be captured");
                    assert!(light.is_file());
                });
            } else {
                with_control(&binary, scenario, &inputs.catalog, |control, _data| match name {
                    "ipc" => {
                        let health = control.health().expect("health RPC must reply");
                        assert!(health.is_object(), "health RPC must return object");
                        let snapshot = control.command("lc_snapshot", &Value::Null).expect("snapshot command must reply");
                        assert_eq!(snapshot["version"].as_u64(), Some(1));
                        assert!(snapshot["viewGeneration"].is_number());
                        assert!(snapshot["controls"].is_array());
                        let viewport =
                            control.eval("return {width: window.innerWidth, height: window.innerHeight};").expect("viewport query must execute");
                        assert!(viewport["width"].as_u64().is_some_and(|width| width >= 1280));
                        assert!(viewport["height"].as_u64().is_some_and(|height| height >= 800));
                        let library = scenario.dir().join("route-library.png");
                        control.screenshot_to(&library).expect("library route screenshot must be captured");
                        control.key("D").expect("develop route key must execute");
                        let develop = scenario.dir().join("route-develop.png");
                        control.screenshot_to(&develop).expect("develop route screenshot must be captured");
                        control.key("G").expect("library route key must execute");
                    }
                    "stalePreview" => {
                        let snapshot = control.command("lc_snapshot", &Value::Null).expect("snapshot must reply");
                        let generation = snapshot["viewGeneration"].as_u64().expect("snapshot generation required");
                        let slice = control
                            .command("lc_view_slice", &json!({"generation": generation + 1, "offset": 0, "limit": 512}))
                            .expect("slice must reply");
                        assert_eq!(slice["generationChanged"].as_bool(), Some(true), "stale generation must be rejected");
                    }
                    "cache" => {
                        let imported = import_file(control, &inputs.png);
                        let slice = control
                            .command("lc_view_slice", &json!({"generation": imported["viewGeneration"], "offset": 0, "limit": 4096}))
                            .expect("bounded slice must reply");
                        assert!(slice["photos"].as_array().map_or(false, |photos| photos.len() <= 512));
                        assert_eq!(slice["generation"], imported["viewGeneration"]);
                        let dom = control.dom(".lc-grid-scroll").expect("grid DOM query must execute");
                        assert!(!dom.is_empty(), "library grid must exist in hidden WebView");
                    }
                    "gesture" => {
                        let imported = import_file(control, &inputs.png);
                        let before = imported["undo"].as_u64().expect("snapshot undo count required");
                        let active = imported["active"].as_u64().expect("import must select active photo");
                        run(control, "develop.beginInteraction", json!({"label": "Qualification exposure drag"}));
                        run(control, "develop.set", json!({"control": "light.exposure", "value": 1.25, "ids": [active]}));
                        run(control, "develop.endInteraction", json!({}));
                        let edited = snapshot(control);
                        assert_eq!(edited["undo"].as_u64(), Some(before + 1), "one gesture must create one undo step");
                        run(control, "develop.beginInteraction", json!({"label": "Cancelled exposure drag"}));
                        run(control, "develop.set", json!({"control": "light.exposure", "value": -1.25, "ids": [active]}));
                        run(control, "develop.cancelInteraction", json!({}));
                        let cancelled = snapshot(control);
                        assert_eq!(cancelled["undo"].as_u64(), edited["undo"].as_u64(), "cancelled gesture must not create undo step");
                        control.move_to(300.0, 300.0).expect("pointer move must execute");
                        control.drag((300.0, 300.0), (420.0, 320.0), 8).expect("pointer drag must execute");
                        control.wheel(420.0, 320.0, 0.0, -120.0).expect("wheel must execute");
                        control.key("Escape").expect("Escape must execute");
                        assert!(control
                            .eval("return document.activeElement !== null;")
                            .expect("active element query must execute")
                            .as_bool()
                            .unwrap_or(false));
                    }
                    "arwImport" => {
                        let imported = import_file(control, &inputs.arw);
                        let descriptor = preview_imported(control, &imported);
                        let screenshot = scenario.dir().join("arw-preview.png");
                        control.screenshot_to(&screenshot).expect("ARW preview screenshot must be captured");
                        assert!(screenshot.is_file());
                        assert!(descriptor["width"].as_u64().is_some_and(|width| width > 0));
                    }
                    "lightroomImport" => {
                        let first = run(control, "library.importLightroom", json!({"path": inputs.catalog, "updateExisting": false}));
                        assert_eq!(first["photos"].as_u64(), Some(2), "Lightroom fixture must import master & virtual copy: {first}");
                        assert_eq!(first["collections"].as_u64(), Some(2), "nested Lightroom collections must import: {first}");
                        assert_eq!(first["missing"].as_array().map(Vec::len), Some(0));
                        let mapping = first["mapping"].clone();
                        let second = run(control, "library.importLightroom", json!({"path": inputs.catalog, "updateExisting": true}));
                        assert_eq!(second["mapping"], mapping, "reimport must preserve source identities");
                        assert_eq!(snapshot(control)["counts"]["catalog"].as_u64(), Some(2));
                    }
                    "cliExport" => {
                        let imported = import_file(control, &inputs.png);
                        let id = imported["active"].as_u64().expect("PNG import must select photo");
                        let output = scenario.dir().join("export");
                        fs::create_dir_all(&output).expect("export directory must exist");
                        let started = run(control, "app.export", json!({"ids": [id], "dir": output, "format": "png", "longEdge": 96}));
                        let task_id = started["taskId"].as_str().expect("export must return task id").to_string();
                        wait_task(control, &task_id);
                        let exported = fs::read_dir(&output)
                            .expect("export output must exist")
                            .filter_map(Result::ok)
                            .map(|entry| entry.path())
                            .find(|path| path.is_file())
                            .expect("export must write file");
                        let bytes = fs::read(exported).expect("exported pixels must be readable");
                        assert!(bytes.starts_with(b"\x89PNG\r\n\x1a\n"), "CLI export must contain PNG pixels");
                    }
                    "rollback" => {
                        let before = import_file(control, &inputs.png);
                        let before_counts = before["counts"].clone();
                        let backup = scenario.dir().join("library-backup");
                        let backup_result = run(control, "library.backup", json!({"path": backup}));
                        assert!(backup_result["path"].as_str().is_some(), "backup must return destination: {backup_result}");
                        assert!(backup.is_dir(), "backup directory must exist");
                        let backup_fingerprint = tree_fingerprint(&backup);
                        let restored = run(control, "library.restore", json!({"path": backup}));
                        let backup_text = backup.to_string_lossy().to_string();
                        assert_eq!(restored["libraryPath"].as_str(), Some(backup_text.as_str()));
                        let after = snapshot(control);
                        assert_eq!(after["counts"], before_counts, "restore must recover catalog counts");
                        assert_eq!(tree_fingerprint(&backup), backup_fingerprint, "restore must leave backup bytes unchanged");
                        control.screenshot_to(&baseline_capture).expect("baseline screenshot must be captured");
                        assert!(baseline_capture.is_file());
                        let receipt = scenario.dir().join("rollback.json");
                        fs::write(
                            &receipt,
                            serde_json::to_vec_pretty(&json!({
                                "schema": 1,
                                "sourceRevision": revision,
                                "installedArtifactSha256": installed_hash,
                                "baseline": baseline["baseline"],
                                "beforeCounts": before_counts,
                                "afterCounts": after["counts"],
                                "backupFingerprint": backup_fingerprint,
                                "restoredLibraryPath": restored["libraryPath"],
                            }))
                            .expect("rollback receipt must serialize"),
                        )
                        .expect("rollback receipt must be writable");
                        scenario.keep("rollback.json", &receipt);
                    }
                    _ => unreachable!("scenario inventory is static"),
                });
            }
            scenario.note(format!("executed hidden {platform} control journey: {name}"));
        });
        assert!(!outcome.is_skipped(), "native journey {name} must execute, not skip");
    }
}
