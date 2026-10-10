//! Assisted culling: score photos for focus and clipping, group similar shots taken close
//! together (bursts) and mark the sharpest of each group; optionally reject blurry ones and pick
//! each group's best. Scores are kept on the photos (rules / smart albums: `sharpness`,
//! `bestOfGroup`).

use lightcraft_catalog::{Analysis, Flag, Op, PhotoId};
use serde_json::{Value, json};

use super::{CommandSpec, always, cmd};
use crate::{Result, Session};

/// Seconds between shots of the same burst at most.
const BURST_GAP: i64 = 10;
/// Signature similarity for "the same scene".
const SAME: f32 = 0.93;
const PROPOSAL_VERSION: u64 = 1;

fn secs(iso: &str) -> Option<i64> {
    let iso = iso.trim();
    let bytes = iso.as_bytes();
    // Culling needs precise capture times so partial dates never create burst groups. Keep
    // parser work bounded, then reject extra clock fields and malformed fractions that
    // `DateTime::parse_iso` intentionally tolerates.
    if bytes.len() > 128
        || bytes.len() < 19
        || bytes[4] != b'-'
        || bytes[7] != b'-'
        || bytes[10] != b'T'
        || bytes[13] != b':'
        || bytes[16] != b':'
        || !bytes[..4]
            .iter()
            .chain(&bytes[5..7])
            .chain(&bytes[8..10])
            .chain(&bytes[11..13])
            .chain(&bytes[14..16])
            .chain(&bytes[17..19])
            .all(u8::is_ascii_digit)
    {
        return None;
    }
    let parsed = lightcraft_meta::DateTime::parse_iso(iso)?;
    let time = iso.get(11..)?;
    let zone_at = time.find(['Z', '+', '-']).unwrap_or(time.len());
    let clock = &time[..zone_at];
    let mut fields = clock.split(':');
    fields.next()?;
    fields.next()?;
    let seconds = fields.next()?;
    if fields.next().is_some() {
        return None;
    }
    if let Some((_, fraction)) = seconds.split_once('.')
        && (fraction.is_empty() || !fraction.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    let days = match parsed.month {
        1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        4 | 6 | 9 | 11 => 30,
        2 if parsed.year % 400 == 0 || (parsed.year % 4 == 0 && parsed.year % 100 != 0) => 29,
        2 => 28,
        _ => return None,
    };
    if parsed.day == 0 || parsed.day > days || parsed.hour >= 24 || parsed.minute >= 60 || parsed.second >= 60 {
        return None;
    }
    Some(parsed.unix_seconds())
}

fn source_identity(photo: &lightcraft_catalog::Photo) -> String {
    let key = crate::media::content_key(photo);
    let lightcraft_catalog::Source::File { path } = &photo.source else {
        return key;
    };
    let Ok(meta) = std::fs::metadata(path) else {
        return format!("{key}|stat:missing");
    };
    let modified = meta
        .modified()
        .ok()
        .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
        .map(|time| format!("{}:{}", time.as_secs(), time.subsec_nanos()))
        .unwrap_or_else(|| "unknown".into());
    format!("{key}|stat:{modified}:len:{}", meta.len())
}

fn fresh_source(s: &mut Session, id: PhotoId) -> std::result::Result<std::sync::Arc<lightcraft_raster::Rgb32f>, String> {
    let photo = s.catalog.photo(id).ok_or_else(|| format!("unknown photo id {}", id.0))?.clone();
    if !matches!(&photo.source, lightcraft_catalog::Source::File { .. }) {
        return s.source_now(id, crate::media::SourceLevel::Thumb);
    }
    let before = source_identity(&photo);
    let image = s.media.origin_ref(&photo.source, crate::media::SourceLevel::Thumb.max_edge()).load()?;
    let after = s.catalog.photo(id).map(source_identity).ok_or_else(|| format!("unknown photo id {}", id.0))?;
    if before != after {
        return Err("source changed while measuring".into());
    }
    Ok(image)
}

#[derive(Clone)]
struct CullRow {
    id: PhotoId,
    sharpness: f32,
    clipped: f32,
    source: String,
    flag: Flag,
    group: Option<u32>,
    best: bool,
    decision: lightcraft_pipeline::cull::report::Decision,
    uncertainty: lightcraft_pipeline::cull::report::Uncertainty,
    reason_codes: Vec<lightcraft_pipeline::cull::report::ReasonCode>,
}

fn strict_policy(p: &Value, cmd: &str) -> Result<(Option<f32>, bool)> {
    let reject_below = match p.get("rejectBelow") {
        None | Some(Value::Null) => None,
        Some(v) => {
            let n = v
                .as_f64()
                .filter(|n| n.is_finite() && (0.0..=100.0).contains(n))
                .ok_or_else(|| super::bad(cmd, "`rejectBelow` must be a finite number 0..100"))?;
            Some(n as f32)
        }
    };
    let pick_best = match p.get("pickBest") {
        None => false,
        Some(v) => v.as_bool().ok_or_else(|| super::bad(cmd, "`pickBest` must be a boolean"))?,
    };
    Ok((reject_below, pick_best))
}

fn strict_ids(s: &mut Session, p: &Value, cmd: &str) -> Result<Vec<PhotoId>> {
    let ids = if let Some(raw) = p.get("ids") {
        let a = raw.as_array().ok_or_else(|| super::bad(cmd, "`ids` must be an array of photo ids"))?;
        if a.len() > lightcraft_pipeline::cull::report::MAX_MEASUREMENTS {
            return Err(super::bad(cmd, format!("too many culling photo ids: {}", a.len())));
        }
        a.iter()
            .map(|v| v.as_u64().map(PhotoId).ok_or_else(|| super::bad(cmd, "`ids` must contain only unsigned photo ids")))
            .collect::<Result<Vec<_>>>()?
    } else if s.selection.ids.len() > 1 {
        s.targets(p)
    } else {
        s.visible_cloned()
    };
    if ids.len() > lightcraft_pipeline::cull::report::MAX_MEASUREMENTS {
        return Err(super::bad(cmd, format!("too many culling photo ids: {}", ids.len())));
    }
    let mut seen = std::collections::HashSet::new();
    for id in &ids {
        if !seen.insert(*id) {
            return Err(super::bad(cmd, format!("duplicate photo id {}", id.0)));
        }
        if s.catalog.photo(*id).is_none() {
            return Err(super::bad(cmd, format!("unknown photo id {}", id.0)));
        }
    }
    Ok(ids)
}

fn measure(s: &mut Session, ids: &[PhotoId], cmd: &str, reject_below: Option<f32>, pick_best: bool) -> Result<(Vec<CullRow>, Vec<Value>)> {
    let mut rows = Vec::new();
    let mut measurements = Vec::new();
    let mut failed = Vec::new();
    for id in ids {
        let (captured, source, flag) = {
            let p = s.catalog.photo(*id).ok_or_else(|| super::bad(cmd, format!("unknown photo id {}", id.0)))?;
            (p.captured.as_deref().and_then(secs), source_identity(p), p.flag)
        };
        match fresh_source(s, *id) {
            Ok(src) => {
                use lightcraft_pipeline::cull;
                match cull::measure_checked(&src) {
                    Ok(measured) => {
                        measurements.push(lightcraft_pipeline::cull::report::Measurement {
                            id: id.0,
                            captured_secs: captured,
                            sharpness: measured.sharpness,
                            clipped: measured.clipped,
                            signature: measured.signature,
                        });
                        rows.push(CullRow {
                            id: *id,
                            sharpness: measured.sharpness,
                            clipped: measured.clipped,
                            source,
                            flag,
                            group: None,
                            best: false,
                            decision: lightcraft_pipeline::cull::report::Decision::Abstain,
                            uncertainty: lightcraft_pipeline::cull::report::Uncertainty::Abstain,
                            reason_codes: Vec::new(),
                        });
                    }
                    Err(e) => failed.push(json!([id.0, e.to_string()])),
                }
            }
            Err(e) => failed.push(json!([id.0, e])),
        }
    }
    let policy = lightcraft_pipeline::cull::report::CullPolicy { burst_gap_secs: BURST_GAP, similarity_threshold: SAME, reject_below, pick_best };
    let report = lightcraft_pipeline::cull::report::plan(&measurements, &policy).map_err(|e| super::bad(cmd, e.to_string()))?;
    let report_by_id: std::collections::HashMap<u64, &lightcraft_pipeline::cull::report::PhotoReport> =
        report.photos.iter().map(|photo| (photo.id, photo)).collect();
    for row in &mut rows {
        if let Some(measured) = report_by_id.get(&row.id.0) {
            row.group = measured.group;
            row.best = measured.best;
            row.decision = measured.decision;
            row.uncertainty = measured.uncertainty;
            row.reason_codes = measured.reason_codes.clone();
        }
    }
    Ok((rows, failed))
}

fn flag_name(flag: Flag) -> &'static str {
    match flag {
        Flag::None => "none",
        Flag::Pick => "pick",
        Flag::Reject => "reject",
    }
}

