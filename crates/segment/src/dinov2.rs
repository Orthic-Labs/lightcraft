//! Offline-only DINOv2 ViT-S/14 similarity prototype.
//!
//! Architecture follows Meta's pinned DINOv2 source at commit
//! `7764ea0f912e53c92e82eb78a2a1631e92725fc8`: 384 channels, 12 blocks, 6 heads,
//! patch size 14, MLP ratio 4, fused QKV, LayerScale initial value 1.0 and
//! LayerNorm epsilon 1e-6. Use [`crate::dinov2_artifact`] for the pinned, immutable
//! size- and SHA-256-verified artifact & [`crate::dinov2_input`] for RGB8 input
//! preparation. The core is not wired into catalog or culling decisions.
//!
//! Modified work (Apache License 2.0, §4(b)): translated to Rust on Candle by
//! Ember contributors (2026), with bounded inputs & explicit validation.
//! Copyright (c) Meta Platforms, Inc. and affiliates. See root NOTICE.
//!
//! References: <https://raw.githubusercontent.com/facebookresearch/dinov2/7764ea0f912e53c92e82eb78a2a1631e92725fc8/dinov2/models/vision_transformer.py>,
//! <https://raw.githubusercontent.com/facebookresearch/dinov2/7764ea0f912e53c92e82eb78a2a1631e92725fc8/dinov2/layers/attention.py>,
//! <https://raw.githubusercontent.com/facebookresearch/dinov2/7764ea0f912e53c92e82eb78a2a1631e92725fc8/dinov2/layers/block.py>.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Module, Tensor};
use candle_nn::{Conv2d, Conv2dConfig, LayerNorm, Linear, VarBuilder};

#[cfg(test)]
use candle_core::Device;

use crate::dinov2_positions::position_table;
use crate::nn::{Mlp, ln};
use crate::{Error, Result};

const EMBED: usize = 384;
const DEPTH: usize = 12;
const HEADS: usize = 6;
const HEAD_DIM: usize = EMBED / HEADS;
const PATCH: usize = 14;
const MLP_HIDDEN: usize = EMBED * 4;
const INPUT_224: usize = 224;
const INPUT_518: usize = 518;
const POSITION_TOKENS_518: usize = 1 + (INPUT_518 / PATCH) * (INPUT_518 / PATCH);
const LN_EPS: f64 = 1e-6;

/// Final normalized features returned by DINOv2.
#[derive(Debug)]
pub struct DinoV2Output {
    /// Final LayerNorm CLS token, shape `[1, 384]`.
    pub cls_token: Tensor,
    /// Final LayerNorm patch tokens, shape `[1, N, 384]`.
    pub patch_tokens: Tensor,
}

struct LayerScale {
    gamma: Tensor,
}

impl LayerScale {
    fn new(vb: VarBuilder) -> Result<Self> {
        Ok(Self { gamma: vb.get(EMBED, "gamma")? })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        Ok(x.broadcast_mul(&self.gamma)?)
    }
}

struct Attention {
    qkv: Linear,
    proj: Linear,
}

impl Attention {
    fn new(vb: VarBuilder) -> Result<Self> {
        Ok(Self { qkv: candle_nn::linear(EMBED, EMBED * 3, vb.pp("qkv"))?, proj: candle_nn::linear(EMBED, EMBED, vb.pp("proj"))? })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let (batch, tokens, channels) = x.dims3()?;
        if channels != EMBED {
            return Err(Error::Model(format!("DINOv2 attention expected {EMBED} channels, got {channels}")));
        }
        let qkv = self.qkv.forward(x)?.reshape((batch, tokens, 3, HEADS, HEAD_DIM))?.permute((0, 2, 3, 1, 4))?;
        // `permute` leaves strided views; materialize each view before sdpa's matmul.
        let q = qkv.narrow(1, 0, 1)?.squeeze(1)?.contiguous()?;
        let k = qkv.narrow(1, 1, 1)?.squeeze(1)?.contiguous()?;
        let v = qkv.narrow(1, 2, 1)?.squeeze(1)?.contiguous()?;
        let y = crate::nn::sdpa(&q, &k, &v, None, (HEAD_DIM as f64).sqrt().recip())?;
        let y = y.transpose(1, 2)?.reshape((batch, tokens, EMBED))?;
        Ok(self.proj.forward(&y)?)
    }
}

struct Block {
    norm1: LayerNorm,
    attention: Attention,
    ls1: LayerScale,
    norm2: LayerNorm,
    mlp: Mlp,
    ls2: LayerScale,
}

impl Block {
    fn new(index: usize, vb: VarBuilder) -> Result<Self> {
        let vb = vb.pp(format!("blocks.{index}"));
        Ok(Self {
            norm1: ln(EMBED, LN_EPS, vb.pp("norm1"))?,
            attention: Attention::new(vb.pp("attn"))?,
            ls1: LayerScale::new(vb.pp("ls1"))?,
            norm2: ln(EMBED, LN_EPS, vb.pp("norm2"))?,
            mlp: Mlp::new(EMBED, MLP_HIDDEN, true, vb.pp("mlp"))?,
            ls2: LayerScale::new(vb.pp("ls2"))?,
        })
    }

