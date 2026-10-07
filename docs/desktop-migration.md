# Desktop migration CI & native qualification

LightCraft desktop qualification runs through generated `@rightkit/git@0.2.25` workflows in public `Orthic-Labs/lightcraft`. Local agents may run source-static checks only. Any compile, test, candidate build, package, install, signing, or qualification runs on target-native GitHub Actions runners with RightKit-managed caches, targets, receipts, & credentials.

## Owned scripts

| Script | Role |
| --- | --- |
| `scripts/desktop/static-check.mjs` | local-safe source/config/fixture contract check; no compile or test |
| `scripts/desktop/gate.sh` | generated rust-hybrid CI entrypoint |
| `scripts/desktop/gate.mjs` | React typecheck/build plus `@rightkit/release` managed Cargo `run --locked -p xtask -- ci`; retains fmt, clippy, test, parity, layers, assets, WASM |
| `scripts/desktop/admit.mjs` | validates exact main revision, SemVer, signed-qualification/publication envelope, & emits RightKit outputs |
| `scripts/desktop/stage-summary.mjs` | writes RightKit stage init/finalize evidence with source/artifact hashes |
| `scripts/desktop/candidate.mjs` | consumes generated Cargo compiler-artifact JSON, materializes unsigned target-native candidates, & verifies hashes |
| `scripts/desktop/installed-qualification.mjs` | resolves finalized compiler-artifact records, then invokes fixed `rightkit` managed Cargo Rust QA test with hidden `rightkit-control` |
| `scripts/desktop/fixtures-contract.test.mjs` | fixture contract tests for synthetic catalog, IPC/control journey, & installed rollback |

Run local static checks with:

```text
node scripts/desktop/static-check.mjs
node --test scripts/desktop/fixtures-contract.test.mjs
```

`gate.mjs`, candidate scripts, installed qualification, packaging, signing, & release publication fail outside GitHub Actions. No script fabricates native pass counts or skips missing artifact/evidence records.

## Fixtures & journeys

`fixtures/desktop/catalog.json` describes deterministic procedural RGB/CFA inputs plus synthetic Lightroom catalog metadata. Inputs are CC0-labelled, source hashes are unique, ARW import must preserve source identity, Lightroom import must preserve virtual-copy/collection identity, & CLI export must use engine encoders.

`fixtures/desktop/native-journey.json` is one shared scenario inventory consumed by target-native QA. It requires hidden control journeys through WKWebView on macOS & WebView2 on Windows at 1280×800 & 1600×1000. Scenarios cover IPC snapshot, stale preview rejection, generation-bound slices/cache, unknown-field preference persistence, coalesced gesture undo/Escape cancellation, ARW import/reload, Lightroom import/recovery, CLI export, & installed rollback.

`fixtures/desktop/installed-baseline.json` requires baseline evidence before candidate install, exact artifact/source identity, then restoration after failed install. Baseline/rollback evidence belongs to RightKit QA output and is required for release-chain qualification.

## Root integration requirements

Root owns package/config/workflow changes. Add package scripts that map these generated manifest hooks to owned scripts:

```json
{
  "desktop:static": "node scripts/desktop/static-check.mjs",
  "desktop:fixtures": "node --test scripts/desktop/fixtures-contract.test.mjs",
  "ci:desktop": "node scripts/desktop/gate.mjs",
  "release:admit": "node scripts/desktop/admit.mjs",
  "release:stage-summary": "node scripts/desktop/stage-summary.mjs",
  "release:candidate:build": "node scripts/desktop/candidate.mjs build",
  "release:candidate:check": "node scripts/desktop/candidate.mjs check",
  "release:qualify:installed": "node scripts/desktop/installed-qualification.mjs",
  "release:evidence:verify": "node scripts/desktop/evidence-verify.mjs"
}
```

`.rightgit.json` must use generated `rust-hybrid` lanes only:

