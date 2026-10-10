use lightcraft_catalog::{Flag, Op, Photo, PhotoId, Source};
use lightcraft_pipeline::SourceInfo;
use lightcraft_raster::Rgb32f;
use serde_json::{Value, json};
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};

use crate::Session;

fn unflagged(s: &Session, n: usize) -> Vec<PhotoId> {
    s.catalog.photos().filter(|p| p.flag == Flag::None).take(n).map(|p| p.id).collect()
}

fn flags(s: &Session, ids: &[PhotoId]) -> Vec<Flag> {
    ids.iter().map(|id| s.catalog.photo(*id).map(|p| p.flag).unwrap_or(Flag::Reject)).collect()
}

fn proposal(s: &mut Session, ids: &[PhotoId]) -> Value {
    s.execute("photo.cullSuggest", &json!({"ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(), "rejectBelow": 100.0, "pickBest": true}))
        .expect("cull suggestion")
        .get("proposal")
        .cloned()
        .expect("nested cull proposal")
}

fn suggested_id(p: &Value, flag: &str) -> PhotoId {
    p["photos"]
        .as_array()
        .expect("proposal photos")
        .iter()
        .find_map(|photo| (photo["proposedFlag"].as_str() == Some(flag)).then(|| PhotoId(photo["id"].as_u64().expect("proposal photo id"))))
        .expect("proposal has requested suggestion")
}

#[test]
fn dry_run_and_suggest_leave_catalog_undo_pending_log_and_journal_unchanged() {
    let mut s = Session::with_demo();
    let ids = unflagged(&s, 3);
    assert_eq!(ids.len(), 3);
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let before_journal = s.journal.clone();
    let _ = s.drain_log();

    let dry = s
        .execute(
            "photo.analyze",
            &json!({"ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(), "dryRun": true, "rejectBelow": 100.0, "pickBest": true}),
        )
        .unwrap();
    assert!(dry.get("proposal").is_some(), "dry run returns an applyable proposal: {dry}");
    let suggested = s
        .execute("photo.cullSuggest", &json!({"ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(), "rejectBelow": 100.0, "pickBest": true}))
        .unwrap();
    assert!(suggested.get("proposal").is_some(), "suggestion returns an applyable proposal: {suggested}");

    assert_eq!(s.catalog.to_snapshot(), before_catalog, "read-only culling does not write analysis or flags");
    assert_eq!(s.undo.len(), before_undo, "read-only culling does not add undo");
    assert_eq!(s.journal, before_journal, "read-only culling does not journal");
    assert!(s.drain_log().is_empty(), "read-only culling leaves no pending catalog ops");
}

#[test]
fn explicit_accept_is_one_undo_step_and_restores_every_flag() {
    let mut s = Session::with_demo();
    let ids = unflagged(&s, 3);
    let before_catalog = s.catalog.to_snapshot();
    let before_flags = flags(&s, &ids);
    let before_undo = s.undo.len();
    let p = proposal(&mut s, &ids);
    let accepted = suggested_id(&p, "reject");

    let result = s.execute("photo.cullApply", &json!({"proposal": p, "accept": [{"id": accepted.0, "flag": "reject"}]})).unwrap();
    assert_eq!(result["accepted"], 1);
    assert_eq!(s.catalog.photo(accepted).unwrap().flag, Flag::Reject);
    assert_eq!(s.undo.len(), before_undo + 1, "all explicit accepts share one undo step");

    s.execute("edit.undo", &json!({})).unwrap();
    assert_eq!(s.catalog.to_snapshot(), before_catalog, "undo restores catalog including analysis and flags");
    assert_eq!(flags(&s, &ids), before_flags);
}

#[test]
fn cancelled_empty_accept_stale_proposal_and_invalid_ids_never_partially_mutate() {
    let mut s = Session::with_demo();
    let ids = unflagged(&s, 3);
    let p = proposal(&mut s, &ids);
    let before = s.catalog.to_snapshot();
    let undo_before = s.undo.len();

    let cancelled = s.execute("photo.cullApply", &json!({"proposal": p, "accept": []})).unwrap();
    assert_eq!(cancelled["accepted"], 0, "empty acceptance is a cancellation/no-op");
    assert_eq!(s.catalog.to_snapshot(), before);
    assert_eq!(s.undo.len(), undo_before);

    let p = proposal(&mut s, &ids);
    s.execute("photo.flag", &json!({"ids": [ids[0].0], "flag": "pick"})).unwrap();
    let changed = s.catalog.to_snapshot();
    let error = s
        .execute("photo.cullApply", &json!({"proposal": p, "accept": [{"id": ids[1].0, "flag": "reject"}, {"id": ids[2].0, "flag": "reject"}]}))
        .unwrap_err()
        .to_string();
    assert!(error.contains("stale"), "stale proposal is rejected: {error}");
    assert_eq!(s.catalog.to_snapshot(), changed, "stale rejection does not partially apply");

    let unknown = s.execute("photo.cullSuggest", &json!({"ids": [ids[1].0, u64::MAX], "rejectBelow": 100.0})).unwrap_err().to_string();
    assert!(unknown.contains("unknown photo id"), "unknown IDs are rejected distinctly: {unknown}");
    assert_eq!(s.catalog.to_snapshot(), changed);
}

#[test]
fn duplicate_ids_nonfinite_thresholds_unknown_accepts_and_unpicked_are_rejected() {
    let mut s = Session::with_demo();
    let ids = unflagged(&s, 2);
    let duplicate = s.execute("photo.cullSuggest", &json!({"ids": [ids[0].0, ids[0].0]})).unwrap_err().to_string();
    assert!(duplicate.contains("duplicate photo id"), "duplicate IDs are rejected: {duplicate}");
    let nonfinite = s.execute("photo.cullSuggest", &json!({"ids": [ids[0].0], "rejectBelow": "NaN"})).unwrap_err().to_string();
    assert!(nonfinite.contains("rejectBelow"), "non-finite threshold input is rejected: {nonfinite}");
    assert!(s.execute("photo.cullSuggest", &json!({"ids": [ids[0].0], "rejectBelow": -1.0})).is_err());
    assert!(s.execute("photo.cullSuggest", &json!({"ids": [ids[0].0], "rejectBelow": 101.0})).is_err());

    let p = s
        .execute("photo.cullSuggest", &json!({"ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(), "rejectBelow": 0.0, "pickBest": false}))
        .unwrap()["proposal"]
        .clone();
    let before = s.catalog.to_snapshot();
    let unpicked = s.execute("photo.cullApply", &json!({"proposal": p, "accept": [{"id": ids[1].0, "flag": "pick"}]})).unwrap_err().to_string();
    assert!(unpicked.contains("does not match an available proposal"), "existing but unpicked photo is distinct: {unpicked}");
    assert_eq!(s.catalog.to_snapshot(), before);

    let p = proposal(&mut s, &ids);
    let unknown = s.execute("photo.cullApply", &json!({"proposal": p, "accept": [{"id": u64::MAX, "flag": "reject"}]})).unwrap_err().to_string();
    assert!(unknown.contains("not in proposal"), "unknown accepted photo is distinct: {unknown}");
    assert_eq!(s.catalog.to_snapshot(), before);
}

#[test]
fn tampered_proposal_and_valid_then_unknown_accept_are_atomic() {
    let mut s = Session::with_demo();
    let ids = unflagged(&s, 3);
    let mut tampered = proposal(&mut s, &ids);
    let accepted = suggested_id(&tampered, "reject");
    tampered["photos"][0]["source"] = json!("tampered-source");
    let before = s.catalog.to_snapshot();
    let tampered_error =
        s.execute("photo.cullApply", &json!({"proposal": tampered, "accept": [{"id": accepted.0, "flag": "reject"}]})).unwrap_err().to_string();
    assert!(tampered_error.contains("binding does not match contents"), "tampering is rejected: {tampered_error}");
    assert_eq!(s.catalog.to_snapshot(), before, "tampered proposal cannot partially apply");

    let p = proposal(&mut s, &ids);
    let before = s.catalog.to_snapshot();
    let mixed_error = s
        .execute("photo.cullApply", &json!({"proposal": p, "accept": [{"id": accepted.0, "flag": "reject"}, {"id": u64::MAX, "flag": "reject"}]}))
        .unwrap_err()
        .to_string();
    assert!(mixed_error.contains("not in proposal"), "unknown after valid accept is rejected: {mixed_error}");
    assert_eq!(s.catalog.to_snapshot(), before, "valid prefix is not committed before unknown ID");
}

#[test]
fn burst_ordering_uses_real_seconds_across_month_boundary() {
    let mut s = Session::with_demo();
    let first = s.visible_cloned()[0];
    let mut clone = s.catalog.photo(first).expect("demo photo").as_ref().clone();
    let second = s.catalog.alloc_photo_id();
    clone.id = second;
    clone.file_name = "month-boundary-copy.jpg".into();
    clone.flag = Flag::None;
    clone.analysis = None;
    s.commit("fixture", Op::AddPhoto { photo: Box::new(clone) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: first, captured: Some("2026-02-28T23:59:55".into()) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: second, captured: Some("2026-03-01T00:00:05".into()) }).unwrap();
    let result = s.execute("photo.cullSuggest", &json!({"ids": [first.0, second.0]})).unwrap();
    assert_eq!(result["groups"], 1, "ten-second burst crosses month boundary: {result}");
}

#[test]
fn invalid_capture_clock_is_treated_as_unknown_time() {
    let mut s = Session::with_demo();
    let id = s.visible_cloned()[0];
    for captured in ["2026-03-01T24:00:00", "2026-03-01T23:59:99"] {
        s.commit("fixture", Op::SetCaptured { id, captured: Some(captured.into()) }).unwrap();
        let result = s.execute("photo.cullSuggest", &json!({"ids": [id.0]})).unwrap();
        let reasons = result["photos"][0]["reasonCodes"].as_array().expect("reason codes");
        assert!(reasons.iter().any(|reason| reason == "missingCaptureTime"), "invalid clock is not used for ordering: {result}");
    }
}

#[test]
fn timezone_equivalent_capture_times_join_burst() {
    let mut s = Session::with_demo();
    let first = s.visible_cloned()[0];
    let mut clone = s.catalog.photo(first).expect("demo photo").as_ref().clone();
    let second = s.catalog.alloc_photo_id();
    clone.id = second;
    clone.file_name = "timezone-copy.jpg".into();
    clone.flag = Flag::None;
    clone.analysis = None;
    s.commit("fixture", Op::AddPhoto { photo: Box::new(clone) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: first, captured: Some("2026-04-01T10:00:00+02:00".into()) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: second, captured: Some("2026-04-01T08:00:05Z".into()) }).unwrap();
    let result = s.execute("photo.cullSuggest", &json!({"ids": [first.0, second.0]})).unwrap();
    assert_eq!(result["groups"], 1, "equivalent timezone instants form one burst: {result}");
}

#[test]
fn changed_decoded_file_pixels_invalidate_cached_cull_proposal() {
    let changed = Arc::new(AtomicBool::new(false));
    let initial = Rgb32f::from_fn(64, 64, |x, y| {
        let v = if (x / 4 + y / 4) % 2 == 0 { 0.1 } else { 0.9 };
        [v, v, v]
    });
    let replacement = Rgb32f::filled(64, 64, [0.5, 0.5, 0.5]);
    let initial_loader = initial.clone();
    let replacement_loader = replacement.clone();
    let loader_state = changed.clone();
    let mut s = Session::new();
    s.catalog
        .apply(Op::AddPhoto {
            photo: Box::new(Photo::new(
                PhotoId(1),
                Source::File { path: "fixture://cull-cache.jpg".into() },
                "cull-cache.jpg",
                "JPEG",
                64,
                64,
                "2026-04-01T00:00:00",
            )),
        })
        .unwrap();
    s.media.file_loader = Some(Arc::new(move |path, _| {
        if path != "fixture://cull-cache.jpg" {
            return Err(format!("unexpected fixture path: {path}"));
        }
        Ok((if loader_state.load(Ordering::Relaxed) { replacement_loader.clone() } else { initial_loader.clone() }, SourceInfo::default()))
    }));

    let suggested = s.execute("photo.cullSuggest", &json!({"ids": [1], "rejectBelow": 100.0})).unwrap();
    let proposal = suggested["proposal"].clone();
    let before = s.catalog.to_snapshot();
    let undo_before = s.undo.len();
    changed.store(true, Ordering::Relaxed);
    let error = s.execute("photo.cullApply", &json!({"proposal": proposal, "accept": []})).unwrap_err().to_string();
    assert!(error.contains("stale"), "changed decoded pixels invalidate proposal: {error}");
    assert_eq!(s.catalog.to_snapshot(), before, "stale decoded source cannot mutate catalog");
    assert_eq!(s.undo.len(), undo_before, "stale decoded source cannot add undo");
}

#[test]
fn tied_burst_members_are_review_only_without_best_or_pick() {
    let mut s = Session::with_demo();
    let first = s.visible_cloned()[0];
    let mut clone = s.catalog.photo(first).expect("demo photo").as_ref().clone();
    let second = s.catalog.alloc_photo_id();
    clone.id = second;
    clone.file_name = "tie-copy.jpg".into();
    clone.flag = Flag::None;
    clone.analysis = None;
    s.commit("fixture", Op::AddPhoto { photo: Box::new(clone) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: first, captured: Some("2026-04-01T10:00:00".into()) }).unwrap();
    s.commit("fixture", Op::SetCaptured { id: second, captured: Some("2026-04-01T10:00:01".into()) }).unwrap();
    let result = s.execute("photo.cullSuggest", &json!({"ids": [first.0, second.0], "pickBest": true})).unwrap();
    assert_eq!(result["groups"], 1, "identical burst members form one group: {result}");
    for row in result["photos"].as_array().expect("photos") {
        assert_eq!(row["decision"], "review", "ties require review: {result}");
        assert!(!row["best"].as_bool().unwrap_or(true), "ties have no best member: {result}");
        assert!(row["proposedFlag"].is_null(), "ties do not emit a pick: {result}");
    }
}
