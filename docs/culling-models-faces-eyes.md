# Face & eye models for learned culling

Status: qualification research only. Exact YuNet & MediaPipe bytes were fetched only into outside-repo scratch for
static inspection; no weights were added, converted, benchmarked or executed. No model is qualified for Ember shipping.
`candle-core`/`candle-nn` are pinned to `0.9.2`; runtime must stay pure Rust, with no C++ or ONNX Runtime dependency.

## Decision

Best bounded qualification candidate: **MediaPipe Face Landmarker float16 bundle, URL version `1`**. Its official
guide describes one task bundle with face detection, 478 3D landmarks & 52 blendshape coefficients, including
`eyeBlinkLeft`/`eyeBlinkRight` & `eyeWideLeft`/`eyeWideRight`. Official model cards state Apache-2.0 for
BlazeFace, FaceMesh-V2 & Blendshape weights. This gives one candidate for face boxes, landmarks & eye-state
features; **no model is qualified yet**.

Exact bundle receipt is recorded in [`models/mediapipe-face-landmarker-v1-qualification.json`](models/mediapipe-face-landmarker-v1-qualification.json): 3,758,596 bytes, SHA-256
`64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff`, ZIP entries & child hashes. Pinned version-1
graph input is detector `[1,128,128,3]` FLOAT32, FaceMesh `[1,256,256,3]` FLOAT32 & Blendshape `[1,146,2]`
FLOAT32; current guide table lists 192x192 for its current model row, so guide dimensions must not replace exact
artifact facts. FLOAT16 occurs in constant tensors followed by TFLite `DEQUANTIZE` ops. `num_faces` is positive
integer; temporal smoothing applies only when `num_faces = 1`.