```json
{
  "schemaVersion": 1,
  "compile": "github-actions-only",
  "lanes": ["ci", "release-candidate"],
  "matrix": { "os": ["ubuntu-24.04"], "node": ["26.8.1"] },
  "packageManager": "pnpm@11.24.0",
  "templateVersion": "<right-git template pin>",
  "profile": "rust-hybrid",
  "ownership": { "roots": ["."] },
  "rustCache": { "workspaces": ["."], "requiredForGate": true },
  "gate": { "path": "scripts/desktop/gate.sh" },
  "candidate": {
    "buildScript": "release:candidate:build",
    "checkScript": "release:candidate:check",
    "admissionScript": "release:admit",
    "stageSummaryScript": "release:stage-summary",
    "artifactRoot": "lightcraft-candidate",
    "targets": [
      { "os": "macos-15", "platform": "macos", "architecture": "arm64" },
      { "os": "windows-2025", "platform": "windows", "architecture": "x86_64" }
    ],
    "signWindows": false,
    "signMacos": false
  }
}
```

Root must choose exact current runner labels during `right-git` admission. Candidate target entries must remain one native macOS arm64 target & one native Windows x64 target. `rustCache.requiredForGate` stays true because desktop QA executes Rust-backed binaries. Candidate sign flags stay false; signing/finalization occur in generated protected chain.

`right-release.config.mjs` must use `@rightkit/release@0.2.124` schema 1: `targets.mac` & `targets.win` hold signed package contracts (`package: { cmd, args }`, platform signing contract, pre-package/pre-seal hooks). Candidate scripts do not invent release-config targets or execute arbitrary build commands: generated Cargo emits compiler-artifact JSON, `candidate.mjs build` resolves that record & copies hashed outputs to candidate root, while `candidate.mjs check` verifies identity, executable presence, & hashes. Unsigned candidate packaging remains separate from `right-release build`, which is signing-aware.

For local/dev-only unsigned launches, Right Release's supported shape is `development.targets.mac|win.build: [{ cmd, args }]`; generated candidate CI does not use this build command. `right-release build` requires signed `targets.*` contracts, so candidate CI keeps `signMacos`/`signWindows` false & defers package/sign/finalize to protected release stages.

Installed qualification uses fixed `runCargoSync(["test", "--locked", "-p", "lightcraft-desktop", "--test", "native_qualification", "--features", "qa-native", "--", "--nocapture"])` from `@rightkit/release/managed-cargo.mjs`. `native_qualification.rs` launches actual target binary through `rightkit_qa::control::{launch, Mode::Hidden}`, proves health/DOM/IPC/stale generation/cache/preferences/gestures, exercises native ARW/Lightroom/export path contracts, captures baseline evidence, & checks endpoint/process/background invariants. `rightkit-control::embedded::Control::<tauri::Wry>::new().build_if_enabled()` is debug/QA-gated in app host. `rightkit-qa` emits per-scenario `evidence.json` plus `summary.jsonl`; verifier requires every bundle/check passed & rollback evidence.

Build inputs must retain pinned CRAFT fonts clone & OFL licence input, with external managed Cargo target/cache roots. Package/sign/finalize remain delegated to `@rightkit/release@0.2.124`; publication remains separate after installed qualification.

No new workflow YAML is hand-authored. Root runs `right-git sync`, then `right-git drift --all`; generated CI invokes `scripts/desktop/gate.sh`, generated candidate invokes admission → build → check → stage summary, & generated protected chain invokes RightKit signing/finalization → installed qualification → evidence verification → optional publication.

## Required CI sequence

`gate.mjs` runs React typecheck/build then invokes `runCargoSync(["run", "--locked", "-p", "xtask", "--", "ci"])` from `@rightkit/release/managed-cargo.mjs`. On first CI, it runs managed `generate-lockfile`, emits bounded base64 between `LC_CARGO_LOCK_BASE64_BEGIN/END` stdout markers plus SHA-256 in job outputs/summary, & fails until committed lock is present; later gates use `--locked`. This preserves existing fmt, clippy, workspace tests, parity, layer, asset, & WASM gates for egui native/Linux/web/CLI while adding desktop contracts. Native qualification is separate: candidate artifacts are built/uploaded as an unsigned handoff, then signed/finalized by RightKit, then installed/qualified on each target-native host. No candidate step uploads or publishes.

Artifact records bind platform, architecture, source revision, artifact paths, sizes, & SHA-256. Stage summaries hash evidence files. Installed QA must verify source/artifact identity, baseline behavior, hidden IPC journeys, WebView-specific rendering, preferences, gesture transactions, ARW/Lightroom/CLI regressions, & rollback before publication can become eligible.
