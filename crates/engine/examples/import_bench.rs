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

fn jpeg_fixture(index: usize, edge: usize) -> Result<Vec<u8>, Box<dyn Error>> {
    let width = edge.max(64);
    let height = (width * 2 / 3).max(48);
    let mut pixels = Vec::with_capacity(width * height * 3);
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
    let height = (width * 2 / 3).max(48);
    let cfa = Cfa::bayer("RGGB").ok_or_else(|| std::io::Error::other("invalid benchmark CFA"))?;
    let mut data = Vec::with_capacity(width * height);
    for y in 0..height {
        for x in 0..width {
            let noise = ((x.wrapping_mul(11) ^ y.wrapping_mul(7) ^ index.wrapping_mul(19)) & 127) as u16;
            data.push(256 + ((x * 14_000 / width + y * 1_000 / height) as u16).saturating_add(noise));
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
    let count = std::env::var("COUNT").ok().and_then(|x| x.parse().ok()).unwrap_or(1_300usize);
    let jpeg_edge = std::env::var("JPEG_EDGE").ok().and_then(|x| x.parse().ok()).unwrap_or(1_600usize);
    let raw_edge = std::env::var("RAW_EDGE").ok().and_then(|x| x.parse().ok()).unwrap_or(1_200usize);
    let root = std::env::temp_dir().join(format!("lightcraft-import-bench-{}", std::process::id()));
    std::fs::create_dir_all(&root)?;
    let sidecars = write_fixtures(&root, count, jpeg_edge, raw_edge)?;

    let mut session = Session::new().with_fs();
    let started = std::time::Instant::now();
    let (input, paths) = import::ScanInput::new(&mut session, &[root.to_string_lossy().into()]);
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

    let copy_root = root.join("library");
    let import_paths: Vec<String> =
        candidates.iter().filter(|c| c.error.is_none() && c.duplicate.is_none()).take(8).map(|c| c.path.clone()).collect();
    let opts = ImportOptions { mode: ImportMode::Copy, destination: Some(copy_root.to_string_lossy().into()), ..Default::default() };
    let mut job = import::ImportJob::new(&mut session, opts.clone())?;
    let prepared = job.prepare(&import_paths, &std::sync::atomic::AtomicBool::new(false));
    let report = import::commit_prepared(&mut session, &opts, job.now(), prepared)?;
    let selected_metrics = job.metrics.snapshot();
    println!(
        "selected-import requested={} imported={} failed={} sidecars={} probe_reads={}",
        import_paths.len(),
        report.imported.len(),
        report.failed.len(),
        report.sidecars,
        selected_metrics.probes
    );
    let cancel = std::sync::atomic::AtomicBool::new(true);
    let mut cancelled_job = import::ImportJob::new(&mut session, ImportOptions::default())?;
    let cancelled = cancelled_job.prepare(&paths, &cancel);
    println!("cancelled-prepare entries={} cancelled={}", cancelled.len(), cancelled_job.metrics.snapshot().cancelled);
    let _ = std::fs::remove_dir_all(root);
    Ok(())
}
