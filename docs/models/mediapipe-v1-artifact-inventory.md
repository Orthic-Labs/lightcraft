# MediaPipe Face Landmarker v1 static artifact inventory

Generated from exact artifact bytes by `tools/inspect-mediapipe-v1.py`. This is static metadata only: no inference, model import, conversion, or floating-point tensor values. Only bounded INT32 StridedSlice control payloads are read for stride validation.

## Pinned provenance

- Artifact: `3758596` bytes, SHA-256 `64184e229b263107bc2b804c6625db1341ff2bb731874b0bcc2fe6544e0bc9ff`.
- TFLite schema: [`schema.fbs` at revision e3388c0b93738d8d1a59211ffe2d0643da907291](https://github.com/tensorflow/tflite-micro/blob/e3388c0b93738d8d1a59211ffe2d0643da907291/tensorflow/lite/schema/schema.fbs), SHA-256 `e4739320658d85923286a2dedf1dd0c77387170c470ad7da2f3c22082edd9c4f`.
- The local schema hash is checked before generation; all opcode names, tensor type names, BuiltinOptions union tags, option field names, enum names, and schema defaults are parsed from that source.
- The model URL is the versioned MediaPipe Face Landmarker float16/1 task. Model-card/license evidence remains in `mediapipe-face-landmarker-v1-qualification.json`.

## ZIP entries

| name | absolute data offset | bytes | SHA-256 | CRC32 |
| --- | ---: | ---: | --- | --- |
| `face_detector.tflite` | 52 | 229746 | `b4578f35940bf5a1a655214a1cce5cab13eba73c1297cd78e1a04c2380b0152f` | `6763ac6d` |
| `face_landmarks_detector.tflite` | 229860 | 2553590 | `c7d54204ce0448474c7f3fa9af494787c0965cbdd6f20fc72867e43046bd43d5` | `8079302f` |
| `geometry_pipeline_metadata_landmarks.binarypb` | 2783528 | 19376 | `bdbcda96dfcb7da883da124aaa2c55dee49770d934f0fcc71747f8c21bdc75b4` | `f38ea9a5` |
| `face_blendshapes.tflite` | 2802960 | 955312 | `4f36dded049db18d76048567439b2a7f58f1daabc00d78bfe8f3ad396a2d2082` | `00d43c64` |

## Graphs

Operators are listed in FlatBuffer topological order. Tensor offsets are absolute offsets in full `.task` bytes; `None/0` means runtime tensor without payload.

### Detector — `face_detector.tflite`

- Schema version: `3`; subgraph: `keras2tflite_facedetector-front.tflite.generated`.
- Tensors: `250`; nodes: `164`.
- Inputs: `[0]`; outputs: `[175, 174]`.
- Dtypes: `{"FLOAT16": 74, "FLOAT32": 165, "INT32": 11}`.
- Corrected operator inventory: `{"ADD": 16, "CONCATENATION": 2, "CONV_2D": 21, "DEPTHWISE_CONV_2D": 16, "DEQUANTIZE": 74, "MAX_POOL_2D": 3, "PAD": 11, "RELU": 17, "RESHAPE": 4}`.

#### Operator codes

| opcode table index | schema builtin code | name | version | custom code |
| ---: | ---: | --- | ---: | --- |
| 0 | 3 | `CONV_2D` | 1 | `` |
| 1 | 19 | `RELU` | 1 | `` |
| 2 | 4 | `DEPTHWISE_CONV_2D` | 1 | `` |
| 3 | 0 | `ADD` | 1 | `` |
| 4 | 34 | `PAD` | 1 | `` |
| 5 | 17 | `MAX_POOL_2D` | 1 | `` |
| 6 | 22 | `RESHAPE` | 1 | `` |
| 7 | 2 | `CONCATENATION` | 1 | `` |
| 8 | 6 | `DEQUANTIZE` | 2 | `` |

#### Tensors

| index | name | dtype | shape | buffer index | absolute offset | bytes |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 0 | `input` | `FLOAT32` | `[1, 128, 128, 3]` | 0 | None | 0 |
| 1 | `conv2d/Kernel` | `FLOAT16` | `[24, 5, 5, 3]` | 1 | 202600 | 3600 |
| 2 | `conv2d/Bias` | `FLOAT16` | `[24]` | 2 | 202540 | 48 |
| 3 | `conv2d` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 4 | `activation` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 5 | `depthwise_conv2d/Kernel` | `FLOAT16` | `[1, 3, 3, 24]` | 3 | 202096 | 432 |
| 6 | `depthwise_conv2d/Bias` | `FLOAT16` | `[24]` | 4 | 202036 | 48 |
| 7 | `depthwise_conv2d` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 8 | `conv2d_1/Kernel` | `FLOAT16` | `[24, 1, 1, 24]` | 5 | 200872 | 1152 |
| 9 | `conv2d_1/Bias` | `FLOAT16` | `[24]` | 6 | 200812 | 48 |
| 10 | `conv2d_1` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 11 | `add__xeno_compat__1` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 12 | `activation_1` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 13 | `depthwise_conv2d_1/Kernel` | `FLOAT16` | `[1, 3, 3, 24]` | 7 | 200368 | 432 |
| 14 | `depthwise_conv2d_1/Bias` | `FLOAT16` | `[24]` | 8 | 200308 | 48 |
| 15 | `depthwise_conv2d_1` | `FLOAT32` | `[1, 64, 64, 24]` | 0 | None | 0 |
| 16 | `conv2d_2/Kernel` | `FLOAT16` | `[28, 1, 1, 24]` | 9 | 198952 | 1344 |
| 17 | `conv2d_2/Bias` | `FLOAT16` | `[28]` | 10 | 198884 | 56 |
| 18 | `conv2d_2` | `FLOAT32` | `[1, 64, 64, 28]` | 0 | None | 0 |
| 19 | `channel_padding/Paddings` | `INT32` | `[4, 2]` | 11 | 198840 | 32 |
| 20 | `channel_padding` | `FLOAT32` | `[1, 64, 64, 28]` | 0 | None | 0 |
| 21 | `add_1__xeno_compat__1` | `FLOAT32` | `[1, 64, 64, 28]` | 0 | None | 0 |
| 22 | `activation_2` | `FLOAT32` | `[1, 64, 64, 28]` | 0 | None | 0 |
| 23 | `depthwise_conv2d_2/Kernel` | `FLOAT16` | `[1, 3, 3, 28]` | 12 | 198324 | 504 |
| 24 | `depthwise_conv2d_2/Bias` | `FLOAT16` | `[28]` | 13 | 198256 | 56 |
| 25 | `depthwise_conv2d_2` | `FLOAT32` | `[1, 32, 32, 28]` | 0 | None | 0 |
| 26 | `max_pooling2d` | `FLOAT32` | `[1, 32, 32, 28]` | 0 | None | 0 |
| 27 | `conv2d_3/Kernel` | `FLOAT16` | `[32, 1, 1, 28]` | 14 | 196452 | 1792 |
| 28 | `conv2d_3/Bias` | `FLOAT16` | `[32]` | 15 | 196376 | 64 |
| 29 | `conv2d_3` | `FLOAT32` | `[1, 32, 32, 32]` | 0 | None | 0 |
| 30 | `channel_padding_1/Paddings` | `INT32` | `[4, 2]` | 16 | 196332 | 32 |
| 31 | `channel_padding_1` | `FLOAT32` | `[1, 32, 32, 32]` | 0 | None | 0 |
| 32 | `add_2__xeno_compat__1` | `FLOAT32` | `[1, 32, 32, 32]` | 0 | None | 0 |
| 33 | `activation_3` | `FLOAT32` | `[1, 32, 32, 32]` | 0 | None | 0 |
| 34 | `depthwise_conv2d_3/Kernel` | `FLOAT16` | `[1, 3, 3, 32]` | 17 | 195744 | 576 |
| 35 | `depthwise_conv2d_3/Bias` | `FLOAT16` | `[32]` | 18 | 195668 | 64 |
| 36 | `depthwise_conv2d_3` | `FLOAT32` | `[1, 32, 32, 32]` | 0 | None | 0 |
| 37 | `conv2d_4/Kernel` | `FLOAT16` | `[36, 1, 1, 32]` | 19 | 193352 | 2304 |
| 38 | `conv2d_4/Bias` | `FLOAT16` | `[36]` | 20 | 193268 | 72 |
| 39 | `conv2d_4` | `FLOAT32` | `[1, 32, 32, 36]` | 0 | None | 0 |
| 40 | `channel_padding_2/Paddings` | `INT32` | `[4, 2]` | 21 | 193224 | 32 |
| 41 | `channel_padding_2` | `FLOAT32` | `[1, 32, 32, 36]` | 0 | None | 0 |
| 42 | `add_3__xeno_compat__1` | `FLOAT32` | `[1, 32, 32, 36]` | 0 | None | 0 |
| 43 | `activation_4` | `FLOAT32` | `[1, 32, 32, 36]` | 0 | None | 0 |
| 44 | `depthwise_conv2d_4/Kernel` | `FLOAT16` | `[1, 3, 3, 36]` | 22 | 192564 | 648 |
| 45 | `depthwise_conv2d_4/Bias` | `FLOAT16` | `[36]` | 23 | 192480 | 72 |
| 46 | `depthwise_conv2d_4` | `FLOAT32` | `[1, 32, 32, 36]` | 0 | None | 0 |
| 47 | `conv2d_5/Kernel` | `FLOAT16` | `[42, 1, 1, 36]` | 24 | 189444 | 3024 |
| 48 | `conv2d_5/Bias` | `FLOAT16` | `[42]` | 25 | 189348 | 84 |
| 49 | `conv2d_5` | `FLOAT32` | `[1, 32, 32, 42]` | 0 | None | 0 |
| 50 | `channel_padding_3/Paddings` | `INT32` | `[4, 2]` | 26 | 189304 | 32 |
| 51 | `channel_padding_3` | `FLOAT32` | `[1, 32, 32, 42]` | 0 | None | 0 |
| 52 | `add_4__xeno_compat__1` | `FLOAT32` | `[1, 32, 32, 42]` | 0 | None | 0 |
| 53 | `activation_5` | `FLOAT32` | `[1, 32, 32, 42]` | 0 | None | 0 |
| 54 | `depthwise_conv2d_5/Kernel` | `FLOAT16` | `[1, 3, 3, 42]` | 27 | 188536 | 756 |
| 55 | `depthwise_conv2d_5/Bias` | `FLOAT16` | `[42]` | 28 | 188440 | 84 |
| 56 | `depthwise_conv2d_5` | `FLOAT32` | `[1, 16, 16, 42]` | 0 | None | 0 |
| 57 | `max_pooling2d_1` | `FLOAT32` | `[1, 16, 16, 42]` | 0 | None | 0 |
| 58 | `conv2d_6/Kernel` | `FLOAT16` | `[48, 1, 1, 42]` | 29 | 184396 | 4032 |
| 59 | `conv2d_6/Bias` | `FLOAT16` | `[48]` | 30 | 184288 | 96 |
| 60 | `conv2d_6` | `FLOAT32` | `[1, 16, 16, 48]` | 0 | None | 0 |
| 61 | `channel_padding_4/Paddings` | `INT32` | `[4, 2]` | 31 | 184244 | 32 |
| 62 | `channel_padding_4` | `FLOAT32` | `[1, 16, 16, 48]` | 0 | None | 0 |
| 63 | `add_5__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 48]` | 0 | None | 0 |
| 64 | `activation_6` | `FLOAT32` | `[1, 16, 16, 48]` | 0 | None | 0 |
| 65 | `depthwise_conv2d_6/Kernel` | `FLOAT16` | `[1, 3, 3, 48]` | 32 | 183368 | 864 |
| 66 | `depthwise_conv2d_6/Bias` | `FLOAT16` | `[48]` | 33 | 183260 | 96 |
| 67 | `depthwise_conv2d_6` | `FLOAT32` | `[1, 16, 16, 48]` | 0 | None | 0 |
| 68 | `conv2d_7/Kernel` | `FLOAT16` | `[56, 1, 1, 48]` | 34 | 177872 | 5376 |
| 69 | `conv2d_7/Bias` | `FLOAT16` | `[56]` | 35 | 177748 | 112 |
| 70 | `conv2d_7` | `FLOAT32` | `[1, 16, 16, 56]` | 0 | None | 0 |
| 71 | `channel_padding_5/Paddings` | `INT32` | `[4, 2]` | 36 | 177704 | 32 |
| 72 | `channel_padding_5` | `FLOAT32` | `[1, 16, 16, 56]` | 0 | None | 0 |
| 73 | `add_6__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 56]` | 0 | None | 0 |
| 74 | `activation_7` | `FLOAT32` | `[1, 16, 16, 56]` | 0 | None | 0 |
| 75 | `depthwise_conv2d_7/Kernel` | `FLOAT16` | `[1, 3, 3, 56]` | 37 | 176684 | 1008 |
| 76 | `depthwise_conv2d_7/Bias` | `FLOAT16` | `[56]` | 38 | 176560 | 112 |
| 77 | `depthwise_conv2d_7` | `FLOAT32` | `[1, 16, 16, 56]` | 0 | None | 0 |
| 78 | `conv2d_8/Kernel` | `FLOAT16` | `[64, 1, 1, 56]` | 39 | 169380 | 7168 |
| 79 | `conv2d_8/Bias` | `FLOAT16` | `[64]` | 40 | 169240 | 128 |
| 80 | `conv2d_8` | `FLOAT32` | `[1, 16, 16, 64]` | 0 | None | 0 |
| 81 | `channel_padding_6/Paddings` | `INT32` | `[4, 2]` | 41 | 169196 | 32 |
| 82 | `channel_padding_6` | `FLOAT32` | `[1, 16, 16, 64]` | 0 | None | 0 |
| 83 | `add_7__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 64]` | 0 | None | 0 |
| 84 | `activation_8` | `FLOAT32` | `[1, 16, 16, 64]` | 0 | None | 0 |
| 85 | `depthwise_conv2d_8/Kernel` | `FLOAT16` | `[1, 3, 3, 64]` | 42 | 168032 | 1152 |
| 86 | `depthwise_conv2d_8/Bias` | `FLOAT16` | `[64]` | 43 | 167892 | 128 |
| 87 | `depthwise_conv2d_8` | `FLOAT32` | `[1, 16, 16, 64]` | 0 | None | 0 |
| 88 | `conv2d_9/Kernel` | `FLOAT16` | `[72, 1, 1, 64]` | 44 | 158664 | 9216 |
| 89 | `conv2d_9/Bias` | `FLOAT16` | `[72]` | 45 | 158508 | 144 |
| 90 | `conv2d_9` | `FLOAT32` | `[1, 16, 16, 72]` | 0 | None | 0 |
| 91 | `channel_padding_7/Paddings` | `INT32` | `[4, 2]` | 46 | 158464 | 32 |
| 92 | `channel_padding_7` | `FLOAT32` | `[1, 16, 16, 72]` | 0 | None | 0 |
| 93 | `add_8__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 72]` | 0 | None | 0 |
| 94 | `activation_9` | `FLOAT32` | `[1, 16, 16, 72]` | 0 | None | 0 |
| 95 | `depthwise_conv2d_9/Kernel` | `FLOAT16` | `[1, 3, 3, 72]` | 47 | 157156 | 1296 |
| 96 | `depthwise_conv2d_9/Bias` | `FLOAT16` | `[72]` | 48 | 157000 | 144 |
| 97 | `depthwise_conv2d_9` | `FLOAT32` | `[1, 16, 16, 72]` | 0 | None | 0 |
| 98 | `conv2d_10/Kernel` | `FLOAT16` | `[80, 1, 1, 72]` | 49 | 145468 | 11520 |
| 99 | `conv2d_10/Bias` | `FLOAT16` | `[80]` | 50 | 145296 | 160 |
| 100 | `conv2d_10` | `FLOAT32` | `[1, 16, 16, 80]` | 0 | None | 0 |
| 101 | `channel_padding_8/Paddings` | `INT32` | `[4, 2]` | 51 | 145252 | 32 |
| 102 | `channel_padding_8` | `FLOAT32` | `[1, 16, 16, 80]` | 0 | None | 0 |
| 103 | `add_9__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 80]` | 0 | None | 0 |
| 104 | `activation_10` | `FLOAT32` | `[1, 16, 16, 80]` | 0 | None | 0 |
| 105 | `depthwise_conv2d_10/Kernel` | `FLOAT16` | `[1, 3, 3, 80]` | 52 | 143800 | 1440 |
| 106 | `depthwise_conv2d_10/Bias` | `FLOAT16` | `[80]` | 53 | 143628 | 160 |
| 107 | `depthwise_conv2d_10` | `FLOAT32` | `[1, 16, 16, 80]` | 0 | None | 0 |
| 108 | `conv2d_11/Kernel` | `FLOAT16` | `[88, 1, 1, 80]` | 54 | 129536 | 14080 |
| 109 | `conv2d_11/Bias` | `FLOAT16` | `[88]` | 55 | 129348 | 176 |
| 110 | `conv2d_11` | `FLOAT32` | `[1, 16, 16, 88]` | 0 | None | 0 |
| 111 | `channel_padding_9/Paddings` | `INT32` | `[4, 2]` | 56 | 129304 | 32 |
| 112 | `channel_padding_9` | `FLOAT32` | `[1, 16, 16, 88]` | 0 | None | 0 |
| 113 | `add_10__xeno_compat__1` | `FLOAT32` | `[1, 16, 16, 88]` | 0 | None | 0 |
| 114 | `activation_11` | `FLOAT32` | `[1, 16, 16, 88]` | 0 | None | 0 |
| 115 | `depthwise_conv2d_11/Kernel` | `FLOAT16` | `[1, 3, 3, 88]` | 57 | 127708 | 1584 |
| 116 | `depthwise_conv2d_11/Bias` | `FLOAT16` | `[88]` | 58 | 127520 | 176 |
| 117 | `depthwise_conv2d_11` | `FLOAT32` | `[1, 8, 8, 88]` | 0 | None | 0 |
| 118 | `max_pooling2d_2` | `FLOAT32` | `[1, 8, 8, 88]` | 0 | None | 0 |
| 119 | `conv2d_12/Kernel` | `FLOAT16` | `[96, 1, 1, 88]` | 59 | 110612 | 16896 |
| 120 | `conv2d_12/Bias` | `FLOAT16` | `[96]` | 60 | 110408 | 192 |
| 121 | `conv2d_12` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 122 | `channel_padding_10/Paddings` | `INT32` | `[4, 2]` | 61 | 110364 | 32 |
| 123 | `channel_padding_10` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 124 | `add_11__xeno_compat__1` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 125 | `activation_12` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 126 | `depthwise_conv2d_12/Kernel` | `FLOAT16` | `[1, 3, 3, 96]` | 62 | 108624 | 1728 |
| 127 | `depthwise_conv2d_12/Bias` | `FLOAT16` | `[96]` | 63 | 108420 | 192 |
| 128 | `depthwise_conv2d_12` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 129 | `conv2d_13/Kernel` | `FLOAT16` | `[96, 1, 1, 96]` | 64 | 89976 | 18432 |
| 130 | `conv2d_13/Bias` | `FLOAT16` | `[96]` | 65 | 89772 | 192 |
| 131 | `conv2d_13` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 132 | `add_12__xeno_compat__1` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 133 | `activation_13` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 134 | `depthwise_conv2d_13/Kernel` | `FLOAT16` | `[1, 3, 3, 96]` | 66 | 88032 | 1728 |
| 135 | `depthwise_conv2d_13/Bias` | `FLOAT16` | `[96]` | 67 | 87828 | 192 |
| 136 | `depthwise_conv2d_13` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 137 | `conv2d_14/Kernel` | `FLOAT16` | `[96, 1, 1, 96]` | 68 | 69384 | 18432 |
| 138 | `conv2d_14/Bias` | `FLOAT16` | `[96]` | 69 | 69180 | 192 |
| 139 | `conv2d_14` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 140 | `add_13__xeno_compat__1` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 141 | `activation_14` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 142 | `depthwise_conv2d_14/Kernel` | `FLOAT16` | `[1, 3, 3, 96]` | 70 | 67440 | 1728 |
| 143 | `depthwise_conv2d_14/Bias` | `FLOAT16` | `[96]` | 71 | 67236 | 192 |
| 144 | `depthwise_conv2d_14` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 145 | `conv2d_15/Kernel` | `FLOAT16` | `[96, 1, 1, 96]` | 72 | 48792 | 18432 |
| 146 | `conv2d_15/Bias` | `FLOAT16` | `[96]` | 73 | 48588 | 192 |
| 147 | `conv2d_15` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 148 | `add_14__xeno_compat__1` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 149 | `activation_15` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 150 | `depthwise_conv2d_15/Kernel` | `FLOAT16` | `[1, 3, 3, 96]` | 74 | 46848 | 1728 |
| 151 | `depthwise_conv2d_15/Bias` | `FLOAT16` | `[96]` | 75 | 46644 | 192 |
| 152 | `depthwise_conv2d_15` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 153 | `conv2d_16/Kernel` | `FLOAT16` | `[96, 1, 1, 96]` | 76 | 28200 | 18432 |
| 154 | `conv2d_16/Bias` | `FLOAT16` | `[96]` | 77 | 27996 | 192 |
| 155 | `conv2d_16` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 156 | `add_15__xeno_compat__1` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 157 | `activation_16` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 158 | `classificator_8/Kernel` | `FLOAT16` | `[2, 1, 1, 88]` | 78 | 27632 | 352 |
| 159 | `classificator_8/Bias` | `FLOAT16` | `[2]` | 79 | 27616 | 4 |
| 160 | `classificator_8` | `FLOAT32` | `[1, 16, 16, 2]` | 0 | None | 0 |
| 161 | `classificator_16/Kernel` | `FLOAT16` | `[6, 1, 1, 96]` | 80 | 26452 | 1152 |
| 162 | `classificator_16/Bias` | `FLOAT16` | `[6]` | 81 | 26428 | 12 |
| 163 | `classificator_16` | `FLOAT32` | `[1, 8, 8, 6]` | 0 | None | 0 |
| 164 | `regressor_8/Kernel` | `FLOAT16` | `[32, 1, 1, 88]` | 82 | 20784 | 5632 |
| 165 | `regressor_8/Bias` | `FLOAT16` | `[32]` | 83 | 20708 | 64 |
| 166 | `regressor_8` | `FLOAT32` | `[1, 16, 16, 32]` | 0 | None | 0 |
| 167 | `regressor_16/Kernel` | `FLOAT16` | `[96, 1, 1, 96]` | 84 | 2264 | 18432 |
| 168 | `regressor_16/Bias` | `FLOAT16` | `[96]` | 85 | 2060 | 192 |
| 169 | `regressor_16` | `FLOAT32` | `[1, 8, 8, 96]` | 0 | None | 0 |
| 170 | `reshape` | `FLOAT32` | `[1, 512, 1]` | 0 | None | 0 |
| 171 | `reshape_2` | `FLOAT32` | `[1, 384, 1]` | 0 | None | 0 |
| 172 | `reshape_1` | `FLOAT32` | `[1, 512, 16]` | 0 | None | 0 |
| 173 | `reshape_3` | `FLOAT32` | `[1, 384, 16]` | 0 | None | 0 |
| 174 | `classificators` | `FLOAT32` | `[1, 896, 1]` | 0 | None | 0 |
| 175 | `regressors` | `FLOAT32` | `[1, 896, 16]` | 0 | None | 0 |
| 176 | `conv2d_3/Bias_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 177 | `conv2d_16/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 178 | `conv2d_2/Bias_dequantize` | `FLOAT32` | `[28]` | 0 | None | 0 |
| 179 | `depthwise_conv2d_3/Bias_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 180 | `depthwise_conv2d_14/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 181 | `classificator_16/Kernel_dequantize` | `FLOAT32` | `[6, 1, 1, 96]` | 0 | None | 0 |
| 182 | `conv2d_9/Bias_dequantize` | `FLOAT32` | `[72]` | 0 | None | 0 |
| 183 | `regressor_16/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 184 | `depthwise_conv2d_2/Bias_dequantize` | `FLOAT32` | `[28]` | 0 | None | 0 |
| 185 | `depthwise_conv2d/Bias_dequantize` | `FLOAT32` | `[24]` | 0 | None | 0 |
| 186 | `depthwise_conv2d_15/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 96]` | 0 | None | 0 |
| 187 | `conv2d_8/Kernel_dequantize` | `FLOAT32` | `[64, 1, 1, 56]` | 0 | None | 0 |
| 188 | `depthwise_conv2d_9/Bias_dequantize` | `FLOAT32` | `[72]` | 0 | None | 0 |
| 189 | `depthwise_conv2d_1/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 24]` | 0 | None | 0 |
| 190 | `regressor_8/Kernel_dequantize` | `FLOAT32` | `[32, 1, 1, 88]` | 0 | None | 0 |
| 191 | `conv2d_15/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 192 | `depthwise_conv2d_8/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 193 | `conv2d/Bias_dequantize` | `FLOAT32` | `[24]` | 0 | None | 0 |
| 194 | `conv2d_16/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 96]` | 0 | None | 0 |
| 195 | `conv2d_10/Bias_dequantize` | `FLOAT32` | `[80]` | 0 | None | 0 |
| 196 | `depthwise_conv2d_13/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 197 | `conv2d_4/Bias_dequantize` | `FLOAT32` | `[36]` | 0 | None | 0 |
| 198 | `depthwise_conv2d_14/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 96]` | 0 | None | 0 |
| 199 | `conv2d_9/Kernel_dequantize` | `FLOAT32` | `[72, 1, 1, 64]` | 0 | None | 0 |
| 200 | `depthwise_conv2d_10/Bias_dequantize` | `FLOAT32` | `[80]` | 0 | None | 0 |
| 201 | `conv2d_3/Kernel_dequantize` | `FLOAT32` | `[32, 1, 1, 28]` | 0 | None | 0 |
| 202 | `depthwise_conv2d_4/Bias_dequantize` | `FLOAT32` | `[36]` | 0 | None | 0 |
| 203 | `conv2d_1/Bias_dequantize` | `FLOAT32` | `[24]` | 0 | None | 0 |
| 204 | `conv2d_6/Bias_dequantize` | `FLOAT32` | `[48]` | 0 | None | 0 |
| 205 | `depthwise_conv2d_9/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 72]` | 0 | None | 0 |
| 206 | `depthwise_conv2d_3/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 207 | `conv2d_2/Kernel_dequantize` | `FLOAT32` | `[28, 1, 1, 24]` | 0 | None | 0 |
| 208 | `regressor_16/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 96]` | 0 | None | 0 |
| 209 | `conv2d_12/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 210 | `conv2d_5/Bias_dequantize` | `FLOAT32` | `[42]` | 0 | None | 0 |
| 211 | `depthwise_conv2d_6/Bias_dequantize` | `FLOAT32` | `[48]` | 0 | None | 0 |
| 212 | `depthwise_conv2d_2/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 28]` | 0 | None | 0 |
| 213 | `conv2d_14/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 214 | `depthwise_conv2d/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 24]` | 0 | None | 0 |
| 215 | `conv2d_4/Kernel_dequantize` | `FLOAT32` | `[36, 1, 1, 32]` | 0 | None | 0 |
| 216 | `depthwise_conv2d_5/Bias_dequantize` | `FLOAT32` | `[42]` | 0 | None | 0 |
| 217 | `conv2d_11/Bias_dequantize` | `FLOAT32` | `[88]` | 0 | None | 0 |
| 218 | `depthwise_conv2d_12/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 219 | `depthwise_conv2d_4/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 36]` | 0 | None | 0 |
| 220 | `conv2d_15/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 96]` | 0 | None | 0 |
| 221 | `conv2d_1/Kernel_dequantize` | `FLOAT32` | `[24, 1, 1, 24]` | 0 | None | 0 |
| 222 | `classificator_8/Bias_dequantize` | `FLOAT32` | `[2]` | 0 | None | 0 |
| 223 | `depthwise_conv2d_13/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 96]` | 0 | None | 0 |
| 224 | `conv2d/Kernel_dequantize` | `FLOAT32` | `[24, 5, 5, 3]` | 0 | None | 0 |
| 225 | `conv2d_10/Kernel_dequantize` | `FLOAT32` | `[80, 1, 1, 72]` | 0 | None | 0 |
| 226 | `depthwise_conv2d_11/Bias_dequantize` | `FLOAT32` | `[88]` | 0 | None | 0 |
| 227 | `conv2d_7/Bias_dequantize` | `FLOAT32` | `[56]` | 0 | None | 0 |
| 228 | `depthwise_conv2d_10/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 80]` | 0 | None | 0 |
| 229 | `conv2d_12/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 88]` | 0 | None | 0 |
| 230 | `conv2d_14/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 96]` | 0 | None | 0 |
| 231 | `conv2d_13/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 232 | `conv2d_6/Kernel_dequantize` | `FLOAT32` | `[48, 1, 1, 42]` | 0 | None | 0 |
| 233 | `depthwise_conv2d_7/Bias_dequantize` | `FLOAT32` | `[56]` | 0 | None | 0 |
| 234 | `classificator_16/Bias_dequantize` | `FLOAT32` | `[6]` | 0 | None | 0 |
| 235 | `conv2d_11/Kernel_dequantize` | `FLOAT32` | `[88, 1, 1, 80]` | 0 | None | 0 |
| 236 | `depthwise_conv2d_12/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 96]` | 0 | None | 0 |
| 237 | `conv2d_5/Kernel_dequantize` | `FLOAT32` | `[42, 1, 1, 36]` | 0 | None | 0 |
| 238 | `depthwise_conv2d_6/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 48]` | 0 | None | 0 |
| 239 | `depthwise_conv2d_15/Bias_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 240 | `conv2d_8/Bias_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 241 | `depthwise_conv2d_11/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 88]` | 0 | None | 0 |
| 242 | `depthwise_conv2d_5/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 42]` | 0 | None | 0 |
| 243 | `conv2d_7/Kernel_dequantize` | `FLOAT32` | `[56, 1, 1, 48]` | 0 | None | 0 |
| 244 | `depthwise_conv2d_8/Bias_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 245 | `classificator_8/Kernel_dequantize` | `FLOAT32` | `[2, 1, 1, 88]` | 0 | None | 0 |
| 246 | `depthwise_conv2d_7/Kernel_dequantize` | `FLOAT32` | `[1, 3, 3, 56]` | 0 | None | 0 |
| 247 | `depthwise_conv2d_1/Bias_dequantize` | `FLOAT32` | `[24]` | 0 | None | 0 |
| 248 | `regressor_8/Bias_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 249 | `conv2d_13/Kernel_dequantize` | `FLOAT32` | `[96, 1, 1, 96]` | 0 | None | 0 |

#### Nodes

| index | op | inputs | output | exact schema-derived options |
| ---: | --- | --- | ---: | --- |
| 0 | `DEQUANTIZE` | `[2]` | 193 | `{}` |
| 1 | `DEQUANTIZE` | `[1]` | 224 | `{}` |
| 2 | `CONV_2D` | `[0, 224, 193]` | 3 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 3 | `RELU` | `[3]` | 4 | `{}` |
| 4 | `DEQUANTIZE` | `[6]` | 185 | `{}` |
| 5 | `DEQUANTIZE` | `[5]` | 214 | `{}` |
| 6 | `DEPTHWISE_CONV_2D` | `[4, 214, 185]` | 7 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 7 | `DEQUANTIZE` | `[9]` | 203 | `{}` |
| 8 | `DEQUANTIZE` | `[8]` | 221 | `{}` |
| 9 | `CONV_2D` | `[7, 221, 203]` | 10 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 10 | `ADD` | `[4, 10]` | 11 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 11 | `RELU` | `[11]` | 12 | `{}` |
| 12 | `DEQUANTIZE` | `[13]` | 189 | `{}` |
| 13 | `DEQUANTIZE` | `[14]` | 247 | `{}` |
| 14 | `DEPTHWISE_CONV_2D` | `[12, 189, 247]` | 15 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 15 | `DEQUANTIZE` | `[17]` | 178 | `{}` |
| 16 | `DEQUANTIZE` | `[16]` | 207 | `{}` |
| 17 | `CONV_2D` | `[15, 207, 178]` | 18 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 18 | `PAD` | `[12, 19]` | 20 | `{}` |
| 19 | `ADD` | `[20, 18]` | 21 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 20 | `RELU` | `[21]` | 22 | `{}` |
| 21 | `DEQUANTIZE` | `[24]` | 184 | `{}` |
| 22 | `DEQUANTIZE` | `[23]` | 212 | `{}` |
| 23 | `DEPTHWISE_CONV_2D` | `[22, 212, 184]` | 25 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 24 | `MAX_POOL_2D` | `[22]` | 26 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 25 | `DEQUANTIZE` | `[28]` | 176 | `{}` |
| 26 | `DEQUANTIZE` | `[27]` | 201 | `{}` |
| 27 | `CONV_2D` | `[25, 201, 176]` | 29 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 28 | `PAD` | `[26, 30]` | 31 | `{}` |
| 29 | `ADD` | `[31, 29]` | 32 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 30 | `RELU` | `[32]` | 33 | `{}` |
| 31 | `DEQUANTIZE` | `[35]` | 179 | `{}` |
| 32 | `DEQUANTIZE` | `[34]` | 206 | `{}` |
| 33 | `DEPTHWISE_CONV_2D` | `[33, 206, 179]` | 36 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 34 | `DEQUANTIZE` | `[38]` | 197 | `{}` |
| 35 | `DEQUANTIZE` | `[37]` | 215 | `{}` |
| 36 | `CONV_2D` | `[36, 215, 197]` | 39 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 37 | `PAD` | `[33, 40]` | 41 | `{}` |
| 38 | `ADD` | `[41, 39]` | 42 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 39 | `RELU` | `[42]` | 43 | `{}` |
| 40 | `DEQUANTIZE` | `[45]` | 202 | `{}` |
| 41 | `DEQUANTIZE` | `[44]` | 219 | `{}` |
| 42 | `DEPTHWISE_CONV_2D` | `[43, 219, 202]` | 46 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 43 | `DEQUANTIZE` | `[48]` | 210 | `{}` |
| 44 | `DEQUANTIZE` | `[47]` | 237 | `{}` |
| 45 | `CONV_2D` | `[46, 237, 210]` | 49 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 46 | `PAD` | `[43, 50]` | 51 | `{}` |
| 47 | `ADD` | `[51, 49]` | 52 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 48 | `RELU` | `[52]` | 53 | `{}` |
| 49 | `DEQUANTIZE` | `[55]` | 216 | `{}` |
| 50 | `DEQUANTIZE` | `[54]` | 242 | `{}` |
| 51 | `DEPTHWISE_CONV_2D` | `[53, 242, 216]` | 56 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 52 | `MAX_POOL_2D` | `[53]` | 57 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 53 | `DEQUANTIZE` | `[59]` | 204 | `{}` |
| 54 | `DEQUANTIZE` | `[58]` | 232 | `{}` |
| 55 | `CONV_2D` | `[56, 232, 204]` | 60 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 56 | `PAD` | `[57, 61]` | 62 | `{}` |
| 57 | `ADD` | `[62, 60]` | 63 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 58 | `RELU` | `[63]` | 64 | `{}` |
| 59 | `DEQUANTIZE` | `[66]` | 211 | `{}` |
| 60 | `DEQUANTIZE` | `[65]` | 238 | `{}` |
| 61 | `DEPTHWISE_CONV_2D` | `[64, 238, 211]` | 67 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 62 | `DEQUANTIZE` | `[69]` | 227 | `{}` |
| 63 | `DEQUANTIZE` | `[68]` | 243 | `{}` |
| 64 | `CONV_2D` | `[67, 243, 227]` | 70 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 65 | `PAD` | `[64, 71]` | 72 | `{}` |
| 66 | `ADD` | `[72, 70]` | 73 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 67 | `RELU` | `[73]` | 74 | `{}` |
| 68 | `DEQUANTIZE` | `[76]` | 233 | `{}` |
| 69 | `DEQUANTIZE` | `[75]` | 246 | `{}` |
| 70 | `DEPTHWISE_CONV_2D` | `[74, 246, 233]` | 77 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 71 | `DEQUANTIZE` | `[78]` | 187 | `{}` |
| 72 | `DEQUANTIZE` | `[79]` | 240 | `{}` |
| 73 | `CONV_2D` | `[77, 187, 240]` | 80 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 74 | `PAD` | `[74, 81]` | 82 | `{}` |
| 75 | `ADD` | `[82, 80]` | 83 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 76 | `RELU` | `[83]` | 84 | `{}` |
| 77 | `DEQUANTIZE` | `[85]` | 192 | `{}` |
| 78 | `DEQUANTIZE` | `[86]` | 244 | `{}` |
| 79 | `DEPTHWISE_CONV_2D` | `[84, 192, 244]` | 87 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 80 | `DEQUANTIZE` | `[89]` | 182 | `{}` |
| 81 | `DEQUANTIZE` | `[88]` | 199 | `{}` |
| 82 | `CONV_2D` | `[87, 199, 182]` | 90 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 83 | `PAD` | `[84, 91]` | 92 | `{}` |
| 84 | `ADD` | `[92, 90]` | 93 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 85 | `RELU` | `[93]` | 94 | `{}` |
| 86 | `DEQUANTIZE` | `[96]` | 188 | `{}` |
| 87 | `DEQUANTIZE` | `[95]` | 205 | `{}` |
| 88 | `DEPTHWISE_CONV_2D` | `[94, 205, 188]` | 97 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 89 | `DEQUANTIZE` | `[99]` | 195 | `{}` |
| 90 | `DEQUANTIZE` | `[98]` | 225 | `{}` |
| 91 | `CONV_2D` | `[97, 225, 195]` | 100 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 92 | `PAD` | `[94, 101]` | 102 | `{}` |
| 93 | `ADD` | `[102, 100]` | 103 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 94 | `RELU` | `[103]` | 104 | `{}` |
| 95 | `DEQUANTIZE` | `[106]` | 200 | `{}` |
| 96 | `DEQUANTIZE` | `[105]` | 228 | `{}` |
| 97 | `DEPTHWISE_CONV_2D` | `[104, 228, 200]` | 107 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 98 | `DEQUANTIZE` | `[109]` | 217 | `{}` |
| 99 | `DEQUANTIZE` | `[108]` | 235 | `{}` |
| 100 | `CONV_2D` | `[107, 235, 217]` | 110 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 101 | `PAD` | `[104, 111]` | 112 | `{}` |
| 102 | `ADD` | `[112, 110]` | 113 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 103 | `RELU` | `[113]` | 114 | `{}` |
| 104 | `DEQUANTIZE` | `[116]` | 226 | `{}` |
| 105 | `DEQUANTIZE` | `[115]` | 241 | `{}` |
| 106 | `DEPTHWISE_CONV_2D` | `[114, 241, 226]` | 117 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 107 | `MAX_POOL_2D` | `[114]` | 118 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 2, "stride_w": 2}` |
| 108 | `DEQUANTIZE` | `[120]` | 209 | `{}` |
| 109 | `DEQUANTIZE` | `[119]` | 229 | `{}` |
| 110 | `CONV_2D` | `[117, 229, 209]` | 121 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 111 | `PAD` | `[118, 122]` | 123 | `{}` |
| 112 | `ADD` | `[123, 121]` | 124 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 113 | `RELU` | `[124]` | 125 | `{}` |
| 114 | `DEQUANTIZE` | `[127]` | 218 | `{}` |
| 115 | `DEQUANTIZE` | `[126]` | 236 | `{}` |
| 116 | `DEPTHWISE_CONV_2D` | `[125, 236, 218]` | 128 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 117 | `DEQUANTIZE` | `[130]` | 231 | `{}` |
| 118 | `DEQUANTIZE` | `[129]` | 249 | `{}` |
| 119 | `CONV_2D` | `[128, 249, 231]` | 131 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 120 | `ADD` | `[125, 131]` | 132 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 121 | `RELU` | `[132]` | 133 | `{}` |
| 122 | `DEQUANTIZE` | `[135]` | 196 | `{}` |
| 123 | `DEQUANTIZE` | `[134]` | 223 | `{}` |
| 124 | `DEPTHWISE_CONV_2D` | `[133, 223, 196]` | 136 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 125 | `DEQUANTIZE` | `[138]` | 213 | `{}` |
| 126 | `DEQUANTIZE` | `[137]` | 230 | `{}` |
| 127 | `CONV_2D` | `[136, 230, 213]` | 139 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 128 | `ADD` | `[133, 139]` | 140 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 129 | `RELU` | `[140]` | 141 | `{}` |
| 130 | `DEQUANTIZE` | `[143]` | 180 | `{}` |
| 131 | `DEQUANTIZE` | `[142]` | 198 | `{}` |
| 132 | `DEPTHWISE_CONV_2D` | `[141, 198, 180]` | 144 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 133 | `DEQUANTIZE` | `[146]` | 191 | `{}` |
| 134 | `DEQUANTIZE` | `[145]` | 220 | `{}` |
| 135 | `CONV_2D` | `[144, 220, 191]` | 147 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 136 | `ADD` | `[141, 147]` | 148 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 137 | `RELU` | `[148]` | 149 | `{}` |
| 138 | `DEQUANTIZE` | `[150]` | 186 | `{}` |
| 139 | `DEQUANTIZE` | `[151]` | 239 | `{}` |
| 140 | `DEPTHWISE_CONV_2D` | `[149, 186, 239]` | 152 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 141 | `DEQUANTIZE` | `[154]` | 177 | `{}` |
| 142 | `DEQUANTIZE` | `[153]` | 194 | `{}` |
| 143 | `CONV_2D` | `[152, 194, 177]` | 155 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 144 | `ADD` | `[149, 155]` | 156 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 145 | `RELU` | `[156]` | 157 | `{}` |
| 146 | `DEQUANTIZE` | `[159]` | 222 | `{}` |
| 147 | `DEQUANTIZE` | `[158]` | 245 | `{}` |
| 148 | `CONV_2D` | `[114, 245, 222]` | 160 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 149 | `DEQUANTIZE` | `[161]` | 181 | `{}` |
| 150 | `DEQUANTIZE` | `[162]` | 234 | `{}` |
| 151 | `CONV_2D` | `[157, 181, 234]` | 163 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 152 | `DEQUANTIZE` | `[164]` | 190 | `{}` |
| 153 | `DEQUANTIZE` | `[165]` | 248 | `{}` |
| 154 | `CONV_2D` | `[114, 190, 248]` | 166 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 155 | `DEQUANTIZE` | `[168]` | 183 | `{}` |
| 156 | `DEQUANTIZE` | `[167]` | 208 | `{}` |
| 157 | `CONV_2D` | `[157, 208, 183]` | 169 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 158 | `RESHAPE` | `[160]` | 170 | `{"new_shape": [1, -1, 1]}` |
| 159 | `RESHAPE` | `[163]` | 171 | `{"new_shape": [1, -1, 1]}` |
| 160 | `RESHAPE` | `[166]` | 172 | `{"new_shape": [1, -1, 16]}` |
| 161 | `RESHAPE` | `[169]` | 173 | `{"new_shape": [1, -1, 16]}` |
| 162 | `CONCATENATION` | `[170, 171]` | 174 | `{"axis": 1, "fused_activation_function": "NONE"}` |
| 163 | `CONCATENATION` | `[172, 173]` | 175 | `{"axis": 1, "fused_activation_function": "NONE"}` |

