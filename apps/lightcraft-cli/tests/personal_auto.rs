//! End-to-end coverage for explicit-file Personal Auto extraction, training & evaluation.

use std::process::Command;

use lightcraft_codecs::{EncodeImage, encode_png};
use lightcraft_raster::Rgba8;
use serde_json::{Value, json};

const BIN: &str = env!("CARGO_BIN_EXE_lightcraft-cli");
const VALID_SIDECAR: &str = r#"<x:xmpmeta xmlns:x="adobe:ns:meta/">
 <rdf:RDF xmlns:rdf="http://www.w3.org/1999/02/22-rdf-syntax-ns#">
  <rdf:Description rdf:about=""
    xmlns:xmp="http://ns.adobe.com/xap/1.0/"
    xmp:CreatorTool="Lightcraft CLI owned test"
    xmp:Label="FixtureSentinel"/>
 </rdf:RDF>
</x:xmpmeta>"#;

#[test]
fn explicit_extract_train_evaluate_preserves_sources_and_binds_receipts() {
    let root = unique_temp_dir();
    std::fs::create_dir_all(&root).unwrap();
    let train = root.join("train.png");
    let heldout = root.join("heldout.png");
    gradient_png(&train, 7);
    gradient_png(&heldout, 19);
    let train_before = std::fs::read(&train).unwrap();
    let heldout_before = std::fs::read(&heldout).unwrap();
    let train_sidecar = train.with_extension("xmp");
    let heldout_sidecar = heldout.with_extension("xmp");
    std::fs::write(&train_sidecar, VALID_SIDECAR.as_bytes()).unwrap();
    std::fs::write(&heldout_sidecar, VALID_SIDECAR.as_bytes()).unwrap();
    let train_sidecar_before = std::fs::read(&train_sidecar).unwrap();
    let heldout_sidecar_before = std::fs::read(&heldout_sidecar).unwrap();

    let input = root.join("input.json");
    std::fs::write(&input, extraction_input(&train, &heldout).to_string()).unwrap();
    let features = root.join("features.json");
    let extracted = run_extract(&input, &features);
    assert!(extracted.status.success(), "{}", String::from_utf8_lossy(&extracted.stderr));
    let extracted_value: Value = serde_json::from_slice(&std::fs::read(&features).unwrap()).unwrap();
    assert_eq!(extracted_value["feature_schema"].as_array().map(Vec::len), Some(16));
    assert_eq!(extracted_value["shoots"].as_array().map(Vec::len), Some(2));
    assert_eq!(extracted_value["shoots"][0]["photos"].as_array().map(Vec::len), Some(1));
    assert_eq!(extracted_value["shoots"][1]["photos"].as_array().map(Vec::len), Some(1));
    let extracted_text = std::fs::read_to_string(&features).unwrap();
    assert!(!extracted_text.contains("private/original.xmp"));
    assert!(!extracted_text.contains("private-editor"));
    assert!(extracted_value["shoots"][0]["photos"][0]["emberGroundTruth"]["provenance"]["path"].is_null());
    assert!(extracted_value["shoots"][0]["photos"][0]["emberGroundTruth"]["provenance"]["provenanceSha256"].as_str().is_some());

    let model = root.join("model.json");
    let trained = Command::new(BIN).args(["ai", "personal", "train", "--manifest"]).arg(&features).args(["--out"]).arg(&model).output().unwrap();
    assert!(trained.status.success(), "{}", String::from_utf8_lossy(&trained.stderr));
    let model_value: Value = serde_json::from_slice(&std::fs::read(&model).unwrap()).unwrap();
    let receipt = &model_value["trainingLabelReceipts"]["emberGroundTruth"][0];
    assert_eq!(receipt["photoId"], "train-photo");
    assert_eq!(receipt["shootId"], "train-shoot");
    assert!(receipt["provenanceSha256"].as_str().is_some());
    let receipt_text = std::fs::read_to_string(&model).unwrap();
    assert!(!receipt_text.contains("private/original.xmp"));
    assert!(!receipt_text.contains("private-editor"));

    let report = root.join("report.json");
    let evaluated = Command::new(BIN)
        .args(["ai", "personal", "evaluate", "--manifest"])
        .arg(&features)
        .args(["--model"])
        .arg(&model)
        .args(["--variant", "both", "--out"])
        .arg(&report)
        .output()
        .unwrap();
    assert!(evaluated.status.success(), "{}", String::from_utf8_lossy(&evaluated.stderr));
    let report_value: Value = serde_json::from_slice(&std::fs::read(&report).unwrap()).unwrap();
    assert_eq!(report_value["status"], "numericEvaluatedUnqualified");
    for variant in ["weakLabelColdStart", "emberGroundTruth"] {
        let coverage = &report_value["variants"][variant]["splits"]["test"]["coverage"];
        assert_eq!(coverage["input"], 1);
        assert_eq!(coverage["eligible"], 1);
        assert_eq!(coverage["scored"], 1);
        assert_eq!(
            coverage["scored"].as_u64(),
            Some(coverage["modelScored"].as_u64().unwrap_or(0) + coverage["fallbackScored"].as_u64().unwrap_or(0))
        );
    }
    assert_eq!(report_value["claims"]["personalPreference"], "unclaimed");

    assert_eq!(std::fs::read(&train).unwrap(), train_before);
    assert_eq!(std::fs::read(&heldout).unwrap(), heldout_before);
    assert_eq!(std::fs::read(&train_sidecar).unwrap(), train_sidecar_before);
    assert_eq!(std::fs::read(&heldout_sidecar).unwrap(), heldout_sidecar_before);

    let second = run_extract(&input, &features);
    assert!(!second.status.success(), "extract must refuse existing output");
    assert!(String::from_utf8_lossy(&second.stderr).contains("new extracted manifest"));
    assert_eq!(std::fs::read(&train).unwrap(), train_before);

    let malformed = root.join("malformed.json");
    let malformed_output = root.join("malformed-output.json");
    let mut malformed_value = extraction_input(&train, &heldout);
    malformed_value["userConsented"] = json!(false);
    std::fs::write(&malformed, malformed_value.to_string()).unwrap();
    let rejected = run_extract(&malformed, &malformed_output);
    assert!(!rejected.status.success());
    assert!(!malformed_output.exists(), "malformed extraction must not create output");

    std::fs::remove_dir_all(root).unwrap();
}

