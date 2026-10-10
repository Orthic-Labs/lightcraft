//! Summarize validated classical culling measurement timing receipts.

use std::collections::BTreeSet;

use serde_json::{Value, json};

const INPUT_SCHEMA: &str = "lightcraft.cull-timing.v1";
const OUTPUT_SCHEMA: &str = "lightcraft.cull-timing-summary.v1";
const SOURCE_POLICY: &str = "uncached-origin-thumbnail";
const SCOPE: &str = "measurement-and-planning";
const EXCLUDES: [&str; 5] = ["timingReceiptSerialization", "proposalSerialization", "proposalBinding", "dispatch", "import"];
const MAX_ITEMS: usize = 10_000;

/// Build a stage-only timing summary from culling engine receipts.
///
/// Every row must be an independently measured successful classical culling sample. The helper
/// keeps missing machine/build/time/cache metadata explicit, and never infers it from receipts.
pub(crate) fn summarize(engine_reports: &[Value], expected_count: usize) -> Result<Value, String> {
    if expected_count > MAX_ITEMS {
        return Err(format!("culling timing expected count exceeds {MAX_ITEMS}"));
    }
    if engine_reports.len() > MAX_ITEMS {
        return Err(format!("culling timing report count exceeds {MAX_ITEMS}"));
    }

    let mut decode = Vec::new();
    let mut analysis = Vec::new();
    let mut total = Vec::new();
    let mut grouping_sum = 0.0;
    let mut job_total_sum = 0.0;
    for engine_report in engine_reports {
        let report = engine_report.get("report").and_then(Value::as_object).ok_or("culling timing engine report is missing report object")?;
        let timing = report.get("measurementTiming").ok_or("culling timing report is missing measurementTiming")?;
        let timing_object = timing.as_object().ok_or("culling measurementTiming must be an object")?;
        if timing_object.get("schema").and_then(Value::as_str) != Some(INPUT_SCHEMA)
            || timing_object.get("classicalOnly").and_then(Value::as_bool) != Some(true)
            || timing_object.get("sourcePolicy").and_then(Value::as_str) != Some(SOURCE_POLICY)
            || timing_object.get("scope").and_then(Value::as_str) != Some(SCOPE)
            || timing_object.get("excludes") != Some(&json!(EXCLUDES))
        {
            return Err("culling measurementTiming policy is unsupported".into());
        }
        validate_stages(timing_object.get("stages").ok_or("culling measurementTiming is missing stages")?)?;
        let grouping = finite_duration(timing_object.get("groupingMs"), "groupingMs")?;
        let job_total = finite_duration(timing_object.get("jobTotalMs"), "jobTotalMs")?;
        grouping_sum += grouping;
        job_total_sum += job_total;
        if !grouping_sum.is_finite() || !job_total_sum.is_finite() {
            return Err("culling timing aggregate is non-finite".into());
        }

        let rows = timing_object.get("perPhoto").and_then(Value::as_array).ok_or("culling measurementTiming is missing perPhoto array")?;
        if rows.len() > MAX_ITEMS {
            return Err(format!("culling perPhoto row count exceeds {MAX_ITEMS}"));
        }
        let mut seen = BTreeSet::new();
        for row in rows {
            let object = row.as_object().ok_or("culling perPhoto row must be an object")?;
            let id = object.get("id").and_then(Value::as_u64).ok_or("culling perPhoto row id must be an unsigned integer")?;
            if !seen.insert(id) {
                return Err("culling perPhoto ids must be unique within report".into());
            }
            if object.get("status").and_then(Value::as_str) != Some("ok") {
                return Err("culling perPhoto row status is not ok".into());
            }
            let decode_ms = finite_duration(object.get("decodeMs"), "decodeMs")?;
            let analysis_ms = finite_duration(object.get("analysisMs"), "analysisMs")?;
            let total_ms = finite_duration(object.get("totalMs"), "totalMs")?;
            if total_ms < decode_ms || total_ms < analysis_ms {
                return Err("culling perPhoto totalMs is less than a measured stage".into());
            }
            decode.push(decode_ms);
            analysis.push(analysis_ms);
            total.push(total_ms);
            if total.len() > MAX_ITEMS || decode.len() > MAX_ITEMS || analysis.len() > MAX_ITEMS {
                return Err(format!("culling timing sample count exceeds {MAX_ITEMS}"));
            }
        }
    }
    if total.len() != expected_count {
        return Err(format!("culling timing sample count {} does not match expected {expected_count}", total.len()));
    }

    Ok(json!({
        "schema": OUTPUT_SCHEMA,
        "status": "stage-only",
        "sampleCount": total.len(),
        "hardware": Value::Null,
        "build": Value::Null,
        "timestamps": Value::Null,
        "cache": Value::Null,
        "metadataMissing": ["hardware", "build", "timestamps", "cache"],
        "decode": measured_stage(&decode),
        "analysis": measured_stage(&analysis),
        "perPhotoTotal": measured_stage(&total),
        "grouping": {"sumMs": grouping_sum, "sampleCount": engine_reports.len()},
        "jobTotal": {"sumMs": job_total_sum},
        "crops": {"status": "notApplicable"},
        "inference": {"status": "notApplicable"},
        "cachePolicy": SOURCE_POLICY,
        "scope": SCOPE,
        "excludes": EXCLUDES,
    }))
}

