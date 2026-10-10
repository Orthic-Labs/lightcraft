//! Offline DINOv2 similarity qualification for an explicit, consented P6 manifest.
//!
//! This example is a bounded measurement harness. It never reads a catalog, changes culling
//! state, promotes a model, or writes input paths, EXIF, or pixels to its receipt.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(not(target_arch = "wasm32"))]
mod native {

    use std::collections::{BTreeMap, HashMap, HashSet};
    use std::fs::{File, OpenOptions};
    use std::io::{Read, Write};
    use std::path::{Path, PathBuf};
    use std::time::Instant;

    use candle_core::{Device, Tensor};
    use lightcraft_fetch::sha256_bytes;
    use lightcraft_segment::dinov2::DinoV2;
    use lightcraft_segment::dinov2_artifact::{MODEL_BYTES, MODEL_SHA256, load_file};
    use lightcraft_segment::dinov2_input::{DinoInputSize, preprocess_rgb8};
    use serde::{Deserialize, Serialize};

    const MANIFEST_SCHEMA: &str = "lightcraft.dinov2-qualification.v1";
    const REFERENCE_SCHEMA: &str = "lightcraft.dinov2-reference-embeddings.v1";
    const RECEIPT_SCHEMA: &str = "lightcraft.dinov2-qualification-receipt.v1";
    const MAX_MANIFEST_BYTES: usize = 16 * 1024 * 1024;
    const MAX_REFERENCE_BYTES: usize = 64 * 1024 * 1024;
    const MAX_PPM_BYTES: usize = 100 * 1024 * 1024;
    const MAX_IMAGE_PIXELS: usize = 32_000_000;
    const MAX_IMAGE_SIDE: usize = 16_384;
    const MAX_IMAGES: usize = 10_000;
    const MAX_PAIRS: usize = 50_000;
    const MAX_ID_BYTES: usize = 96;
    const MAX_TEXT_BYTES: usize = 512;
    const MAX_PATH_BYTES: usize = 512;
    const MAX_RECEIPT_BYTES: usize = 64 * 1024 * 1024;
    const MAX_REPEATS: usize = 30;
    const EMBEDDING_DIMS: usize = 384;
    const REPEAT_MAX_ABS_TOLERANCE: f64 = 1e-3;
    const REPEAT_MIN_COSINE: f32 = 0.999;
    const SYNTHETIC_CROSS_SHOOT: &str = "cross-shoot";

