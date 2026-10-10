//! Integration coverage for original procedural denoise baseline receipts.

use std::process::Command;

use serde_json::Value;

const BIN: &str = env!("CARGO_BIN_EXE_lightcraft-cli");

#[test]
fn baseline_receipt_is_numeric_repeatable_bounded_and_unqualified() {
    let first = temp("first.json");
    let second = temp("second.json");
    let a = run(&first, "MacBook-Pro", "rev-1");
    assert!(a.status.success(), "{}", String::from_utf8_lossy(&a.stderr));
    let b = run(&second, "MacBook-Pro", "rev-1");
    assert!(b.status.success(), "{}", String::from_utf8_lossy(&b.stderr));
    let first_value: Value = serde_json::from_slice(&std::fs::read(&first).unwrap()).unwrap();
    let second_value: Value = serde_json::from_slice(&std::fs::read(&second).unwrap()).unwrap();
    assert_eq!(first_value["status"], "alwaysUNQUALIFIED");
    assert_eq!(first_value["hardware"], "MacBook-Pro");
    assert_eq!(first_value["sourceRevision"], "rev-1");
    assert_eq!(first_value["cases"].as_array().map(Vec::len), Some(5));
    assert!(first_value["cases"].as_array().unwrap().iter().all(|case| {
        case["identity"]["metrics"]["mse"].as_f64().is_some()
            && case["identity"]["outputSha256"] == case["noisySha256"]
            && case["amounts"].as_array().map(Vec::len) == Some(5)
            && case["amounts"].as_array().unwrap().iter().all(|amount| {
                amount["cleanSha256"].as_str().map(str::len) == Some(64)
                    && amount["noisySha256"].as_str().map(str::len) == Some(64)
                    && amount["outputSha256"].as_str().map(str::len) == Some(64)
                    && amount["timingMs"]["warmP50"].as_f64().is_some()
                    && amount["timingMs"]["warmP95"].as_f64().is_some()
                    && amount["repeatability"]["errors"].as_array().map(Vec::is_empty) == Some(true)
                    && amount["validation"]["outOfRangeChannelsAcrossRuns"].as_u64().is_some()
                    && amount["validation"]["outputRangeWithinUnit"].as_bool()
                        == Some(amount["validation"]["outOfRangeChannelsAcrossRuns"].as_u64() == Some(0))
            })
            && case["amounts"][0]["outputSha256"] == case["noisySha256"]
            && case["amounts"][0]["metrics"] == case["identity"]["metrics"]
    }));
    assert!(first_value["cases"].as_array().unwrap().iter().all(|case| {
        case["amounts"][0]["timingMs"]["warmP50"].as_f64().unwrap_or(0.0) <= case["amounts"][0]["timingMs"]["warmP95"].as_f64().unwrap_or(0.0)
    }));
    let first_digests = digests(&first_value);
    assert_eq!(first_digests, digests(&second_value));
    let text = std::fs::read_to_string(&first).unwrap();
    assert!(text.ends_with('\n') && text.len() <= 4 * 1024 * 1024);
    assert!(!text.contains("imageFile") && !text.contains("\"pixels\""));
    cleanup(&first);
    cleanup(&second);
}

#[test]
fn hostile_labels_and_existing_output_are_rejected_without_replacement() {
    let hostile = temp("hostile.json");
    let rejected = Command::new(BIN).args(["ai", "denoise", "baseline", "--hardware", "bad/name", "--out"]).arg(&hostile).output().unwrap();
    assert!(!rejected.status.success());
    assert!(!hostile.exists());

    let existing = temp("existing.json");
    std::fs::write(&existing, b"sentinel\n").unwrap();
    let rejected = run(&existing, "CPU", "rev");
    assert!(!rejected.status.success());
    assert_eq!(std::fs::read(&existing).unwrap(), b"sentinel\n");
    cleanup(&existing);

    for args in [vec!["64".to_owned(), "1".to_owned()], vec!["513".to_owned(), "2".to_owned()]] {
        let out = temp(&format!("bounds-{}.json", args[0]));
        let result = Command::new(BIN)
            .args(["ai", "denoise", "baseline", "--hardware", "CPU", "--out"])
            .arg(&out)
            .args(["--size", &args[0], "--repeats", &args[1]])
            .output()
            .unwrap();
        assert!(!result.status.success());
        assert!(!out.exists());
    }

    let duplicate = temp("duplicate.json");
    let result =
        Command::new(BIN).args(["ai", "denoise", "baseline", "--hardware", "CPU", "--hardware", "GPU", "--out"]).arg(&duplicate).output().unwrap();
    assert!(!result.status.success());
    assert!(!duplicate.exists());
}

fn run(out: &std::path::Path, hardware: &str, revision: &str) -> std::process::Output {
    Command::new(BIN)
        .args(["ai", "denoise", "baseline", "--hardware", hardware, "--out"])
        .arg(out)
        .args(["--size", "64", "--repeats", "2", "--source-revision", revision])
        .output()
        .unwrap()
}

fn digests(value: &Value) -> Vec<String> {
    value["cases"]
        .as_array()
        .unwrap()
        .iter()
        .flat_map(|case| case["amounts"].as_array().unwrap())
        .map(|amount| amount["outputSha256"].as_str().unwrap().to_owned())
        .collect()
}

fn temp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("lightcraft-denoise-{}-{name}", std::process::id()))
}

fn cleanup(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}
