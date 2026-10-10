//! MediaPipe Face Detector v1 post-processing for the fixed 128px graph.
//!
//! Modified work (Apache License 2.0, section 4(b)): translated to bounded, pure Rust by
//! Ember contributors in 2026 from MediaPipe source at commit
//! `458200ffced12a9db596ddf1cce3380ea05f71fb`.  Anchor order follows
//! `mediapipe/calculators/tflite/ssd_anchors_calculator.cc`; box decoding follows
//! `mediapipe/calculators/tensor/tensors_to_detections_calculator.cc`; weighted NMS follows
//! `mediapipe/calculators/util/non_max_suppression_calculator.cc`; graph settings follow
//! `mediapipe/tasks/cc/vision/face_detector/face_detector_graph.cc`.  The pinned graph sets
//! `reverse_output_order`, which selects XYWH (source `GetBoxFormat`, lines 132–139).

use std::cmp::Ordering;

const NUM_BOXES: usize = 896;
const NUM_COORDS: usize = 16;
const NUM_KEYPOINTS: usize = 6;
const INPUT_SIDE: f32 = 128.0;
const SCORE_CLIP: f32 = 100.0;
const MIN_SCALE: f32 = 0.1484375;
const MAX_SCALE: f32 = 0.75;
const NUM_SCALE_STEPS: f32 = 3.0;

/// One decoded face in normalized 128×128 tensor coordinates.
#[derive(Clone, Debug, PartialEq)]
pub struct Detection {
    pub score: f32,
    /// `[xmin, ymin, width, height]`; coordinates remain unconstrained.
    pub bounds: [f32; 4],
    /// Six `[x, y]` points in detector tensor coordinates.
    pub keypoints: [[f32; 2]; 6],
}

#[derive(Clone, Copy)]
struct Anchor {
    x_center: f32,
    y_center: f32,
}

#[derive(Clone)]
struct Candidate {
    index: usize,
    detection: Detection,
}

/// Decode fixed Face Detector v1 heads, filter scores, then apply source weighted IoU NMS.
///
/// Inputs must be the exact graph outputs `[1, 896, 16]` and `[1, 896, 1]` flattened in
/// row-major order. Scores use source sigmoid clipping at ±100. Invalid thresholds, counts,
/// dimensions, non-finite logits, negative decoded sizes and non-finite decoded values return an
/// error or skip the affected candidate without panicking.
pub fn decode(regressors: &[f32], logits: &[f32], score_threshold: f32, nms_threshold: f32, max_faces: usize) -> crate::Result<Vec<Detection>> {
    if regressors.len() != NUM_BOXES * NUM_COORDS {
        return Err(error("regressors must contain exactly 896 * 16 values"));
    }
    if logits.len() != NUM_BOXES {
        return Err(error("logits must contain exactly 896 values"));
    }
    if !score_threshold.is_finite() || !(0.0..=1.0).contains(&score_threshold) {
        return Err(error("score threshold must be finite and in [0, 1]"));
    }
    if !nms_threshold.is_finite() || !(0.0..=1.0).contains(&nms_threshold) {
        return Err(error("NMS threshold must be finite and in [0, 1]"));
    }
    if !(1..=32).contains(&max_faces) {
        return Err(error("max_faces must be in 1..=32"));
    }

    let anchors = anchors()?;
    let mut candidates = Vec::with_capacity(NUM_BOXES);
    for index in 0..NUM_BOXES {
        let raw_score = *logits.get(index).ok_or_else(|| error("logit index overflow"))?;
        // Safety boundary: unlike source's unconstrained float path, reject ±∞ before ±100
        // clipping so malformed graph output cannot become an accepted face score.
        if !raw_score.is_finite() {
            continue;
        }
        let score = sigmoid_clipped(raw_score);
        if !score.is_finite() || score < score_threshold {
            continue;
        }
        let Some(anchor) = anchors.get(index).copied() else {
            return Err(error("anchor index overflow"));
        };
        let Some(detection) = decode_one(regressors, index, anchor, score) else {
            continue;
        };
        candidates.push(Candidate { index, detection });
    }
    Ok(weighted_nms(candidates, nms_threshold, max_faces))
}

fn error(message: &str) -> crate::Error {
    crate::Error::Model(format!("MediaPipe detector: {message}"))
}

fn sigmoid_clipped(logit: f32) -> f32 {
    let clipped = logit.clamp(-SCORE_CLIP, SCORE_CLIP);
    1.0 / (1.0 + (-clipped).exp())
}

