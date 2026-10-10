# Local culling evaluation

This document freezes manifest shape, annotation rules & qualification gates for local culling. Current planner is
classical: `photo.analyze`/`photo.cullSuggest` emit `sharpness`, `clipped`, `group` & `best`; CLI `baseline` uses
read-only `photo.cullSuggest` with `pickBest: true`. No learned model is involved.

## Evidence status

No genuine consented photo annotations are present in this checkout. `corpus/raw/**`, `corpus/images/**` &
`corpus/pngsuite/**` are git-ignored test-corpus paths referenced by decoder tests; all were absent during read-only
inventory. When fetched, raw files are CC0 decode samples, with no keep/reject adjudication. `docs/images/**` are demo
assets, `fixtures/desktop/**` are desktop contract fixtures, & neither is culling evidence. No Lightroom catalog,
application data or user picks were inspected.

`tests/fixtures/culling/synthetic-v1.json`, `synthetic-cases.json` & direct score payloads are deliberately
`SYNTHETIC`: they contain IDs, labels & `synthetic://` references only. They contain no media, protected data or
pretend user choices. A real set requires documented consent, retention scope & human annotation provenance before it
can qualify a candidate.

## Frozen manifest schema (version 1)

Top-level fields:

```json
{
  "version": 1,
  "fixture_kind": "SYNTHETIC",
  "dataset_id": "synthetic-culling-v1",
  "shoots": []
}
```

Each `shoots[]` entry has `shoot_id`, `split` (`train`, `validation` or `test`), `photos[]` & `bursts[]`.
Each photo has `photo_id`, `asset_ref`, `burst_id` (string or `null`) & explicit `ground_truth` (`keep`, `reject` or
`unknown`). Each burst has `burst_id`, `photo_ids[]` & `acceptable_winners[]`. `acceptable_winners` is a set: any one
listed ID is correct, so ties or equally good frames do not create false failures. An empty set means no adjudicated
winner exists.

CLI boundary must accept these canonical snake_case keys. Compatibility aliases (`shootId`, `photoId`, `burstId`,
`groundTruth`, `acceptableWinnerIds`) may be accepted, but must normalize to this shape before scoring. Planner actions
map `pick`/`keep` to `keep`, `reject` to `reject`, & `abstain`/`review`/missing action to `unknown`.

IDs are unique within manifest. Every burst photo belongs to one shoot, one split & exactly one listed burst. Every
related frame stays in same shoot split; split is assigned per shoot, never per frame. `unknown` means no label was
given. It is excluded from keep/reject denominators & is never converted to `reject` because no pick was recorded.

For a future consented manifest, retain this shape & add a provenance object with consent record ID, annotation rubric
version, pseudonymous annotator ID, annotation timestamp & retention scope. Do not store names, catalog paths or
original media in repo fixtures.

## Held-out protocol

1. Validate IDs, splits, burst membership, winner subsets & fixture kind before scoring. Reject duplicate IDs or any
   burst crossing shoot/split boundaries.
2. Archive exact validated manifest plus rubric before candidate tuning. CLI report does not compute or emit manifest
   hashes; if an external runner records a digest, keep it beside archived inputs. Tune thresholds only on `train`; use
   `validation` once to select config; keep `test` unread until final scoring. A shoot, not an individual frame, is split
   unit, so near-duplicates cannot leak across splits.
3. Run frozen current-classical baseline with explicit threshold (`pickBest` is always true):

   ```text
   lightcraft-cli cull baseline --manifest CONSENTED_FILE_MANIFEST.json --reject-below 50 --out baseline.json
   ```

   `--reject-below` accepts finite `0..100`; replace `50` with one threshold frozen from `train`. Baseline manifest
   requires each frame's real readable `path`; synthetic label fixtures have no media paths & cannot run `baseline`.
   Candidate runs use identical shoot lists, decode policy & output fields.
4. Emit one prediction per photo: `photo_id`, `decision` (`keep`, `reject` or `unknown`), optional numeric `score` &
   optional `burst_id`. Emit one `selected_winner_id` per burst when planner commits a winner; omit it for abstention.
   An evaluator must not infer `reject` from absent `keep`, absent `best` or absent `selected_winner_id`.
5. Preserve raw output plus metrics. Record rubric version, candidate ID, config, build revision & hardware record beside
   archived inputs. `cull score` accepts supplied reports, so it does not decode media; `cull baseline` imports explicit
   manifest paths into disposable in-memory session & emits decisions for scoring.

## CLI score inputs & outputs

Score a supplied prediction payload against frozen labels:

```text
lightcraft-cli cull score --predictions tests/fixtures/culling/synthetic-predictions-test.json \
  --labels tests/fixtures/culling/synthetic-v1.json --split test --out test-report.json
```

