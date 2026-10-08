#![cfg(any(target_os = "macos", target_os = "windows"))]

mod fixture_inputs;

use std::fs;
use std::io::Cursor;
use std::panic::{AssertUnwindSafe, catch_unwind, resume_unwind};
use std::path::{Path, PathBuf};
use std::thread::sleep;
use std::time::{Duration, Instant};

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
    let env = ws
        .env
        .iter()
        .filter(|(key, _)| {
            #[cfg(target_os = "macos")]
            {
                key.starts_with("RIGHTKIT_")
            }
            #[cfg(not(target_os = "macos"))]
            {
                let _ = key;
                true
            }
        })
        .map(|(key, value)| (key.clone(), value.clone()))
        .collect::<Vec<_>>();
    #[cfg(target_os = "macos")]
    let launch_binary = binary
        .ancestors()
        .find(|path| path.extension().is_some_and(|extension| extension == "app"))
        .expect("native qualification must launch the SDK-installed app bundle")
        .to_path_buf();
    #[cfg(not(target_os = "macos"))]
    let launch_binary = binary.to_path_buf();
    let spec = LaunchSpec {
        binary: launch_binary,
        mode: Mode::Hidden,
        env: {
            let mut values = env;
            values.push(("RIGHTKIT_QA_CATALOG".into(), catalog.display().to_string()));
            values
        },
        startup_timeout: Duration::from_secs(90),
        label: "lightcraft-desktop-native".into(),
    };
    let startup_started = Instant::now();
    let control = launch(&spec, &ws, scenario.tracker()).expect("hidden native app must expose rightkit-control");
    // Control becomes available before React mounts its command subscriptions.
    let ready = catch_unwind(AssertUnwindSafe(|| {
        // Native control may answer in about:blank before WebView navigation.
        // Renderer readiness shares existing launch deadline, rather than receiving
        // an unrelated five-second interaction timeout.
        wait_for_dom_with_timeout(
            &control,
            "return document.querySelector('.lc-content') !== null;",
            spec.startup_timeout.saturating_sub(startup_started.elapsed()),
        );
    }));
    if let Err(payload) = ready {
        let dom = control.eval("return {url: location.href, readyState: document.readyState, title: document.title, body: document.body?.innerText, root: document.getElementById('root')?.innerHTML, width: innerWidth, height: innerHeight};");
        if let Ok(value) = dom
            && let Ok(bytes) = serde_json::to_vec_pretty(&value)
        {
            let _ = fs::write(scenario.dir().join("startup-dom.json"), bytes);
        }
        let _ = control.screenshot_to(&scenario.dir().join("startup-native.png"));
        resume_unwind(payload);
    }
    let focused = control
        .eval("document.activeElement?.blur(); document.body.tabIndex = -1; document.body.focus(); return document.activeElement === document.body;")
        .expect("native shortcut target must focus after renderer readiness");
    assert_eq!(focused.as_bool(), Some(true), "native shortcuts must target app content");
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
        assert_eq!(source_fingerprint(path), before.as_str(), "native journey mutated source fixture {label}");
    }
}

fn wait_for_dom(control: &rightkit_qa::control::Control, expression: &str) -> Value {
    wait_for_dom_with_timeout(control, expression, Duration::from_secs(5))
}

fn wait_for_dom_with_timeout(control: &rightkit_qa::control::Control, expression: &str, timeout: Duration) -> Value {
    let started = Instant::now();
    loop {
        let value = control.eval(expression).expect("DOM route query must execute");
        if value.as_bool() == Some(true) {
            return value;
        }
        if started.elapsed() >= timeout {
            panic!("DOM route condition did not become true: {expression}; last result={value}; elapsed={:?}", started.elapsed());
        }
        sleep(Duration::from_millis(50));
    }
}

fn assert_library_header_contrast(control: &rightkit_qa::control::Control, mode: &str) {
    let colors = control
        .eval(
            "return (() => { const header = document.querySelector('.lc-library-header'); const title = document.querySelector('.lc-library-title h1'); const actions = Array.from(document.querySelectorAll('.lc-library-header .lc-header-action')).slice(0, 2); if (!header || !title || actions.length < 2) return null; const rgb = (value) => { const match = value.match(/rgba?\\(([^)]+)\\)/); if (!match) return null; return match[1].split(/[,\\s/]+/).slice(0, 3).map(Number); }; const luminance = (value) => { const channels = rgb(value); if (!channels || channels.length < 3 || channels.some((channel) => !Number.isFinite(channel))) return null; return channels.map((channel) => { const normalized = channel / 255; return normalized <= 0.03928 ? normalized / 12.92 : ((normalized + 0.055) / 1.055) ** 2.4; }).reduce((sum, channel, index) => sum + channel * [0.2126, 0.7152, 0.0722][index], 0); }; const background = getComputedStyle(header).backgroundColor; const titleColor = getComputedStyle(title).color; const actionColors = actions.map((node) => getComputedStyle(node).color); const bg = luminance(background); const titleLum = luminance(titleColor); const actionLums = actionColors.map(luminance); const ratio = (foreground) => bg === null || foreground === null ? null : (Math.max(bg, foreground) + 0.05) / (Math.min(bg, foreground) + 0.05); return { background, titleColor, actionColors, titleRatio: ratio(titleLum), actionRatios: actionLums.map(ratio) }; })();",
        )
        .expect("library header contrast query must execute");
    assert!(colors.is_object(), "{mode} library header must render colors");
    let title_ratio = colors["titleRatio"].as_f64().unwrap_or(0.0);
    let action_ratios = colors["actionRatios"].as_array().expect("library Import & Export colors must be reported");
    eprintln!("[qa] {mode} library header contrast: {colors}");
    assert!(title_ratio >= 4.5, "{mode} library title contrast must meet WCAG AA: {colors}");
    assert!(
        action_ratios.len() >= 2 && action_ratios.iter().all(|ratio| ratio.as_f64().is_some_and(|value| value >= 4.5)),
        "{mode} Import & Export contrast must meet WCAG AA: {colors}"
    );
}

fn click_dom(control: &rightkit_qa::control::Control, selector: &str, message: &str) {
    let geometry = control
        .eval(&format!(
            "return (() => {{ const node = document.querySelector({selector:?}); if (!node) return null; const rect = node.getBoundingClientRect(); const x = rect.x + rect.width / 2; const y = rect.y + rect.height / 2; const hit = document.elementFromPoint(x, y); const describe = (value) => value ? {{ tag: value.tagName, aria: value.getAttribute('aria-label'), title: value.getAttribute('title'), className: value.className }} : null; window.__rkClickDiagnostic = []; const capture = (event) => {{ const target = event.target && event.target.closest ? event.target.closest('button,[role=button]') : event.target; if (window.__rkClickDiagnostic.length < 64) window.__rkClickDiagnostic.push({{ type: event.type, isTrusted: event.isTrusted, clientX: event.clientX, clientY: event.clientY, target: describe(target), selected: target?.classList?.contains('selected') === true }}); }}; window.__rkClickDiagnosticCleanup = () => {{ ['pointerdown', 'pointerup', 'click'].forEach((type) => document.removeEventListener(type, capture, true)); }}; ['pointerdown', 'pointerup', 'click'].forEach((type) => document.addEventListener(type, capture, true)); return {{ x, y, rect: {{ x: rect.x, y: rect.y, width: rect.width, height: rect.height }}, target: describe(node), hit: describe(hit), viewport: {{ width: window.innerWidth, height: window.innerHeight, scale: window.devicePixelRatio }} }}; }})();"
        ))
        .expect("DOM click geometry query must execute");
    let x = geometry["x"].as_f64().expect("DOM click x must be numeric");
    let y = geometry["y"].as_f64().expect("DOM click y must be numeric");
    let result = control.click(x, y, "left", 1);
    let events = control
        .eval("return (() => { const events = window.__rkClickDiagnostic || []; if (window.__rkClickDiagnosticCleanup) window.__rkClickDiagnosticCleanup(); delete window.__rkClickDiagnostic; delete window.__rkClickDiagnosticCleanup; return events; })();")
        .unwrap_or(Value::Null);
    eprintln!("[qa] click selector={selector:?} geometry={geometry} events={events}");
    result.expect(message);
}

