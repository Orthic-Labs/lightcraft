//! Deterministic, offline personal Auto residual suggestions.
//!
//! This module is a pure learner core. It does not read files, mutate catalogs, call providers, or
//! replace `auto_tone`. A model predicts only bounded style residuals for contrast, vibrance, and
//! saturation; callers keep deterministic Auto as explicit fallback.

use std::collections::BTreeSet;
use std::fmt;

use lightcraft_color::luminance_2020;
use lightcraft_color::perceptual::oklab_from_2020;
use lightcraft_develop::{DevelopSettings, controls};
use lightcraft_raster::Rgb32f;
use serde::{Deserialize, Serialize};

/// Serialized model schema. Bump when payload shape or semantics change.
pub const MODEL_SCHEMA: &str = "lightcraft.personal-auto.v1";
/// Receipt schema for an extracted deterministic baseline. The receipt binds decoded proxy pixels
/// to source facts/settings; it is an integrity identity, not proof of source ownership.
pub const BASELINE_RECEIPT_SCHEMA: &str = "lightcraft.personal-auto.baseline-receipt.v1";
/// Bump when deterministic Auto or baseline preparation semantics change.
pub const BASELINE_AUTO_REVISION: &str = "lightcraft.deterministic-auto.v1";
/// Serialized model version.
pub const MODEL_VERSION: u32 = 2;
/// Serialized split-manifest schema.
pub const SPLIT_SCHEMA: &str = "lightcraft.personal-auto-split.v1";
/// Stable feature schema name.
pub const FEATURE_SCHEMA: &str = "lightcraft.personal-auto-features.v1";
/// Number of fixed, camera-identifier-free input features.
pub const FEATURE_COUNT: usize = 16;
/// Number of first-pass style controls.
pub const STYLE_CONTROL_COUNT: usize = 3;
/// Maximum decoded pixels accepted for feature extraction.
pub const MAX_FEATURE_PIXELS: usize = 64_000_000;
/// Fixed coordinate-descent passes. This is part of deterministic model semantics.
pub const RIDGE_PASSES: usize = 64;
/// Small positive ridge penalty. This is part of deterministic model semantics.
pub const RIDGE_LAMBDA: f64 = 1e-3;
/// Bounds for one residual label before final control clamping.
pub const MAX_RESIDUAL: f64 = 200.0;
/// Bound serialized train-shoot metadata.
pub const MAX_TRAIN_SHOOTS: usize = 100_000;
/// Bound serialized train samples; fitting remains offline and bounded.
pub const MAX_TRAIN_SAMPLES: usize = 1_000_000;
/// Bound serialized provenance label-source metadata.
pub const MAX_LABEL_SOURCES: usize = 2;
/// Minimum train shoots for a data-sufficiency report; fitting never enforces this gate.
pub const MIN_ELIGIBLE_TRAIN_SHOOTS: usize = 20;

/// Fixed feature order. Values are finite, train-normalized, and contain no camera identifier.
///
/// EV percentiles and spread, luminance/chroma statistics, scene-tail shares, aspect, raw flag,
/// baseline exposure, and source/WB metadata are intentionally scalar nuisance/scene evidence.
pub const FEATURE_NAMES: [&str; FEATURE_COUNT] = [
    "ev.p01",
    "ev.p05",
    "ev.median",
    "ev.p95",
    "ev.p995",
    "ev.spread",
    "chroma.p90",
    "luminance.mean",
    "luminance.stddev",
    "scene.shadow_share",
    "scene.highlight_share",
    "source.aspect",
    "source.raw",
    "baseline.exposure",
    "source.wb_temp",
    "source.wb_tint",
];

/// First-pass style-only controls. Their current Ember ranges are all −100..100.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum StyleControl {
    Contrast,
    Vibrance,
    Saturation,
}

impl StyleControl {
    /// Stable control order used by labels, regressors, and serialized payloads.
    pub const ALL: [Self; STYLE_CONTROL_COUNT] = [Self::Contrast, Self::Vibrance, Self::Saturation];

    fn id(self) -> &'static str {
        match self {
            Self::Contrast => "light.contrast",
            Self::Vibrance => "color.vibrance",
            Self::Saturation => "color.saturation",
        }
    }

    fn index(self) -> usize {
        match self {
            Self::Contrast => 0,
            Self::Vibrance => 1,
            Self::Saturation => 2,
        }
    }
}

/// Fixed-size feature vector supplied by an offline evaluator.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct FeatureVector(pub [f64; FEATURE_COUNT]);