`score` also accepts positional `PREDICTIONS LABELS`. Flat prediction payloads use `version`, `fixture_kind`, `split`,
`predictions[]` (`photo_id`, `decision`) & optional `winner_predictions[]` (`burst_id`, `selected_winner_id`). The
repository includes direct CLI payloads for unknown rejection (`synthetic-predictions-test.json`), false rejection
(`synthetic-predictions-false-reject.json`) & multiple acceptable winners (`synthetic-predictions-validation.json`). A
shoot-shaped baseline report uses `shoots[]` with `shootId`, `split`, `frames[]` (`id`, `decision`, `burstId`) &
`winnerIds[]`. `keep`/`pick` score as keep, `reject` scores as reject, & `unknown`/`abstain`/`review` scores as
unknown.

Baseline input is separate from label input & requires explicit readable paths; minimal shape is
`{"version":1,"shoots":[{"shoot_id":"s","split":"test","frames":[{"id":"p","path":"/consented/p.raw"}]}]}`.
No such path manifest or real media is committed here.

Report file has `version: 1`, `schema: "lightcraft.cull-eval.v1"`, `mode: "score"`, `split`, `metrics`, `coverage`,
`limits` & `labelPolicy`. `metrics` includes `rejectPrecision`, `rejectRecall`, `falseRejects`, `falseRejectCount`,
`falseRejectRate`, `unknownRejects`, `unknownRejectCount`, `winnerAgreement`, `winnerCoverage`, `decisionCoverage`,
`abstentions` & `elapsedMs`; `coverage` contains selected/labeled shoots & labeled/evaluated/predicted frame counts.
With `--out`, stdout only acknowledges report path; JSON report remains in named file.

Baseline report has same `version` & `schema`, `mode: "baseline"`, `source: "ephemeral-session"`,
`libraryMutated: false`, `policy` (`rejectBelow`, `pickBest`), `inputFileCount`, `timing`, `shoots`, `engineReports`,
`elapsedMs` & `limits`. Its `timing.elapsedMs` currently covers import & decode plus culling; it does not supply named
hardware or separate decode/crops/inference stages.

## Metrics

Metrics are computed per split & pooled. For frame labels, `unknown` rows are reported separately & excluded from
known-label denominators.

| Metric | Definition |
| --- | --- |
| Reject precision | `predicted reject ∩ ground-truth reject` / all predicted rejects with known labels |
| Reject recall | `predicted reject ∩ ground-truth reject` / all ground-truth rejects |
| False rejects | Count of `predicted reject ∩ ground-truth keep`; also report rate over ground-truth keeps |
| Unknown reject count | Count of `predicted reject ∩ ground-truth unknown`; never fold into false rejects |
| Decision coverage | Known-label photos with `keep` or `reject` prediction / known-label photos |
| Abstention | `unknown` predictions / all photos, plus unknown predictions on known labels |
| Winner agreement | Bursts with `selected_winner_id ∈ acceptable_winners` / bursts with non-empty acceptable set |
| Winner coverage | Bursts with selected winner / bursts with non-empty acceptable set |

For `selected_winner_id`, a listed acceptable winner is sufficient; do not require one canonical frame. Bursts without
an adjudicated winner are reported as unadjudicated & excluded from winner agreement/coverage denominators.

## Timing record

Each cold & warm run over exactly 2,000 RAW files must include `started_at`, `finished_at`, `hardware` (machine,
CPU, GPU, RAM, storage, OS), `build_revision`, `cache_state`, sample count & stage timings for `decode`, `crops`,
`inference` & `total` (p50/p95 milliseconds). Cold means fresh process with no warmed decode/crop/inference cache;
warm means same process after one documented priming pass. Use same file order & config for both. Missing timestamp,
hardware identity, stage timing or sample count makes timing status `incomplete`, never success. Current CLI baseline
only emits one elapsed duration, so supplementary runner records are required for this gate. No timing claim is made
until a 2,000-RAW corpus exists.

## Frozen gates

Harness acceptance, applicable now to synthetic fixtures:

- schema, archived input identity & split isolation validate 100%;
- unknown labels stay out of reject precision/recall & false-reject counts;
- acceptable-winner sets are honored; synthetic evaluator cases pass exact expected counts;
- baseline & candidate outputs are deterministic for same archived inputs/config;
- cold & warm timing records are either complete per schema or explicitly `incomplete`;

Candidate qualification targets, applicable only after a consented human test set is locked:

- reject precision ≥ 0.99, reject recall ≥ 0.90;
- false-reject rate ≤ 0.01 on explicitly kept photos, with zero unexplained false rejects;
- winner agreement ≥ 0.90 & winner coverage ≥ 0.90 on adjudicated test bursts;
- no safety-metric regression against frozen classical baseline, & no candidate is promoted from synthetic evidence alone;
- complete cold/warm 2,000-RAW timing records with named hardware & decode/crops/inference stages.

These are proposed qualification targets, not current product claims. Synthetic fixtures test parser, split, metric &
abstention correctness only; they do not prove culling quality. The first real annotation package must be reviewed for
consent, rubric consistency & split leakage before any numbers are treated as product evidence.