    fn forward(&self, x: &Tensor) -> Result<Tensor> {
        let attention = self.attention.forward(&self.norm1.forward(x)?)?;
        let x = (x + self.ls1.forward(&attention)?)?;
        let mlp = self.mlp.forward(&self.norm2.forward(&x)?)?;
        Ok((&x + self.ls2.forward(&mlp)?)?)
    }
}

/// DINOv2 ViT-S/14 with no preprocessing or checkpoint-path side effects.
pub struct DinoV2 {
    patch_embed: Conv2d,
    cls_token: Tensor,
    /// Kept to force shape validation against official state keys; masks are not exposed here.
    mask_token: Tensor,
    position_224: Tensor,
    position_518: Tensor,
    blocks: Vec<Block>,
    norm: LayerNorm,
}

impl DinoV2 {
    /// Build from a VarBuilder whose caller verified exact artifact size/SHA.
    ///
    /// Candle 0.9.2's path-based `VarBuilder::from_pth` reopens its path for each
    /// tensor, so this constructor has no checkpoint-path input or artifact identity
    /// check. It validates architecture shapes & dtype only. Prefer
    /// [`crate::dinov2_artifact::load_file`] or [`crate::dinov2_artifact::load_verified_bytes`],
    /// which construct tensors from one immutable, verified copy of the pinned `.pth`.
    pub fn from_var_builder(vb: VarBuilder) -> Result<Self> {
        if vb.dtype() != DType::F32 {
            return Err(Error::Model(format!("DINOv2 VarBuilder expects F32 tensors, got {:?}", vb.dtype())));
        }
        let device = vb.device().clone();
        let patch_embed = candle_nn::conv2d(3, EMBED, PATCH, Conv2dConfig { stride: PATCH, ..Default::default() }, vb.pp("patch_embed.proj"))?;
        let cls_token = vb.get((1, 1, EMBED), "cls_token")?;
        let mask_token = vb.get((1, EMBED), "mask_token")?;
        let position = vb.get((1, POSITION_TOKENS_518, EMBED), "pos_embed")?;
        let position_224 = position_table(&position, INPUT_224, &device)?;
        let position_518 = position_table(&position, INPUT_518, &device)?;
        let blocks = (0..DEPTH).map(|i| Block::new(i, vb.clone())).collect::<Result<Vec<_>>>()?;
        let norm = ln(EMBED, LN_EPS, vb.pp("norm"))?;
        Ok(Self { patch_embed, cls_token, mask_token, position_224, position_518, blocks, norm })
    }

    /// Run already normalized F32 pixels with shape `[1, 3, 224, 224]` or `[1, 3, 518, 518]`.
    pub fn forward(&self, pixels: &Tensor) -> Result<DinoV2Output> {
        let height = validate_pixels(pixels)?;
        let patch = self.patch_embed.forward(pixels)?;
        let (_, _, grid_h, grid_w) = patch.dims4()?;
        let tokens = patch.flatten_from(2)?.transpose(1, 2)?;
        let x = Tensor::cat(&[self.cls_token.clone(), tokens], 1)?;
        let position = if height == INPUT_224 { &self.position_224 } else { &self.position_518 };
        let mut x = x.broadcast_add(position)?;
        for block in &self.blocks {
            x = block.forward(&x)?;
        }
        let x = self.norm.forward(&x)?;
        require_finite(&x, "output")?;
        let cls_token = x.narrow(1, 0, 1)?.squeeze(1)?;
        let patch_tokens = x.narrow(1, 1, grid_h * grid_w)?;
        Ok(DinoV2Output { cls_token, patch_tokens })
    }

    /// Cosine similarity for two CLS embeddings; rejects non-finite or zero vectors.
    pub fn cosine(a: &Tensor, b: &Tensor) -> Result<f32> {
        if a.dims() != [1, EMBED] || b.dims() != [1, EMBED] || a.dtype() != DType::F32 || b.dtype() != DType::F32 {
            return Err(Error::Model("DINOv2 cosine expects two F32 [1,384] embeddings".into()));
        }
        let av = a.to_vec2::<f32>()?.into_iter().next().ok_or_else(|| Error::Model("DINOv2 cosine missing first vector".into()))?;
        let bv = b.to_vec2::<f32>()?.into_iter().next().ok_or_else(|| Error::Model("DINOv2 cosine missing second vector".into()))?;
        let (mut dot, mut aa, mut bb) = (0.0f64, 0.0f64, 0.0f64);
        for (&x, &y) in av.iter().zip(&bv) {
            if !x.is_finite() || !y.is_finite() {
                return Err(Error::Model("DINOv2 cosine received non-finite embedding".into()));
            }
            let (x, y) = (x as f64, y as f64);
            dot += x * y;
            aa += x * x;
            bb += y * y;
        }
        if aa <= f64::EPSILON || bb <= f64::EPSILON {
            return Err(Error::Model("DINOv2 cosine received zero embedding".into()));
        }
        Ok((dot / (aa.sqrt() * bb.sqrt())).clamp(-1.0, 1.0) as f32)
    }

