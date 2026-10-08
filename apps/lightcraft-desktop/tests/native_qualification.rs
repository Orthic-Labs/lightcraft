#![cfg(any(target_os = "macos", target_os = "windows"))]

mod fixture_inputs;

use std::fs;
use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::Duration;

use rightkit_qa::control::{LaunchSpec, Mode, launch};
use rightkit_qa::harness::Harness;
use rightkit_qa::util::sha256_hex;
use rightkit_qa::workspace;
use serde_json::{Value, json};

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
    let env = ws.env.clone().into_iter().collect::<Vec<_>>();
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

fn wait_for_snapshot(control: &rightkit_qa::control::Control, ready: impl Fn(&Value) -> bool, message: &str) -> Value {
    for _ in 0..100 {
        let value = snapshot(control);
        if ready(&value) {
            return value;
        }
        sleep(Duration::from_millis(50));
    }
    panic!("{message}");
}

fn source_hash(path: &Path) -> String {
    sha256_hex(&fs::read(path).unwrap_or_else(|error| panic!("fixture {} must be readable: {error}", path.display())))
}

fn source_fingerprint(path: &Path) -> String {
    if path.is_dir() { tree_fingerprint(path) } else { source_hash(path) }
}

fn assert_sources_unchanged(paths: &[(&str, &Path)], expected: &[(&str, String)]) {
    assert_eq!(paths.len(), expected.len(), "fixture hash inventory must match source inventory");
    for ((label, path), (expected_label, before)) in paths.iter().zip(expected.iter()) {
        assert_eq!(label, expected_label, "fixture hash inventory labels must match");
        assert_eq!(source_hash(path), before.as_str(), "native journey mutated source fixture {label}");
    }
}

fn wait_for_dom(control: &rightkit_qa::control::Control, expression: &str) -> Value {
    for _ in 0..100 {
        let value = control.eval(expression).expect("DOM route query must execute");
        if value.as_bool() == Some(true) {
            return value;
        }
        sleep(Duration::from_millis(50));
    }
    panic!("DOM route condition did not become true: {expression}");
}

fn click_dom(control: &rightkit_qa::control::Control, selector: &str, message: &str) {
    let geometry = control
        .eval(&format!(
            "return (() => {{ const node = document.querySelector({selector:?}); if (!node) return null; const rect = node.getBoundingClientRect(); return {{x: rect.x + rect.width / 2, y: rect.y + rect.height / 2}}; }})();"
        ))
        .expect("DOM click geometry query must execute");
    let x = geometry["x"].as_f64().expect("DOM click x must be numeric");
    let y = geometry["y"].as_f64().expect("DOM click y must be numeric");
    control.click(x, y, "left", 1).expect(message);
}

fn set_native_viewport(control: &rightkit_qa::control::Control, width: u64, height: u64) -> Value {
    let result = control.command("lc_qa_viewport", &json!({"width": width, "height": height})).expect("QA viewport resize command must execute");
    assert_eq!(result["requested"]["width"].as_u64(), Some(width), "QA viewport must report requested width");
    assert_eq!(result["requested"]["height"].as_u64(), Some(height), "QA viewport must report requested height");
    assert!(result["observedInnerSize"]["width"].as_u64().is_some_and(|value| value > 0), "QA viewport must report observed width: {result}");
    assert!(result["observedInnerSize"]["height"].as_u64().is_some_and(|value| value > 0), "QA viewport must report observed height: {result}");
    let settled = wait_for_dom(control, &format!("return window.innerWidth === {width} && window.innerHeight === {height};"));
    assert_eq!(settled.as_bool(), Some(true), "WebView inner size must match requested QA viewport");
    result
}

fn assert_layout_settled(control: &rightkit_qa::control::Control, selector: &str) {
    let expression = format!(
        "return (() => {{ const node = document.querySelector({selector:?}); if (!node) return false; const rect = node.getBoundingClientRect(); return rect.width >= 1 && rect.height >= 1 && getComputedStyle(node).display !== 'none'; }})();"
    );
    assert_eq!(wait_for_dom(control, &expression).as_bool(), Some(true), "layout must settle for {selector}");
}

fn is_scoped_preview_src(src: &str) -> bool {
    let prefix = if cfg!(target_os = "windows") { "http://lightcraft-preview.localhost/" } else { "lightcraft-preview://localhost/" };
    src.strip_prefix(prefix).is_some_and(|handle| !handle.is_empty() && !handle.contains("..") && !handle.contains('/'))
}

fn wait_for_rendered_preview(control: &rightkit_qa::control::Control, selector: &str, expected_src: Option<&str>) -> Value {
    let expression = format!(
        "return (() => {{ const img = document.querySelector({selector:?}); if (!img) return {{ready:false, reason:'missing'}}; const cell = img.closest('.lc-photo-cell'); const src = img.getAttribute('src') || ''; return {{ready: img.complete && img.naturalWidth > 0 && img.naturalHeight > 0, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight, src, active: Boolean(cell?.classList.contains('is-active')), selected: Boolean(cell?.getAttribute('aria-selected') === 'true')}}; }})();"
    );
    for _ in 0..160 {
        let value = control.eval(&expression).expect("rendered preview DOM query must execute");
        let ready = value["ready"].as_bool() == Some(true);
        let scoped = value["src"].as_str().is_some_and(is_scoped_preview_src);
        let changed = expected_src.is_none_or(|previous| value["src"].as_str() != Some(previous));
        if ready && scoped && changed {
            assert!(value["naturalWidth"].as_u64().is_some_and(|width| width > 0));
            assert!(value["naturalHeight"].as_u64().is_some_and(|height| height > 0));
            if selector.contains("lc-photo-cell") {
                assert_eq!(value["active"].as_bool(), Some(true), "active grid cell must own rendered preview: {value}");
                assert_eq!(value["selected"].as_bool(), Some(true), "active grid cell must be selected: {value}");
            }
            return value;
        }
        sleep(Duration::from_millis(50));
    }
    panic!("rendered preview did not become ready for selector {selector}: {expression}");
}

