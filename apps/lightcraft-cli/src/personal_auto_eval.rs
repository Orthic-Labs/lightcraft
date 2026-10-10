//! Offline, metadata-free Personal Auto training & evaluation.
//!
//! Inputs are explicit numeric feature/setting manifests. This module never opens a
//! catalog, decodes a photo, renders an image, calls a provider, or applies a setting.

use std::collections::{BTreeMap, BTreeSet};
use std::fs::{File, OpenOptions};
use std::io::{Read, Write};
use std::path::Path;

use lightcraft_develop::{DevelopSettings, controls};
use lightcraft_pipeline::personal_auto::{
    FEATURE_COUNT, FEATURE_NAMES, FeatureVector, FieldLabel, LabelSource, MODEL_SCHEMA as CORE_MODEL_SCHEMA, PersonalAutoModel, ShootAssignment,
    Split, SplitManifest, StyleControl, StyleLabels,
};
use serde_json::{Map, Value, json};

const VERSION: u64 = 1;
const MODEL_SCHEMA: &str = "lightcraft.personal-auto-eval.model.v1";
const REPORT_SCHEMA: &str = "lightcraft.personal-auto.report.v1";
const MAX_BYTES: u64 = 16 * 1024 * 1024;
const MAX_SHOOTS: usize = 100_000;
const MAX_PHOTOS: usize = 1_000_000;
const MAX_FEATURES: usize = 64;
const MAX_CONTROLS: usize = 32;
const STYLE_IDS: [&str; 3] = ["light.contrast", "color.vibrance", "color.saturation"];

#[derive(Clone, Debug)]
struct Manifest {
    features: Vec<String>,
    controls: Vec<String>,
    bounds: BTreeMap<String, (f64, f64)>,
    shoots: Vec<Shoot>,
}

#[derive(Clone, Debug)]
struct Shoot {
    id: String,
    split: String,
    camera: String,
    photos: Vec<Photo>,
}

#[derive(Clone, Debug)]
struct Photo {
    id: String,
    features: Vec<f64>,
    baseline: BTreeMap<String, f64>,
    weak: Option<Target>,
    ember: Option<Target>,
}

#[derive(Clone, Debug)]
struct Target {
    values: BTreeMap<String, f64>,
    confidence: BTreeMap<String, f64>,
    deltas: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Variant {
    Weak,
    Ember,
}

impl Variant {
    fn key(self) -> &'static str {
        match self {
            Self::Weak => "weakLabelColdStart",
            Self::Ember => "emberGroundTruth",
        }
    }

    fn target(self, photo: &Photo) -> Option<&Target> {
        match self {
            Self::Weak => photo.weak.as_ref(),
            Self::Ember => photo.ember.as_ref(),
        }
    }
}

pub fn run(args: &[String]) -> Result<(), String> {
    let (mode, offset) = if args.first().map(String::as_str) == Some("personal") {
        (args.get(1).map(String::as_str), 2)
    } else {
        (args.first().map(String::as_str), 1)
    };
    match mode {
        Some("train") => train_command(&args[offset..]),
        Some("evaluate") => evaluate_command(&args[offset..]),
        Some("--help") | Some("-h") => {
            println!("ai personal train --manifest FILE --out NEW_MODEL");
            println!("ai personal evaluate --manifest FILE --model FILE [--variant weakLabelColdStart|emberGroundTruth|both] [--out NEW_REPORT]");
            Ok(())
        }
        _ => Err("ai personal expects train or evaluate; see --help".into()),
    }
}

fn train_command(args: &[String]) -> Result<(), String> {
    let (manifest_path, output_path) = parse_train_args(args)?;
    let (value, digest) = read_json(Path::new(&manifest_path), "manifest")?;
    let manifest = parse_manifest(&value)?;
    let model = train_model(&manifest, &digest)?;
    write_new_json(Path::new(&output_path), &model, "model")?;
    println!("{}", json!({"model": output_path, "schema": MODEL_SCHEMA, "version": VERSION}));
    Ok(())
}

fn evaluate_command(args: &[String]) -> Result<(), String> {
    let (manifest_path, model_path, variant, output_path) = parse_evaluate_args(args)?;
    let (manifest_value, digest) = read_json(Path::new(&manifest_path), "manifest")?;
    let (model, _) = read_json(Path::new(&model_path), "model")?;
    let manifest = parse_manifest(&manifest_value)?;
    let report = evaluate_model(&manifest, &model, &digest, variant)?;
    if let Some(path) = output_path {
        write_new_json(Path::new(&path), &report, "report")?;
        println!("{}", json!({"report": path, "schema": REPORT_SCHEMA, "version": VERSION}));
    } else {
        let bytes = serde_json::to_vec_pretty(&report).map_err(|_| "could not encode evaluation report")?;
        println!("{}", String::from_utf8_lossy(&bytes));
    }
    Ok(())
}

fn parse_train_args(args: &[String]) -> Result<(String, String), String> {
    let mut manifest = None;
    let mut out = None;
    let mut i = 0usize;
    while i < args.len() {
        match args.get(i).map(String::as_str).unwrap_or("") {
            "--manifest" => manifest = Some(next_arg(args, &mut i, "--manifest")?),
            "--out" | "-o" => out = Some(next_arg(args, &mut i, "--out")?),
            "--help" | "-h" => return Err("ai personal train --manifest FILE --out NEW_MODEL".into()),
            value if value.starts_with('-') => return Err(format!("unknown personal option {value}")),
            _ => return Err("personal train accepts named options only".into()),
        }
        i = i.saturating_add(1);
    }
    Ok((manifest.ok_or("personal train requires --manifest")?, out.ok_or("personal train requires --out NEW_MODEL")?))
}

