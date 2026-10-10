# Offline MediaPipe raw-graph qualification

`crates/segment/examples/mediapipe_qualify.rs` measures pinned MediaPipe Face Landmarker v1 graph execution from an explicit finite tensor fixture. It always emits `qualification: "UNQUALIFIED"`. It does not read catalog state, load photos, convert images, choose face crops, decode detector anchors, remap 478 landmarks to 146 points, calibrate eye state, write culling decisions, or claim accuracy, quality, or throughput.

## Run

Use generated RightKit Actions with an exact pinned bundle and host description:

```text
cargo run -p lightcraft-segment --example mediapipe_qualify -- \
  --weights /path/to/face_landmarker-float16-v1.task \
  --input /path/to/raw-graph-input.json \
  --device cpu \
  --hardware "Apple M-series CPU, host label supplied by operator" \
  --out /path/to/new-receipt.json \
  --repeats 3 \
  --source-revision REVISION
```

`--weights`, `--input`, `--device`, `--hardware`, & `--out` are required. `--device` is `cpu` or macOS-only `metal`; `--repeats` is `2..=30`; `--reference` is optional; `--source-revision` is optional & bounded to 512 bytes. Output uses create-new semantics & refuses an existing path. Receipt serialization, including final newline, is capped at 4 MiB.

Loader verifies whole artifact bytes against pinned `modelSha256` & `modelBytes`. Input receipt records only raw input SHA-256, graph, shape, output shapes, timings, hardware metadata, & model identity; it never stores input paths or tensor values. Optional reference receipt records its SHA-256 & supplied provenance as unverified text, without reference values.

## Input contract

Input JSON uses strict schema `ember.mediapipe-graph-input.v1`:

```json
{
  "schema": "ember.mediapipe-graph-input.v1",
  "userConsented": true,
  "graph": "detector",
  "shape": [1, 128, 128, 3],
  "values": [0.0]
}
```

Example `values` is abbreviated for readability; real input must contain every finite F32 value in exact graph shape, with input JSON bounded to 8 MiB & 196,608 elements. Exact shapes are:

| Graph | Input shape | Raw outputs |
|---|---:|---|
| `detector` | `[1,128,128,3]` | `regressors [1,896,16]`; `classifierLogits [1,896,1]` |
| `landmarks` | `[1,256,256,3]` | `coordinates [1,1,1,1434]`; `auxiliary1 [1,1,1,1]`; `auxiliary2 [1,1]` |
| `blendshapes` | `[1,146,2]` | `coefficients [52]` |

Values are passed directly to graph APIs. No RGB/BGR conversion, scale, mean, normalization, face crop, anchor decode, sigmoid policy, landmark remapping, or eye classification is applied.

## Optional reference

Reference JSON uses strict schema `ember.mediapipe-graph-reference.v1`:

```json
{
  "schema": "ember.mediapipe-graph-reference.v1",
  "labelProvenance": "independent runner; unverified",
  "graph": "blendshapes",
  "outputs": [
    {"name": "coefficients", "shape": [52], "values": [0.0]}
  ]
}
```

Example `values` is abbreviated for readability; real reference files must contain exact output lengths. Reference graph, output names, exact shapes, lengths, & finite values must match selected graph. Receipt reports supplied/matched/model output counts plus per-output maximum absolute error & RMSE. `validation` remains `UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY`; no label, framework, or acceptance claim is made.

## Timing & qualification boundary

First-run & warm-repeat timings include graph execution & output readback. Warm repeats are compared to first outputs; any repeat maximum absolute error above `1e-3` aborts. Per-node finite checks in graph execution & output readbacks are diagnostic safety checks that synchronize GPU work, so throughput is unqualified. This harness does not provide detector anchors, face crop selection, 478-to-146 mapping, eye calibration, or culling policy. Any later qualification needs independent reference review, graph preprocessing protocol, output decoding, held-out inputs, & explicit acceptance criteria.
