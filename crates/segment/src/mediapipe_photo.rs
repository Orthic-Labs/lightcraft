//! Experimental still-image composition of verified MediaPipe v1 graphs.
//!
//! Policy follows Google MediaPipe at revision
//! `458200ffced12a9db596ddf1cce3380ea05f71fb` (Apache-2.0), translated & modified
//! by Ember contributors (2026). See `NOTICE` & `docs/models/mediapipe-image-contract.md`.
//! This path has no catalog access & supplies no culling or calibrated eye decision.
//! Reference agreement, model quality & native performance remain unqualified.

#![cfg(not(target_arch = "wasm32"))]

use candle_core::{DType, Device, Tensor};

use crate::mediapipe_artifact::Bundle;
use crate::mediapipe_blendshapes::{Coefficient, EyeEvidence};
use crate::mediapipe_detector::Detection;
use crate::mediapipe_input::{NormalizedRect, RgbImage};
use crate::{Error, Result};

/// Explicit experimental thresholds, to be frozen before independent evaluation.
#[derive(Clone, Copy, Debug)]
pub struct ImageOptions {
    pub detection_threshold: f32,
    pub nms_threshold: f32,
    pub presence_threshold: f32,
    pub max_faces: usize,
}

impl ImageOptions {
    /// Validate before allocating image tensors or executing any graph.
    pub fn validate(&self) -> Result<()> {
        for (name, value) in [("detection", self.detection_threshold), ("NMS", self.nms_threshold), ("presence", self.presence_threshold)] {
            if !value.is_finite() || !(0.0..=1.0).contains(&value) {
                return Err(Error::Model(format!("MediaPipe {name} threshold must be finite & within [0,1]")));
            }
        }
        if !(1..=32).contains(&self.max_faces) {
            return Err(Error::Model("MediaPipe max_faces must be within 1..=32".into()));
        }
        Ok(())
    }
}

/// One detector proposal. Missing landmarks mean presence did not pass its gate;
/// they never mean closed eyes. Coefficients remain raw expression evidence.
#[derive(Clone, Debug)]
pub struct FaceObservation {
    /// Full-image normalized box & six keypoints, without image-boundary clipping.
    pub detection: Detection,
    pub crop: NormalizedRect,
    /// Source presence-logit sigmoid; not calibrated on Ember photographs.
    pub presence_score: f32,
    pub presence_passed: bool,
    /// Third mesh output, named `tongue_out` in metadata; retained raw & uncalibrated.
    pub raw_auxiliary_2: f32,
    /// 478 xyz triples normalized in original-image coordinates.
    pub landmarks: Option<Vec<[f32; 3]>>,
    pub coefficients: Option<Vec<Coefficient>>,
    pub eye_evidence: Option<EyeEvidence>,
}

/// Bounded read-only model observations in deterministic detector order.
#[derive(Clone, Debug)]
pub struct ImageAnalysis {
    pub width: usize,
    pub height: usize,
    pub faces: Vec<FaceObservation>,
}

impl Bundle {
    /// Run detector → rotated face crops → projected landmarks → blendshapes.
    ///
    /// Callers supply upright RGB8 pixels & same device used to load this bundle.
    /// No EXIF orientation, file decoding, tracking, smoothing, catalog mutation,
    /// automatic accept/reject decision or probability calibration is performed.
    pub fn analyze_image(&self, image: &RgbImage<'_>, options: &ImageOptions, device: &Device) -> Result<ImageAnalysis> {
        options.validate()?;
        let detector_image = crate::mediapipe_input::detector_input(image, device)?;
        let raw_detector = self.forward_detector(&detector_image.tensor)?;
        let regressors = raw_detector.regressors.flatten_all()?.to_vec1::<f32>()?;
        let logits = raw_detector.classifier_logits.flatten_all()?.to_vec1::<f32>()?;
        let detections =
            crate::mediapipe_detector::decode(&regressors, &logits, options.detection_threshold, options.nms_threshold, options.max_faces)?;
        let mut faces = Vec::with_capacity(detections.len());
        for detection in detections {
            let detection = detector_image.project_detection(&detection)?;
            validate_crop_detection(&detection)?;
            let crop = crate::mediapipe_input::face_rect(image, &detection)?;
            let landmark_image = crate::mediapipe_input::landmark_input(image, &crop, device)?;
            let raw_landmarks = self.forward_landmarks(&landmark_image.tensor)?;
            let presence_logit = scalar(&raw_landmarks.auxiliary_1)?;
            let (presence_score, presence_passed) = presence(presence_logit, options.presence_threshold)?;
            let raw_auxiliary_2 = scalar(&raw_landmarks.auxiliary_2)?;
            let mut face = FaceObservation {
                detection,
                crop,
                presence_score,
                presence_passed,
                raw_auxiliary_2,
                landmarks: None,
                coefficients: None,
                eye_evidence: None,
            };
            if presence_passed {
                let coordinates = raw_landmarks.coordinates.flatten_all()?.to_vec1::<f32>()?;
                let landmarks = crate::mediapipe_input::project_landmarks(&coordinates, &landmark_image)?;
                let blendshape_input = crate::mediapipe_blendshapes::input(&landmarks, image.width(), image.height(), device)?;
                let coefficients = self.forward_blendshapes(&blendshape_input)?.flatten_all()?.to_vec1::<f32>()?;
                face.coefficients = Some(crate::mediapipe_blendshapes::named_coefficients(&coefficients)?);
                face.eye_evidence = Some(crate::mediapipe_blendshapes::eye_evidence(&coefficients)?);
                face.landmarks = Some(landmarks);
            }
            faces.push(face);
        }
        Ok(ImageAnalysis { width: image.width(), height: image.height(), faces })
    }
}

