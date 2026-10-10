# MediaPipe Face Landmarker v1 image contract

This is a static contract for exact task bytes `3,758,596` bytes, SHA-256
`64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff` from the
versioned [Face Landmarker float16/1 task](https://storage.googleapis.com/mediapipe-models/face_landmarker/face_landmarker/float16/1/face_landmarker.task).
The task is a stored ZIP containing four entries. Child hashes are recorded in
the [artifact inventory](mediapipe-v1-artifact-inventory.md) and qualification
receipt. No inference, float-weight read, accuracy claim, or performance claim
is made here.

## Image tensors

The model's TFLite metadata was read from the exact child FlatBuffers. Metadata
normalization is `(x - mean) / std`, per the pinned
[metadata schema](https://raw.githubusercontent.com/google-ai-edge/mediapipe/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/metadata/metadata_schema.fbs#L388-L403).
The detector and landmark inputs are FLOAT32 NHWC RGB tensors; blendshape
input is a FLOAT32 `[1,146,2]` landmark-coordinate tensor.

| graph | metadata model/input | shape | mean | std | byte-domain result |
| --- | --- | --- | ---: | ---: | --- |
| detector | `Short Range Face Detection` / `image` | `[1,128,128,3]` | `127.5` | `127.5` | RGB `[0,255]` → `[-1,1]` |
| landmarks | `Face mesh detection model v2` / `image` | `[1,256,256,3]` | `0` | `255` | RGB `[0,255]` → `[0,1]` |

The detector graph configures `keep_aspect_ratio=true` and
`border_mode=BORDER_ZERO` ([pinned graph](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_detector/face_detector_graph.cc#L343-L365)).
The generic `ImageToTensorCalculator` proto defaults `keep_aspect_ratio` to
false (proto2 bool absent) and documents `BORDER_REPLICATE` as default; pinned
conversion maps `BORDER_UNSPECIFIED` to replicate
([proto](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/image_to_tensor_calculator.proto#L50-L85),
[mapping](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/image_to_tensor_utils.cc#L242-L252)).
The landmark graph does not override either option, so its generic defaults are
stretch (`keep_aspect_ratio=false`) + replicate border. In still-image mode,
initial face ROI comes from detector's
`RectTransformationCalculator`, configured only with `scale_x=scale_y=1.5`
(no `square_long`) at
[`face_detector_graph.cc#L249-L253`](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_detector/face_detector_graph.cc#L249-L253),
then supplied as `EXPANDED_FACE_RECTS` to landmark graph. The
`square_long=true` setting at
[`face_landmarks_detector_graph.cc#L147-L154`](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/face_landmarks_detector_graph.cc#L147-L154)
belongs to `FACE_RECT_NEXT_FRAME`, a tracking/video prediction path, not
initial still-image crop.

For CPU reference parity, pinned OpenCV conversion uses
`cv::warpPerspective` with `cv::INTER_LINEAR`
([converter](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/image_to_tensor_converter_opencv.cc#L135-L173),
[default flag](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/image_to_tensor_converter_opencv.h#L27-L31)).

## Outputs and ordering

Detector output order is `regressors` `[1,896,16]` (metadata name
`raw boxes/keypoints`) followed by `classificators` `[1,896,1]` (metadata name
`scores`). These are raw model heads; anchor decode, sigmoid, threshold, and
NMS happen in graph calculators.

Static model metadata gives these landmark child outputs, in TFLite output
order: `Identity` `[1,1,1,1434]` (478 xyz triples), `Identity_1`
`[1,1,1,1]` (presence logit), `Identity_2` `[1,1]` (`tongue_out`). The
metadata names are respectively `face_landmarks`, `presence`, `tongue_out`.
The graph splits and consumes only first two outputs: first is landmarks,
second is presence; third is not consumed by
`SingleFaceLandmarksDetectorGraph`
([pinned graph](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/face_landmarks_detector_graph.cc#L285-L323)).

Landmark index order is model order `0..477`; the pinned tensor-to-landmarks
graph sets `num_landmarks=478` and reads each triple in order
([pinned mapping graph](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/tensors_to_face_landmarks_graph.cc#L47-L52),
[decoder](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/tensors_to_landmarks_calculator.cc#L141-L180)).
The blendshape graph selects this exact 146-index subset, in listed order:

```text
0,1,4,5,6,7,8,10,13,14,17,21,33,37,39,40,46,52,53,54,55,58,61,63,65,66,
67,70,78,80,81,82,84,87,88,91,93,95,103,105,107,109,127,132,133,136,144,
145,146,148,149,150,152,153,154,155,157,158,159,160,161,162,163,168,172,
173,176,178,181,185,191,195,197,234,246,249,251,263,267,269,270,276,282,
283,284,285,288,291,293,295,296,297,300,308,310,311,312,314,317,318,321,
323,324,332,334,336,338,356,361,362,365,373,374,375,377,378,379,380,381,
382,384,385,386,387,388,389,390,397,398,400,402,405,409,415,454,466,468,
469,470,471,472,473,474,475,476,477
```

Blendshape model output is `[52]` (`StatefulPartitionedCall:0`), labels in
graph order begin `_neutral` at 0,
`eyeBlinkLeft` at 9, `eyeBlinkRight` at 10, `eyeWideLeft` at 21, and
`eyeWideRight` at 22. Pinned graph sets `top_k=0` and
`min_score_threshold=-1.0`, returning coefficients as-is; eye coefficients
are not calibrated eyelid-open probabilities
([pinned graph](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/face_blendshapes_graph.cc#L51-L115),
[raw conversion](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/face_blendshapes_graph.cc#L161-L176)).

## Threshold semantics

Landmark presence takes raw second-output scalar, applies
`1/(1+exp(-x))`, then `ThresholdingCalculator`. The threshold is
`min_detection_confidence`, whose pinned default is `0.5`; acceptance is
strict `score > threshold`, so equality rejects and suppresses landmark output
([sigmoid](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/tensors_to_floats_calculator.cc#L23-L26),
[strict threshold](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/util/thresholding_calculator.cc#L115-L125),
[default](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/proto/face_landmarks_detector_graph_options.proto#L36-L45)).

Detector score path applies sigmoid to each raw class logit, then rejects only
when `score < min_detection_confidence`; equality is retained
([pinned score path](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/tensors_to_detections_calculator.cc#L428-L459),
[inclusive filter](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/calculators/tensor/tensors_to_detections_calculator.cc#L926-L942)). Detector default confidence and NMS threshold are both `0.5`
([options](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_detector/proto/face_detector_graph_options.proto#L35-L44)).

## Still-image mode

`use_stream_mode=false` is the still-image contract: Face Landmarker always
runs face detection for each input and does not use previous-frame tracking,
association, or smoothing. Tracking/association is enabled only in stream mode
([pinned graph](https://github.com/google-ai-edge/mediapipe/blob/458200ffced12a9db596ddf1cce3380ea05f71fb/mediapipe/tasks/cc/vision/face_landmarker/face_landmarker_graph.cc#L448-L513)).
Results are projected back to unrotated, uncropped input-image coordinates.

## Reference-parity gate

Before any quality or accuracy qualification, compare a new float-bilinear
implementation against pinned CPU OpenCV `INTER_LINEAR` on fixed RGB edge,
ROI, rotation, and out-of-bounds cases. Run detector `(128, keep=true,
zero-border)` and landmark `(256, keep=false, replicate-border)` separately;
compare preprocessing tensors, letterbox padding, and crop-to-image mapping.
OpenCV's discretized bilinear kernel and a new float-bilinear kernel can differ
at sampling boundaries, so differences must be recorded and resolved before
parity is claimed. This document defines no tolerance or accuracy result.

## Pinned source receipt

All source files below are raw files at MediaPipe revision
`458200ffced12a9db596ddf1cce3380ea05f71fb`; hashes are SHA-256 of downloaded
bytes. The artifact/child hashes are in the inventory linked above.

| source file | bytes | SHA-256 |
| --- | ---: | --- |
| `calculators/tensor/image_to_tensor_calculator.proto` | 2,897 | `fbd2463f226f2f8638efcd6f6867d85bb959ba491e44b4e7824b817a09decb5f` |
| `calculators/tensor/image_to_tensor_utils.cc` | 11,071 | `ce3bc5b8669181c987e01d6aebae131995a2c62946453bbff4fd0b3553033e9f` |
| `calculators/tensor/image_to_tensor_converter_opencv.cc` | 8,454 | `b87345136f2e892ec2c75eb7a0c99ff830017df0f02d278161e0b9dbac8ab9be` |
| `calculators/tensor/tensors_to_detections_calculator.cc` | 58,933 | `25c5c02abaaad32b7f8edbc6fa721ab591ec3742226da507f656839bca6f5dfa` |
| `calculators/tensor/tensors_to_floats_calculator.cc` | 3,870 | `6796476e7a450c456ff2eb1558c42494fd367682f81ed2530c985e761013c8fc` |
| `calculators/util/thresholding_calculator.cc` | 4,867 | `af988e9ded209178a93ac7f074b8119845fcbd369d2156ac71ac1a8e8c771a68` |
| `calculators/tflite/ssd_anchors_calculator.cc` | 14,039 | `27000d2e492054aed3ae4fb7efa2afcfeeb5238784e58fa08b3c81415ede5175` |
| `calculators/util/non_max_suppression_calculator.cc` | 17,840 | `f14e17d8f7a9b5ef883bae83ce8da35942618db57b3294dd358ea947fd89cece` |
| `calculators/util/detections_to_rects_calculator.cc` | 12,877 | `f216dbb97daf504b1c567c7c0dba6f8c39b060ead965f01d943f257d71a5210e` |
| `calculators/util/rect_transformation_calculator.cc` | 9,368 | `dd7794d32a62fff171a2a105214d723ca82ef04ab1ab76febe07ae7450f3f966` |
| `calculators/tensor/landmarks_to_tensor_calculator.cc` | 6,156 | `4eb2dba1dbde11a7fc1dc5a3b2ad793eabff5fe291a11a7b0f631d295999da99` |
| `calculators/util/landmark_projection_calculator.cc` | 6,891 | `e26ecd3c31240d4ef327435e97333b35581d74bb8a0493796b420b014f3874d2` |
| `tasks/cc/vision/face_detector/face_detector_graph.cc` | 20,745 | `6eda509b59ae50fd645505dfeb20f4c7cc9fc9204f1cac966c0366a27d10863d` |
| `tasks/cc/vision/face_detector/proto/face_detector_graph_options.proto` | 1,794 | `f08cd4ff74b6531d89b46c15e6802ed0b5e2fc45fb87fa5119e4824316f78d28` |
| `tasks/cc/vision/face_landmarker/face_landmarker_graph.cc` | 24,915 | `d753bdf62c23e122deb134f723aa759385dcae274954659a7da3c1de56381272` |
| `tasks/cc/vision/face_landmarker/face_landmarks_detector_graph.cc` | 29,010 | `2f133f326b1a638d5ad577222e4197b3350b27b117f95e83414543878f10b872` |
| `tasks/cc/vision/face_landmarker/face_blendshapes_graph.cc` | 11,562 | `a3826ad2738315f1e046b36fd883825980e151db677e53d21ff35eaff64e4bd3` |
| `tasks/cc/vision/face_landmarker/tensors_to_face_landmarks_graph.cc` | 10,901 | `c1870ad079e3f247c3aebdfb2a833d809b05e36d80fd879a60613d9536cce910` |
| `tasks/cc/components/processors/image_preprocessing_graph.cc` | 12,172 | `32ab446690da40133fc901a0722e54fb76a496e8232758d63f3d99411f3049ad` |
| `tasks/metadata/metadata_schema.fbs` | 28,030 | `54f0cc99a87d56045bc32ad177bf5d02a29836416493cab3e98a06057b60797b` |

The source snapshot contains a 2025 copyright header on
`face_detector_graph.cc`, while the exact model object has HTTP
`Last-Modified: 2023-05-03` and metadata `min_runtime_version=1.14.0`.
Those dates are provenance signals only; they do not prove that this later
graph source generated or exactly matches model-v1 bytes. Static model shapes,
names, and metadata above come from the pinned task itself.

The reproducible metadata receipt is [`mediapipe-v1-image-metadata.json`](mediapipe-v1-image-metadata.json), generated by [`inspect-mediapipe-image-metadata.py`](../../tools/inspect-mediapipe-image-metadata.py). Run it with exact task, pinned TFLite `schema.fbs` (SHA-256 `e4739320658d85923286a2dedf1dd0c77387170c470ad7da2f3c22082edd9c4f`), and pinned MediaPipe `metadata_schema.fbs` (SHA-256 `54f0cc99a87d56045bc32ad177bf5d02a29836416493cab3e98a06057b60797b`); inspector verifies whole-artifact/schema hashes before parsing, reads only graph descriptors plus `TFLITE_METADATA`, and never interprets model-weight buffers.
