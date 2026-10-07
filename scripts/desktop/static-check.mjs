import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import path from "node:path";
import { repoRoot, fail, readJson, requireFile } from "./lib.mjs";

const required = [
  "apps/lightcraft-desktop/Cargo.toml",
  "apps/lightcraft-desktop/tauri.conf.json",
  "apps/lightcraft-desktop/capabilities/main.json",
  "apps/lightcraft-desktop/web/tsconfig.json",
  "apps/lightcraft-desktop/web/vite.config.ts",
  "crates/desktop-host/Cargo.toml",
  "fixtures/desktop/catalog.json",
  "fixtures/desktop/native-journey.json",
  "fixtures/desktop/installed-baseline.json",
  "apps/lightcraft-desktop/tests/native_qualification.rs",
];

function checkRequiredFiles() {
  for (const file of required) requireFile(path.join(repoRoot, file), file);
}

function checkPins() {
  const packageJson = readJson("package.json");
  const all = { ...packageJson.dependencies, ...packageJson.devDependencies };
  const pins = {
    "@rightkit/app-shell": "0.2.1",
    "@rightkit/platform-ui": "0.1.2",
    "@rightkit/shell": "0.1.0",
    "@rightkit/theme": "0.2.1",
    "@rightkit/qa-chrome": "0.1.0",
    "@rightkit/git": "0.2.25",
    "@rightkit/release": "0.2.124",
  };
  for (const [name, version] of Object.entries(pins)) {
    if (all[name] !== version) fail(`${name} must stay pinned to ${version}`);
  }
  const cargo = readFileSync(path.join(repoRoot, "apps/lightcraft-desktop/Cargo.toml"), "utf8");
  for (const forbidden of [/path\s*=/, /git\s*=/, /EngineSupervisor/, /sidecar/]) {
    if (forbidden.test(cargo)) fail(`desktop Cargo.toml contains forbidden dependency form or sidecar reference: ${forbidden}`);
  }
  if (!/rightkit-shell\s*=\s*\{[^}]*default-features\s*=\s*false/.test(cargo)) fail("rightkit-shell must disable default features");
  if (!/rightkit-control\s*=\s*\{[^}]*version\s*=\s*\"=0\.1\.6\"/.test(cargo)) fail("rightkit-control must stay pinned to =0.1.6");
  if (!/rightkit-qa\s*=\s*\"=0\.2\.6\"/.test(cargo)) fail("rightkit-qa must stay pinned to =0.2.6");
}

function checkNativeConfig() {
  const tauri = readJson("apps/lightcraft-desktop/tauri.conf.json");
  const csp = tauri.app?.security?.csp ?? "";
  if (!csp.includes("lightcraft-preview:")) fail("Tauri CSP must allow scoped lightcraft-preview protocol");
  if (!Array.isArray(tauri.app?.security?.capabilities) || !tauri.app.security.capabilities.includes("main")) fail("Tauri main capability must be declared");
  const capability = readJson("apps/lightcraft-desktop/capabilities/main.json");
  if (!capability.permissions?.includes("core:window:allow-create")) fail("secondary-window permission is required");
}

function checkFixtures() {
  const catalog = readJson("fixtures/desktop/catalog.json");
  if (catalog.schema !== 1 || catalog.kind !== "synthetic-catalog") fail("synthetic catalog fixture schema mismatch");
  if (!Array.isArray(catalog.photos) || catalog.photos.length < 3) fail("synthetic catalog needs at least three photos");
  if (catalog.photos.some((photo) => !/^sha256:[0-9a-f]{64}$/.test(photo.sourceHash))) fail("every fixture photo needs source SHA-256");
  if (new Set(catalog.photos.map((photo) => photo.sourceHash)).size !== catalog.photos.length) fail("fixture source hashes must be unique");
  for (const photo of catalog.photos) {
    const bytes = Buffer.from(`lightcraft-desktop-fixture-v1\n${photo.fileName}\n${photo.pattern}\n`, "utf8");
    const expected = `sha256:${createHash("sha256").update(bytes).digest("hex")}`;
    if (photo.sourceHash !== expected) fail(`fixture source hash does not match procedural input ${photo.fileName}`);
  }
  if (catalog.photos.some((photo) => photo.inputLicense !== "CC0")) fail("fixture inputs must declare CC0 license");
  const journey = readJson("fixtures/desktop/native-journey.json");
  if (journey.schema !== 1 || journey.kind !== "native-control-journey" || journey.hidden !== true) fail("native journey must be hidden-control schema 1");
  for (const name of ["ipc", "stalePreview", "cache", "preferences", "gesture", "arwImport", "lightroomImport", "cliExport", "rollback"]) {
    if (!journey.scenarios?.some((scenario) => scenario.id === name)) fail(`native journey missing scenario ${name}`);
  }
  const baseline = readJson("fixtures/desktop/installed-baseline.json");
  if (baseline.schema !== 1 || baseline.rollback?.required !== true || !baseline.artifact?.sha256) fail("installed baseline must bind artifact hash & rollback");
}

function checkSourceContracts() {
  const gate = readFileSync(path.join(repoRoot, "scripts/desktop/gate.mjs"), "utf8");
  const candidate = readFileSync(path.join(repoRoot, "scripts/desktop/candidate.mjs"), "utf8");
  const installed = readFileSync(path.join(repoRoot, "scripts/desktop/installed-qualification.mjs"), "utf8");
  const nativeQa = readFileSync(path.join(repoRoot, "apps/lightcraft-desktop/tests/native_qualification.rs"), "utf8");
  const nativeHost = readFileSync(path.join(repoRoot, "apps/lightcraft-desktop/src/native.rs"), "utf8");
  if (!gate.includes("@rightkit/release/managed-cargo.mjs") || !gate.includes("generate-lockfile") || !gate.includes("--locked")) fail("desktop gate must use managed Cargo adapter & bounded lock bootstrap");
  if (!candidate.includes("cargoCompilerArtifacts") || !candidate.includes("materializeArtifacts") || candidate.includes("config.candidate") || candidate.includes("config.qualification")) fail("candidate must consume Cargo compiler records through real release config");
  if (!installed.includes("runCargoSync") || !installed.includes('"native_qualification"') || installed.includes("target.command")) fail("installed qualification must invoke fixed RightKit QA Rust test through managed Cargo");
  for (const token of ["rightkit_qa::control", "Mode::Hidden", "RIGHTKIT_QA_UI_BINARY", "stalePreview", "lightroomImport", "rollback"]) {
    if (!nativeQa.includes(token)) fail(`native qualification is missing executed control contract ${token}`);
  }
  if (!nativeHost.includes("rightkit_control::embedded::Control") || !nativeHost.includes("build_if_enabled") || !nativeHost.includes("qa-native")) fail("native app must register debug-gated rightkit-control");
  const ownedScripts = `${candidate}\n${installed}`;
  if (ownedScripts.includes("candidate.targets") || ownedScripts.includes("qualification.targets")) fail("desktop scripts must not use invented candidate/qualification release config APIs");
  const preview = readFileSync(path.join(repoRoot, "apps/lightcraft-desktop/web/src/preview/usePreview.ts"), "utf8");
  if (!/descriptor\.viewGeneration/.test(preview) || !/descriptor\.sequence/.test(preview)) fail("preview presentation must reject stale generation & sequence");
  const host = readFileSync(path.join(repoRoot, "crates/desktop-host/src/lib.rs"), "utf8");
  const renderer = readFileSync(path.join(repoRoot, "crates/desktop-host/src/render.rs"), "utf8");
  const previewStore = readFileSync(path.join(repoRoot, "crates/desktop-host/src/preview.rs"), "utf8");
  for (const token of ["sync_channel", "MAX_JS_SAFE_ID"]) if (!host.includes(token)) fail(`desktop host missing bounded IPC contract ${token}`);
  for (const token of ["MAX_PENDING", "MAX_THUMBS_PER_PHOTO", "MAX_THUMBS", "MAX_SEQUENCE"]) if (!renderer.includes(token)) fail(`desktop renderer missing bounded stale/cache contract ${token}`);
  for (const token of ["DEFAULT_MAX_ITEMS", "DEFAULT_MAX_BYTES", "DEFAULT_TTL"]) if (!previewStore.includes(token)) fail(`preview store missing bounded contract ${token}`);
  const web = readFileSync(path.join(repoRoot, "apps/lightcraft-desktop/web/src/library/libraryData.ts"), "utf8");
  if (!/MAX_SLICE = 512/.test(web) || !/generation/.test(web)) fail("library slice cache must remain generation-bound & capped");
}

checkRequiredFiles();
checkPins();
checkNativeConfig();
checkFixtures();
checkSourceContracts();
console.log("desktop static contract: PASS (source, pins, native config, fixtures, IPC/cache contracts)");
