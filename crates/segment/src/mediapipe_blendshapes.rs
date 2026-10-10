//! MediaPipe Face Landmarker blendshape input mapping & raw coefficient labels.
//!
//! Source attribution (Apache-2.0): `LANDMARK_INDICES` & `COEFFICIENT_NAMES` are
//! copied from MediaPipe commit `458200ffced12a9db596ddf1cce3380ea05f71fb`,
//! `mediapipe/tasks/cc/vision/face_landmarker/face_blendshapes_graph.cc`,
//! lines 51-115. Pixel scaling follows that commit's
//! `mediapipe/calculators/tensor/landmarks_to_tensor_calculator.cc`, which
//! multiplies normalized X by image width & normalized Y by image height.
//!
//! Ember modification: this module exposes checked fixed-array input conversion,
//! Candle tensor construction, raw coefficient labels & raw eye evidence. It does
//! not calibrate coefficients or make closed/open, probability, confidence or
//! culling decisions.

#![cfg(not(target_arch = "wasm32"))]
#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use candle_core::{Device, Tensor};

use crate::{Error, Result};

/// Exact MediaPipe HUND blendshape subset, in model input order.
pub const LANDMARK_INDICES: [usize; 146] = [
    0, 1, 4, 5, 6, 7, 8, 10, 13, 14, 17, 21, 33, 37, 39, 40, 46, 52, 53, 54, 55, 58, 61, 63, 65, 66, 67, 70, 78, 80, 81, 82, 84, 87, 88, 91, 93, 95,
    103, 105, 107, 109, 127, 132, 133, 136, 144, 145, 146, 148, 149, 150, 152, 153, 154, 155, 157, 158, 159, 160, 161, 162, 163, 168, 172, 173, 176,
    178, 181, 185, 191, 195, 197, 234, 246, 249, 251, 263, 267, 269, 270, 276, 282, 283, 284, 285, 288, 291, 293, 295, 296, 297, 300, 308, 310, 311,
    312, 314, 317, 318, 321, 323, 324, 332, 334, 336, 338, 356, 361, 362, 365, 373, 374, 375, 377, 378, 379, 380, 381, 382, 384, 385, 386, 387, 388,
    389, 390, 397, 398, 400, 402, 405, 409, 415, 454, 466, 468, 469, 470, 471, 472, 473, 474, 475, 476, 477,
];

/// Exact MediaPipe Face Landmarker blendshape output labels, in model order.
pub const COEFFICIENT_NAMES: [&str; 52] = [
    "_neutral",
    "browDownLeft",
    "browDownRight",
    "browInnerUp",
    "browOuterUpLeft",
    "browOuterUpRight",
    "cheekPuff",
    "cheekSquintLeft",
    "cheekSquintRight",
    "eyeBlinkLeft",
    "eyeBlinkRight",
    "eyeLookDownLeft",
    "eyeLookDownRight",
    "eyeLookInLeft",
    "eyeLookInRight",
    "eyeLookOutLeft",
    "eyeLookOutRight",
    "eyeLookUpLeft",
    "eyeLookUpRight",
    "eyeSquintLeft",
    "eyeSquintRight",
    "eyeWideLeft",
    "eyeWideRight",
    "jawForward",
    "jawLeft",
    "jawOpen",
    "jawRight",
    "mouthClose",
    "mouthDimpleLeft",
    "mouthDimpleRight",
    "mouthFrownLeft",
    "mouthFrownRight",
    "mouthFunnel",
    "mouthLeft",
    "mouthLowerDownLeft",
    "mouthLowerDownRight",
    "mouthPressLeft",
    "mouthPressRight",
    "mouthPucker",
    "mouthRight",
    "mouthRollLower",
    "mouthRollUpper",
    "mouthShrugLower",
    "mouthShrugUpper",
    "mouthSmileLeft",
    "mouthSmileRight",
    "mouthStretchLeft",
    "mouthStretchRight",
    "mouthUpperUpLeft",
    "mouthUpperUpRight",
    "noseSneerLeft",
    "noseSneerRight",
];

