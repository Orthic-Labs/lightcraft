# Local AI delivery

Status, 2026-10-10: source implementation & static review only. No current-revision compilation, test pass, native QA, learned-model accuracy or 2,000-RAW timing is claimed. No new dependencies or model weights were added.

## Work order

| Work | Current evidence | Next gate |
| --- | --- | --- |
| CI restoration | Fork usage suspension confirmed; API toggles/dispatch attempts recorded | Maintainer re-enable at [fork Actions](https://github.com/Orthic-Labs/lightcraft/actions), then generated CI & exact-artifact hidden Mac/Windows QA |
| Culling decisions & evaluation | Detached cancellable desktop jobs, paged React review, explicit acceptance/undo, CLI baseline/scorer & synthetic regression source | Generated gates & exact-artifact hidden RightKit review/apply/undo journey; lock human-labelled shoot-level test corpus |
| Face/eye & similarity weights | YuNet bytes pinned & static graph inventoried; experimental Candle port; DINOv2 candidate pinned; no model qualified | Exact port/reference agreement, larger-crop accuracy/abstention & named-platform latency; choose eye-state weights |
| Cloud BYOK comparison | Strict bounded multi-image OpenRouter contract/transport in source, unknown blur/blink & reviewable outputs | Shared transport qualification, real provider credentials & controlled ambiguous-burst evaluation |
| Personal Auto | Deterministic style-residual learner, real pipeline feature extraction & offline train/evaluate CLI in source | Shoot-disjoint rendered/preference comparison against deterministic Auto; numeric distance alone cannot promote a model |
| ChatGPT-plan OAuth | Researched spec retained | Login convenience after useful photo workflows qualify |

## Reviewable culling commands

`photo.cullSuggest` accepts explicit `ids`, optional finite `rejectBelow` (0..100) & `pickBest`. It reads measurements without writing catalog analysis, flags, undo or command journal. `photo.analyze` with strict boolean `dryRun: true` uses same proposal path. Existing `photo.analyze` without dry-run remains legacy catalog analysis/flagging.

```json
{"command":"photo.cullSuggest","params":{"ids":[1,2],"rejectBelow":50,"pickBest":true}}
```

Result includes per-photo sharpness, clipping, group, unique-best marker, reason codes, qualitative uncertainty, failures & nested `proposal`. Exact sharpness ties receive review status & no pick. Low focus is technical evidence only; intentional blur may deserve keeping. Existing flags receive no replacement proposal.

Pass returned proposal unchanged, with only explicitly accepted suggestions:

```json
{"command":"photo.cullApply","params":{"proposal":"<returned proposal object>","accept":[{"id":1,"flag":"reject"}]}}
```

Replace placeholder string with actual JSON proposal object. Apply verifies catalog revision, source identity, measurements & accepted flags before one undoable batch. Stale/tampered proposals, unknown/duplicate IDs or mismatched flags fail before flag operations are committed. Empty acceptance changes no catalog flags. No original file is deleted.

Desktop `photo.cullSuggest` returns `{taskId,kind,total}` immediately. Detached workers decode without holding Session; `snapshot.status.jobs` reports progress & `completedJobs.result` contains validated proposal. `task.cancel` cancels analysis. React culling review defaults to no selected acceptance, protects existing flags, pages 50 rows at a time & cancels on close. Engine/CLI/MCP direct command dispatch retains synchronous read-only result shape.

Desktop acceptance also returns a `cullApply` task. Fresh measurement rechecks happen off-thread; live revision/source/flags are checked again before one journalled durable batch. UI waits for terminal confirmation before reporting acceptance or cancellation.

## Evaluation

```text
lightcraft-cli cull baseline --manifest FILE_MANIFEST.json --reject-below 50 --out baseline.json
lightcraft-cli cull score --predictions baseline.json --labels LABELS.json --split test --out metrics.json
```

Baseline imports only explicit manifest files into disposable in-memory session; import/decode time is included. Freeze threshold on training shoots. Scorer reports reject precision/recall, false/unknown rejects, decision coverage, abstention & acceptable burst-winner agreement/coverage. Synthetic fixtures verify contracts; they cannot qualify photographic quality. No consented annotations or real path manifest were found in checkout.

Evidence protocols:

- [Culling labels, metrics & timing](culling-evaluation.md)
- [Face/eye model candidates](culling-models-faces-eyes.md)
- [Aesthetic/quality/similarity candidates](culling-models-aesthetic.md)
- [Personal Auto evaluation](personal-auto-evaluation.md)
- [CI restoration](ci-restoration.md)
- [Model qualification checklist](models/culling-qualification-checklist.md)

## Personal Auto source harness

```text
lightcraft-cli ai personal train --manifest tests/fixtures/personal-auto/minimal.json --out NEW_MODEL.json
lightcraft-cli ai personal evaluate --manifest tests/fixtures/personal-auto/minimal.json --model NEW_MODEL.json --out NEW_REPORT.json
```

Fixture contains synthetic numeric labels only. Learner fits train shoots, checks frozen split/provenance, skips missing per-control labels & predicts bounded contrast/vibrance/saturation residuals. Lightroom weak labels & accepted Ember labels produce separate experimental variants. Extraction uses 16 fixed pipeline features, including `baseline.exposure`; it excludes camera identifiers. Reports separate validation/test shoot & camera aggregates. No production Auto replacement, render-quality claim or automatic model promotion exists.

## Detector qualification source

`crates/segment/examples/yunet_qualify.rs` reads explicit consented P6 RGB8 PPM manifests with normalized face boxes. It records known-negative vs unknown labels, shoot/split/crop slices, threshold configuration, supplied hardware, exact weight hash, model-load timing, first-per-image detection & repeated timings. First pass is not claimed as a cold-process or cold-filesystem benchmark. Metrics remain `UNQUALIFIED` until independent reference parity & held-out quality gates pass.

Run this example only through generated Actions. Required arguments: `--weights`, `--manifest`, `--device cpu|metal`, `--hardware`, `--out`, `--score-threshold`, `--nms-threshold`, `--match-iou-threshold`; optional bounded `--repeats` & `--source-revision`. It does not run eye-state inference, cull photos, change catalogs or publish private input paths.
