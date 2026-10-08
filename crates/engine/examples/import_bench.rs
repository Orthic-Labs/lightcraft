//! Import-review benchmark for 1,300 mixed procedural JPEG/DNG files.
//!
//! Run through generated CI actions on local SSD and mounted NAS hosts. Output records stage
//! counters/timers; this source tree claims no SSD/NAS numbers until those actions execute.
//! Fixtures are generated in a temporary directory, use production `fs_hooks`/`probe_bytes`, and
//! are removed before exit. `COUNT`, `JPEG_EDGE`, and `RAW_EDGE` can bound action runtime.

use lightcraft_catalog::MediaKind;
use lightcraft_color::Mat3;
use lightcraft_engine::Session;
use lightcraft_engine::import::{self, ImportCandidate, ImportMode, ImportOptions, ScanProgress};
use lightcraft_geom::{Orientation, Rect};
use lightcraft_raw::{BlackLevel, Cfa, ColorData, DngCompression, DngWriteOptions, OpcodeLists, RawData, RawFormat, RawImage};
use std::error::Error;
use std::path::Path;

const MAX_COUNT: usize = 10_000;
const MAX_EDGE: usize = 4_096;
const MAX_SELECTED: usize = 256;

struct TempDir(std::path::PathBuf);

impl Drop for TempDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn env_usize(name: &str, default: usize, max: usize) -> Result<usize, Box<dyn Error>> {
    let value = std::env::var(name).ok().map(|v| v.parse::<usize>()).transpose()?.unwrap_or(default);
    if value == 0 || value > max {
        return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, format!("{name} must be in 1..={max}")).into());
    }
    Ok(value)
}

fn jpeg_fixture(index: usize, edge: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let width = edge.max(64);
    let height = width.checked_mul(2).ok_or_else(|| std::io::Error::other("JPEG fixture dimensions overflow"))?.checked_div(3).unwrap_or(0).max(48);
    let area = width.checked_mul(height).ok_or_else(|| std::io::Error::other("JPEG fixture area overflow"))?;
    let channels = area.checked_mul(3).ok_or_else(|| std::io::Error::other("JPEG fixture allocation overflow"))?;
    let mut pixels = Vec::with_capacity(channels);
    for y in 0..height {
        for x in 0..width {
            let n = ((x.wrapping_mul(31) ^ y.wrapping_mul(17) ^ index.wrapping_mul(13)) & 31) as u8;
            pixels.extend_from_slice(&[(x * 255 / width) as u8 ^ n, (y * 255 / height) as u8 ^ n, n.wrapping_mul(3)]);
        }
    }
    let image = lightcraft_codecs::EncodeImage::new(width as u32, height as u32, 3, lightcraft_codecs::Samples::U8(&pixels));
    Ok(lightcraft_codecs::encode_jpeg(&image, 88, lightcraft_codecs::ChromaSubsampling::S420, &lightcraft_codecs::EncodeMeta::default())?)
}

fn dng_fixture(index: usize, edge: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let width = edge.max(64);
    let height = width.checked_mul(2).ok_or_else(|| std::io::Error::other("DNG fixture dimensions overflow"))?.checked_div(3).unwrap_or(0).max(48);
    let cfa = Cfa::bayer("RGGB").ok_or_else(|| std::io::Error::other("invalid benchmark CFA"))?;
    let area = width.checked_mul(height).ok_or_else(|| std::io::Error::other("DNG fixture allocation overflow"))?;
    let mut data = Vec::with_capacity(area);
    for y in 0..height {
        for x in 0..width {
            let noise = ((x.wrapping_mul(11) ^ y.wrapping_mul(7) ^ index.wrapping_mul(19)) & 127) as u16;
            let gradient = x.checked_mul(14_000).ok_or_else(|| std::io::Error::other("DNG fixture gradient overflow"))? / width
                + y.checked_mul(1_000).ok_or_else(|| std::io::Error::other("DNG fixture gradient overflow"))? / height;
            data.push(256 + (gradient as u16).saturating_add(noise));
        }
    }
    let raw = RawImage {
        format: RawFormat::Dng,
        width,
        height,
        cpp: 1,
        data: RawData::U16(data),
        cfa: Some(cfa),
        bits: 14,
        black: BlackLevel::uniform(256.0),
        white: vec![16_000.0],
        active_area: Rect::new(0, 0, width, height),
        crop: Rect::new(0, 0, width, height),
        orientation: Orientation::Normal,
        color: ColorData {
            illuminant: [17, 21],
            color_matrix: [Some(Mat3([[0.7, 0.2, 0.1], [0.1, 0.8, 0.1], [0.05, 0.15, 0.8]])), None],
            as_shot_neutral: Some([0.5, 1.0, 0.7]),
            ..Default::default()
        },
        wb_multipliers: None,
        linearized: false,
        opcodes: OpcodeLists::default(),
        metadata: lightcraft_meta::Metadata {
            make: Some("LightCraft Synthetic".into()),
            model: Some("DNG-Import-Bench".into()),
            ..Default::default()
        },
    };
    Ok(lightcraft_raw::write_dng(&raw, &DngWriteOptions { compression: DngCompression::Deflate { tile: 64, half: false }, ..Default::default() })?)
}