fn validate_stages(value: &Value) -> Result<(), String> {
    let stages = value.as_object().ok_or("culling measurementTiming stages must be an object")?;
    for stage in ["decode", "analysis"] {
        if stages.get(stage).and_then(Value::as_object).and_then(|object| object.get("status")).and_then(Value::as_str) != Some("measured") {
            return Err(format!("culling timing {stage} stage is not measured"));
        }
    }
    for stage in ["crops", "inference"] {
        if stages.get(stage).and_then(Value::as_object).and_then(|object| object.get("status")).and_then(Value::as_str) != Some("notApplicable") {
            return Err(format!("culling timing {stage} stage must be notApplicable"));
        }
    }
    Ok(())
}

fn finite_duration(value: Option<&Value>, field: &str) -> Result<f64, String> {
    value
        .and_then(Value::as_f64)
        .filter(|value| value.is_finite() && *value >= 0.0)
        .ok_or_else(|| format!("culling timing {field} must be finite and nonnegative"))
}

fn measured_stage(values: &[f64]) -> Value {
    json!({
        "status": "measured",
        "p50Ms": percentile(values, 0.50),
        "p95Ms": percentile(values, 0.95),
    })
}

fn percentile(values: &[f64], probability: f64) -> Option<f64> {
    if values.is_empty() {
        return None;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f64::total_cmp);
    let rank = (probability * sorted.len() as f64).ceil() as usize;
    sorted.get(rank.saturating_sub(1)).copied()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn report(rows: Value, grouping: f64, job_total: f64) -> Value {
        json!({"shootId":"s","report":{"measurementTiming":{
            "schema":INPUT_SCHEMA,"classicalOnly":true,"sourcePolicy":SOURCE_POLICY,
            "scope":SCOPE,"excludes":EXCLUDES,
            "perPhoto":rows,"groupingMs":grouping,"jobTotalMs":job_total,
            "stages":{"decode":{"status":"measured"},"analysis":{"status":"measured"},"crops":{"status":"notApplicable"},"inference":{"status":"notApplicable"}}
        }}})
    }

    fn row(id: u64, decode: f64, analysis: f64, total: f64) -> Value {
        json!({"id":id,"decodeMs":decode,"analysisMs":analysis,"totalMs":total,"status":"ok"})
    }

    #[test]
    fn nearest_rank_percentiles_handle_odd_and_even_samples() {
        let odd = summarize(&[report(json!([row(1, 1.0, 2.0, 3.0), row(2, 5.0, 6.0, 7.0), row(3, 9.0, 10.0, 11.0)]), 2.0, 4.0)], 3).unwrap();
        assert_eq!(odd["decode"]["p50Ms"], 5.0);
        assert_eq!(odd["decode"]["p95Ms"], 9.0);
        let even = summarize(&[report(json!([row(1, 1.0, 2.0, 3.0), row(2, 5.0, 6.0, 7.0)]), 2.0, 4.0)], 2).unwrap();
        assert_eq!(even["decode"]["p50Ms"], 1.0);
        assert_eq!(even["decode"]["p95Ms"], 5.0);
    }

    #[test]
    fn zero_rows_emit_null_percentiles() {
        let summary = summarize(&[report(json!([]), 0.0, 0.0)], 0).unwrap();
        assert!(summary["decode"]["p50Ms"].is_null());
        assert!(summary["perPhotoTotal"]["p95Ms"].is_null());
        assert_eq!(summary["grouping"]["sampleCount"], 1);
    }

    #[test]
    fn duplicate_ids_missing_stage_nonfinite_negative_error_rows_and_count_mismatch_fail() {
        let duplicate = summarize(&[report(json!([row(1, 1.0, 2.0, 3.0), row(1, 1.0, 2.0, 3.0)]), 0.0, 0.0)], 2);
        assert!(duplicate.is_err());
        let mut missing = report(json!([row(1, 1.0, 2.0, 3.0)]), 0.0, 0.0);
        missing["report"]["measurementTiming"]["stages"]["decode"]["status"] = json!("missing");
        assert!(summarize(&[missing], 1).is_err());
        let mut nonfinite = report(json!([row(1, 1.0, 2.0, 3.0)]), 0.0, 0.0);
        nonfinite["report"]["measurementTiming"]["perPhoto"][0]["decodeMs"] = json!("NaN");
        assert!(summarize(&[nonfinite], 1).is_err());
        assert!(summarize(&[report(json!([row(1, -1.0, 2.0, 3.0)]), 0.0, 0.0)], 1).is_err());
        assert!(summarize(&[report(json!([row(1, 3.0, 2.0, 2.0)]), 0.0, 0.0)], 1).is_err());
        let mut error_row = row(1, 1.0, 2.0, 3.0);
        error_row["status"] = json!("decodeFailed");
        assert!(summarize(&[report(json!([error_row]), 0.0, 0.0)], 1).is_err());
        assert!(summarize(&[report(json!([row(1, 1.0, 2.0, 3.0)]), 0.0, 0.0)], 2).is_err());
    }

    #[test]
    fn rejects_non_numeric_engine_ids_and_preserves_missing_metadata() {
        let mut invalid = row(1, 1.0, 2.0, 3.0);
        invalid["id"] = json!({"id": 1});
        assert!(summarize(&[report(json!([invalid]), 0.0, 3.0)], 1).is_err());
        let summary = summarize(&[report(json!([row(1, 1.0, 2.0, 3.0)]), 0.0, 3.0)], 1).unwrap();
        assert_eq!(summary["status"], "stage-only");
        assert_eq!(summary["metadataMissing"], json!(["hardware", "build", "timestamps", "cache"]));
        assert_eq!(summary["crops"]["status"], "notApplicable");
        assert_eq!(summary["inference"]["status"], "notApplicable");
    }
}
