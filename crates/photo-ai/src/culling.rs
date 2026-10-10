//! Read-only ambiguous-burst culling contract.
//!
//! This module only prepares bounded provider requests & validates suggestions. It never
//! writes catalog state, deletes files, or claims calibrated visual probabilities.

use std::collections::BTreeSet;

use base64::{Engine, engine::general_purpose::STANDARD};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

use crate::{MAX_OUTPUT_TOKENS, MAX_RESPONSE_BYTES, Proxy, models, validate_model};

pub const VERSION: &str = "photo-culling-v1";
pub const PROMPT: &str = include_str!("../culling.prompt.txt");
pub const SCHEMA: &str = include_str!("../culling.schema.json");
pub const MIN_PROXIES: usize = 2;
pub const MAX_PROXIES: usize = 8;
pub const MAX_OPAQUE_ID_CHARS: usize = 64;
pub const MAX_REASON_CHARS: usize = 240;
pub const MAX_REQUEST_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_PROXY_INPUT_BYTES: usize = MAX_REQUEST_BYTES / 2;
/// Conservative request admission bound from frozen experiment policy.
pub const MAX_INPUT_TOKENS_RESERVE: u64 = 20_000;
pub const MAX_BUDGET_USD: f64 = 5.0;
pub const MAX_DEADLINE_SECS: u64 = 45;

/// Input carries only caller-chosen opaque ID & metadata-free encoded proxy.
pub struct ProxyInput<'a> {
    pub id: &'a str,
    pub proxy: &'a Proxy,
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct RequestOptions {
    pub budget_usd: f64,
    pub deadline_secs: u64,
}

impl Default for RequestOptions {
    fn default() -> Self {
        Self { budget_usd: MAX_BUDGET_USD, deadline_secs: MAX_DEADLINE_SECS }
    }
}

