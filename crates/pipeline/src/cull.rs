//! Assisted culling measurements on a decoded source (any size; measured on a fixed-size copy so
//! scores compare across photos): focus, clipping, and a tiny signature for finding similar
//! shots (bursts). Classical image statistics — no learned models.

pub mod report;

use lightcraft_raster::Rgb32f;
use std::fmt;

/// Long edge the measurements run at.
const EDGE: usize = 512;

/// Why a classical culling measurement could not be computed.
///
/// Measurements operate on decoded pixels, so dimensions, storage length, and every channel
/// must be coherent before any indexing or score calculation begins.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MeasurementError {
    /// Width or height is zero.
    EmptyImage,
    /// Width × height (or an internal bounded calculation) does not fit in `usize`.
    DimensionOverflow,
    /// Declared dimensions do not match interleaved pixel storage.
    DataLength { expected: usize, actual: usize },
    /// A source channel is NaN or ±infinity.
    NonFinitePixel { index: usize, channel: usize },
    /// A signature contains NaN or ±infinity.
    NonFiniteSignature { index: usize },
}

impl fmt::Display for MeasurementError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyImage => f.write_str("culling image has empty dimensions"),
            Self::DimensionOverflow => f.write_str("culling image dimensions overflow"),
            Self::DataLength { expected, actual } => write!(f, "culling image has {actual} pixels, expected {expected}"),
            Self::NonFinitePixel { index, channel } => write!(f, "culling pixel {index} channel {channel} is not finite"),
            Self::NonFiniteSignature { index } => write!(f, "culling signature value {index} is not finite"),
        }
    }
}

impl std::error::Error for MeasurementError {}

/// Classical culling measurements computed from one validated decoded image.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Measurements {
    pub sharpness: f32,
    pub clipped: f32,
    pub signature: [f32; 64],
}

/// Validate decoded image storage before running any measurement.
pub fn validate_image(img: &Rgb32f) -> Result<(), MeasurementError> {
    if img.width == 0 || img.height == 0 {
        return Err(MeasurementError::EmptyImage);
    }
    let expected = img.width.checked_mul(img.height).ok_or(MeasurementError::DimensionOverflow)?;
    if img.data.len() != expected {
        return Err(MeasurementError::DataLength { expected, actual: img.data.len() });
    }
    for (index, pixel) in img.data.iter().enumerate() {
        for (channel, value) in pixel.iter().enumerate() {
            if !value.is_finite() {
                return Err(MeasurementError::NonFinitePixel { index, channel });
            }
        }
    }
    Ok(())
}

/// Display-ish luminance (sRGB-encoded) of `img` resampled to fit `EDGE` (box filter).
fn luma(img: &Rgb32f) -> Result<(Vec<f32>, usize, usize), MeasurementError> {
    validate_image(img)?;
    let (w, h) = (img.width, img.height);
    let s = (EDGE as f64 / w.max(h) as f64).min(1.0);
    let (ow, oh) = (((w as f64 * s).round() as usize).max(8), ((h as f64 * s).round() as usize).max(8));
    let output_len = ow.checked_mul(oh).ok_or(MeasurementError::DimensionOverflow)?;
    let mut out = vec![0f32; output_len];
    for (y, row) in out.chunks_mut(ow).enumerate() {
        let y0 = y.checked_mul(h).ok_or(MeasurementError::DimensionOverflow)? / oh;
        let next_y = (y + 1).checked_mul(h).ok_or(MeasurementError::DimensionOverflow)? / oh;
        let y1 = next_y.max(y0.saturating_add(1)).min(h);
        for (x, v) in row.iter_mut().enumerate() {
            let x0 = x.checked_mul(w).ok_or(MeasurementError::DimensionOverflow)? / ow;
            let next_x = (x + 1).checked_mul(w).ok_or(MeasurementError::DimensionOverflow)? / ow;
            let x1 = next_x.max(x0.saturating_add(1)).min(w);
            let mut acc = 0.0f64;
            for yy in y0..y1 {
                for xx in x0..x1 {
                    let c = img.data[yy * w + xx];
                    acc += 0.2627 * c[0] as f64 + 0.678 * c[1] as f64 + 0.0593 * c[2] as f64;
                }
            }
            let count = (y1 - y0).checked_mul(x1 - x0).ok_or(MeasurementError::DimensionOverflow)? as f64;
            let l = (acc / count).clamp(0.0, 1.0) as f32;
            *v = lightcraft_color::transfer::linear_to_srgb(l);
        }
    }
    Ok((out, ow, oh))
}