fn parse_evaluate_args(args: &[String]) -> Result<(String, String, Option<Variant>, Option<String>), String> {
    let mut manifest = None;
    let mut model = None;
    let mut variant = None;
    let mut out = None;
    let mut i = 0usize;
    while i < args.len() {
        match args.get(i).map(String::as_str).unwrap_or("") {
            "--manifest" => manifest = Some(next_arg(args, &mut i, "--manifest")?),
            "--model" => model = Some(next_arg(args, &mut i, "--model")?),
            "--variant" => {
                let value = next_arg(args, &mut i, "--variant")?;
                variant = match value.as_str() {
                    "weakLabelColdStart" | "weaklabelcoldstart" | "weak" => Some(Variant::Weak),
                    "emberGroundTruth" | "embergroundtruth" | "ember" => Some(Variant::Ember),
                    "both" => None,
                    _ => return Err("--variant expects weakLabelColdStart, emberGroundTruth or both".into()),
                };
            }
            "--out" | "-o" => out = Some(next_arg(args, &mut i, "--out")?),
            "--help" | "-h" => return Err("ai personal evaluate --manifest FILE --model FILE [--variant VARIANT] [--out NEW_REPORT]".into()),
            value if value.starts_with('-') => return Err(format!("unknown personal option {value}")),
            _ => return Err("personal evaluate accepts named options only".into()),
        }
        i = i.saturating_add(1);
    }
    Ok((manifest.ok_or("personal evaluate requires --manifest")?, model.ok_or("personal evaluate requires --model")?, variant, out))
}

fn next_arg(args: &[String], index: &mut usize, option: &str) -> Result<String, String> {
    *index = index.saturating_add(1);
    args.get(*index).filter(|value| !value.starts_with('-')).cloned().ok_or_else(|| format!("{option}: missing value"))
}

fn read_json(path: &Path, kind: &str) -> Result<(Value, String), String> {
    let file = File::open(path).map_err(|_| format!("could not read {kind} input"))?;
    let mut bytes = Vec::new();
    file.take(MAX_BYTES.saturating_add(1)).read_to_end(&mut bytes).map_err(|_| format!("could not read {kind} input"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!("{kind} input exceeds {MAX_BYTES} byte cap"));
    }
    let digest = lightcraft_photo_ai::digest(&bytes);
    let value = serde_json::from_slice(&bytes).map_err(|_| format!("{kind} input is not valid JSON"))?;
    Ok((value, digest))
}

fn write_new_json(path: &Path, value: &Value, kind: &str) -> Result<(), String> {
    let bytes = serde_json::to_vec_pretty(value).map_err(|_| format!("could not encode {kind}"))?;
    if bytes.len() as u64 > MAX_BYTES {
        return Err(format!("{kind} output exceeds {MAX_BYTES} byte cap"));
    }
    let mut file = OpenOptions::new().write(true).create_new(true).open(path).map_err(|_| format!("could not create new {kind} output"))?;
    file.write_all(&bytes).map_err(|_| format!("could not write {kind} output"))?;
    Ok(())
}

fn parse_manifest(value: &Value) -> Result<Manifest, String> {
    let object = value.as_object().ok_or("personal manifest root must be an object")?;
    if object.get("version").and_then(Value::as_u64) != Some(1) {
        return Err("personal manifest requires version 1".into());
    }
    let features = string_array(object, &["feature_schema", "featureNames", "feature_names"])?;
    let canonical_features = FEATURE_NAMES.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    if features != canonical_features || features.is_empty() || features.len() > MAX_FEATURES || !unique_strings(&features) {
        return Err("personal manifest has invalid feature schema".into());
    }
    let controls = string_array(object, &["controls", "eligible_controls"])?;
    let canonical_controls = STYLE_IDS.iter().map(|name| (*name).to_owned()).collect::<Vec<_>>();
    if controls != canonical_controls || controls.is_empty() || controls.len() > MAX_CONTROLS || !unique_strings(&controls) {
        return Err("personal manifest has invalid control schema".into());
    }
    for control in &controls {
        if !eligible_control(control) {
            return Err("personal manifest includes excluded scene, mask or geometry control".into());
        }
    }
    let bounds = parse_bounds(object.get("control_bounds"), &controls)?;
    let raw_shoots = object.get("shoots").and_then(Value::as_array).ok_or("personal manifest requires shoots")?;
    if raw_shoots.is_empty() || raw_shoots.len() > MAX_SHOOTS {
        return Err("personal manifest shoot count is outside bounds".into());
    }
    let mut shoots = Vec::new();
    let mut shoot_ids = BTreeSet::new();
    let mut photo_ids = BTreeSet::new();
    let mut photo_count = 0usize;
    for raw in raw_shoots {
        let shoot_object = raw.as_object().ok_or("personal shoot must be an object")?;
        let id = text(shoot_object, &["shoot_id", "shootId", "id"]).ok_or("personal shoot is missing shoot_id")?;
        valid_id(&id, "shoot_id")?;
        if !shoot_ids.insert(id.clone()) {
            return Err("duplicate shoot_id".into());
        }
        let split = text(shoot_object, &["split", "partition"]).ok_or("personal shoot is missing split")?;
        if !matches!(split.as_str(), "train" | "validation" | "eval" | "test") {
            return Err("personal shoot has invalid split".into());
        }
        let camera = camera_key(shoot_object.get("camera").or_else(|| shoot_object.get("camera_model")));
        let raw_photos =
            shoot_object.get("photos").or_else(|| shoot_object.get("frames")).and_then(Value::as_array).ok_or("personal shoot is missing photos")?;
        if raw_photos.is_empty() {
            return Err("personal shoot has no photos".into());
        }
        let mut photos = Vec::new();
        for raw_photo in raw_photos {
            photo_count = photo_count.saturating_add(1);
            if photo_count > MAX_PHOTOS {
                return Err("personal manifest photo count exceeds cap".into());
            }
            let photo_object = raw_photo.as_object().ok_or("personal photo must be an object")?;
            let photo_id = text(photo_object, &["photo_id", "photoId", "id"]).ok_or("personal photo is missing photo_id")?;
            valid_id(&photo_id, "photo_id")?;
            if !photo_ids.insert(photo_id.clone()) {
                return Err("duplicate photo_id across manifest".into());
            }
            let values = photo_object.get("features").ok_or("personal photo is missing numeric features")?;
            let photo_features = parse_features(values, &features)?;
            let baseline_value = photo_object
                .get("baseline")
                .or_else(|| photo_object.get("deterministicbaseline"))
                .or_else(|| photo_object.get("deterministicBaseline"))
                .ok_or("personal photo is missing baseline")?;
            let baseline = baseline_map(baseline_value, &controls)?;
            let weak = parse_target(
                photo_object
                    .get("weakLabelColdStart")
                    .or_else(|| photo_object.get("weaklabelcoldstart"))
                    .or_else(|| photo_object.get("weak_label_cold_start")),
                &controls,
                false,
            )
            .map_err(|error| format!("{photo_id}: weakLabelColdStart {error}"))?;
            let ember = parse_target(
                photo_object
                    .get("emberGroundTruth")
                    .or_else(|| photo_object.get("embergroundtruth"))
                    .or_else(|| photo_object.get("ember_ground_truth")),
                &controls,
                true,
            )
            .map_err(|error| format!("{photo_id}: emberGroundTruth {error}"))?;
            photos.push(Photo { id: photo_id, features: photo_features, baseline, weak, ember });
        }
        shoots.push(Shoot { id, split, camera, photos });
    }
    Ok(Manifest { features, controls, bounds, shoots })
}

