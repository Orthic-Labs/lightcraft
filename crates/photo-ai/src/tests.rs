use super::*;
use lightcraft_codecs::{EncodeImage, EncodeMeta, encode_png};
use lightcraft_raster::Rgba8;

fn png(meta: EncodeMeta<'_>) -> Vec<u8> {
    encode_png(&EncodeImage::rgba8(&Rgba8::filled(2, 2, [100, 100, 100, 255])), &meta).unwrap()
}
fn valid() -> Value {
    json!({"decision":"adjust", "confidence":0.8, "reason":"Dark midtones with highlight headroom", "recipe": {
        "light.exposure":1, "light.contrast":0, "light.highlights":-10, "light.shadows":10,
        "light.whites":0, "light.blacks":0, "color.vibrance":5, "color.saturation":0
    }})
}
fn envelope(assessment: &Value) -> Value {
    json!({"id":"test", "model":"deepseek/deepseek-v4.1-flash", "provider":"test", "usage":{"cost":0.001}, "choices":[{"finish_reason":"stop", "message":{"content":assessment.to_string()}}]})
}
fn parse(value: &Value) -> Receipt {
    parse_response("deepseek/deepseek-v4.1-flash", "source-digest", 42, &serde_json::to_vec(value).unwrap())
}

#[test]
fn real_codec_proxy_and_request_preserve_privacy_and_scope() {
    let bytes = png(EncodeMeta::default());
    let proxy = Proxy::new(bytes.clone()).unwrap();
    assert_eq!(proxy.digest, digest(&bytes));
    let request = request_body("deepseek/deepseek-v4.1-flash", &proxy, "intentional silhouette").unwrap();
    assert_eq!(request["provider"]["allow_fallbacks"], false);
    assert_eq!(request["provider"]["require_parameters"], true);
    assert_eq!(request["provider"]["data_collection"], "deny");
    assert_eq!(request["provider"]["zdr"], true);
    assert_eq!(request["response_format"]["json_schema"]["strict"], true);
    assert!(request.get("tools").is_none());
    let url = request["messages"][1]["content"][1]["image_url"]["url"].as_str().unwrap();
    assert_eq!(STANDARD.decode(url.strip_prefix("data:image/png;base64,").unwrap()).unwrap(), bytes);
}

#[test]
fn private_metadata_oversized_and_broken_proxies_are_rejected() {
    assert!(Proxy::new(png(EncodeMeta { exif: Some(b"private metadata"), ..Default::default() })).is_err());
    assert!(Proxy::new(png(EncodeMeta { xmp: Some("private metadata"), ..Default::default() })).is_err());
    let mut bytes = png(EncodeMeta::default());
    bytes.extend_from_slice(b"private trailer");
    assert!(Proxy::new(bytes).is_err());
    let mut bytes = png(EncodeMeta::default());
    bytes.get_mut(16..20).unwrap().copy_from_slice(&513u32.to_be_bytes());
    assert!(Proxy::new(bytes).is_err());
    assert!(Proxy::new(vec![0; MAX_PROXY_BYTES + 1]).is_err());
    assert!(Proxy::new(vec![]).is_err());
}

#[test]
fn schema_and_native_control_limits_agree() {
    let schema = schema().unwrap();
    let properties = &schema["properties"]["recipe"]["anyOf"][1]["properties"];
    assert_eq!(properties.as_object().unwrap().len(), CONTROLS.len());
    for (name, bound) in CONTROLS {
        assert_eq!(properties[name]["maximum"].as_f64(), Some(bound));
        assert_eq!(properties[name]["minimum"].as_f64(), Some(-bound));
        let spec = lightcraft_develop::controls::find(name).unwrap();
        assert!(spec.min <= -bound && spec.max >= bound);
    }
}

#[test]
fn complete_valid_assessment_has_source_and_usage_receipt() {
    let receipt = parse(&envelope(&valid()));
    assert!(receipt.error.is_none());
    assert!(receipt.assessment.is_some());
    assert_eq!(receipt.proxy_sha256, "source-digest");
    assert_eq!(receipt.usage["cost"], 0.001);
    assert_eq!(receipt.latency_ms, 42);
}