### Landmarks — `face_landmarks_detector.tflite`

- Schema version: `3`; subgraph: `main`.
- Tensors: `727`; nodes: `471`.
- Inputs: `[0]`; outputs: `[473, 472, 475]`.
- Dtypes: `{"FLOAT16": 251, "FLOAT32": 472, "INT32": 4}`.
- Corrected operator inventory: `{"ADD": 34, "CONV_2D": 72, "DEPTHWISE_CONV_2D": 34, "DEQUANTIZE": 251, "LOGISTIC": 1, "MAX_POOL_2D": 6, "PAD": 3, "PRELU": 69, "RESHAPE": 1}`.

#### Operator codes

| opcode table index | schema builtin code | name | version | custom code |
| ---: | ---: | --- | ---: | --- |
| 0 | 3 | `CONV_2D` | 1 | `` |
| 1 | 54 | `PRELU` | 1 | `` |
| 2 | 4 | `DEPTHWISE_CONV_2D` | 1 | `` |
| 3 | 0 | `ADD` | 1 | `` |
| 4 | 17 | `MAX_POOL_2D` | 1 | `` |
| 5 | 34 | `PAD` | 1 | `` |
| 6 | 14 | `LOGISTIC` | 1 | `` |
| 7 | 22 | `RESHAPE` | 1 | `` |
| 8 | 6 | `DEQUANTIZE` | 2 | `` |

#### Tensors

