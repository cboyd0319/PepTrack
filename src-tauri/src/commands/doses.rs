use anyhow::Result;
use peptrack_core::models::DoseLog;
use serde::Deserialize;
use tauri::State;

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct LogDosePayload {
    pub protocol_id: String,
    pub site: String,
    pub amount_mg: f32,
    pub notes: Option<String>,
}

/// Logs a new dose
#[tauri::command]
pub async fn log_dose(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: LogDosePayload,
) -> Result<DoseLog, String> {
    let mut log = DoseLog::new(payload.protocol_id, payload.site, payload.amount_mg);
    log.notes = payload.notes;

    state
        .storage
        .append_dose_log(&log)
        .map_err(|err| err.to_string())?;

    Ok(log)
}

/// Lists all dose logs
#[tauri::command]
pub async fn list_dose_logs(state: State<'_, std::sync::Arc<AppState>>) -> Result<Vec<DoseLog>, String> {
    state.storage.list_dose_logs().map_err(|err| err.to_string())
}

/// Lists dose logs for a specific protocol
#[tauri::command]
pub async fn list_dose_logs_for_protocol(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
) -> Result<Vec<DoseLog>, String> {
    state
        .storage
        .list_dose_logs_for_protocol(&protocol_id)
        .map_err(|err| err.to_string())
}

/// Deletes a specific dose log
#[tauri::command]
pub async fn delete_dose_log(
    state: State<'_, std::sync::Arc<AppState>>,
    log_id: String,
) -> Result<(), String> {
    state
        .storage
        .delete_dose_log(&log_id)
        .map_err(|err| err.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_log_dose_payload_serialization() {
        let json = r#"{
            "protocolId": "protocol-123",
            "site": "deltoid",
            "amountMg": 5.5,
            "notes": "Morning dose"
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.protocol_id, "protocol-123");
        assert_eq!(payload.site, "deltoid");
        assert_eq!(payload.amount_mg, 5.5);
        assert_eq!(payload.notes, Some("Morning dose".to_string()));
    }

    #[test]
    fn test_log_dose_payload_without_notes() {
        let json = r#"{
            "protocolId": "protocol-456",
            "site": "abdomen",
            "amountMg": 10.0
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.protocol_id, "protocol-456");
        assert_eq!(payload.site, "abdomen");
        assert_eq!(payload.amount_mg, 10.0);
        assert_eq!(payload.notes, None);
    }

    #[test]
    fn test_log_dose_payload_with_decimal_amount() {
        let json = r#"{
            "protocolId": "p1",
            "site": "thigh",
            "amountMg": 2.75
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.amount_mg, 2.75);
    }

    #[test]
    fn test_log_dose_payload_with_zero_amount() {
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 0.0
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.amount_mg, 0.0);
    }

    #[test]
    fn test_log_dose_payload_with_large_amount() {
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 999.999
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert!((payload.amount_mg - 999.999).abs() < 0.001);
    }

    #[test]
    fn test_log_dose_payload_with_empty_notes() {
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 5.0,
            "notes": ""
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.notes, Some("".to_string()));
    }

    #[test]
    fn test_log_dose_payload_with_special_characters_in_site() {
        let json = r#"{
            "protocolId": "p1",
            "site": "left deltoid (upper)",
            "amountMg": 5.0
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.site, "left deltoid (upper)");
    }

    #[test]
    fn test_log_dose_payload_with_unicode_notes() {
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 5.0,
            "notes": "Test with émojis 💉 and ünicode"
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.notes, Some("Test with émojis 💉 and ünicode".to_string()));
    }

    #[test]
    fn test_log_dose_payload_missing_required_field() {
        // Missing protocol_id
        let json = r#"{
            "site": "test",
            "amountMg": 5.0
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_err());
    }

    #[test]
    fn test_log_dose_payload_invalid_amount_type() {
        // String instead of number for amountMg
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": "not a number"
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_err());
    }

    #[test]
    fn test_log_dose_payload_null_notes() {
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 5.0,
            "notes": null
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.notes, None);
    }

    #[test]
    fn test_log_dose_payload_camel_case_conversion() {
        // Verify that camelCase is properly handled
        let json = r#"{
            "protocolId": "p1",
            "site": "test",
            "amountMg": 5.0
        }"#;

        let payload: Result<LogDosePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        // snake_case fields should be populated from camelCase JSON
        let payload = payload.unwrap();
        assert_eq!(payload.protocol_id, "p1");
        assert_eq!(payload.amount_mg, 5.0);
    }

    #[test]
    fn test_log_dose_payload_debug_format() {
        let payload = LogDosePayload {
            protocol_id: "p1".to_string(),
            site: "test".to_string(),
            amount_mg: 5.0,
            notes: Some("test notes".to_string()),
        };

        let debug_str = format!("{:?}", payload);
        assert!(debug_str.contains("LogDosePayload"));
        assert!(debug_str.contains("p1"));
        assert!(debug_str.contains("test"));
        assert!(debug_str.contains("5.0") || debug_str.contains("5"));
    }
}