fn run_extract(input: &std::path::Path, output: &std::path::Path) -> std::process::Output {
    Command::new(BIN).args(["ai", "personal", "extract", "--manifest"]).arg(input).args(["--out"]).arg(output).output().unwrap()
}

fn extraction_input(train: &std::path::Path, heldout: &std::path::Path) -> Value {
    json!({
        "version": 1,
        "userConsented": true,
        "shoots": [
            {"shoot_id": "train-shoot", "split": "train", "camera": "synthetic-camera", "photos": [{"photo_id": "train-photo", "path": train.to_string_lossy(), "weakLabelColdStart": weak_label(), "emberGroundTruth": ember_label()}]},
            {"shoot_id": "heldout-shoot", "split": "test", "camera": "synthetic-camera", "photos": [{"photo_id": "heldout-photo", "path": heldout.to_string_lossy(), "weakLabelColdStart": weak_label(), "emberGroundTruth": ember_label()}]}
        ]
    })
}

fn weak_label() -> Value {
    json!({
        "values": {"light.contrast": 2.0, "color.vibrance": 1.0, "color.saturation": 1.5},
        "confidence": {"light.contrast": 1.0, "color.vibrance": 0.9, "color.saturation": 0.8},
        "provenance": {"source": "lightroom", "path": "/private/original.xmp", "editor": "private-editor"}
    })
}

fn ember_label() -> Value {
    json!({
        "values": {"light.contrast": 3.0, "color.vibrance": 1.5, "color.saturation": 2.0},
        "confidence": {"light.contrast": 1.0, "color.vibrance": 0.9, "color.saturation": 0.8},
        "provenance": {"accepted": true, "origin": "human-edit", "path": "/private/original.xmp", "editor": "private-editor"}
    })
}

fn gradient_png(path: &std::path::Path, seed: u32) {
    let image = Rgba8::from_fn(32, 24, |x, y| [((x as u32 + seed) % 256) as u8, ((y as u32 * 3 + seed) % 256) as u8, 100, 255]);
    let png = encode_png(&EncodeImage::rgba8(&image), &Default::default()).unwrap();
    std::fs::write(path, png).unwrap();
}

fn unique_temp_dir() -> std::path::PathBuf {
    let stamp = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos();
    std::env::temp_dir().join(format!("lightcraft-personal-auto-{}-{stamp}", std::process::id()))
}