| index | name | dtype | shape | buffer index | absolute offset | bytes |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 0 | `input_12` | `FLOAT32` | `[1, 256, 256, 3]` | 1 | None | 0 |
| 1 | `model_5/model_4/model_3/batch_normalization_132/FusedBatchNormV3;model_5/model_4/model_3/conv2d_81/BiasAdd/ReadVariableOp/resource;model_5/model_4/model_3/conv2d_81/BiasAdd` | `FLOAT16` | `[16]` | 2 | 2639728 | 32 |
| 2 | `model_5/model_4/model_3/p_re_lu_126/add;model_5/model_4/model_3/p_re_lu_126/Relu;model_5/model_4/model_3/p_re_lu_126/Neg_1;model_5/model_4/model_3/p_re_lu_126/Relu_1;model_5/model_4/model_3/p_re_lu_126/mul` | `FLOAT16` | `[1, 1, 16]` | 3 | 2639676 | 32 |
| 3 | `model_5/model_4/model_3/batch_normalization_133/FusedBatchNormV3` | `FLOAT16` | `[8]` | 4 | 2639648 | 16 |
| 4 | `model_5/model_4/model_3/p_re_lu_127/add;model_5/model_4/model_3/p_re_lu_127/Relu;model_5/model_4/model_3/p_re_lu_127/Neg_1;model_5/model_4/model_3/p_re_lu_127/Relu_1;model_5/model_4/model_3/p_re_lu_127/mul` | `FLOAT16` | `[1, 1, 8]` | 5 | 2639620 | 16 |
| 5 | `model_5/model_4/model_3/batch_normalization_134/FusedBatchNormV3` | `FLOAT16` | `[16]` | 6 | 2639576 | 32 |
| 6 | `model_5/model_4/model_3/p_re_lu_128/add;model_5/model_4/model_3/p_re_lu_128/Relu;model_5/model_4/model_3/p_re_lu_128/Neg_1;model_5/model_4/model_3/p_re_lu_128/Relu_1;model_5/model_4/model_3/p_re_lu_128/mul` | `FLOAT16` | `[1, 1, 16]` | 7 | 2639532 | 32 |
| 7 | `model_5/model_4/model_3/batch_normalization_135/FusedBatchNormV3` | `FLOAT16` | `[8]` | 8 | 2639504 | 16 |
| 8 | `model_5/model_4/model_3/p_re_lu_129/add;model_5/model_4/model_3/p_re_lu_129/Relu;model_5/model_4/model_3/p_re_lu_129/Neg_1;model_5/model_4/model_3/p_re_lu_129/Relu_1;model_5/model_4/model_3/p_re_lu_129/mul` | `FLOAT16` | `[1, 1, 8]` | 9 | 2639476 | 16 |
| 9 | `model_5/model_4/model_3/batch_normalization_136/FusedBatchNormV3` | `FLOAT16` | `[16]` | 10 | 2639432 | 32 |
| 10 | `model_5/model_4/model_3/p_re_lu_130/add;model_5/model_4/model_3/p_re_lu_130/Relu;model_5/model_4/model_3/p_re_lu_130/Neg_1;model_5/model_4/model_3/p_re_lu_130/Relu_1;model_5/model_4/model_3/p_re_lu_130/mul` | `FLOAT16` | `[1, 1, 16]` | 11 | 2639388 | 32 |
| 11 | `model_5/model_4/model_3/batch_normalization_137/FusedBatchNormV3` | `FLOAT16` | `[8]` | 12 | 2639360 | 16 |
| 12 | `model_5/model_4/model_3/p_re_lu_131/add;model_5/model_4/model_3/p_re_lu_131/Relu;model_5/model_4/model_3/p_re_lu_131/Neg_1;model_5/model_4/model_3/p_re_lu_131/Relu_1;model_5/model_4/model_3/p_re_lu_131/mul` | `FLOAT16` | `[1, 1, 8]` | 13 | 2639332 | 16 |
| 13 | `model_5/model_4/model_3/batch_normalization_138/FusedBatchNormV3` | `FLOAT16` | `[16]` | 14 | 2639288 | 32 |
| 14 | `model_5/model_4/model_3/p_re_lu_132/add;model_5/model_4/model_3/p_re_lu_132/Relu;model_5/model_4/model_3/p_re_lu_132/Neg_1;model_5/model_4/model_3/p_re_lu_132/Relu_1;model_5/model_4/model_3/p_re_lu_132/mul` | `FLOAT16` | `[1, 1, 16]` | 15 | 2639244 | 32 |
| 15 | `model_5/model_4/model_3/batch_normalization_139/FusedBatchNormV3` | `FLOAT16` | `[8]` | 16 | 2639216 | 16 |
| 16 | `model_5/model_4/model_3/p_re_lu_133/add;model_5/model_4/model_3/p_re_lu_133/Relu;model_5/model_4/model_3/p_re_lu_133/Neg_1;model_5/model_4/model_3/p_re_lu_133/Relu_1;model_5/model_4/model_3/p_re_lu_133/mul` | `FLOAT16` | `[1, 1, 8]` | 17 | 2639188 | 16 |
| 17 | `model_5/model_4/model_3/batch_normalization_140/FusedBatchNormV3` | `FLOAT16` | `[16]` | 18 | 2639144 | 32 |
| 18 | `model_5/model_4/model_3/p_re_lu_134/add;model_5/model_4/model_3/p_re_lu_134/Relu;model_5/model_4/model_3/p_re_lu_134/Neg_1;model_5/model_4/model_3/p_re_lu_134/Relu_1;model_5/model_4/model_3/p_re_lu_134/mul` | `FLOAT16` | `[1, 1, 16]` | 19 | 2639100 | 32 |
| 19 | `model_5/model_4/model_3/batch_normalization_141/FusedBatchNormV3` | `FLOAT16` | `[16]` | 20 | 2639056 | 32 |
| 20 | `model_5/model_4/model_3/p_re_lu_135/add;model_5/model_4/model_3/p_re_lu_135/Relu;model_5/model_4/model_3/p_re_lu_135/Neg_1;model_5/model_4/model_3/p_re_lu_135/Relu_1;model_5/model_4/model_3/p_re_lu_135/mul` | `FLOAT16` | `[1, 1, 16]` | 21 | 2639012 | 32 |
| 21 | `model_5/model_4/model_3/batch_normalization_142/FusedBatchNormV3` | `FLOAT16` | `[32]` | 22 | 2638936 | 64 |
| 22 | `model_5/model_4/model_3/p_re_lu_136/add;model_5/model_4/model_3/p_re_lu_136/Relu;model_5/model_4/model_3/p_re_lu_136/Neg_1;model_5/model_4/model_3/p_re_lu_136/Relu_1;model_5/model_4/model_3/p_re_lu_136/mul` | `FLOAT16` | `[1, 1, 32]` | 23 | 2638860 | 64 |
| 23 | `model_5/model_4/model_3/batch_normalization_143/FusedBatchNormV3` | `FLOAT16` | `[16]` | 24 | 2638816 | 32 |
| 24 | `model_5/model_4/model_3/p_re_lu_137/add;model_5/model_4/model_3/p_re_lu_137/Relu;model_5/model_4/model_3/p_re_lu_137/Neg_1;model_5/model_4/model_3/p_re_lu_137/Relu_1;model_5/model_4/model_3/p_re_lu_137/mul` | `FLOAT16` | `[1, 1, 16]` | 25 | 2638772 | 32 |
| 25 | `model_5/model_4/model_3/batch_normalization_144/FusedBatchNormV3` | `FLOAT16` | `[32]` | 26 | 2638696 | 64 |
| 26 | `model_5/model_4/model_3/p_re_lu_138/add;model_5/model_4/model_3/p_re_lu_138/Relu;model_5/model_4/model_3/p_re_lu_138/Neg_1;model_5/model_4/model_3/p_re_lu_138/Relu_1;model_5/model_4/model_3/p_re_lu_138/mul` | `FLOAT16` | `[1, 1, 32]` | 27 | 2638620 | 64 |
| 27 | `model_5/model_4/model_3/batch_normalization_145/FusedBatchNormV3` | `FLOAT16` | `[16]` | 28 | 2638576 | 32 |
| 28 | `model_5/model_4/model_3/p_re_lu_139/add;model_5/model_4/model_3/p_re_lu_139/Relu;model_5/model_4/model_3/p_re_lu_139/Neg_1;model_5/model_4/model_3/p_re_lu_139/Relu_1;model_5/model_4/model_3/p_re_lu_139/mul` | `FLOAT16` | `[1, 1, 16]` | 29 | 2638532 | 32 |
| 29 | `model_5/model_4/model_3/batch_normalization_146/FusedBatchNormV3` | `FLOAT16` | `[32]` | 30 | 2638456 | 64 |
| 30 | `model_5/model_4/model_3/p_re_lu_140/add;model_5/model_4/model_3/p_re_lu_140/Relu;model_5/model_4/model_3/p_re_lu_140/Neg_1;model_5/model_4/model_3/p_re_lu_140/Relu_1;model_5/model_4/model_3/p_re_lu_140/mul` | `FLOAT16` | `[1, 1, 32]` | 31 | 2638380 | 64 |
| 31 | `model_5/model_4/model_3/batch_normalization_147/FusedBatchNormV3` | `FLOAT16` | `[16]` | 32 | 2638336 | 32 |
| 32 | `model_5/model_4/model_3/p_re_lu_141/add;model_5/model_4/model_3/p_re_lu_141/Relu;model_5/model_4/model_3/p_re_lu_141/Neg_1;model_5/model_4/model_3/p_re_lu_141/Relu_1;model_5/model_4/model_3/p_re_lu_141/mul` | `FLOAT16` | `[1, 1, 16]` | 33 | 2638292 | 32 |
| 33 | `model_5/model_4/model_3/batch_normalization_148/FusedBatchNormV3` | `FLOAT16` | `[32]` | 34 | 2638216 | 64 |
| 34 | `model_5/model_4/model_3/p_re_lu_142/add;model_5/model_4/model_3/p_re_lu_142/Relu;model_5/model_4/model_3/p_re_lu_142/Neg_1;model_5/model_4/model_3/p_re_lu_142/Relu_1;model_5/model_4/model_3/p_re_lu_142/mul` | `FLOAT16` | `[1, 1, 32]` | 35 | 2638140 | 64 |
| 35 | `model_5/model_4/model_3/batch_normalization_149/FusedBatchNormV3` | `FLOAT16` | `[16]` | 36 | 2638096 | 32 |
| 36 | `model_5/model_4/model_3/p_re_lu_143/add;model_5/model_4/model_3/p_re_lu_143/Relu;model_5/model_4/model_3/p_re_lu_143/Neg_1;model_5/model_4/model_3/p_re_lu_143/Relu_1;model_5/model_4/model_3/p_re_lu_143/mul` | `FLOAT16` | `[1, 1, 16]` | 37 | 2638052 | 32 |
| 37 | `model_5/model_4/model_3/batch_normalization_150/FusedBatchNormV3` | `FLOAT16` | `[32]` | 38 | 2637976 | 64 |
| 38 | `model_5/model_4/model_3/p_re_lu_144/add;model_5/model_4/model_3/p_re_lu_144/Relu;model_5/model_4/model_3/p_re_lu_144/Neg_1;model_5/model_4/model_3/p_re_lu_144/Relu_1;model_5/model_4/model_3/p_re_lu_144/mul` | `FLOAT16` | `[1, 1, 32]` | 39 | 2637900 | 64 |
| 39 | `model_5/model_4/model_3/batch_normalization_151/FusedBatchNormV3` | `FLOAT16` | `[32]` | 40 | 2637824 | 64 |
| 40 | `model_5/model_4/model_3/p_re_lu_145/add;model_5/model_4/model_3/p_re_lu_145/Relu;model_5/model_4/model_3/p_re_lu_145/Neg_1;model_5/model_4/model_3/p_re_lu_145/Relu_1;model_5/model_4/model_3/p_re_lu_145/mul` | `FLOAT16` | `[1, 1, 32]` | 41 | 2637748 | 64 |
| 41 | `model_5/model_4/model_3/batch_normalization_152/FusedBatchNormV3` | `FLOAT16` | `[64]` | 42 | 2637608 | 128 |
| 42 | `model_5/model_4/model_3/p_re_lu_146/add;model_5/model_4/model_3/p_re_lu_146/Relu;model_5/model_4/model_3/p_re_lu_146/Neg_1;model_5/model_4/model_3/p_re_lu_146/Relu_1;model_5/model_4/model_3/p_re_lu_146/mul` | `FLOAT16` | `[1, 1, 64]` | 43 | 2637468 | 128 |
| 43 | `model_5/model_4/model_3/batch_normalization_153/FusedBatchNormV3` | `FLOAT16` | `[32]` | 44 | 2637392 | 64 |
| 44 | `model_5/model_4/model_3/p_re_lu_147/add;model_5/model_4/model_3/p_re_lu_147/Relu;model_5/model_4/model_3/p_re_lu_147/Neg_1;model_5/model_4/model_3/p_re_lu_147/Relu_1;model_5/model_4/model_3/p_re_lu_147/mul` | `FLOAT16` | `[1, 1, 32]` | 45 | 2637316 | 64 |
| 45 | `model_5/model_4/model_3/batch_normalization_154/FusedBatchNormV3` | `FLOAT16` | `[64]` | 46 | 2637176 | 128 |
| 46 | `model_5/model_4/model_3/p_re_lu_148/add;model_5/model_4/model_3/p_re_lu_148/Relu;model_5/model_4/model_3/p_re_lu_148/Neg_1;model_5/model_4/model_3/p_re_lu_148/Relu_1;model_5/model_4/model_3/p_re_lu_148/mul` | `FLOAT16` | `[1, 1, 64]` | 47 | 2637036 | 128 |
| 47 | `model_5/model_4/model_3/batch_normalization_155/FusedBatchNormV3` | `FLOAT16` | `[32]` | 48 | 2636960 | 64 |
| 48 | `model_5/model_4/model_3/p_re_lu_149/add;model_5/model_4/model_3/p_re_lu_149/Relu;model_5/model_4/model_3/p_re_lu_149/Neg_1;model_5/model_4/model_3/p_re_lu_149/Relu_1;model_5/model_4/model_3/p_re_lu_149/mul` | `FLOAT16` | `[1, 1, 32]` | 49 | 2636884 | 64 |
| 49 | `model_5/model_4/model_3/batch_normalization_156/FusedBatchNormV3` | `FLOAT16` | `[64]` | 50 | 2636744 | 128 |
| 50 | `model_5/model_4/model_3/p_re_lu_150/add;model_5/model_4/model_3/p_re_lu_150/Relu;model_5/model_4/model_3/p_re_lu_150/Neg_1;model_5/model_4/model_3/p_re_lu_150/Relu_1;model_5/model_4/model_3/p_re_lu_150/mul` | `FLOAT16` | `[1, 1, 64]` | 51 | 2636604 | 128 |
| 51 | `model_5/model_4/model_3/batch_normalization_157/FusedBatchNormV3` | `FLOAT16` | `[32]` | 52 | 2636528 | 64 |
| 52 | `model_5/model_4/model_3/p_re_lu_151/add;model_5/model_4/model_3/p_re_lu_151/Relu;model_5/model_4/model_3/p_re_lu_151/Neg_1;model_5/model_4/model_3/p_re_lu_151/Relu_1;model_5/model_4/model_3/p_re_lu_151/mul` | `FLOAT16` | `[1, 1, 32]` | 53 | 2636452 | 64 |
| 53 | `model_5/model_4/model_3/batch_normalization_158/FusedBatchNormV3` | `FLOAT16` | `[64]` | 54 | 2636312 | 128 |
| 54 | `model_5/model_4/model_3/p_re_lu_152/add;model_5/model_4/model_3/p_re_lu_152/Relu;model_5/model_4/model_3/p_re_lu_152/Neg_1;model_5/model_4/model_3/p_re_lu_152/Relu_1;model_5/model_4/model_3/p_re_lu_152/mul` | `FLOAT16` | `[1, 1, 64]` | 55 | 2636172 | 128 |
| 55 | `model_5/model_4/model_3/batch_normalization_159/FusedBatchNormV3` | `FLOAT16` | `[32]` | 56 | 2636096 | 64 |
| 56 | `model_5/model_4/model_3/p_re_lu_153/add;model_5/model_4/model_3/p_re_lu_153/Relu;model_5/model_4/model_3/p_re_lu_153/Neg_1;model_5/model_4/model_3/p_re_lu_153/Relu_1;model_5/model_4/model_3/p_re_lu_153/mul` | `FLOAT16` | `[1, 1, 32]` | 57 | 2636020 | 64 |
| 57 | `model_5/model_4/model_3/batch_normalization_160/FusedBatchNormV3` | `FLOAT16` | `[64]` | 58 | 2635880 | 128 |
| 58 | `model_5/model_4/model_3/p_re_lu_154/add;model_5/model_4/model_3/p_re_lu_154/Relu;model_5/model_4/model_3/p_re_lu_154/Neg_1;model_5/model_4/model_3/p_re_lu_154/Relu_1;model_5/model_4/model_3/p_re_lu_154/mul` | `FLOAT16` | `[1, 1, 64]` | 59 | 2635740 | 128 |
| 59 | `model_5/model_4/model_3/batch_normalization_161/FusedBatchNormV3` | `FLOAT16` | `[64]` | 60 | 2635600 | 128 |
| 60 | `model_5/model_4/model_3/p_re_lu_155/add;model_5/model_4/model_3/p_re_lu_155/Relu;model_5/model_4/model_3/p_re_lu_155/Neg_1;model_5/model_4/model_3/p_re_lu_155/Relu_1;model_5/model_4/model_3/p_re_lu_155/mul` | `FLOAT16` | `[1, 1, 64]` | 61 | 2635460 | 128 |
| 61 | `model_5/model_4/model_3/batch_normalization_162/FusedBatchNormV3` | `FLOAT16` | `[128]` | 62 | 2635192 | 256 |
| 62 | `model_5/model_4/model_3/p_re_lu_156/add;model_5/model_4/model_3/p_re_lu_156/Relu;model_5/model_4/model_3/p_re_lu_156/Neg_1;model_5/model_4/model_3/p_re_lu_156/Relu_1;model_5/model_4/model_3/p_re_lu_156/mul` | `FLOAT16` | `[1, 1, 128]` | 63 | 2634924 | 256 |
| 63 | `model_5/model_4/model_3/batch_normalization_163/FusedBatchNormV3` | `FLOAT16` | `[64]` | 64 | 2634784 | 128 |
| 64 | `model_5/model_4/model_3/p_re_lu_157/add;model_5/model_4/model_3/p_re_lu_157/Relu;model_5/model_4/model_3/p_re_lu_157/Neg_1;model_5/model_4/model_3/p_re_lu_157/Relu_1;model_5/model_4/model_3/p_re_lu_157/mul` | `FLOAT16` | `[1, 1, 64]` | 65 | 2634644 | 128 |
| 65 | `model_5/model_4/model_3/batch_normalization_164/FusedBatchNormV3` | `FLOAT16` | `[128]` | 66 | 2634376 | 256 |
| 66 | `model_5/model_4/model_3/p_re_lu_158/add;model_5/model_4/model_3/p_re_lu_158/Relu;model_5/model_4/model_3/p_re_lu_158/Neg_1;model_5/model_4/model_3/p_re_lu_158/Relu_1;model_5/model_4/model_3/p_re_lu_158/mul` | `FLOAT16` | `[1, 1, 128]` | 67 | 2634108 | 256 |
| 67 | `model_5/model_4/model_3/batch_normalization_165/FusedBatchNormV3` | `FLOAT16` | `[64]` | 68 | 2633968 | 128 |
| 68 | `model_5/model_4/model_3/p_re_lu_159/add;model_5/model_4/model_3/p_re_lu_159/Relu;model_5/model_4/model_3/p_re_lu_159/Neg_1;model_5/model_4/model_3/p_re_lu_159/Relu_1;model_5/model_4/model_3/p_re_lu_159/mul` | `FLOAT16` | `[1, 1, 64]` | 69 | 2633828 | 128 |
| 69 | `model_5/model_4/model_3/batch_normalization_166/FusedBatchNormV3` | `FLOAT16` | `[128]` | 70 | 2633560 | 256 |
| 70 | `model_5/model_4/model_3/p_re_lu_160/add;model_5/model_4/model_3/p_re_lu_160/Relu;model_5/model_4/model_3/p_re_lu_160/Neg_1;model_5/model_4/model_3/p_re_lu_160/Relu_1;model_5/model_4/model_3/p_re_lu_160/mul` | `FLOAT16` | `[1, 1, 128]` | 71 | 2633292 | 256 |
| 71 | `model_5/model_4/model_3/batch_normalization_167/FusedBatchNormV3` | `FLOAT16` | `[64]` | 72 | 2633152 | 128 |
| 72 | `model_5/model_4/model_3/p_re_lu_161/add;model_5/model_4/model_3/p_re_lu_161/Relu;model_5/model_4/model_3/p_re_lu_161/Neg_1;model_5/model_4/model_3/p_re_lu_161/Relu_1;model_5/model_4/model_3/p_re_lu_161/mul` | `FLOAT16` | `[1, 1, 64]` | 73 | 2633012 | 128 |
| 73 | `model_5/model_4/model_3/batch_normalization_168/FusedBatchNormV3` | `FLOAT16` | `[128]` | 74 | 2632744 | 256 |
| 74 | `model_5/model_4/model_3/p_re_lu_162/add;model_5/model_4/model_3/p_re_lu_162/Relu;model_5/model_4/model_3/p_re_lu_162/Neg_1;model_5/model_4/model_3/p_re_lu_162/Relu_1;model_5/model_4/model_3/p_re_lu_162/mul` | `FLOAT16` | `[1, 1, 128]` | 75 | 2632476 | 256 |
| 75 | `model_5/model_4/model_3/batch_normalization_169/FusedBatchNormV3` | `FLOAT16` | `[64]` | 76 | 2632336 | 128 |
| 76 | `model_5/model_4/model_3/p_re_lu_163/add;model_5/model_4/model_3/p_re_lu_163/Relu;model_5/model_4/model_3/p_re_lu_163/Neg_1;model_5/model_4/model_3/p_re_lu_163/Relu_1;model_5/model_4/model_3/p_re_lu_163/mul` | `FLOAT16` | `[1, 1, 64]` | 77 | 2632196 | 128 |
| 77 | `model_5/model_4/model_3/batch_normalization_170/FusedBatchNormV3` | `FLOAT16` | `[128]` | 78 | 2631928 | 256 |
| 78 | `model_5/model_4/model_3/p_re_lu_164/add;model_5/model_4/model_3/p_re_lu_164/Relu;model_5/model_4/model_3/p_re_lu_164/Neg_1;model_5/model_4/model_3/p_re_lu_164/Relu_1;model_5/model_4/model_3/p_re_lu_164/mul` | `FLOAT16` | `[1, 1, 128]` | 79 | 2631660 | 256 |
| 79 | `model_5/model_4/model_3/batch_normalization_171/FusedBatchNormV3` | `FLOAT16` | `[64]` | 80 | 2631520 | 128 |
| 80 | `model_5/model_4/model_3/p_re_lu_165/add;model_5/model_4/model_3/p_re_lu_165/Relu;model_5/model_4/model_3/p_re_lu_165/Neg_1;model_5/model_4/model_3/p_re_lu_165/Relu_1;model_5/model_4/model_3/p_re_lu_165/mul` | `FLOAT16` | `[1, 1, 64]` | 81 | 2631380 | 128 |
| 81 | `model_5/model_4/model_3/batch_normalization_172/FusedBatchNormV3` | `FLOAT16` | `[128]` | 82 | 2631112 | 256 |
| 82 | `model_5/model_4/model_3/p_re_lu_166/add;model_5/model_4/model_3/p_re_lu_166/Relu;model_5/model_4/model_3/p_re_lu_166/Neg_1;model_5/model_4/model_3/p_re_lu_166/Relu_1;model_5/model_4/model_3/p_re_lu_166/mul` | `FLOAT16` | `[1, 1, 128]` | 83 | 2630844 | 256 |
| 83 | `model_5/model_4/model_3/batch_normalization_173/FusedBatchNormV3` | `FLOAT16` | `[64]` | 84 | 2630704 | 128 |
| 84 | `model_5/model_4/model_3/p_re_lu_167/add;model_5/model_4/model_3/p_re_lu_167/Relu;model_5/model_4/model_3/p_re_lu_167/Neg_1;model_5/model_4/model_3/p_re_lu_167/Relu_1;model_5/model_4/model_3/p_re_lu_167/mul` | `FLOAT16` | `[1, 1, 64]` | 85 | 2630564 | 128 |
| 85 | `model_5/model_4/model_3/batch_normalization_174/FusedBatchNormV3` | `FLOAT16` | `[128]` | 86 | 2630296 | 256 |
| 86 | `model_5/model_4/model_3/p_re_lu_168/add;model_5/model_4/model_3/p_re_lu_168/Relu;model_5/model_4/model_3/p_re_lu_168/Neg_1;model_5/model_4/model_3/p_re_lu_168/Relu_1;model_5/model_4/model_3/p_re_lu_168/mul` | `FLOAT16` | `[1, 1, 128]` | 87 | 2630028 | 256 |
| 87 | `model_5/model_4/model_3/batch_normalization_175/FusedBatchNormV3` | `FLOAT16` | `[64]` | 88 | 2629888 | 128 |
| 88 | `model_5/model_4/model_3/p_re_lu_169/add;model_5/model_4/model_3/p_re_lu_169/Relu;model_5/model_4/model_3/p_re_lu_169/Neg_1;model_5/model_4/model_3/p_re_lu_169/Relu_1;model_5/model_4/model_3/p_re_lu_169/mul` | `FLOAT16` | `[1, 1, 64]` | 89 | 2629748 | 128 |
| 89 | `model_5/model_4/model_3/batch_normalization_176/FusedBatchNormV3` | `FLOAT16` | `[128]` | 90 | 2629480 | 256 |
| 90 | `model_5/model_4/model_3/p_re_lu_170/add;model_5/model_4/model_3/p_re_lu_170/Relu;model_5/model_4/model_3/p_re_lu_170/Neg_1;model_5/model_4/model_3/p_re_lu_170/Relu_1;model_5/model_4/model_3/p_re_lu_170/mul` | `FLOAT16` | `[1, 1, 128]` | 91 | 2629212 | 256 |
| 91 | `model_5/model_4/model_3/batch_normalization_177/FusedBatchNormV3` | `FLOAT16` | `[64]` | 92 | 2629072 | 128 |
| 92 | `model_5/model_4/model_3/p_re_lu_171/add;model_5/model_4/model_3/p_re_lu_171/Relu;model_5/model_4/model_3/p_re_lu_171/Neg_1;model_5/model_4/model_3/p_re_lu_171/Relu_1;model_5/model_4/model_3/p_re_lu_171/mul` | `FLOAT16` | `[1, 1, 64]` | 93 | 2628932 | 128 |
| 93 | `model_5/model_4/model_3/batch_normalization_178/FusedBatchNormV3` | `FLOAT16` | `[128]` | 94 | 2628664 | 256 |
| 94 | `model_5/model_4/model_3/p_re_lu_172/add;model_5/model_4/model_3/p_re_lu_172/Relu;model_5/model_4/model_3/p_re_lu_172/Neg_1;model_5/model_4/model_3/p_re_lu_172/Relu_1;model_5/model_4/model_3/p_re_lu_172/mul` | `FLOAT16` | `[1, 1, 128]` | 95 | 2628396 | 256 |
| 95 | `model_5/model_4/model_3/batch_normalization_179/FusedBatchNormV3` | `FLOAT16` | `[64]` | 96 | 2628256 | 128 |
| 96 | `model_5/model_4/model_3/p_re_lu_173/add;model_5/model_4/model_3/p_re_lu_173/Relu;model_5/model_4/model_3/p_re_lu_173/Neg_1;model_5/model_4/model_3/p_re_lu_173/Relu_1;model_5/model_4/model_3/p_re_lu_173/mul` | `FLOAT16` | `[1, 1, 64]` | 97 | 2628116 | 128 |
| 97 | `model_5/model_4/model_3/batch_normalization_180/FusedBatchNormV3` | `FLOAT16` | `[128]` | 98 | 2627848 | 256 |
| 98 | `model_5/model_4/model_3/p_re_lu_174/add;model_5/model_4/model_3/p_re_lu_174/Relu;model_5/model_4/model_3/p_re_lu_174/Neg_1;model_5/model_4/model_3/p_re_lu_174/Relu_1;model_5/model_4/model_3/p_re_lu_174/mul` | `FLOAT16` | `[1, 1, 128]` | 99 | 2627580 | 256 |
| 99 | `model_5/model_4/model_3/batch_normalization_181/FusedBatchNormV3` | `FLOAT16` | `[64]` | 100 | 2627440 | 128 |
| 100 | `model_5/model_4/model_3/p_re_lu_175/add;model_5/model_4/model_3/p_re_lu_175/Relu;model_5/model_4/model_3/p_re_lu_175/Neg_1;model_5/model_4/model_3/p_re_lu_175/Relu_1;model_5/model_4/model_3/p_re_lu_175/mul` | `FLOAT16` | `[1, 1, 64]` | 101 | 2627300 | 128 |
| 101 | `model_5/model_4/model_3/batch_normalization_182/FusedBatchNormV3` | `FLOAT16` | `[128]` | 102 | 2627032 | 256 |
| 102 | `model_5/model_4/model_3/p_re_lu_176/add;model_5/model_4/model_3/p_re_lu_176/Relu;model_5/model_4/model_3/p_re_lu_176/Neg_1;model_5/model_4/model_3/p_re_lu_176/Relu_1;model_5/model_4/model_3/p_re_lu_176/mul` | `FLOAT16` | `[1, 1, 128]` | 103 | 2626764 | 256 |
| 103 | `model_5/model_4/model_3/batch_normalization_183/FusedBatchNormV3` | `FLOAT16` | `[64]` | 104 | 2626624 | 128 |
| 104 | `model_5/model_4/model_3/p_re_lu_177/add;model_5/model_4/model_3/p_re_lu_177/Relu;model_5/model_4/model_3/p_re_lu_177/Neg_1;model_5/model_4/model_3/p_re_lu_177/Relu_1;model_5/model_4/model_3/p_re_lu_177/mul` | `FLOAT16` | `[1, 1, 64]` | 105 | 2626484 | 128 |
| 105 | `model_5/model_4/model_3/batch_normalization_184/FusedBatchNormV3` | `FLOAT16` | `[128]` | 106 | 2626216 | 256 |
| 106 | `model_5/model_4/model_3/p_re_lu_178/add;model_5/model_4/model_3/p_re_lu_178/Relu;model_5/model_4/model_3/p_re_lu_178/Neg_1;model_5/model_4/model_3/p_re_lu_178/Relu_1;model_5/model_4/model_3/p_re_lu_178/mul` | `FLOAT16` | `[1, 1, 128]` | 107 | 2625948 | 256 |
| 107 | `model_5/model_4/model_3/batch_normalization_185/FusedBatchNormV3` | `FLOAT16` | `[64]` | 108 | 2625808 | 128 |
| 108 | `model_5/model_4/model_3/p_re_lu_179/add;model_5/model_4/model_3/p_re_lu_179/Relu;model_5/model_4/model_3/p_re_lu_179/Neg_1;model_5/model_4/model_3/p_re_lu_179/Relu_1;model_5/model_4/model_3/p_re_lu_179/mul` | `FLOAT16` | `[1, 1, 64]` | 109 | 2625668 | 128 |
| 109 | `model_5/model_4/model_3/batch_normalization_186/FusedBatchNormV3` | `FLOAT16` | `[128]` | 110 | 2625400 | 256 |
| 110 | `model_5/model_4/model_3/p_re_lu_180/add;model_5/model_4/model_3/p_re_lu_180/Relu;model_5/model_4/model_3/p_re_lu_180/Neg_1;model_5/model_4/model_3/p_re_lu_180/Relu_1;model_5/model_4/model_3/p_re_lu_180/mul` | `FLOAT16` | `[1, 1, 128]` | 111 | 2625132 | 256 |
| 111 | `model_5/model_4/model_3/batch_normalization_187/FusedBatchNormV3` | `FLOAT16` | `[64]` | 112 | 2624992 | 128 |
| 112 | `model_5/model_4/model_3/p_re_lu_181/add;model_5/model_4/model_3/p_re_lu_181/Relu;model_5/model_4/model_3/p_re_lu_181/Neg_1;model_5/model_4/model_3/p_re_lu_181/Relu_1;model_5/model_4/model_3/p_re_lu_181/mul` | `FLOAT16` | `[1, 1, 64]` | 113 | 2624852 | 128 |
| 113 | `model_5/model_4/model_3/batch_normalization_188/FusedBatchNormV3` | `FLOAT16` | `[128]` | 114 | 2624584 | 256 |
| 114 | `model_5/model_4/model_3/p_re_lu_182/add;model_5/model_4/model_3/p_re_lu_182/Relu;model_5/model_4/model_3/p_re_lu_182/Neg_1;model_5/model_4/model_3/p_re_lu_182/Relu_1;model_5/model_4/model_3/p_re_lu_182/mul` | `FLOAT16` | `[1, 1, 128]` | 115 | 2624316 | 256 |
| 115 | `model_5/model_4/model_3/batch_normalization_189/FusedBatchNormV3` | `FLOAT16` | `[64]` | 116 | 2624176 | 128 |
| 116 | `model_5/model_4/model_3/p_re_lu_183/add;model_5/model_4/model_3/p_re_lu_183/Relu;model_5/model_4/model_3/p_re_lu_183/Neg_1;model_5/model_4/model_3/p_re_lu_183/Relu_1;model_5/model_4/model_3/p_re_lu_183/mul` | `FLOAT16` | `[1, 1, 64]` | 117 | 2624036 | 128 |
| 117 | `model_5/model_4/model_3/batch_normalization_190/FusedBatchNormV3` | `FLOAT16` | `[128]` | 118 | 2623768 | 256 |
| 118 | `model_5/model_4/model_3/p_re_lu_184/add;model_5/model_4/model_3/p_re_lu_184/Relu;model_5/model_4/model_3/p_re_lu_184/Neg_1;model_5/model_4/model_3/p_re_lu_184/Relu_1;model_5/model_4/model_3/p_re_lu_184/mul` | `FLOAT16` | `[1, 1, 128]` | 119 | 2623500 | 256 |
| 119 | `model_5/model_4/model_3/batch_normalization_191/FusedBatchNormV3` | `FLOAT16` | `[64]` | 120 | 2623360 | 128 |
| 120 | `model_5/model_4/model_3/p_re_lu_185/add;model_5/model_4/model_3/p_re_lu_185/Relu;model_5/model_4/model_3/p_re_lu_185/Neg_1;model_5/model_4/model_3/p_re_lu_185/Relu_1;model_5/model_4/model_3/p_re_lu_185/mul` | `FLOAT16` | `[1, 1, 64]` | 121 | 2623220 | 128 |
| 121 | `model_5/model_4/model_3/batch_normalization_192/FusedBatchNormV3` | `FLOAT16` | `[128]` | 122 | 2622952 | 256 |
| 122 | `model_5/model_4/model_3/p_re_lu_186/add;model_5/model_4/model_3/p_re_lu_186/Relu;model_5/model_4/model_3/p_re_lu_186/Neg_1;model_5/model_4/model_3/p_re_lu_186/Relu_1;model_5/model_4/model_3/p_re_lu_186/mul` | `FLOAT16` | `[1, 1, 128]` | 123 | 2622684 | 256 |
| 123 | `model_5/model_4/model_3/batch_normalization_193/FusedBatchNormV3` | `FLOAT16` | `[64]` | 124 | 2622544 | 128 |
| 124 | `model_5/model_4/model_3/p_re_lu_187/add;model_5/model_4/model_3/p_re_lu_187/Relu;model_5/model_4/model_3/p_re_lu_187/Neg_1;model_5/model_4/model_3/p_re_lu_187/Relu_1;model_5/model_4/model_3/p_re_lu_187/mul` | `FLOAT16` | `[1, 1, 64]` | 125 | 2622404 | 128 |
| 125 | `model_5/model_4/model_3/batch_normalization_194/FusedBatchNormV3` | `FLOAT16` | `[128]` | 126 | 2622136 | 256 |
| 126 | `model_5/model_4/model_3/p_re_lu_188/add;model_5/model_4/model_3/p_re_lu_188/Relu;model_5/model_4/model_3/p_re_lu_188/Neg_1;model_5/model_4/model_3/p_re_lu_188/Relu_1;model_5/model_4/model_3/p_re_lu_188/mul` | `FLOAT16` | `[1, 1, 128]` | 127 | 2621868 | 256 |
| 127 | `model_5/model_4/model_3/batch_normalization_195/FusedBatchNormV3` | `FLOAT16` | `[64]` | 128 | 2621728 | 128 |
| 128 | `model_5/model_4/model_3/p_re_lu_189/add;model_5/model_4/model_3/p_re_lu_189/Relu;model_5/model_4/model_3/p_re_lu_189/Neg_1;model_5/model_4/model_3/p_re_lu_189/Relu_1;model_5/model_4/model_3/p_re_lu_189/mul` | `FLOAT16` | `[1, 1, 64]` | 129 | 2621588 | 128 |
| 129 | `model_5/model_4/model_3/batch_normalization_196/FusedBatchNormV3` | `FLOAT16` | `[128]` | 130 | 2621320 | 256 |
| 130 | `model_5/model_4/model_3/p_re_lu_190/add;model_5/model_4/model_3/p_re_lu_190/Relu;model_5/model_4/model_3/p_re_lu_190/Neg_1;model_5/model_4/model_3/p_re_lu_190/Relu_1;model_5/model_4/model_3/p_re_lu_190/mul` | `FLOAT16` | `[1, 1, 128]` | 131 | 2621052 | 256 |
| 131 | `model_5/model_4/model_3/batch_normalization_197/FusedBatchNormV3` | `FLOAT16` | `[64]` | 132 | 2620912 | 128 |
| 132 | `model_5/model_4/model_3/p_re_lu_191/add;model_5/model_4/model_3/p_re_lu_191/Relu;model_5/model_4/model_3/p_re_lu_191/Neg_1;model_5/model_4/model_3/p_re_lu_191/Relu_1;model_5/model_4/model_3/p_re_lu_191/mul` | `FLOAT16` | `[1, 1, 64]` | 133 | 2620772 | 128 |
| 133 | `model_5/model_4/model_3/batch_normalization_198/FusedBatchNormV3` | `FLOAT16` | `[128]` | 134 | 2620504 | 256 |
| 134 | `model_5/model_4/model_3/p_re_lu_192/add;model_5/model_4/model_3/p_re_lu_192/Relu;model_5/model_4/model_3/p_re_lu_192/Neg_1;model_5/model_4/model_3/p_re_lu_192/Relu_1;model_5/model_4/model_3/p_re_lu_192/mul` | `FLOAT16` | `[1, 1, 128]` | 135 | 2620236 | 256 |
| 135 | `model_5/model_4/model_3/batch_normalization_199/FusedBatchNormV3` | `FLOAT16` | `[64]` | 136 | 2620096 | 128 |
| 136 | `model_5/model_4/model_3/p_re_lu_193/add;model_5/model_4/model_3/p_re_lu_193/Relu;model_5/model_4/model_3/p_re_lu_193/Neg_1;model_5/model_4/model_3/p_re_lu_193/Relu_1;model_5/model_4/model_3/p_re_lu_193/mul` | `FLOAT16` | `[1, 1, 64]` | 137 | 2619956 | 128 |
| 137 | `model_5/model_4/model_3/batch_normalization_200/FusedBatchNormV3` | `FLOAT16` | `[128]` | 138 | 2619688 | 256 |
| 138 | `model_5/model_4/model_3/p_re_lu_194/add;model_5/model_4/model_3/p_re_lu_194/Relu;model_5/model_4/model_3/p_re_lu_194/Neg_1;model_5/model_4/model_3/p_re_lu_194/Relu_1;model_5/model_4/model_3/p_re_lu_194/mul` | `FLOAT16` | `[1, 1, 128]` | 139 | 2619420 | 256 |
| 139 | `model_5/conv2d_152/BiasAdd/ReadVariableOp/resource` | `FLOAT16` | `[1]` | 140 | 2619404 | 2 |
| 140 | `model_5/model_4/conv2d_151/BiasAdd/ReadVariableOp/resource` | `FLOAT16` | `[1]` | 141 | 2619388 | 2 |
| 141 | `model_5/model_4/model_3/conv2d_150/BiasAdd/ReadVariableOp/resource` | `FLOAT16` | `[1434]` | 142 | 2616508 | 2868 |
| 142 | `model_5/model_4/model_3/depthwise_conv2d_68/depthwise` | `FLOAT16` | `[16]` | 143 | 2616464 | 32 |
| 143 | `model_5/model_4/model_3/conv2d_81/Conv2D` | `FLOAT16` | `[16, 3, 3, 3]` | 144 | 2615588 | 864 |
| 144 | `model_5/model_4/model_3/depthwise_conv2d_63/depthwise` | `FLOAT16` | `[8]` | 145 | 2615560 | 16 |
| 145 | `model_5/model_4/model_3/conv2d_82/Conv2D` | `FLOAT16` | `[8, 1, 1, 16]` | 146 | 2615292 | 256 |
| 146 | `model_5/model_4/model_3/depthwise_conv2d_60/depthwise` | `FLOAT16` | `[1, 3, 3, 8]` | 147 | 2615136 | 144 |
| 147 | `model_5/model_4/model_3/conv2d_83/Conv2D` | `FLOAT16` | `[16, 1, 1, 8]` | 148 | 2614868 | 256 |
| 148 | `model_5/model_4/model_3/conv2d_84/Conv2D` | `FLOAT16` | `[8, 1, 1, 16]` | 149 | 2614600 | 256 |
| 149 | `model_5/model_4/model_3/depthwise_conv2d_61/depthwise` | `FLOAT16` | `[1, 3, 3, 8]` | 150 | 2614444 | 144 |
| 150 | `model_5/model_4/model_3/conv2d_85/Conv2D` | `FLOAT16` | `[16, 1, 1, 8]` | 151 | 2614176 | 256 |
| 151 | `model_5/model_4/model_3/conv2d_86/Conv2D` | `FLOAT16` | `[8, 1, 1, 16]` | 152 | 2613908 | 256 |
| 152 | `model_5/model_4/model_3/depthwise_conv2d_62/depthwise` | `FLOAT16` | `[1, 3, 3, 8]` | 153 | 2613752 | 144 |
| 153 | `model_5/model_4/model_3/conv2d_87/Conv2D` | `FLOAT16` | `[16, 1, 1, 8]` | 154 | 2613484 | 256 |
| 154 | `model_5/model_4/model_3/conv2d_88/Conv2D` | `FLOAT16` | `[8, 1, 1, 16]` | 155 | 2613216 | 256 |
| 155 | `model_5/model_4/model_3/depthwise_conv2d_63/depthwise1` | `FLOAT16` | `[1, 3, 3, 8]` | 156 | 2613060 | 144 |
| 156 | `model_5/model_4/model_3/conv2d_89/Conv2D` | `FLOAT16` | `[16, 1, 1, 8]` | 157 | 2612792 | 256 |
| 157 | `model_5/model_4/model_3/conv2d_90/Conv2D` | `FLOAT16` | `[16, 2, 2, 16]` | 158 | 2610732 | 2048 |
| 158 | `model_5/model_4/model_3/depthwise_conv2d_64/depthwise` | `FLOAT16` | `[1, 3, 3, 16]` | 159 | 2610432 | 288 |
| 159 | `model_5/model_4/model_3/depthwise_conv2d_73/depthwise` | `FLOAT16` | `[32]` | 160 | 2610356 | 64 |
| 160 | `model_5/model_4/model_3/conv2d_91/Conv2D` | `FLOAT16` | `[32, 1, 1, 16]` | 161 | 2609320 | 1024 |
| 161 | `model_5/model_4/model_3/conv2d_92/Conv2D` | `FLOAT16` | `[16, 1, 1, 32]` | 162 | 2608284 | 1024 |
| 162 | `model_5/model_4/model_3/depthwise_conv2d_65/depthwise` | `FLOAT16` | `[1, 3, 3, 16]` | 163 | 2607984 | 288 |
| 163 | `model_5/model_4/model_3/conv2d_93/Conv2D` | `FLOAT16` | `[32, 1, 1, 16]` | 164 | 2606948 | 1024 |
| 164 | `model_5/model_4/model_3/conv2d_94/Conv2D` | `FLOAT16` | `[16, 1, 1, 32]` | 165 | 2605912 | 1024 |
| 165 | `model_5/model_4/model_3/depthwise_conv2d_66/depthwise` | `FLOAT16` | `[1, 3, 3, 16]` | 166 | 2605612 | 288 |
| 166 | `model_5/model_4/model_3/conv2d_95/Conv2D` | `FLOAT16` | `[32, 1, 1, 16]` | 167 | 2604576 | 1024 |
| 167 | `model_5/model_4/model_3/conv2d_96/Conv2D` | `FLOAT16` | `[16, 1, 1, 32]` | 168 | 2603540 | 1024 |
| 168 | `model_5/model_4/model_3/depthwise_conv2d_67/depthwise` | `FLOAT16` | `[1, 3, 3, 16]` | 169 | 2603240 | 288 |
| 169 | `model_5/model_4/model_3/conv2d_97/Conv2D` | `FLOAT16` | `[32, 1, 1, 16]` | 170 | 2602204 | 1024 |
| 170 | `model_5/model_4/model_3/conv2d_98/Conv2D` | `FLOAT16` | `[16, 1, 1, 32]` | 171 | 2601168 | 1024 |
| 171 | `model_5/model_4/model_3/depthwise_conv2d_68/depthwise1` | `FLOAT16` | `[1, 3, 3, 16]` | 172 | 2600868 | 288 |
| 172 | `model_5/model_4/model_3/conv2d_99/Conv2D` | `FLOAT16` | `[32, 1, 1, 16]` | 173 | 2599832 | 1024 |
| 173 | `model_5/model_4/model_3/conv2d_100/Conv2D` | `FLOAT16` | `[32, 2, 2, 32]` | 174 | 2591628 | 8192 |
| 174 | `model_5/model_4/model_3/depthwise_conv2d_69/depthwise` | `FLOAT16` | `[1, 3, 3, 32]` | 175 | 2591040 | 576 |
| 175 | `model_5/model_4/model_3/depthwise_conv2d_93/depthwise` | `FLOAT16` | `[64]` | 176 | 2590900 | 128 |
| 176 | `model_5/model_4/model_3/conv2d_101/Conv2D` | `FLOAT16` | `[64, 1, 1, 32]` | 177 | 2586792 | 4096 |
| 177 | `model_5/model_4/model_3/conv2d_102/Conv2D` | `FLOAT16` | `[32, 1, 1, 64]` | 178 | 2582684 | 4096 |
| 178 | `model_5/model_4/model_3/depthwise_conv2d_70/depthwise` | `FLOAT16` | `[1, 3, 3, 32]` | 179 | 2582096 | 576 |
| 179 | `model_5/model_4/model_3/conv2d_103/Conv2D` | `FLOAT16` | `[64, 1, 1, 32]` | 180 | 2577988 | 4096 |
| 180 | `model_5/model_4/model_3/conv2d_104/Conv2D` | `FLOAT16` | `[32, 1, 1, 64]` | 181 | 2573880 | 4096 |
| 181 | `model_5/model_4/model_3/depthwise_conv2d_71/depthwise` | `FLOAT16` | `[1, 3, 3, 32]` | 182 | 2573292 | 576 |
| 182 | `model_5/model_4/model_3/conv2d_105/Conv2D` | `FLOAT16` | `[64, 1, 1, 32]` | 183 | 2569184 | 4096 |
| 183 | `model_5/model_4/model_3/conv2d_106/Conv2D` | `FLOAT16` | `[32, 1, 1, 64]` | 184 | 2565076 | 4096 |
| 184 | `model_5/model_4/model_3/depthwise_conv2d_72/depthwise` | `FLOAT16` | `[1, 3, 3, 32]` | 185 | 2564488 | 576 |
| 185 | `model_5/model_4/model_3/conv2d_107/Conv2D` | `FLOAT16` | `[64, 1, 1, 32]` | 186 | 2560380 | 4096 |
| 186 | `model_5/model_4/model_3/conv2d_108/Conv2D` | `FLOAT16` | `[32, 1, 1, 64]` | 187 | 2556272 | 4096 |
| 187 | `model_5/model_4/model_3/depthwise_conv2d_73/depthwise1` | `FLOAT16` | `[1, 3, 3, 32]` | 188 | 2555684 | 576 |
| 188 | `model_5/model_4/model_3/conv2d_109/Conv2D` | `FLOAT16` | `[64, 1, 1, 32]` | 189 | 2551576 | 4096 |
| 189 | `model_5/model_4/model_3/conv2d_110/Conv2D` | `FLOAT16` | `[64, 2, 2, 64]` | 190 | 2518796 | 32768 |
| 190 | `model_5/model_4/model_3/depthwise_conv2d_74/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 191 | 2517632 | 1152 |
| 191 | `model_5/model_4/model_3/conv2d_111/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 192 | 2501236 | 16384 |
| 192 | `model_5/model_4/model_3/conv2d_112/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 193 | 2484840 | 16384 |
| 193 | `model_5/model_4/model_3/depthwise_conv2d_75/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 194 | 2483676 | 1152 |
| 194 | `model_5/model_4/model_3/conv2d_113/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 195 | 2467280 | 16384 |
| 195 | `model_5/model_4/model_3/conv2d_114/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 196 | 2450884 | 16384 |
| 196 | `model_5/model_4/model_3/depthwise_conv2d_76/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 197 | 2449720 | 1152 |
| 197 | `model_5/model_4/model_3/conv2d_115/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 198 | 2433324 | 16384 |
| 198 | `model_5/model_4/model_3/conv2d_116/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 199 | 2416928 | 16384 |
| 199 | `model_5/model_4/model_3/depthwise_conv2d_77/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 200 | 2415764 | 1152 |
| 200 | `model_5/model_4/model_3/conv2d_117/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 201 | 2399368 | 16384 |
| 201 | `model_5/model_4/model_3/conv2d_118/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 202 | 2382972 | 16384 |
| 202 | `model_5/model_4/model_3/depthwise_conv2d_78/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 203 | 2381808 | 1152 |
| 203 | `model_5/model_4/model_3/conv2d_119/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 204 | 2365412 | 16384 |
| 204 | `model_5/model_4/model_3/conv2d_120/Conv2D` | `FLOAT16` | `[64, 2, 2, 128]` | 205 | 2299864 | 65536 |
| 205 | `model_5/model_4/model_3/depthwise_conv2d_79/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 206 | 2298700 | 1152 |
| 206 | `model_5/model_4/model_3/conv2d_121/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 207 | 2282304 | 16384 |
| 207 | `model_5/model_4/model_3/conv2d_122/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 208 | 2265908 | 16384 |
| 208 | `model_5/model_4/model_3/depthwise_conv2d_80/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 209 | 2264744 | 1152 |
| 209 | `model_5/model_4/model_3/conv2d_123/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 210 | 2248348 | 16384 |
| 210 | `model_5/model_4/model_3/conv2d_124/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 211 | 2231952 | 16384 |
| 211 | `model_5/model_4/model_3/depthwise_conv2d_81/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 212 | 2230788 | 1152 |
| 212 | `model_5/model_4/model_3/conv2d_125/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 213 | 2214392 | 16384 |
| 213 | `model_5/model_4/model_3/conv2d_126/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 214 | 2197996 | 16384 |
| 214 | `model_5/model_4/model_3/depthwise_conv2d_82/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 215 | 2196832 | 1152 |
| 215 | `model_5/model_4/model_3/conv2d_127/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 216 | 2180436 | 16384 |
| 216 | `model_5/model_4/model_3/conv2d_128/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 217 | 2164040 | 16384 |
| 217 | `model_5/model_4/model_3/depthwise_conv2d_83/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 218 | 2162876 | 1152 |
| 218 | `model_5/model_4/model_3/conv2d_129/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 219 | 2146480 | 16384 |
| 219 | `model_5/model_4/model_3/conv2d_130/Conv2D` | `FLOAT16` | `[64, 2, 2, 128]` | 220 | 2080932 | 65536 |
| 220 | `model_5/model_4/model_3/depthwise_conv2d_84/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 221 | 2079768 | 1152 |
| 221 | `model_5/model_4/model_3/conv2d_131/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 222 | 2063372 | 16384 |
| 222 | `model_5/model_4/model_3/conv2d_132/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 223 | 2046976 | 16384 |
| 223 | `model_5/model_4/model_3/depthwise_conv2d_85/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 224 | 2045812 | 1152 |
| 224 | `model_5/model_4/model_3/conv2d_133/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 225 | 2029416 | 16384 |
| 225 | `model_5/model_4/model_3/conv2d_134/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 226 | 2013020 | 16384 |
| 226 | `model_5/model_4/model_3/depthwise_conv2d_86/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 227 | 2011856 | 1152 |
| 227 | `model_5/model_4/model_3/conv2d_135/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 228 | 1995460 | 16384 |
| 228 | `model_5/model_4/model_3/conv2d_136/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 229 | 1979064 | 16384 |
| 229 | `model_5/model_4/model_3/depthwise_conv2d_87/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 230 | 1977900 | 1152 |
| 230 | `model_5/model_4/model_3/conv2d_137/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 231 | 1961504 | 16384 |
| 231 | `model_5/model_4/model_3/conv2d_138/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 232 | 1945108 | 16384 |
| 232 | `model_5/model_4/model_3/depthwise_conv2d_88/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 233 | 1943944 | 1152 |
| 233 | `model_5/model_4/model_3/conv2d_139/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 234 | 1927548 | 16384 |
| 234 | `model_5/model_4/model_3/conv2d_140/Conv2D` | `FLOAT16` | `[64, 2, 2, 128]` | 235 | 1862000 | 65536 |
| 235 | `model_5/model_4/model_3/depthwise_conv2d_89/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 236 | 1860836 | 1152 |
| 236 | `model_5/model_4/model_3/conv2d_141/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 237 | 1844440 | 16384 |
| 237 | `model_5/model_4/model_3/conv2d_142/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 238 | 1828044 | 16384 |
| 238 | `model_5/model_4/model_3/depthwise_conv2d_90/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 239 | 1826880 | 1152 |
| 239 | `model_5/model_4/model_3/conv2d_143/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 240 | 1810484 | 16384 |
| 240 | `model_5/model_4/model_3/conv2d_144/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 241 | 1794088 | 16384 |
| 241 | `model_5/model_4/model_3/depthwise_conv2d_91/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 242 | 1792924 | 1152 |
| 242 | `model_5/model_4/model_3/conv2d_145/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 243 | 1776528 | 16384 |
| 243 | `model_5/model_4/model_3/conv2d_146/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 244 | 1760132 | 16384 |
| 244 | `model_5/model_4/model_3/depthwise_conv2d_92/depthwise` | `FLOAT16` | `[1, 3, 3, 64]` | 245 | 1758968 | 1152 |
| 245 | `model_5/model_4/model_3/conv2d_147/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 246 | 1742572 | 16384 |
| 246 | `model_5/model_4/model_3/conv2d_148/Conv2D` | `FLOAT16` | `[64, 1, 1, 128]` | 247 | 1726176 | 16384 |
| 247 | `model_5/model_4/model_3/depthwise_conv2d_93/depthwise1` | `FLOAT16` | `[1, 3, 3, 64]` | 248 | 1725012 | 1152 |
| 248 | `model_5/model_4/model_3/conv2d_149/Conv2D` | `FLOAT16` | `[128, 1, 1, 64]` | 249 | 1708616 | 16384 |
| 249 | `model_5/conv2d_152/Conv2D` | `FLOAT16` | `[1, 2, 2, 128]` | 250 | 1707580 | 1024 |
| 250 | `model_5/model_4/conv2d_151/Conv2D` | `FLOAT16` | `[1, 2, 2, 128]` | 251 | 1706544 | 1024 |
| 251 | `model_5/model_4/model_3/conv2d_150/Conv2D` | `FLOAT16` | `[1434, 2, 2, 128]` | 252 | 238116 | 1468416 |
| 252 | `model_5/tf.reshape/Reshape/shape` | `INT32` | `[2]` | 253 | 238096 | 8 |
| 253 | `model_5/model_4/model_3/channel_padding_2/PartitionedCall/PartitionedCall/Pad/paddings` | `INT32` | `[4, 2]` | 254 | 238052 | 32 |
| 254 | `model_5/model_4/model_3/channel_padding_1/PartitionedCall/PartitionedCall/Pad/paddings` | `INT32` | `[4, 2]` | 255 | 238008 | 32 |
| 255 | `model_5/model_4/model_3/channel_padding/PartitionedCall/PartitionedCall/Pad/paddings` | `INT32` | `[4, 2]` | 256 | 237964 | 32 |
| 256 | `model_5/model_4/model_3/batch_normalization_132/FusedBatchNormV3;model_5/model_4/model_3/conv2d_81/BiasAdd/ReadVariableOp/resource;model_5/model_4/model_3/conv2d_81/BiasAdd;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_81/Conv2D` | `FLOAT32` | `[1, 128, 128, 16]` | 257 | None | 0 |
| 257 | `model_5/model_4/model_3/p_re_lu_126/add;model_5/model_4/model_3/p_re_lu_126/Relu;model_5/model_4/model_3/p_re_lu_126/Neg_1;model_5/model_4/model_3/p_re_lu_126/Relu_1;model_5/model_4/model_3/p_re_lu_126/mul1` | `FLOAT32` | `[1, 128, 128, 16]` | 258 | None | 0 |
| 258 | `model_5/model_4/model_3/batch_normalization_133/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_63/depthwise;model_5/model_4/model_3/conv2d_82/Conv2D` | `FLOAT32` | `[1, 128, 128, 8]` | 259 | None | 0 |
| 259 | `model_5/model_4/model_3/p_re_lu_127/add;model_5/model_4/model_3/p_re_lu_127/Relu;model_5/model_4/model_3/p_re_lu_127/Neg_1;model_5/model_4/model_3/p_re_lu_127/Relu_1;model_5/model_4/model_3/p_re_lu_127/mul1` | `FLOAT32` | `[1, 128, 128, 8]` | 260 | None | 0 |
| 260 | `model_5/model_4/model_3/depthwise_conv2d_60/depthwise1` | `FLOAT32` | `[1, 128, 128, 8]` | 261 | None | 0 |
| 261 | `model_5/model_4/model_3/batch_normalization_134/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_83/Conv2D` | `FLOAT32` | `[1, 128, 128, 16]` | 262 | None | 0 |
| 262 | `model_5/model_4/model_3/add_56/add` | `FLOAT32` | `[1, 128, 128, 16]` | 263 | None | 0 |
| 263 | `model_5/model_4/model_3/p_re_lu_128/add;model_5/model_4/model_3/p_re_lu_128/Relu;model_5/model_4/model_3/p_re_lu_128/Neg_1;model_5/model_4/model_3/p_re_lu_128/Relu_1;model_5/model_4/model_3/p_re_lu_128/mul1` | `FLOAT32` | `[1, 128, 128, 16]` | 264 | None | 0 |
| 264 | `model_5/model_4/model_3/batch_normalization_135/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_63/depthwise;model_5/model_4/model_3/conv2d_84/Conv2D` | `FLOAT32` | `[1, 128, 128, 8]` | 265 | None | 0 |
| 265 | `model_5/model_4/model_3/p_re_lu_129/add;model_5/model_4/model_3/p_re_lu_129/Relu;model_5/model_4/model_3/p_re_lu_129/Neg_1;model_5/model_4/model_3/p_re_lu_129/Relu_1;model_5/model_4/model_3/p_re_lu_129/mul1` | `FLOAT32` | `[1, 128, 128, 8]` | 266 | None | 0 |
| 266 | `model_5/model_4/model_3/depthwise_conv2d_61/depthwise1` | `FLOAT32` | `[1, 128, 128, 8]` | 267 | None | 0 |
| 267 | `model_5/model_4/model_3/batch_normalization_136/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_85/Conv2D` | `FLOAT32` | `[1, 128, 128, 16]` | 268 | None | 0 |
| 268 | `model_5/model_4/model_3/add_57/add` | `FLOAT32` | `[1, 128, 128, 16]` | 269 | None | 0 |
| 269 | `model_5/model_4/model_3/p_re_lu_130/add;model_5/model_4/model_3/p_re_lu_130/Relu;model_5/model_4/model_3/p_re_lu_130/Neg_1;model_5/model_4/model_3/p_re_lu_130/Relu_1;model_5/model_4/model_3/p_re_lu_130/mul1` | `FLOAT32` | `[1, 128, 128, 16]` | 270 | None | 0 |
| 270 | `model_5/model_4/model_3/batch_normalization_137/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_63/depthwise;model_5/model_4/model_3/conv2d_86/Conv2D` | `FLOAT32` | `[1, 128, 128, 8]` | 271 | None | 0 |
| 271 | `model_5/model_4/model_3/p_re_lu_131/add;model_5/model_4/model_3/p_re_lu_131/Relu;model_5/model_4/model_3/p_re_lu_131/Neg_1;model_5/model_4/model_3/p_re_lu_131/Relu_1;model_5/model_4/model_3/p_re_lu_131/mul1` | `FLOAT32` | `[1, 128, 128, 8]` | 272 | None | 0 |
| 272 | `model_5/model_4/model_3/depthwise_conv2d_62/depthwise1` | `FLOAT32` | `[1, 128, 128, 8]` | 273 | None | 0 |
| 273 | `model_5/model_4/model_3/batch_normalization_138/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_87/Conv2D` | `FLOAT32` | `[1, 128, 128, 16]` | 274 | None | 0 |
| 274 | `model_5/model_4/model_3/add_58/add` | `FLOAT32` | `[1, 128, 128, 16]` | 275 | None | 0 |
| 275 | `model_5/model_4/model_3/p_re_lu_132/add;model_5/model_4/model_3/p_re_lu_132/Relu;model_5/model_4/model_3/p_re_lu_132/Neg_1;model_5/model_4/model_3/p_re_lu_132/Relu_1;model_5/model_4/model_3/p_re_lu_132/mul1` | `FLOAT32` | `[1, 128, 128, 16]` | 276 | None | 0 |
| 276 | `model_5/model_4/model_3/batch_normalization_139/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_63/depthwise;model_5/model_4/model_3/conv2d_88/Conv2D` | `FLOAT32` | `[1, 128, 128, 8]` | 277 | None | 0 |
| 277 | `model_5/model_4/model_3/p_re_lu_133/add;model_5/model_4/model_3/p_re_lu_133/Relu;model_5/model_4/model_3/p_re_lu_133/Neg_1;model_5/model_4/model_3/p_re_lu_133/Relu_1;model_5/model_4/model_3/p_re_lu_133/mul1` | `FLOAT32` | `[1, 128, 128, 8]` | 278 | None | 0 |
| 278 | `model_5/model_4/model_3/depthwise_conv2d_63/depthwise2` | `FLOAT32` | `[1, 128, 128, 8]` | 279 | None | 0 |
| 279 | `model_5/model_4/model_3/batch_normalization_140/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_89/Conv2D` | `FLOAT32` | `[1, 128, 128, 16]` | 280 | None | 0 |
| 280 | `model_5/model_4/model_3/add_59/add` | `FLOAT32` | `[1, 128, 128, 16]` | 281 | None | 0 |
| 281 | `model_5/model_4/model_3/p_re_lu_134/add;model_5/model_4/model_3/p_re_lu_134/Relu;model_5/model_4/model_3/p_re_lu_134/Neg_1;model_5/model_4/model_3/p_re_lu_134/Relu_1;model_5/model_4/model_3/p_re_lu_134/mul1` | `FLOAT32` | `[1, 128, 128, 16]` | 282 | None | 0 |
| 282 | `model_5/model_4/model_3/batch_normalization_141/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_90/Conv2D` | `FLOAT32` | `[1, 64, 64, 16]` | 283 | None | 0 |
| 283 | `model_5/model_4/model_3/max_pooling2d_21/MaxPool` | `FLOAT32` | `[1, 64, 64, 16]` | 284 | None | 0 |
| 284 | `model_5/model_4/model_3/channel_padding/PartitionedCall/PartitionedCall/Pad` | `FLOAT32` | `[1, 64, 64, 32]` | 285 | None | 0 |
| 285 | `model_5/model_4/model_3/p_re_lu_135/add;model_5/model_4/model_3/p_re_lu_135/Relu;model_5/model_4/model_3/p_re_lu_135/Neg_1;model_5/model_4/model_3/p_re_lu_135/Relu_1;model_5/model_4/model_3/p_re_lu_135/mul1` | `FLOAT32` | `[1, 64, 64, 16]` | 286 | None | 0 |
| 286 | `model_5/model_4/model_3/depthwise_conv2d_64/depthwise1` | `FLOAT32` | `[1, 64, 64, 16]` | 287 | None | 0 |
| 287 | `model_5/model_4/model_3/batch_normalization_142/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_91/Conv2D` | `FLOAT32` | `[1, 64, 64, 32]` | 288 | None | 0 |
| 288 | `model_5/model_4/model_3/add_60/add` | `FLOAT32` | `[1, 64, 64, 32]` | 289 | None | 0 |
| 289 | `model_5/model_4/model_3/p_re_lu_136/add;model_5/model_4/model_3/p_re_lu_136/Relu;model_5/model_4/model_3/p_re_lu_136/Neg_1;model_5/model_4/model_3/p_re_lu_136/Relu_1;model_5/model_4/model_3/p_re_lu_136/mul1` | `FLOAT32` | `[1, 64, 64, 32]` | 290 | None | 0 |
| 290 | `model_5/model_4/model_3/batch_normalization_143/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_92/Conv2D` | `FLOAT32` | `[1, 64, 64, 16]` | 291 | None | 0 |
| 291 | `model_5/model_4/model_3/p_re_lu_137/add;model_5/model_4/model_3/p_re_lu_137/Relu;model_5/model_4/model_3/p_re_lu_137/Neg_1;model_5/model_4/model_3/p_re_lu_137/Relu_1;model_5/model_4/model_3/p_re_lu_137/mul1` | `FLOAT32` | `[1, 64, 64, 16]` | 292 | None | 0 |
| 292 | `model_5/model_4/model_3/depthwise_conv2d_65/depthwise1` | `FLOAT32` | `[1, 64, 64, 16]` | 293 | None | 0 |
| 293 | `model_5/model_4/model_3/batch_normalization_144/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_93/Conv2D` | `FLOAT32` | `[1, 64, 64, 32]` | 294 | None | 0 |
| 294 | `model_5/model_4/model_3/add_61/add` | `FLOAT32` | `[1, 64, 64, 32]` | 295 | None | 0 |
| 295 | `model_5/model_4/model_3/p_re_lu_138/add;model_5/model_4/model_3/p_re_lu_138/Relu;model_5/model_4/model_3/p_re_lu_138/Neg_1;model_5/model_4/model_3/p_re_lu_138/Relu_1;model_5/model_4/model_3/p_re_lu_138/mul1` | `FLOAT32` | `[1, 64, 64, 32]` | 296 | None | 0 |
| 296 | `model_5/model_4/model_3/batch_normalization_145/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_94/Conv2D` | `FLOAT32` | `[1, 64, 64, 16]` | 297 | None | 0 |
| 297 | `model_5/model_4/model_3/p_re_lu_139/add;model_5/model_4/model_3/p_re_lu_139/Relu;model_5/model_4/model_3/p_re_lu_139/Neg_1;model_5/model_4/model_3/p_re_lu_139/Relu_1;model_5/model_4/model_3/p_re_lu_139/mul1` | `FLOAT32` | `[1, 64, 64, 16]` | 298 | None | 0 |
| 298 | `model_5/model_4/model_3/depthwise_conv2d_66/depthwise1` | `FLOAT32` | `[1, 64, 64, 16]` | 299 | None | 0 |
| 299 | `model_5/model_4/model_3/batch_normalization_146/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_95/Conv2D` | `FLOAT32` | `[1, 64, 64, 32]` | 300 | None | 0 |
| 300 | `model_5/model_4/model_3/add_62/add` | `FLOAT32` | `[1, 64, 64, 32]` | 301 | None | 0 |
| 301 | `model_5/model_4/model_3/p_re_lu_140/add;model_5/model_4/model_3/p_re_lu_140/Relu;model_5/model_4/model_3/p_re_lu_140/Neg_1;model_5/model_4/model_3/p_re_lu_140/Relu_1;model_5/model_4/model_3/p_re_lu_140/mul1` | `FLOAT32` | `[1, 64, 64, 32]` | 302 | None | 0 |
| 302 | `model_5/model_4/model_3/batch_normalization_147/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_96/Conv2D` | `FLOAT32` | `[1, 64, 64, 16]` | 303 | None | 0 |
| 303 | `model_5/model_4/model_3/p_re_lu_141/add;model_5/model_4/model_3/p_re_lu_141/Relu;model_5/model_4/model_3/p_re_lu_141/Neg_1;model_5/model_4/model_3/p_re_lu_141/Relu_1;model_5/model_4/model_3/p_re_lu_141/mul1` | `FLOAT32` | `[1, 64, 64, 16]` | 304 | None | 0 |
| 304 | `model_5/model_4/model_3/depthwise_conv2d_67/depthwise1` | `FLOAT32` | `[1, 64, 64, 16]` | 305 | None | 0 |
| 305 | `model_5/model_4/model_3/batch_normalization_148/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_97/Conv2D` | `FLOAT32` | `[1, 64, 64, 32]` | 306 | None | 0 |
| 306 | `model_5/model_4/model_3/add_63/add` | `FLOAT32` | `[1, 64, 64, 32]` | 307 | None | 0 |
| 307 | `model_5/model_4/model_3/p_re_lu_142/add;model_5/model_4/model_3/p_re_lu_142/Relu;model_5/model_4/model_3/p_re_lu_142/Neg_1;model_5/model_4/model_3/p_re_lu_142/Relu_1;model_5/model_4/model_3/p_re_lu_142/mul1` | `FLOAT32` | `[1, 64, 64, 32]` | 308 | None | 0 |
| 308 | `model_5/model_4/model_3/batch_normalization_149/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_68/depthwise;model_5/model_4/model_3/conv2d_98/Conv2D` | `FLOAT32` | `[1, 64, 64, 16]` | 309 | None | 0 |
| 309 | `model_5/model_4/model_3/p_re_lu_143/add;model_5/model_4/model_3/p_re_lu_143/Relu;model_5/model_4/model_3/p_re_lu_143/Neg_1;model_5/model_4/model_3/p_re_lu_143/Relu_1;model_5/model_4/model_3/p_re_lu_143/mul1` | `FLOAT32` | `[1, 64, 64, 16]` | 310 | None | 0 |
| 310 | `model_5/model_4/model_3/depthwise_conv2d_68/depthwise2` | `FLOAT32` | `[1, 64, 64, 16]` | 311 | None | 0 |
| 311 | `model_5/model_4/model_3/batch_normalization_150/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_99/Conv2D` | `FLOAT32` | `[1, 64, 64, 32]` | 312 | None | 0 |
| 312 | `model_5/model_4/model_3/add_64/add` | `FLOAT32` | `[1, 64, 64, 32]` | 313 | None | 0 |
| 313 | `model_5/model_4/model_3/p_re_lu_144/add;model_5/model_4/model_3/p_re_lu_144/Relu;model_5/model_4/model_3/p_re_lu_144/Neg_1;model_5/model_4/model_3/p_re_lu_144/Relu_1;model_5/model_4/model_3/p_re_lu_144/mul1` | `FLOAT32` | `[1, 64, 64, 32]` | 314 | None | 0 |
| 314 | `model_5/model_4/model_3/batch_normalization_151/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_100/Conv2D` | `FLOAT32` | `[1, 32, 32, 32]` | 315 | None | 0 |
| 315 | `model_5/model_4/model_3/max_pooling2d_22/MaxPool` | `FLOAT32` | `[1, 32, 32, 32]` | 316 | None | 0 |
| 316 | `model_5/model_4/model_3/channel_padding_1/PartitionedCall/PartitionedCall/Pad` | `FLOAT32` | `[1, 32, 32, 64]` | 317 | None | 0 |
| 317 | `model_5/model_4/model_3/p_re_lu_145/add;model_5/model_4/model_3/p_re_lu_145/Relu;model_5/model_4/model_3/p_re_lu_145/Neg_1;model_5/model_4/model_3/p_re_lu_145/Relu_1;model_5/model_4/model_3/p_re_lu_145/mul1` | `FLOAT32` | `[1, 32, 32, 32]` | 318 | None | 0 |
| 318 | `model_5/model_4/model_3/depthwise_conv2d_69/depthwise1` | `FLOAT32` | `[1, 32, 32, 32]` | 319 | None | 0 |
| 319 | `model_5/model_4/model_3/batch_normalization_152/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_101/Conv2D` | `FLOAT32` | `[1, 32, 32, 64]` | 320 | None | 0 |
| 320 | `model_5/model_4/model_3/add_65/add` | `FLOAT32` | `[1, 32, 32, 64]` | 321 | None | 0 |
| 321 | `model_5/model_4/model_3/p_re_lu_146/add;model_5/model_4/model_3/p_re_lu_146/Relu;model_5/model_4/model_3/p_re_lu_146/Neg_1;model_5/model_4/model_3/p_re_lu_146/Relu_1;model_5/model_4/model_3/p_re_lu_146/mul1` | `FLOAT32` | `[1, 32, 32, 64]` | 322 | None | 0 |
| 322 | `model_5/model_4/model_3/batch_normalization_153/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_102/Conv2D` | `FLOAT32` | `[1, 32, 32, 32]` | 323 | None | 0 |
| 323 | `model_5/model_4/model_3/p_re_lu_147/add;model_5/model_4/model_3/p_re_lu_147/Relu;model_5/model_4/model_3/p_re_lu_147/Neg_1;model_5/model_4/model_3/p_re_lu_147/Relu_1;model_5/model_4/model_3/p_re_lu_147/mul1` | `FLOAT32` | `[1, 32, 32, 32]` | 324 | None | 0 |
| 324 | `model_5/model_4/model_3/depthwise_conv2d_70/depthwise1` | `FLOAT32` | `[1, 32, 32, 32]` | 325 | None | 0 |
| 325 | `model_5/model_4/model_3/batch_normalization_154/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_103/Conv2D` | `FLOAT32` | `[1, 32, 32, 64]` | 326 | None | 0 |
| 326 | `model_5/model_4/model_3/add_66/add` | `FLOAT32` | `[1, 32, 32, 64]` | 327 | None | 0 |
| 327 | `model_5/model_4/model_3/p_re_lu_148/add;model_5/model_4/model_3/p_re_lu_148/Relu;model_5/model_4/model_3/p_re_lu_148/Neg_1;model_5/model_4/model_3/p_re_lu_148/Relu_1;model_5/model_4/model_3/p_re_lu_148/mul1` | `FLOAT32` | `[1, 32, 32, 64]` | 328 | None | 0 |
| 328 | `model_5/model_4/model_3/batch_normalization_155/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_104/Conv2D` | `FLOAT32` | `[1, 32, 32, 32]` | 329 | None | 0 |
| 329 | `model_5/model_4/model_3/p_re_lu_149/add;model_5/model_4/model_3/p_re_lu_149/Relu;model_5/model_4/model_3/p_re_lu_149/Neg_1;model_5/model_4/model_3/p_re_lu_149/Relu_1;model_5/model_4/model_3/p_re_lu_149/mul1` | `FLOAT32` | `[1, 32, 32, 32]` | 330 | None | 0 |
| 330 | `model_5/model_4/model_3/depthwise_conv2d_71/depthwise1` | `FLOAT32` | `[1, 32, 32, 32]` | 331 | None | 0 |
| 331 | `model_5/model_4/model_3/batch_normalization_156/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_105/Conv2D` | `FLOAT32` | `[1, 32, 32, 64]` | 332 | None | 0 |
| 332 | `model_5/model_4/model_3/add_67/add` | `FLOAT32` | `[1, 32, 32, 64]` | 333 | None | 0 |
| 333 | `model_5/model_4/model_3/p_re_lu_150/add;model_5/model_4/model_3/p_re_lu_150/Relu;model_5/model_4/model_3/p_re_lu_150/Neg_1;model_5/model_4/model_3/p_re_lu_150/Relu_1;model_5/model_4/model_3/p_re_lu_150/mul1` | `FLOAT32` | `[1, 32, 32, 64]` | 334 | None | 0 |
| 334 | `model_5/model_4/model_3/batch_normalization_157/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_106/Conv2D` | `FLOAT32` | `[1, 32, 32, 32]` | 335 | None | 0 |
| 335 | `model_5/model_4/model_3/p_re_lu_151/add;model_5/model_4/model_3/p_re_lu_151/Relu;model_5/model_4/model_3/p_re_lu_151/Neg_1;model_5/model_4/model_3/p_re_lu_151/Relu_1;model_5/model_4/model_3/p_re_lu_151/mul1` | `FLOAT32` | `[1, 32, 32, 32]` | 336 | None | 0 |
| 336 | `model_5/model_4/model_3/depthwise_conv2d_72/depthwise1` | `FLOAT32` | `[1, 32, 32, 32]` | 337 | None | 0 |
| 337 | `model_5/model_4/model_3/batch_normalization_158/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_107/Conv2D` | `FLOAT32` | `[1, 32, 32, 64]` | 338 | None | 0 |
| 338 | `model_5/model_4/model_3/add_68/add` | `FLOAT32` | `[1, 32, 32, 64]` | 339 | None | 0 |
| 339 | `model_5/model_4/model_3/p_re_lu_152/add;model_5/model_4/model_3/p_re_lu_152/Relu;model_5/model_4/model_3/p_re_lu_152/Neg_1;model_5/model_4/model_3/p_re_lu_152/Relu_1;model_5/model_4/model_3/p_re_lu_152/mul1` | `FLOAT32` | `[1, 32, 32, 64]` | 340 | None | 0 |
| 340 | `model_5/model_4/model_3/batch_normalization_159/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_73/depthwise;model_5/model_4/model_3/conv2d_108/Conv2D` | `FLOAT32` | `[1, 32, 32, 32]` | 341 | None | 0 |
| 341 | `model_5/model_4/model_3/p_re_lu_153/add;model_5/model_4/model_3/p_re_lu_153/Relu;model_5/model_4/model_3/p_re_lu_153/Neg_1;model_5/model_4/model_3/p_re_lu_153/Relu_1;model_5/model_4/model_3/p_re_lu_153/mul1` | `FLOAT32` | `[1, 32, 32, 32]` | 342 | None | 0 |
| 342 | `model_5/model_4/model_3/depthwise_conv2d_73/depthwise2` | `FLOAT32` | `[1, 32, 32, 32]` | 343 | None | 0 |
| 343 | `model_5/model_4/model_3/batch_normalization_160/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_109/Conv2D` | `FLOAT32` | `[1, 32, 32, 64]` | 344 | None | 0 |
| 344 | `model_5/model_4/model_3/add_69/add` | `FLOAT32` | `[1, 32, 32, 64]` | 345 | None | 0 |
| 345 | `model_5/model_4/model_3/p_re_lu_154/add;model_5/model_4/model_3/p_re_lu_154/Relu;model_5/model_4/model_3/p_re_lu_154/Neg_1;model_5/model_4/model_3/p_re_lu_154/Relu_1;model_5/model_4/model_3/p_re_lu_154/mul1` | `FLOAT32` | `[1, 32, 32, 64]` | 346 | None | 0 |
| 346 | `model_5/model_4/model_3/batch_normalization_161/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_110/Conv2D` | `FLOAT32` | `[1, 16, 16, 64]` | 347 | None | 0 |
| 347 | `model_5/model_4/model_3/max_pooling2d_23/MaxPool` | `FLOAT32` | `[1, 16, 16, 64]` | 348 | None | 0 |
| 348 | `model_5/model_4/model_3/channel_padding_2/PartitionedCall/PartitionedCall/Pad` | `FLOAT32` | `[1, 16, 16, 128]` | 349 | None | 0 |
| 349 | `model_5/model_4/model_3/p_re_lu_155/add;model_5/model_4/model_3/p_re_lu_155/Relu;model_5/model_4/model_3/p_re_lu_155/Neg_1;model_5/model_4/model_3/p_re_lu_155/Relu_1;model_5/model_4/model_3/p_re_lu_155/mul1` | `FLOAT32` | `[1, 16, 16, 64]` | 350 | None | 0 |
| 350 | `model_5/model_4/model_3/depthwise_conv2d_74/depthwise1` | `FLOAT32` | `[1, 16, 16, 64]` | 351 | None | 0 |
| 351 | `model_5/model_4/model_3/batch_normalization_162/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_111/Conv2D` | `FLOAT32` | `[1, 16, 16, 128]` | 352 | None | 0 |
| 352 | `model_5/model_4/model_3/add_70/add` | `FLOAT32` | `[1, 16, 16, 128]` | 353 | None | 0 |
| 353 | `model_5/model_4/model_3/p_re_lu_156/add;model_5/model_4/model_3/p_re_lu_156/Relu;model_5/model_4/model_3/p_re_lu_156/Neg_1;model_5/model_4/model_3/p_re_lu_156/Relu_1;model_5/model_4/model_3/p_re_lu_156/mul1` | `FLOAT32` | `[1, 16, 16, 128]` | 354 | None | 0 |
| 354 | `model_5/model_4/model_3/batch_normalization_163/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_112/Conv2D` | `FLOAT32` | `[1, 16, 16, 64]` | 355 | None | 0 |
| 355 | `model_5/model_4/model_3/p_re_lu_157/add;model_5/model_4/model_3/p_re_lu_157/Relu;model_5/model_4/model_3/p_re_lu_157/Neg_1;model_5/model_4/model_3/p_re_lu_157/Relu_1;model_5/model_4/model_3/p_re_lu_157/mul1` | `FLOAT32` | `[1, 16, 16, 64]` | 356 | None | 0 |
| 356 | `model_5/model_4/model_3/depthwise_conv2d_75/depthwise1` | `FLOAT32` | `[1, 16, 16, 64]` | 357 | None | 0 |
| 357 | `model_5/model_4/model_3/batch_normalization_164/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_113/Conv2D` | `FLOAT32` | `[1, 16, 16, 128]` | 358 | None | 0 |
| 358 | `model_5/model_4/model_3/add_71/add` | `FLOAT32` | `[1, 16, 16, 128]` | 359 | None | 0 |
| 359 | `model_5/model_4/model_3/p_re_lu_158/add;model_5/model_4/model_3/p_re_lu_158/Relu;model_5/model_4/model_3/p_re_lu_158/Neg_1;model_5/model_4/model_3/p_re_lu_158/Relu_1;model_5/model_4/model_3/p_re_lu_158/mul1` | `FLOAT32` | `[1, 16, 16, 128]` | 360 | None | 0 |
| 360 | `model_5/model_4/model_3/batch_normalization_165/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_114/Conv2D` | `FLOAT32` | `[1, 16, 16, 64]` | 361 | None | 0 |
| 361 | `model_5/model_4/model_3/p_re_lu_159/add;model_5/model_4/model_3/p_re_lu_159/Relu;model_5/model_4/model_3/p_re_lu_159/Neg_1;model_5/model_4/model_3/p_re_lu_159/Relu_1;model_5/model_4/model_3/p_re_lu_159/mul1` | `FLOAT32` | `[1, 16, 16, 64]` | 362 | None | 0 |
| 362 | `model_5/model_4/model_3/depthwise_conv2d_76/depthwise1` | `FLOAT32` | `[1, 16, 16, 64]` | 363 | None | 0 |
| 363 | `model_5/model_4/model_3/batch_normalization_166/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_115/Conv2D` | `FLOAT32` | `[1, 16, 16, 128]` | 364 | None | 0 |
| 364 | `model_5/model_4/model_3/add_72/add` | `FLOAT32` | `[1, 16, 16, 128]` | 365 | None | 0 |
| 365 | `model_5/model_4/model_3/p_re_lu_160/add;model_5/model_4/model_3/p_re_lu_160/Relu;model_5/model_4/model_3/p_re_lu_160/Neg_1;model_5/model_4/model_3/p_re_lu_160/Relu_1;model_5/model_4/model_3/p_re_lu_160/mul1` | `FLOAT32` | `[1, 16, 16, 128]` | 366 | None | 0 |
| 366 | `model_5/model_4/model_3/batch_normalization_167/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_116/Conv2D` | `FLOAT32` | `[1, 16, 16, 64]` | 367 | None | 0 |
| 367 | `model_5/model_4/model_3/p_re_lu_161/add;model_5/model_4/model_3/p_re_lu_161/Relu;model_5/model_4/model_3/p_re_lu_161/Neg_1;model_5/model_4/model_3/p_re_lu_161/Relu_1;model_5/model_4/model_3/p_re_lu_161/mul1` | `FLOAT32` | `[1, 16, 16, 64]` | 368 | None | 0 |
| 368 | `model_5/model_4/model_3/depthwise_conv2d_77/depthwise1` | `FLOAT32` | `[1, 16, 16, 64]` | 369 | None | 0 |
| 369 | `model_5/model_4/model_3/batch_normalization_168/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_117/Conv2D` | `FLOAT32` | `[1, 16, 16, 128]` | 370 | None | 0 |
| 370 | `model_5/model_4/model_3/add_73/add` | `FLOAT32` | `[1, 16, 16, 128]` | 371 | None | 0 |
| 371 | `model_5/model_4/model_3/p_re_lu_162/add;model_5/model_4/model_3/p_re_lu_162/Relu;model_5/model_4/model_3/p_re_lu_162/Neg_1;model_5/model_4/model_3/p_re_lu_162/Relu_1;model_5/model_4/model_3/p_re_lu_162/mul1` | `FLOAT32` | `[1, 16, 16, 128]` | 372 | None | 0 |
| 372 | `model_5/model_4/model_3/batch_normalization_169/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_118/Conv2D` | `FLOAT32` | `[1, 16, 16, 64]` | 373 | None | 0 |
| 373 | `model_5/model_4/model_3/p_re_lu_163/add;model_5/model_4/model_3/p_re_lu_163/Relu;model_5/model_4/model_3/p_re_lu_163/Neg_1;model_5/model_4/model_3/p_re_lu_163/Relu_1;model_5/model_4/model_3/p_re_lu_163/mul1` | `FLOAT32` | `[1, 16, 16, 64]` | 374 | None | 0 |
| 374 | `model_5/model_4/model_3/depthwise_conv2d_78/depthwise1` | `FLOAT32` | `[1, 16, 16, 64]` | 375 | None | 0 |
| 375 | `model_5/model_4/model_3/batch_normalization_170/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_119/Conv2D` | `FLOAT32` | `[1, 16, 16, 128]` | 376 | None | 0 |
| 376 | `model_5/model_4/model_3/add_74/add` | `FLOAT32` | `[1, 16, 16, 128]` | 377 | None | 0 |
| 377 | `model_5/model_4/model_3/p_re_lu_164/add;model_5/model_4/model_3/p_re_lu_164/Relu;model_5/model_4/model_3/p_re_lu_164/Neg_1;model_5/model_4/model_3/p_re_lu_164/Relu_1;model_5/model_4/model_3/p_re_lu_164/mul1` | `FLOAT32` | `[1, 16, 16, 128]` | 378 | None | 0 |
| 378 | `model_5/model_4/model_3/batch_normalization_171/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_120/Conv2D` | `FLOAT32` | `[1, 8, 8, 64]` | 379 | None | 0 |
| 379 | `model_5/model_4/model_3/max_pooling2d_24/MaxPool` | `FLOAT32` | `[1, 8, 8, 128]` | 380 | None | 0 |
| 380 | `model_5/model_4/model_3/p_re_lu_165/add;model_5/model_4/model_3/p_re_lu_165/Relu;model_5/model_4/model_3/p_re_lu_165/Neg_1;model_5/model_4/model_3/p_re_lu_165/Relu_1;model_5/model_4/model_3/p_re_lu_165/mul1` | `FLOAT32` | `[1, 8, 8, 64]` | 381 | None | 0 |
| 381 | `model_5/model_4/model_3/depthwise_conv2d_79/depthwise1` | `FLOAT32` | `[1, 8, 8, 64]` | 382 | None | 0 |
| 382 | `model_5/model_4/model_3/batch_normalization_172/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_121/Conv2D` | `FLOAT32` | `[1, 8, 8, 128]` | 383 | None | 0 |
| 383 | `model_5/model_4/model_3/add_75/add` | `FLOAT32` | `[1, 8, 8, 128]` | 384 | None | 0 |
| 384 | `model_5/model_4/model_3/p_re_lu_166/add;model_5/model_4/model_3/p_re_lu_166/Relu;model_5/model_4/model_3/p_re_lu_166/Neg_1;model_5/model_4/model_3/p_re_lu_166/Relu_1;model_5/model_4/model_3/p_re_lu_166/mul1` | `FLOAT32` | `[1, 8, 8, 128]` | 385 | None | 0 |
| 385 | `model_5/model_4/model_3/batch_normalization_173/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_122/Conv2D` | `FLOAT32` | `[1, 8, 8, 64]` | 386 | None | 0 |
| 386 | `model_5/model_4/model_3/p_re_lu_167/add;model_5/model_4/model_3/p_re_lu_167/Relu;model_5/model_4/model_3/p_re_lu_167/Neg_1;model_5/model_4/model_3/p_re_lu_167/Relu_1;model_5/model_4/model_3/p_re_lu_167/mul1` | `FLOAT32` | `[1, 8, 8, 64]` | 387 | None | 0 |
| 387 | `model_5/model_4/model_3/depthwise_conv2d_80/depthwise1` | `FLOAT32` | `[1, 8, 8, 64]` | 388 | None | 0 |
| 388 | `model_5/model_4/model_3/batch_normalization_174/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_123/Conv2D` | `FLOAT32` | `[1, 8, 8, 128]` | 389 | None | 0 |
| 389 | `model_5/model_4/model_3/add_76/add` | `FLOAT32` | `[1, 8, 8, 128]` | 390 | None | 0 |
| 390 | `model_5/model_4/model_3/p_re_lu_168/add;model_5/model_4/model_3/p_re_lu_168/Relu;model_5/model_4/model_3/p_re_lu_168/Neg_1;model_5/model_4/model_3/p_re_lu_168/Relu_1;model_5/model_4/model_3/p_re_lu_168/mul1` | `FLOAT32` | `[1, 8, 8, 128]` | 391 | None | 0 |
| 391 | `model_5/model_4/model_3/batch_normalization_175/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_124/Conv2D` | `FLOAT32` | `[1, 8, 8, 64]` | 392 | None | 0 |
| 392 | `model_5/model_4/model_3/p_re_lu_169/add;model_5/model_4/model_3/p_re_lu_169/Relu;model_5/model_4/model_3/p_re_lu_169/Neg_1;model_5/model_4/model_3/p_re_lu_169/Relu_1;model_5/model_4/model_3/p_re_lu_169/mul1` | `FLOAT32` | `[1, 8, 8, 64]` | 393 | None | 0 |
| 393 | `model_5/model_4/model_3/depthwise_conv2d_81/depthwise1` | `FLOAT32` | `[1, 8, 8, 64]` | 394 | None | 0 |
| 394 | `model_5/model_4/model_3/batch_normalization_176/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_125/Conv2D` | `FLOAT32` | `[1, 8, 8, 128]` | 395 | None | 0 |
| 395 | `model_5/model_4/model_3/add_77/add` | `FLOAT32` | `[1, 8, 8, 128]` | 396 | None | 0 |
| 396 | `model_5/model_4/model_3/p_re_lu_170/add;model_5/model_4/model_3/p_re_lu_170/Relu;model_5/model_4/model_3/p_re_lu_170/Neg_1;model_5/model_4/model_3/p_re_lu_170/Relu_1;model_5/model_4/model_3/p_re_lu_170/mul1` | `FLOAT32` | `[1, 8, 8, 128]` | 397 | None | 0 |
| 397 | `model_5/model_4/model_3/batch_normalization_177/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_126/Conv2D` | `FLOAT32` | `[1, 8, 8, 64]` | 398 | None | 0 |
| 398 | `model_5/model_4/model_3/p_re_lu_171/add;model_5/model_4/model_3/p_re_lu_171/Relu;model_5/model_4/model_3/p_re_lu_171/Neg_1;model_5/model_4/model_3/p_re_lu_171/Relu_1;model_5/model_4/model_3/p_re_lu_171/mul1` | `FLOAT32` | `[1, 8, 8, 64]` | 399 | None | 0 |
| 399 | `model_5/model_4/model_3/depthwise_conv2d_82/depthwise1` | `FLOAT32` | `[1, 8, 8, 64]` | 400 | None | 0 |
| 400 | `model_5/model_4/model_3/batch_normalization_178/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_127/Conv2D` | `FLOAT32` | `[1, 8, 8, 128]` | 401 | None | 0 |
| 401 | `model_5/model_4/model_3/add_78/add` | `FLOAT32` | `[1, 8, 8, 128]` | 402 | None | 0 |
| 402 | `model_5/model_4/model_3/p_re_lu_172/add;model_5/model_4/model_3/p_re_lu_172/Relu;model_5/model_4/model_3/p_re_lu_172/Neg_1;model_5/model_4/model_3/p_re_lu_172/Relu_1;model_5/model_4/model_3/p_re_lu_172/mul1` | `FLOAT32` | `[1, 8, 8, 128]` | 403 | None | 0 |
| 403 | `model_5/model_4/model_3/batch_normalization_179/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_128/Conv2D` | `FLOAT32` | `[1, 8, 8, 64]` | 404 | None | 0 |
| 404 | `model_5/model_4/model_3/p_re_lu_173/add;model_5/model_4/model_3/p_re_lu_173/Relu;model_5/model_4/model_3/p_re_lu_173/Neg_1;model_5/model_4/model_3/p_re_lu_173/Relu_1;model_5/model_4/model_3/p_re_lu_173/mul1` | `FLOAT32` | `[1, 8, 8, 64]` | 405 | None | 0 |
| 405 | `model_5/model_4/model_3/depthwise_conv2d_83/depthwise1` | `FLOAT32` | `[1, 8, 8, 64]` | 406 | None | 0 |
| 406 | `model_5/model_4/model_3/batch_normalization_180/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_129/Conv2D` | `FLOAT32` | `[1, 8, 8, 128]` | 407 | None | 0 |
| 407 | `model_5/model_4/model_3/add_79/add` | `FLOAT32` | `[1, 8, 8, 128]` | 408 | None | 0 |
| 408 | `model_5/model_4/model_3/p_re_lu_174/add;model_5/model_4/model_3/p_re_lu_174/Relu;model_5/model_4/model_3/p_re_lu_174/Neg_1;model_5/model_4/model_3/p_re_lu_174/Relu_1;model_5/model_4/model_3/p_re_lu_174/mul1` | `FLOAT32` | `[1, 8, 8, 128]` | 409 | None | 0 |
| 409 | `model_5/model_4/model_3/batch_normalization_181/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_130/Conv2D` | `FLOAT32` | `[1, 4, 4, 64]` | 410 | None | 0 |
| 410 | `model_5/model_4/model_3/max_pooling2d_25/MaxPool` | `FLOAT32` | `[1, 4, 4, 128]` | 411 | None | 0 |
| 411 | `model_5/model_4/model_3/p_re_lu_175/add;model_5/model_4/model_3/p_re_lu_175/Relu;model_5/model_4/model_3/p_re_lu_175/Neg_1;model_5/model_4/model_3/p_re_lu_175/Relu_1;model_5/model_4/model_3/p_re_lu_175/mul1` | `FLOAT32` | `[1, 4, 4, 64]` | 412 | None | 0 |
| 412 | `model_5/model_4/model_3/depthwise_conv2d_84/depthwise1` | `FLOAT32` | `[1, 4, 4, 64]` | 413 | None | 0 |
| 413 | `model_5/model_4/model_3/batch_normalization_182/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_131/Conv2D` | `FLOAT32` | `[1, 4, 4, 128]` | 414 | None | 0 |
| 414 | `model_5/model_4/model_3/add_80/add` | `FLOAT32` | `[1, 4, 4, 128]` | 415 | None | 0 |
| 415 | `model_5/model_4/model_3/p_re_lu_176/add;model_5/model_4/model_3/p_re_lu_176/Relu;model_5/model_4/model_3/p_re_lu_176/Neg_1;model_5/model_4/model_3/p_re_lu_176/Relu_1;model_5/model_4/model_3/p_re_lu_176/mul1` | `FLOAT32` | `[1, 4, 4, 128]` | 416 | None | 0 |
| 416 | `model_5/model_4/model_3/batch_normalization_183/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_132/Conv2D` | `FLOAT32` | `[1, 4, 4, 64]` | 417 | None | 0 |
| 417 | `model_5/model_4/model_3/p_re_lu_177/add;model_5/model_4/model_3/p_re_lu_177/Relu;model_5/model_4/model_3/p_re_lu_177/Neg_1;model_5/model_4/model_3/p_re_lu_177/Relu_1;model_5/model_4/model_3/p_re_lu_177/mul1` | `FLOAT32` | `[1, 4, 4, 64]` | 418 | None | 0 |
| 418 | `model_5/model_4/model_3/depthwise_conv2d_85/depthwise1` | `FLOAT32` | `[1, 4, 4, 64]` | 419 | None | 0 |
| 419 | `model_5/model_4/model_3/batch_normalization_184/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_133/Conv2D` | `FLOAT32` | `[1, 4, 4, 128]` | 420 | None | 0 |
| 420 | `model_5/model_4/model_3/add_81/add` | `FLOAT32` | `[1, 4, 4, 128]` | 421 | None | 0 |
| 421 | `model_5/model_4/model_3/p_re_lu_178/add;model_5/model_4/model_3/p_re_lu_178/Relu;model_5/model_4/model_3/p_re_lu_178/Neg_1;model_5/model_4/model_3/p_re_lu_178/Relu_1;model_5/model_4/model_3/p_re_lu_178/mul1` | `FLOAT32` | `[1, 4, 4, 128]` | 422 | None | 0 |
| 422 | `model_5/model_4/model_3/batch_normalization_185/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_134/Conv2D` | `FLOAT32` | `[1, 4, 4, 64]` | 423 | None | 0 |
| 423 | `model_5/model_4/model_3/p_re_lu_179/add;model_5/model_4/model_3/p_re_lu_179/Relu;model_5/model_4/model_3/p_re_lu_179/Neg_1;model_5/model_4/model_3/p_re_lu_179/Relu_1;model_5/model_4/model_3/p_re_lu_179/mul1` | `FLOAT32` | `[1, 4, 4, 64]` | 424 | None | 0 |
| 424 | `model_5/model_4/model_3/depthwise_conv2d_86/depthwise1` | `FLOAT32` | `[1, 4, 4, 64]` | 425 | None | 0 |
| 425 | `model_5/model_4/model_3/batch_normalization_186/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_135/Conv2D` | `FLOAT32` | `[1, 4, 4, 128]` | 426 | None | 0 |
| 426 | `model_5/model_4/model_3/add_82/add` | `FLOAT32` | `[1, 4, 4, 128]` | 427 | None | 0 |
| 427 | `model_5/model_4/model_3/p_re_lu_180/add;model_5/model_4/model_3/p_re_lu_180/Relu;model_5/model_4/model_3/p_re_lu_180/Neg_1;model_5/model_4/model_3/p_re_lu_180/Relu_1;model_5/model_4/model_3/p_re_lu_180/mul1` | `FLOAT32` | `[1, 4, 4, 128]` | 428 | None | 0 |
| 428 | `model_5/model_4/model_3/batch_normalization_187/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_136/Conv2D` | `FLOAT32` | `[1, 4, 4, 64]` | 429 | None | 0 |
| 429 | `model_5/model_4/model_3/p_re_lu_181/add;model_5/model_4/model_3/p_re_lu_181/Relu;model_5/model_4/model_3/p_re_lu_181/Neg_1;model_5/model_4/model_3/p_re_lu_181/Relu_1;model_5/model_4/model_3/p_re_lu_181/mul1` | `FLOAT32` | `[1, 4, 4, 64]` | 430 | None | 0 |
| 430 | `model_5/model_4/model_3/depthwise_conv2d_87/depthwise1` | `FLOAT32` | `[1, 4, 4, 64]` | 431 | None | 0 |
| 431 | `model_5/model_4/model_3/batch_normalization_188/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_137/Conv2D` | `FLOAT32` | `[1, 4, 4, 128]` | 432 | None | 0 |
| 432 | `model_5/model_4/model_3/add_83/add` | `FLOAT32` | `[1, 4, 4, 128]` | 433 | None | 0 |
| 433 | `model_5/model_4/model_3/p_re_lu_182/add;model_5/model_4/model_3/p_re_lu_182/Relu;model_5/model_4/model_3/p_re_lu_182/Neg_1;model_5/model_4/model_3/p_re_lu_182/Relu_1;model_5/model_4/model_3/p_re_lu_182/mul1` | `FLOAT32` | `[1, 4, 4, 128]` | 434 | None | 0 |
| 434 | `model_5/model_4/model_3/batch_normalization_189/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_138/Conv2D` | `FLOAT32` | `[1, 4, 4, 64]` | 435 | None | 0 |
| 435 | `model_5/model_4/model_3/p_re_lu_183/add;model_5/model_4/model_3/p_re_lu_183/Relu;model_5/model_4/model_3/p_re_lu_183/Neg_1;model_5/model_4/model_3/p_re_lu_183/Relu_1;model_5/model_4/model_3/p_re_lu_183/mul1` | `FLOAT32` | `[1, 4, 4, 64]` | 436 | None | 0 |
| 436 | `model_5/model_4/model_3/depthwise_conv2d_88/depthwise1` | `FLOAT32` | `[1, 4, 4, 64]` | 437 | None | 0 |
| 437 | `model_5/model_4/model_3/batch_normalization_190/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_139/Conv2D` | `FLOAT32` | `[1, 4, 4, 128]` | 438 | None | 0 |
| 438 | `model_5/model_4/model_3/add_84/add` | `FLOAT32` | `[1, 4, 4, 128]` | 439 | None | 0 |
| 439 | `model_5/model_4/model_3/p_re_lu_184/add;model_5/model_4/model_3/p_re_lu_184/Relu;model_5/model_4/model_3/p_re_lu_184/Neg_1;model_5/model_4/model_3/p_re_lu_184/Relu_1;model_5/model_4/model_3/p_re_lu_184/mul1` | `FLOAT32` | `[1, 4, 4, 128]` | 440 | None | 0 |
| 440 | `model_5/model_4/model_3/batch_normalization_191/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_140/Conv2D` | `FLOAT32` | `[1, 2, 2, 64]` | 441 | None | 0 |
| 441 | `model_5/model_4/model_3/max_pooling2d_26/MaxPool` | `FLOAT32` | `[1, 2, 2, 128]` | 442 | None | 0 |
| 442 | `model_5/model_4/model_3/p_re_lu_185/add;model_5/model_4/model_3/p_re_lu_185/Relu;model_5/model_4/model_3/p_re_lu_185/Neg_1;model_5/model_4/model_3/p_re_lu_185/Relu_1;model_5/model_4/model_3/p_re_lu_185/mul1` | `FLOAT32` | `[1, 2, 2, 64]` | 443 | None | 0 |
| 443 | `model_5/model_4/model_3/depthwise_conv2d_89/depthwise1` | `FLOAT32` | `[1, 2, 2, 64]` | 444 | None | 0 |
| 444 | `model_5/model_4/model_3/batch_normalization_192/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_141/Conv2D` | `FLOAT32` | `[1, 2, 2, 128]` | 445 | None | 0 |
| 445 | `model_5/model_4/model_3/add_85/add` | `FLOAT32` | `[1, 2, 2, 128]` | 446 | None | 0 |
| 446 | `model_5/model_4/model_3/p_re_lu_186/add;model_5/model_4/model_3/p_re_lu_186/Relu;model_5/model_4/model_3/p_re_lu_186/Neg_1;model_5/model_4/model_3/p_re_lu_186/Relu_1;model_5/model_4/model_3/p_re_lu_186/mul1` | `FLOAT32` | `[1, 2, 2, 128]` | 447 | None | 0 |
| 447 | `model_5/model_4/model_3/batch_normalization_193/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_142/Conv2D` | `FLOAT32` | `[1, 2, 2, 64]` | 448 | None | 0 |
| 448 | `model_5/model_4/model_3/p_re_lu_187/add;model_5/model_4/model_3/p_re_lu_187/Relu;model_5/model_4/model_3/p_re_lu_187/Neg_1;model_5/model_4/model_3/p_re_lu_187/Relu_1;model_5/model_4/model_3/p_re_lu_187/mul1` | `FLOAT32` | `[1, 2, 2, 64]` | 449 | None | 0 |
| 449 | `model_5/model_4/model_3/depthwise_conv2d_90/depthwise1` | `FLOAT32` | `[1, 2, 2, 64]` | 450 | None | 0 |
| 450 | `model_5/model_4/model_3/batch_normalization_194/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_143/Conv2D` | `FLOAT32` | `[1, 2, 2, 128]` | 451 | None | 0 |
| 451 | `model_5/model_4/model_3/add_86/add` | `FLOAT32` | `[1, 2, 2, 128]` | 452 | None | 0 |
| 452 | `model_5/model_4/model_3/p_re_lu_188/add;model_5/model_4/model_3/p_re_lu_188/Relu;model_5/model_4/model_3/p_re_lu_188/Neg_1;model_5/model_4/model_3/p_re_lu_188/Relu_1;model_5/model_4/model_3/p_re_lu_188/mul1` | `FLOAT32` | `[1, 2, 2, 128]` | 453 | None | 0 |
| 453 | `model_5/model_4/model_3/batch_normalization_195/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_144/Conv2D` | `FLOAT32` | `[1, 2, 2, 64]` | 454 | None | 0 |
| 454 | `model_5/model_4/model_3/p_re_lu_189/add;model_5/model_4/model_3/p_re_lu_189/Relu;model_5/model_4/model_3/p_re_lu_189/Neg_1;model_5/model_4/model_3/p_re_lu_189/Relu_1;model_5/model_4/model_3/p_re_lu_189/mul1` | `FLOAT32` | `[1, 2, 2, 64]` | 455 | None | 0 |
| 455 | `model_5/model_4/model_3/depthwise_conv2d_91/depthwise1` | `FLOAT32` | `[1, 2, 2, 64]` | 456 | None | 0 |
| 456 | `model_5/model_4/model_3/batch_normalization_196/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_145/Conv2D` | `FLOAT32` | `[1, 2, 2, 128]` | 457 | None | 0 |
| 457 | `model_5/model_4/model_3/add_87/add` | `FLOAT32` | `[1, 2, 2, 128]` | 458 | None | 0 |
| 458 | `model_5/model_4/model_3/p_re_lu_190/add;model_5/model_4/model_3/p_re_lu_190/Relu;model_5/model_4/model_3/p_re_lu_190/Neg_1;model_5/model_4/model_3/p_re_lu_190/Relu_1;model_5/model_4/model_3/p_re_lu_190/mul1` | `FLOAT32` | `[1, 2, 2, 128]` | 459 | None | 0 |
| 459 | `model_5/model_4/model_3/batch_normalization_197/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_146/Conv2D` | `FLOAT32` | `[1, 2, 2, 64]` | 460 | None | 0 |
| 460 | `model_5/model_4/model_3/p_re_lu_191/add;model_5/model_4/model_3/p_re_lu_191/Relu;model_5/model_4/model_3/p_re_lu_191/Neg_1;model_5/model_4/model_3/p_re_lu_191/Relu_1;model_5/model_4/model_3/p_re_lu_191/mul1` | `FLOAT32` | `[1, 2, 2, 64]` | 461 | None | 0 |
| 461 | `model_5/model_4/model_3/depthwise_conv2d_92/depthwise1` | `FLOAT32` | `[1, 2, 2, 64]` | 462 | None | 0 |
| 462 | `model_5/model_4/model_3/batch_normalization_198/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D;model_5/model_4/model_3/conv2d_147/Conv2D` | `FLOAT32` | `[1, 2, 2, 128]` | 463 | None | 0 |
| 463 | `model_5/model_4/model_3/add_88/add` | `FLOAT32` | `[1, 2, 2, 128]` | 464 | None | 0 |
| 464 | `model_5/model_4/model_3/p_re_lu_192/add;model_5/model_4/model_3/p_re_lu_192/Relu;model_5/model_4/model_3/p_re_lu_192/Neg_1;model_5/model_4/model_3/p_re_lu_192/Relu_1;model_5/model_4/model_3/p_re_lu_192/mul1` | `FLOAT32` | `[1, 2, 2, 128]` | 465 | None | 0 |
| 465 | `model_5/model_4/model_3/batch_normalization_199/FusedBatchNormV3;model_5/model_4/model_3/depthwise_conv2d_93/depthwise;model_5/model_4/model_3/conv2d_148/Conv2D` | `FLOAT32` | `[1, 2, 2, 64]` | 466 | None | 0 |
| 466 | `model_5/model_4/model_3/p_re_lu_193/add;model_5/model_4/model_3/p_re_lu_193/Relu;model_5/model_4/model_3/p_re_lu_193/Neg_1;model_5/model_4/model_3/p_re_lu_193/Relu_1;model_5/model_4/model_3/p_re_lu_193/mul1` | `FLOAT32` | `[1, 2, 2, 64]` | 467 | None | 0 |
| 467 | `model_5/model_4/model_3/depthwise_conv2d_93/depthwise2` | `FLOAT32` | `[1, 2, 2, 64]` | 468 | None | 0 |
| 468 | `model_5/model_4/model_3/batch_normalization_200/FusedBatchNormV3;model_5/model_4/model_3/conv2d_149/Conv2D` | `FLOAT32` | `[1, 2, 2, 128]` | 469 | None | 0 |
| 469 | `model_5/model_4/model_3/add_89/add` | `FLOAT32` | `[1, 2, 2, 128]` | 470 | None | 0 |
| 470 | `model_5/model_4/model_3/p_re_lu_194/add;model_5/model_4/model_3/p_re_lu_194/Relu;model_5/model_4/model_3/p_re_lu_194/Neg_1;model_5/model_4/model_3/p_re_lu_194/Relu_1;model_5/model_4/model_3/p_re_lu_194/mul1` | `FLOAT32` | `[1, 2, 2, 128]` | 471 | None | 0 |
| 471 | `model_5/conv2d_152/BiasAdd;model_5/model_4/conv2d_151/Conv2D;model_5/conv2d_152/Conv2D;model_5/conv2d_152/BiasAdd/ReadVariableOp/resource` | `FLOAT32` | `[1, 1, 1, 1]` | 472 | None | 0 |
| 472 | `Identity_1` | `FLOAT32` | `[1, 1, 1, 1]` | 473 | None | 0 |
| 473 | `Identity` | `FLOAT32` | `[1, 1, 1, 1434]` | 474 | None | 0 |
| 474 | `model_5/tf.math.sigmoid/Sigmoid;model_5/tf.reshape/Reshape` | `FLOAT32` | `[1, 1, 1, 1]` | 475 | None | 0 |
| 475 | `Identity_2` | `FLOAT32` | `[1, 1]` | 476 | None | 0 |
| 476 | `model_5/model_4/model_3/p_re_lu_172/add;model_5/model_4/model_3/p_re_lu_172/Relu;model_5/model_4/model_3/p_re_lu_172/Neg_1;model_5/model_4/model_3/p_re_lu_172/Relu_1;model_5/model_4/model_3/p_re_lu_172/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 477 | `model_5/model_4/model_3/conv2d_147/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 478 | `model_5/model_4/model_3/depthwise_conv2d_90/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 479 | `model_5/model_4/model_3/conv2d_95/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 16]` | 0 | None | 0 |
| 480 | `model_5/model_4/model_3/batch_normalization_139/FusedBatchNormV3_dequantize` | `FLOAT32` | `[8]` | 0 | None | 0 |
| 481 | `model_5/model_4/model_3/batch_normalization_182/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 482 | `model_5/model_4/model_3/p_re_lu_136/add;model_5/model_4/model_3/p_re_lu_136/Relu;model_5/model_4/model_3/p_re_lu_136/Neg_1;model_5/model_4/model_3/p_re_lu_136/Relu_1;model_5/model_4/model_3/p_re_lu_136/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 483 | `model_5/model_4/model_3/conv2d_100/Conv2D_dequantize` | `FLOAT32` | `[32, 2, 2, 32]` | 0 | None | 0 |
| 484 | `model_5/model_4/model_3/depthwise_conv2d_62/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 8]` | 0 | None | 0 |
| 485 | `model_5/model_4/model_3/batch_normalization_132/FusedBatchNormV3;model_5/model_4/model_3/conv2d_81/BiasAdd/ReadVariableOp/resource;model_5/model_4/model_3/conv2d_81/BiasAdd_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 486 | `model_5/model_4/model_3/batch_normalization_168/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 487 | `model_5/model_4/model_3/p_re_lu_129/add;model_5/model_4/model_3/p_re_lu_129/Relu;model_5/model_4/model_3/p_re_lu_129/Neg_1;model_5/model_4/model_3/p_re_lu_129/Relu_1;model_5/model_4/model_3/p_re_lu_129/mul_dequantize` | `FLOAT32` | `[1, 1, 8]` | 0 | None | 0 |
| 488 | `model_5/model_4/model_3/depthwise_conv2d_73/depthwise_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 489 | `model_5/model_4/model_3/batch_normalization_175/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 490 | `model_5/model_4/model_3/conv2d_133/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 491 | `model_5/model_4/model_3/p_re_lu_165/add;model_5/model_4/model_3/p_re_lu_165/Relu;model_5/model_4/model_3/p_re_lu_165/Neg_1;model_5/model_4/model_3/p_re_lu_165/Relu_1;model_5/model_4/model_3/p_re_lu_165/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 492 | `model_5/model_4/model_3/conv2d_138/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 493 | `model_5/model_4/model_3/batch_normalization_161/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 494 | `model_5/model_4/model_3/conv2d_124/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 495 | `model_5/model_4/model_3/batch_normalization_197/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 496 | `model_5/model_4/model_3/conv2d_82/Conv2D_dequantize` | `FLOAT32` | `[8, 1, 1, 16]` | 0 | None | 0 |
| 497 | `model_5/model_4/model_3/p_re_lu_194/add;model_5/model_4/model_3/p_re_lu_194/Relu;model_5/model_4/model_3/p_re_lu_194/Neg_1;model_5/model_4/model_3/p_re_lu_194/Relu_1;model_5/model_4/model_3/p_re_lu_194/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 498 | `model_5/model_4/model_3/p_re_lu_158/add;model_5/model_4/model_3/p_re_lu_158/Relu;model_5/model_4/model_3/p_re_lu_158/Neg_1;model_5/model_4/model_3/p_re_lu_158/Relu_1;model_5/model_4/model_3/p_re_lu_158/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 499 | `model_5/model_4/model_3/depthwise_conv2d_83/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 500 | `model_5/model_4/model_3/batch_normalization_154/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 501 | `model_5/model_4/model_3/depthwise_conv2d_76/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 502 | `model_5/model_4/model_3/p_re_lu_144/add;model_5/model_4/model_3/p_re_lu_144/Relu;model_5/model_4/model_3/p_re_lu_144/Neg_1;model_5/model_4/model_3/p_re_lu_144/Relu_1;model_5/model_4/model_3/p_re_lu_144/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 503 | `model_5/model_4/model_3/conv2d_110/Conv2D_dequantize` | `FLOAT32` | `[64, 2, 2, 64]` | 0 | None | 0 |
| 504 | `model_5/model_4/model_3/batch_normalization_190/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 505 | `model_5/model_4/model_3/p_re_lu_151/add;model_5/model_4/model_3/p_re_lu_151/Relu;model_5/model_4/model_3/p_re_lu_151/Neg_1;model_5/model_4/model_3/p_re_lu_151/Relu_1;model_5/model_4/model_3/p_re_lu_151/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 506 | `model_5/model_4/model_3/conv2d_119/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 507 | `model_5/model_4/model_3/p_re_lu_187/add;model_5/model_4/model_3/p_re_lu_187/Relu;model_5/model_4/model_3/p_re_lu_187/Neg_1;model_5/model_4/model_3/p_re_lu_187/Relu_1;model_5/model_4/model_3/p_re_lu_187/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 508 | `model_5/model_4/model_3/batch_normalization_174/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 509 | `model_5/model_4/model_3/conv2d_141/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 510 | `model_5/model_4/model_3/p_re_lu_164/add;model_5/model_4/model_3/p_re_lu_164/Relu;model_5/model_4/model_3/p_re_lu_164/Neg_1;model_5/model_4/model_3/p_re_lu_164/Relu_1;model_5/model_4/model_3/p_re_lu_164/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 511 | `model_5/model_4/model_3/conv2d_90/Conv2D_dequantize` | `FLOAT32` | `[16, 2, 2, 16]` | 0 | None | 0 |
| 512 | `model_5/model_4/model_3/p_re_lu_128/add;model_5/model_4/model_3/p_re_lu_128/Relu;model_5/model_4/model_3/p_re_lu_128/Neg_1;model_5/model_4/model_3/p_re_lu_128/Relu_1;model_5/model_4/model_3/p_re_lu_128/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 513 | `model_5/model_4/model_3/p_re_lu_171/add;model_5/model_4/model_3/p_re_lu_171/Relu;model_5/model_4/model_3/p_re_lu_171/Neg_1;model_5/model_4/model_3/p_re_lu_171/Relu_1;model_5/model_4/model_3/p_re_lu_171/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 514 | `model_5/model_4/model_3/depthwise_conv2d_87/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 515 | `model_5/model_4/model_3/conv2d_146/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 516 | `model_5/model_4/model_3/batch_normalization_138/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 517 | `model_5/model_4/model_3/conv2d_94/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 32]` | 0 | None | 0 |
| 518 | `model_5/model_4/model_3/conv2d_81/Conv2D_dequantize` | `FLOAT32` | `[16, 3, 3, 3]` | 0 | None | 0 |
| 519 | `model_5/model_4/model_3/p_re_lu_157/add;model_5/model_4/model_3/p_re_lu_157/Relu;model_5/model_4/model_3/p_re_lu_157/Neg_1;model_5/model_4/model_3/p_re_lu_157/Relu_1;model_5/model_4/model_3/p_re_lu_157/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 520 | `model_5/model_4/model_3/conv2d_85/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 8]` | 0 | None | 0 |
| 521 | `model_5/model_4/model_3/conv2d_127/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 522 | `model_5/model_4/model_3/batch_normalization_167/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 523 | `model_5/model_4/model_3/conv2d_132/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 524 | `model_5/model_4/model_3/p_re_lu_150/add;model_5/model_4/model_3/p_re_lu_150/Relu;model_5/model_4/model_3/p_re_lu_150/Neg_1;model_5/model_4/model_3/p_re_lu_150/Relu_1;model_5/model_4/model_3/p_re_lu_150/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 525 | `model_5/model_4/model_3/conv2d_118/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 526 | `model_5/model_4/model_3/p_re_lu_186/add;model_5/model_4/model_3/p_re_lu_186/Relu;model_5/model_4/model_3/p_re_lu_186/Neg_1;model_5/model_4/model_3/p_re_lu_186/Relu_1;model_5/model_4/model_3/p_re_lu_186/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 527 | `model_5/model_4/model_3/batch_normalization_160/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 528 | `model_5/model_4/model_3/depthwise_conv2d_80/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 529 | `model_5/model_4/model_3/p_re_lu_193/add;model_5/model_4/model_3/p_re_lu_193/Relu;model_5/model_4/model_3/p_re_lu_193/Neg_1;model_5/model_4/model_3/p_re_lu_193/Relu_1;model_5/model_4/model_3/p_re_lu_193/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 530 | `model_5/model_4/model_3/batch_normalization_196/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 531 | `model_5/model_4/model_3/p_re_lu_179/add;model_5/model_4/model_3/p_re_lu_179/Relu;model_5/model_4/model_3/p_re_lu_179/Neg_1;model_5/model_4/model_3/p_re_lu_179/Relu_1;model_5/model_4/model_3/p_re_lu_179/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 532 | `model_5/model_4/model_3/p_re_lu_143/add;model_5/model_4/model_3/p_re_lu_143/Relu;model_5/model_4/model_3/p_re_lu_143/Neg_1;model_5/model_4/model_3/p_re_lu_143/Relu_1;model_5/model_4/model_3/p_re_lu_143/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 533 | `model_5/model_4/model_3/depthwise_conv2d_73/depthwise1_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 534 | `model_5/model_4/model_3/batch_normalization_146/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 535 | `model_5/model_4/model_3/conv2d_104/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 64]` | 0 | None | 0 |
| 536 | `model_5/model_4/model_3/batch_normalization_153/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 537 | `model_5/model_4/model_3/conv2d_113/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 538 | `model_5/model_4/model_3/batch_normalization_189/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 539 | `model_5/model_4/model_3/p_re_lu_181/add;model_5/model_4/model_3/p_re_lu_181/Relu;model_5/model_4/model_3/p_re_lu_181/Neg_1;model_5/model_4/model_3/p_re_lu_181/Relu_1;model_5/model_4/model_3/p_re_lu_181/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 540 | `model_5/model_4/model_3/batch_normalization_148/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 541 | `model_5/model_4/model_3/depthwise_conv2d_72/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 542 | `model_5/model_4/model_3/batch_normalization_191/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 543 | `model_5/model_4/model_3/p_re_lu_145/add;model_5/model_4/model_3/p_re_lu_145/Relu;model_5/model_4/model_3/p_re_lu_145/Neg_1;model_5/model_4/model_3/p_re_lu_145/Relu_1;model_5/model_4/model_3/p_re_lu_145/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 544 | `model_5/model_4/model_3/conv2d_111/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 545 | `model_5/model_4/model_3/batch_normalization_141/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 546 | `model_5/model_4/model_3/conv2d_98/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 32]` | 0 | None | 0 |
| 547 | `model_5/model_4/model_3/batch_normalization_177/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 548 | `model_5/model_4/model_3/conv2d_145/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 549 | `model_5/model_4/model_3/batch_normalization_184/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 550 | `model_5/model_4/model_3/p_re_lu_174/add;model_5/model_4/model_3/p_re_lu_174/Relu;model_5/model_4/model_3/p_re_lu_174/Neg_1;model_5/model_4/model_3/p_re_lu_174/Relu_1;model_5/model_4/model_3/p_re_lu_174/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 551 | `model_5/model_4/model_3/conv2d_102/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 64]` | 0 | None | 0 |
| 552 | `model_5/model_4/model_3/p_re_lu_138/add;model_5/model_4/model_3/p_re_lu_138/Relu;model_5/model_4/model_3/p_re_lu_138/Neg_1;model_5/model_4/model_3/p_re_lu_138/Relu_1;model_5/model_4/model_3/p_re_lu_138/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 553 | `model_5/conv2d_152/Conv2D_dequantize` | `FLOAT32` | `[1, 2, 2, 128]` | 0 | None | 0 |
| 554 | `model_5/model_4/model_3/conv2d_136/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 555 | `model_5/model_4/model_3/conv2d_89/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 8]` | 0 | None | 0 |
| 556 | `model_5/model_4/model_3/batch_normalization_134/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 557 | `model_5/model_4/model_3/depthwise_conv2d_61/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 8]` | 0 | None | 0 |
| 558 | `model_5/model_4/model_3/batch_normalization_170/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 559 | `model_5/model_4/model_3/p_re_lu_131/add;model_5/model_4/model_3/p_re_lu_131/Relu;model_5/model_4/model_3/p_re_lu_131/Neg_1;model_5/model_4/model_3/p_re_lu_131/Relu_1;model_5/model_4/model_3/p_re_lu_131/mul_dequantize` | `FLOAT32` | `[1, 1, 8]` | 0 | None | 0 |
| 560 | `model_5/model_4/model_3/conv2d_93/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 16]` | 0 | None | 0 |
| 561 | `model_5/model_4/model_3/p_re_lu_167/add;model_5/model_4/model_3/p_re_lu_167/Relu;model_5/model_4/model_3/p_re_lu_167/Neg_1;model_5/model_4/model_3/p_re_lu_167/Relu_1;model_5/model_4/model_3/p_re_lu_167/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 562 | `model_5/model_4/model_3/depthwise_conv2d_89/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 563 | `model_5/model_4/model_3/batch_normalization_163/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 564 | `model_5/model_4/model_3/depthwise_conv2d_82/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 565 | `model_5/model_4/model_3/batch_normalization_199/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 566 | `model_5/model_4/model_3/p_re_lu_160/add;model_5/model_4/model_3/p_re_lu_160/Relu;model_5/model_4/model_3/p_re_lu_160/Neg_1;model_5/model_4/model_3/p_re_lu_160/Relu_1;model_5/model_4/model_3/p_re_lu_160/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 567 | `model_5/model_4/model_3/conv2d_131/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 568 | `model_5/model_4/model_3/depthwise_conv2d_68/depthwise_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 569 | `model_5/model_4/model_3/batch_normalization_183/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 570 | `model_5/model_4/model_3/p_re_lu_137/add;model_5/model_4/model_3/p_re_lu_137/Relu;model_5/model_4/model_3/p_re_lu_137/Neg_1;model_5/model_4/model_3/p_re_lu_137/Relu_1;model_5/model_4/model_3/p_re_lu_137/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 571 | `model_5/model_4/model_3/depthwise_conv2d_93/depthwise_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 572 | `model_5/model_4/model_3/p_re_lu_180/add;model_5/model_4/model_3/p_re_lu_180/Relu;model_5/model_4/model_3/p_re_lu_180/Neg_1;model_5/model_4/model_3/p_re_lu_180/Relu_1;model_5/model_4/model_3/p_re_lu_180/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 573 | `model_5/model_4/model_3/conv2d_105/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 32]` | 0 | None | 0 |
| 574 | `model_5/model_4/model_3/batch_normalization_147/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 575 | `model_5/model_4/model_3/conv2d_92/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 32]` | 0 | None | 0 |
| 576 | `model_5/model_4/model_3/p_re_lu_130/add;model_5/model_4/model_3/p_re_lu_130/Relu;model_5/model_4/model_3/p_re_lu_130/Neg_1;model_5/model_4/model_3/p_re_lu_130/Relu_1;model_5/model_4/model_3/p_re_lu_130/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 577 | `model_5/model_4/model_3/p_re_lu_166/add;model_5/model_4/model_3/p_re_lu_166/Relu;model_5/model_4/model_3/p_re_lu_166/Neg_1;model_5/model_4/model_3/p_re_lu_166/Relu_1;model_5/model_4/model_3/p_re_lu_166/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 578 | `model_5/model_4/model_3/conv2d_139/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 579 | `model_5/model_4/model_3/batch_normalization_140/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 580 | `model_5/model_4/model_3/depthwise_conv2d_67/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 16]` | 0 | None | 0 |
| 581 | `model_5/model_4/model_3/p_re_lu_173/add;model_5/model_4/model_3/p_re_lu_173/Relu;model_5/model_4/model_3/p_re_lu_173/Neg_1;model_5/model_4/model_3/p_re_lu_173/Relu_1;model_5/model_4/model_3/p_re_lu_173/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 582 | `model_5/model_4/model_3/depthwise_conv2d_93/depthwise1_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 583 | `model_5/model_4/model_3/batch_normalization_176/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 584 | `model_5/model_4/model_3/conv2d_144/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 585 | `model_5/model_4/model_3/p_re_lu_159/add;model_5/model_4/model_3/p_re_lu_159/Relu;model_5/model_4/model_3/p_re_lu_159/Neg_1;model_5/model_4/model_3/p_re_lu_159/Relu_1;model_5/model_4/model_3/p_re_lu_159/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 586 | `model_5/model_4/model_3/conv2d_83/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 8]` | 0 | None | 0 |
| 587 | `model_5/model_4/model_3/conv2d_130/Conv2D_dequantize` | `FLOAT32` | `[64, 2, 2, 128]` | 0 | None | 0 |
| 588 | `model_5/model_4/conv2d_151/BiasAdd/ReadVariableOp/resource_dequantize` | `FLOAT32` | `[1]` | 0 | None | 0 |
| 589 | `model_5/model_4/model_3/batch_normalization_133/FusedBatchNormV3_dequantize` | `FLOAT32` | `[8]` | 0 | None | 0 |
| 590 | `model_5/model_4/model_3/conv2d_88/Conv2D_dequantize` | `FLOAT32` | `[8, 1, 1, 16]` | 0 | None | 0 |
| 591 | `model_5/model_4/model_3/batch_normalization_169/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 592 | `model_5/model_4/model_3/depthwise_conv2d_86/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 593 | `model_5/model_4/model_3/p_re_lu_152/add;model_5/model_4/model_3/p_re_lu_152/Relu;model_5/model_4/model_3/p_re_lu_152/Neg_1;model_5/model_4/model_3/p_re_lu_152/Relu_1;model_5/model_4/model_3/p_re_lu_152/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 594 | `model_5/model_4/model_3/depthwise_conv2d_79/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 595 | `model_5/model_4/model_3/batch_normalization_155/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 596 | `model_5/model_4/model_3/conv2d_116/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 597 | `model_5/model_4/model_3/p_re_lu_188/add;model_5/model_4/model_3/p_re_lu_188/Relu;model_5/model_4/model_3/p_re_lu_188/Neg_1;model_5/model_4/model_3/p_re_lu_188/Relu_1;model_5/model_4/model_3/p_re_lu_188/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 598 | `model_5/model_4/model_3/batch_normalization_162/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 599 | `model_5/model_4/model_3/conv2d_125/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 600 | `model_5/model_4/model_3/batch_normalization_198/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 601 | `model_5/model_4/model_3/p_re_lu_190/add;model_5/model_4/model_3/p_re_lu_190/Relu;model_5/model_4/model_3/p_re_lu_190/Neg_1;model_5/model_4/model_3/p_re_lu_190/Relu_1;model_5/model_4/model_3/p_re_lu_190/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 602 | `model_5/model_4/model_3/batch_normalization_157/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 603 | `model_5/model_4/model_3/depthwise_conv2d_78/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 604 | `model_5/model_4/model_3/batch_normalization_164/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 605 | `model_5/model_4/model_3/conv2d_128/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 606 | `model_5/model_4/model_3/p_re_lu_154/add;model_5/model_4/model_3/p_re_lu_154/Relu;model_5/model_4/model_3/p_re_lu_154/Neg_1;model_5/model_4/model_3/p_re_lu_154/Relu_1;model_5/model_4/model_3/p_re_lu_154/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 607 | `model_5/model_4/model_3/conv2d_123/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 608 | `model_5/model_4/model_3/batch_normalization_200/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 609 | `model_5/model_4/model_3/batch_normalization_150/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 610 | `model_5/model_4/model_3/conv2d_109/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 32]` | 0 | None | 0 |
| 611 | `model_5/model_4/model_3/p_re_lu_183/add;model_5/model_4/model_3/p_re_lu_183/Relu;model_5/model_4/model_3/p_re_lu_183/Neg_1;model_5/model_4/model_3/p_re_lu_183/Relu_1;model_5/model_4/model_3/p_re_lu_183/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 612 | `model_5/model_4/model_3/batch_normalization_186/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 613 | `model_5/model_4/model_3/batch_normalization_193/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 614 | `model_5/model_4/model_3/conv2d_114/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 615 | `model_5/model_4/model_3/p_re_lu_147/add;model_5/model_4/model_3/p_re_lu_147/Relu;model_5/model_4/model_3/p_re_lu_147/Neg_1;model_5/model_4/model_3/p_re_lu_147/Relu_1;model_5/model_4/model_3/p_re_lu_147/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 616 | `model_5/model_4/model_3/batch_normalization_143/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 617 | `model_5/model_4/model_3/depthwise_conv2d_69/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 618 | `model_5/model_4/model_3/batch_normalization_179/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 619 | `model_5/model_4/model_3/conv2d_148/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 620 | `model_5/model_4/model_3/depthwise_conv2d_71/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 621 | `model_5/model_4/model_3/p_re_lu_140/add;model_5/model_4/model_3/p_re_lu_140/Relu;model_5/model_4/model_3/p_re_lu_140/Neg_1;model_5/model_4/model_3/p_re_lu_140/Relu_1;model_5/model_4/model_3/p_re_lu_140/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 622 | `model_5/model_4/model_3/p_re_lu_176/add;model_5/model_4/model_3/p_re_lu_176/Relu;model_5/model_4/model_3/p_re_lu_176/Neg_1;model_5/model_4/model_3/p_re_lu_176/Relu_1;model_5/model_4/model_3/p_re_lu_176/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 623 | `model_5/model_4/model_3/batch_normalization_172/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 624 | `model_5/model_4/model_3/depthwise_conv2d_88/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 625 | `model_5/model_4/model_3/p_re_lu_126/add;model_5/model_4/model_3/p_re_lu_126/Relu;model_5/model_4/model_3/p_re_lu_126/Neg_1;model_5/model_4/model_3/p_re_lu_126/Relu_1;model_5/model_4/model_3/p_re_lu_126/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 626 | `model_5/model_4/model_3/conv2d_87/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 8]` | 0 | None | 0 |
| 627 | `model_5/model_4/model_3/p_re_lu_169/add;model_5/model_4/model_3/p_re_lu_169/Relu;model_5/model_4/model_3/p_re_lu_169/Neg_1;model_5/model_4/model_3/p_re_lu_169/Relu_1;model_5/model_4/model_3/p_re_lu_169/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 628 | `model_5/model_4/model_3/conv2d_96/Conv2D_dequantize` | `FLOAT32` | `[16, 1, 1, 32]` | 0 | None | 0 |
| 629 | `model_5/model_4/model_3/p_re_lu_133/add;model_5/model_4/model_3/p_re_lu_133/Relu;model_5/model_4/model_3/p_re_lu_133/Neg_1;model_5/model_4/model_3/p_re_lu_133/Relu_1;model_5/model_4/model_3/p_re_lu_133/mul_dequantize` | `FLOAT32` | `[1, 1, 8]` | 0 | None | 0 |
| 630 | `model_5/model_4/model_3/conv2d_91/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 16]` | 0 | None | 0 |
| 631 | `model_5/model_4/model_3/batch_normalization_136/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 632 | `model_5/model_4/model_3/conv2d_143/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 633 | `model_5/model_4/model_3/batch_normalization_192/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 634 | `model_5/model_4/model_3/p_re_lu_146/add;model_5/model_4/model_3/p_re_lu_146/Relu;model_5/model_4/model_3/p_re_lu_146/Neg_1;model_5/model_4/model_3/p_re_lu_146/Relu_1;model_5/model_4/model_3/p_re_lu_146/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 635 | `model_5/model_4/model_3/depthwise_conv2d_75/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 636 | `model_5/model_4/model_3/p_re_lu_189/add;model_5/model_4/model_3/p_re_lu_189/Relu;model_5/model_4/model_3/p_re_lu_189/Neg_1;model_5/model_4/model_3/p_re_lu_189/Relu_1;model_5/model_4/model_3/p_re_lu_189/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 637 | `model_5/model_4/model_3/p_re_lu_153/add;model_5/model_4/model_3/p_re_lu_153/Relu;model_5/model_4/model_3/p_re_lu_153/Neg_1;model_5/model_4/model_3/p_re_lu_153/Relu_1;model_5/model_4/model_3/p_re_lu_153/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 638 | `model_5/model_4/model_3/conv2d_122/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 639 | `model_5/model_4/model_3/conv2d_117/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 640 | `model_5/model_4/model_3/batch_normalization_156/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 641 | `model_5/model_4/model_3/p_re_lu_139/add;model_5/model_4/model_3/p_re_lu_139/Relu;model_5/model_4/model_3/p_re_lu_139/Neg_1;model_5/model_4/model_3/p_re_lu_139/Relu_1;model_5/model_4/model_3/p_re_lu_139/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 642 | `model_5/model_4/model_3/conv2d_103/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 32]` | 0 | None | 0 |
| 643 | `model_5/model_4/model_3/batch_normalization_185/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 644 | `model_5/model_4/model_3/p_re_lu_175/add;model_5/model_4/model_3/p_re_lu_175/Relu;model_5/model_4/model_3/p_re_lu_175/Neg_1;model_5/model_4/model_3/p_re_lu_175/Relu_1;model_5/model_4/model_3/p_re_lu_175/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 645 | `model_5/model_4/model_3/conv2d_150/Conv2D_dequantize` | `FLOAT32` | `[1434, 2, 2, 128]` | 0 | None | 0 |
| 646 | `model_5/model_4/model_3/p_re_lu_182/add;model_5/model_4/model_3/p_re_lu_182/Relu;model_5/model_4/model_3/p_re_lu_182/Neg_1;model_5/model_4/model_3/p_re_lu_182/Relu_1;model_5/model_4/model_3/p_re_lu_182/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 647 | `model_5/model_4/model_3/batch_normalization_149/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 648 | `model_5/model_4/model_3/conv2d_108/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 64]` | 0 | None | 0 |
| 649 | `model_5/model_4/model_3/p_re_lu_132/add;model_5/model_4/model_3/p_re_lu_132/Relu;model_5/model_4/model_3/p_re_lu_132/Neg_1;model_5/model_4/model_3/p_re_lu_132/Relu_1;model_5/model_4/model_3/p_re_lu_132/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 650 | `model_5/model_4/model_3/depthwise_conv2d_66/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 16]` | 0 | None | 0 |
| 651 | `model_5/model_4/model_3/batch_normalization_135/FusedBatchNormV3_dequantize` | `FLOAT32` | `[8]` | 0 | None | 0 |
| 652 | `model_5/model_4/model_3/depthwise_conv2d_64/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 16]` | 0 | None | 0 |
| 653 | `model_5/model_4/model_3/p_re_lu_168/add;model_5/model_4/model_3/p_re_lu_168/Relu;model_5/model_4/model_3/p_re_lu_168/Neg_1;model_5/model_4/model_3/p_re_lu_168/Relu_1;model_5/model_4/model_3/p_re_lu_168/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 654 | `model_5/model_4/model_3/conv2d_142/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 655 | `model_5/model_4/model_3/conv2d_99/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 16]` | 0 | None | 0 |
| 656 | `model_5/model_4/model_3/batch_normalization_142/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 657 | `model_5/model_4/model_3/batch_normalization_178/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 658 | `model_5/model_4/model_3/depthwise_conv2d_92/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 659 | `model_5/model_4/model_3/p_re_lu_161/add;model_5/model_4/model_3/p_re_lu_161/Relu;model_5/model_4/model_3/p_re_lu_161/Neg_1;model_5/model_4/model_3/p_re_lu_161/Relu_1;model_5/model_4/model_3/p_re_lu_161/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 660 | `model_5/model_4/model_3/depthwise_conv2d_85/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 661 | `model_5/model_4/model_3/depthwise_conv2d_63/depthwise_dequantize` | `FLOAT32` | `[8]` | 0 | None | 0 |
| 662 | `model_5/model_4/model_3/batch_normalization_171/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 663 | `model_5/model_4/model_3/conv2d_137/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 664 | `model_5/model_4/model_3/conv2d_86/Conv2D_dequantize` | `FLOAT32` | `[8, 1, 1, 16]` | 0 | None | 0 |
| 665 | `model_5/model_4/model_3/conv2d_84/Conv2D_dequantize` | `FLOAT32` | `[8, 1, 1, 16]` | 0 | None | 0 |
| 666 | `model_5/model_4/model_3/p_re_lu_163/add;model_5/model_4/model_3/p_re_lu_163/Relu;model_5/model_4/model_3/p_re_lu_163/Neg_1;model_5/model_4/model_3/p_re_lu_163/Relu_1;model_5/model_4/model_3/p_re_lu_163/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 667 | `model_5/model_4/model_3/conv2d_135/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 668 | `model_5/model_4/model_3/batch_normalization_166/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 669 | `model_5/model_4/model_3/depthwise_conv2d_84/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 670 | `model_5/model_4/model_3/batch_normalization_173/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 671 | `model_5/model_4/model_3/conv2d_140/Conv2D_dequantize` | `FLOAT32` | `[64, 2, 2, 128]` | 0 | None | 0 |
| 672 | `model_5/model_4/model_3/p_re_lu_127/add;model_5/model_4/model_3/p_re_lu_127/Relu;model_5/model_4/model_3/p_re_lu_127/Neg_1;model_5/model_4/model_3/p_re_lu_127/Relu_1;model_5/model_4/model_3/p_re_lu_127/mul_dequantize` | `FLOAT32` | `[1, 1, 8]` | 0 | None | 0 |
| 673 | `model_5/model_4/model_3/depthwise_conv2d_63/depthwise1_dequantize` | `FLOAT32` | `[1, 3, 3, 8]` | 0 | None | 0 |
| 674 | `model_5/model_4/model_3/p_re_lu_192/add;model_5/model_4/model_3/p_re_lu_192/Relu;model_5/model_4/model_3/p_re_lu_192/Neg_1;model_5/model_4/model_3/p_re_lu_192/Relu_1;model_5/model_4/model_3/p_re_lu_192/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 675 | `model_5/model_4/model_3/batch_normalization_159/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 676 | `model_5/model_4/model_3/conv2d_121/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 677 | `model_5/model_4/model_3/conv2d_150/BiasAdd/ReadVariableOp/resource_dequantize` | `FLOAT32` | `[1434]` | 0 | None | 0 |
| 678 | `model_5/model_4/model_3/p_re_lu_156/add;model_5/model_4/model_3/p_re_lu_156/Relu;model_5/model_4/model_3/p_re_lu_156/Neg_1;model_5/model_4/model_3/p_re_lu_156/Relu_1;model_5/model_4/model_3/p_re_lu_156/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 679 | `model_5/model_4/model_3/conv2d_126/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 680 | `model_5/model_4/model_3/batch_normalization_152/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 681 | `model_5/model_4/model_3/conv2d_112/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 682 | `model_5/model_4/model_3/batch_normalization_188/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 683 | `model_5/model_4/model_3/p_re_lu_149/add;model_5/model_4/model_3/p_re_lu_149/Relu;model_5/model_4/model_3/p_re_lu_149/Neg_1;model_5/model_4/model_3/p_re_lu_149/Relu_1;model_5/model_4/model_3/p_re_lu_149/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 684 | `model_5/model_4/model_3/depthwise_conv2d_77/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 685 | `model_5/model_4/model_3/batch_normalization_195/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 686 | `model_5/model_4/model_3/p_re_lu_185/add;model_5/model_4/model_3/p_re_lu_185/Relu;model_5/model_4/model_3/p_re_lu_185/Neg_1;model_5/model_4/model_3/p_re_lu_185/Relu_1;model_5/model_4/model_3/p_re_lu_185/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 687 | `model_5/model_4/model_3/batch_normalization_181/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 688 | `model_5/model_4/conv2d_151/Conv2D_dequantize` | `FLOAT32` | `[1, 2, 2, 128]` | 0 | None | 0 |
| 689 | `model_5/model_4/model_3/depthwise_conv2d_68/depthwise1_dequantize` | `FLOAT32` | `[1, 3, 3, 16]` | 0 | None | 0 |
| 690 | `model_5/model_4/model_3/p_re_lu_135/add;model_5/model_4/model_3/p_re_lu_135/Relu;model_5/model_4/model_3/p_re_lu_135/Neg_1;model_5/model_4/model_3/p_re_lu_135/Relu_1;model_5/model_4/model_3/p_re_lu_135/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 691 | `model_5/model_4/model_3/p_re_lu_142/add;model_5/model_4/model_3/p_re_lu_142/Relu;model_5/model_4/model_3/p_re_lu_142/Neg_1;model_5/model_4/model_3/p_re_lu_142/Relu_1;model_5/model_4/model_3/p_re_lu_142/mul_dequantize` | `FLOAT32` | `[1, 1, 32]` | 0 | None | 0 |
| 692 | `model_5/model_4/model_3/conv2d_107/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 32]` | 0 | None | 0 |
| 693 | `model_5/model_4/model_3/batch_normalization_145/FusedBatchNormV3_dequantize` | `FLOAT32` | `[16]` | 0 | None | 0 |
| 694 | `model_5/model_4/model_3/depthwise_conv2d_70/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 32]` | 0 | None | 0 |
| 695 | `model_5/model_4/model_3/p_re_lu_178/add;model_5/model_4/model_3/p_re_lu_178/Relu;model_5/model_4/model_3/p_re_lu_178/Neg_1;model_5/model_4/model_3/p_re_lu_178/Relu_1;model_5/model_4/model_3/p_re_lu_178/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 696 | `model_5/conv2d_152/BiasAdd/ReadVariableOp/resource_dequantize` | `FLOAT32` | `[1]` | 0 | None | 0 |
| 697 | `model_5/model_4/model_3/p_re_lu_155/add;model_5/model_4/model_3/p_re_lu_155/Relu;model_5/model_4/model_3/p_re_lu_155/Neg_1;model_5/model_4/model_3/p_re_lu_155/Relu_1;model_5/model_4/model_3/p_re_lu_155/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 698 | `model_5/model_4/model_3/depthwise_conv2d_81/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 699 | `model_5/model_4/model_3/p_re_lu_162/add;model_5/model_4/model_3/p_re_lu_162/Relu;model_5/model_4/model_3/p_re_lu_162/Neg_1;model_5/model_4/model_3/p_re_lu_162/Relu_1;model_5/model_4/model_3/p_re_lu_162/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 700 | `model_5/model_4/model_3/conv2d_134/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 128]` | 0 | None | 0 |
| 701 | `model_5/model_4/model_3/batch_normalization_165/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 702 | `model_5/model_4/model_3/depthwise_conv2d_60/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 8]` | 0 | None | 0 |
| 703 | `model_5/model_4/model_3/conv2d_129/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 704 | `model_5/model_4/model_3/batch_normalization_194/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |
| 705 | `model_5/model_4/model_3/p_re_lu_184/add;model_5/model_4/model_3/p_re_lu_184/Relu;model_5/model_4/model_3/p_re_lu_184/Neg_1;model_5/model_4/model_3/p_re_lu_184/Relu_1;model_5/model_4/model_3/p_re_lu_184/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 706 | `model_5/model_4/model_3/p_re_lu_148/add;model_5/model_4/model_3/p_re_lu_148/Relu;model_5/model_4/model_3/p_re_lu_148/Neg_1;model_5/model_4/model_3/p_re_lu_148/Relu_1;model_5/model_4/model_3/p_re_lu_148/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 707 | `model_5/model_4/model_3/conv2d_115/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 708 | `model_5/model_4/model_3/p_re_lu_191/add;model_5/model_4/model_3/p_re_lu_191/Relu;model_5/model_4/model_3/p_re_lu_191/Neg_1;model_5/model_4/model_3/p_re_lu_191/Relu_1;model_5/model_4/model_3/p_re_lu_191/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 709 | `model_5/model_4/model_3/batch_normalization_158/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 710 | `model_5/model_4/model_3/conv2d_120/Conv2D_dequantize` | `FLOAT32` | `[64, 2, 2, 128]` | 0 | None | 0 |
| 711 | `model_5/model_4/model_3/p_re_lu_141/add;model_5/model_4/model_3/p_re_lu_141/Relu;model_5/model_4/model_3/p_re_lu_141/Neg_1;model_5/model_4/model_3/p_re_lu_141/Relu_1;model_5/model_4/model_3/p_re_lu_141/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 712 | `model_5/model_4/model_3/conv2d_106/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 64]` | 0 | None | 0 |
| 713 | `model_5/model_4/model_3/p_re_lu_177/add;model_5/model_4/model_3/p_re_lu_177/Relu;model_5/model_4/model_3/p_re_lu_177/Neg_1;model_5/model_4/model_3/p_re_lu_177/Relu_1;model_5/model_4/model_3/p_re_lu_177/mul_dequantize` | `FLOAT32` | `[1, 1, 64]` | 0 | None | 0 |
| 714 | `model_5/model_4/model_3/depthwise_conv2d_74/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 715 | `model_5/model_4/model_3/batch_normalization_151/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 716 | `model_5/model_4/model_3/batch_normalization_187/FusedBatchNormV3_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 717 | `model_5/model_4/model_3/p_re_lu_170/add;model_5/model_4/model_3/p_re_lu_170/Relu;model_5/model_4/model_3/p_re_lu_170/Neg_1;model_5/model_4/model_3/p_re_lu_170/Relu_1;model_5/model_4/model_3/p_re_lu_170/mul_dequantize` | `FLOAT32` | `[1, 1, 128]` | 0 | None | 0 |
| 718 | `model_5/model_4/model_3/depthwise_conv2d_91/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 64]` | 0 | None | 0 |
| 719 | `model_5/model_4/model_3/depthwise_conv2d_65/depthwise_dequantize` | `FLOAT32` | `[1, 3, 3, 16]` | 0 | None | 0 |
| 720 | `model_5/model_4/model_3/batch_normalization_137/FusedBatchNormV3_dequantize` | `FLOAT32` | `[8]` | 0 | None | 0 |
| 721 | `model_5/model_4/model_3/conv2d_149/Conv2D_dequantize` | `FLOAT32` | `[128, 1, 1, 64]` | 0 | None | 0 |
| 722 | `model_5/model_4/model_3/conv2d_101/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 32]` | 0 | None | 0 |
| 723 | `model_5/model_4/model_3/batch_normalization_144/FusedBatchNormV3_dequantize` | `FLOAT32` | `[32]` | 0 | None | 0 |
| 724 | `model_5/model_4/model_3/p_re_lu_134/add;model_5/model_4/model_3/p_re_lu_134/Relu;model_5/model_4/model_3/p_re_lu_134/Neg_1;model_5/model_4/model_3/p_re_lu_134/Relu_1;model_5/model_4/model_3/p_re_lu_134/mul_dequantize` | `FLOAT32` | `[1, 1, 16]` | 0 | None | 0 |
| 725 | `model_5/model_4/model_3/conv2d_97/Conv2D_dequantize` | `FLOAT32` | `[32, 1, 1, 16]` | 0 | None | 0 |
| 726 | `model_5/model_4/model_3/batch_normalization_180/FusedBatchNormV3_dequantize` | `FLOAT32` | `[128]` | 0 | None | 0 |

#### Nodes

| index | op | inputs | output | exact schema-derived options |
| ---: | --- | --- | ---: | --- |
| 0 | `DEQUANTIZE` | `[1]` | 485 | `{}` |
| 1 | `DEQUANTIZE` | `[143]` | 518 | `{}` |
| 2 | `CONV_2D` | `[0, 518, 485]` | 256 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 3 | `DEQUANTIZE` | `[2]` | 625 | `{}` |
| 4 | `PRELU` | `[256, 625]` | 257 | `{}` |
| 5 | `DEQUANTIZE` | `[145]` | 496 | `{}` |
| 6 | `DEQUANTIZE` | `[3]` | 589 | `{}` |
| 7 | `CONV_2D` | `[257, 496, 589]` | 258 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 8 | `DEQUANTIZE` | `[4]` | 672 | `{}` |
| 9 | `PRELU` | `[258, 672]` | 259 | `{}` |
| 10 | `DEQUANTIZE` | `[144]` | 661 | `{}` |
| 11 | `DEQUANTIZE` | `[146]` | 702 | `{}` |
| 12 | `DEPTHWISE_CONV_2D` | `[259, 702, 661]` | 260 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 13 | `DEQUANTIZE` | `[5]` | 556 | `{}` |
| 14 | `DEQUANTIZE` | `[147]` | 586 | `{}` |
| 15 | `CONV_2D` | `[260, 586, 556]` | 261 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 16 | `ADD` | `[257, 261]` | 262 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 17 | `DEQUANTIZE` | `[6]` | 512 | `{}` |
| 18 | `PRELU` | `[262, 512]` | 263 | `{}` |
| 19 | `DEQUANTIZE` | `[7]` | 651 | `{}` |
| 20 | `DEQUANTIZE` | `[148]` | 665 | `{}` |
| 21 | `CONV_2D` | `[263, 665, 651]` | 264 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 22 | `DEQUANTIZE` | `[8]` | 487 | `{}` |
| 23 | `PRELU` | `[264, 487]` | 265 | `{}` |
| 24 | `DEQUANTIZE` | `[149]` | 557 | `{}` |
| 25 | `DEPTHWISE_CONV_2D` | `[265, 557, 661]` | 266 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 26 | `DEQUANTIZE` | `[150]` | 520 | `{}` |
| 27 | `DEQUANTIZE` | `[9]` | 631 | `{}` |
| 28 | `CONV_2D` | `[266, 520, 631]` | 267 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 29 | `ADD` | `[263, 267]` | 268 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 30 | `DEQUANTIZE` | `[10]` | 576 | `{}` |
| 31 | `PRELU` | `[268, 576]` | 269 | `{}` |
| 32 | `DEQUANTIZE` | `[151]` | 664 | `{}` |
| 33 | `DEQUANTIZE` | `[11]` | 720 | `{}` |
| 34 | `CONV_2D` | `[269, 664, 720]` | 270 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 35 | `DEQUANTIZE` | `[12]` | 559 | `{}` |
| 36 | `PRELU` | `[270, 559]` | 271 | `{}` |
| 37 | `DEQUANTIZE` | `[152]` | 484 | `{}` |
| 38 | `DEPTHWISE_CONV_2D` | `[271, 484, 661]` | 272 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 39 | `DEQUANTIZE` | `[13]` | 516 | `{}` |
| 40 | `DEQUANTIZE` | `[153]` | 626 | `{}` |
| 41 | `CONV_2D` | `[272, 626, 516]` | 273 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 42 | `ADD` | `[269, 273]` | 274 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 43 | `DEQUANTIZE` | `[14]` | 649 | `{}` |
| 44 | `PRELU` | `[274, 649]` | 275 | `{}` |
| 45 | `DEQUANTIZE` | `[15]` | 480 | `{}` |
| 46 | `DEQUANTIZE` | `[154]` | 590 | `{}` |
| 47 | `CONV_2D` | `[275, 590, 480]` | 276 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 48 | `DEQUANTIZE` | `[16]` | 629 | `{}` |
| 49 | `PRELU` | `[276, 629]` | 277 | `{}` |
| 50 | `DEQUANTIZE` | `[155]` | 673 | `{}` |
| 51 | `DEPTHWISE_CONV_2D` | `[277, 673, 661]` | 278 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 52 | `DEQUANTIZE` | `[156]` | 555 | `{}` |
| 53 | `DEQUANTIZE` | `[17]` | 579 | `{}` |
| 54 | `CONV_2D` | `[278, 555, 579]` | 279 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 55 | `ADD` | `[275, 279]` | 280 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 56 | `DEQUANTIZE` | `[18]` | 724 | `{}` |
| 57 | `PRELU` | `[280, 724]` | 281 | `{}` |
| 58 | `DEQUANTIZE` | `[157]` | 511 | `{}` |
| 59 | `DEQUANTIZE` | `[19]` | 545 | `{}` |
| 60 | `CONV_2D` | `[281, 511, 545]` | 282 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 61 | `MAX_POOL_2D` | `[281]` | 283 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 62 | `PAD` | `[283, 255]` | 284 | `{}` |
| 63 | `DEQUANTIZE` | `[20]` | 690 | `{}` |
| 64 | `PRELU` | `[282, 690]` | 285 | `{}` |
| 65 | `DEQUANTIZE` | `[142]` | 568 | `{}` |
| 66 | `DEQUANTIZE` | `[158]` | 652 | `{}` |
| 67 | `DEPTHWISE_CONV_2D` | `[285, 652, 568]` | 286 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 68 | `DEQUANTIZE` | `[160]` | 630 | `{}` |
| 69 | `DEQUANTIZE` | `[21]` | 656 | `{}` |
| 70 | `CONV_2D` | `[286, 630, 656]` | 287 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 71 | `ADD` | `[284, 287]` | 288 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 72 | `DEQUANTIZE` | `[22]` | 482 | `{}` |
| 73 | `PRELU` | `[288, 482]` | 289 | `{}` |
| 74 | `DEQUANTIZE` | `[161]` | 575 | `{}` |
| 75 | `DEQUANTIZE` | `[23]` | 616 | `{}` |
| 76 | `CONV_2D` | `[289, 575, 616]` | 290 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 77 | `DEQUANTIZE` | `[24]` | 570 | `{}` |
| 78 | `PRELU` | `[290, 570]` | 291 | `{}` |
| 79 | `DEQUANTIZE` | `[162]` | 719 | `{}` |
| 80 | `DEPTHWISE_CONV_2D` | `[291, 719, 568]` | 292 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 81 | `DEQUANTIZE` | `[163]` | 560 | `{}` |
| 82 | `DEQUANTIZE` | `[25]` | 723 | `{}` |
| 83 | `CONV_2D` | `[292, 560, 723]` | 293 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 84 | `ADD` | `[289, 293]` | 294 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 85 | `DEQUANTIZE` | `[26]` | 552 | `{}` |
| 86 | `PRELU` | `[294, 552]` | 295 | `{}` |
| 87 | `DEQUANTIZE` | `[164]` | 517 | `{}` |
| 88 | `DEQUANTIZE` | `[27]` | 693 | `{}` |
| 89 | `CONV_2D` | `[295, 517, 693]` | 296 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 90 | `DEQUANTIZE` | `[28]` | 641 | `{}` |
| 91 | `PRELU` | `[296, 641]` | 297 | `{}` |
| 92 | `DEQUANTIZE` | `[165]` | 650 | `{}` |
| 93 | `DEPTHWISE_CONV_2D` | `[297, 650, 568]` | 298 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 94 | `DEQUANTIZE` | `[166]` | 479 | `{}` |
| 95 | `DEQUANTIZE` | `[29]` | 534 | `{}` |
| 96 | `CONV_2D` | `[298, 479, 534]` | 299 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 97 | `ADD` | `[295, 299]` | 300 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 98 | `DEQUANTIZE` | `[30]` | 621 | `{}` |
| 99 | `PRELU` | `[300, 621]` | 301 | `{}` |
| 100 | `DEQUANTIZE` | `[31]` | 574 | `{}` |
| 101 | `DEQUANTIZE` | `[167]` | 628 | `{}` |
| 102 | `CONV_2D` | `[301, 628, 574]` | 302 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 103 | `DEQUANTIZE` | `[32]` | 711 | `{}` |
| 104 | `PRELU` | `[302, 711]` | 303 | `{}` |
| 105 | `DEQUANTIZE` | `[168]` | 580 | `{}` |
| 106 | `DEPTHWISE_CONV_2D` | `[303, 580, 568]` | 304 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 107 | `DEQUANTIZE` | `[33]` | 540 | `{}` |
| 108 | `DEQUANTIZE` | `[169]` | 725 | `{}` |
| 109 | `CONV_2D` | `[304, 725, 540]` | 305 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 110 | `ADD` | `[301, 305]` | 306 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 111 | `DEQUANTIZE` | `[34]` | 691 | `{}` |
| 112 | `PRELU` | `[306, 691]` | 307 | `{}` |
| 113 | `DEQUANTIZE` | `[170]` | 546 | `{}` |
| 114 | `DEQUANTIZE` | `[35]` | 647 | `{}` |
| 115 | `CONV_2D` | `[307, 546, 647]` | 308 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 116 | `DEQUANTIZE` | `[36]` | 532 | `{}` |
| 117 | `PRELU` | `[308, 532]` | 309 | `{}` |
| 118 | `DEQUANTIZE` | `[171]` | 689 | `{}` |
| 119 | `DEPTHWISE_CONV_2D` | `[309, 689, 568]` | 310 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 120 | `DEQUANTIZE` | `[37]` | 609 | `{}` |
| 121 | `DEQUANTIZE` | `[172]` | 655 | `{}` |
| 122 | `CONV_2D` | `[310, 655, 609]` | 311 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 123 | `ADD` | `[307, 311]` | 312 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 124 | `DEQUANTIZE` | `[38]` | 502 | `{}` |
| 125 | `PRELU` | `[312, 502]` | 313 | `{}` |
| 126 | `DEQUANTIZE` | `[173]` | 483 | `{}` |
| 127 | `DEQUANTIZE` | `[39]` | 715 | `{}` |
| 128 | `CONV_2D` | `[313, 483, 715]` | 314 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 129 | `MAX_POOL_2D` | `[313]` | 315 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 130 | `PAD` | `[315, 254]` | 316 | `{}` |
| 131 | `DEQUANTIZE` | `[40]` | 543 | `{}` |
| 132 | `PRELU` | `[314, 543]` | 317 | `{}` |
| 133 | `DEQUANTIZE` | `[159]` | 488 | `{}` |
| 134 | `DEQUANTIZE` | `[174]` | 617 | `{}` |
| 135 | `DEPTHWISE_CONV_2D` | `[317, 617, 488]` | 318 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 136 | `DEQUANTIZE` | `[41]` | 680 | `{}` |
| 137 | `DEQUANTIZE` | `[176]` | 722 | `{}` |
| 138 | `CONV_2D` | `[318, 722, 680]` | 319 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 139 | `ADD` | `[316, 319]` | 320 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 140 | `DEQUANTIZE` | `[42]` | 634 | `{}` |
| 141 | `PRELU` | `[320, 634]` | 321 | `{}` |
| 142 | `DEQUANTIZE` | `[43]` | 536 | `{}` |
| 143 | `DEQUANTIZE` | `[177]` | 551 | `{}` |
| 144 | `CONV_2D` | `[321, 551, 536]` | 322 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 145 | `DEQUANTIZE` | `[44]` | 615 | `{}` |
| 146 | `PRELU` | `[322, 615]` | 323 | `{}` |
| 147 | `DEQUANTIZE` | `[178]` | 694 | `{}` |
| 148 | `DEPTHWISE_CONV_2D` | `[323, 694, 488]` | 324 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 149 | `DEQUANTIZE` | `[45]` | 500 | `{}` |
| 150 | `DEQUANTIZE` | `[179]` | 642 | `{}` |
| 151 | `CONV_2D` | `[324, 642, 500]` | 325 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 152 | `ADD` | `[321, 325]` | 326 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 153 | `DEQUANTIZE` | `[46]` | 706 | `{}` |
| 154 | `PRELU` | `[326, 706]` | 327 | `{}` |
| 155 | `DEQUANTIZE` | `[180]` | 535 | `{}` |
| 156 | `DEQUANTIZE` | `[47]` | 595 | `{}` |
| 157 | `CONV_2D` | `[327, 535, 595]` | 328 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 158 | `DEQUANTIZE` | `[48]` | 683 | `{}` |
| 159 | `PRELU` | `[328, 683]` | 329 | `{}` |
| 160 | `DEQUANTIZE` | `[181]` | 620 | `{}` |
| 161 | `DEPTHWISE_CONV_2D` | `[329, 620, 488]` | 330 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 162 | `DEQUANTIZE` | `[182]` | 573 | `{}` |
| 163 | `DEQUANTIZE` | `[49]` | 640 | `{}` |
| 164 | `CONV_2D` | `[330, 573, 640]` | 331 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 165 | `ADD` | `[327, 331]` | 332 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 166 | `DEQUANTIZE` | `[50]` | 524 | `{}` |
| 167 | `PRELU` | `[332, 524]` | 333 | `{}` |
| 168 | `DEQUANTIZE` | `[51]` | 602 | `{}` |
| 169 | `DEQUANTIZE` | `[183]` | 712 | `{}` |
| 170 | `CONV_2D` | `[333, 712, 602]` | 334 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 171 | `DEQUANTIZE` | `[52]` | 505 | `{}` |
| 172 | `PRELU` | `[334, 505]` | 335 | `{}` |
| 173 | `DEQUANTIZE` | `[184]` | 541 | `{}` |
| 174 | `DEPTHWISE_CONV_2D` | `[335, 541, 488]` | 336 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 175 | `DEQUANTIZE` | `[185]` | 692 | `{}` |
| 176 | `DEQUANTIZE` | `[53]` | 709 | `{}` |
| 177 | `CONV_2D` | `[336, 692, 709]` | 337 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 178 | `ADD` | `[333, 337]` | 338 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 179 | `DEQUANTIZE` | `[54]` | 593 | `{}` |
| 180 | `PRELU` | `[338, 593]` | 339 | `{}` |
| 181 | `DEQUANTIZE` | `[186]` | 648 | `{}` |
| 182 | `DEQUANTIZE` | `[55]` | 675 | `{}` |
| 183 | `CONV_2D` | `[339, 648, 675]` | 340 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 184 | `DEQUANTIZE` | `[56]` | 637 | `{}` |
| 185 | `PRELU` | `[340, 637]` | 341 | `{}` |
| 186 | `DEQUANTIZE` | `[187]` | 533 | `{}` |
| 187 | `DEPTHWISE_CONV_2D` | `[341, 533, 488]` | 342 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 188 | `DEQUANTIZE` | `[57]` | 527 | `{}` |
| 189 | `DEQUANTIZE` | `[188]` | 610 | `{}` |
| 190 | `CONV_2D` | `[342, 610, 527]` | 343 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 191 | `ADD` | `[339, 343]` | 344 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 192 | `DEQUANTIZE` | `[58]` | 606 | `{}` |
| 193 | `PRELU` | `[344, 606]` | 345 | `{}` |
| 194 | `DEQUANTIZE` | `[59]` | 493 | `{}` |
| 195 | `DEQUANTIZE` | `[189]` | 503 | `{}` |
| 196 | `CONV_2D` | `[345, 503, 493]` | 346 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 197 | `MAX_POOL_2D` | `[345]` | 347 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 198 | `PAD` | `[347, 253]` | 348 | `{}` |
| 199 | `DEQUANTIZE` | `[60]` | 697 | `{}` |
| 200 | `PRELU` | `[346, 697]` | 349 | `{}` |
| 201 | `DEQUANTIZE` | `[175]` | 571 | `{}` |
| 202 | `DEQUANTIZE` | `[190]` | 714 | `{}` |
| 203 | `DEPTHWISE_CONV_2D` | `[349, 714, 571]` | 350 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 204 | `DEQUANTIZE` | `[191]` | 544 | `{}` |
| 205 | `DEQUANTIZE` | `[61]` | 598 | `{}` |
| 206 | `CONV_2D` | `[350, 544, 598]` | 351 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 207 | `ADD` | `[348, 351]` | 352 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 208 | `DEQUANTIZE` | `[62]` | 678 | `{}` |
| 209 | `PRELU` | `[352, 678]` | 353 | `{}` |
| 210 | `DEQUANTIZE` | `[63]` | 563 | `{}` |
| 211 | `DEQUANTIZE` | `[192]` | 681 | `{}` |
| 212 | `CONV_2D` | `[353, 681, 563]` | 354 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 213 | `DEQUANTIZE` | `[64]` | 519 | `{}` |
| 214 | `PRELU` | `[354, 519]` | 355 | `{}` |
| 215 | `DEQUANTIZE` | `[193]` | 635 | `{}` |
| 216 | `DEPTHWISE_CONV_2D` | `[355, 635, 571]` | 356 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 217 | `DEQUANTIZE` | `[194]` | 537 | `{}` |
| 218 | `DEQUANTIZE` | `[65]` | 604 | `{}` |
| 219 | `CONV_2D` | `[356, 537, 604]` | 357 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 220 | `ADD` | `[353, 357]` | 358 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 221 | `DEQUANTIZE` | `[66]` | 498 | `{}` |
| 222 | `PRELU` | `[358, 498]` | 359 | `{}` |
| 223 | `DEQUANTIZE` | `[195]` | 614 | `{}` |
| 224 | `DEQUANTIZE` | `[67]` | 701 | `{}` |
| 225 | `CONV_2D` | `[359, 614, 701]` | 360 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 226 | `DEQUANTIZE` | `[68]` | 585 | `{}` |
| 227 | `PRELU` | `[360, 585]` | 361 | `{}` |
| 228 | `DEQUANTIZE` | `[196]` | 501 | `{}` |
| 229 | `DEPTHWISE_CONV_2D` | `[361, 501, 571]` | 362 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 230 | `DEQUANTIZE` | `[69]` | 668 | `{}` |
| 231 | `DEQUANTIZE` | `[197]` | 707 | `{}` |
| 232 | `CONV_2D` | `[362, 707, 668]` | 363 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 233 | `ADD` | `[359, 363]` | 364 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 234 | `DEQUANTIZE` | `[70]` | 566 | `{}` |
| 235 | `PRELU` | `[364, 566]` | 365 | `{}` |
| 236 | `DEQUANTIZE` | `[71]` | 522 | `{}` |
| 237 | `DEQUANTIZE` | `[198]` | 596 | `{}` |
| 238 | `CONV_2D` | `[365, 596, 522]` | 366 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 239 | `DEQUANTIZE` | `[72]` | 659 | `{}` |
| 240 | `PRELU` | `[366, 659]` | 367 | `{}` |
| 241 | `DEQUANTIZE` | `[199]` | 684 | `{}` |
| 242 | `DEPTHWISE_CONV_2D` | `[367, 684, 571]` | 368 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 243 | `DEQUANTIZE` | `[73]` | 486 | `{}` |
| 244 | `DEQUANTIZE` | `[200]` | 639 | `{}` |
| 245 | `CONV_2D` | `[368, 639, 486]` | 369 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 246 | `ADD` | `[365, 369]` | 370 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 247 | `DEQUANTIZE` | `[74]` | 699 | `{}` |
| 248 | `PRELU` | `[370, 699]` | 371 | `{}` |
| 249 | `DEQUANTIZE` | `[201]` | 525 | `{}` |
| 250 | `DEQUANTIZE` | `[75]` | 591 | `{}` |
| 251 | `CONV_2D` | `[371, 525, 591]` | 372 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 252 | `DEQUANTIZE` | `[76]` | 666 | `{}` |
| 253 | `PRELU` | `[372, 666]` | 373 | `{}` |
| 254 | `DEQUANTIZE` | `[202]` | 603 | `{}` |
| 255 | `DEPTHWISE_CONV_2D` | `[373, 603, 571]` | 374 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 256 | `DEQUANTIZE` | `[203]` | 506 | `{}` |
| 257 | `DEQUANTIZE` | `[77]` | 558 | `{}` |
| 258 | `CONV_2D` | `[374, 506, 558]` | 375 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 259 | `ADD` | `[371, 375]` | 376 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 260 | `DEQUANTIZE` | `[78]` | 510 | `{}` |
| 261 | `PRELU` | `[376, 510]` | 377 | `{}` |
| 262 | `DEQUANTIZE` | `[79]` | 662 | `{}` |
| 263 | `DEQUANTIZE` | `[204]` | 710 | `{}` |
| 264 | `CONV_2D` | `[377, 710, 662]` | 378 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 265 | `MAX_POOL_2D` | `[377]` | 379 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 266 | `DEQUANTIZE` | `[80]` | 491 | `{}` |
| 267 | `PRELU` | `[378, 491]` | 380 | `{}` |
| 268 | `DEQUANTIZE` | `[205]` | 594 | `{}` |
| 269 | `DEPTHWISE_CONV_2D` | `[380, 594, 571]` | 381 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 270 | `DEQUANTIZE` | `[81]` | 623 | `{}` |
| 271 | `DEQUANTIZE` | `[206]` | 676 | `{}` |
| 272 | `CONV_2D` | `[381, 676, 623]` | 382 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 273 | `ADD` | `[379, 382]` | 383 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 274 | `DEQUANTIZE` | `[82]` | 577 | `{}` |
| 275 | `PRELU` | `[383, 577]` | 384 | `{}` |
| 276 | `DEQUANTIZE` | `[207]` | 638 | `{}` |
| 277 | `DEQUANTIZE` | `[83]` | 670 | `{}` |
| 278 | `CONV_2D` | `[384, 638, 670]` | 385 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 279 | `DEQUANTIZE` | `[84]` | 561 | `{}` |
| 280 | `PRELU` | `[385, 561]` | 386 | `{}` |
| 281 | `DEQUANTIZE` | `[208]` | 528 | `{}` |
| 282 | `DEPTHWISE_CONV_2D` | `[386, 528, 571]` | 387 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 283 | `DEQUANTIZE` | `[85]` | 508 | `{}` |
| 284 | `DEQUANTIZE` | `[209]` | 607 | `{}` |
| 285 | `CONV_2D` | `[387, 607, 508]` | 388 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 286 | `ADD` | `[384, 388]` | 389 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 287 | `DEQUANTIZE` | `[86]` | 653 | `{}` |
| 288 | `PRELU` | `[389, 653]` | 390 | `{}` |
| 289 | `DEQUANTIZE` | `[87]` | 489 | `{}` |
| 290 | `DEQUANTIZE` | `[210]` | 494 | `{}` |
| 291 | `CONV_2D` | `[390, 494, 489]` | 391 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 292 | `DEQUANTIZE` | `[88]` | 627 | `{}` |
| 293 | `PRELU` | `[391, 627]` | 392 | `{}` |
| 294 | `DEQUANTIZE` | `[211]` | 698 | `{}` |
| 295 | `DEPTHWISE_CONV_2D` | `[392, 698, 571]` | 393 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 296 | `DEQUANTIZE` | `[89]` | 583 | `{}` |
| 297 | `DEQUANTIZE` | `[212]` | 599 | `{}` |
| 298 | `CONV_2D` | `[393, 599, 583]` | 394 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 299 | `ADD` | `[390, 394]` | 395 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 300 | `DEQUANTIZE` | `[90]` | 717 | `{}` |
| 301 | `PRELU` | `[395, 717]` | 396 | `{}` |
| 302 | `DEQUANTIZE` | `[91]` | 547 | `{}` |
| 303 | `DEQUANTIZE` | `[213]` | 679 | `{}` |
| 304 | `CONV_2D` | `[396, 679, 547]` | 397 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 305 | `DEQUANTIZE` | `[92]` | 513 | `{}` |
| 306 | `PRELU` | `[397, 513]` | 398 | `{}` |
| 307 | `DEQUANTIZE` | `[214]` | 564 | `{}` |
| 308 | `DEPTHWISE_CONV_2D` | `[398, 564, 571]` | 399 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 309 | `DEQUANTIZE` | `[215]` | 521 | `{}` |
| 310 | `DEQUANTIZE` | `[93]` | 657 | `{}` |
| 311 | `CONV_2D` | `[399, 521, 657]` | 400 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 312 | `ADD` | `[396, 400]` | 401 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 313 | `DEQUANTIZE` | `[94]` | 476 | `{}` |
| 314 | `PRELU` | `[401, 476]` | 402 | `{}` |
| 315 | `DEQUANTIZE` | `[216]` | 605 | `{}` |
| 316 | `DEQUANTIZE` | `[95]` | 618 | `{}` |
| 317 | `CONV_2D` | `[402, 605, 618]` | 403 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 318 | `DEQUANTIZE` | `[96]` | 581 | `{}` |
| 319 | `PRELU` | `[403, 581]` | 404 | `{}` |
| 320 | `DEQUANTIZE` | `[217]` | 499 | `{}` |
| 321 | `DEPTHWISE_CONV_2D` | `[404, 499, 571]` | 405 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 322 | `DEQUANTIZE` | `[218]` | 703 | `{}` |
| 323 | `DEQUANTIZE` | `[97]` | 726 | `{}` |
| 324 | `CONV_2D` | `[405, 703, 726]` | 406 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 325 | `ADD` | `[402, 406]` | 407 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 326 | `DEQUANTIZE` | `[98]` | 550 | `{}` |
| 327 | `PRELU` | `[407, 550]` | 408 | `{}` |
| 328 | `DEQUANTIZE` | `[219]` | 587 | `{}` |
| 329 | `DEQUANTIZE` | `[99]` | 687 | `{}` |
| 330 | `CONV_2D` | `[408, 587, 687]` | 409 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 331 | `MAX_POOL_2D` | `[408]` | 410 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 332 | `DEQUANTIZE` | `[100]` | 644 | `{}` |
| 333 | `PRELU` | `[409, 644]` | 411 | `{}` |
| 334 | `DEQUANTIZE` | `[220]` | 669 | `{}` |
| 335 | `DEPTHWISE_CONV_2D` | `[411, 669, 571]` | 412 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 336 | `DEQUANTIZE` | `[101]` | 481 | `{}` |
| 337 | `DEQUANTIZE` | `[221]` | 567 | `{}` |
| 338 | `CONV_2D` | `[412, 567, 481]` | 413 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 339 | `ADD` | `[410, 413]` | 414 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 340 | `DEQUANTIZE` | `[102]` | 622 | `{}` |
| 341 | `PRELU` | `[414, 622]` | 415 | `{}` |
| 342 | `DEQUANTIZE` | `[222]` | 523 | `{}` |
| 343 | `DEQUANTIZE` | `[103]` | 569 | `{}` |
| 344 | `CONV_2D` | `[415, 523, 569]` | 416 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 345 | `DEQUANTIZE` | `[104]` | 713 | `{}` |
| 346 | `PRELU` | `[416, 713]` | 417 | `{}` |
| 347 | `DEQUANTIZE` | `[223]` | 660 | `{}` |
| 348 | `DEPTHWISE_CONV_2D` | `[417, 660, 571]` | 418 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 349 | `DEQUANTIZE` | `[224]` | 490 | `{}` |
| 350 | `DEQUANTIZE` | `[105]` | 549 | `{}` |
| 351 | `CONV_2D` | `[418, 490, 549]` | 419 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 352 | `ADD` | `[415, 419]` | 420 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 353 | `DEQUANTIZE` | `[106]` | 695 | `{}` |
| 354 | `PRELU` | `[420, 695]` | 421 | `{}` |
| 355 | `DEQUANTIZE` | `[107]` | 643 | `{}` |
| 356 | `DEQUANTIZE` | `[225]` | 700 | `{}` |
| 357 | `CONV_2D` | `[421, 700, 643]` | 422 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 358 | `DEQUANTIZE` | `[108]` | 531 | `{}` |
| 359 | `PRELU` | `[422, 531]` | 423 | `{}` |
| 360 | `DEQUANTIZE` | `[226]` | 592 | `{}` |
| 361 | `DEPTHWISE_CONV_2D` | `[423, 592, 571]` | 424 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 362 | `DEQUANTIZE` | `[109]` | 612 | `{}` |
| 363 | `DEQUANTIZE` | `[227]` | 667 | `{}` |
| 364 | `CONV_2D` | `[424, 667, 612]` | 425 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 365 | `ADD` | `[421, 425]` | 426 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 366 | `DEQUANTIZE` | `[110]` | 572 | `{}` |
| 367 | `PRELU` | `[426, 572]` | 427 | `{}` |
| 368 | `DEQUANTIZE` | `[228]` | 554 | `{}` |
| 369 | `DEQUANTIZE` | `[111]` | 716 | `{}` |
| 370 | `CONV_2D` | `[427, 554, 716]` | 428 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 371 | `DEQUANTIZE` | `[112]` | 539 | `{}` |
| 372 | `PRELU` | `[428, 539]` | 429 | `{}` |
| 373 | `DEQUANTIZE` | `[229]` | 514 | `{}` |
| 374 | `DEPTHWISE_CONV_2D` | `[429, 514, 571]` | 430 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 375 | `DEQUANTIZE` | `[230]` | 663 | `{}` |
| 376 | `DEQUANTIZE` | `[113]` | 682 | `{}` |
| 377 | `CONV_2D` | `[430, 663, 682]` | 431 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 378 | `ADD` | `[427, 431]` | 432 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 379 | `DEQUANTIZE` | `[114]` | 646 | `{}` |
| 380 | `PRELU` | `[432, 646]` | 433 | `{}` |
| 381 | `DEQUANTIZE` | `[231]` | 492 | `{}` |
| 382 | `DEQUANTIZE` | `[115]` | 538 | `{}` |
| 383 | `CONV_2D` | `[433, 492, 538]` | 434 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 384 | `DEQUANTIZE` | `[116]` | 611 | `{}` |
| 385 | `PRELU` | `[434, 611]` | 435 | `{}` |
| 386 | `DEQUANTIZE` | `[232]` | 624 | `{}` |
| 387 | `DEPTHWISE_CONV_2D` | `[435, 624, 571]` | 436 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 388 | `DEQUANTIZE` | `[117]` | 504 | `{}` |
| 389 | `DEQUANTIZE` | `[233]` | 578 | `{}` |
| 390 | `CONV_2D` | `[436, 578, 504]` | 437 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 391 | `ADD` | `[433, 437]` | 438 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 392 | `DEQUANTIZE` | `[118]` | 705 | `{}` |
| 393 | `PRELU` | `[438, 705]` | 439 | `{}` |
| 394 | `DEQUANTIZE` | `[119]` | 542 | `{}` |
| 395 | `DEQUANTIZE` | `[234]` | 671 | `{}` |
| 396 | `CONV_2D` | `[439, 671, 542]` | 440 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 2, "stride_w": 2}` |
| 397 | `MAX_POOL_2D` | `[439]` | 441 | `{"filter_height": 2, "filter_width": 2, "fused_activation_function": "NONE", "padding": "VALID", "stride_h": 2, "stride_w": 2}` |
| 398 | `DEQUANTIZE` | `[120]` | 686 | `{}` |
| 399 | `PRELU` | `[440, 686]` | 442 | `{}` |
| 400 | `DEQUANTIZE` | `[235]` | 562 | `{}` |
| 401 | `DEPTHWISE_CONV_2D` | `[442, 562, 571]` | 443 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 402 | `DEQUANTIZE` | `[236]` | 509 | `{}` |
| 403 | `DEQUANTIZE` | `[121]` | 633 | `{}` |
| 404 | `CONV_2D` | `[443, 509, 633]` | 444 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 405 | `ADD` | `[441, 444]` | 445 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 406 | `DEQUANTIZE` | `[122]` | 526 | `{}` |
| 407 | `PRELU` | `[445, 526]` | 446 | `{}` |
| 408 | `DEQUANTIZE` | `[123]` | 613 | `{}` |
| 409 | `DEQUANTIZE` | `[237]` | 654 | `{}` |
| 410 | `CONV_2D` | `[446, 654, 613]` | 447 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 411 | `DEQUANTIZE` | `[124]` | 507 | `{}` |
| 412 | `PRELU` | `[447, 507]` | 448 | `{}` |
| 413 | `DEQUANTIZE` | `[238]` | 478 | `{}` |
| 414 | `DEPTHWISE_CONV_2D` | `[448, 478, 571]` | 449 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 415 | `DEQUANTIZE` | `[239]` | 632 | `{}` |
| 416 | `DEQUANTIZE` | `[125]` | 704 | `{}` |
| 417 | `CONV_2D` | `[449, 632, 704]` | 450 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 418 | `ADD` | `[446, 450]` | 451 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 419 | `DEQUANTIZE` | `[126]` | 597 | `{}` |
| 420 | `PRELU` | `[451, 597]` | 452 | `{}` |
| 421 | `DEQUANTIZE` | `[240]` | 584 | `{}` |
| 422 | `DEQUANTIZE` | `[127]` | 685 | `{}` |
| 423 | `CONV_2D` | `[452, 584, 685]` | 453 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 424 | `DEQUANTIZE` | `[128]` | 636 | `{}` |
| 425 | `PRELU` | `[453, 636]` | 454 | `{}` |
| 426 | `DEQUANTIZE` | `[241]` | 718 | `{}` |
| 427 | `DEPTHWISE_CONV_2D` | `[454, 718, 571]` | 455 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 428 | `DEQUANTIZE` | `[129]` | 530 | `{}` |
| 429 | `DEQUANTIZE` | `[242]` | 548 | `{}` |
| 430 | `CONV_2D` | `[455, 548, 530]` | 456 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 431 | `ADD` | `[452, 456]` | 457 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 432 | `DEQUANTIZE` | `[130]` | 601 | `{}` |
| 433 | `PRELU` | `[457, 601]` | 458 | `{}` |
| 434 | `DEQUANTIZE` | `[131]` | 495 | `{}` |
| 435 | `DEQUANTIZE` | `[243]` | 515 | `{}` |
| 436 | `CONV_2D` | `[458, 515, 495]` | 459 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 437 | `DEQUANTIZE` | `[132]` | 708 | `{}` |
| 438 | `PRELU` | `[459, 708]` | 460 | `{}` |
| 439 | `DEQUANTIZE` | `[244]` | 658 | `{}` |
| 440 | `DEPTHWISE_CONV_2D` | `[460, 658, 571]` | 461 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 441 | `DEQUANTIZE` | `[245]` | 477 | `{}` |
| 442 | `DEQUANTIZE` | `[133]` | 600 | `{}` |
| 443 | `CONV_2D` | `[461, 477, 600]` | 462 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 444 | `ADD` | `[458, 462]` | 463 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 445 | `DEQUANTIZE` | `[134]` | 674 | `{}` |
| 446 | `PRELU` | `[463, 674]` | 464 | `{}` |
| 447 | `DEQUANTIZE` | `[135]` | 565 | `{}` |
| 448 | `DEQUANTIZE` | `[246]` | 619 | `{}` |
| 449 | `CONV_2D` | `[464, 619, 565]` | 465 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 450 | `DEQUANTIZE` | `[136]` | 529 | `{}` |
| 451 | `PRELU` | `[465, 529]` | 466 | `{}` |
| 452 | `DEQUANTIZE` | `[247]` | 582 | `{}` |
| 453 | `DEPTHWISE_CONV_2D` | `[466, 582, 571]` | 467 | `{"depth_multiplier": 1, "dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "SAME", "stride_h": 1, "stride_w": 1}` |
| 454 | `DEQUANTIZE` | `[137]` | 608 | `{}` |
| 455 | `DEQUANTIZE` | `[248]` | 721 | `{}` |
| 456 | `CONV_2D` | `[467, 721, 608]` | 468 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 457 | `ADD` | `[464, 468]` | 469 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 458 | `DEQUANTIZE` | `[138]` | 497 | `{}` |
| 459 | `PRELU` | `[469, 497]` | 470 | `{}` |
| 460 | `DEQUANTIZE` | `[249]` | 553 | `{}` |
| 461 | `DEQUANTIZE` | `[139]` | 696 | `{}` |
| 462 | `CONV_2D` | `[470, 553, 696]` | 471 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 463 | `DEQUANTIZE` | `[140]` | 588 | `{}` |
| 464 | `DEQUANTIZE` | `[250]` | 688 | `{}` |
| 465 | `CONV_2D` | `[470, 688, 588]` | 472 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 466 | `DEQUANTIZE` | `[251]` | 645 | `{}` |
| 467 | `DEQUANTIZE` | `[141]` | 677 | `{}` |
| 468 | `CONV_2D` | `[470, 645, 677]` | 473 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 469 | `LOGISTIC` | `[471]` | 474 | `{}` |
| 470 | `RESHAPE` | `[474, 252]` | 475 | `{}` |

