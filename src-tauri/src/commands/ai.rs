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
pub async fn check_ai_availability(state: State<'_, AppState>) -> Result<AiAvailabilityStatus, String> {
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
    state: State<'_, AppState>,
    payload: SummarizePayload,
) -> Result<SummarizeResult, String> {
    info!("Summarizing text: title='{}'", payload.title);

    let request = SummarizeRequest {
        title: payload.title.clone(),
        content: payload.content,
        format: payload.format.unwrap_or(SummaryFormat::Markdown),
    };

    let response = state
        .ai_client
        .summarize(request)
        .await
        .map_err(|err| {
            warn!("AI summarization failed: {:#}", err);
            format!("AI summarization failed: {}. Make sure Codex CLI or Claude CLI is installed.", err)
        })?;

    info!("Summarization successful using {:?}", response.provider);

    Ok(SummarizeResult {
        provider: format!("{:?}", response.provider),
        output: response.raw_output,
    })
}
