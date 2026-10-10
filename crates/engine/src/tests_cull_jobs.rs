use lightcraft_catalog::{Flag, Op, PhotoId, Source};
use serde_json::{Value, json};
use std::sync::{
    Mutex,
    atomic::{AtomicUsize, Ordering},
};

use crate::{CullJobResult, Session, cmd::cull::CullJob};

fn demo_ids(s: &mut Session, count: usize) -> Vec<PhotoId> {
    s.visible_cloned().into_iter().take(count).collect()
}

fn tampered_result(base: &CullJobResult, job: &CullJob, mutate: impl FnOnce(&mut serde_json::Value)) -> CullJobResult {
    let mut value = base.value.clone();
    mutate(&mut value);
    CullJobResult { catalog_revision: job.catalog_revision, value }
}

fn assert_tamper_rejected(session: &Session, job: &CullJob, base: &CullJobResult, label: &str, mutate: impl FnOnce(&mut serde_json::Value)) {
    let tampered = tampered_result(base, job, mutate);
    let error = session.validate_cull_job_result(job, &tampered).err().expect("tampered worker result must be rejected").to_string();
    assert!(!error.is_empty(), "tampered {label} result reports a reason");
}

fn detached_apply_params(s: &mut Session) -> (Value, PhotoId, String) {
    let ids = demo_ids(s, 3);
    let suggestion = s
        .execute(
            "photo.cullSuggest",
            &json!({
                "ids": ids.iter().map(|id| id.0).collect::<Vec<_>>(),
                "rejectBelow": 100.0,
                "pickBest": true
            }),
        )
        .expect("cull suggestion");
    let proposal = suggestion["proposal"].clone();
    let row = proposal["photos"]
        .as_array()
        .expect("proposal photos")
        .iter()
        .find(|photo| photo["proposedFlag"].is_string())
        .expect("at least one proposed cull flag");
    let id = PhotoId(row["id"].as_u64().expect("proposal photo ID"));
    let flag = row["proposedFlag"].as_str().expect("proposal flag").to_string();
    let params = json!({
        "proposal": proposal,
        "accept": [{"id": id.0, "flag": flag}]
    });
    (params, id, flag)
}

#[test]
fn detached_job_runs_without_session_and_reports_bounded_progress() {
    let job = {
        let mut s = Session::with_demo();
        let ids = demo_ids(&mut s, 3);
        s.plan_cull_job(&ids, Some(100.0), true).unwrap()
    };
    let progress = Mutex::new(Vec::new());
    let result = job
        .run(&|fraction, phase| {
            progress.lock().expect("progress mutex").push((fraction, phase.to_string()));
            true
        })
        .unwrap();
    let events = progress.into_inner().expect("progress mutex");
    assert!(!events.is_empty(), "worker reports progress");
    assert!(events.iter().all(|(fraction, _)| (0.0..=1.0).contains(fraction)), "progress remains bounded: {events:?}");
    assert!(events.windows(2).all(|pair| pair[0].0 <= pair[1].0), "progress is monotonic: {events:?}");
    assert_eq!(result.catalog_revision, job.catalog_revision);
    assert!(result.value["proposal"].is_object(), "worker returns nested applyable proposal: {}", result.value);
}

#[test]
fn planning_and_running_job_are_read_only() {
    let mut s = Session::with_demo();
    let ids = demo_ids(&mut s, 3);
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let before_journal = s.journal.clone();
    let _ = s.drain_log();
    let job = s.plan_cull_job(&ids, Some(100.0), true).unwrap();
    let _ = job.run(&|_, _| true).unwrap();
    assert_eq!(s.catalog.to_snapshot(), before_catalog, "detached planning leaves analysis and flags unchanged");
    assert_eq!(s.undo.len(), before_undo, "detached planning does not add undo");
    assert_eq!(s.journal, before_journal, "detached planning does not journal");
    assert!(s.drain_log().is_empty(), "detached planning leaves no pending catalog operations");
}