### Blendshapes — `face_blendshapes.tflite`

- Schema version: `3`; subgraph: `main`.
- Tensors: `245`; nodes: `182`.
- Inputs: `[0]`; outputs: `[195]`.
- Dtypes: `{"FLOAT16": 49, "FLOAT32": 183, "INT32": 13}`.
- Corrected operator inventory: `{"ADD": 24, "CONCATENATION": 1, "CONV_2D": 19, "DEQUANTIZE": 49, "DIV": 1, "LOGISTIC": 1, "MEAN": 18, "MUL": 26, "NEG": 8, "RESHAPE": 2, "RSQRT": 8, "SQRT": 1, "SQUARED_DIFFERENCE": 8, "STRIDED_SLICE": 4, "SUB": 1, "SUM": 1, "TRANSPOSE": 10}`.

#### Operator codes

| opcode table index | schema builtin code | name | version | custom code |
| ---: | ---: | --- | ---: | --- |
| 0 | 40 | `MEAN` | 1 | `` |
| 1 | 41 | `SUB` | 1 | `` |
| 2 | 18 | `MUL` | 1 | `` |
| 3 | 74 | `SUM` | 1 | `` |
| 4 | 75 | `SQRT` | 1 | `` |
| 5 | 42 | `DIV` | 1 | `` |
| 6 | 22 | `RESHAPE` | 1 | `` |
| 7 | 45 | `STRIDED_SLICE` | 1 | `` |
| 8 | 39 | `TRANSPOSE` | 1 | `` |
| 9 | 3 | `CONV_2D` | 1 | `` |
| 10 | 2 | `CONCATENATION` | 1 | `` |
| 11 | 59 | `NEG` | 1 | `` |
| 12 | 99 | `SQUARED_DIFFERENCE` | 1 | `` |
| 13 | 0 | `ADD` | 1 | `` |
| 14 | 76 | `RSQRT` | 1 | `` |
| 15 | 14 | `LOGISTIC` | 1 | `` |
| 16 | 6 | `DEQUANTIZE` | 2 | `` |

