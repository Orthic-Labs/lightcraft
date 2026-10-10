//! Bounded RGB input, face crops, and coordinate projection for MediaPipe Face Landmarker.
//!
//! The graph source is pinned to MediaPipe `458200ffced12a9db596ddf1cce3380ea05f71fb`.
//! `face_detector_graph.cc` selects aspect-preserving preprocessing with `BORDER_ZERO`;
//! v1 detector metadata maps RGB with `(x - 127.5) / 127.5` to `[-1, 1]`. Landmark
//! metadata maps RGB with `(x - 0) / 255` to `[0, 1]`; its graph uses ImageToTensor's
//! default non-aspect-preserving ROI path and default `BORDER_REPLICATE`.
//!
//! Host sampling is intentionally explicit: image coordinates use an upper-left origin,
//! source pixel centers are integer coordinates, and destination samples are taken at
//! `(x, y)` before bilinear interpolation. This follows the coordinate convention used by
//! the pinned OpenCV converter's `[0,width] × [0,height]` destination corners, while using
//! floating bilinear weights instead of OpenCV's 1/32-quantized interpolation tables.
//! Numerical parity remains experimental until checked against the reference converter.
//!
//! Source reference (Apache-2.0, Copyright The MediaPipe Authors):
//! `tasks/cc/vision/face_detector/face_detector_graph.cc`,
//! `tasks/cc/vision/face_landmarker/face_landmarks_detector_graph.cc`,
//! `tasks/cc/components/processors/image_preprocessing_graph.cc`,
//! `calculators/tensor/image_to_tensor_utils.cc`,
//! `calculators/tensor/image_to_tensor_converter_opencv.cc`,
//! `calculators/util/detections_to_rects_calculator.cc`,
//! `calculators/util/rect_transformation_calculator.cc`,
//! `calculators/util/landmark_projection_calculator.cc`, and
//! `calculators/tensor/tensors_to_landmarks_calculator.cc`, all at the pinned commit above.
//! This is an independent Rust implementation of those bounded contracts; no C++ source is
//! copied into this crate.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{Device, Tensor};

use crate::{Error, Result};

const MAX_SIDE: usize = 16_384;
const MAX_PIXELS: usize = 32_000_000;
const DETECTOR_SIDE: usize = 128;
const LANDMARK_SIDE: usize = 256;
const LANDMARK_COUNT: usize = 478;
const CHANNELS: usize = 3;
const MAX_NORMALIZED_COORD: f32 = 8.0;
const MAX_RECT_SIZE: f32 = 8.0;

/// A borrowed, validated, row-major interleaved RGB8 image.
pub struct RgbImage<'a> {
    width: usize,
    height: usize,
    rgb: &'a [u8],
}

impl<'a> RgbImage<'a> {
    /// Validate an RGB8 image without copying its pixels.
    pub fn new(width: usize, height: usize, rgb: &'a [u8]) -> Result<Self> {
        let pixels = width.checked_mul(height).ok_or_else(|| Error::Model("MediaPipe image dimensions overflow".into()))?;
        if width == 0 || height == 0 || width > MAX_SIDE || height > MAX_SIDE || pixels > MAX_PIXELS {
            return Err(Error::Model("MediaPipe image dimensions exceed bounded RGB8 limits".into()));
        }
        let expected = pixels.checked_mul(CHANNELS).ok_or_else(|| Error::Model("MediaPipe RGB8 length overflow".into()))?;
        if rgb.len() != expected {
            return Err(Error::Model(format!("MediaPipe RGB8 length {}, expected {expected} for {width} × {height}", rgb.len())));
        }
        Ok(Self { width, height, rgb })
    }

    pub fn width(&self) -> usize {
        self.width
    }

    pub fn height(&self) -> usize {
        self.height
    }
}

/// A normalized, possibly rotated region of interest.
#[derive(Clone, Copy, Debug)]
pub struct NormalizedRect {
    pub center: [f32; 2],
    pub size: [f32; 2],
    pub rotation: f32,
}

/// Tensor plus the transform needed to return model coordinates to original-image space.
pub struct PreparedImage {
    pub tensor: Tensor,
    pub rect: NormalizedRect,
    orig_width: usize,
    orig_height: usize,
    /// Normalized padding on left, top, right, bottom in tensor coordinates.
    letterbox: [f32; 4],
}

