# Offline DINOv2 similarity qualification

`crates/segment/examples/dinov2_qualify.rs` is a bounded, offline similarity harness. It consumes one explicit JSON manifest, consented P6 RGB8 files, one verified DINOv2 ViT-S/14 artifact, and an optional numerical reference file. It emits one create-new JSON receipt. It does not read Ember catalog state, write culling decisions, delete files, promote a model, or include input paths, EXIF, or RGB data in its receipt.

The receipt is always `qualification: "UNQUALIFIED"`. A reference comparison is evidence only: `reference.validation` remains `UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY`, and `labelProvenance` is recorded as supplied text. A maintainer must review reference parity and held-out gates before any model decision.

## Run

Use a generated Actions job with an exact artifact and a named host description:

```text
cargo run -p lightcraft-segment --example dinov2_qualify -- \
  --weights /path/to/dinov2_vits14_pretrain.pth \
  --manifest /path/to/manifest.json \
  --device cpu \
  --hardware "Apple M-series CPU, host label supplied by operator" \
  --size small224 \
  --threshold 0.80 \
  --repeats 3 \
  --source-revision REVISION \
  --out /path/to/new-receipt.json
```

`--device` is `cpu` or `metal` (Metal is macOS-only). `--size` is `small224` or `large518`; the helper uses short-edge 256 or 592, then center-crops 224 or 518. Receipt records both `inputSide` and `resizeShortEdge` from `dinov2_input-v1`: bicubic `A=-0.5`, half-pixel coordinates, edge tap normalization, floor long-edge sizing, floor-centered crop, round/clamp RGB8, then ImageNet normalization. This is Ember's bounded contract; it does not claim byte-for-byte torchvision or Pillow parity. `--threshold` is required, finite, externally supplied, and frozen status is unverified; receipt records this provenance. `--repeats` is bounded to `2..=30`. `--out` is create-new: an existing path is refused. `--source-revision` and `--hardware` are bounded receipt metadata (maximum 512 bytes).

The model loader reads one bounded file and verifies exact pinned bytes before constructing tensors. Receipt records `modelBytes` and `modelSha256` from `dinov2_artifact`, `manifestSha256`, optional reference-file SHA, and each image's PPM-file and actual RGB input SHA, plus model-load timing. Per-image timing separates first embedding from repeat p50/p95, records repeat max-absolute error and minimum cosine, and rejects repeats beyond max-absolute `1e-3` or cosine `0.999`; repeats recompute preprocessing and forward passes, while final embeddings are cached for pair scoring. These are elapsed harness timings, not cold-process, filesystem, or 2,000-RAW benchmarks.

## Manifest schema

The top level must be:

```json
{
  "schema": "lightcraft.dinov2-qualification.v1",
  "userConsented": true,
  "images": [
    {"id": "opaque-image-1", "shootId": "opaque-shoot-a", "split": "train", "ppm": "files/one.ppm"},
    {"id": "opaque-image-2", "shootId": "opaque-shoot-a", "split": "train", "ppm": "files/two.ppm"}
  ],
  "pairs": [
    {"id": "pair-1", "leftId": "opaque-image-1", "rightId": "opaque-image-2", "label": "similar"}
  ]
}
```

`id`, `shootId`, and pair `id` are opaque bounded ASCII identifiers (maximum 96 bytes, with path separators/control bytes rejected). `split` is exactly `train`, `validation`, or `test`; each shoot must occur in one split only. `ppm` is a bounded manifest-relative or absolute path used only for reading. Images use binary P6, maxval 255, exact RGB8 raster length, maximum 16,384px per side, maximum 32,000,000 pixels, and maximum 100 MiB file size. Pair references must resolve, be distinct, use one split, and never duplicate or reverse an existing endpoint pair. Pair labels are exactly `similar`, `different`, or `unknown`.

The validator caps manifest bytes, image count (10,000), pair count (50,000), IDs, PPM bytes, and pixels. It rejects duplicate IDs, unknown references, split leakage, identical RGB inputs assigned across splits, malformed P6, arithmetic overflow, and missing consent. Receipts are capped at 64 MiB. Unknown pairs are abstentions: they count in `allPairs` and `unknownPairs`, are excluded from precision/recall, and are never treated as different.

## Receipt metrics

Each image report includes supplied width/height and PPM/RGB hashes without a path. Each pair reports its opaque IDs, split, labels, pair embedding cosine, eligibility, and threshold decision. Pair embedding cosine is a similarity score; reference vector cosine fields describe numerical agreement with supplied embeddings and are not pair agreement. `overall`, `bySplit`, and `byShoot` report:

- `allPairs`, `eligiblePairs`, `unknownPairs`;
- `truePositive`, `falsePositive`, `trueNegative`, `falseNegative`;
- `coverage` (`eligiblePairs / allPairs`), precision, and recall.

Pairs whose endpoints use different shoots are grouped under the synthetic `cross-shoot` report key; that shoot ID is reserved in manifests, and no pair is duplicated. Shoot-disjoint split validation still applies to every image.

## Optional reference embeddings

Pass `--reference-embeddings FILE` for explicit numerical comparison:

```json
{
  "schema": "lightcraft.dinov2-reference-embeddings.v1",
  "labelProvenance": "supplied by independent reference runner; unverified",
  "embeddings": [
    {"id": "opaque-image-1", "values": [0.0, 0.0, 0.0]}
  ]
}
```

The reference list must be non-empty, contain known image IDs, and each `values` array must contain 384 finite numbers. The example reports supplied count, matched count, model-embedding total, explicit matched/model coverage, maximum absolute error, RMSE, minimum reference-vector cosine, and mean reference-vector cosine. The example does not validate labels, reference provenance, framework parity, or acceptance thresholds automatically. A real file must contain all 384 values; the abbreviated array above is schema-shaped illustration only.

## Qualification boundary

This harness measures pairwise embedding similarity only. It does not infer quality, focus, face state, aesthetic value, or culling decisions. `small224` and `large518` are separate preprocessing conditions; do not combine their scores without a frozen protocol. Qualification requires independent numerical reference parity, supplied label review, frozen threshold selection on training shoots, and held-out validation/test shoot gates. Until those gates are reviewed, downstream Ember code must treat this receipt as experimental evidence and abstain on unknowns.