**YuNet 2023mar** is best small detector fallback: five facial keypoints & MIT model-directory license. This deliberately pins older fixed-shape artifact; OpenCV later added
`2026may` dynamic re-export in [commit `47534e2`](https://github.com/opencv/opencv_zoo/commit/47534e27c9851bb1128ccc0102f1145e27f23f98), which is not selected. It cannot classify eye-open state; pair it with separately qualified landmarker or use geometric eye-state only after validation. YuNet's WIDER Face training-data rights are not stated in model README, so data provenance remains a release gate.

Static inspection of exact pinned ONNX (inventory in [`models/yunet-2023mar-graph.json`](models/yunet-2023mar-graph.json)) records fixed `[1,3,640,640]` FLOAT32 input, 106 nodes, 112 initializers, 53,112 FLOAT initializer elements, & 12 raw heads: `cls_8/16/32`, `obj_8/16/32`, `bbox_8/16/32`, `kps_8/16/32`. Graph operators are Conv, MaxPool, Relu, Resize, Add, Transpose, Reshape & Sigmoid; it has no image color/scale normalization, decoded boxes or NMS. Paper reports 75,856 parameters, 149 MFLOPs & 320x320; those source-paper figures stay separate from this artifact-level graph inventory. Candle port must freeze external preprocessing, decode, score fusion & NMS against this exact graph.

All Ember accuracy & latency fields below are **UNMEASURED**. Paper/model-card numbers are source-reported only;
they are not Ember results, confidence calibration or ms/frame claims. No named Mac or Windows held-out larger-crop
evaluation exists yet.

## Candidate table

| Candidate/version & role | Code license; weight license; data license | Published size/params & input | Pure-Rust Candle fit & porting work | Blur, occlusion, multiple faces & unknown handling | Qualification result |
|---|---|---|---|---|---|
| [MediaPipe Face Landmarker](https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker), exact URL [`face_landmarker/float16/1`](https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task): detector + 478 landmarks + blink/wide blendshapes | MediaPipe repo [Apache-2.0](https://github.com/google-ai-edge/mediapipe/blob/master/LICENSE); official [BlazeFace card](https://storage.googleapis.com/mediapipe-assets/MediaPipe%20BlazeFace%20Model%20Card%20(Short%20Range).pdf), [FaceMesh-V2 card](https://storage.googleapis.com/mediapipe-assets/Model%20Card%20MediaPipe%20Face%20Mesh%20V2.pdf) & [Blendshape card](https://storage.googleapis.com/mediapipe-assets/Model%20Card%20Blendshape%20V2.pdf) each state Apache-2.0 for model. Cards describe consented smartphone imagery for detector/mesh & synthetic GHUM-derived blendshape samples; no separate data redistribution grant is stated. | Exact v1 static graph inventory: detector `[1,128,128,3]` -> `[1,896,16]` + `[1,896,1]`; mesh `[1,256,256,3]` -> `[1,1,1,1434]`, `[1,1,1,1]`, `[1,1]`; blendshape `[1,146,2]` -> `[52]`. Bundle size/hash & operator inventory are pinned in [`models/mediapipe-face-landmarker-v1-qualification.json`](models/mediapipe-face-landmarker-v1-qualification.json). | `.task` is a ZIP/FlatBuffer bundle with TFLite graphs, not Candle weights. Port builtin Conv2D, depthwise Conv2D, Add, Pad, Pool, Reshape, activation, concat, reductions, transpose & `DEQUANTIZE`; reproduce detector decode/NMS, mesh crop remap & blendshape normalization in Candle 0.9.2. No C++/ONNX Runtime. Medium/high port. | Guide supports `num_faces > 1`; smoothing only applies at `num_faces = 1`. FaceMesh card expects one centered face crop with 25% margin, <=8° roll & sensitivity to position/scale/orientation. Cards report degradation with blur/noise/motion/overlap. Use detector/presence thresholds; emit `unknown` when face presence, landmark geometry or eye blendshape quality is below frozen gates; never turn missing landmarks into closed eyes. | **Best qualification candidate; not qualified**. Exact bytes & graph inventory recorded; eye coefficients still need threshold/held-out validation. |
| [YuNet `face_detection_yunet_2023mar.onnx`](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/README.md): face detector + 5 keypoints | OpenCV Zoo is Apache-2.0; pinned model directory [README blob `4097ac9b0ab6c83e98b994697fb1449a5637e408`](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/README.md) says **MIT for all files**, with pinned [LICENSE blob `4cdf89a445b7e84d2352700309440f8428be5f71`](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/LICENSE). Immutable repo commit is [`f12e12798e8314f7c074a6656816c048dcc95b7a`](https://github.com/opencv/opencv_zoo/commit/f12e12798e8314f7c074a6656816c048dcc95b7a). Paper/model README name WIDER Face evaluation; training-data redistribution/commercial grant is not stated. | [Paper](https://link.springer.com/article/10.1007/s11633-023-1423-y): 75,856 params, 149 MFLOPs, 320x320; exact graph inventory reports fixed `[1,3,640,640]` FLOAT32 input, 106 nodes, 112 initializers & 53,112 FLOAT initializer elements; official Git LFS pointer is pinned by model blob `2d8804a5986e229f1fde3a1994feacc66c91b58b`, LFS OID `sha256:8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4`, size 232,589 bytes (227 KiB). Outputs three stride heads for class/objectness, boxes & five keypoints; input preprocessing, decode & NMS are external. | ONNX graph is not consumable directly by Candle. Recreate depthwise/pointwise Conv graph, MaxPool, nearest Resize, Add, Transpose, Reshape & Sigmoid; implement external preprocessing, decode, score fusion & NMS. No NMS node exists in graph. Candle has required conv/depthwise primitives. Low/medium port. | Designed for multiple faces & external NMS. README gives scale range, but no model-card blur/occlusion guarantees; treat blur, profile, overlap & tiny/partial faces as **unknown until eval**. Five keypoints locate eyes but cannot establish open/closed. Abstain below detector threshold or invalid geometry. | **Detector fallback / qualification candidate**. Exact digest/license clarity; artifact input shape differs from paper figure, so freeze graph contract before port. |
| [SCRFD-500M-KPS](https://github.com/deepinsight/insightface/blob/master/detection/scrfd/README.md): detector + 5 keypoints | InsightFace code MIT, but official [model policy](https://github.com/deepinsight/insightface/blob/master/python-package/docs/model_zoo.md) says pretrained models are **non-commercial research only**. Training data/model-derived rights are not a commercial grant. | Official table: 0.57M params, 500M FLOPs, VGA evaluation, 5-keypoint variant; download is an unpinned OneDrive/GDrive artifact. No official SHA-256. | MobileNetV1 + PAFPN + depthwise head, GN, multi-stride anchors, quality focal/DIoU decode & NMS. Candle can express layers, but graph export/conversion & postprocess are medium work; no ONNX Runtime in product. | Multi-face detector/keypoints. README has no blur/occlusion/eye-state guarantees; keypoints cannot classify eye-open. Low scores, invalid boxes or missing keypoints must be `unknown`. | **Reject for Ember distribution**: weight license fails commercial/permissive gate despite compact architecture. |
| [3DDFA-V2 MobileNet](https://github.com/cleardusk/3DDFA_V2): 68-point sparse/3D alignment after external detector | Repo code MIT; default `weights/mb1_120x120.pth` & `mb05_120x120.pth` have no separate weight license or SHA-256 in upstream README. FAQ names 300W-LP training data; data license/provenance grant is not recorded there. | Official README: 120x120 crop, MobileNet 3.27M params or x0.5 0.85M; 68 2D landmarks plus 3DMM outputs. Paper reports 6.2 ms TF CPU / 19.2 ms one-core in its own hardware; not Ember timing. | PyTorch checkpoint conversion, MobileNet/3DMM parameter decode & external FaceBoxes detector; no C++/ONNX Runtime permitted. Medium/high port. | External detector supports multiple faces; alignment expects per-face crop. Upstream explicitly says eyes are inaccurate when closed because 300W-LP has few closed-eye samples. Motion/pose >90° can fail; failure or low pose/landmark quality => `unknown`. | **Reject pending relicensing/data review**. Useful eye-landmark failure case, not a qualified eye-state model. |
| [PFLD 0.25X](https://arxiv.org/abs/1902.10859): sparse landmark regressor | [Official implementation](https://github.com/guoqiangqi/PFLD) has no LICENSE file; paper's public checkpoint link has no immutable weight license or SHA-256. Paper evaluates 300W/AFLW; dataset rights are separate & no model grant is stated. | Paper reports 2.1 MB practical model & >140 fps on Qualcomm 845; landmark crop/input details are not fixed by a model card. Public implementations commonly use 112x112 crops, but that is not an upstream release contract. | MobileNet-like PFLD, auxiliary pose head, checkpoint conversion & crop contract; medium port. | Single-face crop, not a detector; paper trains for pose/light/occlusion but provides no eye-open output. Derive eye aspect ratio only when eye landmarks pass quality checks; otherwise `unknown`. | **Reject**: absent code/weight license & checksum make redistribution unqualified. |

### License reading rule

Inference code, model weights & training data are separate artifacts. A permissive repository license does not cure
research-only weights, absent weight terms, absent checksums or unknown dataset rights. No GPL/LGPL/AGPL or Adobe
source/assets are included in this review.

## Bounded pure-Rust prototype

1. Use exact MediaPipe receipt in [`models/mediapipe-face-landmarker-v1-qualification.json`](models/mediapipe-face-landmarker-v1-qualification.json); preserve source URL, retrieval date, model-card URLs, notices & embedded metadata. Do not substitute current mutable bytes.
2. For YuNet fallback, use the static [graph inventory](models/yunet-2023mar-graph.json) to map exact input/output
   shapes & operator attributes; freeze external float32 preprocessing, three stride-head decode, score fusion & NMS
   before any Candle implementation. Do not place model tensor values in repository.
3. Build an offline MediaPipe converter that extracts each TFLite tensor to safetensors, records source tensor name/shape/dtype,
   then verifies every expected tensor count, shape & output against a reference fixture. Port exact v1 detector 128x128,
   mesh 256x256 & blendshape 1x146x2 graphs to Candle 0.9.2; implement FLOAT16 `DEQUANTIZE`, keep preprocessing,
   crop margins & coordinate transforms explicit.
4. Add deterministic postprocessing: multi-face NMS, face-presence threshold, landmark finite/range checks, eye
   blendshape extraction & `unknown` on no face, partial/occluded face, invalid crop or low quality. No calibrated
   probability claim; thresholds are qualification parameters.
5. Compare against YuNet 2023mar as detector-only baseline using same larger image crops. Record source-paper numbers
   separately from Ember outputs. Run held-out, consented larger-crop set on one named Mac & one named Windows machine;
   report detection/landmark/eye-state precision, recall, abstention, false-reject rate & cold/warm timing only after
   both platforms have complete records per [culling evaluation protocol](culling-evaluation.md).
6. Qualify for learned culling only if existing gates pass: reject precision >=0.99, reject recall >=0.90,
   false-reject rate <=0.01 on explicit keeps, winner agreement/coverage >=0.90, complete 2,000-RAW timing records,
   & legal artifact record. Keep `unknown` out of known-label denominators.

## Training fallback

If exact MediaPipe bundle terms, digest or conversion cannot be frozen, train an own compact model. Use a YuNet-like
depthwise detector with five landmarks, then a 112–128px per-face landmark + eye-state head. Train only from consented,
documented images plus original procedural/synthetic images; annotate face box, five landmarks, each eye state
(`open`, `closed`, `occluded`, `unknown`) & blur/visibility. Release code, weights & annotations under chosen
permissive terms only after checking every source. Hold out shoots, not frames, & preserve `unknown` for ambiguous
eyes. This removes third-party weight/data uncertainty while retaining a small Candle-friendly architecture.

## Sources

- [MediaPipe Face Landmarker guide](https://ai.google.dev/edge/mediapipe/solutions/vision/face_landmarker) — bundle inputs,
  outputs, `num_faces`, thresholds & model URL.
- [MediaPipe v1 static bundle receipt](models/mediapipe-face-landmarker-v1-qualification.json) — exact bytes, child
  hashes, ZIP contents, TFLite inputs/outputs, tensor dtypes & builtin operator inventories; values omitted.
- [Official TensorFlow Lite schema](https://github.com/tensorflow/tflite-micro/blob/main/tensorflow/lite/schema/schema.fbs) —
  FlatBuffer schema used for static graph parsing.
- [BlazeFace short-range model card](https://storage.googleapis.com/mediapipe-assets/MediaPipe%20BlazeFace%20Model%20Card%20(Short%20Range).pdf) —
  224 KB card, 128x128 standalone detector, Apache-2.0, limitations & source-reported metrics.
- [FaceMesh-V2 model card](https://storage.googleapis.com/mediapipe-assets/Model%20Card%20MediaPipe%20Face%20Mesh%20V2.pdf) —
  Apache-2.0, 256x256 crop, 478 landmarks, visibility/pose limitations & evaluation.
- [Blendshape-V2 model card](https://storage.googleapis.com/mediapipe-assets/Model%20Card%20Blendshape%20V2.pdf) — Apache-2.0,
  146-landmark input, 52 outputs including eye blink/wide, occlusion/noise limitations.
- [YuNet pinned OpenCV Zoo README](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/README.md),
  [pinned model LICENSE](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/LICENSE),
  [pinned model pointer](https://github.com/opencv/opencv_zoo/blob/f12e12798e8314f7c074a6656816c048dcc95b7a/models/face_detection_yunet/face_detection_yunet_2023mar.onnx) &
  [YuNet paper](https://link.springer.com/article/10.1007/s11633-023-1423-y) &
  [static graph inventory](models/yunet-2023mar-graph.json) — immutable commit `f12e12798e8314f7c074a6656816c048dcc95b7a`,
  README blob `4097ac9b0ab6c83e98b994697fb1449a5637e408`, LICENSE blob `4cdf89a445b7e84d2352700309440f8428be5f71`,
  model blob `2d8804a5986e229f1fde3a1994feacc66c91b58b`, LFS OID
  `sha256:8f2383e4dd3cfbb4553ea8718107fc0423210dc964f9f4280604804ed2552fa4`, 232,589 bytes.
- [SCRFD README](https://github.com/deepinsight/insightface/blob/master/detection/scrfd/README.md) & [InsightFace model policy](https://github.com/deepinsight/insightface/blob/master/python-package/docs/model_zoo.md) —
  architecture figures & non-commercial pretrained-model terms.
- [3DDFA-V2 README](https://github.com/cleardusk/3DDFA_V2/blob/master/readme.md) & [paper](https://arxiv.org/abs/2009.09960) —
  crop/model sizes, source-reported timings, closed-eye limitation & 300W-LP provenance.
- [PFLD paper](https://arxiv.org/abs/1902.10859) & [official implementation](https://github.com/guoqiangqi/PFLD) —
  compact model claim, occlusion-focused training description & absent upstream license/checksum.