fn validate_crop_detection(detection: &Detection) -> Result<()> {
    let width = detection.bounds.get(2).copied().ok_or_else(|| Error::Model("MediaPipe face width is missing".into()))?;
    let height = detection.bounds.get(3).copied().ok_or_else(|| Error::Model("MediaPipe face height is missing".into()))?;
    if !width.is_finite() || !height.is_finite() || width <= 0.0 || height <= 0.0 {
        // A diagnostic error preserves unknown evidence. Never silently discard a
        // high-scoring malformed face or publish a partial successful analysis.
        return Err(Error::Model("MediaPipe detected degenerate face geometry; image analysis aborted before crop inference".into()));
    }
    Ok(())
}

fn scalar(tensor: &Tensor) -> Result<f32> {
    if tensor.dtype() != DType::F32 || tensor.elem_count() != 1 {
        return Err(Error::Model("MediaPipe presence/auxiliary output must contain one F32 value".into()));
    }
    let value = tensor.flatten_all()?.to_vec1::<f32>()?.first().copied().ok_or_else(|| Error::Model("MediaPipe scalar output is missing".into()))?;
    if !value.is_finite() {
        return Err(Error::Model("MediaPipe scalar output is non-finite".into()));
    }
    Ok(value)
}

fn presence(logit: f32, threshold: f32) -> Result<(f32, bool)> {
    if !logit.is_finite() || !threshold.is_finite() || !(0.0..=1.0).contains(&threshold) {
        return Err(Error::Model("MediaPipe presence logit/threshold is invalid".into()));
    }
    let exp = (-logit.abs()).exp();
    let score = if logit >= 0.0 { 1.0 / (1.0 + exp) } else { exp / (1.0 + exp) };
    // MediaPipe ThresholdingCalculator uses strict >, including at equality.
    Ok((score, score > threshold))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn degenerate_faces_fail_before_crop_inference() {
        let mut face = Detection { score: 0.9, bounds: [0.1, 0.2, 0.3, 0.4], keypoints: [[0.0; 2]; 6] };
        assert!(validate_crop_detection(&face).is_ok());
        for width in [0.0, -1.0, f32::INFINITY, f32::NAN] {
            face.bounds[2] = width;
            assert!(validate_crop_detection(&face).is_err());
        }
        face.bounds[2] = 0.3;
        face.bounds[3] = 0.0;
        assert!(validate_crop_detection(&face).is_err());
    }

    #[test]
    fn presence_gate_is_strict_and_extreme_logits_stay_finite() {
        assert_eq!(presence(0.0, 0.5).unwrap(), (0.5, false));
        assert_eq!(presence(f32::MAX, 1.0).unwrap(), (1.0, false));
        assert_eq!(presence(-f32::MAX, 0.0).unwrap(), (0.0, false));
        assert!(presence(f32::NAN, 0.5).is_err());
        assert!(presence(0.0, f32::INFINITY).is_err());
    }

    #[test]
    fn explicit_options_reject_nan_thresholds_and_unbounded_faces() {
        let mut options = ImageOptions { detection_threshold: 0.5, nms_threshold: 0.3, presence_threshold: 0.5, max_faces: 1 };
        assert!(options.validate().is_ok());
        options.presence_threshold = f32::NAN;
        assert!(options.validate().is_err());
        options.presence_threshold = 0.5;
        for count in [0, 33, usize::MAX] {
            options.max_faces = count;
            assert!(options.validate().is_err());
        }
    }

    #[test]
    fn scalar_contract_rejects_wrong_count_type_and_nonfinite_value() {
        let good = Tensor::from_vec(vec![0.0f32], &[1, 1], &Device::Cpu).unwrap();
        assert_eq!(scalar(&good).unwrap(), 0.0);
        let two = Tensor::from_vec(vec![0.0f32, 0.0], &[2], &Device::Cpu).unwrap();
        assert!(scalar(&two).is_err());
        let integer = Tensor::from_vec(vec![1i64], &[1], &Device::Cpu).unwrap();
        assert!(scalar(&integer).is_err());
        let invalid = Tensor::from_vec(vec![f32::INFINITY], &[1], &Device::Cpu).unwrap();
        assert!(scalar(&invalid).is_err());
    }
}