/// Focus 0..100: detail energy (Laplacian) of the sharpest areas — the 90th percentile of
/// 16 × 16-pixel blocks, so a sharp subject on a soft background still reads as sharp.
pub fn sharpness(img: &Rgb32f) -> f32 {
    sharpness_checked(img).unwrap_or(f32::NAN)
}

fn sharpness_from_luma(l: &[f32], w: usize, h: usize) -> f32 {
    let b = 16;
    let mut blocks = Vec::new();
    for by in (1..h.saturating_sub(1)).step_by(b) {
        for bx in (1..w.saturating_sub(1)).step_by(b) {
            let mut e = 0.0f32;
            let mut n = 0;
            for y in by..(by + b).min(h - 1) {
                for x in bx..(bx + b).min(w - 1) {
                    let c = l[y * w + x];
                    let lap = 4.0 * c - l[y * w + x - 1] - l[y * w + x + 1] - l[(y - 1) * w + x] - l[(y + 1) * w + x];
                    e += lap * lap;
                    n += 1;
                }
            }
            if n > 0 {
                blocks.push(e / n as f32);
            }
        }
    }
    if blocks.is_empty() {
        return 0.0;
    }
    let k = ((blocks.len() as f32 * 0.9) as usize).min(blocks.len() - 1);
    let (_, v, _) = blocks.select_nth_unstable_by(k, f32::total_cmp);
    // map energy to 0..100: a crisp, ordinary photo at 512 px reads ≈ 50–80 (calibrated on the
    // CC0 corpus), soft ones well below
    100.0 * (1.0 - (-*v / 0.012).exp())
}

/// Fallible form of [`sharpness`].
pub fn sharpness_checked(img: &Rgb32f) -> Result<f32, MeasurementError> {
    let (l, w, h) = luma(img)?;
    Ok(sharpness_from_luma(&l, w, h))
}

/// Share of clipped pixels (crushed blacks + blown highlights), 0..1.
pub fn clipped(img: &Rgb32f) -> f32 {
    clipped_checked(img).unwrap_or(f32::NAN)
}

fn clipped_from_validated(img: &Rgb32f) -> f32 {
    let n = img.data.len();
    let c = img.data.iter().filter(|p| p.iter().any(|v| *v >= 0.985) || p.iter().all(|v| *v <= 0.002)).count();
    (c as f64 / n as f64) as f32
}

/// Fallible form of [`clipped`].
pub fn clipped_checked(img: &Rgb32f) -> Result<f32, MeasurementError> {
    validate_image(img)?;
    Ok(clipped_from_validated(img))
}

/// A tiny look-alike signature: 8 × 8 luminance, normalised to zero mean and unit norm.
pub fn signature(img: &Rgb32f) -> [f32; 64] {
    signature_checked(img).unwrap_or([f32::NAN; 64])
}

fn signature_from_luma(l: &[f32], w: usize, h: usize) -> [f32; 64] {
    let mut s = [0f32; 64];
    for (i, v) in s.iter_mut().enumerate() {
        let (cx, cy) = (i % 8, i / 8);
        let (x0, x1) = (cx * w / 8, (cx + 1) * w / 8);
        let (y0, y1) = (cy * h / 8, (cy + 1) * h / 8);
        let mut acc = 0.0;
        for y in y0..y1 {
            for x in x0..x1 {
                acc += l[y * w + x];
            }
        }
        *v = acc / ((x1 - x0) * (y1 - y0)).max(1) as f32;
    }
    let mean = s.iter().sum::<f32>() / 64.0;
    s.iter_mut().for_each(|v| *v -= mean);
    let norm = s.iter().map(|v| v * v).sum::<f32>().sqrt().max(1e-6);
    s.iter_mut().for_each(|v| *v /= norm);
    s
}

/// Fallible form of [`signature`].
pub fn signature_checked(img: &Rgb32f) -> Result<[f32; 64], MeasurementError> {
    let (l, w, h) = luma(img)?;
    Ok(signature_from_luma(&l, w, h))
}

/// Compute sharpness, clipping, and signature with one image validation and one luma resample.
pub fn measure_checked(img: &Rgb32f) -> Result<Measurements, MeasurementError> {
    let (l, w, h) = luma(img)?;
    Ok(Measurements { sharpness: sharpness_from_luma(&l, w, h), clipped: clipped_from_validated(img), signature: signature_from_luma(&l, w, h) })
}

/// Similarity of two signatures: 1 = the same picture, 0 = unrelated.
pub fn similarity(a: &[f32; 64], b: &[f32; 64]) -> f32 {
    similarity_checked(a, b).unwrap_or(f32::NAN)
}