#### Tensors

| index | name | dtype | shape | buffer index | absolute offset | bytes |
| ---: | --- | --- | --- | ---: | ---: | ---: |
| 0 | `serving_default_input_points:0` | `FLOAT32` | `[1, 146, 2]` | 1 | None | 0 |
| 1 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[384]` | 2 | 3708144 | 768 |
| 2 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[256]` | 3 | 3707616 | 512 |
| 3 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[384]` | 4 | 3706832 | 768 |
| 4 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[256]` | 5 | 3706304 | 512 |
| 5 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[384]` | 6 | 3705520 | 768 |
| 6 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[256]` | 7 | 3704992 | 512 |
| 7 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[384]` | 8 | 3704208 | 768 |
| 8 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT16` | `[256]` | 9 | 3703680 | 512 |
| 9 | `model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/BiasAdd/ReadVariableOp` | `FLOAT16` | `[96]` | 10 | 3703472 | 192 |
| 10 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/BiasAdd/ReadVariableOp` | `FLOAT16` | `[64]` | 11 | 3703328 | 128 |
| 11 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[97]` | 12 | 3703120 | 194 |
| 12 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[64]` | 13 | 3702976 | 128 |
| 13 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[97]` | 14 | 3702768 | 194 |
| 14 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[64]` | 15 | 3702624 | 128 |
| 15 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[97]` | 16 | 3702416 | 194 |
| 16 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[64]` | 17 | 3702272 | 128 |
| 17 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[97]` | 18 | 3702064 | 194 |
| 18 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT16` | `[64]` | 19 | 3701920 | 128 |
| 19 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/BiasAdd/ReadVariableOp` | `FLOAT16` | `[52]` | 20 | 3701792 | 104 |
| 20 | `model_1/tf.__operators__.getitem_2/strided_slice` | `INT32` | `[4]` | 21 | 3701760 | 16 |
| 21 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem_1/strided_slice` | `INT32` | `[4]` | 22 | 3701728 | 16 |
| 22 | `model_1/tf.__operators__.getitem_2/strided_slice1` | `INT32` | `[4]` | 23 | 3701696 | 16 |
| 23 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem_1/strided_slice1` | `INT32` | `[4]` | 24 | 3701664 | 16 |
| 24 | `model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/Conv2D` | `FLOAT16` | `[96, 1, 1, 146]` | 25 | 3673616 | 28032 |
| 25 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/Conv2D` | `FLOAT16` | `[64, 1, 1, 2]` | 26 | 3673344 | 256 |
| 26 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[384, 1, 1, 97]` | 27 | 3598832 | 74496 |
| 27 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[97, 1, 1, 384]` | 28 | 3524320 | 74496 |
| 28 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[256, 1, 1, 64]` | 29 | 3491536 | 32768 |
| 29 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[64, 1, 1, 256]` | 30 | 3458752 | 32768 |
| 30 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[384, 1, 1, 97]` | 31 | 3384240 | 74496 |
| 31 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[97, 1, 1, 384]` | 32 | 3309728 | 74496 |
| 32 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[256, 1, 1, 64]` | 33 | 3276944 | 32768 |
| 33 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[64, 1, 1, 256]` | 34 | 3244160 | 32768 |
| 34 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[384, 1, 1, 97]` | 35 | 3169648 | 74496 |
| 35 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[97, 1, 1, 384]` | 36 | 3095136 | 74496 |
| 36 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[256, 1, 1, 64]` | 37 | 3062352 | 32768 |
| 37 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[64, 1, 1, 256]` | 38 | 3029568 | 32768 |
| 38 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[384, 1, 1, 97]` | 39 | 2955056 | 74496 |
| 39 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[97, 1, 1, 384]` | 40 | 2880544 | 74496 |
| 40 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D` | `FLOAT16` | `[256, 1, 1, 64]` | 41 | 2847760 | 32768 |
| 41 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D` | `FLOAT16` | `[64, 1, 1, 256]` | 42 | 2814976 | 32768 |
| 42 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem/strided_slice` | `INT32` | `[4]` | 43 | 2814944 | 16 |
| 43 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem_1/strided_slice2` | `INT32` | `[4]` | 44 | 2814912 | 16 |
| 44 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/Conv2D` | `FLOAT16` | `[52, 1, 1, 64]` | 45 | 2808240 | 6656 |
| 45 | `model_1/tf.__operators__.getitem_3/strided_slice` | `INT32` | `[2]` | 46 | 2808208 | 8 |
| 46 | `model_1/GhumMarkerPoserMlpMixerGeneral/reshape_7/Reshape/shape` | `INT32` | `[2]` | 47 | 2808176 | 8 |
| 47 | `model_1/tf.__operators__.getitem_3/strided_slice1` | `INT32` | `[2]` | 48 | 2808144 | 8 |
| 48 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/transpose/perm` | `INT32` | `[4]` | 49 | 2808112 | 16 |
| 49 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/AddExtraTokens/strided_slice_1` | `FLOAT16` | `[1, 1, 1, 64]` | 50 | 2807968 | 128 |
| 50 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 51 | 2807824 | 128 |
| 51 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 52 | 2807680 | 128 |
| 52 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 53 | 2807536 | 128 |
| 53 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 54 | 2807392 | 128 |
| 54 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 55 | 2807248 | 128 |
| 55 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 56 | 2807104 | 128 |
| 56 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 57 | 2806960 | 128 |
| 57 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/mul/ReadVariableOp` | `FLOAT16` | `[64]` | 58 | 2806816 | 128 |
| 58 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv;model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv/y` | `FLOAT16` | `[]` | 59 | 2806800 | 2 |
| 59 | `model_1/tf.compat.v1.norm/norm/Sum/reduction_indices` | `INT32` | `[1]` | 60 | 2806784 | 4 |
| 60 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/moments/mean/reduction_indices` | `INT32` | `[1]` | 61 | 2806768 | 4 |
| 61 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/add/y` | `FLOAT16` | `[]` | 62 | 2806752 | 2 |
| 62 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/AddExtraTokens/concat/axis` | `INT32` | `[]` | 63 | 2806736 | 4 |
| 63 | `model_1/tf.math.reduce_mean/Mean` | `FLOAT32` | `[1, 1, 2]` | 64 | None | 0 |
| 64 | `model_1/tf.math.subtract/Sub` | `FLOAT32` | `[1, 146, 2]` | 65 | None | 0 |
| 65 | `model_1/tf.compat.v1.norm/norm/mul` | `FLOAT32` | `[1, 146, 2]` | 66 | None | 0 |
| 66 | `model_1/tf.compat.v1.norm/norm/Sum` | `FLOAT32` | `[1, 146, 1]` | 67 | None | 0 |
| 67 | `model_1/tf.compat.v1.norm/norm/Sqrt` | `FLOAT32` | `[1, 146, 1]` | 68 | None | 0 |
| 68 | `model_1/tf.math.reduce_mean_1/Mean` | `FLOAT32` | `[1, 1, 1]` | 69 | None | 0 |
| 69 | `model_1/tf.math.truediv_1/truediv` | `FLOAT32` | `[1, 146, 2]` | 70 | None | 0 |
| 70 | `model_1/tf.__operators__.getitem_2/strided_slice2` | `FLOAT32` | `[1, 1, 146, 2]` | 71 | None | 0 |
| 71 | `model_1/tf.__operators__.getitem_2/strided_slice3` | `FLOAT32` | `[1, 1, 146, 2]` | 72 | None | 0 |
| 72 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv;model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv/y1` | `FLOAT32` | `[1, 1, 146, 2]` | 73 | None | 0 |
| 73 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.compat.v1.transpose/transpose` | `FLOAT32` | `[1, 1, 2, 146]` | 74 | None | 0 |
| 74 | `model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 2, 96]` | 75 | None | 0 |
| 75 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.compat.v1.transpose_1/transpose` | `FLOAT32` | `[1, 1, 96, 2]` | 76 | None | 0 |
| 76 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 96, 64]` | 77 | None | 0 |
| 77 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/AddExtraTokens/concat` | `FLOAT32` | `[1, 1, 97, 64]` | 78 | None | 0 |
| 78 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 79 | None | 0 |
| 79 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 80 | None | 0 |
| 80 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 81 | None | 0 |
| 81 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 82 | None | 0 |
| 82 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 83 | None | 0 |
| 83 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 84 | None | 0 |
| 84 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 85 | None | 0 |
| 85 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 86 | None | 0 |
| 86 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 87 | None | 0 |
| 87 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 88 | None | 0 |
| 88 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/add_11` | `FLOAT32` | `[1, 1, 64, 97]` | 89 | None | 0 |
| 89 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 384]` | 90 | None | 0 |
| 90 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 97]` | 91 | None | 0 |
| 91 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/transpose_1` | `FLOAT32` | `[1, 1, 97, 64]` | 92 | None | 0 |
| 92 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/residual_tokens/add` | `FLOAT32` | `[1, 1, 97, 64]` | 93 | None | 0 |
| 93 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 94 | None | 0 |
| 94 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 95 | None | 0 |
| 95 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 96 | None | 0 |
| 96 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 97 | None | 0 |
| 97 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 98 | None | 0 |
| 98 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 99 | None | 0 |
| 99 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 100 | None | 0 |
| 100 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 101 | None | 0 |
| 101 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 102 | None | 0 |
| 102 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 103 | None | 0 |
| 103 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 256]` | 104 | None | 0 |
| 104 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 64]` | 105 | None | 0 |
| 105 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/residual_channels/add` | `FLOAT32` | `[1, 1, 97, 64]` | 106 | None | 0 |
| 106 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 107 | None | 0 |
| 107 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 108 | None | 0 |
| 108 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 109 | None | 0 |
| 109 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 110 | None | 0 |
| 110 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 111 | None | 0 |
| 111 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 112 | None | 0 |
| 112 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 113 | None | 0 |
| 113 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 114 | None | 0 |
| 114 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 115 | None | 0 |
| 115 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 116 | None | 0 |
| 116 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/add_11` | `FLOAT32` | `[1, 1, 64, 97]` | 117 | None | 0 |
| 117 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 384]` | 118 | None | 0 |
| 118 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 97]` | 119 | None | 0 |
| 119 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/transpose_1` | `FLOAT32` | `[1, 1, 97, 64]` | 120 | None | 0 |
| 120 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/residual_tokens/add` | `FLOAT32` | `[1, 1, 97, 64]` | 121 | None | 0 |
| 121 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 122 | None | 0 |
| 122 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 123 | None | 0 |
| 123 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 124 | None | 0 |
| 124 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 125 | None | 0 |
| 125 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 126 | None | 0 |
| 126 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 127 | None | 0 |
| 127 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 128 | None | 0 |
| 128 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 129 | None | 0 |
| 129 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 130 | None | 0 |
| 130 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 131 | None | 0 |
| 131 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 256]` | 132 | None | 0 |
| 132 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 64]` | 133 | None | 0 |
| 133 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/residual_channels/add` | `FLOAT32` | `[1, 1, 97, 64]` | 134 | None | 0 |
| 134 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 135 | None | 0 |
| 135 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 136 | None | 0 |
| 136 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 137 | None | 0 |
| 137 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 138 | None | 0 |
| 138 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 139 | None | 0 |
| 139 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 140 | None | 0 |
| 140 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 141 | None | 0 |
| 141 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 142 | None | 0 |
| 142 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 143 | None | 0 |
| 143 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 144 | None | 0 |
| 144 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/add_11` | `FLOAT32` | `[1, 1, 64, 97]` | 145 | None | 0 |
| 145 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 384]` | 146 | None | 0 |
| 146 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 97]` | 147 | None | 0 |
| 147 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/transpose_1` | `FLOAT32` | `[1, 1, 97, 64]` | 148 | None | 0 |
| 148 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/residual_tokens/add` | `FLOAT32` | `[1, 1, 97, 64]` | 149 | None | 0 |
| 149 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 150 | None | 0 |
| 150 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 151 | None | 0 |
| 151 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 152 | None | 0 |
| 152 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 153 | None | 0 |
| 153 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 154 | None | 0 |
| 154 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 155 | None | 0 |
| 155 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 156 | None | 0 |
| 156 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 157 | None | 0 |
| 157 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 158 | None | 0 |
| 158 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 159 | None | 0 |
| 159 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 256]` | 160 | None | 0 |
| 160 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 64]` | 161 | None | 0 |
| 161 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/residual_channels/add` | `FLOAT32` | `[1, 1, 97, 64]` | 162 | None | 0 |
| 162 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 163 | None | 0 |
| 163 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 164 | None | 0 |
| 164 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 165 | None | 0 |
| 165 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 166 | None | 0 |
| 166 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 167 | None | 0 |
| 167 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 168 | None | 0 |
| 168 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 169 | None | 0 |
| 169 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 170 | None | 0 |
| 170 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 171 | None | 0 |
| 171 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 172 | None | 0 |
| 172 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/add_11` | `FLOAT32` | `[1, 1, 64, 97]` | 173 | None | 0 |
| 173 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 384]` | 174 | None | 0 |
| 174 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 64, 97]` | 175 | None | 0 |
| 175 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/transpose_1` | `FLOAT32` | `[1, 1, 97, 64]` | 176 | None | 0 |
| 176 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/residual_tokens/add` | `FLOAT32` | `[1, 1, 97, 64]` | 177 | None | 0 |
| 177 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/moments/mean` | `FLOAT32` | `[1, 1, 97, 1]` | 178 | None | 0 |
| 178 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/Neg` | `FLOAT32` | `[1, 1, 97, 1]` | 179 | None | 0 |
| 179 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/moments/SquaredDifference` | `FLOAT32` | `[1, 1, 97, 64]` | 180 | None | 0 |
| 180 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/moments/variance` | `FLOAT32` | `[1, 1, 97, 1]` | 181 | None | 0 |
| 181 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/add` | `FLOAT32` | `[1, 1, 97, 1]` | 182 | None | 0 |
| 182 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/Rsqrt` | `FLOAT32` | `[1, 1, 97, 1]` | 183 | None | 0 |
| 183 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/mul` | `FLOAT32` | `[1, 1, 97, 64]` | 184 | None | 0 |
| 184 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/mul_1` | `FLOAT32` | `[1, 1, 97, 64]` | 185 | None | 0 |
| 185 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/mul_2` | `FLOAT32` | `[1, 1, 97, 64]` | 186 | None | 0 |
| 186 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/add_1` | `FLOAT32` | `[1, 1, 97, 64]` | 187 | None | 0 |
| 187 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Relu;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 256]` | 188 | None | 0 |
| 188 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 97, 64]` | 189 | None | 0 |
| 189 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/residual_channels/add` | `FLOAT32` | `[1, 1, 97, 64]` | 190 | None | 0 |
| 190 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem/strided_slice1` | `FLOAT32` | `[1, 1, 2, 64]` | 191 | None | 0 |
| 191 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.__operators__.getitem_1/strided_slice3` | `FLOAT32` | `[1, 1, 1, 64]` | 192 | None | 0 |
| 192 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/BiasAdd;model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/Conv2D;model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/BiasAdd/ReadVariableOp` | `FLOAT32` | `[1, 1, 1, 52]` | 193 | None | 0 |
| 193 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/Sigmoid` | `FLOAT32` | `[1, 1, 1, 52]` | 194 | None | 0 |
| 194 | `model_1/GhumMarkerPoserMlpMixerGeneral/reshape_7/Reshape` | `FLOAT32` | `[1, 52]` | 195 | None | 0 |
| 195 | `StatefulPartitionedCall:0` | `FLOAT32` | `[52]` | 196 | None | 0 |
| 196 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 256]` | 0 | None | 0 |
| 197 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[97]` | 0 | None | 0 |
| 198 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[256]` | 0 | None | 0 |
| 199 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[384, 1, 1, 97]` | 0 | None | 0 |
| 200 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/Conv2D_dequantize` | `FLOAT32` | `[52, 1, 1, 64]` | 0 | None | 0 |
| 201 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm2/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 202 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[97, 1, 1, 384]` | 0 | None | 0 |
| 203 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/AddExtraTokens/strided_slice_1_dequantize` | `FLOAT32` | `[1, 1, 1, 64]` | 0 | None | 0 |
| 204 | `model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv;model_1/GhumMarkerPoserMlpMixerGeneral/tf.math.truediv/truediv/y_dequantize` | `FLOAT32` | `[]` | 0 | None | 0 |
| 205 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[256]` | 0 | None | 0 |
| 206 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[97]` | 0 | None | 0 |
| 207 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[97, 1, 1, 384]` | 0 | None | 0 |
| 208 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[256, 1, 1, 64]` | 0 | None | 0 |
| 209 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[256, 1, 1, 64]` | 0 | None | 0 |
| 210 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm2/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 211 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[256, 1, 1, 64]` | 0 | None | 0 |
| 212 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 213 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[384]` | 0 | None | 0 |
| 214 | `model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[96]` | 0 | None | 0 |
| 215 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm2/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 216 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 256]` | 0 | None | 0 |
| 217 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 218 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/add/y_dequantize` | `FLOAT32` | `[]` | 0 | None | 0 |
| 219 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[384, 1, 1, 97]` | 0 | None | 0 |
| 220 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 2]` | 0 | None | 0 |
| 221 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm2/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 222 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 223 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[384]` | 0 | None | 0 |
| 224 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/layer_norm1/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 225 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[256, 1, 1, 64]` | 0 | None | 0 |
| 226 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/input_tokens_embedding/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 227 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[384]` | 0 | None | 0 |
| 228 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 256]` | 0 | None | 0 |
| 229 | `model_1/GhumMarkerPoserMlpMixerGeneral/output_blendweights/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[52]` | 0 | None | 0 |
| 230 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/layer_norm1/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 231 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[97, 1, 1, 384]` | 0 | None | 0 |
| 232 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_channel_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 233 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[384]` | 0 | None | 0 |
| 234 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_token_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[384, 1, 1, 97]` | 0 | None | 0 |
| 235 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/layer_norm1/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 236 | `model_1/GhumMarkerPoserMlpMixerGeneral/conv2d/Conv2D_dequantize` | `FLOAT32` | `[96, 1, 1, 146]` | 0 | None | 0 |
| 237 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_1/mlp_channel_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[64, 1, 1, 256]` | 0 | None | 0 |
| 238 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[97]` | 0 | None | 0 |
| 239 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[256]` | 0 | None | 0 |
| 240 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/layer_norm1/batchnorm/mul/ReadVariableOp_dequantize` | `FLOAT32` | `[64]` | 0 | None | 0 |
| 241 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_0/mlp_token_mixing/Mlp_1/Conv2D_dequantize` | `FLOAT32` | `[384, 1, 1, 97]` | 0 | None | 0 |
| 242 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_2/mlp_token_mixing/Mlp_2/Conv2D_dequantize` | `FLOAT32` | `[97, 1, 1, 384]` | 0 | None | 0 |
| 243 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_token_mixing/Mlp_2/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[97]` | 0 | None | 0 |
| 244 | `model_1/GhumMarkerPoserMlpMixerGeneral/MLPMixer/MixerBlock_3/mlp_channel_mixing/Mlp_1/BiasAdd/ReadVariableOp_dequantize` | `FLOAT32` | `[256]` | 0 | None | 0 |