fn parse_flag(v: &Value, cmd: &str, key: &str) -> Result<Flag> {
    let name = v.as_str().ok_or_else(|| super::bad(cmd, format!("`{key}` must be a flag string")))?;
    Flag::parse(name).ok_or_else(|| super::bad(cmd, format!("unknown flag `{name}` in `{key}`")))
}

fn proposed_flag(row: &CullRow, reject_below: Option<f32>, pick_best: bool) -> Option<Flag> {
    if row.flag != Flag::None {
        return None;
    }
    if reject_below.is_some_and(|threshold| row.sharpness < threshold) {
        Some(Flag::Reject)
    } else if pick_best && row.best {
        Some(Flag::Pick)
    } else {
        None
    }
}

fn row_json(row: &CullRow, reject_below: Option<f32>, pick_best: bool) -> Value {
    let proposed = proposed_flag(row, reject_below, pick_best).map(flag_name);
    json!({
        "id": row.id.0,
        "source": row.source,
        "flag": flag_name(row.flag),
        "proposedFlag": proposed,
        "sharpness": (row.sharpness * 10.0).round() / 10.0,
        "clipped": row.clipped,
        "group": row.group,
        "best": row.best,
        "decision": row.decision,
        "uncertainty": row.uncertainty,
        "reasonCodes": row.reason_codes,
    })
}