fn string_array(object: &Map<String, Value>, keys: &[&str]) -> Result<Vec<String>, String> {
    let value = keys.iter().find_map(|key| object.get(*key)).ok_or("personal manifest is missing schema array")?;
    let values = value.as_array().ok_or("personal schema must be an array")?;
    let mut result = Vec::new();
    for value in values {
        let text = value.as_str().ok_or("personal schema names must be strings")?.trim();
        if text.is_empty() || text.len() > 128 || text.chars().any(char::is_control) {
            return Err("personal schema name is invalid".into());
        }
        result.push(text.to_owned());
    }
    Ok(result)
}

fn unique_strings(values: &[String]) -> bool {
    values.iter().collect::<BTreeSet<_>>().len() == values.len()
}

fn parse_bounds(value: Option<&Value>, controls: &[String]) -> Result<BTreeMap<String, (f64, f64)>, String> {
    let mut bounds = BTreeMap::new();
    let object = value.and_then(Value::as_object);
    for control in controls {
        let pair = object
            .and_then(|map| map.get(control))
            .map(|value| {
                if let Some(array) = value.as_array() {
                    let low = array.first().and_then(Value::as_f64).ok_or("control bound is invalid")?;
                    let high = array.get(1).and_then(Value::as_f64).ok_or("control bound is invalid")?;
                    Ok((low, high))
                } else if let Some(map) = value.as_object() {
                    let low = map.get("min").and_then(Value::as_f64).ok_or("control bound is invalid")?;
                    let high = map.get("max").and_then(Value::as_f64).ok_or("control bound is invalid")?;
                    Ok((low, high))
                } else {
                    Err("control bound is invalid")
                }
            })
            .transpose()?
            .unwrap_or((-100.0, 100.0));
        if !pair.0.is_finite() || !pair.1.is_finite() || pair.0 >= pair.1 {
            return Err("control bound is invalid".into());
        }
        bounds.insert(control.clone(), pair);
    }
    Ok(bounds)
}

fn parse_features(value: &Value, names: &[String]) -> Result<Vec<f64>, String> {
    let mut result = Vec::with_capacity(names.len());
    if let Some(array) = value.as_array() {
        if array.len() != names.len() {
            return Err("personal feature vector has wrong length".into());
        }
        for value in array {
            result.push(finite_number(value, "feature")?);
        }
    } else {
        let object = value.as_object().ok_or("personal features must be numeric object or array")?;
        for name in names {
            result.push(finite_number(object.get(name).ok_or("personal feature is missing")?, "feature")?);
        }
        if object.keys().any(|name| !names.contains(name)) {
            return Err("personal features contain unknown name".into());
        }
    }
    Ok(result)
}

fn numeric_map(value: &Value, controls: &[String], kind: &str) -> Result<BTreeMap<String, f64>, String> {
    let object = value.as_object().ok_or_else(|| format!("{kind} must be a numeric object"))?;
    let mut result = BTreeMap::new();
    for control in controls {
        let number = finite_number(object.get(control).ok_or_else(|| format!("{kind} is missing control"))?, kind)?;
        result.insert(control.clone(), number);
    }
    if object.keys().any(|name| !controls.contains(name)) {
        return Err(format!("{kind} contains unknown control"));
    }
    Ok(result)
}