impl FeatureVector {
    /// Construct a vector after rejecting non-finite values.
    pub fn new(values: [f64; FEATURE_COUNT]) -> Result<Self, PersonalAutoError> {
        let out = Self(values);
        out.validate()?;
        Ok(out)
    }

    /// Validate fixed dimensionality and finite values.
    pub fn validate(&self) -> Result<(), PersonalAutoError> {
        if self.0.iter().all(|v| v.is_finite()) { Ok(()) } else { Err(PersonalAutoError::NonFinite("feature vector".into())) }
    }

    /// Read one feature by fixed index.
    pub fn get(&self, index: usize) -> Option<f64> {
        self.0.get(index).copied()
    }

    /// Extract fixed scene/source evidence from same proxy path used by deterministic Auto.
    ///
    /// `baseline.light.exposure` is recorded as `baseline.exposure`; `SourceInfo` carries decoder
    /// facts but no exposure field. Callers should pass deterministic Auto settings; no catalog,
    /// filesystem, camera identifier, or provider state is consulted.
    pub fn from_pipeline(src: &Rgb32f, info: &crate::SourceInfo, baseline: &DevelopSettings) -> Result<Self, PersonalAutoError> {
        crate::cull::validate_image(src).map_err(|error| match error {
            crate::cull::MeasurementError::NonFinitePixel { .. } => PersonalAutoError::NonFinite("source image".into()),
            _ => PersonalAutoError::Malformed(format!("source image: {error}")),
        })?;
        let pixel_count = src.width.checked_mul(src.height).ok_or_else(|| PersonalAutoError::Malformed("source dimensions overflow".into()))?;
        if pixel_count > MAX_FEATURE_PIXELS {
            return Err(PersonalAutoError::Malformed("source image exceeds feature pixel cap".into()));
        }
        if !baseline.light.exposure.is_finite() || !baseline.wb.temp.is_finite() || !baseline.wb.tint.is_finite() {
            return Err(PersonalAutoError::NonFinite("baseline source settings".into()));
        }

        // Match auto_tone's bounded proxy + WB preparation, while leaving exposure at zero so
        // scene evidence remains comparable across baseline exposure suggestions.
        let mut img = lightcraft_raster::resample::fit(src, 512, 512, lightcraft_raster::resample::Filter::Box);
        let wb_only = DevelopSettings { wb: baseline.wb, ..DevelopSettings::default() };
        crate::local::scene_linear_pre(&mut img, info, &wb_only);

        let mut ev = Vec::with_capacity(img.data.len());
        let mut chroma = Vec::with_capacity(img.data.len());
        let mut luminance = Vec::with_capacity(img.data.len());
        for pixel in &img.data {
            let y = f64::from(luminance_2020(*pixel));
            if !y.is_finite() || y <= 1e-6 {
                continue;
            }
            let value = (y / 0.18).log2();
            if !value.is_finite() {
                continue;
            }
            ev.push(value);
            luminance.push(y);
            let lab = oklab_from_2020(*pixel);
            let c = f64::from(lab[1]).hypot(f64::from(lab[2]));
            if c.is_finite() {
                chroma.push(c);
            }
        }
        if ev.is_empty() || chroma.is_empty() {
            return Err(PersonalAutoError::Malformed("source has no finite scene evidence".into()));
        }
        ev.sort_by(f64::total_cmp);
        chroma.sort_by(f64::total_cmp);
        let p01 = fixed_percentile(&ev, 0.01);
        let p05 = fixed_percentile(&ev, 0.05);
        let median = fixed_percentile(&ev, 0.50);
        let p95 = fixed_percentile(&ev, 0.95);
        let p995 = fixed_percentile(&ev, 0.995);
        let mean = luminance.iter().sum::<f64>() / luminance.len() as f64;
        let variance = luminance.iter().map(|value| (value - mean).powi(2)).sum::<f64>() / luminance.len() as f64;
        // Tail shares use same valid y > 1e-6 population as auto_tone's EV percentiles; an
        // all-black/invalid source is rejected above rather than silently treated as neutral.
        let shadow_share = ev.iter().filter(|value| **value < -4.0).count() as f64 / ev.len() as f64;
        let highlight_share = ev.iter().filter(|value| **value > 2.2).count() as f64 / ev.len() as f64;
        let (wb_temp, wb_tint) = crate::local::effective_wb(info, &wb_only);
        Self::new([
            p01,
            p05,
            median,
            p95,
            p995,
            p95 - p05,
            fixed_percentile(&chroma, 0.90),
            mean,
            variance.sqrt(),
            shadow_share,
            highlight_share,
            src.width as f64 / src.height as f64,
            if info.raw { 1.0 } else { 0.0 },
            baseline.light.exposure,
            wb_temp,
            wb_tint,
        ])
    }
}