fn binding_hash(revision: u64, reject_below: Option<f32>, pick_best: bool, rows: &[CullRow]) -> String {
    let mut sorted = rows.to_vec();
    sorted.sort_by_key(|r| r.id);
    let mut h = lightcraft_preview::Hasher128::new();
    h.str("photo.cull.v1").u64(revision);
    match reject_below {
        Some(v) => h.u64(v.to_bits() as u64),
        None => h.u64(u64::MAX),
    };
    h.u64(pick_best as u64);
    for row in sorted {
        h.u64(row.id.0).str(&row.source).str(flag_name(row.flag));
        h.u64(row.sharpness.to_bits() as u64).u64(row.clipped.to_bits() as u64);
        h.u64(row.group.unwrap_or(0) as u64).u64(row.best as u64);
        h.str(proposed_flag(&row, reject_below, pick_best).map(flag_name).unwrap_or(""));
    }
    h.finish().to_string()
}

fn proposal_value(s: &Session, rows: &[CullRow], reject_below: Option<f32>, pick_best: bool) -> Value {
    let photos: Vec<Value> = rows.iter().map(|row| row_json(row, reject_below, pick_best)).collect();
    json!({
        "version": PROPOSAL_VERSION,
        "catalogRevision": s.catalog.revision,
        "policy": {"rejectBelow": reject_below, "pickBest": pick_best},
        "photos": photos,
        "binding": binding_hash(s.catalog.revision, reject_below, pick_best, rows),
    })
}

