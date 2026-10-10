//! Bounded RGB8 input preparation for DINOv2.
//!
//! The policy follows the pinned DINOv2 evaluation transform's observable contract:
//! resize the short edge, center-crop, convert RGB8 to tensor, then apply ImageNet
//! normalization. The interpolation kernel is implemented here so image decoding and
//! inference do not acquire a runtime image-processing dependency.
//!
//! Source facts: <https://raw.githubusercontent.com/facebookresearch/dinov2/7764ea0f912e53c92e82eb78a2a1631e92725fc8/dinov2/data/transforms.py>
//! (Apache-2.0, Copyright Meta Platforms, Inc. and affiliates). This implementation
//! independently defines the bounded numerical contract documented in
//! `docs/models/dinov2-input-contract.md`; it does not claim byte-for-byte parity with
//! Pillow or torchvision.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{Device, Tensor};

#[cfg(test)]
use candle_core::DType;

use crate::{Error, Result};

const MAX_PIXELS: usize = 32_000_000;
const MAX_SIDE: usize = 16_384;
const SMALL_SIDE: usize = 224;
const LARGE_SIDE: usize = 518;
const SMALL_RESIZE_SHORT_EDGE: usize = 256;
const LARGE_RESIZE_SHORT_EDGE: usize = 592;
const IMAGE_MEAN: [f64; 3] = [0.485, 0.456, 0.406];
const IMAGE_STD: [f64; 3] = [0.229, 0.224, 0.225];
const BICUBIC_A: f64 = -0.5;
const MAX_FILTER_TAPS: usize = 4 * (MAX_SIDE / SMALL_RESIZE_SHORT_EDGE) + 1;

/// DINOv2 square input variants supported by Ember.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DinoInputSize {
    /// Official DINOv2 evaluation size: 224 crop after a 256 short-edge resize.
    Small224,
    /// Experimental 518 crop after a 592 short-edge resize.
    Large518,
}

impl DinoInputSize {
    /// Square tensor side in pixels.
    pub const fn side(self) -> usize {
        match self {
            Self::Small224 => SMALL_SIDE,
            Self::Large518 => LARGE_SIDE,
        }
    }

    /// Short edge used before center-cropping.
    pub const fn resize_short_edge(self) -> usize {
        match self {
            Self::Small224 => SMALL_RESIZE_SHORT_EDGE,
            Self::Large518 => LARGE_RESIZE_SHORT_EDGE,
        }
    }
}