    /// Shape-checked mask token retained for future masked-token evaluation.
    pub fn mask_token(&self) -> &Tensor {
        &self.mask_token
    }
}

fn validate_pixels(pixels: &Tensor) -> Result<usize> {
    if pixels.dtype() != DType::F32 {
        return Err(Error::Model(format!("DINOv2 expects F32 pixels, got {:?}", pixels.dtype())));
    }
    let (batch, channels, height, width) = pixels.dims4()?;
    if batch != 1 || channels != 3 || height != width || (height != INPUT_224 && height != INPUT_518) {
        return Err(Error::Model(format!("DINOv2 expects [1,3,224,224] or [1,3,518,518], got {batch:?}x{channels:?}x{height:?}x{width:?}")));
    }
    require_finite(pixels, "input")?;
    Ok(height)
}

/// Check one tensor boundary with one device reduction, avoiding per-layer host readbacks.
fn require_finite(tensor: &Tensor, label: &str) -> Result<()> {
    // `x - x` is zero for every finite x, while NaN and infinities remain NaN;
    // sum propagates that marker without overflow from large finite inputs.
    let finite_marker = tensor.broadcast_sub(tensor)?.sum_all()?.to_scalar::<f32>()?;
    if !finite_marker.is_finite() {
        return Err(Error::Model(format!("DINOv2 {label} contains non-finite values")));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_invalid_pixel_shape() -> Result<()> {
        let pixels = Tensor::zeros((1, 3, 224, 223), DType::F32, &Device::Cpu)?;
        assert!(validate_pixels(&pixels).is_err());
        Ok(())
    }

    #[test]
    fn rejects_nonfinite_pixels() -> Result<()> {
        for bad in [f32::NAN, f32::NEG_INFINITY, f32::INFINITY] {
            let mut values = vec![0.0f32; 3 * INPUT_224 * INPUT_224];
            let last = values.len() - 1;
            values[last] = bad;
            let pixels = Tensor::from_vec(values, (1, 3, INPUT_224, INPUT_224), &Device::Cpu)?;
            assert!(validate_pixels(&pixels).is_err());
        }
        Ok(())
    }

    #[test]
    fn rejects_non_f32_var_builder_before_key_access() {
        let vb = VarBuilder::from_tensors(std::collections::HashMap::new(), DType::F16, &Device::Cpu);
        assert!(DinoV2::from_var_builder(vb).is_err());
    }

    #[test]
    fn attention_materializes_qkv_views_and_matches_reference() -> Result<()> {
        let mut tensors = std::collections::HashMap::new();
        let mut qkv_weight = vec![0.0f32; EMBED * 3 * EMBED];
        for projection in 0..3 {
            let offset = projection * EMBED * EMBED;
            for coordinate in 0..EMBED {
                qkv_weight[offset + coordinate * EMBED + coordinate] = 1.0;
            }
        }
        tensors.insert("qkv.weight".to_string(), Tensor::from_vec(qkv_weight, (EMBED * 3, EMBED), &Device::Cpu)?);
        tensors.insert("qkv.bias".to_string(), Tensor::zeros(EMBED * 3, DType::F32, &Device::Cpu)?);
        let mut projection_weight = vec![0.0f32; EMBED * EMBED];
        for coordinate in 0..EMBED {
            projection_weight[coordinate * EMBED + coordinate] = 1.0;
        }
        tensors.insert("proj.weight".to_string(), Tensor::from_vec(projection_weight, (EMBED, EMBED), &Device::Cpu)?);
        tensors.insert("proj.bias".to_string(), Tensor::zeros(EMBED, DType::F32, &Device::Cpu)?);
        let attention = Attention::new(VarBuilder::from_tensors(tensors, DType::F32, &Device::Cpu))?;

        let mut input = vec![0.0f32; 3 * EMBED];
        input[0] = 1.0;
        input[EMBED] = 0.0;
        input[2 * EMBED] = -1.0;
        let output = attention.forward(&Tensor::from_vec(input, (1, 3, EMBED), &Device::Cpu)?)?.to_vec3::<f32>()?;
        let score = 0.125f32;
        let expected = 2.0 * score.sinh() / (1.0 + 2.0 * score.cosh());
        for (actual, target) in [expected, 0.0, -expected].into_iter().enumerate() {
            assert!((output[0][actual][0] - target).abs() < 1e-5);
            for coordinate in 1..EMBED {
                assert!(output[0][actual][coordinate].abs() < 1e-5);
            }
        }
        Ok(())
    }

    #[test]
    fn rejects_zero_or_nonfinite_cosine_inputs() -> Result<()> {
        let zero = Tensor::zeros((1, EMBED), DType::F32, &Device::Cpu)?;
        assert!(DinoV2::cosine(&zero, &zero).is_err());
        let nan = Tensor::from_vec(vec![f32::NAN; EMBED], (1, EMBED), &Device::Cpu)?;
        assert!(DinoV2::cosine(&nan, &nan).is_err());
        Ok(())
    }
}
