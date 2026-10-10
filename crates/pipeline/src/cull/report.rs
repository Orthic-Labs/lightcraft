//! Pure, reviewable planning for assisted culling.
//!
//! This module turns already measured photos into a deterministic report.  It does not alter a
//! catalog, delete files, or claim calibrated probabilities.  Measurements are deliberately
//! classical: focus, clipping, capture time, and a small luminance signature.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::fmt;

use serde::{Deserialize, Serialize};

mod signature_serde {
    use serde::Serialize;
    use serde::de::{self, Deserializer, SeqAccess, Visitor};
    use serde::ser::Serializer;
    use std::fmt;

    pub fn serialize<S>(value: &[f32; 64], serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        value.as_slice().serialize(serializer)
    }

    struct SignatureVisitor;

    impl<'de> Visitor<'de> for SignatureVisitor {
        type Value = [f32; 64];

        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an array of exactly 64 numbers")
        }

        fn visit_seq<A>(self, mut sequence: A) -> Result<Self::Value, A::Error>
        where
            A: SeqAccess<'de>,
        {
            let mut values = Vec::with_capacity(64);
            while let Some(value) = sequence.next_element::<f32>()? {
                if values.len() >= 64 {
                    return Err(de::Error::invalid_length(values.len() + 1, &self));
                }
                values.push(value);
            }
            values.try_into().map_err(|values: Vec<f32>| de::Error::invalid_length(values.len(), &self))
        }
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<[f32; 64], D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_seq(SignatureVisitor)
    }
}

/// Version of [`CullReport`] serialization.
pub const REPORT_VERSION: u32 = 1;
/// Maximum number of measurements accepted by [`plan`].
pub const MAX_MEASUREMENTS: usize = 100_000;
/// Legacy maximum gap between adjacent photos in one burst.
pub const DEFAULT_BURST_GAP_SECS: i64 = 10;
/// Legacy minimum signature similarity for adjacent photos in one burst.
pub const DEFAULT_SIMILARITY_THRESHOLD: f32 = 0.93;

/// A bounded, read-only measurement supplied by an image-analysis stage.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Measurement {
    pub id: u64,
    pub captured_secs: Option<i64>,
    pub sharpness: f32,
    pub clipped: f32,
    #[serde(with = "signature_serde")]
    pub signature: [f32; 64],
}

/// Policy used to form burst groups and produce reviewable suggestions.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CullPolicy {
    /// Adjacent capture times must be no farther apart than this many seconds.
    pub burst_gap_secs: i64,
    /// Adjacent signatures must have at least this cosine similarity.
    pub similarity_threshold: f32,
    /// Optional focus score below which a reject suggestion is emitted.
    pub reject_below: Option<f32>,
    /// Emit a pick suggestion for each unique sharpest member of a group.
    pub pick_best: bool,
}

impl Default for CullPolicy {
    fn default() -> Self {
        Self::baseline()
    }
}

impl CullPolicy {
    /// Legacy-compatible grouping policy: 10 seconds and 0.93 similarity.
    pub const fn baseline() -> Self {
        Self { burst_gap_secs: DEFAULT_BURST_GAP_SECS, similarity_threshold: DEFAULT_SIMILARITY_THRESHOLD, reject_below: None, pick_best: false }
    }
}

/// Validate policy independently of any measurements.
pub fn validate_policy(policy: &CullPolicy) -> Result<(), ReportError> {
    if policy.burst_gap_secs < 0 {
        return Err(ReportError::InvalidPolicy { field: "burstGapSecs", reason: "must be non-negative" });
    }
    if !policy.similarity_threshold.is_finite() || !(0.0..=1.0).contains(&policy.similarity_threshold) {
        return Err(ReportError::InvalidPolicy { field: "similarityThreshold", reason: "must be finite and within 0..1" });
    }
    if let Some(value) = policy.reject_below
        && (!value.is_finite() || !(0.0..=100.0).contains(&value))
    {
        return Err(ReportError::InvalidPolicy { field: "rejectBelow", reason: "must be finite and within 0..100" });
    }
    Ok(())
}