/// Prepare row-major, interleaved sRGB RGB8 pixels for DINOv2.
///
/// The result is F32, channel-first, and shaped `[1, 3, side, side]`. Input dimensions
/// are checked before any allocation. Resampling is a separable cubic convolution with
/// `A = -0.5`, half-pixel source coordinates, a widened filter for downsampling,
/// clipped and renormalized edge taps, floor-rounded long-edge dimensions,
/// floor-centered crop offsets, and explicit round-to-nearest/clamp-to-`0..=255` RGB8
/// quantization before normalization. These choices are part of Ember's contract; see
/// the model document for the required numerical reference gate.
pub fn preprocess_rgb8(rgb: &[u8], width: usize, height: usize, size: DinoInputSize, device: &Device) -> Result<Tensor> {
    let pixels = width.checked_mul(height).ok_or_else(|| Error::Model("DINOv2 image dimensions overflow".into()))?;
    if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE || pixels > MAX_PIXELS {
        return Err(Error::Model("DINOv2 image dimensions exceed bounded RGB8 limits".into()));
    }
    let expected = pixels.checked_mul(3).ok_or_else(|| Error::Model("DINOv2 RGB8 length overflow".into()))?;
    if rgb.len() != expected {
        return Err(Error::Model(format!("DINOv2 RGB8 length {}, expected {expected} for {width} × {height}", rgb.len())));
    }

    let side = size.side();
    let short_edge = size.resize_short_edge();
    let (resized_width, resized_height) = resized_shape(width, height, short_edge)?;
    if resized_width < side || resized_height < side {
        return Err(Error::Model("DINOv2 resize is smaller than its center crop".into()));
    }
    let crop_x = (resized_width - side) / 2;
    let crop_y = (resized_height - side) / 2;
    let x_taps = build_taps(width, resized_width, crop_x, side)?;
    let y_taps = build_taps(height, resized_height, crop_y, side)?;
    let plane = side.checked_mul(side).ok_or_else(|| Error::Model("DINOv2 output plane overflow".into()))?;
    let total = plane.checked_mul(3).ok_or_else(|| Error::Model("DINOv2 output size overflow".into()))?;
    let mut accumulated = vec![[0.0f64; 3]; plane];

    // Build reverse vertical coverage so each required source row is filtered
    // horizontally once, rather than recomputing a 2-D tap product per output pixel.
    // This keeps pixel storage proportional to the fixed output plus one source row;
    // the coverage index itself is bounded by the 16,384-pixel input limit.
    let mut vertical_coverage = vec![Vec::<(usize, f64)>::new(); height];
    for (out_y, y_tap) in y_taps.iter().enumerate() {
        for (&source_y, &weight) in y_tap.indices.iter().zip(y_tap.weights.iter()) {
            vertical_coverage.get_mut(source_y).ok_or_else(|| Error::Model("DINOv2 vertical tap out of range".into()))?.push((out_y, weight));
        }
    }
    let mut horizontal = vec![[0.0f64; 3]; side];
    for source_y in 0..height {
        let coverage = vertical_coverage.get(source_y).ok_or_else(|| Error::Model("DINOv2 vertical coverage out of range".into()))?;
        if coverage.is_empty() {
            continue;
        }
        for (out_x, x_tap) in x_taps.iter().enumerate() {
            let mut channels = [0.0f64; 3];
            for (&source_x, &weight) in x_tap.indices.iter().zip(x_tap.weights.iter()) {
                let pixel = source_y
                    .checked_mul(width)
                    .and_then(|v| v.checked_add(source_x))
                    .and_then(|v| v.checked_mul(3))
                    .ok_or_else(|| Error::Model("DINOv2 RGB8 sample offset overflow".into()))?;
                for (channel, value) in channels.iter_mut().enumerate() {
                    let sample = *rgb
                        .get(pixel.checked_add(channel).ok_or_else(|| Error::Model("DINOv2 RGB8 channel offset overflow".into()))?)
                        .ok_or_else(|| Error::Model("DINOv2 RGB8 sample offset out of range".into()))?;
                    *value += weight * f64::from(sample);
                }
            }
            *horizontal.get_mut(out_x).ok_or_else(|| Error::Model("DINOv2 horizontal output out of range".into()))? = channels;
        }
        for &(out_y, y_weight) in coverage {
            for out_x in 0..side {
                let destination = out_y
                    .checked_mul(side)
                    .and_then(|v| v.checked_add(out_x))
                    .ok_or_else(|| Error::Model("DINOv2 output pixel offset overflow".into()))?;
                let source = *horizontal.get(out_x).ok_or_else(|| Error::Model("DINOv2 horizontal sample out of range".into()))?;
                let target = accumulated.get_mut(destination).ok_or_else(|| Error::Model("DINOv2 output offset out of range".into()))?;
                for channel in 0..3 {
                    target[channel] += y_weight * source[channel];
                }
            }
        }
    }

    let mut output = vec![0.0f32; total];
    for (destination, channels) in accumulated.into_iter().enumerate() {
        for (channel, value) in channels.into_iter().enumerate() {
            let encoded = value.round().clamp(0.0, 255.0);
            let normalized = (encoded / 255.0 - IMAGE_MEAN[channel]) / IMAGE_STD[channel];
            if !normalized.is_finite() {
                return Err(Error::Model("DINOv2 preprocessing produced non-finite data".into()));
            }
            let slot = channel
                .checked_mul(plane)
                .and_then(|v| v.checked_add(destination))
                .ok_or_else(|| Error::Model("DINOv2 output channel offset overflow".into()))?;
            let target = output.get_mut(slot).ok_or_else(|| Error::Model("DINOv2 output offset out of range".into()))?;
            *target = normalized as f32;
        }
    }
    Ok(Tensor::from_vec(output, (1, 3, side, side), device)?)
}

#[derive(Clone, Debug)]
struct Taps {
    indices: Vec<usize>,
    weights: Vec<f64>,
}

fn resized_shape(width: usize, height: usize, short_edge: usize) -> Result<(usize, usize)> {
    if width <= height {
        let long = height.checked_mul(short_edge).ok_or_else(|| Error::Model("DINOv2 resized height overflow".into()))? / width;
        Ok((short_edge, long.max(short_edge)))
    } else {
        let long = width.checked_mul(short_edge).ok_or_else(|| Error::Model("DINOv2 resized width overflow".into()))? / height;
        Ok((long.max(short_edge), short_edge))
    }
}

