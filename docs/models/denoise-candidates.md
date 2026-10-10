# Learned RGB denoise candidate

Research snapshot: 2026-10-10. **NAFNet-SIDD-width32 is first architecture candidate for pure-Rust numerical qualification**, subject to checkpoint-rights evidence. No weights were downloaded, imported, redistributed or executed for this research; no Ember quality, memory or latency result exists. [Offline protocol](../denoise-qualification.md) defines comparison gates.

## Separate rights records

Official source is pinned at [`megvii-research/NAFNet@2b4af71ebe098a92a75910c233a3965a3e93ede4`](https://github.com/megvii-research/NAFNet/tree/2b4af71ebe098a92a75910c233a3965a3e93ede4). Its [licence file](https://raw.githubusercontent.com/megvii-research/NAFNet/2b4af71ebe098a92a75910c233a3965a3e93ede4/LICENSE) grants MIT rights for software & retains BasicSR Apache-2.0 text. [Official README](https://raw.githubusercontent.com/megvii-research/NAFNet/2b4af71ebe098a92a75910c233a3965a3e93ede4/readme.md) links a pretrained `.pth`, without a separate explicit checkpoint grant, SHA or byte size. Record **first-party checkpoint permission unresolved from reviewed evidence**, rather than treating software MIT as proven weight permission.

[SIDD publisher](https://abdokamel.github.io/sidd/index.html) explicitly licenses its dataset/code under MIT. It provides normalized black-level-subtracted raw RGB & gamma-corrected sRGB without tone mapping. This dataset statement does not establish downstream NAFNet checkpoint permission or Ember camera coverage.

[LiteRT converted-artifact publisher](https://huggingface.co/litert-community/NAFNet-SIDD-width32-LiteRT/tree/59984b5234b24ccf2f324ce0ac49cd640b9d5b2c) labels its card MIT & identifies upstream SIDD-width32 weights. That is a community artifact claim, recorded separately from first-party checkpoint permission. Converted revision is `59984b5234b24ccf2f324ce0ac49cd640b9d5b2c`; `nafnet_sidd_width32_fp16.tflite` metadata reports 62,454,048 bytes & LFS SHA-256 `f8fbaa422411683c53e802cf7cc7cf9be0a0de00886ad4af057232e26b172a0c`. Metadata is an identity candidate, not proof of bytes inspected or numerical agreement.

## Architecture evidence

[Pinned training config](https://raw.githubusercontent.com/megvii-research/NAFNet/2b4af71ebe098a92a75910c233a3965a3e93ede4/options/train/SIDD/NAFNet-width32.yml) selects width 32, encoder blocks `[2,2,4,8]`, 12 middle blocks & decoder blocks `[2,2,2,2]`: 16 encoder + 12 middle + 8 decoder blocks. Training crops are 256px; this is not an Ember preview/crop qualification.

[Architecture source](https://raw.githubusercontent.com/megvii-research/NAFNet/2b4af71ebe098a92a75910c233a3965a3e93ede4/basicsr/models/archs/NAFNet_arch.py) uses convolution/depthwise convolution, channel multiplication, channel-wise LayerNorm, average-pool channel attention, residual scales & PixelShuffle. Base inference pads right/bottom with zeros to multiples of 16, then crops output. Channel attention depends on whole-input statistics, so tiled output must be tested independently. Input/transfer, output clamp, precision & local-pooling variants need exact reference contracts; no automatic scene-linear Rec.2020 substitution is valid.

Independent arithmetic gives **29,159,715 parameters**: each NAFBlock contributes `7c²+33c`, including biases, LayerNorm affine terms & beta/gamma; intro/ending/down/up convolutions are counted separately. Raw F16 payload would be 58,319,430 bytes; converted artifact metadata is 4,134,618 bytes larger. Graph/metadata/alignment overhead is unverified. Parameter size is not peak inference memory.

[Official test config](https://raw.githubusercontent.com/megvii-research/NAFNet/2b4af71ebe098a92a75910c233a3965a3e93ede4/options/test/SIDD/NAFNet-width32.yml) selects full `NAFNet`, strict checkpoint loading & `grids:false`. Use that full-image variant as numerical reference. `NAFNetLocal`/TLC replaces global pooling with different spatial behavior; any substitution needs separate contract/reference receipts.

| Pinned text | Bytes | SHA-256 |
| --- | ---: | --- |
| `LICENSE` | 12,466 | `a29ecef3456149898f08e4c71b11b33e7d333664e087bc212e84e18ddd6599ad` |
| `NAFNet_arch.py` | 6,396 | `01b22270cc93f1bb90c0e3e4490e98b023fcf73f8552860b4a9ee880ce5c6967` |
| `arch_util.py` | 12,046 | `5a11af2e7c2d7a7b57c1fbd7e19cf0a50b4b4e8c7ae7dd203a915d7a707e7005` |
| `options/train/SIDD/NAFNet-width32.yml` | 2,181 | `06e4ea3e38581e68ba16771a42994c5c1470433f6dbbc721db87bd25b3fa1307` |
| `options/test/SIDD/NAFNet-width32.yml` | 1,405 | `653d9bbc075df85cf996870d99e2ca7cf1aadbb05d5059a365343a3569088b2c` |

`arch_util.py` defines per-pixel channel mean/variance LayerNorm with epsilon `1e-6`, not spatial LayerNorm. These text hashes identify reviewed source, not executable/model qualification.

## Choice & next measurement

Choose this architecture first because its operator inventory avoids shifted-window attention & its source/config are compact enough to audit against an independent reference. This is an engineering judgment, not a measured speed or image-quality ranking. DRUNet, Restormer & SCUNet remain alternatives if checkpoint rights, transfer behavior or tile quality fail.

Resolve checkpoint permission before loading learned weights. Compare independent full-NAFNet outputs, whole/tiled boundaries & controlled procedural errors, then consented held-out camera/ISO shoots plus human preference. Record named-target latency/memory before production integration; retain original sources & reviewable acceptance.