#[test]
fn cancelled_worker_stops_before_decode_and_after_partial_progress() {
    let mut s = Session::with_demo();
    let ids = demo_ids(&mut s, 3);
    let job = s.plan_cull_job(&ids, Some(100.0), true).unwrap();

    let before = s.catalog.to_snapshot();
    let immediate = job.run(&|_, _| false).err().expect("immediate cancellation");
    assert_eq!(immediate, "cancelled");
    assert_eq!(s.catalog.to_snapshot(), before, "cancellation does not mutate session catalog");

    let calls = AtomicUsize::new(0);
    let partial = job
        .run(&|_, _| {
            let count = calls.fetch_add(1, Ordering::Relaxed) + 1;
            count < 4
        })
        .err()
        .expect("partial cancellation");
    assert_eq!(partial, "cancelled");
    assert!(calls.load(Ordering::Relaxed) >= 2, "worker observes cancellation during bounded work");
    assert_eq!(s.catalog.to_snapshot(), before, "partial cancellation does not mutate session catalog");
}

#[test]
fn worker_result_rejects_stale_catalog_revision() {
    let mut s = Session::with_demo();
    let id = demo_ids(&mut s, 1)[0];
    let job = s.plan_cull_job(&[id], Some(100.0), false).unwrap();
    let result = job.run(&|_, _| true).unwrap();
    s.execute("photo.flag", &json!({"ids": [id.0], "flag": "pick"})).unwrap();
    let error = s.validate_cull_job_result(&job, &result).unwrap_err().to_string();
    assert!(error.contains("catalog changed"), "revision change invalidates worker result: {error}");
}

#[test]
fn worker_result_rechecks_flags_and_source_identity_independently() {
    let mut flag_session = Session::with_demo();
    let flag_id = demo_ids(&mut flag_session, 1)[0];
    let flag_job = flag_session.plan_cull_job(&[flag_id], Some(100.0), false).unwrap();
    let flag_result = flag_job.run(&|_, _| true).unwrap();
    flag_session.commit("fixture", Op::SetFlag { id: flag_id, flag: Flag::Pick }).unwrap();
    flag_session.catalog.revision = flag_job.catalog_revision;
    let flag_error = flag_session.validate_cull_job_result(&flag_job, &flag_result).unwrap_err().to_string();
    assert!(flag_error.contains("source or flags changed"), "flag change invalidates worker result: {flag_error}");

    let mut source_session = Session::with_demo();
    let source_id = demo_ids(&mut source_session, 1)[0];
    let source_job = source_session.plan_cull_job(&[source_id], Some(100.0), false).unwrap();
    let source_result = source_job.run(&|_, _| true).unwrap();
    source_session
        .commit("fixture", Op::SetFile { id: source_id, file_name: "stale-source.jpg".into(), source: Source::Demo { scene: 999 } })
        .unwrap();
    source_session.catalog.revision = source_job.catalog_revision;
    let source_error = source_session.validate_cull_job_result(&source_job, &source_result).unwrap_err().to_string();
    assert!(source_error.contains("source or flags changed"), "source change invalidates worker result: {source_error}");
}

#[test]
fn worker_result_rejects_tampered_policy_rows_and_binding() {
    let mut s = Session::with_demo();
    let ids = demo_ids(&mut s, 2);
    let job = s.plan_cull_job(&ids, Some(100.0), true).unwrap();
    let result = job.run(&|_, _| true).unwrap();

    assert_tamper_rejected(&s, &job, &result, "policy", |value| {
        value["proposal"]["policy"]["pickBest"] = json!(false);
    });
    assert_tamper_rejected(&s, &job, &result, "source", |value| {
        let photos = {
            let rows = value["photos"].as_array_mut().expect("result photos");
            rows[0]["source"] = json!("tampered-source");
            rows.clone()
        };
        value["proposal"]["photos"] = json!(photos);
    });
    assert_tamper_rejected(&s, &job, &result, "flag", |value| {
        let photos = {
            let rows = value["photos"].as_array_mut().expect("result photos");
            rows[0]["flag"] = json!("pick");
            rows.clone()
        };
        value["proposal"]["photos"] = json!(photos);
    });
    assert_tamper_rejected(&s, &job, &result, "file name", |value| {
        let photos = {
            let rows = value["photos"].as_array_mut().expect("result photos");
            rows[0]["fileName"] = json!("tampered-name.jpg");
            rows.clone()
        };
        value["proposal"]["photos"] = json!(photos);
    });
    assert_tamper_rejected(&s, &job, &result, "removed row", |value| {
        let photos = {
            let rows = value["photos"].as_array_mut().expect("result photos");
            rows.pop();
            rows.clone()
        };
        value["proposal"]["photos"] = json!(photos);
    });
    assert_tamper_rejected(&s, &job, &result, "duplicate row ID", |value| {
        let mut photos = value["photos"].as_array().expect("result photos").clone();
        photos[1] = photos[0].clone();
        value["photos"] = json!(photos.clone());
        value["proposal"]["photos"] = json!(photos);
    });
    assert_tamper_rejected(&s, &job, &result, "binding", |value| {
        value["proposal"]["binding"] = json!("00000000000000000000000000000000");
    });
}