#### Nodes

| index | op | inputs | output | exact schema-derived options |
| ---: | --- | --- | ---: | --- |
| 0 | `MEAN` | `[0, 62]` | 63 | `{"keep_dims": true}` |
| 1 | `SUB` | `[0, 63]` | 64 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 2 | `MUL` | `[64, 64]` | 65 | `{"fused_activation_function": "NONE"}` |
| 3 | `SUM` | `[65, 59]` | 66 | `{"keep_dims": true}` |
| 4 | `SQRT` | `[66]` | 67 | `{}` |
| 5 | `MEAN` | `[67, 62]` | 68 | `{"keep_dims": true}` |
| 6 | `DIV` | `[64, 68]` | 69 | `{"fused_activation_function": "NONE"}` |
| 7 | `RESHAPE` | `[69, 20]` | 70 | `{}` |
| 8 | `STRIDED_SLICE` | `[70, 21, 22, 23]` | 71 | `{"begin_mask": 15, "ellipsis_mask": 0, "end_mask": 15, "new_axis_mask": 0, "offset": false, "shrink_axis_mask": 0}` |
| 9 | `DEQUANTIZE` | `[58]` | 204 | `{}` |
| 10 | `MUL` | `[71, 204]` | 72 | `{"fused_activation_function": "NONE"}` |
| 11 | `TRANSPOSE` | `[72, 48]` | 73 | `{}` |
| 12 | `DEQUANTIZE` | `[9]` | 214 | `{}` |
| 13 | `DEQUANTIZE` | `[24]` | 236 | `{}` |
| 14 | `CONV_2D` | `[73, 236, 214]` | 74 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 15 | `TRANSPOSE` | `[74, 48]` | 75 | `{}` |
| 16 | `DEQUANTIZE` | `[25]` | 220 | `{}` |
| 17 | `DEQUANTIZE` | `[10]` | 226 | `{}` |
| 18 | `CONV_2D` | `[75, 220, 226]` | 76 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 19 | `DEQUANTIZE` | `[49]` | 203 | `{}` |
| 20 | `CONCATENATION` | `[203, 76]` | 77 | `{"axis": -2, "fused_activation_function": "NONE"}` |
| 21 | `MEAN` | `[77, 60]` | 78 | `{"keep_dims": true}` |
| 22 | `NEG` | `[78]` | 79 | `{}` |
| 23 | `SQUARED_DIFFERENCE` | `[77, 78]` | 80 | `{}` |
| 24 | `MEAN` | `[80, 60]` | 81 | `{"keep_dims": true}` |
| 25 | `DEQUANTIZE` | `[61]` | 218 | `{}` |
| 26 | `ADD` | `[81, 218]` | 82 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 27 | `RSQRT` | `[82]` | 83 | `{}` |
| 28 | `DEQUANTIZE` | `[57]` | 230 | `{}` |
| 29 | `MUL` | `[83, 230]` | 84 | `{"fused_activation_function": "NONE"}` |
| 30 | `MUL` | `[77, 84]` | 85 | `{"fused_activation_function": "NONE"}` |
| 31 | `MUL` | `[79, 84]` | 86 | `{"fused_activation_function": "NONE"}` |
| 32 | `ADD` | `[85, 86]` | 87 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 33 | `TRANSPOSE` | `[87, 48]` | 88 | `{}` |
| 34 | `DEQUANTIZE` | `[1]` | 227 | `{}` |
| 35 | `DEQUANTIZE` | `[26]` | 241 | `{}` |
| 36 | `CONV_2D` | `[88, 241, 227]` | 89 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 37 | `DEQUANTIZE` | `[11]` | 197 | `{}` |
| 38 | `DEQUANTIZE` | `[27]` | 202 | `{}` |
| 39 | `CONV_2D` | `[89, 202, 197]` | 90 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 40 | `TRANSPOSE` | `[90, 48]` | 91 | `{}` |
| 41 | `ADD` | `[91, 77]` | 92 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 42 | `MEAN` | `[92, 60]` | 93 | `{"keep_dims": true}` |
| 43 | `NEG` | `[93]` | 94 | `{}` |
| 44 | `SQUARED_DIFFERENCE` | `[92, 93]` | 95 | `{}` |
| 45 | `MEAN` | `[95, 60]` | 96 | `{"keep_dims": true}` |
| 46 | `ADD` | `[96, 218]` | 97 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 47 | `RSQRT` | `[97]` | 98 | `{}` |
| 48 | `DEQUANTIZE` | `[56]` | 210 | `{}` |
| 49 | `MUL` | `[98, 210]` | 99 | `{"fused_activation_function": "NONE"}` |
| 50 | `MUL` | `[92, 99]` | 100 | `{"fused_activation_function": "NONE"}` |
| 51 | `MUL` | `[94, 99]` | 101 | `{"fused_activation_function": "NONE"}` |
| 52 | `ADD` | `[100, 101]` | 102 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 53 | `DEQUANTIZE` | `[2]` | 198 | `{}` |
| 54 | `DEQUANTIZE` | `[28]` | 225 | `{}` |
| 55 | `CONV_2D` | `[102, 225, 198]` | 103 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 56 | `DEQUANTIZE` | `[29]` | 196 | `{}` |
| 57 | `DEQUANTIZE` | `[12]` | 232 | `{}` |
| 58 | `CONV_2D` | `[103, 196, 232]` | 104 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 59 | `ADD` | `[104, 92]` | 105 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 60 | `MEAN` | `[105, 60]` | 106 | `{"keep_dims": true}` |
| 61 | `NEG` | `[106]` | 107 | `{}` |
| 62 | `SQUARED_DIFFERENCE` | `[105, 106]` | 108 | `{}` |
| 63 | `MEAN` | `[108, 60]` | 109 | `{"keep_dims": true}` |
| 64 | `ADD` | `[109, 218]` | 110 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 65 | `RSQRT` | `[110]` | 111 | `{}` |
| 66 | `DEQUANTIZE` | `[55]` | 224 | `{}` |
| 67 | `MUL` | `[111, 224]` | 112 | `{"fused_activation_function": "NONE"}` |
| 68 | `MUL` | `[105, 112]` | 113 | `{"fused_activation_function": "NONE"}` |
| 69 | `MUL` | `[107, 112]` | 114 | `{"fused_activation_function": "NONE"}` |
| 70 | `ADD` | `[113, 114]` | 115 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 71 | `TRANSPOSE` | `[115, 48]` | 116 | `{}` |
| 72 | `DEQUANTIZE` | `[3]` | 233 | `{}` |
| 73 | `DEQUANTIZE` | `[30]` | 234 | `{}` |
| 74 | `CONV_2D` | `[116, 234, 233]` | 117 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 75 | `DEQUANTIZE` | `[13]` | 206 | `{}` |
| 76 | `DEQUANTIZE` | `[31]` | 207 | `{}` |
| 77 | `CONV_2D` | `[117, 207, 206]` | 118 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 78 | `TRANSPOSE` | `[118, 48]` | 119 | `{}` |
| 79 | `ADD` | `[119, 105]` | 120 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 80 | `MEAN` | `[120, 60]` | 121 | `{"keep_dims": true}` |
| 81 | `NEG` | `[121]` | 122 | `{}` |
| 82 | `SQUARED_DIFFERENCE` | `[120, 121]` | 123 | `{}` |
| 83 | `MEAN` | `[123, 60]` | 124 | `{"keep_dims": true}` |
| 84 | `ADD` | `[124, 218]` | 125 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 85 | `RSQRT` | `[125]` | 126 | `{}` |
| 86 | `DEQUANTIZE` | `[54]` | 201 | `{}` |
| 87 | `MUL` | `[126, 201]` | 127 | `{"fused_activation_function": "NONE"}` |
| 88 | `MUL` | `[120, 127]` | 128 | `{"fused_activation_function": "NONE"}` |
| 89 | `MUL` | `[122, 127]` | 129 | `{"fused_activation_function": "NONE"}` |
| 90 | `ADD` | `[128, 129]` | 130 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 91 | `DEQUANTIZE` | `[4]` | 205 | `{}` |
| 92 | `DEQUANTIZE` | `[32]` | 211 | `{}` |
| 93 | `CONV_2D` | `[130, 211, 205]` | 131 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 94 | `DEQUANTIZE` | `[14]` | 212 | `{}` |
| 95 | `DEQUANTIZE` | `[33]` | 237 | `{}` |
| 96 | `CONV_2D` | `[131, 237, 212]` | 132 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 97 | `ADD` | `[132, 120]` | 133 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 98 | `MEAN` | `[133, 60]` | 134 | `{"keep_dims": true}` |
| 99 | `NEG` | `[134]` | 135 | `{}` |
| 100 | `SQUARED_DIFFERENCE` | `[133, 134]` | 136 | `{}` |
| 101 | `MEAN` | `[136, 60]` | 137 | `{"keep_dims": true}` |
| 102 | `ADD` | `[137, 218]` | 138 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 103 | `RSQRT` | `[138]` | 139 | `{}` |
| 104 | `DEQUANTIZE` | `[53]` | 240 | `{}` |
| 105 | `MUL` | `[139, 240]` | 140 | `{"fused_activation_function": "NONE"}` |
| 106 | `MUL` | `[133, 140]` | 141 | `{"fused_activation_function": "NONE"}` |
| 107 | `MUL` | `[135, 140]` | 142 | `{"fused_activation_function": "NONE"}` |
| 108 | `ADD` | `[141, 142]` | 143 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 109 | `TRANSPOSE` | `[143, 48]` | 144 | `{}` |
| 110 | `DEQUANTIZE` | `[5]` | 213 | `{}` |
| 111 | `DEQUANTIZE` | `[34]` | 219 | `{}` |
| 112 | `CONV_2D` | `[144, 219, 213]` | 145 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 113 | `DEQUANTIZE` | `[15]` | 238 | `{}` |
| 114 | `DEQUANTIZE` | `[35]` | 242 | `{}` |
| 115 | `CONV_2D` | `[145, 242, 238]` | 146 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 116 | `TRANSPOSE` | `[146, 48]` | 147 | `{}` |
| 117 | `ADD` | `[147, 133]` | 148 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 118 | `MEAN` | `[148, 60]` | 149 | `{"keep_dims": true}` |
| 119 | `NEG` | `[149]` | 150 | `{}` |
| 120 | `SQUARED_DIFFERENCE` | `[148, 149]` | 151 | `{}` |
| 121 | `MEAN` | `[151, 60]` | 152 | `{"keep_dims": true}` |
| 122 | `ADD` | `[152, 218]` | 153 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 123 | `RSQRT` | `[153]` | 154 | `{}` |
| 124 | `DEQUANTIZE` | `[52]` | 221 | `{}` |
| 125 | `MUL` | `[154, 221]` | 155 | `{"fused_activation_function": "NONE"}` |
| 126 | `MUL` | `[148, 155]` | 156 | `{"fused_activation_function": "NONE"}` |
| 127 | `MUL` | `[150, 155]` | 157 | `{"fused_activation_function": "NONE"}` |
| 128 | `ADD` | `[156, 157]` | 158 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 129 | `DEQUANTIZE` | `[36]` | 208 | `{}` |
| 130 | `DEQUANTIZE` | `[6]` | 239 | `{}` |
| 131 | `CONV_2D` | `[158, 208, 239]` | 159 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 132 | `DEQUANTIZE` | `[16]` | 222 | `{}` |
| 133 | `DEQUANTIZE` | `[37]` | 228 | `{}` |
| 134 | `CONV_2D` | `[159, 228, 222]` | 160 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 135 | `ADD` | `[160, 148]` | 161 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 136 | `MEAN` | `[161, 60]` | 162 | `{"keep_dims": true}` |
| 137 | `NEG` | `[162]` | 163 | `{}` |
| 138 | `SQUARED_DIFFERENCE` | `[161, 162]` | 164 | `{}` |
| 139 | `MEAN` | `[164, 60]` | 165 | `{"keep_dims": true}` |
| 140 | `ADD` | `[165, 218]` | 166 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 141 | `RSQRT` | `[166]` | 167 | `{}` |
| 142 | `DEQUANTIZE` | `[51]` | 235 | `{}` |
| 143 | `MUL` | `[167, 235]` | 168 | `{"fused_activation_function": "NONE"}` |
| 144 | `MUL` | `[161, 168]` | 169 | `{"fused_activation_function": "NONE"}` |
| 145 | `MUL` | `[163, 168]` | 170 | `{"fused_activation_function": "NONE"}` |
| 146 | `ADD` | `[169, 170]` | 171 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 147 | `TRANSPOSE` | `[171, 48]` | 172 | `{}` |
| 148 | `DEQUANTIZE` | `[38]` | 199 | `{}` |
| 149 | `DEQUANTIZE` | `[7]` | 223 | `{}` |
| 150 | `CONV_2D` | `[172, 199, 223]` | 173 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 151 | `DEQUANTIZE` | `[39]` | 231 | `{}` |
| 152 | `DEQUANTIZE` | `[17]` | 243 | `{}` |
| 153 | `CONV_2D` | `[173, 231, 243]` | 174 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 154 | `TRANSPOSE` | `[174, 48]` | 175 | `{}` |
| 155 | `ADD` | `[175, 161]` | 176 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 156 | `MEAN` | `[176, 60]` | 177 | `{"keep_dims": true}` |
| 157 | `NEG` | `[177]` | 178 | `{}` |
| 158 | `SQUARED_DIFFERENCE` | `[176, 177]` | 179 | `{}` |
| 159 | `MEAN` | `[179, 60]` | 180 | `{"keep_dims": true}` |
| 160 | `ADD` | `[180, 218]` | 181 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 161 | `RSQRT` | `[181]` | 182 | `{}` |
| 162 | `DEQUANTIZE` | `[50]` | 215 | `{}` |
| 163 | `MUL` | `[182, 215]` | 183 | `{"fused_activation_function": "NONE"}` |
| 164 | `MUL` | `[176, 183]` | 184 | `{"fused_activation_function": "NONE"}` |
| 165 | `MUL` | `[178, 183]` | 185 | `{"fused_activation_function": "NONE"}` |
| 166 | `ADD` | `[184, 185]` | 186 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 167 | `DEQUANTIZE` | `[40]` | 209 | `{}` |
| 168 | `DEQUANTIZE` | `[8]` | 244 | `{}` |
| 169 | `CONV_2D` | `[186, 209, 244]` | 187 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "RELU", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 170 | `DEQUANTIZE` | `[41]` | 216 | `{}` |
| 171 | `DEQUANTIZE` | `[18]` | 217 | `{}` |
| 172 | `CONV_2D` | `[187, 216, 217]` | 188 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 173 | `ADD` | `[188, 176]` | 189 | `{"fused_activation_function": "NONE", "pot_scale_int16": true}` |
| 174 | `STRIDED_SLICE` | `[189, 21, 42, 23]` | 190 | `{"begin_mask": 15, "ellipsis_mask": 0, "end_mask": 11, "new_axis_mask": 0, "offset": false, "shrink_axis_mask": 0}` |
| 175 | `STRIDED_SLICE` | `[190, 21, 43, 23]` | 191 | `{"begin_mask": 11, "ellipsis_mask": 0, "end_mask": 11, "new_axis_mask": 0, "offset": false, "shrink_axis_mask": 0}` |
| 176 | `DEQUANTIZE` | `[44]` | 200 | `{}` |
| 177 | `DEQUANTIZE` | `[19]` | 229 | `{}` |
| 178 | `CONV_2D` | `[191, 200, 229]` | 192 | `{"dilation_h_factor": 1, "dilation_w_factor": 1, "fused_activation_function": "NONE", "padding": "VALID", "quantized_bias_type": "FLOAT32", "stride_h": 1, "stride_w": 1}` |
| 179 | `LOGISTIC` | `[192]` | 193 | `{}` |
| 180 | `RESHAPE` | `[193, 46]` | 194 | `{}` |
| 181 | `STRIDED_SLICE` | `[194, 45, 46, 47]` | 195 | `{"begin_mask": 2, "ellipsis_mask": 0, "end_mask": 2, "new_axis_mask": 0, "offset": false, "shrink_axis_mask": 1}` |

## Reproducibility

```text
python3 tools/inspect-mediapipe-v1.py \
  --artifact /tmp/weights_faces_eyes/mediapipe-v1/face_landmarker-float16-v1.task \
  --schema /tmp/weights_faces_eyes/mediapipe-v1/schema.fbs \
  --rust-output crates/segment/src/mediapipe_inventory.rs \
  --json-output docs/models/mediapipe-face-landmarker-v1-qualification.json \
  --markdown-output docs/models/mediapipe-v1-artifact-inventory.md
```

Validation performed: exact artifact identity; exact stored ZIP entry set and payload ranges; every constant tensor range contained within its own ZIP entry; FlatBuffer bounds; supported dtype/op/option checks; no negative optional inputs; single-output nodes; topological producer order; shape-byte bounds; bounded INT32 StridedSlice controls with positive strides, rank-length shapes, zero ellipsis/new-axis masks, and in-range mask bits; and non-identical payload overlap rejection. Floating-point payload bytes are never read.

After generation, format Rust metadata with repository settings: `rustfmt --edition 2024 crates/segment/src/mediapipe_inventory.rs`.
