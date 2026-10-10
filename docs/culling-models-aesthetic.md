# Culling model candidates: aesthetic, quality & similarity

Status: research only for candle0.9.2. No model, weights or dependency is shipped by this note.

## Existing baseline

`crates/pipeline/src/cull.rs` measures decoded pixels at up to 512 px long edge:

- `sharpness`: 90th percentile block Laplacian energy, mapped to 0..100;
- `clipped`: share of crushed/blown pixels, 0..1;
- `signature`: 8×8 normalized luma grid (64 `f32`s);
- `similarity`: signature dot product, -1..1 (`1` means same normalized layout).

`catalog::Analysis` stores `sharpness`, `clipped`, optional burst `group` & `best`. Existing behavior is deterministic, cheap & pure Rust. Neural output should be optional evidence for burst reranking, never automatic reject. Preserve intentional motion blur, shallow depth of field & soft-focus style through baseline focus score, score uncertainty & user review.

## Candidate matrix

Sizes are exact bytes where source publishes them; MiB uses 2²⁰ bytes. Parameter counts are source-reported or marked derived. `Ember` latency & held-out accuracy are **UNMEASURED** for every candidate.

| Candidate | Pinned primary sources | Code / weights / data licensing | Shape, output & reported paper metric | Porting, dependency & platform cost | Burst-choice fit |
|---|---|---|---|---|---|
| **NIMA MobileNet aesthetic** | [pinned raw source commit `4a22c4b`](https://raw.githubusercontent.com/titu1994/neural-image-assessment/4a22c4b59b8af1a7d23edb94fb7f48a008ba986c/README.md); [MobileNet checkpoint release `v0.3`](https://github.com/titu1994/neural-image-assessment/releases/tag/v0.3) | Code MIT. `mobilenet_weights.h5`: **13,159,768 B** (12.55 MiB); release artifact has no separate weight license, so clearance is required. AVA labels/images are separate dataset terms & are not shipped. | MobileNet α=1, ≈4.25M backbone params + 10,250 score-head params (derived from pinned model code); 10-bin softmax → mean & std on 1..10. Source evaluator supports 224×224 resize or variable dimensions. Repo reports AVA validation EMD **0.0804**; this is not Ember result. | Depthwise Conv2D, BatchNorm, ReLU6, global average pool & Dense are plausible Rust/WGSL port targets. No TensorFlow runtime is needed after graph port. Metal/Windows CPU latency, memory & batch behavior require measurement. | Compact aesthetic candidate; AVA taste can punish intentional blur, unusual framing or documentary style. Use mean + std only as soft rerank evidence; retain baseline sharpness & tie band. |
| **MUSIQ-AVA / KonIQ** | [Google Research code commit `6e6b1ff`](https://github.com/google-research/google-research/tree/6e6b1ff7471be7ed884ff7ce0821c889a3a1b0c9/musiq); [ICCV paper](https://openaccess.thecvf.com/content/ICCV2021/html/Ke_MUSIQ_Multi-Scale_Image_Quality_Transformer_ICCV_2021_paper.html); [AVA checkpoint, GCS generation `1638397898373197`](https://storage.googleapis.com/gresearch/musiq/ava_ckpt.npz?generation=1638397898373197) | Code Apache-2.0. AVA checkpoint: **162,966,730 B** (155.37 MiB), MD5 ETag `6d73ebdb8433d2e2052e634f6adee990`; Google bucket does not state separate weight terms in pinned README, so clearance is required. AVA/KonIQ/PaQ2PiQ/SPAQ terms remain separate. | ≈27M params. Default input is native aspect-ratio image + longer sides 224 & 384; patch 32, hidden 384, 14 transformer layers, 6 heads, 10×10 spatial grid. Single-scale variant exists. Scalar MOS (AVA can emit 10-class distribution). Paper reports AVA ARP multi-scale SRCC **0.726**, PLCC **0.738**; not Ember accuracy. | Native-resolution patch extraction, ResNet embedding, multi-scale token packing, positional/scale embeddings, GELU, LayerNorm & MHSA. Port cost high but bounded; single-scale lowers work. Metal & Windows CPU latency require measurement, with caching/quantization evaluated separately. | Aspect preservation is useful for composition & style, yet MOS is still taste-biased. Full-size path can preserve blur context better than fixed crop. Use as low-weight suggestion, never reject. |
| **MANIQA-KonIQ** | [source commit `f573d86`](https://github.com/IIGROUP/MANIQA/tree/f573d862401243c45641951a0303b8059c1c8577); [CVPR paper](https://arxiv.org/abs/2204.08958); [Koniq10k release](https://github.com/IIGROUP/MANIQA/releases/tag/Koniq10k) | Code Apache-2.0. `ckpt_koniq10k.pt`: **543,335,435 B** (518.12 MiB); release artifact has no separate weight license, so clearance is required. KonIQ images/labels are separate dataset terms. | **135.75M params** (reported complexity table); inference code samples **20 random 224×224 crops**, averages scalar 0..1. Official README reports KonIQ test SRCC **0.930**, PLCC **0.946**; not Ember accuracy. | ViT-B/8 feature hooks + transposed attention + two Swin blocks + patch-weighted heads; needs ViT, window attention, `einops`-style reshape, LayerNorm, softmax & many tensor ops. Largest parameter/checkpoint footprint here. Metal & Windows CPU viability require measurement before selection. | Technical-quality benchmark is not photographic intent. Random crops add score noise & can miss subject; 20-crop inference is costly. Research comparator only; do not use for hard culling. |
| **DINOv2 ViT-S/14 embedding** | [Meta model card at commit `7764ea0`](https://github.com/facebookresearch/dinov2/blob/7764ea0f912e53c92e82eb78a2a1631e92725fc8/MODEL_CARD.md); [Apache license](https://github.com/facebookresearch/dinov2/blob/7764ea0f912e53c92e82eb78a2a1631e92725fc8/LICENSE); [weight URL](https://dl.fbaipublicfiles.com/dinov2/dinov2_vits14/dinov2_vits14_pretrain.pth) | Code & model card state Apache-2.0. ViT-S/14 checkpoint transfer observed at **88,283,115 B** (84.19 MiB), with `Last-Modified` 2023-04-13 & ETag `0bd1417efc23bdb1d69200156f5a22bc-11`; these headers are not immutable pins. LVD-142M training data is not shipped. **SHA-256 capture is a qualification gate before use or packaging.** | 21M params, 384-D CLS embedding + patch tokens. Source code defaults to 518 px; model accepts dimensions that are multiples of 14. Evaluate 224×224 & 518×518 as separate preprocessing conditions; do not treat outputs as equivalent. Output is cosine similarity, not quality. Model card reports ImageNet k-NN 79.0% & Oxford-H retrieval mAP 43.2 for ViT-S/14; these are not burst or Ember metrics. | ViT patch projection, positional interpolation, LayerNorm, GELU, MHSA & MLP. Moderate Rust/WGSL port candidate; no decoder/head. Metal & Windows CPU latency, memory & cache behavior require measurement. | Similarity-only candidate for look-alike grouping & style-preserving comparison; no learned quality reject. Semantic invariance may merge distinct shots; combine with existing 64-D luma similarity & capture-time burst window. |

### License reading

Code license does not automatically license checkpoint bytes or training data. NIMA, MUSIQ & MANIQA weights need explicit redistribution decisions before any download or packaging. DINOv2 model card states Apache-2.0 for model; record exact URL, bytes & SHA-256 in asset attribution if qualified. No GPL, Adobe or C++ ONNX Runtime source is used here.

### Metrics boundary

Paper/repository metrics above describe each author’s dataset, split, preprocessing & task. They do not predict Ember behavior. For candle0.9.2, Ember latency, memory, ranking quality, calibration & held-out accuracy are **UNMEASURED**.

## Recommendation

Do not require neural model for current culling. Existing 512 px Laplacian + 8×8 signature supplies objective sharpness & look-alike grouping at low cost.

First qualification candidate: **DINOv2 ViT-S/14** for similarity-only reranking, since it avoids encoding “sharp = good” & has an Apache-2.0 model statement. Keep **NIMA MobileNet** as compact aesthetic comparator after weight clearance. Defer MUSIQ to second-stage aspect-preserving quality comparison; defer MANIQA until its 518 MiB checkpoint & CPU/GPU measurements justify port cost.

## Explicit qualification task

1. Build offline, read-only evaluator outside shipped crates. Keep current `Analysis` fields unchanged; add candidate outputs beside them.
2. Use burst groups from capture-time proximity + current signature. Preserve each source’s preprocessing: NIMA source resize/variable-size conditions; MUSIQ native + 224 + 384 multi-scale and single-scale; MANIQA 20×224 crops; DINOv2-S/14 at separate 224×224 & source-default 518×518 conditions. Do not treat DINO 224 & 518 outputs as equivalent. No weights/media commit.
3. Label held-out bursts with three-way preference: technical keeper, intentional-style keeper, tie/uncertain. Report pairwise accuracy, tie coverage, blur-preservation rate, score spread, p50/p95 CPU latency, Metal latency, peak memory & cold/warm costs on macOS + Windows CPU.
4. Require abstention: if model disagreement, low margin or intentional-blur label occurs, preserve current order & show suggestion only. Model may not set `best` until held-out results, license review & platform budgets pass.
5. Record exact checkpoint bytes, SHA-256, preprocessing, operator list & source license in `assets/ATTRIBUTION.md` only after qualification selects one. DINOv2 SHA-256 must be captured before any use or packaging; ETag/Last-Modified alone do not qualify as immutable pins.

Next action: qualify DINOv2-S/14 similarity & NIMA MobileNet aesthetic on held-out personal-style burst labels; report evidence before changing pipeline or catalog schema.