fn decode_one(regressors: &[f32], index: usize, anchor: Anchor, score: f32) -> Option<Detection> {
    let base = index.checked_mul(NUM_COORDS)?;
    // reverse_output_order=true selects MediaPipe's XYWH path.
    let raw_x = regressors.get(base).copied()?;
    let raw_y = regressors.get(base.checked_add(1)?).copied()?;
    let raw_w = regressors.get(base.checked_add(2)?).copied()?;
    let raw_h = regressors.get(base.checked_add(3)?).copied()?;
    let x_center = raw_x / INPUT_SIDE + anchor.x_center;
    let y_center = raw_y / INPUT_SIDE + anchor.y_center;
    let width = raw_w / INPUT_SIDE;
    let height = raw_h / INPUT_SIDE;
    if !x_center.is_finite() || !y_center.is_finite() || !width.is_finite() || !height.is_finite() || width < 0.0 || height < 0.0 {
        return None;
    }
    let xmin = x_center - width / 2.0;
    let ymin = y_center - height / 2.0;
    let xmax = x_center + width / 2.0;
    let ymax = y_center + height / 2.0;
    let decoded_width = xmax - xmin;
    let decoded_height = ymax - ymin;
    if !xmin.is_finite()
        || !ymin.is_finite()
        || !xmax.is_finite()
        || !ymax.is_finite()
        || !decoded_width.is_finite()
        || !decoded_height.is_finite()
        || decoded_width < 0.0
        || decoded_height < 0.0
    {
        return None;
    }

    let mut keypoints = [[0.0; 2]; NUM_KEYPOINTS];
    for (keypoint_index, keypoint) in keypoints.iter_mut().enumerate() {
        let offset = base.checked_add(4)?.checked_add(keypoint_index.checked_mul(2)?)?;
        let raw_x = regressors.get(offset).copied()?;
        let raw_y = regressors.get(offset.checked_add(1)?).copied()?;
        let x = raw_x / INPUT_SIDE + anchor.x_center;
        let y = raw_y / INPUT_SIDE + anchor.y_center;
        if !x.is_finite() || !y.is_finite() {
            return None;
        }
        *keypoint = [x, y];
    }
    Some(Detection { score, bounds: [xmin, ymin, decoded_width, decoded_height], keypoints })
}

fn scale(index: usize) -> f32 {
    MIN_SCALE + (MAX_SCALE - MIN_SCALE) * index as f32 / NUM_SCALE_STEPS
}

fn append_layer(anchors: &mut Vec<Anchor>, side: usize, scales: &[f32]) {
    for y in 0..side {
        for x in 0..side {
            let x_center = (x as f32 + 0.5) / side as f32;
            let y_center = (y as f32 + 0.5) / side as f32;
            for _ in scales {
                anchors.push(Anchor { x_center, y_center });
            }
        }
    }
}

fn anchors() -> crate::Result<Vec<Anchor>> {
    let mut result = Vec::with_capacity(NUM_BOXES);
    // Legacy short-range graph: strides [8, 16, 16, 16], fixed unit anchors,
    // one interpolated anchor per layer as emitted by SsdAnchorsCalculator.
    let stride8_scales = [scale(0), (scale(0) * scale(1)).sqrt()];
    append_layer(&mut result, 16, &stride8_scales);
    let mut stride16_scales = [0.0_f32; 6];
    let mut output = 0usize;
    for layer in 1..=3 {
        let current = scale(layer);
        let next = if layer == 3 { 1.0 } else { scale(layer + 1) };
        if let Some(slot) = stride16_scales.get_mut(output) {
            *slot = current;
        } else {
            return Err(error("stride-16 anchor scale overflow"));
        }
        output = output.checked_add(1).ok_or_else(|| error("anchor scale overflow"))?;
        if let Some(slot) = stride16_scales.get_mut(output) {
            *slot = (current * next).sqrt();
        } else {
            return Err(error("stride-16 anchor scale overflow"));
        }
        output = output.checked_add(1).ok_or_else(|| error("anchor scale overflow"))?;
    }
    append_layer(&mut result, 8, &stride16_scales);
    if result.len() != NUM_BOXES {
        return Err(error("generated anchor count does not match detector graph"));
    }
    Ok(result)
}