fn fixed_percentile(sorted: &[f64], q: f64) -> f64 {
    if sorted.is_empty() {
        return 0.0;
    }
    let q = if q.is_finite() { q.clamp(0.0, 1.0) } else { 0.5 };
    sorted[((sorted.len() - 1) as f64 * q) as usize]
}

/// Style residual vector in [`StyleControl::ALL`] order.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct StyleVector(pub [f64; STYLE_CONTROL_COUNT]);

impl StyleVector {
    pub const ZERO: Self = Self([0.0; STYLE_CONTROL_COUNT]);

    pub fn get(&self, control: StyleControl) -> f64 {
        self.0[control.index()]
    }

    pub fn set(&mut self, control: StyleControl, value: f64) {
        self.0[control.index()] = value;
    }

    pub fn validate(&self) -> Result<(), PersonalAutoError> {
        for value in self.0 {
            if !value.is_finite() {
                return Err(PersonalAutoError::NonFinite("style vector".into()));
            }
        }
        Ok(())
    }
}

/// Origin of one field label. Lightroom values are weak relative-style evidence.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum LabelSource {
    WeakLightroom,
    EmberValidated,
}

/// One field-specific residual label. Missing fields are represented by `None` in [`StyleLabels`].
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FieldLabel {
    /// Target style delta from deterministic baseline, not an absolute slider value.
    pub delta: f64,
    pub source: LabelSource,
    /// Positive finite weight in (0, 1]. It is metadata for weak-label trust & training weight.
    pub confidence: f64,
}

impl FieldLabel {
    pub fn new(delta: f64, source: LabelSource, confidence: f64) -> Result<Self, PersonalAutoError> {
        let out = Self { delta, source, confidence };
        out.validate()?;
        Ok(out)
    }

    fn validate(&self) -> Result<(), PersonalAutoError> {
        if !self.delta.is_finite() || self.delta.abs() > MAX_RESIDUAL {
            return Err(PersonalAutoError::Malformed("label delta outside finite residual bound".into()));
        }
        if !self.confidence.is_finite() || !(0.0..=1.0).contains(&self.confidence) || self.confidence == 0.0 {
            return Err(PersonalAutoError::Malformed("label confidence must be finite and in (0, 1]".into()));
        }
        Ok(())
    }
}

/// Per-control labels; omitted field means no training target for that control.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StyleLabels {
    pub contrast: Option<FieldLabel>,
    pub vibrance: Option<FieldLabel>,
    pub saturation: Option<FieldLabel>,
}

impl StyleLabels {
    pub fn get(&self, control: StyleControl) -> Option<FieldLabel> {
        match control {
            StyleControl::Contrast => self.contrast,
            StyleControl::Vibrance => self.vibrance,
            StyleControl::Saturation => self.saturation,
        }
    }

    pub fn set(&mut self, control: StyleControl, label: Option<FieldLabel>) {
        match control {
            StyleControl::Contrast => self.contrast = label,
            StyleControl::Vibrance => self.vibrance = label,
            StyleControl::Saturation => self.saturation = label,
        }
    }

    fn validate(&self) -> Result<(), PersonalAutoError> {
        for control in StyleControl::ALL {
            if let Some(label) = self.get(control) {
                label.validate()?;
            }
        }
        Ok(())
    }
}

/// Dataset split assigned at whole-shoot granularity.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Ord, PartialOrd, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Split {
    Train,
    Development,
    HeldOut,
}

/// Source provenance for one sample. No camera identity is a model feature.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SampleProvenance {
    pub sample_id: String,
    pub shoot_id: String,
    pub split: Split,
    pub source_ref: String,
}

impl SampleProvenance {
    fn validate(&self) -> Result<(), PersonalAutoError> {
        for (name, value) in [("sample_id", &self.sample_id), ("shoot_id", &self.shoot_id), ("source_ref", &self.source_ref)] {
            if value.is_empty() || value.len() > 512 {
                return Err(PersonalAutoError::Malformed(format!("{name} is empty or too long")));
            }
        }
        Ok(())
    }
}

/// A typed, bounded sample. Labels are residuals relative to its deterministic baseline.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingSample {
    pub features: FeatureVector,
    pub labels: StyleLabels,
    pub provenance: SampleProvenance,
}

impl TrainingSample {
    fn validate(&self) -> Result<(), PersonalAutoError> {
        self.features.validate()?;
        self.labels.validate()?;
        self.provenance.validate()
    }
}