#[test]
fn detached_apply_stays_read_only_then_commits_one_undoable_batch() {
    let mut s = Session::with_demo();
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let _ = s.drain_log();
    let before_journal = s.journal.clone();
    let (params, id, flag) = detached_apply_params(&mut s);
    let prepared = s.plan_cull_apply(&params).expect("valid detached apply plan");
    let result = prepared.job.run(&|_, _| true).expect("detached apply worker");

    assert_eq!(s.catalog.to_snapshot(), before_catalog, "planning and worker leave flags unchanged");
    assert_eq!(s.undo.len(), before_undo, "planning and worker add no undo");
    assert_eq!(s.journal, before_journal, "planning and worker add no journal");
    assert!(s.drain_log().is_empty(), "planning and worker add no pending operations");

    let applied = s.execute_prepared_command("photo.cullApply", &params, |owner| owner.finish_cull_apply(prepared, result)).expect("prepared apply");
    assert_eq!(applied["accepted"], 1);
    let expected = match flag.as_str() {
        "pick" => Flag::Pick,
        "reject" => Flag::Reject,
        other => panic!("unexpected proposed flag {other}"),
    };
    assert_eq!(s.catalog.photo(id).expect("accepted photo").flag, expected);
    assert_eq!(s.undo.len(), before_undo + 1, "accepted flags share one undo step");
    assert_eq!(s.journal.len(), before_journal.len() + 1, "prepared apply journals once");
    assert_eq!(s.drain_log().len(), 1, "prepared apply queues one batch operation");

    s.execute("edit.undo", &json!({})).expect("undo cull apply");
    assert_eq!(s.catalog.to_snapshot(), before_catalog, "undo restores detached apply state");
}

#[test]
fn cancelled_detached_apply_worker_cannot_reach_finish_or_mutate_session() {
    let mut s = Session::with_demo();
    let (params, _, _) = detached_apply_params(&mut s);
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let _ = s.drain_log();
    let before_journal = s.journal.clone();
    let prepared = s.plan_cull_apply(&params).expect("valid detached apply plan");
    let error = prepared.job.run(&|_, _| false).err().expect("cancelled detached apply");
    assert_eq!(error, "cancelled");
    assert_eq!(s.catalog.to_snapshot(), before_catalog, "cancelled worker leaves catalog unchanged");
    assert_eq!(s.undo.len(), before_undo, "cancelled worker adds no undo");
    assert_eq!(s.journal, before_journal, "cancelled worker adds no journal");
    assert!(s.drain_log().is_empty(), "cancelled worker adds no pending operations");
}

#[test]
fn stale_detached_apply_rejects_atomically_before_commit() {
    let mut s = Session::with_demo();
    let (params, id, _) = detached_apply_params(&mut s);
    let prepared = s.plan_cull_apply(&params).expect("valid detached apply plan");
    let result = prepared.job.run(&|_, _| true).expect("detached apply worker");
    s.execute("photo.flag", &json!({"ids": [id.0], "flag": "pick"})).expect("make plan stale");
    let before_finish_catalog = s.catalog.to_snapshot();
    let before_finish_undo = s.undo.len();
    let before_finish_journal = s.journal.clone();
    let _ = s.drain_log();

    let error =
        s.execute_prepared_command("photo.cullApply", &params, |owner| owner.finish_cull_apply(prepared, result)).expect_err("stale prepared apply");
    assert!(error.to_string().contains("stale culling result"), "stale apply is rejected: {error}");
    assert_eq!(s.catalog.to_snapshot(), before_finish_catalog, "stale apply does not partially mutate");
    assert_eq!(s.undo.len(), before_finish_undo, "stale apply adds no undo");
    assert_eq!(s.journal, before_finish_journal, "stale apply adds no journal");
    assert!(s.drain_log().is_empty(), "stale apply adds no pending operations");
}