fn baseline_map(value: &Value, controls: &[String]) -> Result<BTreeMap<String, f64>, String> {
    let object = value.as_object().ok_or("baseline must be a numeric object")?;
    if let Some(values) = object.get("values").or_else(|| object.get("settings")) {
        return numeric_map(values, controls, "baseline");
    }
    if object.contains_key("provenance") {
        let mut values = object.clone();
        values.remove("provenance");
        return numeric_map(&Value::Object(values), controls, "baseline");
    }
    numeric_map(value, controls, "baseline")
}

fn parse_target(value: Option<&Value>, controls: &[String], ember: bool) -> Result<Option<Target>, String> {
    let Some(value) = value else { return Ok(None) };
    let object = value.as_object().ok_or("personal label must be an object")?;
    let provenance = object.get("provenance").ok_or("personal label requires provenance")?;
    if ember && !valid_ember_provenance(provenance) {
        return Err("emberGroundTruth requires accepted human-edit provenance".into());
    }
    if !ember && !valid_weak_provenance(provenance) {
        return Err("weakLabelColdStart requires mapped-source provenance".into());
    }
    let deltas = object.get("deltas").is_some();
    let values_value =
        object.get("deltas").or_else(|| object.get("values")).or_else(|| object.get("settings")).ok_or("personal label requires values or deltas")?;
    let values = partial_numeric_map(values_value, controls)?;
    let confidence = partial_confidence(object.get("confidence").ok_or("personal label requires confidence for every target field")?, controls)?;
    if values.keys().any(|control| !confidence.contains_key(control)) {
        return Err("personal label requires confidence for every target field".into());
    }
    Ok(Some(Target { values, confidence, deltas }))
}

fn partial_numeric_map(value: &Value, controls: &[String]) -> Result<BTreeMap<String, f64>, String> {
    let object = value.as_object().ok_or("personal label values must be a numeric object")?;
    let mut result = BTreeMap::new();
    for (key, value) in object {
        if !controls.contains(key) {
            return Err("personal label contains unknown control".into());
        }
        let number = finite_number(value, "target")?;
        result.insert(key.clone(), number);
    }
    if result.is_empty() { Err("personal label values are empty".into()) } else { Ok(result) }
}

fn partial_confidence(value: &Value, controls: &[String]) -> Result<BTreeMap<String, f64>, String> {
    let object = value.as_object().ok_or("personal confidence must be a numeric object")?;
    let mut result = BTreeMap::new();
    for (key, value) in object {
        if !controls.contains(key) {
            return Err("personal confidence contains unknown control".into());
        }
        let number = finite_number(value, "confidence")?;
        if !(0.0..=1.0).contains(&number) || number == 0.0 {
            return Err("personal confidence must be between 0 and 1".into());
        }
        result.insert(key.clone(), number);
    }
    Ok(result)
}

fn valid_ember_provenance(value: &Value) -> bool {
    let Some(object) = value.as_object() else { return false };
    if object.get("accepted").and_then(Value::as_bool) != Some(true) {
        return false;
    }
    let Ok(text) = serde_json::to_string(value) else { return false };
    let text = text.to_ascii_lowercase();
    let human_edit = ["origin", "source", "edit", "editor", "accepted_by", "acceptedBy"]
        .iter()
        .filter_map(|key| object.get(*key).and_then(Value::as_str).map(|value| (*key, value)))
        .any(|(key, value)| {
            let lower = value.to_ascii_lowercase();
            ((lower.contains("human") || lower.contains("ember") || lower.contains("synthetic")) && lower.contains("edit"))
                || ((key == "editor" || key == "accepted_by" || key == "acceptedBy") && lower.contains("human"))
        });
    human_edit && !["auto", "copied", "synced", "imported", "automatic"].iter().any(|word| text.contains(word))
}

fn valid_weak_provenance(value: &Value) -> bool {
    let Some(object) = value.as_object() else { return false };
    ["source", "origin", "labelSource", "label_source"]
        .iter()
        .find_map(|key| object.get(*key).and_then(Value::as_str))
        .map(|value| !value.trim().is_empty())
        .unwrap_or(false)
}

fn finite_number(value: &Value, field: &str) -> Result<f64, String> {
    let number = value.as_f64().ok_or_else(|| format!("{field} must be numeric"))?;
    if !number.is_finite() {
        return Err(format!("{field} must be finite"));
    }
    Ok(number)
}

fn eligible_control(name: &str) -> bool {
    let lower = name.to_ascii_lowercase();
    let excluded = [
        "mask",
        "spot",
        "crop",
        "geometry",
        "lens",
        "calibration",
        "profile",
        "process",
        "exposure",
        "highlights",
        "shadows",
        "whites",
        "blacks",
        "temperature",
        "tint",
        "whitebalance",
        "wb",
    ];
    if excluded.iter().any(|word| lower.contains(word)) {
        return false;
    }
    ["contrast", "vibrance", "saturation", "texture", "clarity", "dehaze"].iter().any(|word| lower.ends_with(word))
}

fn valid_id(id: &str, field: &str) -> Result<(), String> {
    if id.is_empty() || id.len() > 256 || id.chars().any(char::is_control) {
        return Err(format!("invalid {field}"));
    }
    Ok(())
}

fn text(object: &Map<String, Value>, keys: &[&str]) -> Option<String> {
    keys.iter().find_map(|key| object.get(*key).and_then(Value::as_str).map(str::trim).filter(|value| !value.is_empty()).map(str::to_owned))
}

fn camera_key(value: Option<&Value>) -> String {
    match value {
        Some(Value::String(value)) if !value.trim().is_empty() => value.trim().to_owned(),
        Some(Value::Object(object)) => {
            let make = text(object, &["make", "manufacturer"]).unwrap_or_else(|| "unknown".into());
            let model = text(object, &["model", "camera_model"]).unwrap_or_else(|| "unknown".into());
            format!("{make}/{model}")
        }
        _ => "unknown".into(),
    }
}