/// One whole-shoot split assignment. A shoot may occur in exactly one split.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ShootAssignment {
    pub shoot_id: String,
    pub split: Split,
}

/// Manifest identity & whole-shoot partition used for training.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SplitManifest {
    pub schema: String,
    pub identity: String,
    pub shoots: Vec<ShootAssignment>,
}

impl SplitManifest {
    pub fn new(identity: impl Into<String>, shoots: Vec<ShootAssignment>) -> Self {
        Self { schema: SPLIT_SCHEMA.into(), identity: identity.into(), shoots }
    }

    /// Reject duplicate shoot assignments and malformed partition metadata.
    pub fn validate(&self) -> Result<(), PersonalAutoError> {
        if self.schema != SPLIT_SCHEMA {
            return Err(PersonalAutoError::SchemaMismatch { expected: SPLIT_SCHEMA.into(), found: self.schema.clone() });
        }
        if self.identity.is_empty() || self.identity.len() > 512 {
            return Err(PersonalAutoError::Malformed("split manifest identity".into()));
        }
        if self.shoots.is_empty() || self.shoots.len() > MAX_TRAIN_SHOOTS {
            return Err(PersonalAutoError::Malformed("split manifest shoot count".into()));
        }
        let mut seen = BTreeSet::new();
        for assignment in &self.shoots {
            if assignment.shoot_id.is_empty() || assignment.shoot_id.len() > 512 || !seen.insert(&assignment.shoot_id) {
                return Err(PersonalAutoError::Leakage("duplicate or malformed shoot assignment".into()));
            }
        }
        Ok(())
    }

    fn split_for(&self, shoot_id: &str) -> Option<Split> {
        self.shoots.iter().find(|assignment| assignment.shoot_id == shoot_id).map(|assignment| assignment.split)
    }
}

/// Fixed train-only normalization statistics.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FeatureNormalization {
    pub mean: [f64; FEATURE_COUNT],
    pub scale: [f64; FEATURE_COUNT],
}

impl FeatureNormalization {
    fn validate(&self) -> Result<(), PersonalAutoError> {
        for (mean, scale) in self.mean.iter().zip(self.scale) {
            if !mean.is_finite() || !scale.is_finite() || scale <= 0.0 {
                return Err(PersonalAutoError::NonFinite("feature normalization".into()));
            }
        }
        Ok(())
    }

    fn apply(&self, input: &FeatureVector) -> Result<[f64; FEATURE_COUNT], PersonalAutoError> {
        input.validate()?;
        self.validate()?;
        let mut out = [0.0; FEATURE_COUNT];
        for i in 0..FEATURE_COUNT {
            let value = (input.0[i] - self.mean[i]) / self.scale[i];
            if !value.is_finite() {
                return Err(PersonalAutoError::NonFinite("normalized feature".into()));
            }
            out[i] = value;
        }
        Ok(out)
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
struct RidgeRegressor {
    /// Intercept at index 0, followed by [`FEATURE_COUNT`] normalized features.
    coefficients: [f64; FEATURE_COUNT + 1],
}

impl RidgeRegressor {
    fn validate(&self) -> Result<(), PersonalAutoError> {
        if self.coefficients.iter().all(|value| value.is_finite()) { Ok(()) } else { Err(PersonalAutoError::NonFinite("ridge coefficients".into())) }
    }

    fn predict(&self, features: &[f64; FEATURE_COUNT]) -> Result<f64, PersonalAutoError> {
        self.validate()?;
        let mut value = self.coefficients[0];
        for i in 0..FEATURE_COUNT {
            value += self.coefficients[i + 1] * features[i];
        }
        value.is_finite().then_some(value).ok_or_else(|| PersonalAutoError::NonFinite("ridge prediction".into()))
    }
}

/// Compact provenance summary embedded in every model.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ModelProvenance {
    pub sample_count: usize,
    pub field_counts: [usize; STYLE_CONTROL_COUNT],
    pub label_sources: Vec<LabelSource>,
    pub status: ModelStatus,
}

/// Models remain experimental until held-out preference and safety criteria pass externally.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum ModelStatus {
    ExperimentalUnqualified,
}

/// Data sufficiency report only; this does not claim scientific validity or rollout readiness.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct TrainingDataEligibility {
    pub train_shoot_count: usize,
    pub sample_count: usize,
    pub field_counts: [usize; STYLE_CONTROL_COUNT],
    pub minimum_train_shoots: usize,
    pub eligible_by_count: bool,
}