/// Review outcome for one photo.  `Review` covers unresolved evidence, including ties;
/// `Abstain` means no applicable action was requested.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Decision {
    Reject,
    Keep,
    Pick,
    Review,
    Abstain,
}

/// Qualitative uncertainty only; this is not a probability or confidence estimate.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Uncertainty {
    Low,
    Elevated,
    High,
    Abstain,
}

/// Why a decision was emitted.  Codes describe observable inputs and policy, without making
/// claims about faces, eyes, aesthetics, or model confidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ReasonCode {
    BelowRejectThreshold,
    AtOrAboveRejectThreshold,
    BurstMatch,
    BestOfGroup,
    TiedForBest,
    MissingCaptureTime,
    KeepSuggestion,
    NoActionRequested,
    ReviewRequired,
    Abstained,
}

/// One photo's measurements plus a reviewable suggestion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PhotoReport {
    pub id: u64,
    pub sharpness: f32,
    pub clipped: f32,
    pub group: Option<u32>,
    /// True only for a unique sharpest member of a group.  Tied members are all false.
    pub best: bool,
    pub decision: Decision,
    pub reason_codes: Vec<ReasonCode>,
    pub uncertainty: Uncertainty,
}

/// Group-level evidence used to explain a burst suggestion.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GroupReport {
    pub id: u32,
    pub photo_ids: Vec<u64>,
    /// Present only when exactly one member has the highest sharpness.
    pub best_photo_id: Option<u64>,
    /// One or more IDs when sharpness ties for the maximum.
    pub best_candidates: Vec<u64>,
}

/// Aggregate counts for a report.  Action counts count suggestions, not catalog mutations.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AggregateCounts {
    pub photos: usize,
    pub grouped_photos: usize,
    pub groups: usize,
    pub reject: usize,
    pub keep: usize,
    pub pick: usize,
    pub review: usize,
    pub abstain: usize,
    pub tied_groups: usize,
}

/// Separate read-only suggestion channels.  Consumers decide whether to apply any suggestion.
#[derive(Clone, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SuggestionSets {
    pub reject: Vec<u64>,
    pub keep: Vec<u64>,
    pub pick: Vec<u64>,
}

/// Complete deterministic culling report.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CullReport {
    pub version: u32,
    pub policy: CullPolicy,
    pub photos: Vec<PhotoReport>,
    pub groups: Vec<GroupReport>,
    pub counts: AggregateCounts,
    pub suggestions: SuggestionSets,
}

/// Alias useful to callers that refer to the result simply as a report.
pub type Report = CullReport;

/// Errors raised before planning can produce a safe report.
#[derive(Clone, Debug, PartialEq)]
pub enum ReportError {
    TooManyMeasurements { limit: usize, actual: usize },
    DuplicateId { id: u64 },
    NonFiniteMeasurement { id: u64, field: &'static str },
    OutOfRangeMeasurement { id: u64, field: &'static str },
    InvalidPolicy { field: &'static str, reason: &'static str },
}

impl fmt::Display for ReportError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::TooManyMeasurements { limit, actual } => write!(f, "too many culling measurements: {actual} exceeds {limit}"),
            Self::DuplicateId { id } => write!(f, "duplicate culling measurement id {id}"),
            Self::NonFiniteMeasurement { id, field } => write!(f, "non-finite {field} in culling measurement {id}"),
            Self::OutOfRangeMeasurement { id, field } => write!(f, "out-of-range {field} in culling measurement {id}"),
            Self::InvalidPolicy { field, reason } => write!(f, "invalid culling policy {field}: {reason}"),
        }
    }
}

impl std::error::Error for ReportError {}

