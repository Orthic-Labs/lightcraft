# Local AI delivery

Status, 2026-10-10: source implementation & static review only. No current-revision compilation, test pass, native QA, learned-model accuracy or 2,000-RAW timing is claimed. No new dependencies or model weights were added.

## Work order

| Work | Current evidence | Next gate |
| --- | --- | --- |
| CI restoration | Fork usage suspension confirmed; API toggles/dispatch attempts recorded | Maintainer re-enable at [fork Actions](https://github.com/Orthic-Labs/lightcraft/actions), then generated CI & exact-artifact hidden Mac/Windows QA |
| Culling decisions & evaluation | Versioned classical planner, engine proposal/apply commands, CLI baseline/scorer & synthetic regressions in source | Run generated gates; lock human-labelled shoot-level test corpus |
| Face/eye & similarity weights | Primary-source licence/crop/runtime matrices; no model qualified | Pin bytes, pure-Rust Candle port, larger-crop accuracy/abstention & named-platform latency |
| Cloud BYOK comparison | Existing read-only OpenRouter experiment | Ambiguous burst comparison schema, shared transport qualification & controlled provider evaluation |
| Personal Auto | Weak Lightroom cold-start labels separated from validated Ember ground truth | Shoot-disjoint rendered comparison against deterministic Auto |
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

Replace placeholder string with actual JSON proposal object. Apply verifies catalog revision, source identity, measurements & accepted flags before one undoable batch. Stale/tampered proposals, unknown/duplicate IDs or mismatched flags fail before flag operations are committed. Empty acceptance changes no catalog flags. No original file is deleted. These are engine/CLI/MCP interfaces; React review UI remains future integration.

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