/// Serializable personal residual model. Validate after deserialization before prediction.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PersonalAutoModel {
    pub schema: String,
    pub version: u32,
    pub feature_schema: String,
    pub controls: [StyleControl; STYLE_CONTROL_COUNT],
    pub manifest_identity: String,
    pub train_shoots: Vec<String>,
    pub provenance: ModelProvenance,
    normalization: FeatureNormalization,
    regressors: [RidgeRegressor; STYLE_CONTROL_COUNT],
}

impl PersonalAutoModel {
    /// Fit only samples assigned to `Train`; held-out labels are rejected before fitting.
    pub fn fit(samples: &[TrainingSample], manifest: &SplitManifest) -> Result<Self, PersonalAutoError> {
        manifest.validate()?;
        if samples.is_empty() {
            return Err(PersonalAutoError::EmptyTrainingSet);
        }
        if samples.len() > MAX_TRAIN_SAMPLES {
            return Err(PersonalAutoError::Malformed("training sample count".into()));
        }

        let mut ordered: Vec<&TrainingSample> = samples.iter().collect();
        ordered.sort_by(|a, b| a.provenance.sample_id.cmp(&b.provenance.sample_id));
        let mut sample_ids = BTreeSet::new();
        let mut train_shoots = BTreeSet::new();
        for sample in &ordered {
            sample.validate()?;
            if !sample_ids.insert(sample.provenance.sample_id.as_str()) {
                return Err(PersonalAutoError::Leakage("duplicate sample id".into()));
            }
            if sample.provenance.split != Split::Train {
                return Err(PersonalAutoError::Leakage("fit accepts train samples only".into()));
            }
            if manifest.split_for(&sample.provenance.shoot_id) != Some(Split::Train) {
                return Err(PersonalAutoError::Leakage("sample shoot is absent or outside train split".into()));
            }
            train_shoots.insert(sample.provenance.shoot_id.clone());
        }
        if train_shoots.is_empty() {
            return Err(PersonalAutoError::EmptyTrainingSet);
        }

        let normalization = fit_normalization(&ordered)?;
        let mut regressors = [RidgeRegressor { coefficients: [0.0; FEATURE_COUNT + 1] }; STYLE_CONTROL_COUNT];
        let mut field_counts = [0usize; STYLE_CONTROL_COUNT];
        let mut source_set = BTreeSet::new();
        for control in StyleControl::ALL {
            let rows = rows_for(&ordered, &normalization, control)?;
            field_counts[control.index()] = rows.len();
            if rows.is_empty() {
                return Err(PersonalAutoError::NoLabels(control));
            }
            for (_, label, _) in &rows {
                source_set.insert(label.source);
            }
            regressors[control.index()] = fit_ridge(&rows)?;
        }

        let model = Self {
            schema: MODEL_SCHEMA.into(),
            version: MODEL_VERSION,
            feature_schema: FEATURE_SCHEMA.into(),
            controls: StyleControl::ALL,
            manifest_identity: manifest.identity.clone(),
            train_shoots: train_shoots.into_iter().collect(),
            provenance: ModelProvenance {
                sample_count: ordered.len(),
                field_counts,
                label_sources: source_set.into_iter().collect(),
                status: ModelStatus::ExperimentalUnqualified,
            },
            normalization,
            regressors,
        };
        model.validate_against(manifest)?;
        Ok(model)
    }

    /// Alias stating the leakage contract at call sites.
    pub fn fit_train_only(samples: &[TrainingSample], manifest: &SplitManifest) -> Result<Self, PersonalAutoError> {
        Self::fit(samples, manifest)
    }

    /// Validate model schema plus its exact split-manifest binding before prediction.
    pub fn validate_against(&self, manifest: &SplitManifest) -> Result<(), PersonalAutoError> {
        self.validate()?;
        manifest.validate()?;
        if self.manifest_identity != manifest.identity {
            return Err(PersonalAutoError::SchemaMismatch { expected: manifest.identity.clone(), found: self.manifest_identity.clone() });
        }
        let train_shoots: BTreeSet<&str> =
            manifest.shoots.iter().filter(|assignment| assignment.split == Split::Train).map(|assignment| assignment.shoot_id.as_str()).collect();
        if self.train_shoots.iter().any(|shoot| !train_shoots.contains(shoot.as_str())) {
            return Err(PersonalAutoError::Leakage("model train shoots are outside manifest train split".into()));
        }
        Ok(())
    }

    /// Report counts for offline gating; callers must still apply held-out preference/safety criteria.
    pub fn training_data_eligibility(&self) -> TrainingDataEligibility {
        TrainingDataEligibility {
            train_shoot_count: self.train_shoots.len(),
            sample_count: self.provenance.sample_count,
            field_counts: self.provenance.field_counts,
            minimum_train_shoots: MIN_ELIGIBLE_TRAIN_SHOOTS,
            eligible_by_count: self.train_shoots.len() >= MIN_ELIGIBLE_TRAIN_SHOOTS,
        }
    }