fn weighted_nms(mut candidates: Vec<Candidate>, threshold: f32, max_faces: usize) -> Vec<Detection> {
    // MediaPipe uses descending score ordering. Explicit index tie-breaking supplies stable
    // initial-anchor order where C++ std::sort leaves equal scores unspecified.
    candidates.sort_by(|left, right| {
        right.detection.score.partial_cmp(&left.detection.score).unwrap_or(Ordering::Equal).then_with(|| left.index.cmp(&right.index))
    });

    let mut output = Vec::with_capacity(max_faces.min(candidates.len()));
    let mut remaining = candidates;
    while !remaining.is_empty() && output.len() < max_faces {
        let original_len = remaining.len();
        let Some(leader) = remaining.first().cloned() else {
            break;
        };
        let leader_bounds = leader.detection.bounds;
        let mut cluster = Vec::new();
        let mut rest = Vec::new();
        for candidate in remaining {
            if iou(candidate.detection.bounds, leader_bounds) > threshold {
                cluster.push(candidate);
            } else {
                rest.push(candidate);
            }
        }

        let weighted = if cluster.is_empty() { leader.detection } else { weighted_detection(&cluster).unwrap_or(leader.detection) };
        output.push(weighted);

        // This preserves source behavior: a weighted pass with no overlapping candidate emits
        // its leader once and terminates instead of repeatedly selecting the same leader.
        if rest.len() == original_len {
            break;
        }
        remaining = rest;
    }
    output
}

fn weighted_detection(cluster: &[Candidate]) -> Option<Detection> {
    let leader = cluster.first()?.detection.clone();
    let total_score = cluster.iter().try_fold(0.0_f64, |sum, candidate| {
        let score = f64::from(candidate.detection.score);
        let next = sum + score;
        next.is_finite().then_some(next)
    })?;
    if total_score <= 0.0 {
        return Some(leader);
    }
    let mut xmin = 0.0_f64;
    let mut ymin = 0.0_f64;
    let mut xmax = 0.0_f64;
    let mut ymax = 0.0_f64;
    let mut keypoints = [[0.0_f64; 2]; NUM_KEYPOINTS];
    for candidate in cluster {
        let weight = f64::from(candidate.detection.score);
        let bounds = candidate.detection.bounds;
        xmin += f64::from(bounds[0]) * weight;
        ymin += f64::from(bounds[1]) * weight;
        xmax += (f64::from(bounds[0]) + f64::from(bounds[2])) * weight;
        ymax += (f64::from(bounds[1]) + f64::from(bounds[3])) * weight;
        for (index, point) in candidate.detection.keypoints.iter().enumerate() {
            keypoints[index][0] += f64::from(point[0]) * weight;
            keypoints[index][1] += f64::from(point[1]) * weight;
        }
    }
    let weighted_bounds =
        [(xmin / total_score) as f32, (ymin / total_score) as f32, ((xmax - xmin) / total_score) as f32, ((ymax - ymin) / total_score) as f32];
    if weighted_bounds.iter().any(|value| !value.is_finite()) {
        return Some(leader);
    }
    let mut output_points = [[0.0_f32; 2]; NUM_KEYPOINTS];
    for (index, point) in output_points.iter_mut().enumerate() {
        point[0] = (keypoints[index][0] / total_score) as f32;
        point[1] = (keypoints[index][1] / total_score) as f32;
        if !point[0].is_finite() || !point[1].is_finite() {
            return Some(leader);
        }
    }
    Some(Detection { score: leader.score, bounds: weighted_bounds, keypoints: output_points })
}