impl PreparedImage {
    /// Project one normalized tensor XY coordinate into original-image coordinates.
    pub fn project_xy(&self, point: [f32; 2]) -> Result<[f32; 2]> {
        validate_point(point, "MediaPipe tensor coordinate")?;
        tensor_to_source(point, self.rect, self.letterbox, self.orig_width, self.orig_height)
    }

    /// Project one normalized tensor XYZ coordinate, scaling Z by source ROI width.
    pub fn project_xyz(&self, point: [f32; 3]) -> Result<[f32; 3]> {
        validate_point([point[0], point[1]], "MediaPipe tensor coordinate")?;
        if !point[2].is_finite() || point[2].abs() > MAX_NORMALIZED_COORD {
            return Err(Error::Model("MediaPipe tensor Z coordinate is invalid".into()));
        }
        let xy = self.project_xy([point[0], point[1]])?;
        let image_width = self.orig_width as f32;
        let image_height = self.orig_height as f32;
        let aspect = image_width / image_height;
        let content_width = 1.0 - self.letterbox[0] - self.letterbox[2];
        let angle = self.rect.rotation;
        let (_, cos) = angle.sin_cos();
        let sin = angle.sin();
        let z_scale = self.rect.size[0] / content_width * (cos.mul_add(cos, (aspect * sin) * (aspect * sin))).sqrt();
        if !z_scale.is_finite() || z_scale <= 0.0 {
            return Err(Error::Model("MediaPipe projected Z scale is invalid".into()));
        }
        let output = [xy[0], xy[1], point[2] * z_scale];
        if output.iter().any(|value| !value.is_finite() || value.abs() > MAX_NORMALIZED_COORD * MAX_RECT_SIZE) {
            return Err(Error::Model("MediaPipe projected XYZ coordinate is invalid".into()));
        }
        Ok(output)
    }

    /// Project a detector-space detection into original-image normalized coordinates.
    pub fn project_detection(&self, detection: &crate::mediapipe_detector::Detection) -> Result<crate::mediapipe_detector::Detection> {
        if !detection.score.is_finite() {
            return Err(Error::Model("MediaPipe detection score is non-finite".into()));
        }
        let [xmin, ymin, width, height] = detection.bounds;
        validate_rect_values([xmin, ymin], [width, height], "MediaPipe detection bounds")?;
        let corners = [
            self.project_xy([xmin, ymin])?,
            self.project_xy([xmin + width, ymin])?,
            self.project_xy([xmin, ymin + height])?,
            self.project_xy([xmin + width, ymin + height])?,
        ];
        let mut min_x = corners[0][0];
        let mut max_x = corners[0][0];
        let mut min_y = corners[0][1];
        let mut max_y = corners[0][1];
        for corner in corners.iter().skip(1) {
            min_x = min_x.min(corner[0]);
            max_x = max_x.max(corner[0]);
            min_y = min_y.min(corner[1]);
            max_y = max_y.max(corner[1]);
        }
        let keypoints = detection.keypoints.iter().map(|point| self.project_xy(*point)).collect::<Result<Vec<_>>>()?;
        let keypoints: [[f32; 2]; 6] = keypoints.try_into().map_err(|_| Error::Model("MediaPipe detection keypoint count changed".into()))?;
        let projected_width = max_x - min_x;
        let projected_height = max_y - min_y;
        validate_rect_values([min_x, min_y], [projected_width, projected_height], "MediaPipe projected detection")?;
        Ok(crate::mediapipe_detector::Detection { score: detection.score, bounds: [min_x, min_y, projected_width, projected_height], keypoints })
    }
}

/// Prepare a 128 × 128 detector tensor using square, aspect-preserving zero letterboxing.
pub fn detector_input(image: &RgbImage<'_>, device: &Device) -> Result<PreparedImage> {
    let rect = NormalizedRect { center: [0.5, 0.5], size: [1.0, 1.0], rotation: 0.0 };
    prepare(image, rect, DETECTOR_SIDE, true, BorderMode::Zero, PixelRange::MinusOneToOne, device)
}