    /// Reject malformed or incompatible serialized payloads before use.
    pub fn validate(&self) -> Result<(), PersonalAutoError> {
        if self.schema != MODEL_SCHEMA {
            return Err(PersonalAutoError::SchemaMismatch { expected: MODEL_SCHEMA.into(), found: self.schema.clone() });
        }
        if self.version != MODEL_VERSION || self.feature_schema != FEATURE_SCHEMA || self.controls != StyleControl::ALL {
            return Err(PersonalAutoError::SchemaMismatch {
                expected: format!("{MODEL_SCHEMA}/{MODEL_VERSION}/{FEATURE_SCHEMA}"),
                found: "model payload".into(),
            });
        }
        if self.manifest_identity.is_empty() || self.manifest_identity.len() > 512 {
            return Err(PersonalAutoError::Malformed("model manifest identity".into()));
        }
        if self.train_shoots.is_empty() || self.train_shoots.len() > MAX_TRAIN_SHOOTS {
            return Err(PersonalAutoError::Malformed("model train shoots".into()));
        }
        let mut shoots = BTreeSet::new();
        if self.train_shoots.iter().any(|shoot| shoot.is_empty() || shoot.len() > 512 || !shoots.insert(shoot)) {
            return Err(PersonalAutoError::Leakage("model train shoots are duplicate or malformed".into()));
        }
        if self.provenance.sample_count == 0 || self.provenance.sample_count > MAX_TRAIN_SAMPLES {
            return Err(PersonalAutoError::Malformed("model provenance sample count".into()));
        }
        if self.provenance.field_counts.iter().any(|count| *count == 0 || *count > self.provenance.sample_count) {
            return Err(PersonalAutoError::Malformed("model provenance counts".into()));
        }
        if self.train_shoots.len() > self.provenance.sample_count {
            return Err(PersonalAutoError::Malformed("model train shoots exceed sample count".into()));
        }
        if self.provenance.label_sources.is_empty() || self.provenance.label_sources.len() > MAX_LABEL_SOURCES {
            return Err(PersonalAutoError::Malformed("model provenance sources".into()));
        }
        if self.provenance.status != ModelStatus::ExperimentalUnqualified {
            return Err(PersonalAutoError::Malformed("model status".into()));
        }
        for pair in self.provenance.label_sources.windows(2) {
            if pair[0] >= pair[1] {
                return Err(PersonalAutoError::Malformed("model provenance source order".into()));
            }
        }
        self.normalization.validate()?;
        for regressor in self.regressors {
            regressor.validate()?;
        }
        Ok(())
    }

    /// Predict bounded style suggestions; invalid input/model returns untouched baseline explicitly.
    pub fn predict(&self, baseline: &DevelopSettings, features: &FeatureVector) -> Prediction {
        if let Err(error) = self.validate() {
            return Prediction::fallback(baseline, FallbackReason::InvalidModel(error.to_string()));
        }
        if let Err(error) = features.validate() {
            return Prediction::fallback(baseline, FallbackReason::InvalidFeatures(error.to_string()));
        }
        let normalized = match self.normalization.apply(features) {
            Ok(values) => values,
            Err(error) => return Prediction::fallback(baseline, FallbackReason::InvalidFeatures(error.to_string())),
        };
        let mut settings = baseline.clone();
        let mut applied = StyleVector::ZERO;
        for control in StyleControl::ALL {
            let Some(current) = controls::get(baseline, control.id()) else {
                return Prediction::fallback(baseline, FallbackReason::InvalidModel("style control missing".into()));
            };
            if !current.is_finite() {
                return Prediction::fallback(baseline, FallbackReason::InvalidBaseline);
            }
            let raw = match self.regressors[control.index()].predict(&normalized) {
                Ok(value) => value.clamp(-MAX_RESIDUAL, MAX_RESIDUAL),
                Err(error) => return Prediction::fallback(baseline, FallbackReason::InvalidModel(error.to_string())),
            };
            let target = current + raw;
            if !target.is_finite() {
                return Prediction::fallback(baseline, FallbackReason::InvalidModel("non-finite style target".into()));
            }
            let bounded = target.clamp(-100.0, 100.0);
            if !controls::set(&mut settings, control.id(), bounded) {
                return Prediction::fallback(baseline, FallbackReason::InvalidModel("style control rejected".into()));
            }
            let actual = bounded - current;
            if !actual.is_finite() {
                return Prediction::fallback(baseline, FallbackReason::InvalidModel("non-finite applied residual".into()));
            }
            applied.set(control, actual);
        }
        Prediction { settings, residual: applied, used_model: true, fallback: None }
    }
}