const MAX_IMAGE_DIMENSION: usize = 16_384;
// Permit modestly out-of-frame normalized landmarks while bounding hostile pixel values.
const MAX_PIXEL_COORDINATE: f32 = 262_144.0;

/// One raw MediaPipe coefficient, retaining source label & output value.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Coefficient {
    pub name: &'static str,
    pub value: f32,
}

/// Raw eye-related blendshape evidence. Values are model outputs, without calibration.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct EyeEvidence {
    pub blink_left: f32,
    pub blink_right: f32,
    pub squint_left: f32,
    pub squint_right: f32,
    pub wide_left: f32,
    pub wide_right: f32,
}

/// Build MediaPipe blendshape input `[1, 146, 2]` from 478 normalized XYZ landmarks.
///
/// Only X & Y are emitted, matching MediaPipe's graph. Z is validated as part of each
/// source landmark but is intentionally omitted from this model input.
pub fn input(landmarks: &[[f32; 3]], width: usize, height: usize, device: &Device) -> Result<Tensor> {
    if landmarks.len() != 478 {
        return Err(Error::Model("MediaPipe blendshape input requires exactly 478 landmarks".into()));
    }
    validate_dimension(width, "width")?;
    validate_dimension(height, "height")?;

    if landmarks.iter().any(|landmark| landmark.iter().any(|value| !value.is_finite())) {
        return Err(Error::Model("MediaPipe landmarks must contain only finite values".into()));
    }

    let mut data = Vec::with_capacity(LANDMARK_INDICES.len() * 2);
    for index in LANDMARK_INDICES {
        let landmark = landmarks.get(index).ok_or_else(|| Error::Model("MediaPipe landmark index is out of range".into()))?;
        let x = checked_pixel_coordinate(landmark[0], width, "x")?;
        let y = checked_pixel_coordinate(landmark[1], height, "y")?;
        data.extend([x, y]);
    }
    Tensor::from_vec(data, (1, LANDMARK_INDICES.len(), 2), device).map_err(Error::from)
}

/// Attach exact MediaPipe labels to raw `[52]` model output values.
pub fn named_coefficients(values: &[f32]) -> Result<Vec<Coefficient>> {
    validate_coefficients(values)?;
    Ok(COEFFICIENT_NAMES.iter().copied().zip(values.iter().copied()).map(|(name, value)| Coefficient { name, value }).collect())
}

/// Extract raw eye-related coefficients for downstream qualification.
pub fn eye_evidence(values: &[f32]) -> Result<EyeEvidence> {
    validate_coefficients(values)?;
    Ok(EyeEvidence {
        blink_left: coefficient_at(values, 9, "eyeBlinkLeft")?,
        blink_right: coefficient_at(values, 10, "eyeBlinkRight")?,
        squint_left: coefficient_at(values, 19, "eyeSquintLeft")?,
        squint_right: coefficient_at(values, 20, "eyeSquintRight")?,
        wide_left: coefficient_at(values, 21, "eyeWideLeft")?,
        wide_right: coefficient_at(values, 22, "eyeWideRight")?,
    })
}

fn validate_dimension(value: usize, name: &str) -> Result<()> {
    if !(1..=MAX_IMAGE_DIMENSION).contains(&value) {
        return Err(Error::Model(format!("MediaPipe image {name} must be in 1..={MAX_IMAGE_DIMENSION}")));
    }
    Ok(())
}

fn checked_pixel_coordinate(value: f32, dimension: usize, axis: &str) -> Result<f32> {
    let pixel = value * dimension as f32;
    if !pixel.is_finite() || pixel.abs() > MAX_PIXEL_COORDINATE {
        return Err(Error::Model(format!("MediaPipe {axis} pixel coordinate is nonfinite or out of bounds")));
    }
    Ok(pixel)
}