    #[derive(Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct Manifest {
        schema: String,
        user_consented: bool,
        images: Vec<ManifestImage>,
        pairs: Vec<ManifestPair>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ManifestImage {
        id: String,
        shoot_id: String,
        split: Split,
        ppm: String,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ManifestPair {
        id: String,
        left_id: String,
        right_id: String,
        label: PairLabel,
    }

    #[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
    #[serde(rename_all = "lowercase")]
    enum Split {
        Train,
        Validation,
        Test,
    }

    #[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
    #[serde(rename_all = "lowercase")]
    enum PairLabel {
        Similar,
        Different,
        Unknown,
    }

    #[derive(Clone, Debug)]
    struct Ppm {
        width: usize,
        height: usize,
        rgb: Vec<u8>,
        ppm_sha256: String,
    }

    #[derive(Clone, Debug)]
    struct Cli {
        weights: PathBuf,
        manifest: PathBuf,
        device: String,
        out: PathBuf,
        threshold: f32,
        size: DinoInputSize,
        repeats: usize,
        hardware: String,
        source_revision: Option<String>,
        reference_embeddings: Option<PathBuf>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceFile {
        schema: String,
        label_provenance: String,
        embeddings: Vec<ReferenceEmbedding>,
    }

    #[derive(Clone, Debug, Deserialize)]
    #[serde(deny_unknown_fields)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceEmbedding {
        id: String,
        values: Vec<f32>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct TimingReport {
        first_embedding_us: u64,
        repeat_p50_us: u64,
        repeat_p95_us: u64,
        repeat_sample_count: usize,
        repeat_max_abs_error: f64,
        repeat_min_cosine: f32,
        cache_state: &'static str,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ImageReport {
        id: String,
        shoot_id: String,
        split: Split,
        width: usize,
        height: usize,
        input_ppm_sha256: String,
        input_rgb_sha256: String,
        timing: TimingReport,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct PairReport {
        id: String,
        left_id: String,
        right_id: String,
        left_shoot_id: String,
        right_shoot_id: String,
        split: Split,
        label: PairLabel,
        embedding_cosine: f32,
        eligible: bool,
        predicted_similar: Option<bool>,
    }

    #[derive(Clone, Debug, Default, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Metrics {
        all_pairs: usize,
        eligible_pairs: usize,
        unknown_pairs: usize,
        true_positive: usize,
        false_positive: usize,
        true_negative: usize,
        false_negative: usize,
        coverage: Option<f64>,
        precision: Option<f64>,
        recall: Option<f64>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct SplitReport {
        split: Split,
        metrics: Metrics,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ShootReport {
        shoot_id: String,
        split: Split,
        metrics: Metrics,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ReferenceReport {
        provided: bool,
        label_provenance: Option<String>,
        validation: &'static str,
        supplied_embeddings: usize,
        matched_embeddings: usize,
        max_abs_error: Option<f64>,
        rmse: Option<f64>,
        reference_vector_cosine_min: Option<f64>,
        reference_vector_cosine_mean: Option<f64>,
        total_embeddings: usize,
        coverage: Option<f64>,
    }

    #[derive(Clone, Debug, Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Receipt {
        schema: &'static str,
        qualification: &'static str,
        model_sha256: &'static str,
        model_bytes: u64,
        device: String,
        device_debug: String,
        hardware: String,
        source_revision: Option<String>,
        manifest_schema: String,
        manifest_sha256: String,
        input_size: &'static str,
        input_side: usize,
        resize_short_edge: usize,
        threshold: f32,
        threshold_provenance: &'static str,
        repeats: usize,
        preprocessing: &'static str,
        model_load_us: u64,
        model_load_cache_state: &'static str,
        overall: Metrics,
        by_split: Vec<SplitReport>,
        by_shoot: Vec<ShootReport>,
        images: Vec<ImageReport>,
        pairs: Vec<PairReport>,
        reference: ReferenceReport,
        reference_file_sha256: Option<String>,
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
        let manifest_bytes = read_bounded(&cli.manifest, MAX_MANIFEST_BYTES)?;
        let manifest: Manifest = serde_json::from_slice(&manifest_bytes)?;
        validate_manifest(&manifest)?;
        let manifest_sha256 = sha256_bytes(&manifest_bytes);
        let image_ids = manifest.images.iter().map(|image| image.id.clone()).collect::<HashSet<_>>();
        let reference = cli.reference_embeddings.as_deref().map(|path| load_reference(path, &image_ids)).transpose()?;
        let base = cli.manifest.parent().unwrap_or_else(|| Path::new("."));
        let (device, device_name) = select_device(&cli.device).map_err(std::io::Error::other)?;

        let load_start = Instant::now();
        let model = load_file(&cli.weights, &device)?;
        let model_load_us = elapsed_us(load_start);

        let mut embeddings = HashMap::with_capacity(manifest.images.len());
        let mut image_reports = Vec::with_capacity(manifest.images.len());
        let mut image_index = HashMap::with_capacity(manifest.images.len());
        let mut pixel_splits = HashMap::<String, Split>::with_capacity(manifest.images.len());
        for image in &manifest.images {
            let path = resolve_manifest_path(base, &image.ppm);
            let ppm = load_ppm(&path)?;
            let input_rgb_sha256 = sha256_bytes(&ppm.rgb);
            record_pixel_split(&mut pixel_splits, input_rgb_sha256.clone(), image.split).map_err(std::io::Error::other)?;
            let (embedding, timing) = embed_repeated(&model, &ppm, &cli, &device)?;
            embeddings.insert(image.id.clone(), embedding);
            image_index.insert(image.id.clone(), image);
            image_reports.push(ImageReport {
                id: image.id.clone(),
                shoot_id: image.shoot_id.clone(),
                split: image.split,
                width: ppm.width,
                height: ppm.height,
                input_ppm_sha256: ppm.ppm_sha256.clone(),
                input_rgb_sha256,
                timing,
            });
        }

        let mut pair_reports = Vec::with_capacity(manifest.pairs.len());
        for pair in &manifest.pairs {
            let left = embeddings.get(&pair.left_id).ok_or_else(|| std::io::Error::other("pair left embedding missing"))?;
            let right = embeddings.get(&pair.right_id).ok_or_else(|| std::io::Error::other("pair right embedding missing"))?;
            let left_image = image_index.get(&pair.left_id).ok_or_else(|| std::io::Error::other("pair left image missing"))?;
            let right_image = image_index.get(&pair.right_id).ok_or_else(|| std::io::Error::other("pair right image missing"))?;
            let embedding_cosine = DinoV2::cosine(left, right)?;
            let eligible = !matches!(pair.label, PairLabel::Unknown);
            pair_reports.push(PairReport {
                id: pair.id.clone(),
                left_id: pair.left_id.clone(),
                right_id: pair.right_id.clone(),
                left_shoot_id: left_image.shoot_id.clone(),
                right_shoot_id: right_image.shoot_id.clone(),
                split: pair_split(left_image, right_image),
                label: pair.label,
                embedding_cosine,
                eligible,
                predicted_similar: eligible.then_some(embedding_cosine >= cli.threshold),
            });
        }

        let reference_report = reference_report(reference.as_ref().map(|loaded| &loaded.file), &embeddings)?;
        let overall = metrics(&pair_reports, None, None);
        let by_split = split_reports(&pair_reports);
        let by_shoot = shoot_reports(&pair_reports);
        let receipt = Receipt {
            schema: RECEIPT_SCHEMA,
            qualification: "UNQUALIFIED",
            model_sha256: MODEL_SHA256,
            model_bytes: MODEL_BYTES as u64,
            device: device_name,
            device_debug: format!("{device:?}"),
            hardware: cli.hardware,
            source_revision: cli.source_revision,
            manifest_schema: manifest.schema,
            manifest_sha256,
            input_size: size_name(cli.size),
            input_side: cli.size.side(),
            resize_short_edge: cli.size.resize_short_edge(),
            threshold: cli.threshold,
            threshold_provenance: "externally supplied; frozen status unverified",
            repeats: cli.repeats,
            preprocessing: "dinov2_input-v1; RGB8 P6; bicubic A=-0.5; half-pixel; resize short edge; center crop; round/clamp RGB8; ImageNet normalization",
            model_load_us,
            model_load_cache_state: "single-process-start; filesystem-cache-state-unspecified",
            overall,
            by_split,
            by_shoot,
            images: image_reports,
            pairs: pair_reports,
            reference: reference_report,
            reference_file_sha256: reference.as_ref().map(|loaded| loaded.sha256.clone()),
        };
        write_create_new_json(&cli.out, &receipt)?;
        Ok(())
    }

    fn record_pixel_split(seen: &mut HashMap<String, Split>, digest: String, split: Split) -> Result<(), String> {
        if let Some(previous) = seen.insert(digest, split)
            && previous != split
        {
            return Err("identical RGB input pixels assigned to different splits".into());
        }
        Ok(())
    }

    fn parse_args() -> Result<Cli, String> {
        let mut args = std::env::args().skip(1);
        let mut weights = None;
        let mut manifest = None;
        let mut device = None;
        let mut out = None;
        let mut threshold = None;
        let mut size = DinoInputSize::Small224;
        let mut repeats = 3usize;
        let mut hardware = None;
        let mut source_revision = None;
        let mut reference_embeddings = None;
        while let Some(flag) = args.next() {
            match flag.as_str() {
                "--weights" => weights = Some(next_arg(&mut args, &flag)?),
                "--manifest" => manifest = Some(next_arg(&mut args, &flag)?),
                "--device" => device = Some(next_arg(&mut args, &flag)?),
                "--out" => out = Some(next_arg(&mut args, &flag)?),
                "--threshold" => threshold = Some(parse_threshold(next_arg(&mut args, &flag)?, &flag)?),
                "--size" => size = parse_size(&next_arg(&mut args, &flag)?)?,
                "--repeats" => {
                    repeats = next_arg(&mut args, &flag)?.parse::<usize>().map_err(|_| "--repeats must be an integer".to_string())?;
                    if !(2..=MAX_REPEATS).contains(&repeats) {
                        return Err(format!("--repeats must be 2..{MAX_REPEATS}"));
                    }
                }
                "--hardware" => hardware = Some(parse_bounded_text(next_arg(&mut args, &flag)?, &flag, MAX_TEXT_BYTES)?),
                "--source-revision" => source_revision = Some(parse_bounded_text(next_arg(&mut args, &flag)?, &flag, MAX_TEXT_BYTES)?),
                "--reference-embeddings" => reference_embeddings = Some(PathBuf::from(next_arg(&mut args, &flag)?)),
                "--help" | "-h" => return Err(usage()),
                other => return Err(format!("unknown option {other}\n{}", usage())),
            }
        }
        Ok(Cli {
            weights: PathBuf::from(weights.ok_or_else(|| "--weights is required".to_string())?),
            manifest: PathBuf::from(manifest.ok_or_else(|| "--manifest is required".to_string())?),
            device: device.ok_or_else(|| "--device cpu|metal is required".to_string())?,
            out: PathBuf::from(out.ok_or_else(|| "--out is required".to_string())?),
            threshold: threshold.ok_or_else(|| "--threshold is required".to_string())?,
            size,
            repeats,
            hardware: hardware.ok_or_else(|| "--hardware is required".to_string())?,
            source_revision,
            reference_embeddings,
        })
    }

    fn usage() -> String {
        "usage: dinov2_qualify --weights FILE --manifest FILE --device cpu|metal --out FILE --threshold -1..1 --hardware TEXT [--size small224|large518] [--repeats 2..30] [--source-revision TEXT] [--reference-embeddings FILE]".into()
    }

    fn next_arg(args: &mut impl Iterator<Item = String>, flag: &str) -> Result<String, String> {
        args.next().ok_or_else(|| format!("{flag} requires a value"))
    }

    fn parse_threshold(value: String, flag: &str) -> Result<f32, String> {
        let parsed = value.parse::<f32>().map_err(|_| format!("{flag} must be finite and in [-1,1]"))?;
        if !parsed.is_finite() || !(-1.0..=1.0).contains(&parsed) {
            return Err(format!("{flag} must be finite and in [-1,1]"));
        }
        Ok(parsed)
    }

    fn parse_size(value: &str) -> Result<DinoInputSize, String> {
        match value {
            "small224" => Ok(DinoInputSize::Small224),
            "large518" => Ok(DinoInputSize::Large518),
            _ => Err("--size must be small224 or large518".into()),
        }
    }

    fn size_name(size: DinoInputSize) -> &'static str {
        match size {
            DinoInputSize::Small224 => "small224",
            DinoInputSize::Large518 => "large518",
        }
    }

    fn parse_bounded_text(value: String, flag: &str, max_bytes: usize) -> Result<String, String> {
        if value.is_empty() || value.len() > max_bytes {
            return Err(format!("{flag} must be 1..{max_bytes} bytes"));
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
        if manifest.pairs.is_empty() || manifest.pairs.len() > MAX_PAIRS {
            return Err(format!("manifest pairs must contain 1..{MAX_PAIRS} entries"));
        }
        let mut image_ids = HashSet::new();
        let mut image_splits = HashMap::with_capacity(manifest.images.len());
        let mut shoots = BTreeMap::<String, Split>::new();
        for image in &manifest.images {
            validate_id(&image.id, "image id")?;
            validate_id(&image.shoot_id, "shoot id")?;
            if image.shoot_id == SYNTHETIC_CROSS_SHOOT {
                return Err(format!("shoot id {SYNTHETIC_CROSS_SHOOT:?} is reserved"));
            }
            if image.ppm.is_empty() || image.ppm.len() > MAX_PATH_BYTES {
                return Err("ppm reference is empty or too long".into());
            }
            if !image_ids.insert(image.id.clone()) {
                return Err("duplicate image id".into());
            }
            image_splits.insert(image.id.clone(), image.split);
            if let Some(previous) = shoots.insert(image.shoot_id.clone(), image.split)
                && previous != image.split
            {
                return Err("one shoot is assigned to multiple splits".into());
            }
        }
        let mut pair_ids = HashSet::new();
        let mut pair_endpoints = HashSet::new();
        for pair in &manifest.pairs {
            validate_id(&pair.id, "pair id")?;
            if !pair_ids.insert(pair.id.clone()) {
                return Err("duplicate pair id".into());
            }
            if pair.left_id == pair.right_id {
                return Err("pair leftId and rightId must differ".into());
            }
            let endpoints = if pair.left_id < pair.right_id {
                (pair.left_id.clone(), pair.right_id.clone())
            } else {
                (pair.right_id.clone(), pair.left_id.clone())
            };
            if !pair_endpoints.insert(endpoints) {
                return Err("duplicate or reversed pair endpoints".into());
            }
            let left_split = image_splits.get(&pair.left_id).ok_or_else(|| "pair leftId is unknown".to_string())?;
            let right_split = image_splits.get(&pair.right_id).ok_or_else(|| "pair rightId is unknown".to_string())?;
            if left_split != right_split {
                return Err("pair references must be in the same split".into());
            }
        }
        Ok(())
    }

    fn validate_id(value: &str, label: &str) -> Result<(), String> {
        if value.is_empty()
            || value.len() > MAX_ID_BYTES
            || !value.is_ascii()
            || value.bytes().any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\')
        {
            Err(format!("{label} must be 1..{MAX_ID_BYTES} ASCII bytes"))
        } else {
            Ok(())
        }
    }

    fn load_reference(path: &Path, image_ids: &HashSet<String>) -> Result<LoadedReference, Box<dyn std::error::Error>> {
        let bytes = read_bounded(path, MAX_REFERENCE_BYTES)?;
        let reference: ReferenceFile = serde_json::from_slice(&bytes)?;
        if reference.schema != REFERENCE_SCHEMA {
            return Err(std::io::Error::other(format!("reference schema must be {REFERENCE_SCHEMA}")).into());
        }
        if reference.label_provenance.is_empty()
            || reference.label_provenance.len() > MAX_TEXT_BYTES
            || !reference.label_provenance.is_ascii()
            || reference.label_provenance.bytes().any(|byte| byte.is_ascii_control() || byte == b'/' || byte == b'\\')
        {
            return Err(std::io::Error::other("reference labelProvenance is empty or too long").into());
        }
        validate_reference(&reference, image_ids).map_err(std::io::Error::other)?;
        Ok(LoadedReference { file: reference, sha256: sha256_bytes(&bytes) })
    }

    fn validate_reference(reference: &ReferenceFile, image_ids: &HashSet<String>) -> Result<(), String> {
        if reference.embeddings.is_empty() || reference.embeddings.len() > MAX_IMAGES {
            return Err(format!("reference embeddings must contain 1..{MAX_IMAGES} entries"));
        }
        let mut ids = HashSet::new();
        for item in &reference.embeddings {
            validate_id(&item.id, "reference embedding id")?;
            if !image_ids.contains(&item.id) {
                return Err("reference embedding id is unknown".into());
            }
            if !ids.insert(item.id.clone()) {
                return Err("duplicate reference embedding id".into());
            }
            if item.values.len() != EMBEDDING_DIMS || item.values.iter().any(|value| !value.is_finite()) {
                return Err("reference embeddings must contain 384 finite values".into());
            }
        }
        Ok(())
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
            return Err(std::io::Error::other(format!("input exceeds {max_bytes} byte bound")));
        }
        Ok(bytes)
    }

    fn load_ppm(path: &Path) -> Result<Ppm, Box<dyn std::error::Error>> {
        let bytes = read_bounded(path, MAX_PPM_BYTES)?;
        let mut ppm = parse_ppm(&bytes).map_err(std::io::Error::other)?;
        ppm.ppm_sha256 = sha256_bytes(&bytes);
        Ok(ppm)
    }

    fn parse_ppm(bytes: &[u8]) -> Result<Ppm, String> {
        let mut cursor = 0usize;
        if next_token(bytes, &mut cursor)? != b"P6" {
            return Err("PPM must use binary P6".into());
        }
        let width = parse_positive_usize(next_token(bytes, &mut cursor)?, "width")?;
        let height = parse_positive_usize(next_token(bytes, &mut cursor)?, "height")?;
        let maxval = parse_positive_usize(next_token(bytes, &mut cursor)?, "maxval")?;
        if maxval != 255 {
            return Err("PPM maxval must be 255".into());
        }
        if width > MAX_IMAGE_SIDE || height > MAX_IMAGE_SIDE {
            return Err(format!("PPM dimensions exceed {MAX_IMAGE_SIDE}px bound"));
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
        cursor += 1;
        // CRLF is header whitespace only when skipping LF still leaves the exact raster length.
        // If LF is the first pixel, the exact-length check keeps it in the raster.
        if separator == b'\r' && bytes.get(cursor) == Some(&b'\n') && bytes.len().saturating_sub(cursor) > expected {
            cursor += 1;
        }
        let raster = bytes.get(cursor..).ok_or_else(|| "PPM raster offset out of range".to_string())?;
        if raster.len() != expected {
            return Err(format!("PPM raster has {}, expected {expected} bytes", raster.len()));
        }
        Ok(Ppm { width, height, rgb: raster.to_vec(), ppm_sha256: String::new() })
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

    fn embed_repeated(model: &DinoV2, ppm: &Ppm, cli: &Cli, device: &Device) -> Result<(Tensor, TimingReport), Box<dyn std::error::Error>> {
        let mut timings = Vec::with_capacity(cli.repeats);
        let mut first = None;
        let mut repeat_max_abs_error = 0.0f64;
        let mut repeat_min_cosine = 1.0f32;
        for _ in 0..cli.repeats {
            let start = Instant::now();
            let pixels = preprocess_rgb8(&ppm.rgb, ppm.width, ppm.height, cli.size, device)?;
            let embedding = model.forward(&pixels)?.cls_token;
            let elapsed = elapsed_us(start);
            if let Some(reference) = first.as_ref() {
                let max_abs_error = embedding_max_abs_error(reference, &embedding)?;
                let cosine = DinoV2::cosine(reference, &embedding)?;
                repeat_max_abs_error = repeat_max_abs_error.max(max_abs_error);
                repeat_min_cosine = repeat_min_cosine.min(cosine);
                if max_abs_error > REPEAT_MAX_ABS_TOLERANCE || cosine < REPEAT_MIN_COSINE {
                    return Err(std::io::Error::other("repeated DINOv2 embeddings exceeded agreement tolerance").into());
                }
            } else {
                first = Some(embedding);
            }
            timings.push(elapsed);
        }
        let embedding = first.ok_or_else(|| std::io::Error::other("no DINOv2 embedding"))?;
        let warm = timings.get(1..).ok_or_else(|| std::io::Error::other("no DINOv2 repeat timing"))?;
        let mut sorted = warm.to_vec();
        sorted.sort_unstable();
        Ok((
            embedding,
            TimingReport {
                first_embedding_us: *timings.first().ok_or_else(|| std::io::Error::other("no DINOv2 timing"))?,
                repeat_p50_us: percentile(&sorted, 0.50),
                repeat_p95_us: percentile(&sorted, 0.95),
                repeat_sample_count: warm.len(),
                repeat_max_abs_error,
                repeat_min_cosine,
                cache_state: "model-cached; repeats recompute preprocess+forward; embedding cached after repeat set; filesystem-cache-state-unspecified",
            },
        ))
    }

    fn embedding_max_abs_error(left: &Tensor, right: &Tensor) -> Result<f64, Box<dyn std::error::Error>> {
        let left_values = left.to_vec2::<f32>()?.into_iter().next().ok_or_else(|| std::io::Error::other("DINOv2 first embedding row missing"))?;
        let right_values =
            right.to_vec2::<f32>()?.into_iter().next().ok_or_else(|| std::io::Error::other("DINOv2 repeated embedding row missing"))?;
        if left_values.len() != EMBEDDING_DIMS || right_values.len() != EMBEDDING_DIMS {
            return Err(std::io::Error::other("DINOv2 repeated embedding shape mismatch").into());
        }
        Ok(left_values.iter().zip(right_values).map(|(left, right)| f64::from((*left - right).abs())).fold(0.0, f64::max))
    }

    fn pair_split(left: &ManifestImage, right: &ManifestImage) -> Split {
        let _ = right;
        left.split
    }

    fn percentile(sorted: &[u64], q: f64) -> u64 {
        if sorted.is_empty() {
            return 0;
        }
        let rank = ((sorted.len() as f64 * q).ceil() as usize).saturating_sub(1).min(sorted.len() - 1);
        sorted[rank]
    }

    fn elapsed_us(start: Instant) -> u64 {
        start.elapsed().as_micros().min(u64::MAX as u128) as u64
    }

    fn metrics(pairs: &[PairReport], split: Option<Split>, shoot: Option<&str>) -> Metrics {
        let mut result = Metrics::default();
        for pair in pairs {
            if split.is_some_and(|value| value != pair.split) {
                continue;
            }
            if let Some(expected) = shoot {
                let pair_shoot = if pair.left_shoot_id == pair.right_shoot_id { pair.left_shoot_id.as_str() } else { SYNTHETIC_CROSS_SHOOT };
                if pair_shoot != expected {
                    continue;
                }
            }
            add_pair(&mut result, pair);
        }
        finish_metrics(result)
    }

    fn add_pair(result: &mut Metrics, pair: &PairReport) {
        result.all_pairs = result.all_pairs.saturating_add(1);
        if !pair.eligible {
            result.unknown_pairs = result.unknown_pairs.saturating_add(1);
            return;
        }
        result.eligible_pairs = result.eligible_pairs.saturating_add(1);
        let predicted = pair.predicted_similar.unwrap_or(false);
        match (pair.label, predicted) {
            (PairLabel::Similar, true) => result.true_positive = result.true_positive.saturating_add(1),
            (PairLabel::Similar, false) => result.false_negative = result.false_negative.saturating_add(1),
            (PairLabel::Different, true) => result.false_positive = result.false_positive.saturating_add(1),
            (PairLabel::Different, false) => result.true_negative = result.true_negative.saturating_add(1),
            (PairLabel::Unknown, _) => {}
        }
    }

    fn finish_metrics(mut result: Metrics) -> Metrics {
        result.coverage = ratio(result.eligible_pairs, result.all_pairs);
        result.precision = ratio(result.true_positive, result.true_positive.saturating_add(result.false_positive));
        result.recall = ratio(result.true_positive, result.true_positive.saturating_add(result.false_negative));
        result
    }

    fn split_reports(pairs: &[PairReport]) -> Vec<SplitReport> {
        [Split::Train, Split::Validation, Split::Test]
            .into_iter()
            .map(|split| SplitReport { split, metrics: metrics(pairs, Some(split), None) })
            .collect()
    }

    fn shoot_reports(pairs: &[PairReport]) -> Vec<ShootReport> {
        let mut grouped = BTreeMap::<(String, Split), Metrics>::new();
        for pair in pairs {
            let shoot = if pair.left_shoot_id == pair.right_shoot_id { pair.left_shoot_id.clone() } else { SYNTHETIC_CROSS_SHOOT.into() };
            add_pair(grouped.entry((shoot, pair.split)).or_default(), pair);
        }
        grouped.into_iter().map(|((shoot_id, split), values)| ShootReport { shoot_id, split, metrics: finish_metrics(values) }).collect()
    }

    fn reference_report(
        reference: Option<&ReferenceFile>,
        embeddings: &HashMap<String, Tensor>,
    ) -> Result<ReferenceReport, Box<dyn std::error::Error>> {
        let Some(reference) = reference else {
            return Ok(ReferenceReport {
                provided: false,
                label_provenance: None,
                validation: "UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY",
                supplied_embeddings: 0,
                matched_embeddings: 0,
                max_abs_error: None,
                rmse: None,
                reference_vector_cosine_min: None,
                reference_vector_cosine_mean: None,
                total_embeddings: embeddings.len(),
                coverage: None,
            });
        };
        let mut count = 0usize;
        let mut max_abs = 0.0f64;
        let mut sum_sq = 0.0f64;
        let mut value_count = 0usize;
        let mut cosine_sum = 0.0f64;
        let mut cosine_min = f64::INFINITY;
        for expected in &reference.embeddings {
            let Some(actual) = embeddings.get(&expected.id) else { continue };
            let values = actual.to_vec2::<f32>()?.into_iter().next().ok_or_else(|| std::io::Error::other("DINOv2 embedding row missing"))?;
            let cosine = vector_cosine(&values, &expected.values)? as f64;
            cosine_sum += cosine;
            cosine_min = cosine_min.min(cosine);
            for (&left, &right) in values.iter().zip(&expected.values) {
                let difference = (left as f64 - right as f64).abs();
                max_abs = max_abs.max(difference);
                sum_sq += difference * difference;
                value_count = value_count.saturating_add(1);
            }
            count = count.saturating_add(1);
        }
        Ok(ReferenceReport {
            provided: true,
            label_provenance: Some(reference.label_provenance.clone()),
            validation: "UNVERIFIED_NOT_VALIDATED_AUTOMATICALLY",
            supplied_embeddings: reference.embeddings.len(),
            matched_embeddings: count,
            max_abs_error: (count != 0).then_some(max_abs),
            rmse: (value_count != 0).then_some((sum_sq / value_count as f64).sqrt()),
            reference_vector_cosine_min: (count != 0).then_some(cosine_min),
            reference_vector_cosine_mean: (count != 0).then_some(cosine_sum / count as f64),
            total_embeddings: embeddings.len(),
            coverage: ratio(count, embeddings.len()),
        })
    }

    fn vector_cosine(left: &[f32], right: &[f32]) -> Result<f32, std::io::Error> {
        if left.len() != EMBEDDING_DIMS || right.len() != EMBEDDING_DIMS {
            return Err(std::io::Error::other("reference cosine expects 384 values"));
        }
        let (mut dot, mut aa, mut bb) = (0.0f64, 0.0f64, 0.0f64);
        for (&x, &y) in left.iter().zip(right) {
            let (x, y) = (x as f64, y as f64);
            dot += x * y;
            aa += x * x;
            bb += y * y;
        }
        if aa <= f64::EPSILON || bb <= f64::EPSILON {
            return Err(std::io::Error::other("reference cosine received zero embedding"));
        }
        Ok((dot / (aa.sqrt() * bb.sqrt())).clamp(-1.0, 1.0) as f32)
    }

    fn ratio(numerator: usize, denominator: usize) -> Option<f64> {
        (denominator != 0).then_some(numerator as f64 / denominator as f64)
    }

    fn write_create_new_json(path: &Path, receipt: &Receipt) -> Result<(), Box<dyn std::error::Error>> {
        let bytes = serde_json::to_vec_pretty(receipt)?;
        if bytes.len() > MAX_RECEIPT_BYTES {
            return Err(std::io::Error::other(format!("receipt exceeds {MAX_RECEIPT_BYTES} byte bound")).into());
        }
        let mut file = OpenOptions::new().write(true).create_new(true).open(path)?;
        file.write_all(&bytes)?;
        file.write_all(b"\n")?;
        Ok(())
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        fn image(id: &str, shoot_id: &str, split: Split) -> ManifestImage {
            ManifestImage { id: id.into(), shoot_id: shoot_id.into(), split, ppm: format!("{id}.ppm") }
        }

        fn pair(id: &str, left_id: &str, right_id: &str) -> ManifestPair {
            ManifestPair { id: id.into(), left_id: left_id.into(), right_id: right_id.into(), label: PairLabel::Similar }
        }

        #[test]
        fn ppm_parser_is_bounded_binary_p6() {
            let parsed = parse_ppm(b"P6\n2 1\n255\n\x00\x01\x02\x03\x04\x05");
            assert!(parsed.is_ok());
            if let Ok(ppm) = parsed {
                assert_eq!((ppm.width, ppm.height), (2, 1));
                assert_eq!(ppm.rgb.len(), 6);
            }
        }

        #[test]
        fn threshold_is_fixed_and_bounded() {
            assert!(parse_threshold("-1".into(), "--threshold").is_ok());
            assert!(parse_threshold("1".into(), "--threshold").is_ok());
            assert!(parse_threshold("1.01".into(), "--threshold").is_err());
            assert!(parse_threshold("NaN".into(), "--threshold").is_err());
        }

        #[test]
        fn duplicate_or_reversed_pair_endpoints_are_rejected() {
            let manifest = Manifest {
                schema: MANIFEST_SCHEMA.into(),
                user_consented: true,
                images: vec![image("a", "shoot", Split::Train), image("b", "shoot", Split::Train)],
                pairs: vec![pair("p1", "a", "b"), pair("p2", "b", "a")],
            };
            assert!(validate_manifest(&manifest).is_err());
        }

        #[test]
        fn reference_list_requires_known_nonempty_ids() {
            let mut ids = HashSet::new();
            ids.insert("a".into());
            let empty = ReferenceFile { schema: REFERENCE_SCHEMA.into(), label_provenance: "supplied".into(), embeddings: Vec::new() };
            assert!(validate_reference(&empty, &ids).is_err());
            let unknown = ReferenceFile {
                schema: REFERENCE_SCHEMA.into(),
                label_provenance: "supplied".into(),
                embeddings: vec![ReferenceEmbedding { id: "missing".into(), values: vec![0.0; EMBEDDING_DIMS] }],
            };
            assert!(validate_reference(&unknown, &ids).is_err());
        }

        #[test]
        fn shoot_split_leakage_is_rejected() {
            let manifest = Manifest {
                schema: MANIFEST_SCHEMA.into(),
                user_consented: true,
                images: vec![image("a", "same-shoot", Split::Train), image("b", "same-shoot", Split::Test)],
                pairs: vec![pair("p", "a", "b")],
            };
            assert!(validate_manifest(&manifest).is_err());
        }

        #[test]
        fn synthetic_cross_shoot_id_is_reserved() {
            let manifest = Manifest {
                schema: MANIFEST_SCHEMA.into(),
                user_consented: true,
                images: vec![image("a", SYNTHETIC_CROSS_SHOOT, Split::Train), image("b", "other", Split::Train)],
                pairs: vec![pair("p", "a", "b")],
            };
            assert!(validate_manifest(&manifest).is_err());
        }

        #[test]
        fn identical_rgb_digest_cannot_cross_splits() {
            let mut seen = HashMap::new();
            assert!(record_pixel_split(&mut seen, "same-rgb".into(), Split::Train).is_ok());
            assert!(record_pixel_split(&mut seen, "same-rgb".into(), Split::Train).is_ok());
            assert!(record_pixel_split(&mut seen, "same-rgb".into(), Split::Test).is_err());
        }

        #[test]
        fn ppm_raster_prefix_and_suffix_are_rejected() {
            let mut bytes = b"P6\n1 1\n255\n".to_vec();
            bytes.extend_from_slice(&[b'\n', 0, 1]);
            assert!(parse_ppm(&bytes).is_ok());
            bytes.push(2);
            assert!(parse_ppm(&bytes).is_err());
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    native::run()
}

#[cfg(target_arch = "wasm32")]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    Err("dinov2_qualify is unsupported on wasm32; run native cpu or metal qualification".into())
}