/// Fallible form of [`similarity`], rejecting malformed cached signatures.
pub fn similarity_checked(a: &[f32; 64], b: &[f32; 64]) -> Result<f32, MeasurementError> {
    for (index, value) in a.iter().chain(b.iter()).enumerate() {
        if !value.is_finite() {
            return Err(MeasurementError::NonFiniteSignature { index });
        }
    }
    let dot = a.iter().zip(b).map(|(x, y)| *x as f64 * *y as f64).sum::<f64>();
    Ok(dot.clamp(-1.0, 1.0) as f32)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn blur(img: &Rgb32f, r: usize) -> Rgb32f {
        let (w, h) = (img.width, img.height);
        let mut out = img.clone();
        for y in 0..h {
            for x in 0..w {
                let mut acc = [0f32; 3];
                let mut n = 0.0;
                for yy in y.saturating_sub(r)..(y + r + 1).min(h) {
                    for xx in x.saturating_sub(r)..(x + r + 1).min(w) {
                        let c = img.data[yy * w + xx];
                        acc = [acc[0] + c[0], acc[1] + c[1], acc[2] + c[2]];
                        n += 1.0;
                    }
                }
                out.data[y * w + x] = acc.map(|v| v / n);
            }
        }
        out
    }

    #[test]
    fn blur_lowers_focus_and_signatures_match() {
        let img = lightcraft_scenes::demo_library()[0].render(480, 320);
        let soft = blur(&img, 4);
        let (a, b) = (sharpness(&img), sharpness(&soft));
        assert!(a > b + 10.0, "sharp {a} vs blurred {b}");
        assert!(similarity(&signature(&img), &signature(&soft)) > 0.95, "the same picture");
        let other = lightcraft_scenes::demo_library()[5].render(480, 320);
        assert!(similarity(&signature(&img), &signature(&other)) < 0.9);
        assert!(clipped(&img) < 0.5);
    }

    #[test]
    fn malformed_images_are_reported_without_indexing() {
        let mismatch = Rgb32f { width: 2, height: 2, data: vec![[0.5; 3]] };
        assert_eq!(validate_image(&mismatch), Err(MeasurementError::DataLength { expected: 4, actual: 1 }));
        assert_eq!(measure_checked(&mismatch), Err(MeasurementError::DataLength { expected: 4, actual: 1 }));
        assert_eq!(sharpness_checked(&mismatch), Err(MeasurementError::DataLength { expected: 4, actual: 1 }));
        assert!(sharpness(&mismatch).is_nan());
        assert!(clipped(&mismatch).is_nan());
        assert!(signature(&mismatch).iter().all(|v| v.is_nan()));

        let empty = Rgb32f { width: 0, height: 0, data: Vec::new() };
        assert_eq!(signature_checked(&empty), Err(MeasurementError::EmptyImage));
        assert_eq!(clipped_checked(&empty), Err(MeasurementError::EmptyImage));

        let overflow = Rgb32f { width: usize::MAX, height: 2, data: Vec::new() };
        assert_eq!(validate_image(&overflow), Err(MeasurementError::DimensionOverflow));

        let nonfinite = Rgb32f { width: 1, height: 1, data: vec![[0.5, f32::INFINITY, 0.5]] };
        assert_eq!(validate_image(&nonfinite), Err(MeasurementError::NonFinitePixel { index: 0, channel: 1 }));
        assert_eq!(sharpness_checked(&nonfinite), Err(MeasurementError::NonFinitePixel { index: 0, channel: 1 }));
    }

    #[test]
    fn tiny_images_have_finite_checked_measurements() {
        let img = Rgb32f { width: 1, height: 1, data: vec![[0.5; 3]] };
        let sharp = sharpness_checked(&img).expect("valid tiny image");
        let clip = clipped_checked(&img).expect("valid tiny image");
        let sig = signature_checked(&img).expect("valid tiny image");
        let combined = measure_checked(&img).expect("valid tiny image");
        assert!(sharp.is_finite());
        assert!(clip.is_finite());
        assert!(sig.iter().all(|v| v.is_finite()));
        assert_eq!(combined, Measurements { sharpness: sharp, clipped: clip, signature: sig });
        assert_eq!(similarity_checked(&sig, &sig), Ok(0.0));
    }

    #[test]
    fn nonfinite_signatures_are_reported() {
        let bad = [f32::NAN; 64];
        assert_eq!(similarity_checked(&bad, &[0.0; 64]), Err(MeasurementError::NonFiniteSignature { index: 0 }));
        assert!(similarity(&bad, &[0.0; 64]).is_nan());
    }
}
