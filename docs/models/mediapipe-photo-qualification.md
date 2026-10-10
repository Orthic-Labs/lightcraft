# MediaPipe still-image qualification source

Status: **source/static review only, always UNQUALIFIED**. This prototype composes exact SHA-verified v1 graphs with detector decoding, rotated face crops, 478 projected landmarks & 52 named blendshape coefficients. No compilation, inference, independent reference agreement, eye-state calibration, photographic accuracy or native throughput result is claimed.

## Image API

`mediapipe_input::RgbImage::new(width,height,rgb)` accepts upright interleaved RGB8 pixels. Dimensions are bounded to 16,384 per side & 32,000,000 pixels; byte length must match exactly. File decoding, EXIF orientation & image color conversion remain caller responsibilities.

`Bundle::analyze_image(&image,&options,&device)` uses explicit detector, weighted-NMS & presence thresholds in `[0,1]`, plus `max_faces` within `1..=32`. Device must match loaded constants. Still-image processing performs detection per image with no tracking or smoothing. Detector scores retain equality; presence uses strict `score > threshold`. Missing presence produces absent landmarks, coefficients & eye evidence, never a closed-eye label. Degenerate detected geometry returns an error before crop inference; no partial successful analysis is published.

Input policy follows [pinned image contract](mediapipe-image-contract.md): detector 128px RGB `[-1,1]`, aspect-preserving zero border; landmarks 256px RGB `[0,1]`, rotated 1.5× detector rectangle, stretch & replicate border. Floating-point bilinear sampling differs from OpenCV's discretized `INTER_LINEAR`; compare numerical behavior before asserting parity. Full-image normalized XY landmarks are converted to image pixels in exact 146-point source order for blendshape input.

Known reference differences include OpenCV's interpolation-table quantization & RGB8 warp rounding, f64 weighted-NMS accumulation rather than source f32, stable anchor-order ties where source sort leaves equal-score ordering unspecified, & rejection of nonfinite logits before clipping. Record these differences in reference comparisons.

Results retain detector boxes/keypoints, crop, source presence score, raw third mesh output (`tongue_out` metadata), projected landmarks, named coefficients & raw blink/squint/wide evidence. They have no catalog access or automatic flagging path. Eye evidence is uncalibrated expression output, not eyelid probabilities.

## Offline example

Run only through generated native RightKit Actions:

```text
cargo run -p lightcraft-segment --example mediapipe_photo_qualify -- \
  --weights EXACT_FACE_LANDMARKER_V1.task --input CONSENTED_RGB8.ppm \
  --consent yes --device cpu --hardware HARDWARE_LABEL \
  --detection-threshold 0.5 --nms-threshold 0.5 --presence-threshold 0.5 \
  --max-faces 8 --repeats 3 --out NEW_RECEIPT.json
```

Thresholds in example are experimental configuration, not measured operating points. Native CPU is available on Mac/Windows; `--device metal` is macOS-only. Required PPM is single binary P6 RGB8 image with maxval 255, bounded to 97 MiB. Explicit consent must be `yes`; duplicate/unknown options, malformed dimensions, invalid thresholds & existing output paths are refused. `--repeats 2..30` requests warm runs in addition to first pass; default 3 means four total passes. `--source-revision` records supplied bounded text, not verified Git identity.

Receipt uses `ember.mediapipe-photo-qualification-receipt.v1`. It binds exact input/model/reference SHA, supplied source/hardware/device metadata, source-policy revision, normalization/interpolation policy & thresholds. It records model-load time, first end-to-end time, warm p50/p95, repeated numerical agreement, output counts & optional ordered-reference errors. First timing is after model load; filesystem/cache state is unspecified. Diagnostic graph finite checks synchronize GPU work, so these timings require explicit qualification before production performance claims.

Receipt writes create-new with newline-inclusive 4 MiB cap. It excludes paths, RGB pixels, face geometry, landmarks & raw coefficient values. Qualification always stays `UNQUALIFIED`; shape/hash agreement alone establishes no model accuracy.

## Independent reference

Optional `--reference FILE` reads strict camelCase schema `ember.mediapipe-photo-reference.v1`:

```json
{
  "schema": "ember.mediapipe-photo-reference.v1",
  "labelProvenance": "independent pinned CPU runner; supplied unverified",
  "width": 4,
  "height": 2,
  "faces": []
}
```

Each ordered reference face supplies `detection` (`score`, normalized `[xmin,ymin,width,height]` bounds, six XY keypoints), `crop` (`center`, `size`, `rotation`), `presenceScore`, `presencePassed`, `rawAuxiliary2`, `landmarks` & `coefficients`. Presence-passed faces require exactly 478 XYZ triples & 52 `{name,value}` objects in `COEFFICIENT_NAMES` order. Presence-failed faces require absent/null landmarks & coefficients. Numeric values must be finite & meet geometry bounds. Reference input is bounded to 4 MiB & 32 faces; supplied provenance is unverified text.

Comparison follows supplied face order without correspondence matching. Cardinality, optional-state & coefficient-order mismatches produce shape errors; aligned numerical values produce maximum absolute error & RMSE. Repeat maximum absolute error above `1e-3` aborts. Reference numerical comparison is diagnostic, not an acceptance gate for culling quality.

## Remaining qualification

1. Compare preprocessing tensors, detector heads/anchors/NMS, ROI rotation/projection, mesh outputs & blendshape coefficients against independently reviewed pinned reference fixtures.
2. Freeze face/crop/presence quality & abstention rules before eye threshold selection. Evaluate larger original-image face crops, blur, pose, occlusion, partial faces & groups on shoot-disjoint human labels.
3. Measure source-qualified CPU/Metal behavior on named Mac/Windows hardware; compare classical baseline, burst winners & full 2,000-RAW workflow.
4. Pass generated CI & exact-artifact hidden RightKit QA before any desktop promotion.

Model weights, references & private image inputs stay outside Git. Apache source attribution & full license are retained in `NOTICE` & `docs/licenses/mediapipe-LICENSE.txt`.