fn write_fixtures(root: &Path, count: usize, jpeg_edge: usize, raw_edge: usize) -> Result<usize, Box<dyn Error>> {
    let mut sidecars = 0;
    let jpeg = jpeg_fixture(0, jpeg_edge)?;
    let dng = dng_fixture(0, raw_edge)?;
    for i in 0..count {
        let raw = i % 4 == 0;
        let extension = if raw { "dng" } else { "jpg" };
        // Keep every source unique for hash/dedup coverage while encoding each valid base image
        // once. Trailing bytes are accepted by both containers and make fixture setup bounded.
        let mut bytes = if raw { dng.clone() } else { jpeg.clone() };
        bytes.extend_from_slice(format!("LCG synthetic fixture {i:04}\n").as_bytes());
        let path = root.join(format!("LCG{i:04}.{extension}"));
        std::fs::write(&path, bytes)?;
        if i % 20 == 0 {
            std::fs::write(path.with_extension("xmp"), r#"<x:xmpmeta xmlns:x="adobe:ns:meta/"/>"#)?;
            sidecars += 1;
        }
    }
    Ok(sidecars)
}

fn main() -> Result<(), Box<dyn Error>> {
    let count = env_usize("COUNT", 1_300, MAX_COUNT)?;
    let jpeg_edge = env_usize("JPEG_EDGE", 1_600, MAX_EDGE)?;
    let raw_edge = env_usize("RAW_EDGE", 1_200, MAX_EDGE)?;
    let selected_count = env_usize("SELECTED_COUNT", 16, MAX_SELECTED)?;
    let root = std::env::temp_dir().join(format!("lightcraft-import-bench-{}", std::process::id()));
    let temp = TempDir(root);
    std::fs::create_dir_all(&temp.0)?;
    let sidecars = write_fixtures(&temp.0, count, jpeg_edge, raw_edge)?;

    let mut session = Session::new().with_fs();
    let started = std::time::Instant::now();
    let (input, paths) = import::ScanInput::new(&mut session, &[temp.0.to_string_lossy().into()]);
    let progress = ScanProgress::default();
    let output = import::scan_with(input, &paths, &progress);
    let import::ScanOutput { candidates, probes } = output;
    session.import_probes = probes;
    let candidates: Vec<ImportCandidate> = candidates;
    let elapsed = started.elapsed();
    let failed = candidates.iter().filter(|c| c.error.is_some()).count();
    let raws = candidates.iter().filter(|c| c.kind == MediaKind::Raw).count();
    let metrics = progress.metrics.snapshot();
    println!(
        "import-review files={} raws={raws} sidecars={sidecars} failed={failed} elapsed_ms={:.1}",
        candidates.len(),
        elapsed.as_secs_f64() * 1_000.0
    );
    println!(
        "metrics probes={} bytes_read={} cache_hits={} cache_misses={} probe_ms={:.1} expand_ms={:.1}",
        metrics.probes,
        metrics.bytes_read,
        metrics.cache_hits,
        metrics.cache_misses,
        metrics.probe_ns as f64 / 1e6,
        metrics.expand_ns as f64 / 1e6
    );

    let copy_root = temp.0.join("library");
    let import_paths: Vec<String> =
        candidates.iter().filter(|c| c.error.is_none() && c.duplicate.is_none()).take(selected_count).map(|c| c.path.clone()).collect();
    let opts = ImportOptions { mode: ImportMode::Copy, destination: Some(copy_root.to_string_lossy().into()), ..Default::default() };
    let mut job = import::ImportJob::new(&mut session, opts.clone())?;
    let prepare_started = std::time::Instant::now();
    let prepared = job.prepare(&import_paths, &std::sync::atomic::AtomicBool::new(false));
    let prepare_elapsed = prepare_started.elapsed();
    let commit_started = std::time::Instant::now();
    let report = import::commit_prepared(&mut session, &opts, job.now(), prepared)?;
    let commit_elapsed = commit_started.elapsed();
    let selected_metrics = job.metrics.snapshot();
    println!(
        "selected-import requested={} imported={} failed={} sidecars={} transfer_kind=verified-copy prepare_ms={:.1} commit_ms={:.1}",
        import_paths.len(),
        report.imported.len(),
        report.failed.len(),
        report.sidecars,
        prepare_elapsed.as_secs_f64() * 1_000.0,
        commit_elapsed.as_secs_f64() * 1_000.0,
    );
    println!(
        "selected-metrics probes={} bytes_read={} cache_hits={} cache_misses={} sidecars={} transfers={} transfer_failures={} probe_ms={:.1} sidecar_ms={:.1} transfer_ms={:.1}",
        selected_metrics.probes,
        selected_metrics.bytes_read,
        selected_metrics.cache_hits,
        selected_metrics.cache_misses,
        selected_metrics.sidecars,
        selected_metrics.transfers,
        selected_metrics.transfer_failures,
        selected_metrics.probe_ns as f64 / 1e6,
        selected_metrics.sidecar_ns as f64 / 1e6,
        selected_metrics.transfer_ns as f64 / 1e6,
    );
    let cancel = std::sync::atomic::AtomicBool::new(true);
    let mut cancelled_job = import::ImportJob::new(&mut session, ImportOptions::default())?;
    let cancelled = cancelled_job.prepare(&paths, &cancel);
    println!("cancelled-prepare entries={} cancelled={}", cancelled.len(), cancelled_job.metrics.snapshot().cancelled);
    Ok(())
}