fn set_native_viewport(control: &rightkit_qa::control::Control, width: u64, height: u64) -> Value {
    let result = control.command("lc_qa_viewport", &json!({"width": width, "height": height})).expect("QA viewport resize command must execute");
    assert_eq!(result["requested"]["width"].as_u64(), Some(width), "QA viewport must report requested width");
    assert_eq!(result["requested"]["height"].as_u64(), Some(height), "QA viewport must report requested height");
    assert!(result["observedInnerSize"]["width"].as_u64().is_some_and(|value| value > 0), "QA viewport must report observed width: {result}");
    assert!(result["observedInnerSize"]["height"].as_u64().is_some_and(|value| value > 0), "QA viewport must report observed height: {result}");
    let dom_size = control
        .eval("return {width: window.innerWidth, height: window.innerHeight, scale: window.devicePixelRatio};")
        .expect("WebView viewport diagnostics must execute");
    eprintln!("[qa] viewport native={result}; DOM={dom_size}");
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
    let mut last = Value::Null;
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
        last = value;
        sleep(Duration::from_millis(50));
    }
    panic!("rendered preview did not become ready for selector {selector}: {expression}; last result={last}");
}

// Tool fixture has varied RGB pixels. A decoded DOM image can still be absent
// from native compositor output; inspect actual PNG region before accepting it.
fn capture_visible_tool_preview(control: &rightkit_qa::control::Control, path: &Path) {
    let mut last = Value::Null;
    for _ in 0..40 {
        let geometry = control.eval("return (() => { const img = document.querySelector('img.stage-preview'); if (!img) return null; const r = img.getBoundingClientRect(); const css = getComputedStyle(img); const hit = document.elementFromPoint(r.x + r.width / 2, r.y + r.height / 2); return {src: img.src, complete: img.complete, naturalWidth: img.naturalWidth, naturalHeight: img.naturalHeight, rect: {x:r.x,y:r.y,width:r.width,height:r.height}, viewport:{width:innerWidth,height:innerHeight}, display:css.display, visibility:css.visibility, opacity:css.opacity, hit:hit?.tagName, hitClass:hit?.getAttribute('class')}; })();").expect("tool preview geometry must execute");
        control.screenshot_to(path).expect("tool preview screenshot must be captured");
        let mut decoder = png::Decoder::new(Cursor::new(fs::read(path).expect("tool PNG must be readable")));
        decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
        let mut reader = decoder.read_info().expect("tool PNG must decode");
        let mut pixels = vec![0; reader.output_buffer_size().expect("tool PNG size must exist")];
        let info = reader.next_frame(&mut pixels).expect("tool PNG pixels must decode");
        let channels = info.color_type.samples();
        let viewport_width = geometry["viewport"]["width"].as_f64().unwrap_or(0.0);
        let viewport_height = geometry["viewport"]["height"].as_f64().unwrap_or(0.0);
        let rect = &geometry["rect"];
        let mut colors = std::collections::BTreeSet::new();
        if viewport_width > 0.0
            && viewport_height > 0.0
            && rect["width"].as_f64().unwrap_or(0.0) > 0.0
            && rect["height"].as_f64().unwrap_or(0.0) > 0.0
        {
            for row in 0..25 {
                for col in 0..25 {
                    let x = rect["x"].as_f64().unwrap_or(0.0) + rect["width"].as_f64().unwrap_or(0.0) * (0.25 + f64::from(col) / 48.0);
                    let y = rect["y"].as_f64().unwrap_or(0.0) + rect["height"].as_f64().unwrap_or(0.0) * (0.25 + f64::from(row) / 48.0);
                    if !(0.0..viewport_width).contains(&x) || !(0.0..viewport_height).contains(&y) {
                        continue;
                    }
                    let px = (x / viewport_width * f64::from(info.width)).floor() as usize;
                    let py = (y / viewport_height * f64::from(info.height)).floor() as usize;
                    let offset = (py * info.width as usize + px) * channels;
                    let rgb = match info.color_type {
                        png::ColorType::Rgb | png::ColorType::Rgba => [pixels[offset], pixels[offset + 1], pixels[offset + 2]],
                        png::ColorType::Grayscale | png::ColorType::GrayscaleAlpha => [pixels[offset]; 3],
                        png::ColorType::Indexed => panic!("expanded tool PNG must not be indexed"),
                    };
                    colors.insert(rgb);
                }
            }
        }
        let visible = geometry["complete"].as_bool() == Some(true)
            && geometry["naturalWidth"].as_u64().is_some_and(|width| width > 0)
            && geometry["visibility"].as_str() == Some("visible")
            && geometry["display"].as_str() != Some("none")
            && geometry["opacity"].as_str().and_then(|value| value.parse::<f64>().ok()).is_some_and(|opacity| opacity > 0.0)
            && colors.len() >= 16;
        last = json!({"dom": geometry, "sampledRgbColors": colors.len(), "visible": visible});
        fs::write(path.with_extension("json"), serde_json::to_vec_pretty(&last).expect("tool receipt must serialize"))
            .expect("tool receipt must save");
        if visible {
            return;
        }
        sleep(Duration::from_millis(100));
    }
    panic!("tool preview never became visible in actual native screenshot: {last}");
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
        if let Some(completed) =
            value["status"]["completedJobs"].as_array().and_then(|jobs| jobs.iter().find(|job| job["id"].as_str() == Some(task_id)))
        {
            match completed["state"].as_str() {
                Some("done") => return value,
                Some("cancelled") => panic!("native task {task_id} was cancelled: {completed}"),
                Some("failed") => panic!("native task {task_id} failed: {completed}"),
                state => panic!("native task {task_id} has unknown terminal state {state:?}: {completed}"),
            }
        }
        sleep(Duration::from_millis(100));
    }
    panic!("native task {task_id} did not publish exact completedJobs receipt within 90 seconds");
}