/// Explicit fallback reason; there is no hidden quality gate.
#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum FallbackReason {
    InvalidModel(String),
    InvalidFeatures(String),
    InvalidBaseline,
}

/// Prediction result. `settings` is untouched baseline whenever `used_model` is false.
#[derive(Clone, Debug, PartialEq)]
pub struct Prediction {
    pub settings: DevelopSettings,
    pub residual: StyleVector,
    pub used_model: bool,
    pub fallback: Option<FallbackReason>,
}

impl Prediction {
    fn fallback(baseline: &DevelopSettings, reason: FallbackReason) -> Self {
        Self { settings: baseline.clone(), residual: StyleVector::ZERO, used_model: false, fallback: Some(reason) }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PersonalAutoError {
    EmptyTrainingSet,
    NoLabels(StyleControl),
    NonFinite(String),
    Malformed(String),
    Leakage(String),
    SchemaMismatch { expected: String, found: String },
}

impl fmt::Display for PersonalAutoError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyTrainingSet => f.write_str("personal Auto training set is empty"),
            Self::NoLabels(control) => write!(f, "no labels for {control:?}"),
            Self::NonFinite(what) => write!(f, "non-finite {what}"),
            Self::Malformed(what) => write!(f, "malformed {what}"),
            Self::Leakage(what) => write!(f, "training leakage: {what}"),
            Self::SchemaMismatch { expected, found } => write!(f, "schema mismatch: expected {expected}, found {found}"),
        }
    }
}

impl std::error::Error for PersonalAutoError {}

fn fit_normalization(samples: &[&TrainingSample]) -> Result<FeatureNormalization, PersonalAutoError> {
    let mut mean = [0.0; FEATURE_COUNT];
    for sample in samples {
        for i in 0..FEATURE_COUNT {
            mean[i] += sample.features.0[i];
        }
    }
    let count = samples.len() as f64;
    for value in &mut mean {
        *value /= count;
    }
    let mut variance = [0.0; FEATURE_COUNT];
    for sample in samples {
        for i in 0..FEATURE_COUNT {
            let delta = sample.features.0[i] - mean[i];
            variance[i] += delta * delta;
        }
    }
    let mut scale = [1.0; FEATURE_COUNT];
    for i in 0..FEATURE_COUNT {
        let candidate = (variance[i] / count).sqrt();
        if candidate.is_finite() && candidate > 1e-12 {
            scale[i] = candidate;
        }
    }
    let normalization = FeatureNormalization { mean, scale };
    normalization.validate()?;
    Ok(normalization)
}

type Row = ([f64; FEATURE_COUNT], FieldLabel, f64);

fn rows_for(samples: &[&TrainingSample], normalization: &FeatureNormalization, control: StyleControl) -> Result<Vec<Row>, PersonalAutoError> {
    let mut rows = Vec::new();
    for sample in samples {
        let Some(label) = sample.labels.get(control) else { continue };
        label.validate()?;
        let features = normalization.apply(&sample.features)?;
        rows.push((features, label, label.confidence));
    }
    Ok(rows)
}