fn validate_measurement(measurement: &Measurement) -> Result<(), ReportError> {
    if !measurement.sharpness.is_finite() {
        return Err(ReportError::NonFiniteMeasurement { id: measurement.id, field: "sharpness" });
    }
    if !measurement.clipped.is_finite() {
        return Err(ReportError::NonFiniteMeasurement { id: measurement.id, field: "clipped" });
    }
    if !(0.0..=100.0).contains(&measurement.sharpness) {
        return Err(ReportError::OutOfRangeMeasurement { id: measurement.id, field: "sharpness" });
    }
    if !(0.0..=1.0).contains(&measurement.clipped) {
        return Err(ReportError::OutOfRangeMeasurement { id: measurement.id, field: "clipped" });
    }
    for value in measurement.signature.iter() {
        if !value.is_finite() {
            return Err(ReportError::NonFiniteMeasurement { id: measurement.id, field: "signature" });
        }
    }
    Ok(())
}

fn close_in_time(a: &Measurement, b: &Measurement, gap: i64) -> bool {
    match (a.captured_secs, b.captured_secs) {
        (Some(first), Some(second)) => first <= second && second.checked_sub(first).is_some_and(|delta| delta <= gap),
        _ => false,
    }
}

fn close_in_time_and_signature(a: &Measurement, b: &Measurement, policy: &CullPolicy) -> bool {
    if !close_in_time(a, b, policy.burst_gap_secs) {
        return false;
    }
    match super::similarity_checked(&a.signature, &b.signature) {
        Ok(similarity) => similarity >= policy.similarity_threshold,
        Err(_) => false,
    }
}

fn group_parts<'a>(measurements: &'a [Measurement], policy: &CullPolicy) -> Vec<Vec<&'a Measurement>> {
    let mut sorted: Vec<&Measurement> = measurements.iter().collect();
    sorted.sort_by(|a, b| a.captured_secs.unwrap_or(i64::MAX).cmp(&b.captured_secs.unwrap_or(i64::MAX)).then_with(|| a.id.cmp(&b.id)));

    let mut groups: Vec<Vec<&Measurement>> = Vec::new();
    let mut current: Vec<&Measurement> = Vec::new();
    for measurement in sorted {
        let joins = current.last().is_some_and(|previous| close_in_time_and_signature(previous, measurement, policy));
        if joins {
            current.push(measurement);
        } else {
            if current.len() >= 2 {
                groups.push(current);
            }
            current = vec![measurement];
        }
    }
    if current.len() >= 2 {
        groups.push(current);
    }
    groups
}

fn decision_for(measurement: &Measurement, grouped: bool, best: bool, tied: bool, policy: &CullPolicy) -> (Decision, Uncertainty, Vec<ReasonCode>) {
    let mut reasons = Vec::new();
    if measurement.captured_secs.is_none() {
        reasons.push(ReasonCode::MissingCaptureTime);
    }
    if grouped {
        reasons.push(ReasonCode::BurstMatch);
    }
    if tied {
        reasons.push(ReasonCode::TiedForBest);
    } else if best {
        reasons.push(ReasonCode::BestOfGroup);
    }

    if policy.reject_below.is_some_and(|threshold| measurement.sharpness < threshold) {
        reasons.push(ReasonCode::BelowRejectThreshold);
        return (Decision::Reject, if tied { Uncertainty::High } else { Uncertainty::Low }, reasons);
    }
    if policy.reject_below.is_some() {
        reasons.push(ReasonCode::AtOrAboveRejectThreshold);
    }
    if tied {
        reasons.push(ReasonCode::ReviewRequired);
        return (Decision::Review, Uncertainty::High, reasons);
    }
    if policy.pick_best && best {
        return (Decision::Pick, Uncertainty::Low, reasons);
    }
    if policy.pick_best && grouped {
        reasons.push(ReasonCode::KeepSuggestion);
        return (Decision::Keep, Uncertainty::Low, reasons);
    }
    if policy.reject_below.is_some() {
        return (Decision::Keep, Uncertainty::Low, reasons);
    }
    reasons.push(ReasonCode::NoActionRequested);
    reasons.push(ReasonCode::Abstained);
    (Decision::Abstain, Uncertainty::Abstain, reasons)
}