fn task_result(control: &rightkit_qa::control::Control, id: &str, params: Value) -> Value {
    let started = run(control, id, params);
    let task_id = started["taskId"].as_str().expect("background command must return task id");
    let state = wait_task(control, task_id);
    state["status"]["completedJobs"]
        .as_array()
        .expect("completed task history must exist")
        .iter()
        .find(|job| job["id"].as_str() == Some(task_id))
        .expect("exact task must be retained")["result"]
        .clone()
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

fn write_merge_fixture_copies(root: &Path, source: &Path) -> [PathBuf; 2] {
    let first = root.join("merge-procedural-a.png");
    let second = root.join("merge-procedural-b.png");
    fs::copy(source, &first).unwrap_or_else(|error| panic!("copy first procedural merge PNG: {error}"));

    let mut decoder = png::Decoder::new(Cursor::new(fs::read(source).expect("procedural PNG must be readable")));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().expect("procedural PNG must decode");
    let mut pixels = vec![0; reader.output_buffer_size().expect("procedural PNG buffer must exist")];
    let info = reader.next_frame(&mut pixels).expect("procedural PNG pixels must decode");
    assert_eq!(info.color_type, png::ColorType::Rgba, "procedural merge source must decode to RGBA");
    for pixel in pixels[..info.buffer_size()].as_chunks_mut::<4>().0 {
        pixel.swap(0, 2);
    }
    let mut encoded = Vec::new();
    let mut encoder = png::Encoder::new(&mut encoded, info.width, info.height);
    encoder.set_color(png::ColorType::Rgba);
    encoder.set_depth(png::BitDepth::Eight);
    encoder.set_source_srgb(png::SrgbRenderingIntent::Perceptual);
    let mut writer = encoder.write_header().expect("second merge PNG header must encode");
    writer.write_image_data(&pixels[..info.buffer_size()]).expect("second merge PNG pixels must encode");
    drop(writer);
    fs::write(&second, encoded).expect("second procedural merge PNG must be writable");
    assert_ne!(source_hash(&first), source_hash(&second), "merge inputs must be distinct procedural PNG content");
    [first, second]
}

fn completed_task<'a>(snapshot: &'a Value, task_id: &str) -> &'a Value {
    snapshot["status"]["completedJobs"]
        .as_array()
        .and_then(|jobs| jobs.iter().find(|job| job["id"].as_str() == Some(task_id)))
        .unwrap_or_else(|| panic!("completedJobs must contain exact task {task_id}: {snapshot}"))
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

fn set_dialog_field(control: &rightkit_qa::control::Control, label: &str, value: &str) {
    let label_json = serde_json::to_string(label).expect("field label must serialize");
    let value_json = serde_json::to_string(value).expect("field value must serialize");
    let changed = control.eval(&format!(r#"return (() => {{
        const field = [...document.querySelectorAll('.lc-dialog .lc-field')].find((node) => node.querySelector('span')?.textContent === {label_json});
        const node = field?.querySelector('input,select,textarea');
        if (!node) return false;
        const prototype = node instanceof HTMLSelectElement ? HTMLSelectElement.prototype : node instanceof HTMLTextAreaElement ? HTMLTextAreaElement.prototype : HTMLInputElement.prototype;
        Object.getOwnPropertyDescriptor(prototype, 'value').set.call(node, {value_json});
        node.dispatchEvent(new Event(node instanceof HTMLSelectElement ? 'change' : 'input', {{bubbles:true}}));
        return node.value === {value_json};
    }})();"#)).expect("dialog field input must execute in native WebView");
    assert_eq!(changed.as_bool(), Some(true), "native Export field {label} must accept input");
    wait_for_dom(
        control,
        &format!(
            "return [...document.querySelectorAll('.lc-dialog .lc-field')].find((node) => node.querySelector('span')?.textContent === {label_json})?.querySelector('input,select,textarea')?.value === {value_json};"
        ),
    );
}

