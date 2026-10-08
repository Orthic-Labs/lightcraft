//! Import-review benchmark for 1,300 mixed procedural files.
//!
//! Run through generated CI actions on a local SSD and a mounted NAS, for example:
//! `cargo run --release -p lightcraft-engine --example import_bench`.
//! The run prints bounded stage counters/timers; SSD/NAS qualification numbers are deliberately
//! left to the host action because this source tree cannot claim measurements for either medium.
//! RAW entries use deterministic RAW-shaped payloads and a header probe, so orchestration and I/O
//! are measured without committing a camera fixture or requiring a decoder corpus.

use lightcraft_engine::Session;
use lightcraft_engine::import::{self, ImportCandidate, ScanProgress};
use lightcraft_engine::media::ProbeInfo;
use std::sync::Arc;

fn main() {
    let root = std::env::temp_dir().join(format!("lightcraft-import-bench-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("create benchmark directory");
    let mut seed = 0x9e37_79b9_u32;
    for i in 0..1_300usize {
        let raw = i % 4 == 0;
        let extension = if raw { "raw" } else { "jpg" };
        let mut bytes = vec![0u8; if raw { 48 * 1024 } else { 12 * 1024 }];
        for byte in &mut bytes {
            seed ^= seed << 13;
            seed ^= seed >> 17;
            seed ^= seed << 5;
            *byte = seed as u8;
        }
        std::fs::write(root.join(format!("LCG{i:04}.{extension}")), bytes).expect("write benchmark input");
    }

    let mut session = Session::new();
    session.media.file_probe = Some(Arc::new(|path| {
        let bytes = std::fs::read(path).map_err(|e| format!("{path}: {e}"))?;
        let raw = path.ends_with(".raw");
        Ok(ProbeInfo {
            width: if raw { 6_000 } else { 4_000 },
            height: if raw { 4_000 } else { 3_000 },
            format: if raw { "RAW" } else { "JPEG" }.into(),
            kind: if raw { lightcraft_catalog::MediaKind::Raw } else { lightcraft_catalog::MediaKind::Image },
            file_size: bytes.len() as u64,
            content_hash: Some(lightcraft_preview::hash_bytes(&bytes).to_string()),
            ..Default::default()
        })
    }));
    let started = std::time::Instant::now();
    let (input, paths) = import::ScanInput::new(&mut session, &[root.to_string_lossy().into()]);
    let progress = ScanProgress::default();
    let candidates: Vec<ImportCandidate> = import::scan_with(input, &paths, &progress).candidates;
    let elapsed = started.elapsed();
    let failed = candidates.iter().filter(|c| c.error.is_some()).count();
    let duplicates = candidates.iter().filter(|c| c.duplicate.is_some()).count();
    println!("import-review files={} failed={failed} duplicates={duplicates} elapsed_ms={:.1}", candidates.len(), elapsed.as_secs_f64() * 1_000.0);
    let metrics = progress.metrics.snapshot();
    println!(
        "metrics probes={} bytes_read={} cache_hits={} cache_misses={} probe_ms={:.1} expand_ms={:.1}",
        metrics.probes,
        metrics.bytes_read,
        metrics.cache_hits,
        metrics.cache_misses,
        metrics.probe_ns as f64 / 1e6,
        metrics.expand_ns as f64 / 1e6
    );
    println!("qualification=unexecuted; run same command on SSD and NAS action hosts");
    let _ = std::fs::remove_dir_all(root);
}