/// Build a deterministic, read-only culling report from measured photos.
pub fn plan(measurements: &[Measurement], policy: &CullPolicy) -> Result<CullReport, ReportError> {
    if measurements.len() > MAX_MEASUREMENTS {
        return Err(ReportError::TooManyMeasurements { limit: MAX_MEASUREMENTS, actual: measurements.len() });
    }
    validate_policy(policy)?;

    let mut ids = HashSet::with_capacity(measurements.len());
    for measurement in measurements {
        if !ids.insert(measurement.id) {
            return Err(ReportError::DuplicateId { id: measurement.id });
        }
        validate_measurement(measurement)?;
    }

    let parts = group_parts(measurements, policy);
    let mut groups = Vec::with_capacity(parts.len());
    let mut group_by_id = std::collections::HashMap::with_capacity(measurements.len());
    let mut unique_best = HashSet::with_capacity(parts.len());
    let mut tied_groups = HashSet::with_capacity(parts.len());

    for (index, members) in parts.iter().enumerate() {
        let Some(raw_id) = index.checked_add(1) else {
            return Err(ReportError::TooManyMeasurements { limit: MAX_MEASUREMENTS, actual: measurements.len() });
        };
        let Ok(group_id) = u32::try_from(raw_id) else {
            return Err(ReportError::TooManyMeasurements { limit: MAX_MEASUREMENTS, actual: measurements.len() });
        };
        let max_sharpness = members.iter().map(|photo| photo.sharpness).max_by(|a, b| a.total_cmp(b)).unwrap_or(0.0);
        let best_candidates: Vec<u64> =
            members.iter().filter(|photo| photo.sharpness.total_cmp(&max_sharpness) == Ordering::Equal).map(|photo| photo.id).collect();
        let best_photo_id = best_candidates.first().copied().filter(|_| best_candidates.len() == 1);
        if best_photo_id.is_some() {
            unique_best.extend(best_candidates.iter().copied());
        } else {
            tied_groups.insert(group_id);
        }
        for photo in members.iter() {
            group_by_id.insert(photo.id, group_id);
        }
        groups.push(GroupReport { id: group_id, photo_ids: members.iter().map(|photo| photo.id).collect(), best_photo_id, best_candidates });
    }

    let mut sorted: Vec<&Measurement> = measurements.iter().collect();
    sorted.sort_by(|a, b| a.captured_secs.unwrap_or(i64::MAX).cmp(&b.captured_secs.unwrap_or(i64::MAX)).then_with(|| a.id.cmp(&b.id)));

    let mut photos = Vec::with_capacity(measurements.len());
    let mut suggestions = SuggestionSets::default();
    let mut counts =
        AggregateCounts { photos: measurements.len(), groups: groups.len(), tied_groups: tied_groups.len(), ..AggregateCounts::default() };
    for measurement in sorted {
        let group_id = group_by_id.get(&measurement.id).copied();
        let best = unique_best.contains(&measurement.id);
        let tied = group_id.is_some_and(|id| tied_groups.contains(&id));
        let (decision, uncertainty, reason_codes) = decision_for(measurement, group_id.is_some(), best, tied, policy);
        if group_id.is_some() {
            counts.grouped_photos += 1;
        }
        match decision {
            Decision::Reject => {
                counts.reject += 1;
                suggestions.reject.push(measurement.id);
            }
            Decision::Keep => {
                counts.keep += 1;
                suggestions.keep.push(measurement.id);
            }
            Decision::Pick => {
                counts.pick += 1;
                suggestions.pick.push(measurement.id);
            }
            Decision::Review => counts.review += 1,
            Decision::Abstain => counts.abstain += 1,
        }
        photos.push(PhotoReport {
            id: measurement.id,
            sharpness: measurement.sharpness,
            clipped: measurement.clipped,
            group: group_id,
            best,
            decision,
            reason_codes,
            uncertainty,
        });
    }

    Ok(CullReport { version: REPORT_VERSION, policy: *policy, photos, groups, counts, suggestions })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn measurement(id: u64, secs: i64, sharpness: f32, signature: f32) -> Measurement {
        Measurement { id, captured_secs: Some(secs), sharpness, clipped: 0.0, signature: [signature; 64] }
    }

    #[test]
    fn baseline_groups_with_unique_best_and_picks_only_when_requested() {
        let mut policy = CullPolicy { pick_best: true, ..CullPolicy::default() };
        let photos = vec![measurement(2, 2, 40.0, 0.125), measurement(1, 1, 80.0, 0.125), measurement(3, 30, 60.0, -0.125)];
        let report = plan(&photos, &policy).expect("valid report");
        assert_eq!(report.groups.len(), 1);
        assert_eq!(report.groups.first().and_then(|group| group.best_photo_id), Some(1));
        assert_eq!(report.suggestions.pick, vec![1]);
        assert_eq!(report.suggestions.keep, vec![2]);
        assert!(report.photos.iter().any(|photo| photo.id == 1 && photo.best && photo.decision == Decision::Pick));
        policy.reject_below = Some(50.0);
        let report = plan(&photos, &policy).expect("valid report");
        assert_eq!(report.suggestions.reject, vec![2]);
    }

    #[test]
    fn ties_review_without_best() {
        let policy = CullPolicy { pick_best: true, ..CullPolicy::default() };
        let report = plan(&[measurement(1, 1, 50.0, 0.125), measurement(2, 2, 50.0, 0.125)], &policy).expect("valid report");
        assert_eq!(report.groups.first().and_then(|group| group.best_photo_id), None);
        assert!(report.photos.iter().all(|photo| !photo.best && photo.decision == Decision::Review));
        assert_eq!(report.counts.tied_groups, 1);
    }

    #[test]
    fn rejects_duplicate_nonfinite_and_bad_thresholds() {
        let duplicate = [measurement(1, 1, 50.0, 0.0), measurement(1, 2, 50.0, 0.0)];
        assert!(matches!(plan(&duplicate, &CullPolicy::default()), Err(ReportError::DuplicateId { id: 1 })));

        let mut bad = measurement(1, 1, 50.0, 0.0);
        bad.sharpness = f32::NAN;
        assert!(matches!(plan(&[bad], &CullPolicy::default()), Err(ReportError::NonFiniteMeasurement { .. })));

        let policy = CullPolicy { reject_below: Some(f32::INFINITY), ..CullPolicy::default() };
        assert!(matches!(plan(&[], &policy), Err(ReportError::InvalidPolicy { field: "rejectBelow", .. })));
    }

    #[test]
    fn extreme_capture_times_do_not_overflow_gap_math() {
        let photos = [measurement(1, i64::MIN, 20.0, 0.0), measurement(2, i64::MAX, 21.0, 0.0)];
        let report = plan(&photos, &CullPolicy::default()).expect("valid report");
        assert!(report.groups.is_empty());
    }

    #[test]
    fn signature_serialization_is_exactly_bounded() {
        let photo = measurement(1, 1, 20.0, 0.0);
        let encoded = serde_json::to_value(&photo).expect("serializable measurement");
        let signature = encoded.get("signature").and_then(serde_json::Value::as_array).expect("signature array");
        assert_eq!(signature.len(), 64);

        let mut overlong = signature.clone();
        overlong.push(serde_json::json!(0.0));
        let mut object = encoded.as_object().cloned().expect("measurement object");
        object.insert("signature".to_string(), serde_json::Value::Array(overlong));
        assert!(serde_json::from_value::<Measurement>(serde_json::Value::Object(object)).is_err());
    }
}
