//! DINOv2's pinned positional-table interpolation.
//!
//! This is the non-register DINOv2 path from
//! <https://github.com/facebookresearch/dinov2/blob/7764ea0f912e53c92e82eb78a2a1631e92725fc8/dinov2/models/vision_transformer.py>.
//! DINOv2 supplies `scale_factor=(w0 + 0.1) / 37`, bicubic interpolation, and
//! `antialias=false`. The bicubic coefficients below follow PyTorch's pinned
//! native `upsample_bicubic2d` implementation at
//! <https://github.com/pytorch/pytorch/blob/v2.4.0/aten/src/ATen/native/cuda/UpSample.cuh>
//! (A = -0.75, align-corners=false, and bounded edge reads), rather than a
//! substitute interpolation kernel.
//!
//! Modified work: translated to Rust on Candle by Ember contributors (2026),
//! restricted to cached 224/518 tables with explicit validation.
//! Copyright (c) Meta Platforms, Inc. and affiliates. DINOv2 uses Apache-2.0.
//! PyTorch: BSD-style terms retained in docs/licenses/pytorch-bicubic-LICENSE.txt.
//! See root NOTICE.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Device, Tensor};

const EMBED_DIM: usize = 384;
const PRETRAIN_SIDE: usize = 37;
const PRETRAIN_PATCHES: usize = PRETRAIN_SIDE * PRETRAIN_SIDE;
const PRETRAIN_TOKENS: usize = PRETRAIN_PATCHES + 1;
const TARGET_SIDE: usize = 16;
const TARGET_PATCHES: usize = TARGET_SIDE * TARGET_SIDE;
const TARGET_TOKENS: usize = TARGET_PATCHES + 1;
const PATCH_SIZE: usize = 14;
const INTERPOLATE_OFFSET: f64 = 0.1;
const INTERPOLATION_SCALE: f64 = (TARGET_SIDE as f64 + INTERPOLATE_OFFSET) / PRETRAIN_SIDE as f64;
const BICUBIC_A: f64 = -0.75;
const _: () = assert!(518 / PATCH_SIZE == PRETRAIN_SIDE && 224 / PATCH_SIZE == TARGET_SIDE);

/// Validate `position` then adapt the pinned 518-side table to 224-side input.
///
/// Input table must be `[1, 1370, 384]` F32, matching DINOv2 ViT-S/14. The
/// 518-side path returns its original tensor unchanged. The 224-side path
/// copies only after validation to CPU, interpolates there once, and creates
/// the result directly on `device`; callers can cache that result at model
/// load instead of reading back or resampling per frame.
pub(crate) fn position_table(position: &Tensor, input_side: usize, device: &Device) -> crate::Result<Tensor> {
    if position.dtype() != DType::F32 || position.dims() != [1, PRETRAIN_TOKENS, EMBED_DIM] {
        return Err(crate::Error::Model(format!(
            "DINOv2 positional table must be F32 [1, {PRETRAIN_TOKENS}, {EMBED_DIM}], got {:?} {:?}",
            position.dtype(),
            position.dims()
        )));
    }
    if input_side != 518 && input_side != 224 {
        return Err(crate::Error::Model(format!("DINOv2 positional interpolation supports input sides 518 or 224, got {input_side}")));
    }

    let values = position.to_device(&Device::Cpu)?.flatten_all()?.to_vec1::<f32>()?;
    if values.len() != PRETRAIN_TOKENS * EMBED_DIM || values.iter().any(|value| !value.is_finite()) {
        return Err(crate::Error::Model("DINOv2 positional table contains non-finite or invalid data".into()));
    }

    if input_side == 518 {
        return Ok(position.clone());
    }

    let mut output = Vec::with_capacity(TARGET_TOKENS * EMBED_DIM);
    output.extend_from_slice(&values[..EMBED_DIM]);
    for output_y in 0..TARGET_SIDE {
        let (y_base, y_fraction) = interpolation_coordinate(output_y);
        for output_x in 0..TARGET_SIDE {
            let (x_base, x_fraction) = interpolation_coordinate(output_x);
            for channel in 0..EMBED_DIM {
                let mut horizontal = [0.0_f64; 4];
                for (tap, value) in horizontal.iter_mut().enumerate() {
                    let source_y = clamp_index(y_base + tap as isize - 1, PRETRAIN_SIDE);
                    let mut row = [0.0_f64; 4];
                    for (x_tap, sample) in row.iter_mut().enumerate() {
                        let source_x = clamp_index(x_base + x_tap as isize - 1, PRETRAIN_SIDE);
                        // Flattened table starts with one CLS vector; patch rows follow it.
                        let index = EMBED_DIM + (source_y * PRETRAIN_SIDE + source_x) * EMBED_DIM + channel;
                        *sample = f64::from(values[index]);
                    }
                    *value = cubic_interp(row, x_fraction);
                }
                let interpolated = cubic_interp(horizontal, y_fraction) as f32;
                if !interpolated.is_finite() {
                    return Err(crate::Error::Model("DINOv2 positional interpolation produced non-finite data".into()));
                }
                output.push(interpolated);
            }
        }
    }

    if output.len() != TARGET_TOKENS * EMBED_DIM {
        return Err(crate::Error::Model("DINOv2 positional interpolation produced an invalid table size".into()));
    }
    Ok(Tensor::from_vec(output, (1, TARGET_TOKENS, EMBED_DIM), device)?)
}