#[test]
fn malformed_accept_is_rejected_before_detached_worker_start() {
    let mut s = Session::with_demo();
    let (mut params, id, _) = detached_apply_params(&mut s);
    params["accept"] = json!([{"id": id.0, "flag": "invalid-flag"}]);
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let _ = s.drain_log();
    let before_journal = s.journal.clone();

    let error = s.plan_cull_apply(&params).err().expect("malformed accept must fail during planning").to_string();
    assert!(error.contains("unknown flag"), "malformed accept reports validation error: {error}");
    assert_eq!(s.catalog.to_snapshot(), before_catalog, "malformed accept leaves catalog unchanged");
    assert_eq!(s.undo.len(), before_undo, "malformed accept adds no undo");
    assert_eq!(s.journal, before_journal, "malformed accept adds no journal");
    assert!(s.drain_log().is_empty(), "malformed accept adds no pending operations");
}

#[test]
fn unavailable_accept_is_rejected_before_detached_worker_start() {
    let mut s = Session::with_demo();
    let (params, id, flag) = detached_apply_params(&mut s);
    let other_flag = if flag == "pick" { "reject" } else { "pick" };
    let before_catalog = s.catalog.to_snapshot();
    let before_undo = s.undo.len();
    let _ = s.drain_log();
    let before_journal = s.journal.clone();
    for accept in [json!([{"id": u64::MAX, "flag": flag}]), json!([{"id": id.0, "flag": "none"}]), json!([{"id": id.0, "flag": other_flag}])] {
        let mut invalid = params.clone();
        invalid["accept"] = accept;
        let error = s.plan_cull_apply(&invalid).err().expect("unavailable accept must fail during planning").to_string();
        assert!(error.contains("does not match an available proposal"), "unavailable accept reports validation error: {error}");
        assert_eq!(s.catalog.to_snapshot(), before_catalog, "unavailable accept leaves catalog unchanged");
        assert_eq!(s.undo.len(), before_undo, "unavailable accept adds no undo");
        assert_eq!(s.journal, before_journal, "unavailable accept adds no journal");
        assert!(s.drain_log().is_empty(), "unavailable accept adds no pending operations");
    }
}

#[test]
fn detached_job_rejects_malformed_policy_duplicate_and_unknown_ids() {
    let mut s = Session::with_demo();
    let id = demo_ids(&mut s, 1)[0];
    for threshold in [f32::NAN, -1.0, 101.0] {
        let error = s.plan_cull_job(&[id], Some(threshold), false).err().expect("invalid threshold").to_string();
        assert!(error.contains("rejectBelow"), "invalid threshold is rejected: {error}");
    }
    let duplicate = s.plan_cull_job(&[id, id], None, false).err().expect("duplicate ID").to_string();
    assert!(duplicate.contains("duplicate photo id"), "duplicate IDs are rejected: {duplicate}");
    let unknown = s.plan_cull_job(&[PhotoId(u64::MAX)], None, false).err().expect("unknown ID").to_string();
    assert!(unknown.contains("unknown photo id"), "unknown IDs are rejected: {unknown}");
}

#[test]
fn detached_job_constructor_rejects_duplicate_snapshots() {
    let mut s = Session::with_demo();
    let id = demo_ids(&mut s, 1)[0];
    let job = s.plan_cull_job(&[id], None, false).unwrap();
    let duplicate =
        CullJob::new(job.catalog_revision, vec![job.photos[0].clone(), job.photos[0].clone()], None, false).err().expect("duplicate snapshot");
    assert!(duplicate.contains("duplicate photo id"), "detached constructor rejects duplicate snapshots: {duplicate}");
}