/// Convert a detector rectangle into the rotated 1.5× face crop used by Face Landmarker.
pub fn face_rect(image: &RgbImage<'_>, detection: &crate::mediapipe_detector::Detection) -> Result<NormalizedRect> {
    if !detection.score.is_finite() {
        return Err(Error::Model("MediaPipe face detection score is non-finite".into()));
    }
    let [xmin, ymin, width, height] = detection.bounds;
    validate_rect_values([xmin, ymin], [width, height], "MediaPipe face detection")?;
    let eye0 = detection.keypoints.first().copied().ok_or_else(|| Error::Model("MediaPipe detection has no left-eye keypoint".into()))?;
    let eye1 = detection.keypoints.get(1).copied().ok_or_else(|| Error::Model("MediaPipe detection has no right-eye keypoint".into()))?;
    validate_point(eye0, "MediaPipe left-eye keypoint")?;
    validate_point(eye1, "MediaPipe right-eye keypoint")?;
    let dx = (eye1[0] - eye0[0]) * image.width as f32;
    let dy = (eye1[1] - eye0[1]) * image.height as f32;
    if !dx.is_finite() || !dy.is_finite() || dx.abs() + dy.abs() <= f32::EPSILON {
        return Err(Error::Model("MediaPipe eye keypoints do not define a rotation".into()));
    }
    let rotation = normalize_radians(dy.atan2(dx));
    let size = [width * 1.5, height * 1.5];
    validate_rect_values([xmin + width * 0.5, ymin + height * 0.5], size, "MediaPipe face crop")?;
    Ok(NormalizedRect { center: [xmin + width * 0.5, ymin + height * 0.5], size, rotation })
}

/// Prepare a 256 × 256 landmark tensor from a rotated ROI, using replicate borders.
pub fn landmark_input(image: &RgbImage<'_>, rect: &NormalizedRect, device: &Device) -> Result<PreparedImage> {
    prepare(image, *rect, LANDMARK_SIDE, false, BorderMode::Replicate, PixelRange::ZeroToOne, device)
}

/// Project 478 raw Face Landmarker triples into original-image normalized XYZ coordinates.
pub fn project_landmarks(raw: &[f32], prepared: &PreparedImage) -> Result<Vec<[f32; 3]>> {
    let expected = LANDMARK_COUNT.checked_mul(3).ok_or_else(|| Error::Model("MediaPipe landmark count overflow".into()))?;
    if raw.len() != expected {
        return Err(Error::Model(format!("MediaPipe landmark output length {}, expected {expected}", raw.len())));
    }
    raw.chunks_exact(3)
        .map(|triple| prepared.project_xyz([triple[0] / LANDMARK_SIDE as f32, triple[1] / LANDMARK_SIDE as f32, triple[2] / LANDMARK_SIDE as f32]))
        .collect()
}

#[derive(Clone, Copy)]
enum BorderMode {
    Zero,
    Replicate,
}

#[derive(Clone, Copy)]
enum PixelRange {
    MinusOneToOne,
    ZeroToOne,
}

fn prepare(
    image: &RgbImage<'_>,
    rect: NormalizedRect,
    side: usize,
    keep_aspect: bool,
    border: BorderMode,
    range: PixelRange,
    device: &Device,
) -> Result<PreparedImage> {
    validate_rect(rect)?;
    let letterbox = if keep_aspect { letterbox_padding(image, rect)? } else { [0.0; 4] };
    let element_count = side
        .checked_mul(side)
        .and_then(|value| value.checked_mul(CHANNELS))
        .ok_or_else(|| Error::Model("MediaPipe output tensor dimensions overflow".into()))?;
    let mut output = Vec::new();
    output.try_reserve_exact(element_count).map_err(|_| Error::Model("MediaPipe output tensor allocation failed".into()))?;
    for y in 0..side {
        for x in 0..side {
            let point = [x as f32 / side as f32, y as f32 / side as f32];
            let source = tensor_to_source(point, rect, letterbox, image.width, image.height)?;
            let pixel = sample_rgb(image, source[0], source[1], border)?;
            for value in pixel {
                let normalized = match range {
                    PixelRange::MinusOneToOne => value / 127.5 - 1.0,
                    PixelRange::ZeroToOne => value / 255.0,
                };
                if !normalized.is_finite() {
                    return Err(Error::Model("MediaPipe preprocessing produced non-finite data".into()));
                }
                output.push(normalized);
            }
        }
    }
    let tensor = Tensor::from_vec(output, (1, side, side, CHANNELS), device)?;
    Ok(PreparedImage { tensor, rect, orig_width: image.width, orig_height: image.height, letterbox })
}