fn fit_ridge(rows: &[Row]) -> Result<RidgeRegressor, PersonalAutoError> {
    if rows.is_empty() {
        return Err(PersonalAutoError::EmptyTrainingSet);
    }
    let mut coefficients = [0.0; FEATURE_COUNT + 1];
    for _ in 0..RIDGE_PASSES {
        for j in 0..=FEATURE_COUNT {
            let mut rho = 0.0;
            let mut norm = 0.0;
            for (features, label, weight) in rows {
                let x = if j == 0 { 1.0 } else { features[j - 1] };
                let mut prediction = coefficients[0];
                for k in 0..=FEATURE_COUNT {
                    if k != j {
                        let xk = if k == 0 { 1.0 } else { features[k - 1] };
                        prediction += coefficients[k] * xk;
                    }
                }
                let weighted = *weight;
                rho += weighted * x * (label.delta - prediction);
                norm += weighted * x * x;
            }
            let penalty = if j == 0 { 0.0 } else { RIDGE_LAMBDA };
            let denominator = norm + penalty;
            if !denominator.is_finite() || denominator <= 0.0 {
                return Err(PersonalAutoError::NonFinite("ridge denominator".into()));
            }
            coefficients[j] = rho / denominator;
            if !coefficients[j].is_finite() {
                return Err(PersonalAutoError::NonFinite("ridge coefficient".into()));
            }
        }
    }
    let regressor = RidgeRegressor { coefficients };
    regressor.validate()?;
    Ok(regressor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn features(v: f64) -> FeatureVector {
        FeatureVector::new([v; FEATURE_COUNT]).unwrap()
    }

    fn manifest() -> SplitManifest {
        SplitManifest::new(
            "manifest-1",
            vec![
                ShootAssignment { shoot_id: "train-a".into(), split: Split::Train },
                ShootAssignment { shoot_id: "heldout-a".into(), split: Split::HeldOut },
            ],
        )
    }

    fn sample(id: &str, shoot: &str, labels: StyleLabels) -> TrainingSample {
        TrainingSample {
            features: features(id.bytes().map(f64::from).sum::<f64>() / 100.0),
            labels,
            provenance: SampleProvenance { sample_id: id.into(), shoot_id: shoot.into(), split: Split::Train, source_ref: format!("source-{id}") },
        }
    }

    fn labels(delta: f64) -> StyleLabels {
        let label = |value| FieldLabel::new(value, LabelSource::EmberValidated, 1.0).unwrap();
        StyleLabels { contrast: Some(label(delta)), vibrance: Some(label(delta / 2.0)), saturation: Some(label(-delta / 2.0)) }
    }

    #[test]
    fn rejects_nonfinite_features_and_labels() {
        assert!(FeatureVector::new([f64::NAN; FEATURE_COUNT]).is_err());
        assert!(FieldLabel::new(f64::INFINITY, LabelSource::EmberValidated, 1.0).is_err());
        assert!(FieldLabel::new(1.0, LabelSource::EmberValidated, 0.0).is_err());
    }

    #[test]
    fn rejects_whole_shoot_leakage_and_nontrain_labels() {
        let mut heldout = sample("heldout", "heldout-a", labels(1.0));
        heldout.provenance.split = Split::HeldOut;
        assert!(matches!(PersonalAutoModel::fit(&[heldout], &manifest()), Err(PersonalAutoError::Leakage(_))));
        let mut bad_manifest = manifest();
        bad_manifest.shoots.push(ShootAssignment { shoot_id: "train-a".into(), split: Split::HeldOut });
        assert!(bad_manifest.validate().is_err());
    }

    #[test]
    fn missing_field_label_is_excluded_not_zero() {
        let mut only_contrast = labels(2.0);
        only_contrast.vibrance = None;
        only_contrast.saturation = None;
        let err = PersonalAutoModel::fit(&[sample("one", "train-a", only_contrast)], &manifest()).unwrap_err();
        assert_eq!(err, PersonalAutoError::NoLabels(StyleControl::Vibrance));
    }

    #[test]
    fn fit_is_order_independent_and_prediction_is_bounded_style_only() {
        let a = sample("a", "train-a", labels(12.0));
        let b = sample("b", "train-a", labels(-6.0));
        let one = PersonalAutoModel::fit(&[a.clone(), b.clone()], &manifest()).unwrap();
        let two = PersonalAutoModel::fit(&[b, a], &manifest()).unwrap();
        assert_eq!(one, two);
        let mut baseline = DevelopSettings::default();
        baseline.light.exposure = 1.25;
        baseline.effects.clarity = 31.0;
        let prediction = one.predict(&baseline, &features(0.4));
        assert!(prediction.used_model);
        assert_eq!(prediction.settings.light.exposure, 1.25);
        assert_eq!(prediction.settings.effects.clarity, 31.0);
        assert!((-100.0..=100.0).contains(&prediction.settings.light.contrast));
        assert!((-100.0..=100.0).contains(&prediction.settings.color.vibrance));
        assert!((-100.0..=100.0).contains(&prediction.settings.color.saturation));
    }

    #[test]
    fn invalid_model_or_features_return_untouched_baseline() {
        let model = PersonalAutoModel::fit(&[sample("one", "train-a", labels(2.0))], &manifest()).unwrap();
        let mut baseline = DevelopSettings::default();
        baseline.light.exposure = 0.75;
        let mut invalid = model.clone();
        invalid.schema = "other".into();
        let fallback = invalid.predict(&baseline, &features(0.0));
        assert!(!fallback.used_model);
        assert_eq!(fallback.settings, baseline);
        let bad_features = FeatureVector([f64::NAN; FEATURE_COUNT]);
        let fallback = model.predict(&baseline, &bad_features);
        assert!(!fallback.used_model);
        assert_eq!(fallback.settings, baseline);
    }
}
