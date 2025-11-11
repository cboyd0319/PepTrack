use peptrack_local_ai::{LocalAiClient, SummarizeRequest, SummaryFormat};
use serde::{Deserialize, Serialize};
use tauri::State;

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

#[tauri::command]
pub async fn summarize_text(
    state: State<'_, AppState>,
    payload: SummarizePayload,
) -> Result<SummarizeResult, String> {
    let request = SummarizeRequest {
        title: payload.title,
        content: payload.content,
        format: payload.format.unwrap_or(SummaryFormat::Markdown),
    };

    let response = state
        .ai_client
        .summarize(request)
        .await
        .map_err(|err| err.to_string())?;

    Ok(SummarizeResult {
        provider: format!("{:?}", response.provider),
        output: response.raw_output,
    })
}