fn validate_coefficients(values: &[f32]) -> Result<()> {
    if values.len() != COEFFICIENT_NAMES.len() {
        return Err(Error::Model("MediaPipe blendshape output requires exactly 52 coefficients".into()));
    }
    if values.iter().any(|value| !value.is_finite()) {
        return Err(Error::Model("MediaPipe blendshape coefficients must be finite".into()));
    }
    Ok(())
}

fn coefficient_at(values: &[f32], index: usize, name: &str) -> Result<f32> {
    values.get(index).copied().ok_or_else(|| Error::Model(format!("MediaPipe coefficient {name} is missing")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_preserves_source_order() -> Result<()> {
        let mut landmarks = [[0.0; 3]; 478];
        for (index, landmark) in landmarks.iter_mut().enumerate() {
            landmark[0] = index as f32 / 1_000.0;
            landmark[1] = (1_000 - index) as f32 / 1_000.0;
            landmark[2] = 3.0;
        }
        let tensor = input(&landmarks, 1_000, 1_000, &Device::Cpu)?;
        let values = tensor.to_vec3::<f32>()?;
        assert!((values[0][0][0] - 0.0).abs() < 1e-4);
        assert!((values[0][0][1] - 1_000.0).abs() < 1e-4);
        assert!((values[0][1][0] - 1.0).abs() < 1e-4);
        assert!((values[0][1][1] - 999.0).abs() < 1e-4);
        let last = values[0].last().ok_or_else(|| Error::Model("test tensor has no rows".into()))?;
        assert!((last[0] - 477.0).abs() < 1e-4);
        assert!((last[1] - 523.0).abs() < 1e-4);
        Ok(())
    }

    #[test]
    fn input_scales_x_and_y_by_separate_image_dimensions() -> Result<()> {
        let mut landmarks = [[0.0; 3]; 478];
        landmarks[0] = [0.25, 0.75, 0.0];
        let values = input(&landmarks, 100, 200, &Device::Cpu)?.to_vec3::<f32>()?;
        assert!((values[0][0][0] - 25.0).abs() < 1e-4);
        assert!((values[0][0][1] - 150.0).abs() < 1e-4);
        Ok(())
    }

    #[test]
    fn input_rejects_wrong_length_nonfinite_dimensions_and_pixels() {
        let landmarks = [[0.0; 3]; 478];
        assert!(input(&landmarks[..477], 1, 1, &Device::Cpu).is_err());
        assert!(input(&landmarks, 0, 1, &Device::Cpu).is_err());
        assert!(input(&landmarks, 1, MAX_IMAGE_DIMENSION + 1, &Device::Cpu).is_err());

        let mut nonfinite = landmarks;
        nonfinite[0][2] = f32::NAN;
        assert!(input(&nonfinite, 1, 1, &Device::Cpu).is_err());

        let mut unbounded = landmarks;
        unbounded[0][0] = 1_000.0;
        assert!(input(&unbounded, MAX_IMAGE_DIMENSION, 1, &Device::Cpu).is_err());

        assert!(named_coefficients(&[0.0; 51]).is_err());
        let mut nonfinite_coefficients = [0.0; 52];
        nonfinite_coefficients[9] = f32::INFINITY;
        assert!(named_coefficients(&nonfinite_coefficients).is_err());
        assert!(eye_evidence(&[0.0; 51]).is_err());
    }

    #[test]
    fn raw_coefficients_keep_names_values_and_eye_indices() -> Result<()> {
        let values = (0..52).map(|index| index as f32 + 0.25).collect::<Vec<_>>();
        let named = named_coefficients(&values)?;
        assert_eq!(named[9], Coefficient { name: "eyeBlinkLeft", value: 9.25 });
        assert_eq!(named[22], Coefficient { name: "eyeWideRight", value: 22.25 });
        assert_eq!(
            eye_evidence(&values)?,
            EyeEvidence { blink_left: 9.25, blink_right: 10.25, squint_left: 19.25, squint_right: 20.25, wide_left: 21.25, wide_right: 22.25 }
        );
        Ok(())
    }
}