fn build_taps(source: usize, resized: usize, crop: usize, side: usize) -> Result<Vec<Taps>> {
    (0..side)
        .map(|output| {
            let coordinate = crop.checked_add(output).ok_or_else(|| Error::Model("DINOv2 crop coordinate overflow".into()))?;
            let scale = source as f64 / resized as f64;
            let filterscale = scale.max(1.0);
            let support = 2.0 * filterscale;
            let center = (coordinate as f64 + 0.5) * scale;
            if !center.is_finite() || !support.is_finite() {
                return Err(Error::Model("DINOv2 resize coordinate or support is non-finite".into()));
            }
            let left = (center - support + 0.5).trunc() as isize;
            let right = (center + support + 0.5).trunc() as isize;
            let xmin = left.max(0) as usize;
            let xmax = right.min(source as isize) as usize;
            let tap_count = xmax.checked_sub(xmin).ok_or_else(|| Error::Model("DINOv2 resize tap bounds are invalid".into()))?;
            if tap_count == 0 || tap_count > MAX_FILTER_TAPS {
                return Err(Error::Model("DINOv2 resize tap count is out of bounds".into()));
            }
            let mut indices = Vec::with_capacity(tap_count);
            let mut weights = Vec::with_capacity(tap_count);
            let mut sum = 0.0;
            for index in xmin..xmax {
                let weight = bicubic_filter((index as f64 - center + 0.5) / filterscale);
                if !weight.is_finite() {
                    return Err(Error::Model("DINOv2 resize weight is non-finite".into()));
                }
                indices.push(index);
                weights.push(weight);
                sum += weight;
            }
            if !sum.is_finite() || sum.abs() <= f64::EPSILON {
                return Err(Error::Model("DINOv2 resize weights cannot be normalized".into()));
            }
            for weight in &mut weights {
                *weight /= sum;
            }
            Ok(Taps { indices, weights })
        })
        .collect()
}