fn assert_active_grid_is_bounded(control: &rightkit_qa::control::Control) {
    let value = control
        .eval(
            "return (() => { const grid = document.querySelector('.lc-grid-window'); const images = [...document.querySelectorAll('.lc-photo-preview')]; const active = document.querySelectorAll('.lc-photo-cell.is-active'); return {cells: grid?.querySelectorAll('.lc-photo-cell').length || 0, images: images.length, active: active.length, loaded: images.filter((img) => img.complete && img.naturalWidth > 0 && img.naturalHeight > 0).length}; })();",
        )
        .expect("bounded grid DOM query must execute");
    assert!(value["cells"].as_u64().is_some_and(|count| count <= 512), "virtualized grid rendered too many cells: {value}");
    assert!(value["images"].as_u64().is_some_and(|count| count <= 512), "virtualized grid rendered too many images: {value}");
    assert_eq!(value["active"].as_u64(), Some(1), "grid must expose exactly one active photo: {value}");
    assert!(value["loaded"].as_u64().is_some_and(|count| count > 0), "grid must expose decoded WebView pixels: {value}");
}

fn grid_metrics(control: &rightkit_qa::control::Control) -> Value {
    control
        .eval(
            "return (() => { const scroll = document.querySelector('.lc-grid-scroll'); const grid = document.querySelector('.lc-grid-window'); const images = [...(grid?.querySelectorAll('.lc-photo-preview') || [])]; const first = grid?.querySelector('.lc-photo-caption span:first-child')?.textContent?.trim() || ''; return {scrollTop: scroll?.scrollTop || 0, scrollHeight: scroll?.scrollHeight || 0, clientHeight: scroll?.clientHeight || 0, top: Number.parseFloat(grid?.style.top || '0') || 0, cells: grid?.querySelectorAll('.lc-photo-cell').length || 0, images: images.length, loaded: images.filter((img) => img.complete && img.naturalWidth > 0 && img.naturalHeight > 0).length, first, busy: grid?.getAttribute('aria-busy') === 'true'}; })();",
        )
        .expect("scalable grid DOM query must execute")
}

fn wait_for_grid(control: &rightkit_qa::control::Control, scrolled: bool, previous_first: Option<&str>) -> Value {
    for _ in 0..200 {
        let value = grid_metrics(control);
        let visible = value["cells"].as_u64().is_some_and(|count| count <= 512)
            && value["images"].as_u64().is_some_and(|count| count <= 512)
            && value["loaded"].as_u64().is_some_and(|count| count > 0)
            && value["busy"].as_bool() == Some(false)
            && previous_first.is_none_or(|previous| value["first"].as_str() != Some(previous))
            && (!scrolled || (value["scrollTop"].as_f64().unwrap_or(0.0) > 0.0 && value["top"].as_f64().unwrap_or(0.0) > 0.0));
        if visible {
            return value;
        }
        sleep(Duration::from_millis(50));
    }
    panic!("scalable grid did not settle: {:?}", grid_metrics(control));
}

fn enable_grid_info(control: &rightkit_qa::control::Control) {
    let clicked = control
        .eval("return (() => { const button = [...document.querySelectorAll('.lc-footer-actions button')].find((item) => item.textContent?.includes('Grid info')); if (!button) return false; button.click(); return true; })();")
        .expect("grid info control query must execute");
    assert_eq!(clicked.as_bool(), Some(true), "grid info control must exist");
    wait_for_dom(control, "return Boolean(document.querySelector('.lc-grid-window .lc-photo-caption span:first-child')?.textContent?.trim());");
}

fn scroll_grid_to_end(control: &rightkit_qa::control::Control) {
    let changed = control
        .eval("return (() => { const scroll = document.querySelector('.lc-grid-scroll'); if (!scroll) return false; scroll.scrollTop = Math.max(0, scroll.scrollHeight - scroll.clientHeight); scroll.dispatchEvent(new Event('scroll', {bubbles: true})); return scroll.scrollHeight > scroll.clientHeight; })();")
        .expect("grid scroll command must execute");
    assert_eq!(changed.as_bool(), Some(true), "scalable grid must have scrollable height");
}

fn assert_active_grid_identity(control: &rightkit_qa::control::Control, file_name: &str) {
    let value = control
        .eval("return document.querySelector('.lc-photo-cell.is-active .lc-photo-caption span:first-child')?.textContent?.trim() || '';")
        .expect("active grid identity query must execute");
    assert_eq!(value.as_str(), Some(file_name), "active grid DOM identity must match selected source");
}