fn proposal_result(s: &Session, rows: &[CullRow], failed: Vec<Value>, reject_below: Option<f32>, pick_best: bool) -> Value {
    let photos: Vec<Value> = rows.iter().map(|row| row_json(row, reject_below, pick_best)).collect();
    let rejected = rows.iter().filter(|row| proposed_flag(row, reject_below, pick_best) == Some(Flag::Reject)).count();
    let picked = rows.iter().filter(|row| proposed_flag(row, reject_below, pick_best) == Some(Flag::Pick)).count();
    let groups = rows.iter().filter_map(|row| row.group).collect::<std::collections::HashSet<_>>().len();
    json!({"photos": photos, "groups": groups, "rejected": rejected, "picked": picked, "failed": failed, "proposal": proposal_value(s, rows, reject_below, pick_best)})
}

fn cull_suggest(s: &mut Session, p: &Value) -> Result<Value> {
    let (reject_below, pick_best) = strict_policy(p, "photo.cullSuggest")?;
    let ids = strict_ids(s, p, "photo.cullSuggest")?;
    let (rows, failed) = measure(s, &ids, "photo.cullSuggest", reject_below, pick_best)?;
    Ok(proposal_result(s, &rows, failed, reject_below, pick_best))
}

fn cull_apply(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "photo.cullApply";
    let proposal_arg = p.get("proposal").ok_or_else(|| super::bad(C, "missing `proposal`"))?;
    let proposal = proposal_arg.get("version").map_or_else(
        || proposal_arg.get("proposal").ok_or_else(|| super::bad(C, "`proposal` must be a cull proposal or cullSuggest result")),
        |_| Ok(proposal_arg),
    )?;
    let version = proposal.get("version").and_then(Value::as_u64).ok_or_else(|| super::bad(C, "proposal version is missing or invalid"))?;
    if version != PROPOSAL_VERSION {
        return Err(super::bad(C, "unsupported proposal version"));
    }
    let revision =
        proposal.get("catalogRevision").and_then(Value::as_u64).ok_or_else(|| super::bad(C, "proposal catalog revision is missing or invalid"))?;
    if revision != s.catalog.revision {
        return Err(super::bad(C, "stale culling proposal: catalog changed"));
    }
    let binding = proposal.get("binding").and_then(Value::as_str).ok_or_else(|| super::bad(C, "proposal binding is missing or invalid"))?;
    if lightcraft_preview::Hash128::parse(binding).is_none() {
        return Err(super::bad(C, "proposal binding is invalid"));
    }
    let policy = proposal.get("policy").and_then(Value::as_object).ok_or_else(|| super::bad(C, "proposal policy is missing or invalid"))?;
    if !policy.contains_key("rejectBelow") || !policy.contains_key("pickBest") || policy.keys().any(|key| key != "rejectBelow" && key != "pickBest") {
        return Err(super::bad(C, "proposal policy fields are invalid"));
    }
    let (reject_below, pick_best) = strict_policy(&Value::Object(policy.clone()), C)?;
    let photo_values = proposal.get("photos").and_then(Value::as_array).ok_or_else(|| super::bad(C, "proposal photos are missing or invalid"))?;
    if photo_values.len() > lightcraft_pipeline::cull::report::MAX_MEASUREMENTS {
        return Err(super::bad(C, "too many culling proposal photos"));
    }
    let mut ids = Vec::with_capacity(photo_values.len());
    let mut seen_ids = std::collections::HashSet::with_capacity(photo_values.len());
    for photo in photo_values {
        let o = photo.as_object().ok_or_else(|| super::bad(C, "proposal photo must be an object"))?;
        let id = o.get("id").and_then(Value::as_u64).map(PhotoId).ok_or_else(|| super::bad(C, "proposal photo id is invalid"))?;
        if !seen_ids.insert(id) {
            return Err(super::bad(C, format!("duplicate proposal photo id {}", id.0)));
        }
        ids.push(id);
        let _ = o.get("source").and_then(Value::as_str).ok_or_else(|| super::bad(C, "proposal photo source is invalid"))?;
        let _ = parse_flag(o.get("flag").ok_or_else(|| super::bad(C, "proposal photo flag is missing"))?, C, "flag")?;
        if let Some(v) = o.get("proposedFlag")
            && !v.is_null()
        {
            let _ = parse_flag(v, C, "proposedFlag")?;
        }
    }
    let accept = p.get("accept").and_then(Value::as_array).ok_or_else(|| super::bad(C, "`accept` must be an array of {id, flag}"))?;
    if accept.len() > lightcraft_pipeline::cull::report::MAX_MEASUREMENTS {
        return Err(super::bad(C, "too many accepted culling photos"));
    }
    let mut accepted = std::collections::BTreeMap::new();
    for item in accept {
        let o = item.as_object().ok_or_else(|| super::bad(C, "each accepted item must be an object"))?;
        let id = o.get("id").and_then(Value::as_u64).map(PhotoId).ok_or_else(|| super::bad(C, "accepted photo id is invalid"))?;
        if accepted.insert(id, parse_flag(o.get("flag").ok_or_else(|| super::bad(C, "accepted flag is missing"))?, C, "flag")?).is_some() {
            return Err(super::bad(C, format!("duplicate accepted photo id {}", id.0)));
        }
    }
    let (rows, failed) = measure(s, &ids, C, reject_below, pick_best)?;
    if !failed.is_empty() {
        return Err(super::bad(C, "stale culling proposal: a proposed photo can no longer be measured"));
    }
    let actual = proposal_value(s, &rows, reject_below, pick_best);
    if actual.get("binding") != Some(&Value::String(binding.to_string())) || actual.get("photos") != proposal.get("photos") {
        return Err(super::bad(C, "stale culling proposal: source, flags or measurements changed"));
    }
    let rows_by_id: std::collections::HashMap<PhotoId, &CullRow> = rows.iter().map(|row| (row.id, row)).collect();
    let mut ops = Vec::new();
    for (id, flag) in accepted {
        let Some(row) = rows_by_id.get(&id) else {
            return Err(super::bad(C, format!("accepted photo id {} is not in proposal", id.0)));
        };
        let expected =
            proposed_flag(row, reject_below, pick_best).ok_or_else(|| super::bad(C, format!("photo {} has no pending cull suggestion", id.0)))?;
        if flag != expected {
            return Err(super::bad(C, format!("accepted flag for photo {} does not match proposal", id.0)));
        }
        if row.flag != Flag::None {
            return Err(super::bad(C, format!("photo {} already has a flag", id.0)));
        }
        ops.push(Op::SetFlag { id, flag });
    }
    if !ops.is_empty() {
        let count = ops.len();
        s.commit("Accept Assisted Culling", Op::Batch { ops })?;
        return Ok(json!({"accepted": count}));
    }
    Ok(json!({"accepted": 0}))
}

