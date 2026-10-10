//! Offline raw MediaPipe graph qualification for an explicit tensor fixture.
//!
//! This harness measures graph execution only. It performs no image conversion, face detection,
//! crop selection, landmark remapping, eye-state calibration, catalog access, or culling.
//! Results are always unqualified evidence.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(not(target_arch = "wasm32"))]
mod native {
    use std::collections::HashSet;
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use candle_core::{Device, Tensor};
    use lightcraft_fetch::sha256_bytes;
    use lightcraft_segment::mediapipe::{DetectorOutput, LandmarksOutput};
    use lightcraft_segment::mediapipe_artifact::{BUNDLE_BYTES, BUNDLE_SHA256, Bundle, load_file};
    use serde::{Deserialize, Serialize};

    const INPUT_SCHEMA: &str = "ember.mediapipe-graph-input.v1";
    const REFERENCE_SCHEMA: &str = "ember.mediapipe-graph-reference.v1";
    const RECEIPT_SCHEMA: &str = "ember.mediapipe-qualification-receipt.v1";
    const MAX_INPUT_BYTES: usize = 8 * 1024 * 1024;
    const MAX_ELEMENTS: usize = 196_608;
    const MAX_TEXT_BYTES: usize = 512;
    const MAX_RECEIPT_BYTES: usize = 4 * 1024 * 1024;
    const MAX_REPEATS: usize = 30;
    const REPEAT_MAX_ABS_TOLERANCE: f64 = 1e-3;

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(rename_all = "lowercase")]
    enum GraphKind {
        Detector,
        Landmarks,
        Blendshapes,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct InputFile {
        schema: String,
        user_consented: bool,
        graph: GraphKind,
        shape: Vec<usize>,
        values: Vec<f32>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceFile {
        schema: String,
        label_provenance: String,
        graph: GraphKind,
        outputs: Vec<ReferenceOutput>,
    }

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceOutput {
        name: String,
        shape: Vec<usize>,
        values: Vec<f32>,
    }

    #[derive(Clone, Debug)]
    struct Cli {
        weights: PathBuf,
        input: PathBuf,
        device: String,
        hardware: String,
        out: PathBuf,
        reference: Option<PathBuf>,
        repeats: usize,
        source_revision: Option<String>,
    }

    #[derive(Clone, Copy)]
    struct OutputContract {
        name: &'static str,
        shape: &'static [usize],
    }

    #[derive(Clone, Debug)]
    struct RuntimeOutput {
        name: &'static str,
        tensor: Tensor,
    }

    #[derive(Clone, Debug)]
    struct OutputSnapshot {
        name: &'static str,
        shape: Vec<usize>,
        values: Vec<f32>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct OutputShapeReport {
        name: &'static str,
        shape: Vec<usize>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TimingReport {
        first_graph_us: u64,
        repeat_p50_us: u64,
        repeat_p95_us: u64,
        repeat_sample_count: usize,
        repeat_max_abs_error: f64,
        repeat_rmse: f64,
        diagnostic_note: &'static str,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceOutputReport {
        name: String,
        max_abs_error: f64,
        rmse: f64,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceReport {
        provided: bool,
        label_provenance: Option<String>,
        validation: &'static str,
        supplied_outputs: usize,
        matched_outputs: usize,
        model_outputs: usize,
        outputs: Vec<ReferenceOutputReport>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Receipt {
        schema: &'static str,
        qualification: &'static str,
        graph: GraphKind,
        model_sha256: &'static str,
        model_bytes: u64,
        input_sha256: String,
        device: String,
        device_debug: String,
        hardware: String,
        source_revision: Option<String>,
        input_shape: Vec<usize>,
        output_shapes: Vec<OutputShapeReport>,
        repeats: usize,
        model_load_us: u64,
        timing: TimingReport,
        reference_file_sha256: Option<String>,
        reference: ReferenceReport,
    }

    struct LoadedReference {
        file: ReferenceFile,
        sha256: String,
    }

    pub fn run() -> Result<(), Box<dyn std::error::Error>> {
        let cli = parse_args().map_err(std::io::Error::other)?;
        if cli.out.exists() {
            return Err(std::io::Error::other("output exists; refusing overwrite").into());
        }
        let input_bytes = read_bounded(&cli.input, MAX_INPUT_BYTES)?;
        let input: InputFile = serde_json::from_slice(&input_bytes)?;
        validate_input(&input)?;
        let input_sha256 = sha256_bytes(&input_bytes);
        let reference = cli.reference.as_deref().map(|path| load_reference(path, input.graph)).transpose()?;
        let (device, device_name) = select_device(&cli.device).map_err(std::io::Error::other)?;

        let model_start = Instant::now();
        let bundle = load_file(&cli.weights, &device)?;
        let model_load_us = elapsed_us(model_start);
        let input_tensor = Tensor::from_vec(input.values.clone(), input.shape.as_slice(), &device)?;
        let first_start = Instant::now();
        let first = capture_outputs(forward(&bundle, input.graph, &input_tensor)?, input.graph)?;
        let first_graph_us = elapsed_us(first_start);
        let mut timings = Vec::with_capacity(cli.repeats);
        let mut repeat_max_abs_error = 0.0f64;
        let mut repeat_rmse = 0.0f64;
        for _ in 0..cli.repeats {
            let start = Instant::now();
            let current = capture_outputs(forward(&bundle, input.graph, &input_tensor)?, input.graph)?;
            let elapsed = elapsed_us(start);
            let (max_abs_error, rmse) = compare_snapshots(&first, &current)?;
            if max_abs_error > REPEAT_MAX_ABS_TOLERANCE {
                return Err(std::io::Error::other("repeated MediaPipe outputs exceeded max-absolute tolerance").into());
            }
            repeat_max_abs_error = repeat_max_abs_error.max(max_abs_error);
            repeat_rmse = repeat_rmse.max(rmse);
            timings.push(elapsed);
        }
        let mut sorted = timings.clone();
        sorted.sort_unstable();
        let output_shapes = first.iter().map(|output| OutputShapeReport { name: output.name, shape: output.shape.clone() }).collect();
        let reference_report = reference_report(reference.as_ref().map(|loaded| &loaded.file), &first, input.graph)?;
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA,
            qualification: "UNQUALIFIED",
            graph: input.graph,
            model_sha256: BUNDLE_SHA256,
            model_bytes: BUNDLE_BYTES as u64,
            input_sha256,
            device: device_name,
            device_debug: format!("{device:?}"),
            hardware: cli.hardware,
            source_revision: cli.source_revision,
            input_shape: input.shape,
            output_shapes,
            repeats: cli.repeats,
            model_load_us,
            timing: TimingReport {
                first_graph_us,
                repeat_p50_us: percentile(&sorted, 0.50),
                repeat_p95_us: percentile(&sorted, 0.95),
                repeat_sample_count: timings.len(),
                repeat_max_abs_error,
                repeat_rmse,
                diagnostic_note: "per-node finite checks and output readbacks are diagnostic; throughput is unqualified",
            },
            reference_file_sha256: reference.as_ref().map(|loaded| loaded.sha256.clone()),
            reference: reference_report,
        };
        write_create_new_json(&cli.out, &receipt)?;
        Ok(())
    }

    fn output_contracts(graph: GraphKind) -> &'static [OutputContract] {
        match graph {
            GraphKind::Detector => {
                &[OutputContract { name: "regressors", shape: &[1, 896, 16] }, OutputContract { name: "classifierLogits", shape: &[1, 896, 1] }]
            }
            GraphKind::Landmarks => &[
                OutputContract { name: "coordinates", shape: &[1, 1, 1, 1434] },
                OutputContract { name: "auxiliary1", shape: &[1, 1, 1, 1] },
                OutputContract { name: "auxiliary2", shape: &[1, 1] },
            ],
            GraphKind::Blendshapes => &[OutputContract { name: "coefficients", shape: &[52] }],
        }
    }

    fn expected_input_shape(graph: GraphKind) -> &'static [usize] {
        match graph {
            GraphKind::Detector => &[1, 128, 128, 3],
            GraphKind::Landmarks => &[1, 256, 256, 3],
            GraphKind::Blendshapes => &[1, 146, 2],
        }
    }

    fn forward(bundle: &Bundle, graph: GraphKind, input: &Tensor) -> Result<Vec<RuntimeOutput>, Box<dyn std::error::Error>> {
        match graph {
            GraphKind::Detector => {
                let DetectorOutput { regressors, classifier_logits } = bundle.forward_detector(input)?;
                Ok(vec![
                    RuntimeOutput { name: "regressors", tensor: regressors },
                    RuntimeOutput { name: "classifierLogits", tensor: classifier_logits },
                ])
            }
            GraphKind::Landmarks => {
                let LandmarksOutput { coordinates, auxiliary_1, auxiliary_2 } = bundle.forward_landmarks(input)?;
                Ok(vec![
                    RuntimeOutput { name: "coordinates", tensor: coordinates },
                    RuntimeOutput { name: "auxiliary1", tensor: auxiliary_1 },
                    RuntimeOutput { name: "auxiliary2", tensor: auxiliary_2 },
                ])
            }
            GraphKind::Blendshapes => Ok(vec![RuntimeOutput { name: "coefficients", tensor: bundle.forward_blendshapes(input)? }]),
        }
    }

    fn capture_outputs(outputs: Vec<RuntimeOutput>, graph: GraphKind) -> Result<Vec<OutputSnapshot>, Box<dyn std::error::Error>> {
        let contracts = output_contracts(graph);
        if outputs.len() != contracts.len() {
            return Err(std::io::Error::other("MediaPipe graph returned unexpected output count").into());
        }
        outputs
            .into_iter()
            .zip(contracts)
            .map(|(output, contract)| {
                if output.name != contract.name || output.tensor.dims() != contract.shape {
                    return Err(std::io::Error::other(format!("MediaPipe output {} has wrong shape", output.name)).into());
                }
                let values = output.tensor.flatten_all()?.to_vec1::<f32>()?;
                if values.len() != checked_elements(contract.shape)? || values.iter().any(|value| !value.is_finite()) {
                    return Err(std::io::Error::other(format!("MediaPipe output {} is non-finite or has wrong length", output.name)).into());
                }
                Ok(OutputSnapshot { name: output.name, shape: contract.shape.to_vec(), values })
            })
            .collect()
    }

    fn compare_snapshots(left: &[OutputSnapshot], right: &[OutputSnapshot]) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        if left.len() != right.len() {
            return Err(std::io::Error::other("MediaPipe repeat output count changed").into());
        }
        let mut max_abs = 0.0f64;
        let mut sum_sq = 0.0f64;
        let mut count = 0usize;
        for (left, right) in left.iter().zip(right) {
            if left.name != right.name || left.shape != right.shape || left.values.len() != right.values.len() {
                return Err(std::io::Error::other("MediaPipe repeat output shape changed").into());
            }
            for (&a, &b) in left.values.iter().zip(&right.values) {
                let difference = (f64::from(a) - f64::from(b)).abs();
                max_abs = max_abs.max(difference);
                sum_sq += difference * difference;
                count = count.saturating_add(1);
            }
        }
        let rmse = if count == 0 { 0.0 } else { (sum_sq / count as f64).sqrt() };
        Ok((max_abs, rmse))
    }

    fn reference_report(
        reference: Option<&ReferenceFile>,
        actual: &[OutputSnapshot],
        graph: GraphKind,
    ) -> Result<ReferenceReport, Box<dyn std::error::Error>> {
        let Some(reference) = reference else {
            return Ok(ReferenceReport {
                provided: false,
                label_provenance: None,
                validation: "UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY",
                supplied_outputs: 0,
                matched_outputs: 0,
                model_outputs: actual.len(),
                outputs: Vec::new(),
            });
        };
        let contracts = output_contracts(graph);
        let mut reports = Vec::with_capacity(reference.outputs.len());
        for expected in &reference.outputs {
            let actual = actual
                .iter()
                .find(|output| output.name == expected.name)
                .ok_or_else(|| std::io::Error::other("MediaPipe reference output is missing from model output"))?;
            let (max_abs_error, rmse) = compare_values(&actual.values, &expected.values)?;
            reports.push(ReferenceOutputReport { name: expected.name.clone(), max_abs_error, rmse });
        }
        if reports.len() != contracts.len() {
            return Err(std::io::Error::other("MediaPipe reference output count does not match graph contract").into());
        }
        Ok(ReferenceReport {
            provided: true,
            label_provenance: Some(reference.label_provenance.clone()),
            validation: "UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY",
            supplied_outputs: reference.outputs.len(),
            matched_outputs: reports.len(),
            model_outputs: actual.len(),
            outputs: reports,
        })
    }

    fn compare_values(left: &[f32], right: &[f32]) -> Result<(f64, f64), Box<dyn std::error::Error>> {
        if left.len() != right.len() {
            return Err(std::io::Error::other("MediaPipe reference output length mismatch").into());
        }
        let mut max_abs = 0.0f64;
        let mut sum_sq = 0.0f64;
        for (&a, &b) in left.iter().zip(right) {
            if !a.is_finite() || !b.is_finite() {
                return Err(std::io::Error::other("MediaPipe reference comparison is non-finite").into());
            }
            let difference = (f64::from(a) - f64::from(b)).abs();
            max_abs = max_abs.max(difference);
            sum_sq += difference * difference;
        }
        let rmse = if left.is_empty() { 0.0 } else { (sum_sq / left.len() as f64).sqrt() };
        Ok((max_abs, rmse))
    }

    fn validate_input(input: &InputFile) -> Result<(), String> {
        if input.schema != INPUT_SCHEMA {
            return Err(format!("input schema must be {INPUT_SCHEMA}"));
        }
        if !input.user_consented {
            return Err("input userConsented must be true".into());
        }
        let expected = expected_input_shape(input.graph);
        if input.shape.as_slice() != expected {
            return Err(format!("input shape must be {expected:?}"));
        }
        let count = checked_elements(expected).map_err(|error| error.to_string())?;
        if count > MAX_ELEMENTS || input.values.len() != count {
            return Err("input values have wrong bounded element count".into());
        }
        if input.values.iter().any(|value| !value.is_finite()) {
            return Err("input values must be finite F32".into());
        }
        Ok(())
    }

    fn load_reference(path: &Path, graph: GraphKind) -> Result<LoadedReference, Box<dyn std::error::Error>> {
        let bytes = read_bounded(path, MAX_INPUT_BYTES)?;
        let reference: ReferenceFile = serde_json::from_slice(&bytes)?;
        validate_reference(&reference, graph).map_err(std::io::Error::other)?;
        Ok(LoadedReference { file: reference, sha256: sha256_bytes(&bytes) })
    }

    fn validate_reference(reference: &ReferenceFile, graph: GraphKind) -> Result<(), String> {
        if reference.schema != REFERENCE_SCHEMA || reference.graph != graph {
            return Err("reference schema or graph does not match input".into());
        }
        if reference.label_provenance.is_empty()
            || reference.label_provenance.len() > MAX_TEXT_BYTES
            || !reference.label_provenance.is_ascii()
            || reference.label_provenance.bytes().any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\')
        {
            return Err("reference labelProvenance is empty or unbounded".into());
        }
        let contracts = output_contracts(graph);
        if reference.outputs.len() != contracts.len() {
            return Err("reference output count does not match graph contract".into());
        }
        let mut names = HashSet::new();
        for output in &reference.outputs {
            if !names.insert(output.name.clone()) {
                return Err("reference output names must be unique".into());
            }
            let contract =
                contracts.iter().find(|contract| contract.name == output.name).ok_or_else(|| "reference output name is unknown".to_string())?;
            if output.shape.as_slice() != contract.shape
                || output.values.len() != checked_elements(contract.shape).map_err(|error| error.to_string())?
            {
                return Err(format!("reference output {} has wrong shape or length", output.name));
            }
            if output.values.iter().any(|value| !value.is_finite()) {
                return Err(format!("reference output {} contains non-finite values", output.name));
            }
        }
        Ok(())
    }

    fn checked_elements(shape: &[usize]) -> Result<usize, std::io::Error> {
        shape
            .iter()
            .try_fold(1usize, |count, dimension| count.checked_mul(*dimension))
            .ok_or_else(|| std::io::Error::other("tensor element count overflow"))
    }

    fn parse_args() -> Result<Cli, String> {
        let mut args = std::env::args().skip(1);
        let mut weights = None;
        let mut input = None;
        let mut device = None;
        let mut hardware = None;
        let mut out = None;
        let mut reference = None;
        let mut repeats = 3usize;
        let mut source_revision = None;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--weights" => weights = Some(next_arg(&mut args, &flag)?),
                "--input" => input = Some(next_arg(&mut args, &flag)?),
                "--device" => device = Some(next_arg(&mut args, &flag)?),
                "--hardware" => hardware = Some(parse_text(next_arg(&mut args, &flag)?, &flag)?),
                "--out" => out = Some(next_arg(&mut args, &flag)?),
                "--reference" => reference = Some(PathBuf::from(next_arg(&mut args, &flag)?)),
                "--repeats" => {
                    repeats = next_arg(&mut args, &flag)?.parse::<usize>().map_err(|_| "--repeats must be an integer".to_string())?;
                    if !(2..=MAX_REPEATS).contains(&repeats) {
                        return Err(format!("--repeats must be 2..{MAX_REPEATS}"));
                    }
                }
                "--source-revision" => source_revision = Some(parse_text(next_arg(&mut args, &flag)?, &flag)?),
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unknown option {other}\n{}", usage())),
            }
        }
        Ok(Cli {
            weights: PathBuf::from(weights.ok_or_else(|| "--weights is required".to_string())?),
            input: PathBuf::from(input.ok_or_else(|| "--input is required".to_string())?),
            device: device.ok_or_else(|| "--device cpu|metal is required".to_string())?,
            hardware: hardware.ok_or_else(|| "--hardware is required".to_string())?,
            out: PathBuf::from(out.ok_or_else(|| "--out is required".to_string())?),
            reference,
            repeats,
            source_revision,
        })
    }

    fn usage() -> String {
        "usage: mediapipe_qualify --weights FILE --input FILE --device cpu|metal --hardware TEXT --out FILE [--reference FILE] [--repeats 2..30] [--source-revision TEXT]".into()
    }

    fn next_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
        args.next().ok_or_else(|| format!("{flag} requires a value"))
    }

    fn parse_text(value: String, flag: &str) -> Result<String, String> {
        if value.is_empty() || value.len() > MAX_TEXT_BYTES {
            return Err(format!("{flag} must be 1..{MAX_TEXT_BYTES} bytes"));
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

    fn percentile(sorted: &[u64], quantile: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let rank = ((sorted.len() as f64 * quantile).ceil() as usize).saturating_sub(1).min(sorted.len() - 1);
        sorted[rank]
    }

    fn elapsed_us(start: Instant) -> u64 {
        start.elapsed().as_micros().min(u64::MAX as u128) as u64
    }

    fn write_create_new_json(path: &Path, receipt: &Receipt) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = encode_receipt(receipt)?;
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&bytes)?;
        Ok(())
    }

    fn encode_receipt(receipt: &Receipt) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let mut bytes = serde_json::to_vec_pretty(receipt)?;
        bytes.push(b'\n');
        if bytes.len() > MAX_RECEIPT_BYTES {
            return Err(std::io::Error::other(format!("receipt exceeds {MAX_RECEIPT_BYTES} byte bound")).into());
        }
        Ok(bytes)
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn input_contract_rejects_wrong_shape_nonfinite_and_missing_consent() {
            let mut input = InputFile {
                schema: INPUT_SCHEMA.into(),
                user_consented: true,
                graph: GraphKind::Detector,
                shape: vec![1, 128, 128, 3],
                values: vec![0.0; 128 * 128 * 3],
            };
            assert!(validate_input(&input).is_ok());
            input.shape = vec![1, 1, 1, 3];
            assert!(validate_input(&input).is_err());
            input.shape = vec![1, 128, 128, 3];
            input.values[0] = f32::NAN;
            assert!(validate_input(&input).is_err());
            input.values[0] = 0.0;
            input.user_consented = false;
            assert!(validate_input(&input).is_err());
        }

        #[test]
        fn reference_contract_rejects_wrong_graph_shape_and_nonfinite() {
            let reference = ReferenceFile {
                schema: REFERENCE_SCHEMA.into(),
                label_provenance: "independent".into(),
                graph: GraphKind::Blendshapes,
                outputs: vec![ReferenceOutput { name: "coefficients".into(), shape: vec![52], values: vec![0.0; 52] }],
            };
            assert!(validate_reference(&reference, GraphKind::Detector).is_err());
            assert!(validate_reference(&reference, GraphKind::Blendshapes).is_ok());
            let mut nonfinite = reference;
            if let Some(value) = nonfinite.outputs.first_mut().and_then(|output| output.values.first_mut()) {
                *value = f32::NAN;
            }
            assert!(validate_reference(&nonfinite, GraphKind::Blendshapes).is_err());
        }

        #[test]
        fn cli_text_and_output_bounds_are_explicit() {
            assert!(parse_text("hardware".into(), "--hardware").is_ok());
            assert!(parse_text("".into(), "--hardware").is_err());
            assert!(parse_text("x".repeat(MAX_TEXT_BYTES + 1), "--hardware").is_err());
            assert!(BUNDLE_BYTES > 0);
        }

        fn receipt_for_write_test(hardware: String) -> Receipt {
            Receipt {
                schema: RECEIPT_SCHEMA,
                qualification: "UNQUALIFIED",
                graph: GraphKind::Blendshapes,
                model_sha256: BUNDLE_SHA256,
                model_bytes: BUNDLE_BYTES as u64,
                input_sha256: "input".into(),
                device: "cpu".into(),
                device_debug: "cpu".into(),
                hardware,
                source_revision: None,
                input_shape: vec![1, 146, 2],
                output_shapes: vec![OutputShapeReport { name: "coefficients", shape: vec![52] }],
                repeats: 2,
                model_load_us: 0,
                timing: TimingReport {
                    first_graph_us: 0,
                    repeat_p50_us: 0,
                    repeat_p95_us: 0,
                    repeat_sample_count: 1,
                    repeat_max_abs_error: 0.0,
                    repeat_rmse: 0.0,
                    diagnostic_note: "test",
                },
                reference_file_sha256: None,
                reference: ReferenceReport {
                    provided: false,
                    label_provenance: None,
                    validation: "UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY",
                    supplied_outputs: 0,
                    matched_outputs: 0,
                    model_outputs: 1,
                    outputs: Vec::new(),
                },
            }
        }

        #[test]
        fn receipt_bound_and_create_new_semantics_are_enforced() {
            let oversized = receipt_for_write_test("x".repeat(MAX_RECEIPT_BYTES));
            assert!(encode_receipt(&oversized).is_err());
            let path = std::env::temp_dir().join(format!("mediapipe-qualify-{}.json", std::process::id()));
            let _ = std::fs::remove_file(&path);
            let receipt = receipt_for_write_test("cpu".into());
            assert!(write_create_new_json(&path, &receipt).is_ok());
            assert!(write_create_new_json(&path, &receipt).is_err());
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
    Err("mediapipe_qualify is unsupported on wasm32; run native cpu or metal qualification".into())
}
