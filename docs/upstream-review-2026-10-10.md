# Upstream absorption review — 10 October 2026

Decision: selectively port engine, RAW & correctness fixes. Preserve Ember's React/Tauri/RightKit host, branding, generated release gates & reviewable culling contracts. Feature migrations belong in separate batches with compatible host bindings.

## Snapshot & proof

- Shared revision: `b1883231bdf0af341541de1ace336c6bf530571f` (integration batch 13).
- Reviewed upstream: [`b70ae1741e8236757377ca85eecaafb8788a82ab`](https://github.com/storytold/lightcraft/commit/b70ae1741e8236757377ca85eecaafb8788a82ab), [PR #692](https://github.com/storytold/lightcraft/pull/692), integration batch 25.
- Fork source at review: `39a95cefa5c8f5a6317db093f0e02d0248a8d73a`; domain review began at parent `64c88faf`. Wave 4 adds offline DINOv2/source regressions & licences without changing reviewed RAW/develop/catalog behavior.
- Range contains 586 commits including merges, 425 non-merge patches. No complete patch is equivalent by `git cherry`; fork commit `6662d788` nevertheless carries selected hunks, so ancestry/patch counts cannot establish which behavior is missing.
- Exact upstream merged revision has green [Windows/package](https://github.com/storytold/lightcraft/actions/runs/38040715225) & [FreeBSD](https://github.com/storytold/lightcraft/actions/runs/38040715272) runs. These validate upstream configuration; they do not validate Ember's host, local changes, model quality or Mac Metal behavior.
- Ember Actions remains usage-suspended. No compilation, tests, native QA, model inference or upstream merge was performed during this review.

Ten Luna reviews covered CI, dependencies, two RAW groups, engine/import jobs, catalog, develop/Auto, preview/GPU, denoise/HDR & faces/masking. Recommendations below reflect source review plus available upstream checks.

## Absorption order

| Batch | Decision | Source group | Integration requirement |
| --- | --- | --- | --- |
| GPU shutdown | Adapt first | [`5a537d5d`](https://github.com/storytold/lightcraft/commit/5a537d5d) | Port GPU exit gate, engine closing check & bounded preview-worker shutdown; wire through `desktop-host`. Test close during active render, GPU-to-CPU fallback & forced close. |
| Lightroom crop import | Adapt first | [`fcef4c0f`](https://github.com/storytold/lightcraft/commit/fcef4c0f), [`2ec2b5bf`](https://github.com/storytold/lightcraft/commit/2ec2b5bf), [`369c56a6`](https://github.com/storytold/lightcraft/commit/369c56a6) | Handle missing `HasCrop`, orientation & straightening using fork's existing `crs::to_partial_report` API. Preserve one durable undo batch & wrong-library protection. |
| Canon RAW | Adapt narrow groups | [`1206d3fd`](https://github.com/storytold/lightcraft/commit/1206d3fd), `3af7a0c0`, `8d96dab1`; [`a45c2b66`](https://github.com/storytold/lightcraft/commit/a45c2b66); [`04e8aea5`](https://github.com/storytold/lightcraft/commit/04e8aea5) | Add sRAW/mRAW, EOS R7 odd-height C-RAW & CR3 aspect crop. Update Sony caller when lossless-JPEG API changes; retain overflow checks & fork-specific cache invalidation. |
| Nikon, Pentax & Olympus | Adapt narrow groups | [`4ece03a5`](https://github.com/storytold/lightcraft/commit/4ece03a5), `a12f1697`, `3d9f8492`, `cf9d8f7e`; `98942239`, `c7719ddc`, `1399c445`; [`37d36215`](https://github.com/storytold/lightcraft/commit/37d36215), `44b12627`, `82566817` | Keep exact packing/strip geometry, CFA phase, bounded row allocations & explicit unsupported Nikon High Efficiency behavior. Olympus high-resolution samples still need independent color/CFA checks. |
| Sony, Samsung & Hasselblad | Adapt after shared RAW changes | [`5ed59252`](https://github.com/storytold/lightcraft/commit/5ed59252), `cfca2999`, `f1fa336a`, `d1382362`, `4a3f9e9b`, `4a7eb6b3`; [`d9fd2f8a`](https://github.com/storytold/lightcraft/commit/d9fd2f8a); `79b19fd8`, `621dd51c` | Take complete final Sony crop/SR2 corrections, not intermediate refusals. Keep compressed SRW unsupported. Transplant format-discrimination hunks instead of integration merges. Gate Sony lens corrections to validated bodies. |
| Capture dates & deletion | Adapt small safety fixes | [`4266b0ad`](https://github.com/storytold/lightcraft/commit/4266b0ad), `7ba14709`, `11e36bf0`; [`03cb96f8`](https://github.com/storytold/lightcraft/commit/03cb96f8), `c0ff290e`, `35718bc9` | Reject invalid/out-of-range date shifts atomically. Preserve descendants, selection & one undo batch when deleting folders/photos. |
| Highlights, camera color & Auto | Adapt complete groups, then refit | [`3539de95`](https://github.com/storytold/lightcraft/commit/3539de95), [`0ac5cf44`](https://github.com/storytold/lightcraft/commit/0ac5cf44); `b4ab88b7`, `9abf0d42`, `41a61b74`, [`aa6dfe3b`](https://github.com/storytold/lightcraft/commit/aa6dfe3b), [`ac931189`](https://github.com/storytold/lightcraft/commit/ac931189) | Include process-aware Auto, camera-space WB & full RAW/engine/GPU highlight sequence. Camera fitting/DRO changes require paired preview invalidation, branch-local cache bump & fresh baseline/model receipts. Sony-only reported calibration is insufficient evidence of general Lightroom parity. |
| Preview refresh & export | Conditional adaptation | [`04e3b2f1`](https://github.com/storytold/lightcraft/commit/04e3b2f1), [`7a65cc63`](https://github.com/storytold/lightcraft/commit/7a65cc63); [`56307348`](https://github.com/storytold/lightcraft/commit/56307348) | Bring look-version refresh only with camera-look changes; serialize builds & retain offline previews. Bounded export lanes need current host progress/cancel & memory budgeting. |
| Synchronize Folder | Adapt as full feature batch | [`eaa8828b`](https://github.com/storytold/lightcraft/commit/eaa8828b), `5345f0a3`, `1e580466`, `90a38718`, `05178a78`, `8233d1e4` plus reviewed followups | Use displayed scan, split worker/owner work, library identity & undo ownership. Bind into existing host jobs rather than adding another task registry. |
| Faces/YuNet | Adapt reference behavior & management | [`20a801d6`](https://github.com/storytold/lightcraft/commit/20a801d6d20243038658514532f0d93461b6e6b3), [`upstream detector`](https://github.com/storytold/lightcraft/blob/b70ae1741e8236757377ca85eecaafb8788a82ab/crates/faces/src/yunet.rs) | Reconcile preprocessing, decode/NMS & immutable identity with current Candle candidate. Reuse staging/verification concepts. Avoid adding a second ONNX runtime without measured benefit. YuNet/recognition do not classify blinks. |
| DNG JXL & semantic mattes | Separate codec/mask batches | [`1ed80c65`](https://github.com/storytold/lightcraft/commit/1ed80c65), `e253ec5f`, `82f90449`; [`ed59d78e`](https://github.com/storytold/lightcraft/commit/ed59d78e837ea806662694340a83508e6f9ccb44) | Include lossy restoration/deinterleave followups, bounded tile failure, source-info changes & matte crop/orientation/cache integration. |
| Catalog v3–v6, keywords & rich rules | Defer schema changes to their features | `39f5f114`, `cd745e88`, `21cd3b75`, `309d659b`; `d3730a70` and rule-validation followups | Fork uses v2; upstream ultimately uses v6. Intermediate feature commits reuse v4, so direct cherry-picks can corrupt migration/op numbering. Preserve old loading, rewrite-before-append & reject-newer behavior. Extend Rust snapshots & React types together. |
| HEIF | Optional later | [`853ada77`](https://github.com/storytold/lightcraft/commit/853ada77), `90361935`, `d0749ed7` | Pure-Rust `heic-rs` decoder still needs capability, malformed-input & distribution review. Keep optional, preserve unsupported chroma/HDR/depth limits; native panic guard does not protect WASM abort. |
| Denoise & HDR | Defer full feature integration | [`7935999d`](https://github.com/storytold/lightcraft/commit/7935999d), `fd401930`, `bec18415`; [`e11db8fe`](https://github.com/storytold/lightcraft/commit/e11db8fe), `7828f93c`, `88de143d` | Denoise implementation is pure Rust, but optional RawNIND weights are GPL-3.0; do not inherit download/default feature policy. Permissive weights remain needed. HDR needs React export controls, linear/PQ/gain-map contracts & fidelity tests. |

## Keep existing fork choices

`crates/engine/src/lightroom_sqlite.rs` is byte-identical to reviewed upstream. Keep custom bounded reader; no replacement SQLite dependency is needed. Current Lightroom import already protects open-library identity & persists through owner command/undo path.

Upstream Activity is another task registry, not a drop-in for `desktop-host::Tasks`; importing it alongside current jobs would duplicate progress/cancellation state. Review UI behavior through engine adapters. Preserve Ember branding, RightKit generated workflows, React/Tauri host, lockfiles & `photo-ai` contracts.

Current slider coalescing/cancellation fixes already exist in fork. Upstream's >2048px egui zoom fix does not address same React canvas path. Cache versions are fork-local: increment current value when imported pixel behavior changes; do not copy upstream's absolute version.

## Auto evidence & Personal Auto impact

Auto tuning commit `8b9a0200` contains six procedural tests, but no reproducible 24-Sony-RAW evaluation manifest, source hashes or reference exports. Its numerical comparison is developer-reported. Sony DRO correction `5a61fc5f` is a sibling change that alters source tone after fitting; reported DRO examples already underperform an unedited baseline. Treat calibration as a candidate to evaluate, not a proven universal improvement.

Camera-fit changes `cc7bf0e2`, `c691bcfa`, `24427912`, `7408f11f` & `c21c305e` belong together; DRO must retain its explicitly attributed CC0 input. No reference-product assets belong in Ember. Refit Auto after changing RAW reconstruction, WB, camera looks, process behavior or fitting, then re-extract/retrain Personal Auto. Experimental extraction now binds source pixels, source/settings facts, canonical numeric inputs & renderer identity through baseline receipts. Matching supplied receipts establishes consistency; held-out rendered comparisons remain required before any render-quality claim.

## Qualification before desktop adoption

For each small batch: retain relevant upstream regression source, resolve host/API differences, run generated workspace/native gates at exact fork SHA, then hidden RightKit QA on resulting Mac/Windows artifacts. RAW batches additionally need pinned corpus renders; Auto/highlight/color changes need held-out render comparison. Face/eye/model accuracy remains unqualified until labelled larger crops & named-platform measurements exist.

## Source adaptations landed

- [`77efdbf9`](https://github.com/Orthic-Labs/lightcraft/commit/77efdbf9): Olympus capability reporting matches supported ORF containers.
- [`496a555f`](https://github.com/Orthic-Labs/lightcraft/commit/496a555f): Sony sample/frame/strip size arithmetic rejects overflow before allocation or slicing.
- [`5f1b6ca9`](https://github.com/Orthic-Labs/lightcraft/commit/5f1b6ca9): Lightroom catalog crops infer missing `HasCrop`, map eight orientations & straighten with stored aspect; malformed geometry is skipped with warnings. One import undo batch is retained.
- [`6f4c997d`](https://github.com/Orthic-Labs/lightcraft/commit/6f4c997d): Final GPU shutdown gate, bounded preview cleanup, terminal worker failures & unique submission identity. Ember's Tauri & egui hosts own process exit; disposable host shutdown/disconnect keeps process GPU gate open.
- [`8cffadd3`](https://github.com/Orthic-Labs/lightcraft/commit/8cffadd3): Personal Auto renderer/input receipts reject mutated numeric inputs, stale settings/profile/cache identities & partial receipt sets; old model envelopes require retraining.

Rust 2024 formatting, diff checks & desktop static contracts pass. Luna adversarial source reviews
covered crop geometry, shutdown races/lifetime & receipt binding. Procedural regressions were added;
none were executed locally. Current fork compilation, native QA, real RAW renders & held-out model
quality still await generated RightKit Actions plus consented evaluation photos. Installed desktop
remains on its previously qualified revision. Larger model, codec & catalog features retain separate
qualification gates.