fn bicubic_filter(distance: f64) -> f64 {
    let x = distance.abs();
    if x <= 1.0 {
        ((BICUBIC_A + 2.0) * x - (BICUBIC_A + 3.0)) * x * x + 1.0
    } else if x < 2.0 {
        ((BICUBIC_A * x - 5.0 * BICUBIC_A) * x + 8.0 * BICUBIC_A) * x - 4.0 * BICUBIC_A
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sizes_expose_official_and_experimental_contracts() {
        assert_eq!(DinoInputSize::Small224.side(), 224);
        assert_eq!(DinoInputSize::Small224.resize_short_edge(), 256);
        assert_eq!(DinoInputSize::Large518.side(), 518);
        assert_eq!(DinoInputSize::Large518.resize_short_edge(), 592);
    }

    #[test]
    fn shape_channels_and_constant_values_are_stable() -> Result<()> {
        let rgb = [0u8, 128, 255];
        let tensor = preprocess_rgb8(&rgb, 1, 1, DinoInputSize::Small224, &Device::Cpu)?;
        assert_eq!(tensor.dtype(), DType::F32);
        assert_eq!(tensor.dims(), [1, 3, 224, 224]);
        let channels = tensor.squeeze(0)?.to_vec3::<f32>()?;
        let expected = [
            (0.0 - IMAGE_MEAN[0] as f32) / IMAGE_STD[0] as f32,
            (128.0 / 255.0 - IMAGE_MEAN[1] as f32) / IMAGE_STD[1] as f32,
            (1.0 - IMAGE_MEAN[2] as f32) / IMAGE_STD[2] as f32,
        ];
        for (channel, expected) in expected.into_iter().enumerate() {
            let value = channels
                .get(channel)
                .and_then(|plane| plane.first())
                .and_then(|row| row.first())
                .copied()
                .ok_or_else(|| Error::Model("test tensor shape".into()))?;
            assert!((value - expected).abs() < 1e-6);
        }
        Ok(())
    }

    #[test]
    fn large518_shape_and_constant_channel_values_are_stable() -> Result<()> {
        let rgb = [255u8, 255, 255];
        let tensor = preprocess_rgb8(&rgb, 1, 1, DinoInputSize::Large518, &Device::Cpu)?;
        assert_eq!(tensor.dtype(), DType::F32);
        assert_eq!(tensor.dims(), [1, 3, 518, 518]);
        let values = tensor.squeeze(0)?.to_vec3::<f32>()?;
        let expected = [
            (1.0 - IMAGE_MEAN[0] as f32) / IMAGE_STD[0] as f32,
            (1.0 - IMAGE_MEAN[1] as f32) / IMAGE_STD[1] as f32,
            (1.0 - IMAGE_MEAN[2] as f32) / IMAGE_STD[2] as f32,
        ];
        for (channel, expected) in expected.into_iter().enumerate() {
            let value = values
                .get(channel)
                .and_then(|plane| plane.first())
                .and_then(|row| row.first())
                .copied()
                .ok_or_else(|| Error::Model("test tensor shape".into()))?;
            assert!((value - expected).abs() < 1e-6);
        }
        Ok(())
    }

    #[test]
    fn malformed_lengths_and_bounds_are_rejected() {
        assert!(preprocess_rgb8(&[0; 2], 1, 1, DinoInputSize::Small224, &Device::Cpu).is_err());
        assert!(preprocess_rgb8(&[], 0, 1, DinoInputSize::Small224, &Device::Cpu).is_err());
        assert!(preprocess_rgb8(&[], usize::MAX, usize::MAX, DinoInputSize::Small224, &Device::Cpu).is_err());
        assert!(preprocess_rgb8(&[], usize::MAX, 1, DinoInputSize::Small224, &Device::Cpu).is_err());
        assert!(preprocess_rgb8(&[], 16_385, 1, DinoInputSize::Small224, &Device::Cpu).is_err());
        assert!(preprocess_rgb8(&[], 8_000, 4_001, DinoInputSize::Small224, &Device::Cpu).is_err());
    }

    #[test]
    fn downsample_widens_bicubic_support_and_normalizes_edges() -> Result<()> {
        let taps = build_taps(1_024, 256, 16, 224)?;
        assert!(taps.iter().any(|tap| tap.indices.len() > 4));
        for tap in taps.iter().take(3).chain(taps.iter().rev().take(3)) {
            let sum: f64 = tap.weights.iter().sum();
            assert!((sum - 1.0).abs() < 1e-12);
            assert!(tap.indices.iter().all(|index| *index < 1_024));
        }
        let maximum = build_taps(16_384, 256, 0, 224)?
            .iter()
            .map(|tap| tap.indices.len())
            .max()
            .ok_or_else(|| Error::Model("missing maximum test tap".into()))?;
        assert_eq!(maximum, 256);
        Ok(())
    }

    #[test]
    fn large518_odd_nonsquare_resize_and_crop_shape_are_stable() -> Result<()> {
        let (resized_width, resized_height) = resized_shape(101, 77, LARGE_RESIZE_SHORT_EDGE)?;
        assert_eq!((resized_width, resized_height), (776, 592));
        assert_eq!((resized_width - LARGE_SIDE) / 2, 129);
        assert_eq!((resized_height - LARGE_SIDE) / 2, 37);
        Ok(())
    }

    #[test]
    fn nonsquare_resize_keeps_center_crop_in_middle() -> Result<()> {
        let (resized_width, resized_height) = resized_shape(8, 4, SMALL_RESIZE_SHORT_EDGE)?;
        assert_eq!((resized_width, resized_height), (512, 256));
        assert_eq!((resized_width - SMALL_SIDE) / 2, 144);
        assert_eq!((resized_height - SMALL_SIDE) / 2, 16);

        let taps = build_taps(8, resized_width, 144, SMALL_SIDE)?;
        let first = taps.first().ok_or_else(|| Error::Model("missing test taps".into()))?;
        let middle = taps.get(SMALL_SIDE / 2).ok_or_else(|| Error::Model("missing test taps".into()))?;
        assert!(first.indices.first().copied().unwrap_or(usize::MAX) < middle.indices.first().copied().unwrap_or(0));

        let mut rgb = vec![0u8; 8 * 4 * 3];
        for y in 0..4 {
            for x in 0..8 {
                let pixel = (y * 8 + x) * 3;
                if let Some(red) = rgb.get_mut(pixel) {
                    *red = (x * 32) as u8;
                }
            }
        }
        let tensor = preprocess_rgb8(&rgb, 8, 4, DinoInputSize::Small224, &Device::Cpu)?;
        let values = tensor.squeeze(0)?.to_vec3::<f32>()?;
        let red = values.first().ok_or_else(|| Error::Model("missing red plane".into()))?;
        let left = red.get(112).and_then(|row| row.first()).copied().ok_or_else(|| Error::Model("missing left crop sample".into()))?;
        let center = red.get(112).and_then(|row| row.get(112)).copied().ok_or_else(|| Error::Model("missing center crop sample".into()))?;
        let right = red.get(112).and_then(|row| row.get(223)).copied().ok_or_else(|| Error::Model("missing right crop sample".into()))?;
        assert!(left < center && center < right);
        Ok(())
    }

    #[test]
    fn downsampled_checkerboard_is_antialiased() -> Result<()> {
        let side = 1_024usize;
        let mut rgb = vec![0u8; side * side * 3];
        for y in 0..side {
            for x in 0..side {
                let value = if (x + y) % 2 == 0 { 0 } else { 255 };
                let pixel = (y * side + x) * 3;
                for channel in 0..3 {
                    if let Some(sample) = rgb.get_mut(pixel + channel) {
                        *sample = value;
                    }
                }
            }
        }
        let tensor = preprocess_rgb8(&rgb, side, side, DinoInputSize::Small224, &Device::Cpu)?;
        let values = tensor.squeeze(0)?.to_vec3::<f32>()?;
        let red = values
            .first()
            .and_then(|plane| plane.get(112))
            .and_then(|row| row.get(112))
            .copied()
            .ok_or_else(|| Error::Model("missing checkerboard sample".into()))?;
        let decoded = (f64::from(red) * IMAGE_STD[0] + IMAGE_MEAN[0]) * 255.0;
        assert!((100.0..=155.0).contains(&decoded));
        Ok(())
    }
}