fn top_left_magenta_pixels(path: &Path) -> usize {
    let mut decoder = png::Decoder::new(Cursor::new(fs::read(path).expect("watermark PNG must be readable")));
    decoder.set_transformations(png::Transformations::EXPAND | png::Transformations::STRIP_16);
    let mut reader = decoder.read_info().expect("watermark PNG must decode");
    let mut pixels = vec![0; reader.output_buffer_size().expect("watermark buffer must be available")];
    let info = reader.next_frame(&mut pixels).expect("watermark pixels must decode");
    let channels = match info.color_type {
        png::ColorType::Rgb => 3,
        png::ColorType::Rgba => 4,
        _ => panic!("rendered watermark PNG must contain RGB channels"),
    };
    pixels[..info.buffer_size()]
        .chunks_exact(channels)
        .enumerate()
        .filter(|(index, pixel)| {
            let x = *index % info.width as usize;
            let y = *index / info.width as usize;
            x < info.width as usize / 2 && y < info.height as usize / 2 && pixel[0] > 240 && pixel[1] < 20 && pixel[2] > 240
        })
        .count()
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
    if result.is_err() {
        let state = control.eval("return {url: location.href, title: document.title, width: window.innerWidth, height: window.innerHeight, rootChildren: document.getElementById('root')?.childElementCount, activeElement: document.activeElement?.outerHTML.slice(0, 500), preview: document.querySelector('.stage-preview')?.outerHTML, previewStates: Array.from(document.querySelectorAll('[data-preview-state]')).map(node => ({state: node.getAttribute('data-preview-state'), html: node.outerHTML.slice(0, 1000)})), body: document.body.innerText.slice(0, 4000)};");
        eprintln!("[qa] failure DOM={state:?}");
        if let Ok(state) = state {
            let path = scenario.dir().join("failure-dom.json");
            if let Ok(bytes) = serde_json::to_vec_pretty(&state)
                && fs::write(&path, bytes).is_ok()
            {
                scenario.keep("failure-dom.json", &path);
            }
        }
        let path = scenario.dir().join("failure-native.png");
        match control.screenshot_to(&path) {
            Ok(_) => {
                scenario.keep("failure-native.png", &path);
            }
            Err(error) => eprintln!("[qa] failure screenshot unavailable: {error}"),
        }
    }
    let stopped = control.stop().expect("native app must stop cleanly");
    assert!(stopped.endpoint_closed, "rightkit-control endpoint must close");
    assert!(stopped.process_gone, "native app process must exit");
    if cfg!(target_os = "macos") {
        assert!(
            stopped.owned_never_frontmost,
            "hidden qualification must never activate app: states={:?}, owned_frontmost_pids={:?}, before={:?}, after={:?}",
            stopped.frontmost_states, stopped.owned_frontmost_pids, stopped.frontmost_before, stopped.frontmost_after,
        );
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
        let active = before["active"].as_u64().expect("import must select active photo");
        run(control, "develop.beginInteraction", json!({"label": "Backup baseline edit"}));
        run(control, "develop.set", json!({"control": "light.exposure", "value": 1.25, "ids": [active]}));
        run(control, "develop.endInteraction", json!({}));
        let edited = snapshot(control);
        let before_counts = edited["counts"].clone();
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
    let merge_inputs = write_merge_fixture_copies(&input_dir, &inputs.png);
    let source_paths = [
        ("arw", inputs.arw.as_path()),
        ("png", inputs.png.as_path()),
        ("catalog", inputs.catalog.as_path()),
        ("scalabilitySources", inputs.scalability_sources.as_path()),
        ("mergePngA", merge_inputs[0].as_path()),
        ("mergePngB", merge_inputs[1].as_path()),
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
        // Capture core import/edit/undo/export proof before broader parity journeys.
        "engineExport",
        "mergeHdr",
        "ipc",
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
    let mut failures = Vec::new();
    for name in scenario_names {
        // Keep collecting native evidence after a failed journey. Every failure
        // still fails this test, without hiding later platform defects.
        let outcome = catch_unwind(AssertUnwindSafe(|| {
            harness.scenario(name, "fast", &[], |scenario| {
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
                    assert_eq!(reopened["ui"]["theme"].as_str(), Some("dark"), "dark theme choice must survive restart");
                    wait_for_dom(control, "return document.documentElement.getAttribute('data-theme') === 'dark';");
                    let dark = scenario.dir().join("dark-theme.png");
                    control.screenshot_to(&dark).expect("dark theme screenshot must be captured");
                    assert!(dark.is_file());
                    assert_library_header_contrast(control, "dark");
                    control.command("lc_preferences", &json!({"ui": {"theme": "light"}})).expect("light theme preference must persist");
                });
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let reopened = control.command("lc_preferences", &Value::Null).expect("light preference must reopen");
                    assert_eq!(reopened["ui"]["theme"].as_str(), Some("light"), "light theme choice must survive restart");
                    wait_for_dom(control, "return document.documentElement.getAttribute('data-theme') === 'light';");
                    let light = scenario.dir().join("light-theme.png");
                    control.screenshot_to(&light).expect("light theme screenshot must be captured");
                    assert!(light.is_file());
                    assert_library_header_contrast(control, "light");
                });
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    // Open through the actual RightKit command palette, then choose Settings by pointer.
                    click_dom(control, ".rk-search--trigger", "command palette trigger click must execute");
                    assert_eq!(wait_for_dom(control, "return document.querySelector('.rk-palette') !== null;").as_bool(), Some(true));
                    control.key("Escape").expect("command palette Escape must execute");
                    assert_eq!(wait_for_dom(control, "return document.querySelector('.rk-palette') === null;").as_bool(), Some(true));
                    let pre_modal_focus = control
                        .eval("return (() => { const node = document.activeElement; return { tag: node?.tagName, id: node?.id, className: node?.className, aria: node?.getAttribute('aria-label') }; })();")
                        .expect("pre-modal focus query must execute");
                    click_dom(control, ".rk-search--trigger", "settings command palette trigger click must execute");
                    assert_eq!(wait_for_dom(control, "return document.querySelector('.rk-palette') !== null;").as_bool(), Some(true));
                    for key in ["s", "e", "t", "t", "i", "n", "g", "s"] {
                        control.key(key).expect("command palette search key must execute");
                    }
                    assert_eq!(wait_for_dom(control, "return document.querySelector('[data-command=\"app.settings\"]') !== null;").as_bool(), Some(true));
                    click_dom(control, "[data-command=\"app.settings\"]", "settings command palette option click must execute");
                    assert_eq!(
                        wait_for_dom(control, "return document.querySelector('.lc-dialog')?.contains(document.activeElement) === true;").as_bool(),
                        Some(true),
                        "settings modal must receive focus",
                    );
                    let modal_state = control
                        .eval(
                            "return (() => { const root = document.getElementById('root'); const backdrop = document.querySelector('.lc-dialog-backdrop'); const dialog = document.querySelector('.lc-dialog'); return { dialog: Boolean(dialog), dialogInert: Boolean(dialog?.inert), backdropInert: Boolean(backdrop?.inert), rootInert: Boolean(root?.inert), rootHidden: root?.getAttribute('aria-hidden'), activeInside: Boolean(dialog?.contains(document.activeElement)) }; })();",
                        )
                        .expect("settings modal focus scope query must execute");
                    assert_eq!(modal_state["dialog"].as_bool(), Some(true));
                    assert_eq!(modal_state["dialogInert"].as_bool(), Some(false), "settings dialog must remain interactive");
                    assert_eq!(modal_state["backdropInert"].as_bool(), Some(false), "settings backdrop must remain interactive");
                    assert_eq!(modal_state["rootInert"].as_bool(), Some(true), "application background must be inert");
                    assert_eq!(modal_state["rootHidden"].as_str(), Some("true"), "application background must be hidden from assistive technology");
                    assert_eq!(modal_state["activeInside"].as_bool(), Some(true));

                    // Settings contains a real select: changing it must retain focused control through React rerender.
                    for _ in 0..5 {
                        control.key("Tab").expect("native Tab must reach settings select");
                    }
                    assert_eq!(control.eval("return document.activeElement?.tagName;").expect("settings select focus query must execute").as_str(), Some("SELECT"));
                    control.key("Down").expect("native select change must execute");
                    assert_eq!(wait_for_dom(control, "return document.activeElement?.tagName === 'SELECT';").as_bool(), Some(true), "focused settings select must survive rerender");

                    let focusable_count = control
                        .eval("return Array.from(document.querySelectorAll('.lc-dialog button,.lc-dialog [href],.lc-dialog input,.lc-dialog select,.lc-dialog textarea,.lc-dialog [tabindex]:not([tabindex=\"-1\"])')).filter((node) => !node.disabled).length;")
                        .expect("settings focusable count query must execute")
                        .as_u64()
                        .expect("settings modal must expose focusable controls");
                    assert!(focusable_count > 5, "settings modal must expose native tab stops");
                    for _ in 0..(focusable_count - 5) {
                        control.key("Tab").expect("native Tab must reset settings focus cycle");
                    }
                    for _ in 0..focusable_count {
                        control.key("Tab").expect("native Tab must execute in settings modal");
                        let inside = control
                            .eval("return document.querySelector('.lc-dialog')?.contains(document.activeElement) === true;")
                            .expect("settings focus trap query must execute");
                        assert_eq!(inside.as_bool(), Some(true), "native Tab must stay inside settings modal");
                    }
                    let wrapped = control
                        .eval("return (() => { const dialog = document.querySelector('.lc-dialog'); const nodes = Array.from(dialog?.querySelectorAll('button,input,select,textarea,[href],[tabindex]:not([tabindex=\"-1\"])') || []).filter((node) => !node.disabled); return nodes.indexOf(document.activeElement); })();")
                        .expect("settings focus wrap query must execute");
                    assert_eq!(wrapped.as_i64(), Some(0), "native Tab must wrap to first settings control");

                    control.key("Escape").expect("native Escape must close settings modal");
                    assert_eq!(wait_for_dom(control, "return document.querySelector('.lc-dialog') === null;").as_bool(), Some(true));
                    let restored = control
                        .eval("return (() => { const root = document.getElementById('root'); const node = document.activeElement; return { rootInert: Boolean(root?.inert), rootHidden: root?.getAttribute('aria-hidden'), focus: { tag: node?.tagName, id: node?.id, className: node?.className, aria: node?.getAttribute('aria-label') } }; })();")
                        .expect("settings focus restoration query must execute");
                    assert_eq!(restored["rootInert"].as_bool(), Some(false), "background inert state must be restored");
                    assert_eq!(restored["rootHidden"], Value::Null, "background aria-hidden state must be restored");
                    assert_eq!(restored["focus"], pre_modal_focus, "focus must return to actual pre-modal target");
                });
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let metadata_name = "QA Settings Metadata";
                    run(control, "metadata.savePreset", json!({"name": metadata_name, "fields": {"copyright": "© QA Copyright", "creator": "QA Creator", "title": "QA Settings"}}));
                    run(control, "library.preferences", json!({"import": {"copyright": "Fixture Copyright", "creator": "Fixture Creator", "metadataPreset": metadata_name}, "cacheMb": 0}));
                    let before_library = run(control, "library.preferences", json!({}));
                    click_dom(control, ".rk-search--trigger", "settings hydration palette trigger must execute");
                    wait_for_dom(control, "return document.querySelector('.rk-palette') !== null;");
                    for key in ["s", "e", "t", "t", "i", "n", "g", "s"] {
                        control.key(key).expect("settings hydration search must execute");
                    }
                    click_dom(control, "[data-command=\"app.settings\"]", "settings hydration command must open");
                    wait_for_dom(control, "return document.querySelector('.lc-dialog h2')?.textContent === 'Settings';");
                    click_dom(control, ".lc-settings-tab:nth-child(2)", "settings Import tab must open");
                    wait_for_dom(control, "return ['Raw photos','Default copyright','Default creator','Metadata preset'].every((label) => [...document.querySelectorAll('.lc-settings-panel .lc-field span')].some((node) => node.textContent === label)) && document.querySelector('.lc-dialog-actions .lc-button-primary')?.disabled === false;");
                    let hydrated = control
                        .eval("return Object.fromEntries([...document.querySelectorAll('.lc-settings-panel .lc-field')].map((field) => [field.querySelector('span')?.textContent, field.querySelector('input,select')?.value]));")
                        .expect("settings hydration fields must be readable");
                    let raw_expected = before_library["import"]["rawPreset"].as_str().unwrap_or("default");
                    let other_expected = before_library["import"]["otherPreset"].as_str().unwrap_or("default");
                    let metadata_expected = before_library["import"]["metadataPreset"].as_str().unwrap_or("none");
                    assert_eq!(hydrated["Raw photos"].as_str(), Some(raw_expected), "saved raw import default must hydrate");
                    assert_eq!(hydrated["Non-raw photos"].as_str(), Some(other_expected), "saved non-raw import default must hydrate");
                    assert_eq!(hydrated["Metadata preset"].as_str(), Some(metadata_expected), "saved metadata preset must hydrate");
                    assert_eq!(hydrated["Default copyright"].as_str(), Some("Fixture Copyright"), "saved copyright default must hydrate");
                    assert_eq!(hydrated["Default creator"].as_str(), Some("Fixture Creator"), "saved creator default must hydrate");
                    click_dom(control, ".lc-settings-tab:nth-child(3)", "settings Performance tab must open");
                    wait_for_dom(control, "return [...document.querySelectorAll('.lc-settings-panel .lc-field span')].some((node) => node.textContent === 'Memory budget (MB; 0 = automatic)');");
                    let memory = control
                        .eval("return [...document.querySelectorAll('.lc-settings-panel .lc-field')].find((field) => field.querySelector('span')?.textContent === 'Memory budget (MB; 0 = automatic)')?.querySelector('input')?.value || ''; ")
                        .expect("automatic memory setting must be readable");
                    assert_eq!(memory.as_str(), Some("0"), "fresh settings must expose automatic memory budget");
                    click_dom(control, ".lc-settings-tab:nth-child(1)", "settings General tab must open");
                    set_dialog_field(control, "Theme", "dark");
                    click_dom(control, ".lc-dialog-actions .lc-button-primary", "settings field-only save must execute");
                    assert_eq!(wait_for_dom(control, "return document.querySelector('.lc-dialog') === null;").as_bool(), Some(true));
                    let after_library = run(control, "library.preferences", json!({}));
                    assert_eq!(after_library["import"], before_library["import"], "theme-only Settings save must preserve import defaults & metadata preset");
                    let persisted = control.command("lc_preferences", &Value::Null).expect("theme preference must reopen after field-only save");
                    assert_eq!(persisted["ui"]["theme"].as_str(), Some("dark"));
                    control.command("lc_preferences", &json!({"ui": {"theme": "light"}})).expect("settings journey must restore light theme");
                });
            } else if name == "mergeHdr" {
                with_control(&binary, scenario, &inputs.catalog, |control, _data| {
                    let paths = merge_inputs.iter().map(|path| path.to_string_lossy().to_string()).collect::<Vec<_>>();
                    let review = run(control, "library.importPreview", json!({"paths": paths}));
                    let candidates = review["candidates"].as_array().expect("HDR merge fixture review must return candidates");
                    assert_eq!(candidates.len(), 2, "HDR merge review must expose two procedural PNGs: {review}");
                    assert!(candidates.iter().all(|candidate| candidate["duplicate"].is_null()), "HDR merge fixtures must be distinct import candidates: {review}");
                    let started = run(control, "library.import", json!({"paths": paths, "mode": "add"}));
                    let import_task = started["taskId"].as_str().expect("HDR fixture import must return task id").to_string();
                    let imported_snapshot = wait_task(control, &import_task);
                    let imported_receipt = completed_task(&imported_snapshot, &import_task);
                    assert_eq!(imported_receipt["state"].as_str(), Some("done"), "HDR fixture import must complete: {imported_receipt}");
                    let ids = imported_receipt["result"]["imported"]
                        .as_array()
                        .expect("HDR fixture import receipt must list imported ids")
                        .iter()
                        .map(|id| id.as_u64().expect("HDR fixture photo id must be numeric"))
                        .collect::<Vec<_>>();
                    assert_eq!(ids.len(), 2, "HDR fixture import must add both procedural PNGs: {imported_receipt}");
                    run(control, "library.select", json!({"ids": ids, "active": ids[0], "mode": "replace"}));

                    click_dom(control, ".rk-search--trigger", "merge command palette trigger must execute");
                    wait_for_dom(control, "return document.querySelector('.rk-palette') !== null;");
                    for key in ["h", "d", "r"] {
                        control.key(key).expect("HDR command palette search must execute");
                    }
                    wait_for_dom(control, "return document.querySelector('[data-command=\"dialog.mergeHdr\"]') !== null;");
                    click_dom(control, "[data-command=\"dialog.mergeHdr\"]", "HDR merge command must open React dialog");
                    wait_for_dom(control, "return document.querySelector('.lc-dialog h2')?.textContent === 'HDR Merge Preview';");
                    let first_preview = wait_for_rendered_preview(control, ".lc-merge-preview img", None);
                    let first_src = first_preview["src"].as_str().expect("HDR preview must expose scoped source handle").to_string();
                    assert!(first_preview["naturalWidth"].as_u64().is_some_and(|width| width > 0));
                    assert!(first_preview["naturalHeight"].as_u64().is_some_and(|height| height > 0));
                    let summary = control
                        .eval("return document.querySelector('.lc-merge-preview-info')?.textContent?.trim() || '';" )
                        .expect("HDR preview summary query must execute");
                    assert!(summary.as_str().is_some_and(|text| text.contains("2 photos")), "HDR preview must summarize both selected photos: {summary}");
                    let first_screenshot = scenario.dir().join("merge-hdr-preview-initial.png");
                    control.screenshot_to(&first_screenshot).expect("initial HDR preview screenshot must save");

                    set_dialog_field(control, "Deghost amount", "medium");
                    let second_preview = wait_for_rendered_preview(control, ".lc-merge-preview img", Some(&first_src));
                    let second_src = second_preview["src"].as_str().expect("updated HDR preview must expose scoped source handle");
                    assert_ne!(second_src, first_src, "HDR option update must publish a fresh preview handle");
                    assert!(second_preview["naturalWidth"].as_u64().is_some_and(|width| width > 0));
                    assert!(second_preview["naturalHeight"].as_u64().is_some_and(|height| height > 0));
                    let second_screenshot = scenario.dir().join("merge-hdr-preview-deghost.png");
                    control.screenshot_to(&second_screenshot).expect("updated HDR preview screenshot must save");

                    click_dom(control, ".lc-dialog-actions .lc-button-primary", "HDR merge submit must execute");
                    wait_for_dom(control, "return document.querySelector('.lc-dialog h2')?.textContent === 'Merging Photos';");
                    let mut merge_task = None;
                    for _ in 0..200 {
                        let current = snapshot(control);
                        merge_task = current["status"]["jobs"]
                            .as_array()
                            .and_then(|jobs| jobs.iter().find(|job| job["kind"].as_str() == Some("merge")))
                            .and_then(|job| job["id"].as_str())
                            .map(str::to_string)
                            .or_else(|| current["status"]["completedJobs"].as_array().and_then(|jobs| jobs.iter().find(|job| job["kind"].as_str() == Some("merge"))).and_then(|job| job["id"].as_str()).map(str::to_string));
                        if merge_task.is_some() { break; }
                        sleep(Duration::from_millis(50));
                    }
                    let merge_task = merge_task.expect("React HDR submit must expose merge task id through native snapshot");
                    let finished = wait_task(control, &merge_task);
                    let receipt = completed_task(&finished, &merge_task);
                    assert_eq!(receipt["state"].as_str(), Some("done"), "HDR merge task must complete: {receipt}");
                    let result = &receipt["result"];
                    let output_path = result["path"].as_str().expect("HDR merge receipt must expose output path");
                    assert!(output_path.ends_with(".dng"), "HDR merge output must be DNG: {result}");
                    let output = PathBuf::from(output_path);
                    let bytes = fs::read(&output).expect("HDR merge DNG output must be readable");
                    assert!(bytes.starts_with(b"II*\0") || bytes.starts_with(b"MM\0*"), "HDR merge output must contain TIFF/DNG bytes");
                    assert!(bytes.len() > 1024, "HDR merge DNG output must contain image data");
                    assert!(finished["counts"]["catalog"].as_u64().is_some_and(|count| count >= 3), "HDR merge must import merged output into catalog: {finished}");
                    let merged_id = result["id"].as_u64().expect("HDR merge receipt must expose merged photo id");
                    assert_eq!(finished["active"].as_u64(), Some(merged_id), "completed HDR merge must select merged photo");
                    let merged_generation = finished["viewGeneration"].as_u64().expect("merged photo view generation must be numeric");
                    let decoded = control
                        .command("lc_preview", &json!({"request": {"photoId": merged_id, "slot": "merge-qualification", "viewGeneration": merged_generation, "width": 96, "height": 64, "quality": "draft", "before": false, "sequence": 1}}))
                        .expect("merged DNG must decode through lc_preview");
                    assert!(decoded["handle"].as_str().is_some_and(|handle| !handle.is_empty()), "merged DNG preview must return scoped handle: {decoded}");
                    assert!(decoded["width"].as_u64().is_some_and(|width| width > 0));
                    assert!(decoded["height"].as_u64().is_some_and(|height| height > 0));
                    let decoded_handle = decoded["handle"].as_str().expect("merged preview handle must be text").to_string();
                    control.command("lc_preview_ack", &json!({"handle": decoded_handle})).expect("merged preview handle must be acknowledged");
                    click_dom(control, ".lc-dialog-actions .lc-button", "completed HDR dialog must close");
                    wait_for_dom(control, "return document.querySelector('.lc-dialog') === null;");
                    control.key("D").expect("merged DNG develop route must execute");
                    wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                    let merged_dom = wait_for_rendered_preview(control, "img.stage-preview", None);
                    assert!(merged_dom["naturalWidth"].as_u64().is_some_and(|width| width > 0), "merged DNG WebView preview must decode");
                    assert!(merged_dom["naturalHeight"].as_u64().is_some_and(|height| height > 0), "merged DNG WebView preview must decode");
                    let receipt_path = scenario.dir().join("merge-hdr.json");
                    fs::write(&receipt_path, serde_json::to_vec_pretty(&json!({
                        "schema": 1,
                        "journey": "react-hdr-preview-option-update-completed-job",
                        "sourcePaths": paths,
                        "importTask": import_task,
                        "importedIds": ids,
                        "initialPreview": first_preview,
                        "updatedPreview": second_preview,
                        "mergeTask": merge_task,
                        "completedJob": receipt,
                        "outputDng": output,
                        "decodedMergedPreview": decoded,
                        "renderedMergedPreview": merged_dom,
                    })).expect("HDR merge receipt must serialize")).expect("HDR merge receipt must write");
                    scenario.keep("merge-hdr-preview-initial.png", &first_screenshot);
                    scenario.keep("merge-hdr-preview-deghost.png", &second_screenshot);
                    scenario.keep("merge-hdr.json", &receipt_path);
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
                        let _selected_first = run(control, "library.select", json!({"ids": [first_id], "active": first_id, "mode": "replace"}));
                        let first_again = snapshot(control);
                        assert_eq!(first_again["active"].as_u64(), Some(first_id), "selection snapshot must expose first active photo");
                        assert_eq!(first_again["viewGeneration"].as_u64(), Some(second_generation), "selection must preserve visible view generation");
                        let first_again_grid = wait_for_rendered_preview(control, ".lc-photo-cell.is-active img.lc-photo-preview", Some(&second_src));
                        assert_active_grid_identity(control, "procedural-rgb-01.png");
                        assert_ne!(first_again_grid["src"].as_str(), Some(second_src.as_str()), "quick selection must replace second photo pixels");
                        control.key("D").expect("develop route key must execute");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        let first_stage = wait_for_rendered_preview(control, "img.stage-preview", None);
                        let first_stage_src = first_stage["src"].as_str().expect("first stage preview must expose scoped source").to_string();
                        let _selected_second = run(control, "library.select", json!({"ids": [second_id], "active": second_id, "mode": "replace"}));
                        let second_again = snapshot(control);
                        assert_eq!(second_again["active"].as_u64(), Some(second_id), "selection snapshot must expose second active photo");
                        assert_eq!(second_again["viewGeneration"], first_again["viewGeneration"], "selection must preserve visible view generation");
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
                        let _selected = run(control, "library.select", json!({"ids": [first_id], "active": first_id, "mode": "replace"}));
                        let selected_snapshot = snapshot(control);
                        assert_eq!(selected_snapshot["active"].as_u64(), Some(first_id), "scalability selection snapshot must identify first page photo");
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
                        let prepared = control
                            .eval(r#"return (() => {
                                const e = document.querySelector("input[aria-label='Exposure']");
                                if (!e) return {found: false};
                                e.scrollIntoView({block: 'center', inline: 'nearest'});
                                const r = e.getBoundingClientRect();
                                return {
                                    found: true,
                                    rect: {x: r.x, y: r.y, width: r.width, height: r.height},
                                    viewport: {width: innerWidth, height: innerHeight, dpr: devicePixelRatio},
                                };
                            })();"#)
                            .expect("exposure slider visibility query must execute");
                        assert_eq!(prepared["found"].as_bool(), Some(true), "exposure slider must exist before gesture");
                        assert_eq!(
                            wait_for_dom(control, r#"return (() => {
                                const e = document.querySelector("input[aria-label='Exposure']");
                                if (!e) return false;
                                const r = e.getBoundingClientRect();
                                return r.width > 1 && r.height > 1 && r.left >= 0 && r.top >= 0 &&
                                    r.right <= innerWidth && r.bottom <= innerHeight;
                            })();"#)
                            .as_bool(),
                            Some(true),
                            "exposure slider must be fully within native viewport",
                        );
                        let slider = control
                            .eval(r#"return (() => {
                                const e = document.querySelector("input[aria-label='Exposure']");
                                const r = e.getBoundingClientRect();
                                return {
                                    x: r.x, y: r.y, width: r.width, height: r.height,
                                    value: e.value, ariaValueNow: e.getAttribute('aria-valuenow'),
                                    viewport: {width: innerWidth, height: innerHeight, dpr: devicePixelRatio},
                                };
                            })();"#)
                            .expect("exposure slider geometry query must execute");
                        let x = slider["x"].as_f64().expect("exposure slider x must be numeric");
                        let y = slider["y"].as_f64().expect("exposure slider y must be numeric") + slider["height"].as_f64().unwrap_or(16.0) / 2.0;
                        let width = slider["width"].as_f64().expect("exposure slider width must be numeric");
                        assert!(x >= 0.0 && y >= 0.0 && x + width <= slider["viewport"]["width"].as_f64().unwrap_or(0.0), "exposure slider drag start must be within viewport: {slider}");
                        let diagnostic_setup = control
                            .eval(r#"return (() => {
                                const e = document.querySelector("input[aria-label='Exposure']");
                                if (!e) return {found: false};
                                const read = () => {
                                    const r = e.getBoundingClientRect();
                                    return {
                                        value: e.value,
                                        ariaValueNow: e.getAttribute('aria-valuenow'),
                                        rect: {x: r.x, y: r.y, width: r.width, height: r.height},
                                        viewport: {width: innerWidth, height: innerHeight, dpr: devicePixelRatio},
                                        visible: r.left >= 0 && r.top >= 0 && r.right <= innerWidth && r.bottom <= innerHeight,
                                    };
                                };
                                const state = window.__lcGestureDiagnostics || {events: [], bound: false};
                                if (!state.bound) {
                                    for (const type of ['pointerdown', 'pointermove', 'pointerup', 'pointercancel', 'input', 'change']) {
                                        e.addEventListener(type, event => {
                                            if (state.events.length < 64) state.events.push({
                                                type,
                                                trusted: event.isTrusted,
                                                clientX: event.clientX,
                                                clientY: event.clientY,
                                                buttons: event.buttons,
                                                value: e.value,
                                            });
                                        }, true);
                                    }
                                    state.bound = true;
                                }
                                state.before = read();
                                window.__lcGestureDiagnostics = state;
                                return {found: true, before: state.before, events: state.events};
                            })();"#)
                            .expect("exposure slider diagnostics setup must execute");
                        let drag_result = control.drag((x + width * 0.45, y), (x + width * 0.7, y), 8);
                        let after_drag = snapshot(control);
                        let mut edited = None;
                        let mut last_snapshot = after_drag.clone();
                        for _ in 0..100 {
                            let value = snapshot(control);
                            last_snapshot = value.clone();
                            if value["develop"]["light"]["exposure"] != imported["develop"]["light"]["exposure"]
                                && value["undo"].as_u64().is_some_and(|undo| undo > before)
                            {
                                edited = Some(value);
                                break;
                            }
                            sleep(Duration::from_millis(50));
                        }
                        let diagnostic_after = control
                            .eval(r#"return (() => {
                                const e = document.querySelector("input[aria-label='Exposure']");
                                const state = window.__lcGestureDiagnostics || {events: []};
                                if (!e) return {found: false, events: state.events};
                                const r = e.getBoundingClientRect();
                                return {
                                    found: true,
                                    value: e.value,
                                    ariaValueNow: e.getAttribute('aria-valuenow'),
                                    rect: {x: r.x, y: r.y, width: r.width, height: r.height},
                                    viewport: {width: innerWidth, height: innerHeight, dpr: devicePixelRatio},
                                    visible: r.left >= 0 && r.top >= 0 && r.right <= innerWidth && r.bottom <= innerHeight,
                                    events: state.events,
                                };
                            })();"#)
                            .expect("exposure slider diagnostics readback must execute");
                        let compact_host = |value: &Value| {
                            json!({
                                "exposure": value.pointer("/develop/light/exposure").cloned().unwrap_or(Value::Null),
                                "undo": value.get("undo").cloned().unwrap_or(Value::Null),
                                "active": value.get("active").cloned().unwrap_or(Value::Null),
                                "viewGeneration": value.get("viewGeneration").cloned().unwrap_or(Value::Null),
                                "status": {"error": value.pointer("/status/error").cloned().unwrap_or(Value::Null)},
                            })
                        };
                        let events = diagnostic_after
                            .get("events")
                            .and_then(Value::as_array)
                            .map(|values| values.iter().take(64).cloned().collect::<Vec<_>>())
                            .unwrap_or_default();
                        let receipt = scenario.dir().join("gesture-pointer.json");
                        let receipt_value = json!({
                            "slider": slider,
                            "prepared": prepared,
                            "setup": diagnostic_setup.get("before").cloned().unwrap_or(Value::Null),
                            "after": diagnostic_after,
                            "events": events,
                            "host": {
                                "before": compact_host(&imported),
                                "afterDrag": compact_host(&after_drag),
                                "after": compact_host(&last_snapshot),
                            },
                        });
                        if let Ok(bytes) = serde_json::to_vec_pretty(&receipt_value) {
                            let _ = fs::write(&receipt, bytes);
                        }
                        eprintln!("[qa] gesture pointer receipt={}", receipt.display());
                        drag_result.expect("exposure slider drag must execute");
                        let edited = edited.unwrap_or_else(|| panic!("pointer drag edit did not settle through host bridge; receipt={}", receipt.display()));
                        assert_ne!(edited["develop"]["light"]["exposure"], imported["develop"]["light"]["exposure"], "pointer drag must change exposure through UI");
                        assert_eq!(edited["undo"].as_u64(), Some(before + 1), "one pointer gesture must create one undo step");
                        let edited_exposure = edited["develop"]["light"]["exposure"].clone();
                        let focused = control
                            .eval(r#"return (() => { const node = document.querySelector("input[aria-label='Exposure']"); if (!node) return false; node.focus(); return document.activeElement === node; })();"#)
                            .expect("exposure slider focus query must execute");
                        assert_eq!(focused.as_bool(), Some(true), "exposure slider focus must execute");
                        control.eval("window.__lcInteractionDiagnostics = []; return true;").expect("keyboard interaction diagnostics must initialize");
                        let keyboard_result = catch_unwind(AssertUnwindSafe(|| {
                            control.key("Left").expect("exposure keyboard adjustment must execute");
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
                        }));
                        let keyboard_trace = control.eval("return (() => { const trace = window.__lcInteractionDiagnostics || []; delete window.__lcInteractionDiagnostics; return trace; })();").unwrap_or(Value::Null);
                        let keyboard_receipt = json!({"before": edited, "after": snapshot(control), "trace": keyboard_trace});
                        if let Ok(bytes) = serde_json::to_vec_pretty(&keyboard_receipt) {
                            let _ = fs::write(scenario.dir().join("gesture-keyboard.json"), bytes);
                        }
                        if let Err(payload) = keyboard_result {
                            resume_unwind(payload);
                        }
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
                        wait_for_rendered_preview(control, "img.stage-preview", None);
                        capture_visible_tool_preview(control, &scenario.dir().join("tool-crop.png"));

                        run(control, "mask.add", json!({"kind": "radial", "center": [0.5, 0.5], "rx": 0.2, "ry": 0.2}));
                        let masked = snapshot(control);
                        assert_eq!(masked["develop"]["masks"].as_array().map(Vec::len), Some(1), "mask command must create mask state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Masking']", "masking tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Masking']")?.classList.contains('selected') === true;"#);
                        wait_for_rendered_preview(control, "img.stage-preview", None);
                        wait_for_dom(control, "return document.querySelector('.lc-inspector__header small')?.textContent === '1 mask';");
                        capture_visible_tool_preview(control, &scenario.dir().join("tool-masking.png"));

                        run(control, "spot.add", json!({"mode": "remove", "points": [[0.5, 0.5]], "size": 0.05, "source": [0.1, 0.0]}));
                        let spotted = snapshot(control);
                        assert_eq!(spotted["develop"]["spots"].as_array().map(Vec::len), Some(1), "remove tool command must create spot state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Remove']", "remove tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Remove']")?.classList.contains('selected') === true;"#);
                        wait_for_rendered_preview(control, "img.stage-preview", None);
                        capture_visible_tool_preview(control, &scenario.dir().join("tool-remove.png"));

                        run(control, "redeye.add", json!({"center": [0.5, 0.5], "rx": 0.1, "ry": 0.1}));
                        let red_eye = snapshot(control);
                        assert_eq!(red_eye["develop"]["red_eye"].as_array().map(Vec::len), Some(1), "red-eye command must create correction state");
                        click_dom(control, ".stage-toolstrip button[aria-label='Red Eye']", "red-eye tool click must execute");
                        wait_for_dom(control, r#"return document.querySelector(".stage-toolstrip button[aria-label='Red Eye']")?.classList.contains('selected') === true;"#);
                        wait_for_rendered_preview(control, "img.stage-preview", None);
                        capture_visible_tool_preview(control, &scenario.dir().join("tool-red-eye.png"));
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
                        let inspected = task_result(control, "library.inspectLightroom", json!({"path": inputs.catalog}));
                        assert_eq!(inspected["photos"].as_u64(), Some(2));
                        let first = task_result(control, "library.importLightroom", json!({"path": inputs.catalog, "updateExisting": false}));
                        assert_eq!(first["photos"].as_u64(), Some(2), "Lightroom fixture must import master & virtual copy: {first}");
                        assert_eq!(first["collections"].as_u64(), Some(2), "nested Lightroom collections must import: {first}");
                        assert_eq!(first["missing"].as_array().map(Vec::len), Some(0));
                        let mapping = first["mapping"].clone();
                        let second = task_result(control, "library.importLightroom", json!({"path": inputs.catalog, "updateExisting": true}));
                        assert_eq!(second["mapping"], mapping, "reimport must preserve source identities");
                        assert_eq!(second["archive"], first["archive"], "unchanged source must reuse bounded archive");
                        fs::write(scenario.dir().join("lightroom-import.json"), serde_json::to_vec_pretty(&json!({"inspection": inspected, "first": first, "reimport": second})).expect("Lightroom receipt must serialize")).expect("Lightroom receipt must save");
                        assert_eq!(snapshot(control)["counts"]["catalog"].as_u64(), Some(2));
                    }
                    "engineExport" => {
                        let imported = import_file(control, &inputs.png);
                        let id = imported["active"].as_u64().expect("PNG import must select photo");
                        control.key("D").expect("develop route key must execute for engine export journey");
                        wait_for_dom(control, "return document.querySelector('.stage-workspace.stage-detail') !== null;");
                        wait_for_dom(control, "return document.querySelector('.lc-histogram svg path[stroke=\"#df6464\"]')?.getAttribute('d')?.includes('L') === true && document.querySelector('.lc-histogram__footer')?.textContent.includes('samples') === true;");
                        let curve_before = snapshot(control)["develop"]["curve"].clone();
                        click_dom(control, ".lc-curve-picker > button", "curve preset picker must open");
                        wait_for_dom(control, "return document.querySelector('[aria-label=\"Tone curve presets\"]') !== null;");
                        click_dom(control, ".lc-curve-picker__menu [role=menuitemradio]:nth-child(3)", "Strong Contrast preset must apply");
                        let curve_changed = wait_for_snapshot(control, |state| state["develop"]["curve"] != curve_before, "curve preset must change Rust develop settings");
                        assert_ne!(curve_changed["develop"]["curve"], curve_before);
                        run(control, "edit.undo", json!({}));
                        assert_eq!(snapshot(control)["develop"]["curve"], curve_before, "curve preset must be undoable");
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
                        // Exercise actual React dialog, native IPC & encoder, rather than
                        // proving only engine export with hand-authored command parameters.
                        click_dom(control, ".rk-search--trigger", "Export palette trigger must execute");
                        wait_for_dom(control, "return document.querySelector('.rk-palette') !== null;");
                        for key in ["e", "x", "p", "o", "r", "t"] {
                            control.key(key).expect("Export palette search must execute");
                        }
                        wait_for_dom(control, "return document.querySelector('[data-command=\"dialog.export\"]') !== null;");
                        click_dom(control, "[data-command=\"dialog.export\"]", "Export command must open real dialog");
                        wait_for_dom(control, "return document.querySelector('.lc-dialog h2')?.textContent === 'Export';");
                        set_dialog_field(control, "Export folder", &output.display().to_string());
                        set_dialog_field(control, "Format", "png");
                        set_dialog_field(control, "Size", "96");
                        set_dialog_field(control, "Naming template", "dialog-{name}");
                        set_dialog_field(control, "Sharpen", "screen");
                        wait_for_dom(control, "return [...document.querySelectorAll('.lc-dialog .lc-field span')].some((node) => node.textContent === 'Sharpen amount');");
                        set_dialog_field(control, "Sharpen amount", "high");
                        assert_eq!(control.eval("return (() => { const node = [...document.querySelectorAll('.lc-dialog .lc-check')].find((item) => item.querySelector('span')?.textContent === 'Watermark')?.querySelector('input'); if (!node) return false; node.click(); return node.checked; })();").expect("watermark checkbox must execute").as_bool(), Some(true));
                        wait_for_dom(control, "return document.querySelector('.lc-dialog textarea') !== null;");
                        set_dialog_field(control, "Watermark text", "MMMM\nMMMM");
                        set_dialog_field(control, "Size (%)", "15");
                        set_dialog_field(control, "Opacity (%)", "100");
                        set_dialog_field(control, "Colour", "#ff00ff");
                        set_dialog_field(control, "Watermark position", "topLeft");
                        let dialog_fields = control.eval("return Object.fromEntries([...document.querySelectorAll('.lc-dialog .lc-field')].map((field) => [field.querySelector('span')?.textContent, field.querySelector('input,select,textarea')?.value]));").expect("actual Export fields must be recorded");
                        assert_eq!(dialog_fields["Watermark text"].as_str(), Some("MMMM\nMMMM"), "multiline watermark must survive React input");
                        assert_eq!(dialog_fields["Sharpen amount"].as_str(), Some("high"));
                        let dialog_screenshot = scenario.dir().join("export-dialog.png");
                        control.screenshot_to(&dialog_screenshot).expect("actual Export dialog screenshot must save");
                        click_dom(control, ".lc-dialog-actions .lc-button-primary", "real Export submit must execute");
                        let dialog_exported = output.join("dialog-procedural-rgb-01.png");
                        let export_deadline = Instant::now() + Duration::from_secs(30);
                        while !dialog_exported.is_file() && Instant::now() < export_deadline {
                            sleep(Duration::from_millis(50));
                        }
                        assert!(dialog_exported.is_file(), "React Export dialog must create named PNG; native state={}", snapshot(control));
                        assert_png_pixels(&dialog_exported, 96, 64);
                        let baseline_magenta = top_left_magenta_pixels(&exported);
                        let watermark_magenta = top_left_magenta_pixels(&dialog_exported);
                        assert!(watermark_magenta >= baseline_magenta + 5, "React watermark must add visible top-left magenta pixels: before={baseline_magenta}, after={watermark_magenta}");
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
                                "dialogFields": dialog_fields,
                                "dialogExportedPng": dialog_exported,
                                "baselineTopLeftMagentaPixels": baseline_magenta,
                                "watermarkedTopLeftMagentaPixels": watermark_magenta,
                            }))
                            .expect("engine export receipt must serialize"),
                        )
                        .expect("engine export receipt must be writable");
                        scenario.keep("engine-export-stage.png", &screenshot);
                        scenario.keep("engine-export.json", &receipt);
                        scenario.keep("export-dialog.png", &dialog_screenshot);
                        scenario.keep("dialog-export.png", &dialog_exported);
                    }
                    _ => unreachable!("scenario inventory is static"),
                });
            }
            scenario.note(format!("executed hidden {platform} control journey: {name}"));
        })
        }));
        match outcome {
            Ok(outcome) => assert!(!outcome.is_skipped(), "native journey {name} must execute, not skip"),
            Err(_) => failures.push(name),
        }
        assert_sources_unchanged(&source_paths, &source_hashes);
    }
    assert!(failures.is_empty(), "native journeys failed: {}", failures.join(", "));
}