fn core_split_manifest(manifest: &Manifest, identity: &str) -> SplitManifest {
    let shoots = manifest
        .shoots
        .iter()
        .map(|shoot| ShootAssignment {
            shoot_id: shoot.id.clone(),
            split: match shoot.split.as_str() {
                "train" => Split::Train,
                "validation" => Split::Development,
                "eval" | "test" => Split::HeldOut,
                _ => Split::HeldOut,
            },
        })
        .collect();
    SplitManifest::new(identity, shoots)
}

fn core_sample(shoot: &Shoot, photo: &Photo, variant: Variant) -> Result<Option<lightcraft_pipeline::personal_auto::TrainingSample>, String> {
    let Some(target) = variant.target(photo) else { return Ok(None) };
    let values: [f64; FEATURE_COUNT] = photo.features.clone().try_into().map_err(|_| format!("{}: feature vector has wrong length", photo.id))?;
    let features = FeatureVector::new(values).map_err(|_| format!("{}: feature vector is non-finite", photo.id))?;
    let mut labels = StyleLabels::default();
    for (index, control) in STYLE_IDS.iter().enumerate() {
        let Some(value) = target.values.get(*control) else { continue };
        let confidence = target.confidence.get(*control).copied().unwrap_or(1.0);
        if confidence <= 0.0 {
            continue;
        }
        let delta = if target.deltas { *value } else { *value - photo.baseline[*control] };
        let source = match variant {
            Variant::Weak => LabelSource::WeakLightroom,
            Variant::Ember => LabelSource::EmberValidated,
        };
        let label =
            FieldLabel::new(delta, source, confidence).map_err(|_| format!("{}: label residual or confidence is outside bounds", photo.id))?;
        labels.set(StyleControl::ALL[index], Some(label));
    }
    Ok(Some(lightcraft_pipeline::personal_auto::TrainingSample {
        features,
        labels,
        provenance: lightcraft_pipeline::personal_auto::SampleProvenance {
            sample_id: photo.id.clone(),
            shoot_id: shoot.id.clone(),
            split: Split::Train,
            source_ref: format!("manifest:{}", photo.id),
        },
    }))
}

fn train_variant(manifest: &Manifest, variant: Variant, manifest_digest: &str) -> Result<(Option<Value>, Option<Value>), String> {
    let split_manifest = core_split_manifest(manifest, manifest_digest);
    let mut samples = Vec::new();
    for shoot in manifest.shoots.iter().filter(|shoot| shoot.split == "train") {
        for photo in &shoot.photos {
            if let Some(sample) = core_sample(shoot, photo, variant)? {
                samples.push(sample);
            }
        }
    }
    if samples.is_empty() {
        return Ok((None, None));
    }
    let model = PersonalAutoModel::fit_train_only(&samples, &split_manifest).map_err(|error| format!("{}: {error}", variant.key()))?;
    let eligibility =
        serde_json::to_value(model.training_data_eligibility()).map_err(|_| format!("{}: could not encode eligibility", variant.key()))?;
    let model = serde_json::to_value(model).map_err(|_| format!("{}: could not encode pipeline model", variant.key()))?;
    Ok((Some(model), Some(eligibility)))
}

fn train_model(manifest: &Manifest, manifest_digest: &str) -> Result<Value, String> {
    let train_photos: Vec<&Photo> = manifest.shoots.iter().filter(|shoot| shoot.split == "train").flat_map(|shoot| shoot.photos.iter()).collect();
    let mut errors = Map::new();
    let (weak, weak_eligibility) = match train_variant(manifest, Variant::Weak, manifest_digest) {
        Ok(result) => result,
        Err(error) => {
            errors.insert(Variant::Weak.key().to_owned(), Value::String(error));
            (None, None)
        }
    };
    let (ember, ember_eligibility) = match train_variant(manifest, Variant::Ember, manifest_digest) {
        Ok(result) => result,
        Err(error) => {
            errors.insert(Variant::Ember.key().to_owned(), Value::String(error));
            (None, None)
        }
    };
    let status = if weak.is_some() || ember.is_some() { "experimentalTrainedUnqualified" } else { "harness-only" };
    let mut variants = Map::new();
    variants.insert(Variant::Weak.key().to_owned(), weak.map_or(Value::Null, |value| value));
    variants.insert(Variant::Ember.key().to_owned(), ember.map_or(Value::Null, |value| value));
    let mut eligibility = Map::new();
    if let Some(value) = weak_eligibility {
        eligibility.insert(Variant::Weak.key().to_owned(), value);
    }
    if let Some(value) = ember_eligibility {
        eligibility.insert(Variant::Ember.key().to_owned(), value);
    }
    let mut labelled = Map::new();
    let mut control_eligible = Map::new();
    let mut excluded = Map::new();
    for variant in [Variant::Weak, Variant::Ember] {
        let sample_count = eligibility.get(variant.key()).and_then(|value| value.get("sampleCount")).and_then(Value::as_u64).unwrap_or(0) as usize;
        let field_counts = eligibility.get(variant.key()).and_then(|value| value.get("fieldCounts")).cloned().unwrap_or_else(|| json!([]));
        labelled.insert(variant.key().to_owned(), json!(sample_count));
        control_eligible.insert(variant.key().to_owned(), field_counts);
        excluded.insert(variant.key().to_owned(), json!(train_photos.len().saturating_sub(sample_count)));
    }
    Ok(json!({
        "version": VERSION,
        "schema": MODEL_SCHEMA,
        "coreModelSchema": CORE_MODEL_SCHEMA,
        "status": status,
        "trainingSplit": "train",
        "featureSchema": manifest.features,
        "controls": manifest.controls,
        "controlBounds": manifest.bounds,
        "variants": variants,
        "trainingErrors": errors,
        "trainingEligibility": eligibility,
        "harnessOnlyReason": if weak.is_none() && ember.is_none() { json!("no variant had eligible train labels") } else { Value::Null },
        "qualificationStatus": "unqualified_no_promotion",
        "training": {"shootCount": manifest.shoots.iter().filter(|shoot| shoot.split == "train").count(), "inputPhotoCount": train_photos.len(), "labelledPhotoCount": labelled, "controlEligible": control_eligible, "excludedPhotoCount": excluded, "manifestSha256": manifest_digest},
        "provenance": {"baseline": "deterministicbaseline supplied in explicit manifest", "weakLabelColdStart": "per-photo provenance supplied in manifest", "emberGroundTruth": "accepted-human-edit provenance supplied in manifest", "provenanceCompleteness": "numericHarnessOnly"},
        "policy": {"automaticPromotion": false, "automaticApply": false, "catalogRead": false, "network": false, "render": false, "featuresMetadataFree": true, "excluded": ["masks", "sceneControls", "geometry", "cameraProfile", "calibration", "lensCorrection"]}
    }))
}