#[test]
fn usage_keeps_numeric_billing_but_discards_arbitrary_provider_data() {
    let mut response = envelope(&valid());
    response["usage"] = json!({"cost":0.002,"prompt_tokens":100,"extra":"private text","completion_tokens":-1,"total_tokens":"private text","prompt_tokens_details":{"cached_tokens":12,"extra":"private text"}});
    let receipt = parse(&response);
    assert_eq!(receipt.usage, json!({"cost":0.002,"prompt_tokens":100,"prompt_tokens_details.cached_tokens":12}));
    response["usage"]["cost"] = json!(-1);
    assert!(parse(&response).usage.get("cost").is_none());
}

#[test]
fn invalid_recipe_never_clamps_or_becomes_an_edit() {
    for (name, bound) in CONTROLS {
        for bad in [bound + 0.01, -bound - 0.01] {
            let mut assessment = valid();
            assessment["recipe"][name] = json!(bad);
            let receipt = parse(&envelope(&assessment));
            assert!(receipt.assessment.is_none());
            assert!(receipt.error.is_some());
            assert_eq!(receipt.usage["cost"], 0.001);
        }
    }
    let mut assessment = valid();
    assessment["recipe"]["color.temp"] = json!(9000);
    assert!(parse(&envelope(&assessment)).assessment.is_none());
    let mut assessment = valid();
    assessment["confidence"] = json!(0.69);
    assert!(parse(&envelope(&assessment)).assessment.is_none());
    let mut assessment = valid();
    assessment["recipe"].as_object_mut().unwrap().remove("light.exposure");
    assert!(parse(&envelope(&assessment)).assessment.is_none());
}

#[test]
fn refusal_partial_multiple_choices_and_model_switch_remain_review() {
    for finish in ["length", "content_filter", "tool_calls"] {
        let mut response = envelope(&valid());
        response["choices"][0]["finish_reason"] = json!(finish);
        assert!(parse(&response).assessment.is_none());
    }
    let mut response = envelope(&valid());
    response["choices"][0]["message"]["refusal"] = json!("blocked");
    assert!(parse(&response).assessment.is_none());
    let mut response = envelope(&valid());
    response["model"] = json!("fallback-model");
    assert!(parse(&response).assessment.is_none());
    let mut response = envelope(&valid());
    let choice = response["choices"][0].clone();
    response["choices"].as_array_mut().unwrap().push(choice);
    assert!(parse(&response).assessment.is_none());
    assert!(parse_response("deepseek/deepseek-v4.1-flash", "digest", 0, b"not JSON").assessment.is_none());
}

#[test]
fn required_null_recipe_and_duplicate_fields_are_not_silently_accepted() {
    let good = json!({"decision":"review", "confidence":0.2, "reason":"Intent unclear", "recipe":null});
    assert!(parse(&envelope(&good)).assessment.is_some());
    let mut missing = good;
    missing.as_object_mut().unwrap().remove("recipe");
    assert!(parse(&envelope(&missing)).assessment.is_none());
    let duplicate = valid().to_string().replace("\"light.exposure\":1", "\"light.exposure\":1,\"light.exposure\":2");
    let mut response = envelope(&valid());
    response["choices"][0]["message"]["content"] = json!(duplicate);
    assert!(parse(&response).assessment.is_none());
}

#[cfg(all(feature = "network", not(target_arch = "wasm32")))]
#[test]
fn capability_and_revision_drift_fail_before_upload() {
    let model = "deepseek/deepseek-v4.1-flash";
    let mut catalog = json!({"data":[{"id":model,"canonical_slug":"deepseek/deepseek-v4.1-flash-20260910","architecture":{"input_modalities":["text","image"]},"supported_parameters":["structured_outputs"]}]});
    assert!(network::preflight(&catalog, model).is_ok());
    catalog["data"][0]["canonical_slug"] = json!("changed-model-version");
    assert!(network::preflight(&catalog, model).is_err());
    catalog["data"][0]["canonical_slug"] = json!("deepseek/deepseek-v4.1-flash-20260910");
    catalog["data"][0]["architecture"]["input_modalities"] = json!(["text"]);
    assert!(network::preflight(&catalog, model).is_err());
}