fn analyze(s: &mut Session, p: &Value) -> Result<Value> {
    let dry_run = match p.get("dryRun") {
        None => false,
        Some(v) => v.as_bool().ok_or_else(|| super::bad("photo.analyze", "`dryRun` must be a boolean"))?,
    };
    if dry_run {
        let (reject_below, pick_best) = strict_policy(p, "photo.analyze")?;
        let ids = strict_ids(s, p, "photo.analyze")?;
        let (rows, failed) = measure(s, &ids, "photo.analyze", reject_below, pick_best)?;
        return Ok(proposal_result(s, &rows, failed, reject_below, pick_best));
    }
    let ids = strict_ids(s, p, "photo.analyze")?;
    let (reject_below, pick_best) = strict_policy(p, "photo.analyze")?;
    // measure (thumbnail-level sources: fast, and enough for focus at 512 px)
    let mut rows: Vec<(PhotoId, Option<i64>, f32, f32, [f32; 64])> = Vec::new();
    let mut failed = Vec::new();
    for id in ids {
        match s.source_now(id, crate::media::SourceLevel::Thumb) {
            Ok(src) => {
                let t = s.catalog.photo(id).and_then(|p| p.captured.as_deref().and_then(secs));
                use lightcraft_pipeline::cull;
                match cull::measure_checked(&src) {
                    Ok(measured) => rows.push((id, t, measured.sharpness, measured.clipped, measured.signature)),
                    Err(e) => failed.push(json!([id.0, e.to_string()])),
                }
            }
            Err(e) => failed.push(json!([id.0, e])),
        }
    }
    // bursts: consecutive (by capture time) look-alike shots
    rows.sort_by_key(|r| (r.1.unwrap_or(i64::MAX), r.0));
    let mut group_of: Vec<Option<u32>> = vec![None; rows.len()];
    let mut next = 1u32;
    for i in 1..rows.len() {
        let (a, b) = (&rows[i - 1], &rows[i]);
        let close = matches!((a.1, b.1), (Some(x), Some(y)) if x.abs_diff(y) <= BURST_GAP as u64);
        let similar = close
            && match lightcraft_pipeline::cull::similarity_checked(&a.4, &b.4) {
                Ok(similarity) => similarity >= SAME,
                Err(e) => {
                    failed.push(json!([b.0.0, e.to_string()]));
                    false
                }
            };
        if similar {
            let g = *group_of[i - 1].get_or_insert_with(|| {
                next += 1;
                next - 1
            });
            group_of[i] = Some(g);
        }
    }
    let mut best: std::collections::HashMap<u32, (usize, f32)> = Default::default();
    for (i, g) in group_of.iter().enumerate() {
        if let Some(g) = g {
            let e = best.entry(*g).or_insert((i, -1.0));
            if rows[i].2 > e.1 {
                *e = (i, rows[i].2);
            }
        }
    }
    let mut ops = Vec::new();
    let (mut rejected, mut picked) = (0, 0);
    let mut out = Vec::new();
    for (i, (id, _, sharp, clip, _)) in rows.iter().enumerate() {
        let g = group_of[i];
        let is_best = g.is_some_and(|g| best.get(&g).is_some_and(|b| b.0 == i));
        ops.push(Op::SetAnalysis { id: *id, analysis: Some(Analysis { sharpness: *sharp, clipped: *clip, group: g, best: is_best }) });
        let flag = s.catalog.photo(*id).map(|p| p.flag).unwrap_or_default();
        if reject_below.is_some_and(|t| *sharp < t) && flag != Flag::Reject {
            ops.push(Op::SetFlag { id: *id, flag: Flag::Reject });
            rejected += 1;
        } else if pick_best && is_best && flag == Flag::None {
            ops.push(Op::SetFlag { id: *id, flag: Flag::Pick });
            picked += 1;
        }
        out.push(json!({"id": id.0, "sharpness": (sharp * 10.0).round() / 10.0, "clipped": clip, "group": g, "best": is_best}));
    }
    if !ops.is_empty() {
        s.commit("Assisted Culling", Op::Batch { ops })?;
    }
    Ok(json!({"photos": out, "groups": best.len(), "rejected": rejected, "picked": picked, "failed": failed}))
}