fn tensor_to_source(point: [f32; 2], rect: NormalizedRect, letterbox: [f32; 4], image_width: usize, image_height: usize) -> Result<[f32; 2]> {
    let content_width = 1.0 - letterbox[0] - letterbox[2];
    let content_height = 1.0 - letterbox[1] - letterbox[3];
    if !(content_width > 0.0 && content_height > 0.0 && content_width.is_finite() && content_height.is_finite()) {
        return Err(Error::Model("MediaPipe letterbox dimensions are invalid".into()));
    }
    if image_width == 0 || image_height == 0 {
        return Err(Error::Model("MediaPipe source image dimensions are invalid".into()));
    }
    let local_x = (point[0] - letterbox[0]) / content_width - 0.5;
    let local_y = (point[1] - letterbox[1]) / content_height - 0.5;
    let aspect_x = image_height as f32 / image_width as f32;
    let aspect_y = image_width as f32 / image_height as f32;
    let (sin, cos) = rect.rotation.sin_cos();
    let source = [
        rect.center[0] + local_x * rect.size[0] * cos - local_y * rect.size[1] * aspect_x * sin,
        rect.center[1] + local_x * rect.size[0] * aspect_y * sin + local_y * rect.size[1] * cos,
    ];
    validate_point(source, "MediaPipe source coordinate")?;
    Ok(source)
}

fn sample_rgb(image: &RgbImage<'_>, x: f32, y: f32, border: BorderMode) -> Result<[f32; 3]> {
    let x = x * image.width as f32;
    let y = y * image.height as f32;
    if !x.is_finite() || !y.is_finite() || x.abs() > MAX_SIDE as f32 * MAX_NORMALIZED_COORD || y.abs() > MAX_SIDE as f32 * MAX_NORMALIZED_COORD {
        return Err(Error::Model("MediaPipe source sample coordinate is invalid".into()));
    }
    let x0 = x.floor() as isize;
    let y0 = y.floor() as isize;
    let fx = x - x0 as f32;
    let fy = y - y0 as f32;
    let mut output = [0.0f32; 3];
    for (ix, wx) in [(x0, 1.0 - fx), (x0.saturating_add(1), fx)] {
        for (iy, wy) in [(y0, 1.0 - fy), (y0.saturating_add(1), fy)] {
            let weight = wx * wy;
            if weight == 0.0 {
                continue;
            }
            let Some(index) = source_index(image, ix, iy, border)? else { continue };
            for channel in 0..CHANNELS {
                let offset = index.checked_add(channel).ok_or_else(|| Error::Model("MediaPipe RGB8 sample index overflow".into()))?;
                let value = *image.rgb.get(offset).ok_or_else(|| Error::Model("MediaPipe RGB8 sample index is out of range".into()))?;
                output[channel] += weight * f32::from(value);
            }
        }
    }
    Ok(output.map(|value| value.clamp(0.0, 255.0)))
}

fn source_index(image: &RgbImage<'_>, x: isize, y: isize, border: BorderMode) -> Result<Option<usize>> {
    let (x, y) = match border {
        BorderMode::Zero if x < 0 || y < 0 || x >= image.width as isize || y >= image.height as isize => return Ok(None),
        BorderMode::Zero => (x, y),
        BorderMode::Replicate => (x.clamp(0, image.width as isize - 1), y.clamp(0, image.height as isize - 1)),
    };
    let index = (y as usize)
        .checked_mul(image.width)
        .and_then(|value| value.checked_add(x as usize))
        .and_then(|value| value.checked_mul(CHANNELS))
        .ok_or_else(|| Error::Model("MediaPipe source sample index overflow".into()))?;
    Ok(Some(index))
}

fn letterbox_padding(image: &RgbImage<'_>, rect: NormalizedRect) -> Result<[f32; 4]> {
    let roi_width = rect.size[0] * image.width as f32;
    let roi_height = rect.size[1] * image.height as f32;
    if !(roi_width.is_finite() && roi_height.is_finite() && roi_width > 0.0 && roi_height > 0.0) {
        return Err(Error::Model("MediaPipe ROI dimensions are invalid".into()));
    }
    if roi_width > roi_height {
        // A wide ROI needs top/bottom padding when fitted into a square tensor.
        let pad = (1.0 - roi_height / roi_width) * 0.5;
        Ok([0.0, pad, 0.0, pad])
    } else {
        // A tall ROI needs left/right padding when fitted into a square tensor.
        let pad = (1.0 - roi_width / roi_height) * 0.5;
        Ok([pad, 0.0, pad, 0.0])
    }
}