fn iou(left: [f32; 4], right: [f32; 4]) -> f32 {
    let left_xmax = left[0] + left[2];
    let left_ymax = left[1] + left[3];
    let right_xmax = right[0] + right[2];
    let right_ymax = right[1] + right[3];
    if !left_xmax.is_finite() || !left_ymax.is_finite() || !right_xmax.is_finite() || !right_ymax.is_finite() {
        return 0.0;
    }
    let intersection_width = left_xmax.min(right_xmax) - left[0].max(right[0]);
    let intersection_height = left_ymax.min(right_ymax) - left[1].max(right[1]);
    if !(intersection_width > 0.0 && intersection_height > 0.0) {
        return 0.0;
    }
    let intersection = intersection_width * intersection_height;
    let left_area = left[2] * left[3];
    let right_area = right[2] * right[3];
    let union = left_area + right_area - intersection;
    if !intersection.is_finite() || !union.is_finite() || union <= 0.0 { 0.0 } else { intersection / union }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn empty_heads() -> (Vec<f32>, Vec<f32>) {
        (vec![0.0; NUM_BOXES * NUM_COORDS], vec![-100.0; NUM_BOXES])
    }

    #[test]
    fn rejects_wrong_shapes_thresholds_and_face_count() {
        let (regressors, logits) = empty_heads();
        assert!(decode(&regressors[..regressors.len() - 1], &logits, 0.5, 0.5, 1).is_err());
        assert!(decode(&regressors, &logits[..logits.len() - 1], 0.5, 0.5, 1).is_err());
        assert!(decode(&regressors, &logits, f32::NAN, 0.5, 1).is_err());
        assert!(decode(&regressors, &logits, 0.5, 1.1, 1).is_err());
        assert!(decode(&regressors, &logits, 0.5, 0.5, 0).is_err());
        assert!(decode(&regressors, &logits, 0.5, 0.5, 33).is_err());
    }

    #[test]
    fn decodes_anchor_order_and_skips_negative_or_nonfinite_boxes() {
        let (mut regressors, mut logits) = empty_heads();
        logits[0] = 100.0;
        // Anchor 0 is first stride-8 cell, whose center is (0.03125, 0.03125).
        // XYWH ordering must remain visible with distinct nonzero values.
        regressors[0] = 16.0;
        regressors[1] = 32.0;
        regressors[2] = 40.0;
        regressors[3] = 24.0;
        for keypoint_index in 0..NUM_KEYPOINTS {
            let offset = 4 + keypoint_index * 2;
            regressors[offset] = 8.0 + keypoint_index as f32 * 3.0;
            regressors[offset + 1] = 16.0 + keypoint_index as f32 * 5.0;
        }
        let detection = decode(&regressors, &logits, 0.99, 0.5, 1).unwrap().remove(0);
        assert!(detection.bounds[0].abs() < 1e-6);
        assert!((detection.bounds[1] - 0.1875).abs() < 1e-6);
        assert!((detection.bounds[2] - 0.3125).abs() < 1e-6);
        assert!((detection.bounds[3] - 0.1875).abs() < 1e-6);
        assert!((detection.keypoints[0][0] - 0.09375).abs() < 1e-6);
        assert!((detection.keypoints[0][1] - 0.15625).abs() < 1e-6);
        assert!((detection.keypoints[5][0] - 0.2109375).abs() < 1e-6);
        assert!((detection.keypoints[5][1] - 0.3515625).abs() < 1e-6);
        // Negative width is dropped; NaN keypoint is also dropped.
        regressors[2] = -1.0;
        assert!(decode(&regressors, &logits, 0.99, 0.5, 1).unwrap().is_empty());
        regressors[2] = 0.0;
        regressors[4] = f32::NAN;
        assert!(decode(&regressors, &logits, 0.99, 0.5, 1).unwrap().is_empty());
        regressors[4] = 0.0;
        logits[0] = f32::INFINITY;
        assert!(decode(&regressors, &logits, 0.99, 0.5, 1).unwrap().is_empty());
    }

    #[test]
    fn weighted_nms_preserves_leader_score_and_weighted_geometry() {
        let points = [[0.0; 2]; NUM_KEYPOINTS];
        let first = Candidate { index: 0, detection: Detection { score: 0.9, bounds: [0.0, 0.0, 1.0, 1.0], keypoints: points } };
        let second = Candidate { index: 1, detection: Detection { score: 0.6, bounds: [0.2, 0.2, 1.0, 1.0], keypoints: points } };
        let output = weighted_nms(vec![second, first], 0.1, 2);
        assert_eq!(output.len(), 1);
        assert!((output[0].score - 0.9).abs() < 1e-6);
        assert!((output[0].bounds[0] - 0.08).abs() < 1e-6);
        assert!((output[0].bounds[1] - 0.08).abs() < 1e-6);
        assert!((output[0].bounds[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn equal_scores_follow_initial_anchor_order() {
        let points = [[0.0; 2]; NUM_KEYPOINTS];
        let first = Candidate { index: 4, detection: Detection { score: 0.5, bounds: [0.0, 0.0, 0.1, 0.1], keypoints: points } };
        let second = Candidate { index: 2, detection: Detection { score: 0.5, bounds: [1.0, 1.0, 0.1, 0.1], keypoints: points } };
        let output = weighted_nms(vec![first, second], 1.0, 1);
        assert_eq!(output[0].bounds, second.detection.bounds);
    }
}