fn evaluate_model(manifest: &Manifest, model: &Value, manifest_digest: &str, requested: Option<Variant>) -> Result<Value, String> {
    validate_model(model, manifest, manifest_digest)?;
    let variants = requested.into_iter().collect::<Vec<_>>();
    let variants = if variants.is_empty() { vec![Variant::Weak, Variant::Ember] } else { variants };
    let mut reports = Map::new();
    for variant in variants {
        reports.insert(variant.key().to_owned(), evaluate_variant(manifest, model, variant));
    }
    let heldout_photos = manifest.shoots.iter().filter(|shoot| shoot.split != "train").flat_map(|shoot| shoot.photos.iter()).count();
    Ok(json!({
        "version": VERSION,
        "schema": REPORT_SCHEMA,
        "status": if heldout_photos == 0 { "harness-only" } else { "numericEvaluatedUnqualified" },
        "manifestSha256": manifest_digest,
        "excludedSplits": ["train"],
        "variants": reports,
        "modelStatus": model.get("status").cloned().unwrap_or(Value::String("unknown".into())),
        "trainingErrors": model.get("trainingErrors").cloned().unwrap_or_else(|| json!({})),
        "trainingEligibility": model.get("trainingEligibility").cloned().unwrap_or_else(|| json!({})),
        "dataCoverage": model.get("training").cloned().unwrap_or_else(|| json!({})),
        "harnessOnlyReason": model.get("harnessOnlyReason").cloned().unwrap_or(Value::Null),
        "qualificationStatus": "unqualified_no_promotion",
        "provenance": {"baseline": "deterministicbaseline supplied in explicit manifest", "weakLabelColdStart": "mapped settings are style-transfer evidence only", "emberGroundTruth": "accepted-human-edit targets require per-photo provenance", "provenanceCompleteness": "numericHarnessOnly"},
        "claims": {"visualPreference": "unavailable_without_supplied_blinded_votes_and_render_receipts", "personalPreference": "unclaimed", "weakLabelStatus": "style-transfer-only"},
        "policy": {"automaticPromotion": false, "automaticApply": false, "catalogRead": false, "network": false, "render": false, "masksExcluded": true, "sceneControlsExcluded": true}
    }))
}

fn validate_model(model: &Value, manifest: &Manifest, manifest_digest: &str) -> Result<(), String> {
    let object = model.as_object().ok_or("personal model root must be an object")?;
    if object.get("version").and_then(Value::as_u64) != Some(VERSION) || object.get("schema").and_then(Value::as_str) != Some(MODEL_SCHEMA) {
        return Err("personal model schema is unsupported".into());
    }
    if object.get("coreModelSchema").and_then(Value::as_str) != Some(CORE_MODEL_SCHEMA)
        || object.get("featureSchema") != Some(&json!(manifest.features))
        || object.get("controls") != Some(&json!(manifest.controls))
    {
        return Err("personal model schema does not match manifest or pipeline core".into());
    }
    if object.get("training").and_then(Value::as_object).and_then(|training| training.get("manifestSha256")).and_then(Value::as_str)
        != Some(manifest_digest)
    {
        return Err("personal model split manifest does not match evaluation manifest".into());
    }
    let variants = object.get("variants").and_then(Value::as_object).ok_or("personal model is missing variants")?;
    let split_manifest = core_split_manifest(manifest, manifest_digest);
    for variant in [Variant::Weak, Variant::Ember] {
        let Some(value) = variants.get(variant.key()) else { return Err("personal model is missing variant".into()) };
        if !value.is_null() {
            let core: PersonalAutoModel = serde_json::from_value(value.clone()).map_err(|_| "personal model variant is malformed".to_owned())?;
            core.validate_against(&split_manifest).map_err(|_| "personal model variant split manifest is incompatible".to_owned())?;
        }
    }
    Ok(())
}

fn evaluate_variant(manifest: &Manifest, model: &Value, variant: Variant) -> Value {
    let core_model = model
        .get("variants")
        .and_then(|value| value.get(variant.key()))
        .filter(|value| value.is_object())
        .and_then(|value| serde_json::from_value::<PersonalAutoModel>(value.clone()).ok())
        .filter(|value| value.validate().is_ok());
    let mut split_reports = Map::new();
    for split in ["validation", "eval", "test"] {
        split_reports.insert(split.to_owned(), evaluate_split(manifest, core_model.as_ref(), variant, split));
    }
    let status = if core_model.is_some() { "numericEvaluatedUnqualified" } else { "harness-only" };
    json!({"status": status, "coreModelSchema": CORE_MODEL_SCHEMA, "qualificationStatus": "unqualified_no_promotion", "splits": split_reports})
}