fn validate_rect(rect: NormalizedRect) -> Result<()> {
    validate_rect_values(rect.center, rect.size, "MediaPipe normalized rect")?;
    if !rect.rotation.is_finite() || rect.rotation.abs() > 1_000.0 {
        return Err(Error::Model("MediaPipe normalized rect rotation is invalid".into()));
    }
    Ok(())
}

fn validate_rect_values(center: [f32; 2], size: [f32; 2], role: &str) -> Result<()> {
    validate_point(center, role)?;
    if size.iter().any(|value| !value.is_finite() || *value <= 0.0 || *value > MAX_RECT_SIZE) {
        return Err(Error::Model(format!("{role} size is invalid")));
    }
    Ok(())
}

fn validate_point(point: [f32; 2], role: &str) -> Result<()> {
    if point.iter().any(|value| !value.is_finite() || value.abs() > MAX_NORMALIZED_COORD) {
        return Err(Error::Model(format!("{role} is invalid")));
    }
    Ok(())
}

fn normalize_radians(angle: f32) -> f32 {
    let two_pi = std::f32::consts::TAU;
    (angle + std::f32::consts::PI).rem_euclid(two_pi) - std::f32::consts::PI
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_image_rejects_bad_lengths_and_bounds() {
        let short = [0u8, 1];
        let empty: [u8; 0] = [];
        assert!(RgbImage::new(1, 1, &short).is_err());
        assert!(RgbImage::new(0, 1, &empty).is_err());
        assert!(RgbImage::new(16_385, 1, &empty).is_err());
        assert!(RgbImage::new(8_000, 4_001, &empty).is_err());
    }

    #[test]
    fn detector_input_is_nhwc_and_normalized() -> Result<()> {
        let rgb = [0u8, 128, 255];
        let image = RgbImage::new(1, 1, &rgb)?;
        let prepared = detector_input(&image, &Device::Cpu)?;
        assert_eq!(prepared.tensor.dims(), [1, DETECTOR_SIDE, DETECTOR_SIDE, CHANNELS]);
        let values = prepared.tensor.flatten_all()?.to_vec1::<f32>()?;
        let first = values.first().copied().ok_or_else(|| Error::Model("MediaPipe test tensor shape".into()))?;
        assert!((first + 1.0).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn landmark_input_uses_zero_to_one_metadata_range() -> Result<()> {
        let black_rgb = [0u8, 0, 0];
        let white_rgb = [255u8, 255, 255];
        let black = RgbImage::new(1, 1, &black_rgb)?;
        let white = RgbImage::new(1, 1, &white_rgb)?;
        let rect = NormalizedRect { center: [0.5, 0.5], size: [1.0, 1.0], rotation: 0.0 };
        let black_tensor = landmark_input(&black, &rect, &Device::Cpu)?.tensor.flatten_all()?.to_vec1::<f32>()?;
        let white_tensor = landmark_input(&white, &rect, &Device::Cpu)?.tensor.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(black_tensor.first().copied(), Some(0.0));
        assert_eq!(white_tensor.first().copied(), Some(1.0));
        Ok(())
    }

    #[test]
    fn wide_and_tall_detector_letterboxes_put_black_borders_around_content() -> Result<()> {
        let wide_rgb = vec![255u8; 4 * 2 * 3];
        let wide = RgbImage::new(4, 2, &wide_rgb)?;
        let wide_prepared = detector_input(&wide, &Device::Cpu)?;
        assert_eq!(wide_prepared.letterbox, [0.0, 0.25, 0.0, 0.25]);
        assert_eq!(wide_prepared.project_xy([0.5, 0.25])?, [0.5, 0.0]);
        let wide_values = wide_prepared.tensor.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(wide_values.first().copied(), Some(-1.0));
        let wide_center = 64 * DETECTOR_SIDE * CHANNELS;
        assert_eq!(wide_values.get(wide_center).copied(), Some(1.0));

        let tall_rgb = vec![255u8; 2 * 4 * 3];
        let tall = RgbImage::new(2, 4, &tall_rgb)?;
        let tall_prepared = detector_input(&tall, &Device::Cpu)?;
        assert_eq!(tall_prepared.letterbox, [0.25, 0.0, 0.25, 0.0]);
        assert_eq!(tall_prepared.project_xy([0.25, 0.5])?, [0.0, 0.5]);
        let tall_values = tall_prepared.tensor.flatten_all()?.to_vec1::<f32>()?;
        assert_eq!(tall_values.first().copied(), Some(-1.0));
        let tall_center = 64 * CHANNELS;
        assert_eq!(tall_values.get(tall_center).copied(), Some(1.0));
        Ok(())
    }

    #[test]
    fn face_rect_uses_eye_rotation_and_one_point_five_scale() -> Result<()> {
        let rgb = vec![0u8; 100 * 100 * 3];
        let image = RgbImage::new(100, 100, &rgb)?;
        let detection = crate::mediapipe_detector::Detection {
            score: 0.9,
            bounds: [0.2, 0.3, 0.4, 0.2],
            keypoints: [[0.3, 0.4], [0.5, 0.5], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0], [0.0, 0.0]],
        };
        let rect = face_rect(&image, &detection)?;
        assert!((rect.center[0] - 0.4).abs() < 1e-6);
        assert!((rect.center[1] - 0.4).abs() < 1e-6);
        assert!((rect.size[0] - 0.6).abs() < 1e-6);
        assert!((rect.size[1] - 0.3).abs() < 1e-6);
        assert!((rect.rotation - 0.5f32.atan()).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn projection_scales_z_by_source_roi_width() -> Result<()> {
        let rgb = vec![0u8; 200 * 100 * 3];
        let image = RgbImage::new(200, 100, &rgb)?;
        let rect = NormalizedRect { center: [0.5, 0.5], size: [0.5, 0.5], rotation: 0.0 };
        let prepared = landmark_input(&image, &rect, &Device::Cpu)?;
        assert_eq!(prepared.project_xyz([0.5, 0.5, 1.0])?, [0.5, 0.5, 0.5]);
        Ok(())
    }

    #[test]
    fn rotated_non_square_roi_uses_physical_pixel_aspect() -> Result<()> {
        let rgb = vec![0u8; 200 * 100 * 3];
        let image = RgbImage::new(200, 100, &rgb)?;
        let rect = NormalizedRect { center: [0.5, 0.5], size: [0.4, 0.2], rotation: std::f32::consts::FRAC_PI_2 };
        let prepared = landmark_input(&image, &rect, &Device::Cpu)?;
        let projected = prepared.project_xy([1.0, 0.5])?;
        assert!((projected[0] - 0.5).abs() < 1e-6);
        assert!((projected[1] - 0.9).abs() < 1e-6);
        let projected_xyz = prepared.project_xyz([1.0, 0.5, 1.0])?;
        assert!((projected_xyz[2] - 0.8).abs() < 1e-6);
        Ok(())
    }

    #[test]
    fn project_landmarks_requires_478_triples() -> Result<()> {
        let rgb = [0u8, 0, 0];
        let image = RgbImage::new(1, 1, &rgb)?;
        let prepared = landmark_input(&image, &NormalizedRect { center: [0.5, 0.5], size: [1.0, 1.0], rotation: 0.0 }, &Device::Cpu)?;
        assert!(project_landmarks(&[0.0; 3], &prepared).is_err());
        Ok(())
    }

    #[test]
    fn bilinear_sampling_is_fractional_and_border_specific() -> Result<()> {
        let rgb = [100u8, 100, 100, 0, 0, 0];
        let image = RgbImage::new(2, 1, &rgb)?;
        let fractional = sample_rgb(&image, 0.25, 0.0, BorderMode::Replicate)?;
        assert!((fractional[0] - 50.0).abs() < 1e-6);
        let replicated = sample_rgb(&image, -0.25, 0.0, BorderMode::Replicate)?;
        assert!((replicated[0] - 100.0).abs() < 1e-6);
        let zero = sample_rgb(&image, -0.25, 0.0, BorderMode::Zero)?;
        assert!((zero[0] - 50.0).abs() < 1e-6);
        Ok(())
    }
}
