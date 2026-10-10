//! Offline YuNet qualification on an explicit, user-consented PPM manifest.
//!
//! This example measures a pinned YuNet artifact against normalized ground-truth boxes. It reads
//! no catalog, calls no provider, writes only a create-new JSON receipt, and never emits input
//! paths. Metrics stay UNQUALIFIED until an independent golden reference and held-out-shoot
//! criteria are satisfied.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

use std::collections::{BTreeMap, HashSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Instant;

use candle_core::Device;
use serde::{Deserialize, Serialize};

use lightcraft_segment::yunet::{Detection, YuNet};

const MANIFEST_SCHEMA: &str = "lightcraft.yunet-qualification.v1";
const RECEIPT_SCHEMA: &str = "lightcraft.yunet-qualification-receipt.v1";
const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
const MAX_PPM_BYTES: usize = 100 * 1024 * 1024;
const MAX_IMAGE_PIXELS: usize = 64_000_000;
const MAX_IMAGES: usize = 100_000;
const MAX_ID_BYTES: usize = 512;
const MAX_REPEATS: usize = 30;
/// Bounded cross-device repeat tolerance; count, order, and detection cardinality remain exact.
const DETECTION_ABS_TOLERANCE: f32 = 1e-4;
const DETECTION_REL_TOLERANCE: f32 = 1e-4;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct Manifest {
    schema: String,
    user_consented: bool,
    images: Vec<ManifestImage>,
}

#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct ManifestImage {
    id: String,
    shoot_id: String,
    split: Split,
    crop_slice: CropSlice,
    ppm: String,
    /// Some([]) is a known negative; None is unknown and excluded from precision/recall.
    boxes: Option<Vec<NormalizedBox>>,
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
enum Split {
    Train,
    Development,
    Heldout,
}

impl Split {
    fn as_str(self) -> &'static str {
        match self {
            Self::Train => "train",
            Self::Development => "development",
            Self::Heldout => "heldout",
        }
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
enum CropSlice {
    Full,
    Cropped,
    Smallfaces,
}

impl CropSlice {
    fn as_str(self) -> &'static str {
        match self {
            Self::Full => "full",
            Self::Cropped => "cropped",
            Self::Smallfaces => "smallfaces",
        }
    }
}

/// Ground truth box normalized to supplied PPM width/height: 0 <= x/y <= 1.
#[derive(Clone, Copy, Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NormalizedBox {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

#[derive(Clone, Debug)]
struct Ppm {
    width: usize,
    height: usize,
    rgb: Vec<u8>,
}

#[derive(Clone, Debug)]
struct Cli {
    weights: PathBuf,
    manifest: PathBuf,
    device: String,
    out: PathBuf,
    score_threshold: f32,
    nms_threshold: f32,
    match_iou_threshold: f32,
    repeats: usize,
    hardware: String,
    source_revision: Option<String>,
}

#[derive(Clone, Copy, Debug, Default)]
struct Counts {
    true_positive: usize,
    false_positive: usize,
    false_negative: usize,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct MetricReport {
    images_total: usize,
    labelled_images: usize,
    unknown_images_excluded: usize,
    ground_truth_boxes: usize,
    detections_total: usize,
    metric_eligible_detections: usize,
    true_positive: usize,
    false_positive: usize,
    false_negative: usize,
    precision: Option<f64>,
    recall: Option<f64>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct TimingReport {
    repetitions: usize,
    first_pass_us: u64,
    warm_p50_us: u64,
    warm_p95_us: u64,
    warm_sample_count: usize,
    cache_state: &'static str,
    stage: &'static str,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ImageReport {
    id: String,
    shoot_id: String,
    split: Split,
    crop_slice: CropSlice,
    labelled: bool,
    ground_truth_boxes: usize,
    detections: usize,
    true_positive: usize,
    false_positive: usize,
    false_negative: usize,
    timing: TimingReport,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct StratumReport {
    split: Split,
    crop_slice: CropSlice,
    metrics: MetricReport,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ShootReport {
    shoot_id: String,
    split: Split,
    metrics: MetricReport,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct Receipt {
    schema: &'static str,
    qualification: &'static str,
    golden_reference_matched: bool,
    heldout_shoot_criteria_met: bool,
    memory_measured: bool,
    model_sha256: &'static str,
    device: String,
    device_debug: String,
    hardware: String,
    source_revision: Option<String>,
    manifest_schema: String,
    config: ConfigReceipt,
    model_load_us: u64,
    model_load_cache_state: &'static str,
    overall: MetricReport,
    by_stratum: Vec<StratumReport>,
    by_shoot: Vec<ShootReport>,
    images: Vec<ImageReport>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct ConfigReceipt {
    score_threshold: f32,
    nms_threshold: f32,
    match_iou_threshold: f32,
    repeats: usize,
    input: &'static str,
    box_space: &'static str,
    preprocess: &'static str,
    preprocess_status: &'static str,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = parse_args().map_err(std::io::Error::other)?;
    let manifest_bytes = read_bounded(&cli.manifest, MAX_MANIFEST_BYTES)?;
    let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
    validate_manifest(&manifest)?;
    let base = cli.manifest.parent().unwrap_or_else(|| Path::new("."));
    let (device, device_name) = select_device(&cli.device).map_err(std::io::Error::other)?;

    let load_start = Instant::now();
    let model = YuNet::load(&cli.weights, &device)?;
    let model_load_us = elapsed_us(load_start);

    let mut images = Vec::with_capacity(manifest.images.len());
    for item in &manifest.images {
        let path = resolve_manifest_path(base, &item.ppm);
        let ppm = load_ppm(&path)?;
        let (detections, timing) = detect_repeated(&model, &ppm, &cli)?;
        let known_boxes = item.boxes.as_deref();
        let counts = known_boxes.map(|boxes| match_detections(&detections, boxes, ppm.width, ppm.height, cli.match_iou_threshold));
        images.push(ImageReport {
            id: item.id.clone(),
            shoot_id: item.shoot_id.clone(),
            split: item.split,
            crop_slice: item.crop_slice,
            labelled: known_boxes.is_some(),
            ground_truth_boxes: known_boxes.map_or(0, |boxes| boxes.len()),
            detections: detections.len(),
            true_positive: counts.map_or(0, |v| v.true_positive),
            false_positive: counts.map_or(0, |v| v.false_positive),
            false_negative: counts.map_or(0, |v| v.false_negative),
            timing,
        });
    }

    let overall = metric_report(&images, None, None);
    let by_stratum = stratum_reports(&images);
    let by_shoot = shoot_reports(&images);
    let receipt = Receipt {
        schema: RECEIPT_SCHEMA,
        qualification: "UNQUALIFIED",
        golden_reference_matched: false,
        heldout_shoot_criteria_met: false,
        memory_measured: false,
        model_sha256: YuNet::PINNED_SHA256,
        device: device_name,
        device_debug: format!("{device:?}"),
        hardware: cli.hardware,
        source_revision: cli.source_revision,
        manifest_schema: manifest.schema,
        config: ConfigReceipt {
            score_threshold: cli.score_threshold,
            nms_threshold: cli.nms_threshold,
            match_iou_threshold: cli.match_iou_threshold,
            repeats: cli.repeats,
            input: "RGB8 PPM P6",
            box_space: "normalized-to-supplied-PPM",
            preprocess: "YuNet::detect_rgb custom Rust 640x640 RGB-to-BGR 0..255 letterbox",
            preprocess_status: "EXPERIMENTAL_NON_OPENCV_PARITY_UNQUALIFIED",
        },
        model_load_us,
        model_load_cache_state: "single-process-start; filesystem-cache-state-unspecified",
        overall,
        by_stratum,
        by_shoot,
        images,
    };
    write_create_new_json(&cli.out, &receipt)?;
    Ok(())
}

fn parse_args() -> Result<Cli, String> {
    let mut args = std::env::args().skip(1);
    let mut weights = None;
    let mut manifest = None;
    let mut device = None;
    let mut out = None;
    let mut score_threshold = None;
    let mut nms_threshold = None;
    let mut match_iou_threshold = None;
    let mut repeats = 3usize;
    let mut hardware = None;
    let mut source_revision = None;
    while let Some(flag) = args.next() {
        match flag.as_str() {
            "--weights" => weights = Some(next_arg(&mut args, &flag)?),
            "--manifest" => manifest = Some(next_arg(&mut args, &flag)?),
            "--device" => device = Some(next_arg(&mut args, &flag)?),
            "--out" => out = Some(next_arg(&mut args, &flag)?),
            "--score-threshold" => score_threshold = Some(parse_unit(next_arg(&mut args, &flag)?, &flag)?),
            "--nms-threshold" => nms_threshold = Some(parse_unit(next_arg(&mut args, &flag)?, &flag)?),
            "--match-iou-threshold" => match_iou_threshold = Some(parse_unit(next_arg(&mut args, &flag)?, &flag)?),
            "--hardware" => hardware = Some(parse_bounded_text(next_arg(&mut args, &flag)?, &flag)?),
            "--source-revision" => source_revision = Some(parse_bounded_text(next_arg(&mut args, &flag)?, &flag)?),
            "--repeats" => {
                repeats = next_arg(&mut args, &flag)?.parse::<usize>().map_err(|_| "--repeats must be an integer".to_string())?;
                if !(2..=MAX_REPEATS).contains(&repeats) {
                    return Err(format!("--repeats must be 2..{MAX_REPEATS}"));
                }
            }
            "--help" | "-h" => return Err(usage()),
            other => return Err(format!("unknown option {other}\n{}", usage())),
        }
    }
    Ok(Cli {
        weights: PathBuf::from(weights.ok_or_else(|| "--weights is required".to_string())?),
        manifest: PathBuf::from(manifest.ok_or_else(|| "--manifest is required".to_string())?),
        device: device.ok_or_else(|| "--device cpu|metal is required".to_string())?,
        out: PathBuf::from(out.ok_or_else(|| "--out is required".to_string())?),
        score_threshold: score_threshold.ok_or_else(|| "--score-threshold is required".to_string())?,
        nms_threshold: nms_threshold.ok_or_else(|| "--nms-threshold is required".to_string())?,
        match_iou_threshold: match_iou_threshold.ok_or_else(|| "--match-iou-threshold is required".to_string())?,
        repeats,
        hardware: hardware.ok_or_else(|| "--hardware is required".to_string())?,
        source_revision,
    })
}

fn usage() -> String {
    "usage: yunet_qualify --weights PATH --manifest PATH --device cpu|metal --out NEW.json --hardware DESCRIPTOR --score-threshold 0..1 --nms-threshold 0..1 --match-iou-threshold 0..1 [--repeats 2..30] [--source-revision REV]".into()
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

fn parse_bounded_text(value: String, flag: &str) -> Result<String, String> {
    if value.is_empty() || value.len() > MAX_ID_BYTES {
        return Err(format!("{flag} must be 1..{MAX_ID_BYTES} bytes"));
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

fn validate_manifest(manifest: &Manifest) -> Result<(), String> {
    if manifest.schema != MANIFEST_SCHEMA {
        return Err(format!("manifest schema must be {MANIFEST_SCHEMA}"));
    }
    if !manifest.user_consented {
        return Err("manifest userConsented must be true".into());
    }
    if manifest.images.is_empty() || manifest.images.len() > MAX_IMAGES {
        return Err(format!("manifest images must contain 1..{MAX_IMAGES} entries"));
    }
    let mut ids = HashSet::new();
    let mut shoots = BTreeMap::<String, Split>::new();
    let mut ppm_paths = HashSet::new();
    for image in &manifest.images {
        validate_id(&image.id, "image id")?;
        validate_id(&image.shoot_id, "shoot id")?;
        if image.ppm.is_empty() || image.ppm.len() > MAX_ID_BYTES {
            return Err("ppm path is empty or too long".into());
        }
        if !ids.insert(image.id.clone()) {
            return Err("duplicate image id".into());
        }
        if !ppm_paths.insert(image.ppm.clone()) {
            return Err("duplicate ppm path".into());
        }
        if let Some(previous) = shoots.insert(image.shoot_id.clone(), image.split)
            && previous != image.split
        {
            return Err("one shoot is assigned to multiple splits".into());
        }
        if let Some(boxes) = &image.boxes {
            if boxes.len() > 10_000 {
                return Err("too many boxes in one image".into());
            }
            for bbox in boxes {
                validate_box(*bbox)?;
            }
        }
    }
    Ok(())
}

fn validate_id(value: &str, label: &str) -> Result<(), String> {
    if value.is_empty() || value.len() > MAX_ID_BYTES { Err(format!("{label} is empty or too long")) } else { Ok(()) }
}

fn validate_box(bbox: NormalizedBox) -> Result<(), String> {
    let values = [bbox.x0, bbox.y0, bbox.x1, bbox.y1];
    if values.iter().any(|value| !value.is_finite() || !(0.0..=1.0).contains(value)) || bbox.x1 <= bbox.x0 || bbox.y1 <= bbox.y0 {
        Err("ground-truth boxes must be finite normalized rectangles".into())
    } else {
        Ok(())
    }
}

fn resolve_manifest_path(base: &Path, value: &str) -> PathBuf {
    let path = Path::new(value);
    if path.is_absolute() { path.to_path_buf() } else { base.join(path) }
}

fn read_bounded(path: &Path, max_bytes: usize) -> Result<Vec<u8>, std::io::Error> {
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take((max_bytes as u64).saturating_add(1)).read_to_end(&mut bytes)?;
    if bytes.len() > max_bytes {
        return Err(std::io::Error::other(format!("{} exceeds {max_bytes} byte bound", path.display())));
    }
    Ok(bytes)
}

fn load_ppm(path: &Path) -> Result<Ppm, Box<dyn std::error::Error>> {
    let bytes = read_bounded(path, MAX_PPM_BYTES)?;
    parse_ppm(&bytes).map_err(std::io::Error::other).map_err(Into::into)
}

fn parse_ppm(bytes: &[u8]) -> Result<Ppm, String> {
    let mut cursor = 0usize;
    let magic = next_token(bytes, &mut cursor)?;
    if magic != b"P6" {
        return Err("PPM must use binary P6".into());
    }
    let width = parse_positive_usize(next_token(bytes, &mut cursor)?, "width")?;
    let height = parse_positive_usize(next_token(bytes, &mut cursor)?, "height")?;
    let maxval = parse_positive_usize(next_token(bytes, &mut cursor)?, "maxval")?;
    if maxval != 255 {
        return Err("PPM maxval must be 255".into());
    }
    let pixel_count = width.checked_mul(height).ok_or_else(|| "PPM dimensions overflow".to_string())?;
    if pixel_count > MAX_IMAGE_PIXELS {
        return Err(format!("PPM exceeds {MAX_IMAGE_PIXELS} pixel bound"));
    }
    let expected = pixel_count.checked_mul(3).ok_or_else(|| "PPM RGB length overflow".to_string())?;
    let separator = *bytes.get(cursor).ok_or_else(|| "PPM missing raster separator".to_string())?;
    if !separator.is_ascii_whitespace() {
        return Err("PPM header must end with whitespace".into());
    }
    cursor += 1;
    if separator == b'\r' && bytes.get(cursor) == Some(&b'\n') {
        cursor += 1;
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
            *cursor += 1;
        } else if *byte == b'#' {
            while let Some(comment_byte) = bytes.get(*cursor) {
                *cursor += 1;
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
        *cursor += 1;
    }
    if *cursor == start { Err("PPM header token missing".into()) } else { Ok(&bytes[start..*cursor]) }
}

fn parse_positive_usize(token: &[u8], label: &str) -> Result<usize, String> {
    let text = std::str::from_utf8(token).map_err(|_| format!("{label} is not ASCII"))?;
    let value = text.parse::<usize>().map_err(|_| format!("{label} is not a positive integer"))?;
    if value == 0 { Err(format!("{label} must be positive")) } else { Ok(value) }
}

fn detect_repeated(model: &YuNet, ppm: &Ppm, cli: &Cli) -> Result<(Vec<Detection>, TimingReport), Box<dyn std::error::Error>> {
    let mut timings = Vec::with_capacity(cli.repeats);
    let mut reference = None;
    for _ in 0..cli.repeats {
        let start = Instant::now();
        let detections = model.detect_rgb(&ppm.rgb, ppm.width, ppm.height, cli.score_threshold, cli.nms_threshold)?;
        let elapsed = elapsed_us(start);
        if let Some(previous) = &reference {
            if !detections_equivalent(previous, &detections) {
                return Err(std::io::Error::other("repeated YuNet detections changed count/order/content beyond bounded numeric tolerance").into());
            }
        } else {
            reference = Some(detections);
        }
        timings.push(elapsed);
    }
    let first_pass_us = *timings.first().ok_or_else(|| std::io::Error::other("no YuNet timing"))?;
    let warm = &timings[1..];
    let mut sorted = warm.to_vec();
    sorted.sort_unstable();
    let warm_p50_us = percentile(&sorted, 0.50);
    let warm_p95_us = percentile(&sorted, 0.95);
    let detections = reference.ok_or_else(|| std::io::Error::other("no YuNet detections"))?;
    Ok((
        detections,
        TimingReport {
            repetitions: cli.repeats,
            first_pass_us,
            warm_p50_us,
            warm_p95_us,
            warm_sample_count: warm.len(),
            cache_state: "first-per-image-after-model-load; process/filesystem cache-unspecified",
            stage: "detect_rgb_total (preprocess+inference+decode+NMS)",
        },
    ))
}

/// Metal can vary by a few ulps while output count, order, and cardinality must stay identical.
fn detections_equivalent(left: &[Detection], right: &[Detection]) -> bool {
    left.len() == right.len()
        && left.iter().zip(right).all(|(left, right)| {
            approximately_equal(left.score, right.score)
                && left.bbox.iter().zip(right.bbox.iter()).all(|(left, right)| approximately_equal(*left, *right))
                && left
                    .keypoints
                    .iter()
                    .zip(right.keypoints.iter())
                    .all(|(left, right)| approximately_equal(left[0], right[0]) && approximately_equal(left[1], right[1]))
        })
}

fn approximately_equal(left: f32, right: f32) -> bool {
    left.is_finite() && right.is_finite() && (left - right).abs() <= DETECTION_ABS_TOLERANCE + DETECTION_REL_TOLERANCE * left.abs().max(right.abs())
}

fn elapsed_us(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u64::MAX as u128) as u64
}

fn percentile(sorted: &[u64], q: f64) -> u64 {
    if sorted.is_empty() {
        return 0;
    }
    let rank = ((sorted.len() as f64 * q).ceil() as usize).saturating_sub(1).min(sorted.len() - 1);
    sorted[rank]
}

fn match_detections(detections: &[Detection], truth: &[NormalizedBox], width: usize, height: usize, threshold: f32) -> Counts {
    let mut order: Vec<usize> = (0..detections.len()).collect();
    order.sort_by(|left, right| {
        detections[*right]
            .score
            .total_cmp(&detections[*left].score)
            .then_with(|| detections[*left].bbox[0].total_cmp(&detections[*right].bbox[0]))
            .then_with(|| detections[*left].bbox[1].total_cmp(&detections[*right].bbox[1]))
            .then_with(|| detections[*left].bbox[2].total_cmp(&detections[*right].bbox[2]))
            .then_with(|| detections[*left].bbox[3].total_cmp(&detections[*right].bbox[3]))
            .then_with(|| left.cmp(right))
    });
    let mut used = vec![false; truth.len()];
    let mut counts = Counts::default();
    for index in order {
        let detection = &detections[index];
        let detection_box = [
            detection.bbox[0] / width as f32,
            detection.bbox[1] / height as f32,
            detection.bbox[2] / width as f32,
            detection.bbox[3] / height as f32,
        ];
        let mut best = None;
        let mut best_iou = threshold;
        for (truth_index, target) in truth.iter().enumerate() {
            if used[truth_index] {
                continue;
            }
            let target_box = [target.x0, target.y0, target.x1, target.y1];
            let score = iou(&detection_box, &target_box);
            if score >= best_iou {
                if score > best_iou || best.is_none_or(|old| truth_index < old) {
                    best = Some(truth_index);
                    best_iou = score;
                }
            }
        }
        if let Some(truth_index) = best {
            used[truth_index] = true;
            counts.true_positive += 1;
        } else {
            counts.false_positive += 1;
        }
    }
    counts.false_negative = truth.len().saturating_sub(counts.true_positive);
    counts
}

fn iou(left: &[f32; 4], right: &[f32; 4]) -> f32 {
    let x0 = left[0].max(right[0]);
    let y0 = left[1].max(right[1]);
    let x1 = left[2].min(right[2]);
    let y1 = left[3].min(right[3]);
    let intersection = (x1 - x0).max(0.0) * (y1 - y0).max(0.0);
    let left_area = (left[2] - left[0]).max(0.0) * (left[3] - left[1]).max(0.0);
    let right_area = (right[2] - right[0]).max(0.0) * (right[3] - right[1]).max(0.0);
    let union = left_area + right_area - intersection;
    if union <= 0.0 || !union.is_finite() { 0.0 } else { intersection / union }
}

#[derive(Clone, Copy, Debug, Default)]
struct MetricAccumulator {
    images_total: usize,
    labelled_images: usize,
    ground_truth_boxes: usize,
    detections_total: usize,
    metric_eligible_detections: usize,
    counts: Counts,
}

impl MetricAccumulator {
    fn add(&mut self, image: &ImageReport) {
        self.images_total = self.images_total.saturating_add(1);
        self.detections_total = self.detections_total.saturating_add(image.detections);
        if image.labelled {
            self.labelled_images = self.labelled_images.saturating_add(1);
            self.ground_truth_boxes = self.ground_truth_boxes.saturating_add(image.ground_truth_boxes);
            self.metric_eligible_detections = self.metric_eligible_detections.saturating_add(image.detections);
            self.counts.true_positive = self.counts.true_positive.saturating_add(image.true_positive);
            self.counts.false_positive = self.counts.false_positive.saturating_add(image.false_positive);
            self.counts.false_negative = self.counts.false_negative.saturating_add(image.false_negative);
        }
    }

    fn finish(self) -> MetricReport {
        MetricReport {
            images_total: self.images_total,
            labelled_images: self.labelled_images,
            unknown_images_excluded: self.images_total.saturating_sub(self.labelled_images),
            ground_truth_boxes: self.ground_truth_boxes,
            detections_total: self.detections_total,
            metric_eligible_detections: self.metric_eligible_detections,
            true_positive: self.counts.true_positive,
            false_positive: self.counts.false_positive,
            false_negative: self.counts.false_negative,
            precision: ratio(self.counts.true_positive, self.counts.true_positive.saturating_add(self.counts.false_positive)),
            recall: ratio(self.counts.true_positive, self.counts.true_positive.saturating_add(self.counts.false_negative)),
        }
    }
}

fn metric_report(images: &[ImageReport], split: Option<Split>, crop: Option<CropSlice>) -> MetricReport {
    let mut aggregate = MetricAccumulator::default();
    for image in images {
        if split.is_none_or(|value| value == image.split) && crop.is_none_or(|value| value == image.crop_slice) {
            aggregate.add(image);
        }
    }
    aggregate.finish()
}

fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
    (denominator != 0).then_some(numerator as f64 / denominator as f64)
}

fn stratum_reports(images: &[ImageReport]) -> Vec<StratumReport> {
    let mut grouped = BTreeMap::<(Split, CropSlice), MetricAccumulator>::new();
    for image in images {
        grouped.entry((image.split, image.crop_slice)).or_default().add(image);
    }
    grouped.into_iter().map(|((split, crop_slice), aggregate)| StratumReport { split, crop_slice, metrics: aggregate.finish() }).collect()
}

fn shoot_reports(images: &[ImageReport]) -> Vec<ShootReport> {
    let mut grouped = BTreeMap::<(String, Split), MetricAccumulator>::new();
    for image in images {
        grouped.entry((image.shoot_id.clone(), image.split)).or_default().add(image);
    }
    grouped.into_iter().map(|((shoot_id, split), aggregate)| ShootReport { shoot_id, split, metrics: aggregate.finish() }).collect()
}

fn write_create_new_json(path: &Path, receipt: &Receipt) -> Result<(), Box<dyn std::error::Error>> {
    let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
    serde_json::to_writer_pretty(&mut file, receipt)?;
    file.write_all(b"\n")?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ppm_parser_is_bounded_and_binary() {
        let bytes = b"P6\n# comment\n2 1\n255\n\x00\x01\x02\x03\x04\x05";
        let parsed = parse_ppm(bytes);
        assert!(parsed.is_ok());
        if let Ok(ppm) = parsed {
            assert_eq!((ppm.width, ppm.height), (2, 1));
            assert_eq!(ppm.rgb, vec![0, 1, 2, 3, 4, 5]);
        }
    }

    #[test]
    fn matching_is_confidence_greedy_and_one_to_one() {
        let detections = vec![
            Detection { score: 0.9, bbox: [0.0, 0.0, 10.0, 10.0], keypoints: [[0.0; 2]; 5] },
            Detection { score: 0.8, bbox: [0.0, 0.0, 10.0, 10.0], keypoints: [[0.0; 2]; 5] },
        ];
        let truth = vec![NormalizedBox { x0: 0.0, y0: 0.0, x1: 1.0, y1: 1.0 }];
        let result = match_detections(&detections, &truth, 10, 10, 0.5);
        assert_eq!((result.true_positive, result.false_positive, result.false_negative), (1, 1, 0));
    }
}