fn wait_task(control: &rightkit_qa::control::Control, task_id: &str) -> Value {
    for _ in 0..900 {
        let value = snapshot(control);
        let running = value["status"]["jobs"].as_array().is_some_and(|jobs| jobs.iter().any(|job| job["id"].as_str() == Some(task_id)));
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

fn assert_png_pixels(path: &Path, expected_width: u32, expected_height: u32) {
    let bytes = fs::read(path).expect("exported PNG must be readable");
    let decoder = png::Decoder::new(Cursor::new(bytes));
    let mut reader = decoder.read_info().expect("exported PNG must decode");
    let mut pixels = vec![0; reader.output_buffer_size().expect("PNG output buffer size must be available")];
    let info = reader.next_frame(&mut pixels).expect("exported PNG frame must decode");
    assert_eq!(info.width, expected_width, "exported PNG width must match requested long edge");
    assert_eq!(info.height, expected_height, "exported PNG height must match source aspect");
    assert!(info.buffer_size() > 0, "exported PNG must contain decoded pixels");
    assert!(pixels[..info.buffer_size()].iter().any(|&pixel| pixel != 0), "exported PNG pixels must not be all zero");
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

fn copy_tree(source: &Path, destination: &Path) {
    fs::create_dir_all(destination).expect("copy destination must be created");
    let entries = fs::read_dir(source).expect("copy source must be readable").collect::<Result<Vec<_>, _>>().expect("copy entries must be readable");
    for entry in entries {
        let from = entry.path();
        let to = destination.join(entry.file_name());
        if from.is_dir() {
            copy_tree(&from, &to);
        } else {
            fs::copy(&from, &to).unwrap_or_else(|error| panic!("copy {}: {error}", from.display()));
        }
    }
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

fn run_catalog_recovery(
    binary: &Path,
    scenario: &rightkit_qa::harness::Scenario,
    catalog: &Path,
    png: &Path,
    baseline_capture: &Path,
    source_revision: &str,
    installed_hash: &str,
) {
    let (before_counts, active, expected_exposure, backup, backup_fingerprint) = with_control(binary, scenario, catalog, |control, _data| {
        let before = import_file(control, png);
        let before_counts = before["counts"].clone();
        let active = before["active"].as_u64().expect("import must select active photo");
        run(control, "develop.beginInteraction", json!({"label": "Backup baseline edit"}));
        run(control, "develop.set", json!({"control": "light.exposure", "value": 1.25, "ids": [active]}));
        run(control, "develop.endInteraction", json!({}));
        let edited = snapshot(control);
        let expected_exposure = edited["develop"]["light"]["exposure"].as_f64().expect("edited exposure must be numeric");
        let backup = scenario.dir().join("library-backup");
        let backup_result = run(control, "library.backup", json!({"path": backup}));
        assert!(backup_result["path"].as_str().is_some(), "backup must return destination: {backup_result}");
        assert!(backup.is_dir(), "backup directory must exist");
        let backup_fingerprint = tree_fingerprint(&backup);
        (before_counts, active, expected_exposure, backup, backup_fingerprint)
    });

    with_control(binary, scenario, catalog, |control, _data| {
        let current = snapshot(control);
        assert_eq!(current["counts"]["catalog"].as_u64(), before_counts["catalog"].as_u64(), "current catalog must reopen before mutation");
        let current_active = current["active"].as_u64().expect("reopened catalog must select active photo");
        run(control, "develop.beginInteraction", json!({"label": "Mutate before restore"}));
        run(control, "develop.set", json!({"control": "light.exposure", "value": -1.25, "ids": [current_active]}));
        run(control, "develop.endInteraction", json!({}));
        let mutated = snapshot(control);
        assert_ne!(mutated["develop"]["light"]["exposure"].as_f64(), Some(expected_exposure), "current catalog must differ before restore");
    });
    assert_eq!(tree_fingerprint(&backup), backup_fingerprint, "backup must remain unchanged while current catalog is mutated");

    with_control(binary, scenario, catalog, |control, _data| {
        let restored = run(control, "library.restore", json!({"path": backup}));
        let backup_text = backup.to_string_lossy().to_string();
        let source_backup = restored["sourceBackup"].as_str().expect("restore must report original source backup");
        let restored_path = restored["restoredPath"].as_str().expect("restore must report copied restored path");
        assert_eq!(source_backup, backup_text, "restore sourceBackup must identify original backup");
        assert_eq!(restored["libraryPath"].as_str(), Some(restored_path), "restore libraryPath must identify copied restored path");
        assert_ne!(source_backup, restored_path, "restore must open a fresh owned copy");
        assert_eq!(tree_fingerprint(&backup), backup_fingerprint, "backup must remain unchanged immediately after restore");
        let after = snapshot(control);
        assert_eq!(after["counts"], before_counts, "restore must recover catalog counts");
        assert_eq!(after["active"].as_u64(), Some(active), "restore must recover active photo identity");
        assert_eq!(after["develop"]["light"]["exposure"].as_f64(), Some(expected_exposure), "restore must recover edited photo state");

        let open_library = scenario.dir().join("open-library");
        copy_tree(&backup, &open_library);
        let generation_before_open = after["viewGeneration"].as_u64().expect("restore generation must be numeric");
        let opened = control
            .command("lc_native", &json!({"action": "openLibrary", "params": {"path": open_library}}))
            .expect("native openLibrary command must execute");
        let open_library_text = open_library.to_string_lossy().to_string();
        assert_eq!(opened["path"].as_str(), Some(open_library_text.as_str()), "openLibrary must report copied path");
        let opened_snapshot = snapshot(control);
        assert_eq!(opened_snapshot["libraryPath"].as_str(), Some(open_library_text.as_str()), "snapshot must expose newly opened library path");
        assert!(
            opened_snapshot["viewGeneration"].as_u64().is_some_and(|generation| generation > generation_before_open),
            "openLibrary must advance view generation"
        );
        control.key("G").expect("library route key must execute after openLibrary");
        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
        wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", None);
        assert_active_grid_identity(control, "procedural-rgb-01.png");
        assert_eq!(tree_fingerprint(&backup), backup_fingerprint, "original backup must remain unchanged after copied openLibrary journey");
        control.screenshot_to(baseline_capture).expect("catalog recovery screenshot must be captured");
        assert!(baseline_capture.is_file());
        let receipt = scenario.dir().join("catalog-recovery.json");
        fs::write(
            &receipt,
            serde_json::to_vec_pretty(&json!({
                "schema": 2,
                "sourceRevision": source_revision,
                "installedArtifactSha256": installed_hash,
                "catalogRecovery": {"qualified": true, "method": "backup-mutated-catalog-restore"},
                "installerRollback": {"qualified": false, "requiresSeparateEvidence": true},
                "beforeCounts": before_counts,
                "afterCounts": after["counts"],
                "backupFingerprint": backup_fingerprint,
                "sourceBackup": source_backup,
                "restoredLibraryPath": restored_path,
                "openLibraryPath": opened_snapshot["libraryPath"],
                "expectedExposure": expected_exposure,
            }))
            .expect("catalog recovery receipt must serialize"),
        )
        .expect("catalog recovery receipt must be writable");
        scenario.keep("catalog-recovery.json", &receipt);
    });
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
    assert_eq!(installed_hash.len(), 64, "installed artifact hash must be SHA-256");
    assert!(installed_hash.bytes().all(|byte| byte.is_ascii_hexdigit()), "installed artifact hash must be hexadecimal");
    assert_eq!(source_hash(&binary), installed_hash, "installed binary must match admitted artifact hash");
    let platform = if cfg!(target_os = "macos") { "macos" } else { "windows" };
    let baseline: Value =
        serde_json::from_str(&fs::read_to_string(fixture_root().join("installed-baseline.json")).expect("baseline fixture must be readable"))
            .expect("baseline fixture must be valid JSON");
    assert_eq!(baseline["catalogRecovery"]["required"].as_bool(), Some(true));
    assert_eq!(baseline["installerRollback"]["qualified"].as_bool(), Some(false));
    let input_dir = evidence.join("fixture-inputs");
    fs::create_dir_all(&input_dir).expect("fixture input directory must exist");
    let inputs = fixture_inputs::write_fixture_inputs(&input_dir).expect("real ARW/PNG/Lightroom fixtures must be generated");
    let source_paths = [
        ("arw", inputs.arw.as_path()),
        ("png", inputs.png.as_path()),
        ("catalog", inputs.catalog.as_path()),
        ("scalabilitySources", inputs.scalability_sources.as_path()),
    ];
    let source_hashes = source_paths.map(|(label, path)| (label, source_fingerprint(path)));
    let source_manifest = input_dir.join("manifest.json");
    fs::write(
        &source_manifest,
        serde_json::to_vec_pretty(&json!({
            "schema": 2,
            "generator": "fixture_inputs:v2",
            "sourceRevision": revision,
            "installedArtifactSha256": installed_hash,
            "platform": platform,
            "architecture": architecture,
            "inputs": source_hashes.iter().map(|(label, hash)| json!({"label": label, "sha256": hash})).collect::<Vec<_>>(),
        }))
        .expect("source manifest must serialize"),
    )
    .expect("source manifest must be writable");
    let harness = qa_harness(&binary, &evidence, &revision, platform, &architecture);

    let scenario_names = [
        "ipc",
        // Capture core import/edit/undo/export proof before broader parity journeys.
        "engineExport",
        "stalePreview",
        "cache",
        "scalability",
        "preferences",
        "gesture",
        "editingTools",
        "arwImport",
        "lightroomImport",
        "catalogRecovery",
    ];
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
            } else if name == "catalogRecovery" {
                run_catalog_recovery(&binary, scenario, &inputs.catalog, &inputs.png, &baseline_capture, &revision, &installed_hash);
            } else {
                with_control(&binary, scenario, &inputs.catalog, |control, _data| match name {
                    "ipc" => {
                        let health = control.health().expect("health RPC must reply");
                        assert!(health.is_object(), "health RPC must return object");
                        let snapshot = control.command("lc_snapshot", &Value::Null).expect("snapshot command must reply");
                        assert_eq!(snapshot["version"].as_u64(), Some(1));
                        assert!(snapshot["viewGeneration"].is_number());
                        assert!(snapshot["controls"].is_array());
                        for (width, height) in [(1280_u64, 800_u64), (1600_u64, 1000_u64)] {
                            set_native_viewport(control, width, height);
                            wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                            assert_layout_settled(control, ".lc-library-workspace");
                            let viewport = control
                                .eval("return {width: window.innerWidth, height: window.innerHeight};")
                                .expect("viewport query must execute");
                            assert_eq!(viewport["width"].as_u64(), Some(width));
                            assert_eq!(viewport["height"].as_u64(), Some(height));
                            let library = scenario.dir().join(format!("route-library-{width}x{height}.png"));
                            control.screenshot_to(&library).expect("library route screenshot must be captured");
                            assert!(library.is_file());
                            control.key("D").expect("develop route key must execute");
                            wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                            assert_layout_settled(control, ".stage-workspace.stage-detail");
                            let route = control
                                .eval("return document.querySelector('.stage-route-tabs button.selected span')?.textContent?.trim() || '';")
                                .expect("develop route readback must execute");
                            assert_eq!(route.as_str(), Some("Detail"), "D must select Detail route");
                            let develop = scenario.dir().join(format!("route-develop-{width}x{height}.png"));
                            control.screenshot_to(&develop).expect("develop route screenshot must be captured");
                            assert!(develop.is_file());
                            control.key("G").expect("library route key must execute");
                            wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        }
                    }
                    "stalePreview" => {
                        let first = import_file(control, &inputs.png);
                        control.key("G").expect("library route key must execute");
                        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        let first_grid = wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", None);
                        assert_active_grid_identity(control, "procedural-rgb-01.png");
                        let first_src = first_grid["src"].as_str().expect("first grid preview must expose scoped source").to_string();
                        let first_id = first["active"].as_u64().expect("first import must select active photo");
                        let second = import_file(control, &inputs.arw);
                        let second_generation = second["viewGeneration"].as_u64().expect("second import generation required");
                        let second_grid = wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", Some(&first_src));
                        assert_active_grid_identity(control, "synthetic-sonya-01.arw");
                        let second_src = second_grid["src"].as_str().expect("second grid preview must expose scoped source").to_string();
                        assert_ne!(first_src, second_src, "second import must replace active grid pixels");
                        let second_id = second["active"].as_u64().expect("second import must select active photo");
                        let selected_first = run(control, "library.select", json!({"ids": [first_id], "active": first_id, "mode": "replace"}));
                        assert_eq!(selected_first["active"].as_u64(), Some(first_id), "selection must return first active photo");
                        let first_again = snapshot(control);
                        assert!(first_again["viewGeneration"].as_u64().is_some_and(|generation| generation > second_generation), "selection must advance view generation");
                        let first_again_grid = wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", Some(&second_src));
                        assert_active_grid_identity(control, "procedural-rgb-01.png");
                        assert_ne!(first_again_grid["src"].as_str(), Some(second_src.as_str()), "quick selection must replace second photo pixels");
                        control.key("D").expect("develop route key must execute");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        let first_stage = wait_for_rendered_preview(control, "img.stage-preview", None);
                        let first_stage_src = first_stage["src"].as_str().expect("first stage preview must expose scoped source").to_string();
                        let selected_second = run(control, "library.select", json!({"ids": [second_id], "active": second_id, "mode": "replace"}));
                        assert_eq!(selected_second["active"].as_u64(), Some(second_id), "selection must return second active photo");
                        let second_again = snapshot(control);
                        assert!(second_again["viewGeneration"].as_u64().is_some_and(|generation| generation > first_again["viewGeneration"].as_u64().unwrap_or(0)), "second selection must advance view generation");
                        let second_stage = wait_for_rendered_preview(control, "img.stage-preview", Some(&first_stage_src));
                        assert_ne!(second_stage["src"].as_str(), Some(first_stage_src.as_str()), "quick selection must never leave first photo in stage");
                        control.key("G").expect("library route key must execute");
                        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", None);
                        assert_active_grid_identity(control, "synthetic-sonya-01.arw");
                        let stale = control
                            .command("lc_view_slice", &json!({"generation": second_again["viewGeneration"].as_u64().unwrap_or(0) + 1, "offset": 0, "limit": 512}))
                            .expect("slice must reply");
                        assert_eq!(stale["generationChanged"].as_bool(), Some(true), "stale generation must be rejected");
                    }
                    "cache" => {
                        let imported = import_file(control, &inputs.png);
                        control.key("G").expect("library route key must execute");
                        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", None);
                        assert_active_grid_identity(control, "procedural-rgb-01.png");
                        let slice = control
                            .command("lc_view_slice", &json!({"generation": imported["viewGeneration"], "offset": 0, "limit": 4096}))
                            .expect("bounded slice must reply");
                        assert!(slice["photos"].as_array().is_some_and(|photos| photos.len() <= 512));
                        assert_eq!(slice["generation"], imported["viewGeneration"]);
                        let dom = control.dom(".lc-grid-scroll").expect("grid DOM query must execute");
                        assert!(!dom.is_empty(), "library grid must exist in hidden WebView");
                        assert_active_grid_is_bounded(control);
                    }
                    "scalability" => {
                        let opened = control
                            .command("lc_native", &json!({"action": "openLibrary", "params": {"path": inputs.scalability_library}}))
                            .expect("native scalability openLibrary command must execute");
                        let library_path = inputs.scalability_library.to_string_lossy().to_string();
                        assert_eq!(opened["path"].as_str(), Some(library_path.as_str()), "scalability openLibrary must report generated path");
                        let opened_snapshot = snapshot(control);
                        assert_eq!(opened_snapshot["libraryPath"].as_str(), Some(library_path.as_str()), "snapshot must expose scalability library path");
                        assert_eq!(opened_snapshot["counts"]["catalog"].as_u64(), Some(fixture_inputs::SCALABILITY_PHOTO_COUNT as u64));
                        assert_eq!(opened_snapshot["total"].as_u64(), Some(fixture_inputs::SCALABILITY_PHOTO_COUNT as u64));
                        let generation = opened_snapshot["viewGeneration"].as_u64().expect("scalability generation must be numeric");
                        let first_page = control
                            .command("lc_view_slice", &json!({"generation": generation, "offset": 0, "limit": 4096}))
                            .expect("scalability first page must reply");
                        assert_eq!(first_page["total"].as_u64(), Some(fixture_inputs::SCALABILITY_PHOTO_COUNT as u64));
                        assert_eq!(first_page["offset"].as_u64(), Some(0));
                        assert_eq!(first_page["photos"].as_array().map(Vec::len), Some(512), "first page must remain capped at 512 photos");
                        let tail_page = control
                            .command("lc_view_slice", &json!({"generation": generation, "offset": 1536, "limit": 512}))
                            .expect("scalability tail page must reply");
                        assert_eq!(tail_page["offset"].as_u64(), Some(1536));
                        assert_eq!(tail_page["photos"].as_array().map(Vec::len), Some(512), "tail page must contain final 512 photos");
                        let first_id = first_page["photos"][0]["id"].as_u64().expect("first page must expose photo id");
                        let selected = run(control, "library.select", json!({"ids": [first_id], "active": first_id, "mode": "replace"}));
                        assert_eq!(selected["active"].as_u64(), Some(first_id), "scalability selection must identify first page photo");
                        control.key("G").expect("library route key must execute for scalability journey");
                        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        assert_layout_settled(control, ".lc-grid-scroll");
                        enable_grid_info(control);
                        let initial = wait_for_grid(control, false, None);
                        let initial_name = initial["first"].as_str().unwrap_or_default().to_string();
                        assert!(!initial_name.is_empty(), "initial grid must expose first filename");
                        assert!(initial["scrollHeight"].as_f64().unwrap_or(0.0) > initial["clientHeight"].as_f64().unwrap_or(0.0), "2048-photo grid must have scrollable height: {initial}");
                        assert_active_grid_is_bounded(control);
                        scroll_grid_to_end(control);
                        let far = wait_for_grid(control, true, Some(initial_name.as_str()));
                        assert!(far["scrollHeight"].as_f64().unwrap_or(0.0) > far["clientHeight"].as_f64().unwrap_or(0.0), "far grid viewport must preserve scrollable height: {far}");
                        assert!(far["cells"].as_u64().is_some_and(|count| count <= 512), "far grid rendered too many cells: {far}");
                        assert!(far["images"].as_u64().is_some_and(|count| count <= 512), "far grid rendered too many images: {far}");
                        assert_ne!(far["first"].as_str(), Some(initial_name.as_str()), "far grid must render a different page");
                        control
                            .eval("return (() => { const scroll = document.querySelector('.lc-grid-scroll'); if (!scroll) return false; scroll.scrollTop = 0; scroll.dispatchEvent(new Event('scroll', {bubbles: true})); return true; })();")
                            .expect("grid reset scroll command must execute");
                        wait_for_dom(control, "return Number.parseFloat(document.querySelector('.lc-grid-window')?.style.top || '0') === 0;");
                        let _descending = run(control, "library.sort", json!({"key": "fileName", "ascending": false}));
                        let descending_before = snapshot(control);
                        let descending_generation = descending_before["viewGeneration"].as_u64().expect("descending generation must be numeric");
                        assert!(descending_generation > generation, "descending sort must advance view generation");
                        wait_for_dom(control, "return Boolean(document.querySelector('.lc-grid-window .lc-photo-caption span:first-child')?.textContent?.trim());");
                        let descending_name = control
                            .eval("return document.querySelector('.lc-grid-window .lc-photo-caption span:first-child')?.textContent?.trim() || '';")
                            .expect("descending sort filename query must execute")
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        assert!(!descending_name.is_empty(), "descending sort must expose first filename");
                        let _ascending = run(control, "library.sort", json!({"key": "fileName", "ascending": true}));
                        let ascending_snapshot = snapshot(control);
                        let ascending_generation = ascending_snapshot["viewGeneration"].as_u64().expect("ascending generation must be numeric");
                        assert!(ascending_generation > descending_generation, "ascending sort must advance view generation");
                        wait_for_dom(control, &format!("return (document.querySelector('.lc-grid-window .lc-photo-caption span:first-child')?.textContent?.trim() || '') !== {descending_name:?};"));
                        let ascending_name = control
                            .eval("return document.querySelector('.lc-grid-window .lc-photo-caption span:first-child')?.textContent?.trim() || '';")
                            .expect("ascending sort filename query must execute")
                            .as_str()
                            .unwrap_or_default()
                            .to_string();
                        assert!(!ascending_name.is_empty(), "ascending sort must expose first filename");
                        assert_ne!(ascending_name, descending_name, "generation change must replace cached first page");
                        let stale = control
                            .command("lc_view_slice", &json!({"generation": descending_generation, "offset": 1536, "limit": 4096}))
                            .expect("stale scalability slice must reply");
                        assert_eq!(stale["generationChanged"].as_bool(), Some(true), "stale scalability generation must be rejected");
                        assert_eq!(stale["generation"].as_u64(), Some(ascending_generation));
                        let fresh = control
                            .command("lc_view_slice", &json!({"generation": ascending_generation, "offset": 1536, "limit": 512}))
                            .expect("fresh scalability tail must reply");
                        assert_eq!(fresh["generationChanged"].as_bool(), None);
                        assert_eq!(fresh["photos"].as_array().map(Vec::len), Some(512), "fresh tail page must remain capped at 512 photos");
                    }
                    "gesture" => {
                        let imported = import_file(control, &inputs.png);
                        let before = imported["undo"].as_u64().expect("snapshot undo count required");
                        control.key("D").expect("develop route key must execute");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        click_dom(control, ".stage-toolstrip button[aria-label='Edit']", "edit tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector("input[aria-label='Exposure']") !== null;"#);
                        let slider = control
                            .eval(r#"return (() => { const e = document.querySelector("input[aria-label='Exposure']"); const r = e.getBoundingClientRect(); return {x:r.x, y:r.y, width:r.width, height:r.height}; })();"#)
                            .expect("exposure slider geometry query must execute");
                        let x = slider["x"].as_f64().expect("exposure slider x must be numeric");
                        let y = slider["y"].as_f64().expect("exposure slider y must be numeric") + slider["height"].as_f64().unwrap_or(16.0) / 2.0;
                        let width = slider["width"].as_f64().expect("exposure slider width must be numeric");
                        control.drag((x + width * 0.45, y), (x + width * 0.7, y), 8).expect("exposure slider drag must execute");
                        let edited = wait_for_snapshot(
                            control,
                            |value| {
                                value["develop"]["light"]["exposure"] != imported["develop"]["light"]["exposure"]
                                    && value["undo"].as_u64().is_some_and(|undo| undo > before)
                            },
                            "pointer drag edit did not settle through host bridge",
                        );
                        assert_ne!(edited["develop"]["light"]["exposure"], imported["develop"]["light"]["exposure"], "pointer drag must change exposure through UI");
                        assert_eq!(edited["undo"].as_u64(), Some(before + 1), "one pointer gesture must create one undo step");
                        let edited_exposure = edited["develop"]["light"]["exposure"].clone();
                        let focused = control
                            .eval(r#"return (() => { const node = document.querySelector("input[aria-label='Exposure']"); if (!node) return false; node.focus(); return document.activeElement === node; })();"#)
                            .expect("exposure slider focus query must execute");
                        assert_eq!(focused.as_bool(), Some(true), "exposure slider focus must execute");
                        control.key("ArrowLeft").expect("exposure keyboard adjustment must execute");
                        wait_for_snapshot(
                            control,
                            |value| value["develop"]["light"]["exposure"] != edited_exposure,
                            "keyboard adjustment must reach engine before testing cancellation",
                        );
                        control.key("Escape").expect("exposure Escape cancellation must execute");
                        let cancelled = wait_for_snapshot(
                            control,
                            |value| {
                                value["develop"]["light"]["exposure"] == edited_exposure
                                    && value["undo"].as_u64() == edited["undo"].as_u64()
                            },
                            "Escape cancellation did not settle through host bridge",
                        );
                        assert_eq!(cancelled["develop"]["light"]["exposure"], edited_exposure, "Escape must cancel active slider gesture");
                        assert_eq!(cancelled["undo"].as_u64(), edited["undo"].as_u64(), "cancelled gesture must not create undo step");
                        control.move_to(300.0, 300.0).expect("pointer move must execute");
                        control.drag((300.0, 300.0), (420.0, 320.0), 8).expect("pointer drag must execute");
                        control.wheel(420.0, 320.0, 0.0, -120.0).expect("wheel must execute");
                        control.key("Escape").expect("Escape must execute");
                        assert!(
                            control
                                .eval("return document.activeElement !== null;")
                                .expect("active element query must execute")
                                .as_bool()
                                .unwrap_or(false)
                        );
                    }
                    "editingTools" => {
                        let imported = import_file(control, &inputs.png);
                        let active = imported["active"].as_u64().expect("import must select active photo");
                        let before = snapshot(control);
                        control.key("D").expect("develop route key must execute");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        run(control, "crop.set", json!({"rect": [0.1, 0.1, 0.9, 0.9], "angle": 3.0}));
                        let crop = snapshot(control);
                        assert_ne!(crop["develop"]["crop"], before["develop"]["crop"], "crop command must change crop geometry");
                        click_dom(control, ".stage-toolstrip button[aria-label='Crop']", "crop tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Crop']")?.classList.contains('selected') === true;"#);
                        control.screenshot_to(&scenario.dir().join("tool-crop.png")).expect("crop tool screenshot must be captured");

                        run(control, "mask.add", json!({"kind": "radial", "center": [0.5, 0.5], "rx": 0.2, "ry": 0.2}));
                        let masked = snapshot(control);
                        assert_eq!(masked["develop"]["masks"].as_array().map(Vec::len), Some(1), "mask command must create mask state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Masking']", "masking tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Masking']")?.classList.contains('selected') === true;"#);
                        control.screenshot_to(&scenario.dir().join("tool-masking.png")).expect("masking tool screenshot must be captured");

                        run(control, "spot.add", json!({"mode": "remove", "points": [[0.5, 0.5]], "size": 0.05, "source": [0.1, 0.0]}));
                        let spotted = snapshot(control);
                        assert_eq!(spotted["develop"]["spots"].as_array().map(Vec::len), Some(1), "remove tool command must create spot state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Remove']", "remove tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Remove']")?.classList.contains('selected') === true;"#);
                        control.screenshot_to(&scenario.dir().join("tool-remove.png")).expect("remove tool screenshot must be captured");

                        run(control, "redeye.add", json!({"center": [0.5, 0.5], "rx": 0.1, "ry": 0.1}));
                        let red_eye = snapshot(control);
                        assert_eq!(red_eye["develop"]["red_eye"].as_array().map(Vec::len), Some(1), "red-eye command must create correction state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Red Eye']", "red-eye tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Red Eye']")?.classList.contains('selected') === true;"#);
                        control.screenshot_to(&scenario.dir().join("tool-red-eye.png")).expect("red-eye tool screenshot must be captured");
                        assert_eq!(red_eye["active"].as_u64(), Some(active));
                    }
                    "arwImport" => {
                        let imported = import_file(control, &inputs.arw);
                        control.key("G").expect("library route key must execute");
                        wait_for_dom(control, "return document.querySelector('.lc-library-workspace') !== null;");
                        wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", None);
                        assert_active_grid_identity(control, "synthetic-sonya-01.arw");
                        let descriptor = preview_imported(control, &imported);
                        control.key("D").expect("develop route key must execute");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        let rendered = wait_for_rendered_preview(control, "img.stage-preview", None);
                        let screenshot = scenario.dir().join("arw-preview.png");
                        control.screenshot_to(&screenshot).expect("ARW preview screenshot must be captured");
                        assert!(screenshot.is_file());
                        assert!(descriptor["width"].as_u64().is_some_and(|width| width > 0));
                        assert!(rendered["naturalWidth"].as_u64().is_some_and(|width| width > 0));
                        assert!(rendered["naturalHeight"].as_u64().is_some_and(|height| height > 0));
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
                    "engineExport" => {
                        let imported = import_file(control, &inputs.png);
                        let id = imported["active"].as_u64().expect("PNG import must select photo");
                        control.key("D").expect("develop route key must execute for engine export journey");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        let decoded_before = wait_for_rendered_preview(control, "img.stage-preview", None);
                        assert!(decoded_before["naturalWidth"].as_u64().is_some_and(|width| width > 0));
                        assert!(decoded_before["naturalHeight"].as_u64().is_some_and(|height| height > 0));
                        let decoded_before_src = decoded_before["src"].as_str().expect("decoded stage preview must expose source handle").to_string();
                        let before = snapshot(control);
                        let before_exposure = before["develop"]["light"]["exposure"].clone();
                        let before_undo = before["undo"].as_u64().expect("engine export journey must expose undo count");
                        run(control, "develop.beginInteraction", json!({"label": "Qualification edit"}));
                        run(control, "develop.set", json!({"control": "light.exposure", "value": 0.75, "ids": [id]}));
                        run(control, "develop.endInteraction", json!({}));
                        let edited = snapshot(control);
                        assert_ne!(edited["develop"]["light"]["exposure"], before_exposure, "develop.set must change exposure state");
                        assert!(edited["undo"].as_u64().is_some_and(|undo| undo > before_undo), "develop.set must create undo state");
                        let decoded_edited = wait_for_rendered_preview(control, "img.stage-preview", Some(&decoded_before_src));
                        let decoded_edited_src = decoded_edited["src"].as_str().expect("edited stage preview must expose source handle").to_string();
                        run(control, "edit.undo", json!({}));
                        let undone = snapshot(control);
                        assert_eq!(undone["develop"]["light"]["exposure"], before_exposure, "edit.undo must restore exposure state");
                        assert_eq!(undone["undo"].as_u64(), Some(before_undo), "edit.undo must remove qualification edit");
                        let decoded_undone = wait_for_rendered_preview(control, "img.stage-preview", Some(&decoded_edited_src));
                        let screenshot = scenario.dir().join("engine-export-stage.png");
                        control.screenshot_to(&screenshot).expect("engine export stage screenshot must be captured");
                        assert!(screenshot.is_file());
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
                        assert_png_pixels(&exported, 96, 64);
                        let receipt = scenario.dir().join("engine-export.json");
                        fs::write(
                            &receipt,
                            serde_json::to_vec_pretty(&json!({
                                "schema": 1,
                                "journey": "import-decoded-preview-edit-undo-export",
                                "photoId": id,
                                "decodedPreviewBefore": decoded_before,
                                "decodedPreviewEdited": decoded_edited,
                                "decodedPreviewUndone": decoded_undone,
                                "beforeExposure": before_exposure.clone(),
                                "editedExposure": edited["develop"]["light"]["exposure"].clone(),
                                "undoRestored": undone["develop"]["light"]["exposure"] == before_exposure,
                                "exportedPng": exported,
                            }))
                            .expect("engine export receipt must serialize"),
                        )
                        .expect("engine export receipt must be writable");
                        scenario.keep("engine-export-stage.png", &screenshot);
                        scenario.keep("engine-export.json", &receipt);
                    }
                    _ => unreachable!("scenario inventory is static"),
                });
            }
            scenario.note(format!("executed hidden {platform} control journey: {name}"));
        });
        assert!(!outcome.is_skipped(), "native journey {name} must execute, not skip");
        assert_sources_unchanged(&source_paths, &source_hashes);
    }
}
