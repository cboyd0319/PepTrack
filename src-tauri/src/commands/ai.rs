use peptrack_local_ai::{AiProvider, LocalAiClient, SummarizeRequest, SummaryFormat};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{info, warn};

use crate::state::AppState;

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SummarizePayload {
    pub title: String,
    pub content: String,
    pub format: Option<SummaryFormat>,
}

#[derive(Debug, Serialize)]
pub struct SummarizeResult {
    pub provider: String,
    pub output: String,
}

/// Checks which AI providers are available
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiAvailabilityStatus {
    pub codex_available: bool,
    pub claude_available: bool,
    pub any_available: bool,
    pub preferred_provider: Option<String>,
}

#[tauri::command]
pub async fn check_ai_availability(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<AiAvailabilityStatus, String> {
    let providers = state.ai_client.provider_chain();

    let codex_available = providers.iter().any(|p| matches!(p, AiProvider::Codex));
    let claude_available = providers.iter().any(|p| matches!(p, AiProvider::Claude));
    let any_available = !providers.is_empty();

    let preferred_provider = providers.first().map(|p| match p {
        AiProvider::Codex => "Codex (GPT-5)".to_string(),
        AiProvider::Claude => "Claude (Haiku 4.5)".to_string(),
    });

    if any_available {
        info!(
            "AI available: Codex={}, Claude={}, Preferred={:?}",
            codex_available, claude_available, preferred_provider
        );
    } else {
        warn!("No AI providers available (Codex CLI or Claude CLI not found in PATH)");
    }

    Ok(AiAvailabilityStatus {
        codex_available,
        claude_available,
        any_available,
        preferred_provider,
    })
}

#[tauri::command]
pub async fn summarize_text(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: SummarizePayload,
) -> Result<SummarizeResult, String> {
    info!("Summarizing text: title='{}'", payload.title);

    let request = SummarizeRequest {
        title: payload.title.clone(),
        content: payload.content,
        format: payload.format.unwrap_or(SummaryFormat::Markdown),
    };

    let response = state.ai_client.summarize(request).await.map_err(|err| {
        warn!("AI summarization failed: {:#}", err);
        format!(
            "AI summarization failed: {}. Make sure Codex CLI or Claude CLI is installed.",
            err
        )
    })?;

    info!("Summarization successful using {:?}", response.provider);

    Ok(SummarizeResult {
        provider: format!("{:?}", response.provider),
        output: response.raw_output,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_summarize_payload_serialization() {
        let json = r#"{
            "title": "Test Paper",
            "content": "This is test content",
            "format": "Markdown"
        }"#;

        let payload: Result<SummarizePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.title, "Test Paper");
        assert_eq!(payload.content, "This is test content");
        assert!(payload.format.is_some());
    }

    #[test]
    fn test_summarize_payload_without_format() {
        let json = r#"{
            "title": "Test",
            "content": "Content"
        }"#;

        let payload: Result<SummarizePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.title, "Test");
        assert_eq!(payload.content, "Content");
        assert!(payload.format.is_none());
    }

    #[test]
    fn test_summarize_payload_with_json_format() {
        let json = r#"{
            "title": "Test",
            "content": "Content",
            "format": "Json"
        }"#;

        let payload: Result<SummarizePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert!(payload.format.is_some());
    }

    #[test]
    fn test_summarize_payload_with_long_content() {
        let long_content = "a".repeat(10000);
        let json = format!(
            r#"{{
                "title": "Long Paper",
                "content": "{}"
            }}"#,
            long_content
        );

        let payload: Result<SummarizePayload, _> = serde_json::from_str(&json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert_eq!(payload.content.len(), 10000);
    }

    #[test]
    fn test_summarize_payload_with_special_characters() {
        let json = r#"{
            "title": "Test & <Special> \"Characters\"",
            "content": "Content with émojis 🧪 and symbols"
        }"#;

        let payload: Result<SummarizePayload, _> = serde_json::from_str(json);
        assert!(payload.is_ok());

        let payload = payload.unwrap();
        assert!(payload.title.contains("&"));
        assert!(payload.content.contains("🧪"));
    }

    #[test]
    fn test_summarize_result_serialization() {
        let result = SummarizeResult {
            provider: "Codex".to_string(),
            output: "Summary text".to_string(),
        };

        let json = serde_json::to_string(&result);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("provider"));
        assert!(json_str.contains("output"));
        assert!(json_str.contains("Codex"));
    }

    #[test]
    fn test_ai_availability_status_serialization() {
        let status = AiAvailabilityStatus {
            codex_available: true,
            claude_available: false,
            any_available: true,
            preferred_provider: Some("Codex (GPT-5)".to_string()),
        };

        let json = serde_json::to_string(&status);
        assert!(json.is_ok());

        let json_str = json.unwrap();
        assert!(json_str.contains("codexAvailable"));
        assert!(json_str.contains("claudeAvailable"));
        assert!(json_str.contains("anyAvailable"));
        assert!(json_str.contains("preferredProvider"));
    }

    #[test]
    fn test_ai_availability_status_all_available() {
        let status = AiAvailabilityStatus {
            codex_available: true,
            claude_available: true,
            any_available: true,
            preferred_provider: Some("Codex (GPT-5)".to_string()),
        };

        assert!(status.codex_available);
        assert!(status.claude_available);
        assert!(status.any_available);
        assert!(status.preferred_provider.is_some());
    }

    #[test]
    fn test_ai_availability_status_none_available() {
        let status = AiAvailabilityStatus {
            codex_available: false,
            claude_available: false,
            any_available: false,
            preferred_provider: None,
        };

        assert!(!status.codex_available);
        assert!(!status.claude_available);
        assert!(!status.any_available);
        assert!(status.preferred_provider.is_none());
    }

    #[test]
    fn test_ai_availability_status_camel_case_conversion() {
        let status = AiAvailabilityStatus {
            codex_available: true,
            claude_available: false,
            any_available: true,
            preferred_provider: Some("Codex".to_string()),
        };

        let json = serde_json::to_string(&status).unwrap();

        // Should convert to camelCase
        assert!(json.contains("codexAvailable"));
        assert!(json.contains("claudeAvailable"));
        assert!(json.contains("anyAvailable"));
        assert!(json.contains("preferredProvider"));

        // Should NOT contain snake_case
        assert!(!json.contains("codex_available"));
        assert!(!json.contains("claude_available"));
    }

    #[test]
    fn test_summarize_payload_missing_required_field() {
        let json = r#"{
            "title": "Test"
        }"#;

        let payload: Result<SummarizePayload, _> = serde_json::from_str(json);
        assert!(payload.is_err());
    }

    #[test]
    fn test_summarize_result_debug_format() {
        let result = SummarizeResult {
            provider: "Claude".to_string(),
            output: "Test summary".to_string(),
        };

        let debug_str = format!("{:?}", result);
        assert!(debug_str.contains("SummarizeResult"));
        assert!(debug_str.contains("Claude"));
        assert!(debug_str.contains("Test summary"));
    }
}