fn evaluate_split(manifest: &Manifest, model: Option<&PersonalAutoModel>, variant: Variant, split: &str) -> Value {
    let mut pairs = Vec::new();
    let mut per_photo: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut per_shoot: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    let mut per_camera: BTreeMap<String, Vec<(f64, f64)>> = BTreeMap::new();
    for shoot in manifest.shoots.iter().filter(|shoot| shoot.split == split) {
        for photo in &shoot.photos {
            let Some(target) = variant.target(photo) else { continue };
            let Some(model) = model else { continue };
            let Some(baseline_distance) = distance(&photo.baseline, &photo.baseline, target, &manifest.controls, &manifest.bounds) else { continue };
            let Some(predicted) = predict(model, photo, manifest) else { continue };
            let Some(regressor_distance) = distance(&predicted, &photo.baseline, target, &manifest.controls, &manifest.bounds) else { continue };
            pairs.push((baseline_distance, regressor_distance));
            per_photo.insert(photo.id.clone(), vec![(baseline_distance, regressor_distance)]);
            per_shoot.entry(shoot.id.clone()).or_default().push((baseline_distance, regressor_distance));
            per_camera.entry(shoot.camera.clone()).or_default().push((baseline_distance, regressor_distance));
        }
    }
    let shoots = per_shoot.iter().map(|(id, pairs)| (id.clone(), metric(pairs))).collect::<Map<String, Value>>();
    let cameras = per_camera.iter().map(|(id, pairs)| (id.clone(), metric(pairs))).collect::<Map<String, Value>>();
    let photos = per_photo.iter().map(|(id, pairs)| (id.clone(), metric(pairs))).collect::<Map<String, Value>>();
    json!({"photos": metric(&pairs), "perPhoto": photos, "shoots": shoots, "byCamera": cameras})
}

fn predict(model: &PersonalAutoModel, photo: &Photo, manifest: &Manifest) -> Option<BTreeMap<String, f64>> {
    let values: [f64; FEATURE_COUNT] = photo.features.clone().try_into().ok()?;
    let features = FeatureVector::new(values).ok()?;
    let mut baseline = DevelopSettings::default();
    for control in STYLE_IDS {
        if !controls::set(&mut baseline, control, photo.baseline[control]) {
            return None;
        }
    }
    let prediction = model.predict(&baseline, &features);
    if !prediction.used_model {
        return None;
    }
    let mut result = photo.baseline.clone();
    for control in &manifest.controls {
        result.insert(control.clone(), controls::get(&prediction.settings, control)?);
    }
    Some(result)
}

fn distance(
    values: &BTreeMap<String, f64>,
    baseline: &BTreeMap<String, f64>,
    target: &Target,
    controls: &[String],
    bounds: &BTreeMap<String, (f64, f64)>,
) -> Option<f64> {
    let mut total = 0.0;
    let mut count = 0usize;
    for control in controls {
        let Some(target_value) = target.values.get(control) else { continue };
        let value = *values.get(control)?;
        let (low, high) = bounds[control];
        let expected = if target.deltas { baseline[control] + target_value } else { *target_value };
        total += (value - expected).abs() / (high - low);
        count += 1;
    }
    if count == 0 { None } else { Some(total / count as f64) }
}

fn metric(pairs: &[(f64, f64)]) -> Value {
    if pairs.is_empty() {
        return json!({"distance": "normalizedL1StyleEditDistance", "count": 0, "baselineMean": null, "baselineMedian": null, "regressorMean": null, "regressorMedian": null, "gain": null, "pairedImprovementRate": null});
    }
    let baseline: f64 = pairs.iter().map(|pair| pair.0).sum::<f64>() / pairs.len() as f64;
    let regressor: f64 = pairs.iter().map(|pair| pair.1).sum::<f64>() / pairs.len() as f64;
    let mut baseline_values = pairs.iter().map(|pair| pair.0).collect::<Vec<_>>();
    let mut regressor_values = pairs.iter().map(|pair| pair.1).collect::<Vec<_>>();
    baseline_values.sort_by(f64::total_cmp);
    regressor_values.sort_by(f64::total_cmp);
    let middle = pairs.len() / 2;
    let baseline_median = if pairs.len() % 2 == 0 { (baseline_values[middle - 1] + baseline_values[middle]) / 2.0 } else { baseline_values[middle] };
    let regressor_median =
        if pairs.len() % 2 == 0 { (regressor_values[middle - 1] + regressor_values[middle]) / 2.0 } else { regressor_values[middle] };
    let improvements = pairs.iter().filter(|pair| pair.1 < pair.0).count();
    let gain = if baseline > 0.0 { Some((baseline - regressor) / baseline) } else { None };
    json!({"distance": "normalizedL1StyleEditDistance", "count": pairs.len(), "baselineMean": baseline, "baselineMedian": baseline_median, "regressorMean": regressor, "regressorMedian": regressor_median, "gain": gain, "pairedImprovementRate": improvements as f64 / pairs.len() as f64})
}

#[cfg(test)]
mod tests {
    use super::*;