impl RequestOptions {
    pub fn validate(&self) -> Result<(), String> {
        if !self.budget_usd.is_finite() || !(0.0..=MAX_BUDGET_USD).contains(&self.budget_usd) || self.budget_usd == 0.0 {
            return Err("budget must be finite, positive, and at most 5 USD".into());
        }
        if self.deadline_secs == 0 || self.deadline_secs > MAX_DEADLINE_SECS {
            return Err("deadline must be between 1 and 45 seconds".into());
        }
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Decision {
    Reject,
    Keep,
    Pick,
    Review,
    Abstain,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ReasonCode {
    RelativeContent,
    TimeSimilarity,
    AmbiguousQuality,
    InsufficientProxy,
    TieForBest,
    ModelUncertain,
    Abstained,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum Uncertainty {
    Unknown,
    Elevated,
    High,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ProxyQuality {
    Limited512,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum SignalStatus {
    Unavailable,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum CalibrationStatus {
    Unknown,
}

#[derive(Clone, Copy, Debug, Deserialize, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum ConfidenceKind {
    Unqualified,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BudgetPolicy {
    AdmissionReserve,
}

#[derive(Clone, Copy, Debug, Serialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BillingStatus {
    Reported,
    Unknown,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CullingItem {
    pub id: String,
    pub decision: Decision,
    pub reason_codes: Vec<ReasonCode>,
    pub reason: String,
    pub uncertainty: Uncertainty,
    pub confidence: Option<f64>,
    pub confidence_kind: ConfidenceKind,
}

#[derive(Clone, Debug, Deserialize, Serialize, PartialEq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CullingResult {
    pub proxy_quality: ProxyQuality,
    pub sharpness: SignalStatus,
    pub blink: SignalStatus,
    pub calibration: CalibrationStatus,
    pub acceptable_winner_ids: Vec<String>,
    pub items: Vec<CullingItem>,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyReceipt {
    pub id: String,
    pub sha256: String,
}

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Receipt {
    pub version: &'static str,
    pub requested_model: String,
    pub served_model: Option<String>,
    pub provider: Option<String>,
    pub response_id: Option<String>,
    pub proxies: Vec<ProxyReceipt>,
    pub budget_usd: f64,
    pub budget_policy: BudgetPolicy,
    pub admission_reserve_usd: f64,
    pub billing_status: BillingStatus,
    pub deadline_secs: u64,
    pub latency_ms: u64,
    pub usage: Value,
    pub result: Option<CullingResult>,
    pub error: Option<String>,
}

/// Validate IDs, proxy count, unique coverage, and bounded input bytes before encoding.
pub fn validate_inputs(proxies: &[ProxyInput<'_>]) -> Result<(), String> {
    if !(MIN_PROXIES..=MAX_PROXIES).contains(&proxies.len()) {
        return Err("culling requires between 2 and 8 proxies".into());
    }
    let mut ids = BTreeSet::new();
    let mut bytes = 0usize;
    for input in proxies {
        validate_opaque_id(input.id)?;
        if !ids.insert(input.id) {
            return Err("duplicate opaque proxy ID".into());
        }
        bytes = bytes.checked_add(input.proxy.bytes().len()).ok_or("proxy byte count overflow")?;
        if bytes > MAX_PROXY_INPUT_BYTES {
            return Err("aggregate proxy bytes exceed bounded request input".into());
        }
    }
    Ok(())
}

fn validate_opaque_id(id: &str) -> Result<(), String> {
    let count = id.chars().count();
    if count == 0 || count > MAX_OPAQUE_ID_CHARS || !id.bytes().all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.' | b'~'))
    {
        return Err("proxy ID must be a bounded opaque ASCII token".into());
    }
    Ok(())
}

/// Construct one strict structured-output request. This function has no network or write effect.
pub fn request_body(model: &str, proxies: &[ProxyInput<'_>], options: RequestOptions) -> Result<Value, String> {
    validate_model(model)?;
    validate_inputs(proxies)?;
    options.validate()?;
    let admission_reserve = admission_reserve_usd(model)?;
    if options.budget_usd < admission_reserve {
        return Err("budget is below conservative culling admission reserve".into());
    }
    let catalog = models()?;
    let model_info = catalog
        .get("models")
        .and_then(Value::as_array)
        .and_then(|items| items.iter().find(|item| item.get("id").and_then(Value::as_str) == Some(model)))
        .ok_or("missing model limits")?;
    let input_price = model_info.get("inputUsdPerMillion").ok_or("missing model input price")?;
    let output_price = model_info.get("outputUsdPerMillion").ok_or("missing model output price")?;

    let mut content = Vec::with_capacity(proxies.len().saturating_mul(2).saturating_add(1));
    content.push(json!({"type":"text", "text":"Compare only these opaque proxy IDs. Do not infer names, paths, metadata, faces, eyes, or aesthetics. Return one item per ID."}));
    for input in proxies {
        content.push(json!({"type":"text", "text":format!("Opaque ID: {}. Proxy SHA256: {}.", input.id, input.proxy.digest())}));
        content.push(json!({"type":"image_url", "image_url":{"url":format!("data:image/png;base64,{}", STANDARD.encode(input.proxy.bytes()))}}));
    }
    let body = json!({
        "model": model,
        "stream": false,
        "max_tokens": MAX_OUTPUT_TOKENS,
        "reasoning": {"effort": "low", "exclude": true},
        "provider": {"allow_fallbacks": false, "require_parameters": true, "data_collection": "deny", "zdr": true, "max_price": {"prompt": input_price, "completion": output_price}},
        "messages": [
            {"role":"system", "content": PROMPT},
            {"role":"user", "content": content}
        ],
        "response_format": {"type":"json_schema", "json_schema":{"name":"photo_culling_v1", "strict":true, "schema":schema()?}}
    });
    let encoded = serde_json::to_vec(&body).map_err(|_| "could not encode culling request")?;
    if encoded.len() > MAX_REQUEST_BYTES {
        return Err("culling request exceeds 4 MiB byte limit".into());
    }
    if estimate_input_tokens(encoded.len()) > MAX_INPUT_TOKENS_RESERVE {
        return Err("culling request exceeds conservative 20K input-token admission bound".into());
    }
    Ok(body)
}

/// Calculate frozen-rate admission reserve, not provider spend or billing ceiling.
pub fn admission_reserve_usd(model: &str) -> Result<f64, String> {
    validate_model(model)?;
    let catalog = models()?;
    let model_info = catalog
        .get("models")
        .and_then(Value::as_array)
        .and_then(|items| items.iter().find(|item| item.get("id").and_then(Value::as_str) == Some(model)))
        .ok_or("missing model limits")?;
    let input = model_info.get("inputUsdPerMillion").and_then(Value::as_f64).ok_or("missing model input price")?;
    let output = model_info.get("outputUsdPerMillion").and_then(Value::as_f64).ok_or("missing model output price")?;
    if !input.is_finite() || input < 0.0 || !output.is_finite() || output < 0.0 {
        return Err("invalid pinned model prices".into());
    }
    let reserve = (MAX_INPUT_TOKENS_RESERVE as f64 * input + MAX_OUTPUT_TOKENS as f64 * output) / 1_000_000.0;
    if !reserve.is_finite() {
        return Err("admission reserve overflow".into());
    }
    Ok(reserve)
}

/// Estimate input tokens conservatively from encoded request bytes for multi-image admission.
/// Division by two intentionally overestimates typical JSON/image tokenization to keep
/// admission bounded without making claims about provider token accounting.
pub fn estimate_input_tokens(encoded_bytes: usize) -> u64 {
    encoded_bytes.saturating_add(1).saturating_div(2) as u64
}

pub fn schema() -> Result<Value, String> {
    serde_json::from_str(SCHEMA).map_err(|_| "invalid embedded culling schema".into())
}

/// Validate complete model output against exact input ID coverage & read-only policy.
pub fn validate_result(proxies: &[ProxyInput<'_>], result: &CullingResult) -> Result<(), String> {
    validate_inputs(proxies)?;
    if result.proxy_quality != ProxyQuality::Limited512
        || result.sharpness != SignalStatus::Unavailable
        || result.blink != SignalStatus::Unavailable
        || result.calibration != CalibrationStatus::Unknown
    {
        return Err("result must preserve 512px and uncalibrated signal limits".into());
    }
    let input_ids: BTreeSet<&str> = proxies.iter().map(|input| input.id).collect();
    if result.items.len() != input_ids.len() {
        return Err("result must contain exactly one item per input proxy".into());
    }
    let mut item_ids = BTreeSet::new();
    for item in &result.items {
        if !input_ids.contains(item.id.as_str()) || !item_ids.insert(item.id.as_str()) {
            return Err("result contains hallucinated or duplicate proxy ID".into());
        }
        validate_reason(item)?;
        if item.confidence_kind != ConfidenceKind::Unqualified {
            return Err("confidence must remain explicitly unqualified".into());
        }
        if let Some(value) = item.confidence
            && (!value.is_finite() || !(0.0..=1.0).contains(&value))
        {
            return Err("confidence must be finite in 0..1".into());
        }
    }
    if item_ids.len() != input_ids.len() {
        return Err("result omits one or more input proxy IDs".into());
    }

    let mut winners = BTreeSet::new();
    for id in &result.acceptable_winner_ids {
        if !input_ids.contains(id.as_str()) || !winners.insert(id.as_str()) {
            return Err("acceptable winners must be unique input IDs".into());
        }
        let item = result.items.iter().find(|item| item.id.as_str() == id.as_str()).ok_or("winner missing item")?;
        if item.decision != Decision::Pick || item.uncertainty == Uncertainty::Unknown || item.confidence.is_none() {
            return Err("winner must be an explicit, sufficiently qualified pick".into());
        }
    }
    for item in &result.items {
        let selected = winners.contains(item.id.as_str());
        if item.decision == Decision::Pick && !selected {
            return Err("every pick must be listed as an acceptable winner".into());
        }
        if item.decision != Decision::Pick && selected {
            return Err("only picks may be acceptable winners".into());
        }
    }
    Ok(())
}

fn validate_reason(item: &CullingItem) -> Result<(), String> {
    if item.reason_codes.is_empty()
        || item.reason_codes.len() > 4
        || item.reason.trim().is_empty()
        || item.reason.chars().count() > MAX_REASON_CHARS
        || item.reason.chars().any(char::is_control)
    {
        return Err("invalid culling reason".into());
    }
    let lowered = item.reason.to_ascii_lowercase();
    for forbidden in ["face", "eye", "aesthetic", "beautiful", "beauty", "expression", "focus", "sharp", "clipping", "clip", "blur", "blink"] {
        if lowered.contains(forbidden) {
            return Err("culling reason contains prohibited visual claim".into());
        }
    }
    Ok(())
}

/// Parse provider envelope while retaining sanitized billing usage in receipt.
pub fn parse_response(model: &str, proxies: &[ProxyInput<'_>], options: RequestOptions, latency_ms: u64, body: &[u8]) -> Receipt {
    let proxy_receipts = proxies.iter().map(|input| ProxyReceipt { id: input.id.to_owned(), sha256: input.proxy.digest().to_owned() }).collect();
    let value = if body.len() <= MAX_RESPONSE_BYTES { serde_json::from_slice(body).unwrap_or(Value::Null) } else { Value::Null };
    let string = |key: &str| value.get(key).and_then(Value::as_str).filter(|text| text.len() <= 256).map(str::to_owned);
    let usage = sanitized_usage(value.get("usage"));
    let billing_status = if usage.get("cost").is_some() { BillingStatus::Reported } else { BillingStatus::Unknown };
    let admission_reserve = admission_reserve_usd(model).unwrap_or(0.0);
    let mut receipt = Receipt {
        version: VERSION,
        requested_model: model.to_owned(),
        served_model: string("model"),
        provider: string("provider"),
        response_id: string("id"),
        proxies: proxy_receipts,
        budget_usd: options.budget_usd,
        budget_policy: BudgetPolicy::AdmissionReserve,
        admission_reserve_usd: admission_reserve,
        billing_status,
        deadline_secs: options.deadline_secs,
        latency_ms,
        usage,
        result: None,
        error: None,
    };
    let outcome = (|| {
        validate_model(model)?;
        validate_inputs(proxies)?;
        options.validate()?;
        if value.get("error").is_some_and(|entry| !entry.is_null()) {
            return Err("provider returned an error".into());
        }
        if receipt.served_model.as_deref() != Some(model) {
            return Err("served model differs from requested model".into());
        }
        let choices = value.get("choices").and_then(Value::as_array).filter(|items| items.len() == 1).ok_or("expected exactly one completion")?;
        let choice = choices.first().ok_or("missing completion")?;
        if choice.get("finish_reason").and_then(Value::as_str) != Some("stop") {
            return Err("incomplete or blocked completion".into());
        }
        let message = choice.get("message").ok_or("missing message")?;
        if message.get("refusal").is_some_and(|entry| !entry.is_null())
            || message.get("tool_calls").is_some_and(|entry| entry.as_array().is_none_or(|items| !items.is_empty()))
        {
            return Err("refusal or unexpected tool call".into());
        }
        let content =
            message.get("content").and_then(Value::as_str).filter(|text| text.len() <= 16 * 1024).ok_or("missing/oversized culling JSON")?;
        let result: CullingResult = serde_json::from_str(content).map_err(|_| "culling result does not match schema")?;
        validate_result(proxies, &result)?;
        Ok(result)
    })();
    match outcome {
        Ok(result) => receipt.result = Some(result),
        Err(error) => receipt.error = Some(error),
    }
    receipt
}

fn sanitized_usage(value: Option<&Value>) -> Value {
    let Some(value) = value else { return Value::Null };
    let mut usage = serde_json::Map::new();
    for name in ["prompt_tokens", "completion_tokens", "total_tokens"] {
        if let Some(number) = value.get(name).and_then(Value::as_u64) {
            usage.insert(name.to_owned(), json!(number));
        }
    }
    if let Some(number) = value.get("cost").and_then(Value::as_f64).filter(|number| number.is_finite() && *number >= 0.0) {
        usage.insert("cost".into(), json!(number));
    }
    Value::Object(usage)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn proxy(extra_chunk: bool) -> Result<Proxy, String> {
        proxy_with_data(extra_chunk, 1)
    }

    fn proxy_with_data(extra_chunk: bool, data_len: usize) -> Result<Proxy, String> {
        let mut png = b"\x89PNG\r\n\x1a\n".to_vec();
        let mut chunk = |kind: &[u8; 4], data: &[u8]| {
            png.extend_from_slice(&(data.len() as u32).to_be_bytes());
            png.extend_from_slice(kind);
            png.extend_from_slice(data);
            png.extend_from_slice(&[0; 4]);
        };
        chunk(b"IHDR", &[0, 0, 0, 2, 0, 0, 0, 2, 8, 2, 0, 0, 0]);
        if extra_chunk {
            chunk(b"tEXt", b"metadata");
        }
        let data = vec![0; data_len];
        chunk(b"IDAT", &data);
        chunk(b"IEND", &[]);
        Proxy::new(png)
    }

    fn valid_result() -> CullingResult {
        CullingResult {
            proxy_quality: ProxyQuality::Limited512,
            sharpness: SignalStatus::Unavailable,
            blink: SignalStatus::Unavailable,
            calibration: CalibrationStatus::Unknown,
            acceptable_winner_ids: vec!["p0".into()],
            items: vec![
                CullingItem {
                    id: "p0".into(),
                    decision: Decision::Pick,
                    reason_codes: vec![ReasonCode::RelativeContent],
                    reason: "clearer repeated content".into(),
                    uncertainty: Uncertainty::Elevated,
                    confidence: Some(0.8),
                    confidence_kind: ConfidenceKind::Unqualified,
                },
                CullingItem {
                    id: "p1".into(),
                    decision: Decision::Reject,
                    reason_codes: vec![ReasonCode::TimeSimilarity],
                    reason: "near duplicate".into(),
                    uncertainty: Uncertainty::Elevated,
                    confidence: Some(0.7),
                    confidence_kind: ConfidenceKind::Unqualified,
                },
            ],
        }
    }

    #[test]
    fn rejects_hallucinated_or_missing_ids() {
        let proxy = proxy(false).expect("valid proxy fixture must construct");
        let mut result = valid_result();
        if let Some(item) = result.items.get_mut(1) {
            item.id = "ghost".into();
        }
        let inputs = [ProxyInput { id: "p0", proxy: &proxy }, ProxyInput { id: "p1", proxy: &proxy }];
        assert!(validate_result(&inputs, &result).is_err());
        let mut result = valid_result();
        result.items.pop();
        assert!(validate_result(&inputs, &result).is_err());
    }

    #[test]
    fn rejects_uncertain_or_invalid_winners() {
        let proxy = proxy(false).expect("valid proxy fixture must construct");
        let inputs = [ProxyInput { id: "p0", proxy: &proxy }, ProxyInput { id: "p1", proxy: &proxy }];
        let mut result = valid_result();
        if let Some(item) = result.items.first_mut() {
            item.uncertainty = Uncertainty::Unknown;
        }
        assert!(validate_result(&inputs, &result).is_err());
        let mut result = valid_result();
        if let Some(item) = result.items.first_mut() {
            item.decision = Decision::Keep;
        }
        assert!(validate_result(&inputs, &result).is_err());
    }

    #[test]
    fn accepts_explicit_tied_winners() {
        let proxy = proxy(false).expect("valid proxy fixture must construct");
        let inputs = [ProxyInput { id: "p0", proxy: &proxy }, ProxyInput { id: "p1", proxy: &proxy }];
        let mut result = valid_result();
        result.acceptable_winner_ids.push("p1".into());
        if let Some(item) = result.items.get_mut(1) {
            item.decision = Decision::Pick;
            item.reason_codes = vec![ReasonCode::TieForBest];
            item.reason = "equally supported candidate".into();
        }
        assert!(validate_result(&inputs, &result).is_ok());
    }

    #[test]
    fn rejects_size_duplicate_and_metadata_ids() {
        assert!(validate_opaque_id("path/id").is_err());
        assert!(validate_opaque_id(" ").is_err());
        let valid_proxy = proxy(false).expect("valid proxy fixture must construct");
        let duplicate = [ProxyInput { id: "p0", proxy: &valid_proxy }, ProxyInput { id: "p0", proxy: &valid_proxy }];
        assert!(validate_inputs(&duplicate).is_err());
        let too_few = [ProxyInput { id: "p0", proxy: &valid_proxy }];
        assert!(validate_inputs(&too_few).is_err());
        assert!(proxy(true).is_err());
    }

    #[test]
    fn reserves_budget_before_upload_and_bounds_multiimage_tokens() {
        let reserve = admission_reserve_usd("deepseek/deepseek-v4.1-flash").expect("cost fixture must resolve");
        assert!(reserve.is_finite() && reserve > 0.0);
        let proxy = proxy(false).expect("valid proxy fixture must construct");
        let inputs = [ProxyInput { id: "p0", proxy: &proxy }, ProxyInput { id: "p1", proxy: &proxy }];
        let too_small = RequestOptions { budget_usd: reserve / 2.0, ..RequestOptions::default() };
        assert!(request_body("deepseek/deepseek-v4.1-flash", &inputs, too_small).is_err());
        let large_proxy = proxy_with_data(false, 50_000).expect("large valid proxy fixture must construct");
        let large_inputs = [ProxyInput { id: "p0", proxy: &large_proxy }, ProxyInput { id: "p1", proxy: &large_proxy }];
        assert!(request_body("deepseek/deepseek-v4.1-flash", &large_inputs, RequestOptions::default()).is_err());
    }
}