/// Find Similar: photos that look like `id` (signatures from thumbnail-level sources, cached by
/// content), most similar first; the view is filtered to them.
fn find_similar(s: &mut Session, p: &Value) -> Result<Value> {
    const C: &str = "library.findSimilar";
    let id = p.get("id").and_then(Value::as_u64).map(PhotoId).or(s.active()).ok_or_else(|| super::bad(C, "no photo"))?;
    let min = p.get("similarity").map_or(Ok(0.8), |v| {
        v.as_f64()
            .filter(|v| v.is_finite() && (0.0..=1.0).contains(v))
            .map(|v| v as f32)
            .ok_or_else(|| super::bad(C, "`similarity` must be a finite number 0..1"))
    })?;
    let all: Vec<PhotoId> = s.catalog.photos().filter(|q| q.in_library()).map(|q| q.id).collect();
    let sig = |s: &mut Session, id: PhotoId| -> std::result::Result<[f32; 64], String> {
        let key = s.catalog.photo(id).map(|p| crate::media::content_key(p)).ok_or_else(|| "photo not found".to_string())?;
        if let Some(v) = s.signatures.get(&key) {
            return Ok(*v);
        }
        let src = s.source_now(id, crate::media::SourceLevel::Thumb).map_err(|e| e.to_string())?;
        let v = lightcraft_pipeline::cull::measure_checked(&src).map_err(|e| e.to_string())?.signature;
        s.signatures.insert(key, v);
        Ok(v)
    };
    let me = sig(s, id).map_err(|_| super::bad(C, "can't read the photo"))?;
    lightcraft_pipeline::cull::similarity_checked(&me, &me).map_err(|_| super::bad(C, "can't read the photo"))?;
    let mut hits: Vec<(PhotoId, f32)> = Vec::new();
    let mut failed = Vec::new();
    for q in all {
        match sig(s, q) {
            Ok(v) => match lightcraft_pipeline::cull::similarity_checked(&me, &v) {
                Ok(sim) if sim >= min => hits.push((q, sim)),
                Ok(_) => {}
                Err(e) => failed.push(json!([q.0, e.to_string()])),
            },
            Err(e) => failed.push(json!([q.0, e])),
        }
    }
    hits.sort_by(|a, b| b.1.total_cmp(&a.1));
    if p.get("filter").and_then(Value::as_bool).unwrap_or(true) {
        s.filter.only = hits.iter().map(|h| h.0).collect();
    }
    let photos = hits.iter().map(|(i, v)| json!({"id": i.0, "similarity": (v * 1000.0).round() / 1000.0})).collect::<Vec<_>>();
    if failed.is_empty() { Ok(json!({"photos": photos})) } else { Ok(json!({"photos": photos, "failed": failed})) }
}

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(
            "library.findSimilar",
            "Find Similar Photos",
            ["Photo"],
            None,
            "{id?, similarity?: 0..1 (0.8), filter?: bool (true)} — photos that look like the active one (composition and tones), most similar first; filters the view to them (Clear Filters to go back) → {photos: [{id, similarity}]}",
            always,
            find_similar
        ),
        cmd!(
            "photo.analyze",
            "Assisted Culling",
            [],
            None,
            "{ids?, dryRun?: bool, rejectBelow?: sharpness 0..100, pickBest?: bool} — score photos for focus and clipping, group look-alikes and optionally set analysis/flags in one undo step; dryRun returns read-only proposals",
            always,
            analyze
        ),
        cmd!(
            "photo.cullApply",
            "Accept Culling Suggestions",
            [],
            None,
            "{proposal: cullSuggest result, accept: [{id, flag}]} — validate an unchanged proposal, accept explicit flags in one undo step → {accepted}",
            always,
            cull_apply
        ),
        cmd!(query "photo.cullSuggest",
            "Suggest Culling Flags",
            [],
            None,
            "{ids?, rejectBelow?: sharpness 0..100, pickBest?: bool} — read-only measurements and safe flag proposals; returns proposal for explicit cullApply → {photos, groups, rejected, picked, failed, proposal}",
            always,
            cull_suggest
        ),
    ]
}