/// PyTorch's align-corners=false source coordinate with caller-provided scale.
fn interpolation_coordinate(output_index: usize) -> (isize, f64) {
    let source = (output_index as f64 + 0.5) / INTERPOLATION_SCALE - 0.5;
    let base = source.floor();
    (base as isize, source - base)
}

fn clamp_index(index: isize, side: usize) -> usize {
    if index < 0 {
        0
    } else if index >= side as isize {
        side - 1
    } else {
        index as usize
    }
}

/// PyTorch's native bicubic kernel, with A = -0.75.
fn cubic_interp(samples: [f64; 4], t: f64) -> f64 {
    let c0 = cubic_convolution2(t + 1.0, BICUBIC_A);
    let c1 = cubic_convolution1(t, BICUBIC_A);
    let opposite = 1.0 - t;
    let c2 = cubic_convolution1(opposite, BICUBIC_A);
    let c3 = cubic_convolution2(opposite + 1.0, BICUBIC_A);
    samples[0] * c0 + samples[1] * c1 + samples[2] * c2 + samples[3] * c3
}

fn cubic_convolution1(x: f64, a: f64) -> f64 {
    ((a + 2.0) * x - (a + 3.0)) * x * x + 1.0
}

fn cubic_convolution2(x: f64, a: f64) -> f64 {
    ((a * x - 5.0 * a) * x + 8.0 * a) * x - 4.0 * a
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pinned_table_constants_match_dinov2_nonregister_path() {
        assert_eq!(PRETRAIN_TOKENS, 1 + 37 * 37);
        assert_eq!(TARGET_TOKENS, 1 + 16 * 16);
        assert_eq!(PATCH_SIZE, 14);
        assert_eq!(INTERPOLATE_OFFSET, 0.1);
        assert!((INTERPOLATION_SCALE - 16.1 / 37.0).abs() < f64::EPSILON);
        assert_eq!(BICUBIC_A, -0.75);
    }

    #[test]
    fn first_and_last_reference_coordinates_use_pinned_scale() {
        let first = interpolation_coordinate(0);
        let last = interpolation_coordinate(TARGET_SIDE - 1);
        assert_eq!(first.0, 0);
        assert!((first.1 - 0.6490683229813663).abs() < 1e-12);
        assert_eq!(last.0, 35);
        assert!((last.1 - 0.12111801242235984).abs() < 1e-12);
    }

    #[test]
    fn edge_taps_clamp_without_wrapping() {
        assert_eq!(clamp_index(-1, PRETRAIN_SIDE), 0);
        assert_eq!(clamp_index(0, PRETRAIN_SIDE), 0);
        assert_eq!(clamp_index(PRETRAIN_SIDE as isize, PRETRAIN_SIDE), PRETRAIN_SIDE - 1);
        assert_eq!(clamp_index(PRETRAIN_SIDE as isize + 1, PRETRAIN_SIDE), PRETRAIN_SIDE - 1);
    }

    #[test]
    fn bicubic_integer_edges_select_center_taps() {
        let samples = [3.0, 7.0, 11.0, 13.0];
        assert!((cubic_interp(samples, 0.0) - samples[1]).abs() < 1e-12);
        assert!((cubic_interp(samples, 1.0) - samples[2]).abs() < 1e-12);
    }

    fn constant_position(cls: f32, patch: f32) -> crate::Result<Tensor> {
        let mut values = vec![patch; PRETRAIN_TOKENS * EMBED_DIM];
        for value in values.iter_mut().take(EMBED_DIM) {
            *value = cls;
        }
        Ok(Tensor::from_vec(values, (1, PRETRAIN_TOKENS, EMBED_DIM), &Device::Cpu)?)
    }

    #[test]
    fn interpolation_excludes_cls_from_patch_table() -> crate::Result<()> {
        let position = constant_position(-1234.0, 7.0)?;
        let output = position_table(&position, 224, &Device::Cpu)?;
        let values = output.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(values.len(), TARGET_TOKENS * EMBED_DIM);
        assert_eq!(values.first().copied(), Some(-1234.0));
        assert_eq!(values.get(EMBED_DIM).copied(), Some(7.0));
        assert!(values.iter().skip(EMBED_DIM).all(|value| *value == 7.0));
        Ok(())
    }

    #[test]
    fn invalid_and_nonfinite_tables_are_rejected() -> crate::Result<()> {
        let invalid = Tensor::from_vec(vec![0.0_f32; EMBED_DIM], (1, 1, EMBED_DIM), &Device::Cpu)?;
        assert!(position_table(&invalid, 224, &Device::Cpu).is_err());

        let mut nonfinite_values = vec![0.0_f32; PRETRAIN_TOKENS * EMBED_DIM];
        if let Some(value) = nonfinite_values.get_mut(EMBED_DIM) {
            *value = f32::NAN;
        }
        let nonfinite = Tensor::from_vec(nonfinite_values, (1, PRETRAIN_TOKENS, EMBED_DIM), &Device::Cpu)?;
        assert!(position_table(&nonfinite, 224, &Device::Cpu).is_err());
        Ok(())
    }
}