    fn manifest() -> Value {
        json!({"version":1,"feature_schema":["ev.p01","ev.p05","ev.median","ev.p95","ev.p995","ev.spread","chroma.p90","luminance.mean","luminance.stddev","scene.shadow_share","scene.highlight_share","source.aspect","source.raw","baseline.exposure","source.wb_temp","source.wb_tint"],"controls":["light.contrast","color.vibrance","color.saturation"],"shoots":[
            {"shoot_id":"train-1","split":"train","camera":{"make":"Ember","model":"Synthetic"},"photos":[
                {"photo_id":"train-a","features":[0.0,0.0,0.0,0.0,0.0,0.0,0.2,0.2,0.1,0.1,0.1,1.5,1.0,0.0,6500.0,0.0],"baseline":{"light.contrast":0.0,"color.vibrance":0.0,"color.saturation":0.0},"emberGroundTruth":{"values":{"light.contrast":10.0,"color.vibrance":2.0,"color.saturation":1.0},"confidence":{"light.contrast":1.0,"color.vibrance":1.0,"color.saturation":1.0},"provenance":{"accepted":true,"origin":"human-edit"}}},
                {"photo_id":"train-b","features":[1.0,1.0,1.0,1.0,1.0,0.0,0.4,0.4,0.1,0.1,0.1,1.5,1.0,0.0,6500.0,0.0],"baseline":{"light.contrast":0.0,"color.vibrance":0.0,"color.saturation":0.0},"weakLabelColdStart":{"values":{"light.contrast":8.0,"color.vibrance":2.0,"color.saturation":1.0},"confidence":{"light.contrast":1.0,"color.vibrance":1.0,"color.saturation":1.0},"provenance":{"source":"lightroom"}}}
            ]},
            {"shoot_id":"validation-1","split":"validation","camera":{"make":"Ember","model":"Synthetic"},"photos":[
                {"photo_id":"validation-a","features":[0.5,0.5,0.5,0.5,0.5,0.0,0.3,0.3,0.1,0.1,0.1,1.5,1.0,0.0,6500.0,0.0],"baseline":{"light.contrast":0.0,"color.vibrance":0.0,"color.saturation":0.0},"emberGroundTruth":{"values":{"light.contrast":5.0,"color.vibrance":1.0,"color.saturation":0.5},"confidence":{"light.contrast":1.0,"color.vibrance":1.0,"color.saturation":1.0},"provenance":{"accepted":true,"origin":"human-edit"}}}
            ]},
            {"shoot_id":"test-1","split":"test","camera":{"make":"Other","model":"Synthetic"},"photos":[
                {"photo_id":"test-a","features":[0.5,0.5,0.5,0.5,0.5,0.0,0.3,0.3,0.1,0.1,0.1,1.5,1.0,0.0,6500.0,0.0],"baseline":{"light.contrast":0.0,"color.vibrance":0.0,"color.saturation":0.0},"weakLabelColdStart":{"values":{"light.contrast":4.0,"color.vibrance":1.0,"color.saturation":0.5},"confidence":{"light.contrast":1.0,"color.vibrance":1.0,"color.saturation":1.0},"provenance":{"source":"lightroom"}}}
            ]}
        ]})
    }

    #[test]
    fn trains_only_train_shoots_and_evaluates_holdouts() {
        let parsed = parse_manifest(&manifest()).unwrap();
        let model = train_model(&parsed, "fixture").unwrap();
        assert_eq!(model["trainingSplit"], "train");
        assert_eq!(model["training"]["shootCount"], 1);
        let report = evaluate_model(&parsed, &model, "fixture", None).unwrap();
        assert_eq!(report["excludedSplits"][0], "train");
        assert_eq!(report["variants"]["emberGroundTruth"]["splits"]["validation"]["photos"]["count"], 1);
        assert_eq!(report["claims"]["visualPreference"], "unavailable_without_supplied_blinded_votes_and_render_receipts");
    }

    #[test]
    fn excluded_controls_are_rejected() {
        let mut value = manifest();
        value["controls"] = json!(["light.contrast", "color.vibrance", "light.exposure"]);
        assert!(parse_manifest(&value).is_err());
    }

    #[test]
    fn frozen_numeric_fixture_keeps_whole_shoot_splits() {
        let value: Value = serde_json::from_str(include_str!("../../../tests/fixtures/personal-auto/minimal.json")).unwrap();
        let parsed = parse_manifest(&value).unwrap();
        assert_eq!(parsed.shoots.iter().filter(|shoot| shoot.split == "train").count(), 1);
        assert_eq!(parsed.shoots.iter().filter(|shoot| shoot.split == "validation").count(), 1);
        assert_eq!(parsed.shoots.iter().filter(|shoot| shoot.split == "test").count(), 1);
        assert_eq!(parsed.shoots.iter().map(|shoot| shoot.photos.len()).sum::<usize>(), 4);
    }

    #[test]
    fn malformed_label_and_nonhuman_provenance_are_rejected() {
        let mut value = manifest();
        value["shoots"][0]["photos"][0]["emberGroundTruth"]["values"]["light.contrast"] = json!("bad");
        let error = parse_manifest(&value).expect_err("malformed numeric label must not be dropped");
        assert!(error.contains("emberGroundTruth"));

        let mut value = manifest();
        value["shoots"][0]["photos"][0]["emberGroundTruth"].as_object_mut().unwrap().remove("confidence");
        let error = parse_manifest(&value).expect_err("missing confidence must not default silently");
        assert!(error.contains("confidence"));

        let mut value = manifest();
        value["shoots"][0]["photos"][0]["emberGroundTruth"]["provenance"] = json!({"accepted": true, "origin": "automatic-edit"});
        let error = parse_manifest(&value).expect_err("automatic edit is not Ember ground truth");
        assert!(error.contains("accepted human-edit provenance"));
    }
}
