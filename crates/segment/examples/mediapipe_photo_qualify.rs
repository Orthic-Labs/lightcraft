//! Offline, photo-level MediaPipe qualification for one explicit P6 RGB image.
//!
//! This harness measures the native photo API only. Receipts are always unqualified evidence;
//! raw eye evidence is diagnostic and never a probability or culling decision.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::HashSet;
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use candle_core::Device;
    use lightcraft_fetch::sha256_bytes;
    use lightcraft_segment::mediapipe_artifact::{BUNDLE_BYTES, BUNDLE_SHA256, load_file};
    use lightcraft_segment::mediapipe_blendshapes::COEFFICIENT_NAMES;
    use lightcraft_segment::mediapipe_input::RgbImage;
    use lightcraft_segment::mediapipe_photo::ImageOptions;
    use serde::{Deserialize, Serialize};

    const REFERENCE_SCHEMA: &str = "ember.mediapipe-photo-reference.v1";
    const RECEIPT_SCHEMA: &str = "ember.mediapipe-photo-qualification-receipt.v1";
    const MAX_PPM_BYTES: usize = 97 * 1024 * 1024;
    const MAX_IMAGE_PIXELS: usize = 32_000_000;
    const MAX_IMAGE_SIDE: usize = 16_384;
    const MAX_TEXT_BYTES: usize = 512;
    const MAX_RECEIPT_BYTES: usize = 4 * 1024 * 1024;
    const MAX_REPEATS: usize = 30;
    const MAX_LANDMARKS: usize = 10_000;
    const LANDMARK_COUNT: usize = 478;
    const MAX_COEFFICIENTS: usize = 52;
    const REPEAT_MAX_ABS_TOLERANCE: f64 = 1e-3;
    const MEDIAPIPE_SOURCE_REVISION: &str = "458200ffced12a9db596ddf1cce3380ea05f71fb";
    const SOURCE_DETECTOR_NORMALIZATION: &str = "detector RGB [0,255] -> [-1,1] (mean 127.5, std 127.5)";
    const SOURCE_LANDMARK_NORMALIZATION: &str = "landmark RGB [0,255] -> [0,1] (mean 0, std 255)";
    const SOURCE_INTERPOLATION: &str =
        "floating bilinear weights; source pixel centers at integer coordinates; destination samples before interpolation";
    const SOURCE_DETECTOR_PREPROCESS: &str = "128x128 square aspect-preserving zero letterbox";
    const SOURCE_LANDMARK_PREPROCESS: &str = "256x256 rotated ROI, non-aspect-preserving, replicate border";
    const MAX_NORMALIZED_COORD: f32 = 8.0;
    const MAX_RECT_SIZE: f32 = 8.0;
    const MAX_PROJECTED_Z: f32 = MAX_NORMALIZED_COORD * MAX_RECT_SIZE;
    const MAX_ROTATION: f32 = 1_000.0;

    #[derive(Clone, Debug)]
    struct Cli {
        weights: PathBuf,
        input: PathBuf,
        device: String,
        hardware: String,
        out: PathBuf,
        reference: Option<PathBuf>,
        detection_threshold: f32,
        nms_threshold: f32,
        presence_threshold: f32,
        max_faces: usize,
        /// Number of warm repeats after one first pass.
        repeats: usize,
        source_revision: Option<String>,
    }

    #[derive(Clone, Debug)]
    struct Ppm {
        width: usize,
        height: usize,
        rgb: Vec<u8>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceFile {
        schema: String,
        label_provenance: String,
        width: usize,
        height: usize,
        faces: Vec<ReferenceFace>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceFace {
        detection: ReferenceDetection,
        crop: ReferenceCrop,
        presence_score: f32,
        presence_passed: bool,
        raw_auxiliary_2: f32,
        landmarks: Option<Vec<[f32; 3]>>,
        coefficients: Option<Vec<ReferenceCoefficient>>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ReferenceDetection {
        score: f32,
        bounds: [f32; 4],
        keypoints: [[f32; 2]; 6],
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ReferenceCrop {
        center: [f32; 2],
        size: [f32; 2],
        rotation: f32,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    struct ReferenceCoefficient {
        name: String,
        value: f32,
    }

    #[derive(Clone, Debug)]
    struct LoadedReference {
        file: ReferenceFile,
        sha256: String,
    }

    #[derive(Clone, Debug, PartialEq, Eq)]
    struct SnapshotCounts {
        faces: usize,
        presence_passed: usize,
        landmarks_faces: usize,
        landmark_points: usize,
        coefficient_faces: usize,
        coefficients: usize,
        eye_evidence_faces: usize,
    }

    #[derive(Clone, Debug)]
    struct Snapshot {
        core: Vec<f64>,
        eye: Vec<f64>,
        coefficient_names: Vec<String>,
        counts: SnapshotCounts,
        eye_presence: Vec<bool>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SourcePreparation {
        pipeline: &'static str,
        input: &'static str,
        model_identity: &'static str,
        pinned_source_revision: &'static str,
        detector_normalization: &'static str,
        landmark_normalization: &'static str,
        interpolation: &'static str,
        detector_preprocess: &'static str,
        landmark_preprocess: &'static str,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Thresholds {
        detection_threshold: f32,
        nms_threshold: f32,
        presence_threshold: f32,
        max_faces: usize,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TimingReport {
        model_load_us: u64,
        first_end_to_end_us: u64,
        total_pass_count: usize,
        warm_repeat_p50_us: u64,
        warm_repeat_p95_us: u64,
        warm_repeat_count: usize,
        repeat_max_abs_error: f64,
        repeat_rmse: f64,
        stage: &'static str,
        cache_state: &'static str,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OutputCountReport {
        faces: usize,
        presence_passed: usize,
        landmarks_faces: usize,
        landmark_points: usize,
        coefficient_faces: usize,
        coefficients: usize,
        eye_evidence_faces: usize,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ErrorReport {
        shape_errors: usize,
        numeric_errors: usize,
        reference_errors: usize,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceReport {
        provided: bool,
        label_provenance: Option<String>,
        comparison: &'static str,
        supplied_faces: usize,
        compared_faces: usize,
        numeric_values_compared: usize,
        max_abs_error: Option<f64>,
        rmse: Option<f64>,
        shape_errors: usize,
        numeric_errors: usize,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Receipt {
        schema: &'static str,
        qualification: &'static str,
        input_sha256: String,
        model_sha256: &'static str,
        model_bytes: u64,
        reference_file_sha256: Option<String>,
        input_width: usize,
        input_height: usize,
        device: String,
        device_debug: String,
        hardware: String,
        source_revision: Option<String>,
        source_preparation: SourcePreparation,
        thresholds: Thresholds,
        repeats: usize,
        output_counts: OutputCountReport,
        timing: TimingReport,
        errors: ErrorReport,
        reference: ReferenceReport,
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = parse_args().map_err(std::io::Error::other)?;
        if cli.out.exists() {
            return Err(std::io::Error::other("output exists; refusing overwrite").into());
        }
        let input_bytes = read_bounded(&cli.input, MAX_PPM_BYTES)?;
        let input_sha256 = sha256_bytes(&input_bytes);
        let ppm = parse_ppm(&input_bytes).map_err(std::io::Error::other)?;
        let reference = cli.reference.as_deref().map(|path| load_reference(path, ppm.width, ppm.height)).transpose()?;
        let (device, device_name) = select_device(&cli.device).map_err(std::io::Error::other)?;
        let image = RgbImage::new(ppm.width, ppm.height, &ppm.rgb)?;
        let options = ImageOptions {
            detection_threshold: cli.detection_threshold,
            nms_threshold: cli.nms_threshold,
            presence_threshold: cli.presence_threshold,
            max_faces: cli.max_faces,
        };

        let model_start = Instant::now();
        let bundle = load_file(&cli.weights, &device)?;
        let model_load_us = elapsed_us(model_start);
        let mut timings = Vec::with_capacity(cli.repeats.saturating_add(1));
        let mut repeat_max_abs_error = 0.0f64;
        let mut repeat_rmse = 0.0f64;
        let first_start = Instant::now();
        let first_analysis = bundle.analyze_image(&image, &options, &device)?;
        let first_elapsed = elapsed_us(first_start);
        validate_analysis_dimensions(&first_analysis, ppm.width, ppm.height)?;
        let first = snapshot(&first_analysis).map_err(std::io::Error::other)?;
        timings.push(first_elapsed);
        for _ in 0..cli.repeats {
            let start = Instant::now();
            let current = bundle.analyze_image(&image, &options, &device)?;
            let elapsed = elapsed_us(start);
            validate_analysis_dimensions(&current, ppm.width, ppm.height)?;
            let current_snapshot = snapshot(&current).map_err(std::io::Error::other)?;
            let (max_abs, rmse) = compare_snapshots(&first, &current_snapshot).map_err(std::io::Error::other)?;
            if max_abs > REPEAT_MAX_ABS_TOLERANCE {
                return Err(std::io::Error::other("repeated MediaPipe photo outputs exceeded bounded numeric tolerance").into());
            }
            repeat_max_abs_error = repeat_max_abs_error.max(max_abs);
            repeat_rmse = repeat_rmse.max(rmse);
            timings.push(elapsed);
        }
        let mut warm = timings.get(1..).unwrap_or(&[]).to_vec();
        warm.sort_unstable();
        let reference_report = reference_report(reference.as_ref(), &first).map_err(std::io::Error::other)?;
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA,
            qualification: "UNQUALIFIED",
            input_sha256,
            model_sha256: BUNDLE_SHA256,
            model_bytes: BUNDLE_BYTES as u64,
            reference_file_sha256: reference.as_ref().map(|loaded| loaded.sha256.clone()),
            input_width: ppm.width,
            input_height: ppm.height,
            device: device_name,
            device_debug: format!("{device:?}"),
            hardware: cli.hardware,
            source_revision: cli.source_revision,
            source_preparation: SourcePreparation {
                pipeline: "native-rust-mediapipe-photo-v1",
                input: "explicit binary P6 RGB8 PPM; bounded parser",
                model_identity: "pinned MediaPipe Face Landmarker bundle",
                pinned_source_revision: MEDIAPIPE_SOURCE_REVISION,
                detector_normalization: SOURCE_DETECTOR_NORMALIZATION,
                landmark_normalization: SOURCE_LANDMARK_NORMALIZATION,
                interpolation: SOURCE_INTERPOLATION,
                detector_preprocess: SOURCE_DETECTOR_PREPROCESS,
                landmark_preprocess: SOURCE_LANDMARK_PREPROCESS,
            },
            thresholds: Thresholds {
                detection_threshold: cli.detection_threshold,
                nms_threshold: cli.nms_threshold,
                presence_threshold: cli.presence_threshold,
                max_faces: cli.max_faces,
            },
            repeats: cli.repeats,
            output_counts: output_count_report(&first.counts),
            timing: TimingReport {
                model_load_us,
                first_end_to_end_us: timings.first().copied().unwrap_or(0),
                total_pass_count: timings.len(),
                warm_repeat_p50_us: percentile(&warm, 0.50),
                warm_repeat_p95_us: percentile(&warm, 0.95),
                warm_repeat_count: warm.len(),
                repeat_max_abs_error,
                repeat_rmse,
                stage: "analyze_image total (preprocess+inference+postprocess)",
                cache_state: "first-after-model-load; filesystem-cache-state-unspecified",
            },
            errors: ErrorReport {
                shape_errors: 0,
                numeric_errors: 0,
                reference_errors: reference_report.shape_errors + reference_report.numeric_errors,
            },
            reference: reference_report,
        };
        write_create_new_json(&cli.out, &receipt)?;
        Ok(())
    }

    fn output_count_report(counts: &SnapshotCounts) -> OutputCountReport {
        OutputCountReport {
            faces: counts.faces,
            presence_passed: counts.presence_passed,
            landmarks_faces: counts.landmarks_faces,
            landmark_points: counts.landmark_points,
            coefficient_faces: counts.coefficient_faces,
            coefficients: counts.coefficients,
            eye_evidence_faces: counts.eye_evidence_faces,
        }
    }

    fn validate_analysis_dimensions(
        analysis: &lightcraft_segment::mediapipe_photo::ImageAnalysis,
        width: usize,
        height: usize,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if analysis.width != width || analysis.height != height {
            return Err(std::io::Error::other("MediaPipe photo output dimensions changed from input").into());
        }
        if analysis.faces.len() > 32 {
            return Err(std::io::Error::other("MediaPipe photo output face count exceeds 32").into());
        }
        Ok(())
    }

    fn snapshot(analysis: &lightcraft_segment::mediapipe_photo::ImageAnalysis) -> Result<Snapshot, String> {
        if analysis.faces.len() > 32 {
            return Err("face count exceeds 32".into());
        }
        let mut core = Vec::new();
        let mut eye = Vec::new();
        let mut coefficient_names = Vec::new();
        let mut counts = SnapshotCounts {
            faces: analysis.faces.len(),
            presence_passed: 0,
            landmarks_faces: 0,
            landmark_points: 0,
            coefficient_faces: 0,
            coefficients: 0,
            eye_evidence_faces: 0,
        };
        let mut eye_presence = Vec::with_capacity(analysis.faces.len());
        for face in &analysis.faces {
            validate_unit(face.detection.score, "detection score")?;
            validate_bounds(face.detection.bounds, "detection bounds")?;
            for point in face.detection.keypoints {
                validate_point(point, "detection keypoint")?;
            }
            validate_rect(face.crop.center, face.crop.size, face.crop.rotation, "crop")?;
            validate_unit(face.presence_score, "presence score")?;
            push_finite(&mut core, f64::from(face.detection.score), "detection score")?;
            for value in face.detection.bounds {
                push_finite(&mut core, f64::from(value), "detection bounds")?;
            }
            for point in face.detection.keypoints {
                push_finite(&mut core, f64::from(point[0]), "detection keypoint")?;
                push_finite(&mut core, f64::from(point[1]), "detection keypoint")?;
            }
            for value in face.crop.center {
                push_finite(&mut core, f64::from(value), "crop center")?;
            }
            for value in face.crop.size {
                push_finite(&mut core, f64::from(value), "crop size")?;
            }
            push_finite(&mut core, f64::from(face.crop.rotation), "crop rotation")?;
            push_finite(&mut core, f64::from(face.presence_score), "presence score")?;
            push_finite(&mut core, if face.presence_passed { 1.0 } else { 0.0 }, "presence state")?;
            if face.presence_passed {
                counts.presence_passed = counts.presence_passed.saturating_add(1);
            }
            push_finite(&mut core, f64::from(face.raw_auxiliary_2), "raw auxiliary")?;
            match (&face.landmarks, face.presence_passed) {
                (Some(landmarks), true) => {
                    if landmarks.len() > MAX_LANDMARKS {
                        return Err("landmark count exceeds bound".into());
                    }
                    if landmarks.len() != LANDMARK_COUNT {
                        return Err("landmark output must contain exactly 478 points".into());
                    }
                    counts.landmarks_faces = counts.landmarks_faces.saturating_add(1);
                    counts.landmark_points = counts.landmark_points.saturating_add(landmarks.len());
                    core.push(1.0);
                    for point in landmarks {
                        validate_landmark(*point, "landmark")?;
                        for value in point {
                            push_finite(&mut core, f64::from(*value), "landmark")?;
                        }
                    }
                }
                (None, false) => core.push(0.0),
                (Some(_), false) => return Err("presence-failed face must not include landmarks".into()),
                (None, true) => return Err("presence-passed face must include landmarks".into()),
            }
            match (&face.coefficients, face.presence_passed) {
                (Some(coefficients), true) => {
                    if coefficients.len() != MAX_COEFFICIENTS {
                        return Err("coefficient count exceeds bound".into());
                    }
                    counts.coefficient_faces = counts.coefficient_faces.saturating_add(1);
                    counts.coefficients = counts.coefficients.saturating_add(coefficients.len());
                    core.push(1.0);
                    for (index, coefficient) in coefficients.iter().enumerate() {
                        let expected_name = lightcraft_segment::mediapipe_blendshapes::COEFFICIENT_NAMES
                            .get(index)
                            .ok_or("coefficient index exceeds model contract")?;
                        if coefficient.name != *expected_name {
                            return Err("coefficient names/order changed from MediaPipe model contract".into());
                        }
                        coefficient_names.push(coefficient.name.to_owned());
                        push_finite(&mut core, f64::from(coefficient.value), "coefficient")?;
                    }
                }
                (None, false) => core.push(0.0),
                (Some(_), false) => return Err("presence-failed face must not include coefficients".into()),
                (None, true) => return Err("presence-passed face must include coefficients".into()),
            }
            eye_presence.push(face.eye_evidence.is_some());
            match (&face.eye_evidence, face.presence_passed) {
                (Some(evidence), true) => {
                    counts.eye_evidence_faces = counts.eye_evidence_faces.saturating_add(1);
                    for value in [
                        evidence.blink_left,
                        evidence.blink_right,
                        evidence.squint_left,
                        evidence.squint_right,
                        evidence.wide_left,
                        evidence.wide_right,
                    ] {
                        push_finite(&mut eye, f64::from(value), "raw eye evidence")?;
                    }
                }
                (None, false) => {}
                (Some(_), false) => return Err("presence-failed face must not include eye evidence".into()),
                (None, true) => return Err("presence-passed face must include eye evidence".into()),
            }
        }
        Ok(Snapshot { core, eye, coefficient_names, counts, eye_presence })
    }

    fn push_finite(values: &mut Vec<f64>, value: f64, field: &str) -> Result<(), String> {
        if value.is_finite() {
            values.push(value);
            Ok(())
        } else {
            Err(format!("{field} is non-finite"))
        }
    }

    fn compare_snapshots(left: &Snapshot, right: &Snapshot) -> Result<(f64, f64), String> {
        if left.core.len() != right.core.len()
            || left.eye.len() != right.eye.len()
            || left.coefficient_names != right.coefficient_names
            || left.counts != right.counts
            || left.eye_presence != right.eye_presence
        {
            return Err("repeated MediaPipe photo output shape/order changed".into());
        }
        compare_values(
            &left.core.iter().chain(left.eye.iter()).copied().collect::<Vec<_>>(),
            &right.core.iter().chain(right.eye.iter()).copied().collect::<Vec<_>>(),
        )
    }

    fn compare_values(left: &[f64], right: &[f64]) -> Result<(f64, f64), String> {
        if left.len() != right.len() {
            return Err("MediaPipe photo output shape changed".into());
        }
        let mut max_abs = 0.0f64;
        let mut sum_sq = 0.0f64;
        let mut count = 0usize;
        for (left_value, right_value) in left.iter().zip(right) {
            if !left_value.is_finite() || !right_value.is_finite() {
                return Err("MediaPipe photo numeric output is non-finite".into());
            }
            let difference = (left_value - right_value).abs();
            max_abs = max_abs.max(difference);
            sum_sq += difference * difference;
            count = count.saturating_add(1);
        }
        let rmse = if count == 0 { 0.0 } else { (sum_sq / count as f64).sqrt() };
        Ok((max_abs, rmse))
    }

    fn reference_report(reference: Option<&LoadedReference>, model: &Snapshot) -> Result<ReferenceReport, String> {
        let Some(reference) = reference else {
            return Ok(ReferenceReport {
                provided: false,
                label_provenance: None,
                comparison: "not-provided",
                supplied_faces: 0,
                compared_faces: 0,
                numeric_values_compared: 0,
                max_abs_error: None,
                rmse: None,
                shape_errors: 0,
                numeric_errors: 0,
            });
        };
        let expected = reference_snapshot(&reference.file)?;
        let mut report = ReferenceReport {
            provided: true,
            label_provenance: Some(reference.file.label_provenance.clone()),
            comparison: "unverified-ordered-comparison",
            supplied_faces: reference.file.faces.len(),
            compared_faces: 0,
            numeric_values_compared: 0,
            max_abs_error: None,
            rmse: None,
            shape_errors: 0,
            numeric_errors: 0,
        };
        if expected.core.len() != model.core.len()
            || expected.coefficient_names != model.coefficient_names
            || expected.counts != model.counts
            || expected.eye_presence != model.eye_presence
        {
            report.shape_errors = 1;
            return Ok(report);
        }
        let (max_abs, rmse) = compare_values(&expected.core, &model.core)?;
        report.compared_faces = reference.file.faces.len();
        report.numeric_values_compared = expected.core.len();
        report.max_abs_error = Some(max_abs);
        report.rmse = Some(rmse);
        report.numeric_errors = usize::from(max_abs > REPEAT_MAX_ABS_TOLERANCE);
        Ok(report)
    }

    fn reference_snapshot(reference: &ReferenceFile) -> Result<Snapshot, String> {
        let mut core = Vec::new();
        let mut coefficient_names = Vec::new();
        let mut counts = SnapshotCounts {
            faces: reference.faces.len(),
            presence_passed: 0,
            landmarks_faces: 0,
            landmark_points: 0,
            coefficient_faces: 0,
            coefficients: 0,
            eye_evidence_faces: 0,
        };
        let mut eye_presence = Vec::with_capacity(reference.faces.len());
        for face in &reference.faces {
            validate_unit(face.detection.score, "reference detection score")?;
            validate_bounds(face.detection.bounds, "reference detection bounds")?;
            for point in face.detection.keypoints {
                validate_point(point, "reference detection keypoint")?;
            }
            validate_rect(face.crop.center, face.crop.size, face.crop.rotation, "reference crop")?;
            validate_unit(face.presence_score, "reference presence score")?;
            push_finite(&mut core, f64::from(face.detection.score), "reference detection score")?;
            for value in face.detection.bounds {
                push_finite(&mut core, f64::from(value), "reference detection bounds")?;
            }
            for point in face.detection.keypoints {
                push_finite(&mut core, f64::from(point[0]), "reference keypoint")?;
                push_finite(&mut core, f64::from(point[1]), "reference keypoint")?;
            }
            for value in face.crop.center {
                push_finite(&mut core, f64::from(value), "reference crop center")?;
            }
            for value in face.crop.size {
                push_finite(&mut core, f64::from(value), "reference crop size")?;
            }
            push_finite(&mut core, f64::from(face.crop.rotation), "reference crop rotation")?;
            push_finite(&mut core, f64::from(face.presence_score), "reference presence score")?;
            push_finite(&mut core, if face.presence_passed { 1.0 } else { 0.0 }, "reference presence state")?;
            if face.presence_passed {
                counts.presence_passed = counts.presence_passed.saturating_add(1);
            }
            eye_presence.push(face.presence_passed);
            if face.presence_passed {
                counts.eye_evidence_faces = counts.eye_evidence_faces.saturating_add(1);
            }
            push_finite(&mut core, f64::from(face.raw_auxiliary_2), "reference raw auxiliary")?;
            match (&face.landmarks, face.presence_passed) {
                (Some(landmarks), true) => {
                    core.push(1.0);
                    counts.landmarks_faces = counts.landmarks_faces.saturating_add(1);
                    counts.landmark_points = counts.landmark_points.saturating_add(landmarks.len());
                    for point in landmarks {
                        validate_landmark(*point, "reference landmark")?;
                        for value in point {
                            push_finite(&mut core, f64::from(*value), "reference landmark")?;
                        }
                    }
                }
                (None, false) => core.push(0.0),
                _ => return Err("reference optional outputs disagree with presencePassed".into()),
            }
            match (&face.coefficients, face.presence_passed) {
                (Some(coefficients), true) => {
                    core.push(1.0);
                    counts.coefficient_faces = counts.coefficient_faces.saturating_add(1);
                    counts.coefficients = counts.coefficients.saturating_add(coefficients.len());
                    for coefficient in coefficients {
                        coefficient_names.push(coefficient.name.clone());
                        push_finite(&mut core, f64::from(coefficient.value), "reference coefficient")?;
                    }
                }
                (None, false) => core.push(0.0),
                _ => return Err("reference optional outputs disagree with presencePassed".into()),
            }
        }
        Ok(Snapshot { core, eye: Vec::new(), coefficient_names, counts, eye_presence })
    }

    fn load_reference(path: &Path, width: usize, height: usize) -> Result<LoadedReference, Box<dyn std::error::Error>> {
        let bytes = read_bounded(path, MAX_RECEIPT_BYTES)?;
        let reference: ReferenceFile = serde_json::from_slice(&bytes)?;
        validate_reference(&reference, width, height).map_err(std::io::Error::other)?;
        Ok(LoadedReference { file: reference, sha256: sha256_bytes(&bytes) })
    }

    fn validate_reference(reference: &ReferenceFile, width: usize, height: usize) -> Result<(), String> {
        if reference.schema != REFERENCE_SCHEMA {
            return Err(format!("reference schema must be {REFERENCE_SCHEMA}"));
        }
        validate_text(&reference.label_provenance, "reference labelProvenance")?;
        if reference.width != width || reference.height != height {
            return Err("reference dimensions must exactly match input".into());
        }
        if reference.faces.len() > 32 {
            return Err("reference face count exceeds 32".into());
        }
        for face in &reference.faces {
            validate_unit(face.detection.score, "reference detection score")?;
            validate_bounds(face.detection.bounds, "reference detection bounds")?;
            for point in face.detection.keypoints {
                validate_point(point, "reference detection keypoint")?;
            }
            validate_rect(face.crop.center, face.crop.size, face.crop.rotation, "reference crop")?;
            validate_unit(face.presence_score, "reference presence score")?;
            validate_f32(face.raw_auxiliary_2, "reference raw auxiliary")?;
            match (&face.landmarks, face.presence_passed) {
                (Some(landmarks), true) => {
                    if landmarks.len() > MAX_LANDMARKS {
                        return Err("reference landmark count exceeds bound".into());
                    }
                    if landmarks.len() != LANDMARK_COUNT {
                        return Err("reference landmarks must contain exactly 478 points".into());
                    }
                    for point in landmarks {
                        validate_landmark(*point, "reference landmark")?;
                    }
                }
                (None, false) => {}
                _ => return Err("reference landmarks disagree with presencePassed".into()),
            }
            match (&face.coefficients, face.presence_passed) {
                (Some(coefficients), true) => {
                    if coefficients.len() != MAX_COEFFICIENTS {
                        return Err("reference coefficients must contain exactly 52 values".into());
                    }
                    let mut names = HashSet::new();
                    for (index, coefficient) in coefficients.iter().enumerate() {
                        validate_text(&coefficient.name, "reference coefficient name")?;
                        let expected_name = COEFFICIENT_NAMES.get(index).ok_or("reference coefficient index exceeds model contract")?;
                        if coefficient.name != *expected_name {
                            return Err("reference coefficient names must use MediaPipe model order".into());
                        }
                        if !names.insert(coefficient.name.clone()) {
                            return Err("reference coefficient names must be unique per face".into());
                        }
                        validate_f32(coefficient.value, "reference coefficient")?;
                    }
                }
                (None, false) => {}
                _ => return Err("reference coefficients disagree with presencePassed".into()),
            }
        }
        Ok(())
    }

    fn validate_f32(value: f32, field: &str) -> Result<(), String> {
        value.is_finite().then_some(()).ok_or_else(|| format!("{field} must be finite"))
    }

    fn validate_unit(value: f32, field: &str) -> Result<(), String> {
        if value.is_finite() && (0.0..=1.0).contains(&value) { Ok(()) } else { Err(format!("{field} must be finite and within [0,1]")) }
    }

    fn validate_point(point: [f32; 2], field: &str) -> Result<(), String> {
        if point.iter().all(|value| value.is_finite() && value.abs() <= MAX_NORMALIZED_COORD) {
            Ok(())
        } else {
            Err(format!("{field} has an out-of-bounds normalized coordinate"))
        }
    }

    fn validate_bounds(bounds: [f32; 4], field: &str) -> Result<(), String> {
        let [xmin, ymin, width, height] = bounds;
        if !xmin.is_finite()
            || !ymin.is_finite()
            || !width.is_finite()
            || !height.is_finite()
            || xmin.abs() > MAX_NORMALIZED_COORD
            || ymin.abs() > MAX_NORMALIZED_COORD
            || width <= 0.0
            || height <= 0.0
            || width > MAX_RECT_SIZE
            || height > MAX_RECT_SIZE
            || (xmin + width).abs() > MAX_NORMALIZED_COORD
            || (ymin + height).abs() > MAX_NORMALIZED_COORD
        {
            return Err(format!("{field} is non-finite or outside bounded normalized rectangle"));
        }
        Ok(())
    }

    fn validate_rect(center: [f32; 2], size: [f32; 2], rotation: f32, field: &str) -> Result<(), String> {
        validate_point(center, field)?;
        if size.iter().any(|value| !value.is_finite() || *value <= 0.0 || *value > MAX_RECT_SIZE) {
            return Err(format!("{field} size is invalid"));
        }
        if !rotation.is_finite() || rotation.abs() > MAX_ROTATION {
            return Err(format!("{field} rotation is invalid"));
        }
        Ok(())
    }

    fn validate_landmark(point: [f32; 3], field: &str) -> Result<(), String> {
        if point[0].is_finite()
            && point[1].is_finite()
            && point[2].is_finite()
            && point[0].abs() <= MAX_NORMALIZED_COORD
            && point[1].abs() <= MAX_NORMALIZED_COORD
            && point[2].abs() <= MAX_PROJECTED_Z
        {
            Ok(())
        } else {
            Err(format!("{field} has an out-of-bounds normalized coordinate"))
        }
    }

    fn validate_text(value: &str, field: &str) -> Result<(), String> {
        if value.is_empty()
            || value.len() > MAX_TEXT_BYTES
            || !value.is_ascii()
            || value.bytes().any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\')
        {
            return Err(format!("{field} is empty, unbounded, non-ASCII, or path-like"));
        }
        Ok(())
    }

    fn parse_args() -> Result<Cli, String> {
        let mut args = std::env::args().skip(1);
        let mut weights = None;
        let mut input = None;
        let mut consent = None;
        let mut device = None;
        let mut hardware = None;
        let mut out = None;
        let mut reference = None;
        let mut detection_threshold = None;
        let mut nms_threshold = None;
        let mut presence_threshold = None;
        let mut max_faces = None;
        let mut repeats = None;
        let mut source_revision = None;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--weights" => set_once(&mut weights, next_arg(&mut args, &flag)?, &flag)?,
                "--input" => set_once(&mut input, next_arg(&mut args, &flag)?, &flag)?,
                "--consent" => set_once(&mut consent, next_arg(&mut args, &flag)?, &flag)?,
                "--device" => set_once(&mut device, next_arg(&mut args, &flag)?, &flag)?,
                "--hardware" => set_once(&mut hardware, parse_text(next_arg(&mut args, &flag)?, &flag)?, &flag)?,
                "--out" => set_once(&mut out, next_arg(&mut args, &flag)?, &flag)?,
                "--reference" => set_once(&mut reference, PathBuf::from(next_arg(&mut args, &flag)?), &flag)?,
                "--detection-threshold" => set_once(&mut detection_threshold, parse_unit(next_arg(&mut args, &flag)?, &flag)?, &flag)?,
                "--nms-threshold" => set_once(&mut nms_threshold, parse_unit(next_arg(&mut args, &flag)?, &flag)?, &flag)?,
                "--presence-threshold" => set_once(&mut presence_threshold, parse_unit(next_arg(&mut args, &flag)?, &flag)?, &flag)?,
                "--max-faces" => {
                    let value = next_arg(&mut args, &flag)?.parse::<usize>().map_err(|_| "--max-faces must be an integer".to_string())?;
                    if !(1..=32).contains(&value) {
                        return Err("--max-faces must be 1..32".into());
                    }
                    set_once(&mut max_faces, value, &flag)?;
                }
                "--repeats" => {
                    let value = next_arg(&mut args, &flag)?.parse::<usize>().map_err(|_| "--repeats must be an integer".to_string())?;
                    if !(2..=MAX_REPEATS).contains(&value) {
                        return Err(format!("--repeats must be 2..{MAX_REPEATS}"));
                    }
                    set_once(&mut repeats, value, &flag)?;
                }
                "--source-revision" => set_once(&mut source_revision, parse_text(next_arg(&mut args, &flag)?, &flag)?, &flag)?,
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unknown option {other}\n{}", usage())),
            }
        }
        validate_consent(consent.as_deref())?;
        Ok(Cli {
            weights: PathBuf::from(weights.ok_or_else(|| "--weights is required".to_string())?),
            input: PathBuf::from(input.ok_or_else(|| "--input is required".to_string())?),
            device: device.ok_or_else(|| "--device cpu|metal is required".to_string())?,
            hardware: hardware.ok_or_else(|| "--hardware is required".to_string())?,
            out: PathBuf::from(out.ok_or_else(|| "--out is required".to_string())?),
            reference,
            detection_threshold: detection_threshold.ok_or_else(|| "--detection-threshold is required".to_string())?,
            nms_threshold: nms_threshold.ok_or_else(|| "--nms-threshold is required".to_string())?,
            presence_threshold: presence_threshold.ok_or_else(|| "--presence-threshold is required".to_string())?,
            max_faces: max_faces.ok_or_else(|| "--max-faces is required".to_string())?,
            repeats: repeats.unwrap_or(3),
            source_revision,
        })
    }

    fn set_once<T>(slot: &mut Option<T>, value: T, flag: &str) -> Result<(), String> {
        if slot.is_some() {
            return Err(format!("duplicate argument {flag}"));
        }
        *slot = Some(value);
        Ok(())
    }

    fn validate_consent(consent: Option<&str>) -> Result<(), String> {
        if consent == Some("yes") { Ok(()) } else { Err("--consent yes is required".into()) }
    }

    fn usage() -> String {
        "usage: mediapipe_photo_qualify --weights FILE --input P6RGB8PPM --consent yes --device cpu|metal --hardware TEXT --out NEW.json --detection-threshold 0..1 --nms-threshold 0..1 --presence-threshold 0..1 --max-faces 1..32 [--reference FILE] [--repeats WARM_2..30] [--source-revision TEXT]".into()
    }

    fn next_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
        args.next().ok_or_else(|| format!("{flag} requires a value"))
    }

    fn parse_unit(value: String, flag: &str) -> Result<f32, String> {
        let parsed = value.parse::<f32>().map_err(|_| format!("{flag} must be finite and in [0,1]"))?;
        if !parsed.is_finite() || !(0.0..=1.0).contains(&parsed) {
            return Err(format!("{flag} must be finite and in [0,1]"));
        }
        Ok(parsed)
    }

    fn parse_text(value: String, flag: &str) -> Result<String, String> {
        if value.is_empty() || value.len() > MAX_TEXT_BYTES || value.bytes().any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\') {
            return Err(format!("{flag} must be 1..{MAX_TEXT_BYTES} bytes without control characters"));
        }
        Ok(value)
    }

    fn select_device(name: &str) -> Result<(Device, String), String> {
        match name {
            "cpu" => Ok((Device::Cpu, "cpu".into())),
            "metal" => {
                #[cfg(target_os = "macos")]
                {
                    let device = Device::new_metal(0).map_err(|error| format!("Metal device unavailable: {error}"))?;
                    Ok((device, "metal".into()))
                }
                #[cfg(not(target_os = "macos"))]
                {
                    Err("--device metal is supported only on macOS".into())
                }
            }
            other => Err(format!("unknown device {other}; choose cpu or metal")),
        }
    }

    fn read_bounded(path: &Path, max_bytes: usize) -> Result<Vec<u8>, std::io::Error> {
        let file = File::open(path)?;
        let mut bytes = Vec::new();
        file.take((max_bytes as u64).saturating_add(1)).read_to_end(&mut bytes)?;
        if bytes.len() > max_bytes {
            return Err(std::io::Error::other(format!("input exceeds {max_bytes} byte bound")));
        }
        Ok(bytes)
    }

    fn parse_ppm(bytes: &[u8]) -> Result<Ppm, String> {
        let mut cursor = 0usize;
        if next_token(bytes, &mut cursor)? != b"P6" {
            return Err("PPM must use binary P6".into());
        }
        let width = parse_positive_usize(&next_token(bytes, &mut cursor)?, "width")?;
        let height = parse_positive_usize(&next_token(bytes, &mut cursor)?, "height")?;
        if width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE {
            return Err(format!("PPM dimensions must be <= {MAX_IMAGE_SIDE}"));
        }
        let maxval = parse_positive_usize(&next_token(bytes, &mut cursor)?, "maxval")?;
        if maxval != 255 {
            return Err("PPM maxval must be 255".into());
        }
        let pixels = width.checked_mul(height).ok_or_else(|| "PPM dimensions overflow".to_string())?;
        if pixels > MAX_IMAGE_PIXELS {
            return Err(format!("PPM exceeds {MAX_IMAGE_PIXELS} pixel bound"));
        }
        let expected = pixels.checked_mul(3).ok_or_else(|| "PPM RGB length overflow".to_string())?;
        let separator = *bytes.get(cursor).ok_or_else(|| "PPM missing raster separator".to_string())?;
        if !separator.is_ascii_whitespace() {
            return Err("PPM header must end with whitespace".into());
        }
        // P6 permits CRLF. If one-byte consumption already gives the exact raster length,
        // preserve a first raster byte equal to LF; consume two bytes only when needed.
        let one_byte_raster_len = bytes.len().saturating_sub(cursor.saturating_add(1));
        cursor = cursor.saturating_add(1);
        if separator == b'\r'
            && bytes.get(cursor) == Some(&b'\n')
            && one_byte_raster_len != expected
            && one_byte_raster_len.saturating_sub(1) == expected
        {
            cursor = cursor.saturating_add(1);
        }
        let raster = bytes.get(cursor..).ok_or_else(|| "PPM raster offset out of range".to_string())?;
        if raster.len() != expected {
            return Err(format!("PPM raster has {}, expected {expected} bytes", raster.len()));
        }
        Ok(Ppm { width, height, rgb: raster.to_vec() })
    }

    fn next_token<'a>(bytes: &'a [u8], cursor: &mut usize) -> Result<&'a [u8], String> {
        while let Some(byte) = bytes.get(*cursor) {
            if byte.is_ascii_whitespace() {
                *cursor = cursor.saturating_add(1);
            } else if *byte == b'#' {
                while let Some(comment_byte) = bytes.get(*cursor) {
                    *cursor = cursor.saturating_add(1);
                    if *comment_byte == b'\n' {
                        break;
                    }
                }
            } else {
                break;
            }
        }
        let start = *cursor;
        while let Some(byte) = bytes.get(*cursor) {
            if byte.is_ascii_whitespace() || *byte == b'#' {
                break;
            }
            *cursor = cursor.saturating_add(1);
        }
        if *cursor == start {
            Err("PPM header token missing".into())
        } else {
            bytes.get(start..*cursor).ok_or_else(|| "PPM header token exceeds input".into())
        }
    }

    fn parse_positive_usize(token: &[u8], label: &str) -> Result<usize, String> {
        let text = std::str::from_utf8(token).map_err(|_| format!("{label} is not ASCII"))?;
        let value = text.parse::<usize>().map_err(|_| format!("{label} is not a positive integer"))?;
        if value == 0 { Err(format!("{label} must be positive")) } else { Ok(value) }
    }

    fn percentile(sorted: &[u64], quantile: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let rank = ((sorted.len() as f64 * quantile).ceil() as usize).saturating_sub(1).min(sorted.len() - 1);
        sorted.get(rank).copied().unwrap_or(0)
    }

    fn elapsed_us(start: Instant) -> u64 {
        start.elapsed().as_micros().min(u64::MAX as u128) as u64
    }

    fn write_create_new_json(path: &Path, receipt: &Receipt) -> Result<(), Box<dyn std::error::Error>> {
        let mut bytes = serde_json::to_vec_pretty(receipt)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_RECEIPT_BYTES {
            return Err(std::io::Error::other(format!("receipt exceeds {MAX_RECEIPT_BYTES} byte bound")).into());
        }
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn ppm_parser_rejects_non_p6_nonfinite_dimensions_and_wrong_raster() {
            assert!(parse_ppm(b"P3 1 1 255\n\0\0\0").is_err());
            assert!(parse_ppm(b"P6 0 1 255\n").is_err());
            assert!(parse_ppm(b"P6 1 1 255\n\0\0").is_err());
        }

        #[test]
        fn ppm_parser_handles_crlf_without_eating_first_lf_pixel() {
            assert_eq!(parse_ppm(b"P6 1 1 255\r\n\x01\x02\x03").map(|ppm| ppm.rgb), Ok(vec![1, 2, 3]));
            // A one-byte header separator can be followed by an LF-valued first pixel.
            assert_eq!(parse_ppm(b"P6 1 1 255\r\n\x01\x02").map(|ppm| ppm.rgb), Ok(vec![b'\n', 1, 2]));
        }

        #[test]
        fn cli_rejects_duplicate_arguments_and_missing_consent() {
            assert!(set_once(&mut Some("x"), "y", "--hardware").is_err());
            assert!(validate_consent(Some("no")).is_err());
            assert!(validate_consent(None).is_err());
            assert!(validate_consent(Some("yes")).is_ok());
            let mut args = vec!["--consent".to_string(), "no".to_string()].into_iter();
            assert!(next_arg(&mut args, "--consent").is_ok());
            assert!(parse_unit("NaN".into(), "--detection-threshold").is_err());
            assert!(parse_text("hardware/path".into(), "--hardware").is_err());
            assert!(parse_text("revision\\path".into(), "--source-revision").is_err());
        }

        #[test]
        fn landmark_bounds_match_projected_xy_and_z_contract() {
            assert!(validate_landmark([0.5, 0.5, 32.0], "landmark").is_ok());
            assert!(validate_landmark([8.1, 0.5, 0.0], "landmark").is_err());
            assert!(validate_landmark([0.5, 0.5, 65.0], "landmark").is_err());
        }

        #[test]
        fn reference_requires_exact_dimensions_finite_values_and_52_coefficients() {
            let reference =
                ReferenceFile { schema: REFERENCE_SCHEMA.into(), label_provenance: "fixture".into(), width: 2, height: 2, faces: Vec::new() };
            assert!(validate_reference(&reference, 3, 2).is_err());
            assert!(validate_reference(&reference, 2, 2).is_ok());

            let incoherent = ReferenceFile {
                faces: vec![ReferenceFace {
                    detection: ReferenceDetection { score: 0.5, bounds: [0.0; 4], keypoints: [[0.0; 2]; 6] },
                    crop: ReferenceCrop { center: [0.0; 2], size: [1.0; 2], rotation: 0.0 },
                    presence_score: 0.0,
                    presence_passed: false,
                    raw_auxiliary_2: 0.0,
                    landmarks: None,
                    coefficients: Some(Vec::new()),
                }],
                ..reference.clone()
            };
            assert!(validate_reference(&incoherent, 2, 2).is_err());

            let wrong_count = ReferenceFile {
                faces: vec![ReferenceFace {
                    detection: ReferenceDetection { score: 0.5, bounds: [0.0; 4], keypoints: [[0.0; 2]; 6] },
                    crop: ReferenceCrop { center: [0.0; 2], size: [1.0; 2], rotation: 0.0 },
                    presence_score: 1.0,
                    presence_passed: true,
                    raw_auxiliary_2: 0.0,
                    landmarks: Some(vec![[0.0; 3]; LANDMARK_COUNT]),
                    coefficients: Some(Vec::new()),
                }],
                ..reference
            };
            assert!(validate_reference(&wrong_count, 2, 2).is_err());
        }

        #[test]
        fn output_is_create_new() {
            let path = std::env::temp_dir().join(format!("mediapipe-photo-{}.json", std::process::id()));
            let _ = std::fs::remove_file(&path);
            let result = std::fs::OpenOptions::new().write(true).create_new(true).open(&path);
            assert!(result.is_ok());
            assert!(std::fs::OpenOptions::new().write(true).create_new(true).open(&path).is_err());
            let _ = std::fs::remove_file(path);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("mediapipe_photo_qualify is unsupported on wasm32; run native cpu or metal qualification".into())
}
