//! Read-only experiment contract, available to all command hosts. Network runs outside owner.
use super::{CommandSpec, always, bad, cmd};
use serde_json::{Value, json};

pub fn specs() -> Vec<CommandSpec> {
    vec![
        cmd!(query "ai.models", "Photo Assessment Models", [], None, "{} — experimental OpenRouter shortlist, no network", always, |_, _| {
            lightcraft_photo_ai::models().map_err(|e| bad("ai.models", e))
        }),
        cmd!(query "ai.schema", "Photo Assessment Schema", [], None, "{} — typed read-only proposal schema, no network", always, |_, _| {
            Ok(json!({"version":lightcraft_photo_ai::VERSION,"schema":lightcraft_photo_ai::schema().map_err(|e| bad("ai.schema", e))?}))
        }),
        cmd!(query "ai.validate", "Validate Photo Assessment", [], None, "{assessment: object} — validate only; never applies an edit or flag", always, |_, p| {
            let assessment: lightcraft_photo_ai::Assessment = serde_json::from_value(p.get("assessment").cloned().unwrap_or(Value::Null)).map_err(|_| bad("ai.validate", "assessment does not match schema"))?;
            assessment.validate().map_err(|e| bad("ai.validate", e))?;
            Ok(json!({"valid":true,"applied":false,"assessment":assessment}))
        }),
    ]
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn validation_has_no_catalog_history_or_settings_side_effects() {
        let mut session = crate::Session::with_demo();
        let before = serde_json::to_value(&session.catalog).unwrap();
        let undo = session.undo.len();
        let result = session
            .execute("ai.validate", &json!({"assessment":{"decision":"review","confidence":0.1,"reason":"Uncertain intent","recipe":null}}))
            .unwrap();
        assert_eq!(result["applied"], false);
        assert_eq!(serde_json::to_value(&session.catalog).unwrap(), before);
        assert_eq!(session.undo.len(), undo);
        assert!(session.execute("ai.models", &json!({})).is_ok());
        assert!(session.execute("ai.schema", &json!({})).is_ok());
    }
}
