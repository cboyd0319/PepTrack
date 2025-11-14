use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tracing::{instrument, warn};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SummaryFormat {
    Markdown,
    Json,
}

#[derive(Debug, Clone)]
pub struct SummarizeRequest {
    pub title: String,
    pub content: String,
    pub format: SummaryFormat,
}

#[derive(Debug, Clone)]
pub struct SummarizeResponse {
    pub provider: AiProvider,
    pub raw_output: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AiProvider {
    Codex,
    Claude,
}

#[async_trait]
pub trait LocalAiClient: Send + Sync {
    async fn summarize(&self, request: SummarizeRequest) -> Result<SummarizeResponse>;
}

#[derive(Debug, Clone)]
pub struct AiClientConfig {
    pub codex_model: String,
    pub claude_model: String,
    pub preferred: AiProvider,
}

impl Default for AiClientConfig {
    fn default() -> Self {
        Self {
            codex_model: "gpt-5".to_string(),
            claude_model: "claude-haiku-4-5".to_string(),
            preferred: AiProvider::Codex,
        }
    }
}

pub struct LocalAiOrchestrator {
    codex: Option<CodexCli>,
    claude: Option<ClaudeCli>,
    config: AiClientConfig,
}

impl LocalAiOrchestrator {
    pub fn detect(config: AiClientConfig) -> Self {
        let codex = which::which("codex").ok().map(|path| CodexCli {
            binary: path,
            model: config.codex_model.clone(),
        });
        let claude = which::which("claude").ok().map(|path| ClaudeCli {
            binary: path,
            model: config.claude_model.clone(),
        });

        Self {
            codex,
            claude,
            config,
        }
    }

    fn resolve_chain(&self) -> Vec<(AiProvider, Option<ProviderHandle>)> {
        let mut chain = Vec::new();
        match self.config.preferred {
            AiProvider::Codex => {
                chain.push((
                    AiProvider::Codex,
                    self.codex.clone().map(ProviderHandle::Codex),
                ));
                chain.push((
                    AiProvider::Claude,
                    self.claude.clone().map(ProviderHandle::Claude),
                ));
            }
            AiProvider::Claude => {
                chain.push((
                    AiProvider::Claude,
                    self.claude.clone().map(ProviderHandle::Claude),
                ));
                chain.push((
                    AiProvider::Codex,
                    self.codex.clone().map(ProviderHandle::Codex),
                ));
            }
        }
        chain
    }

    pub fn provider_chain(&self) -> Vec<AiProvider> {
        self.resolve_chain()
            .into_iter()
            .filter_map(|(provider, handle)| handle.map(|_| provider))
            .collect()
    }
}

#[cfg(test)]
impl LocalAiOrchestrator {
    pub(crate) fn with_providers(config: AiClientConfig, codex: bool, claude: bool) -> Self {
        let codex_handle = if codex {
            Some(CodexCli {
                binary: PathBuf::from("codex"),
                model: config.codex_model.clone(),
            })
        } else {
            None
        };

        let claude_handle = if claude {
            Some(ClaudeCli {
                binary: PathBuf::from("claude"),
                model: config.claude_model.clone(),
            })
        } else {
            None
        };

        Self {
            codex: codex_handle,
            claude: claude_handle,
            config,
        }
    }
}

#[async_trait]
impl LocalAiClient for LocalAiOrchestrator {
    #[instrument(skip_all, fields(title = %request.title))]
    async fn summarize(&self, request: SummarizeRequest) -> Result<SummarizeResponse> {
        for (provider, handle) in self.resolve_chain() {
            let Some(handle) = handle else {
                continue;
            };

            let result = match handle {
                ProviderHandle::Codex(cli) => cli.summarize(&request).await,
                ProviderHandle::Claude(cli) => cli.summarize(&request).await,
            };

            match result {
                Ok(mut response) => {
                    response.provider = provider;
                    return Ok(response);
                }
                Err(err) => {
                    warn!("Provider {provider:?} failed: {err:#}");
                }
            }
        }

        Err(anyhow!(
            "No available local AI providers (Codex CLI or Claude CLI not detected)."
        ))
    }
}

#[derive(Clone)]
enum ProviderHandle {
    Codex(CodexCli),
    Claude(ClaudeCli),
}

#[derive(Clone)]
struct CodexCli {
    binary: PathBuf,
    model: String,
}

impl CodexCli {
    async fn summarize(&self, request: &SummarizeRequest) -> Result<SummarizeResponse> {
        let prompt = build_summary_prompt(&request.title, &request.content, request.format);

        let mut cmd = Command::new(&self.binary);
        cmd.arg("exec")
            .arg("--json")
            .arg("--model")
            .arg(&self.model)
            .arg("-");

        cmd.stdin(Stdio::piped());
        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let mut child = cmd.spawn().context("Failed to spawn Codex CLI")?;
        if let Some(stdin) = child.stdin.as_mut() {
            stdin
                .write_all(prompt.as_bytes())
                .await
                .context("Failed to write prompt to Codex stdin")?;
        }

        let output = child
            .wait_with_output()
            .await
            .context("Codex CLI execution failed")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Codex CLI exited with code {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let parsed = parse_codex_json(&output.stdout)
            .unwrap_or_else(|| String::from_utf8_lossy(&output.stdout).to_string());

        Ok(SummarizeResponse {
            provider: AiProvider::Codex,
            raw_output: parsed,
        })
    }
}

#[derive(Clone)]
struct ClaudeCli {
    binary: PathBuf,
    model: String,
}

impl ClaudeCli {
    async fn summarize(&self, request: &SummarizeRequest) -> Result<SummarizeResponse> {
        let prompt = build_summary_prompt(&request.title, &request.content, request.format);

        let mut cmd = Command::new(&self.binary);
        cmd.arg("-p")
            .arg("--model")
            .arg(&self.model)
            .arg("--output-format")
            .arg("json")
            .arg(prompt);

        cmd.stdout(Stdio::piped());
        cmd.stderr(Stdio::piped());

        let output = cmd.output().await.context("Claude CLI execution failed")?;

        if !output.status.success() {
            return Err(anyhow!(
                "Claude CLI exited with code {:?}: {}",
                output.status.code(),
                String::from_utf8_lossy(&output.stderr)
            ));
        }

        let parsed = parse_claude_json(&output.stdout)
            .unwrap_or_else(|| String::from_utf8_lossy(&output.stdout).to_string());

        Ok(SummarizeResponse {
            provider: AiProvider::Claude,
            raw_output: parsed,
        })
    }
}

fn build_summary_prompt(title: &str, content: &str, format: SummaryFormat) -> String {
    // If content already starts with strong instructions (like "CRITICAL INSTRUCTION:"),
    // it's a complete prompt - don't wrap it
    if content.trim().starts_with("CRITICAL INSTRUCTION:") || content.contains("OUTPUT FORMAT") {
        return content.to_string();
    }

    // Otherwise, wrap with standard summarization instructions
    let instructions = match format {
        SummaryFormat::Markdown => "Generate a concise Markdown summary with safety flags, core findings, dosing insights, and citations.",
        SummaryFormat::Json => "Return a strict JSON object with keys: highlights[], dosing_notes[], safety_flags[].",
    };

    format!(
        "Summarize the following research context.\nTitle: {title}\nInstructions: {instructions}\n\nContent:\n{content}"
    )
}

#[derive(Deserialize)]
struct ClaudeJsonLine {
    text: Option<String>,
    #[serde(default)]
    message: Option<ClaudeMessage>,
}

#[derive(Deserialize)]
struct ClaudeMessage {
    content: Option<String>,
}

fn parse_claude_json(buffer: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(buffer);

    // Try to parse as a single JSON object first (new format)
    if let Ok(parsed) = serde_json::from_str::<ClaudeJsonLine>(&text) {
        if let Some(message) = parsed.message.and_then(|m| m.content) {
            return Some(message);
        }
        if let Some(text) = parsed.text {
            return Some(text);
        }
    }

    // Fall back to line-by-line parsing (streaming format)
    let mut last = None;
    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }

        if let Ok(parsed) = serde_json::from_str::<ClaudeJsonLine>(line) {
            if let Some(message) = parsed.message.and_then(|m| m.content) {
                last = Some(message);
                continue;
            }
            if let Some(text) = parsed.text {
                last = Some(text);
            }
        }
    }
    last
}

fn parse_codex_json(buffer: &[u8]) -> Option<String> {
    let text = String::from_utf8_lossy(buffer);
    let mut last_message = None;

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            // Codex CLI outputs {"type":"item.completed","item":{"text":"..."}}
            if let Some(text) = value
                .pointer("/item/text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                last_message = Some(text);
            }
        }
    }

    last_message
}

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Provider Chain Tests
    // =============================================================================

    #[test]
    fn provider_chain_prefers_codex_by_default() {
        let orchestrator =
            LocalAiOrchestrator::with_providers(AiClientConfig::default(), true, true);
        assert_eq!(
            orchestrator.provider_chain(),
            vec![AiProvider::Codex, AiProvider::Claude]
        );
    }

    #[test]
    fn provider_chain_skips_missing_providers() {
        let config = AiClientConfig {
            preferred: AiProvider::Claude,
            ..AiClientConfig::default()
        };
        let orchestrator = LocalAiOrchestrator::with_providers(config, false, true);
        assert_eq!(orchestrator.provider_chain(), vec![AiProvider::Claude]);
    }

    #[test]
    fn provider_chain_returns_empty_when_no_providers() {
        let config = AiClientConfig::default();
        let orchestrator = LocalAiOrchestrator::with_providers(config, false, false);
        assert_eq!(orchestrator.provider_chain(), Vec::<AiProvider>::new());
    }

    #[test]
    fn provider_chain_respects_claude_preference() {
        let config = AiClientConfig {
            preferred: AiProvider::Claude,
            ..AiClientConfig::default()
        };
        let orchestrator = LocalAiOrchestrator::with_providers(config, true, true);
        assert_eq!(
            orchestrator.provider_chain(),
            vec![AiProvider::Claude, AiProvider::Codex]
        );
    }

    // =============================================================================
    // Prompt Building Tests (SECURITY CRITICAL)
    // =============================================================================

    #[test]
    fn build_summary_prompt_wraps_simple_content() {
        let prompt = build_summary_prompt("Test Title", "Simple content", SummaryFormat::Markdown);

        assert!(prompt.contains("Test Title"));
        assert!(prompt.contains("Simple content"));
        assert!(prompt.contains("Markdown summary"));
        assert!(prompt.contains("safety flags"));
    }

    #[test]
    fn build_summary_prompt_uses_json_instructions() {
        let prompt = build_summary_prompt("Test", "Content", SummaryFormat::Json);

        assert!(prompt.contains("strict JSON"));
        assert!(prompt.contains("highlights[]"));
        assert!(prompt.contains("dosing_notes[]"));
        assert!(prompt.contains("safety_flags[]"));
    }

    #[test]
    fn build_summary_prompt_preserves_critical_instruction_prefix() {
        let content = "CRITICAL INSTRUCTION: Do not summarize, just extract data.\nPaper content...";
        let prompt = build_summary_prompt("Title", content, SummaryFormat::Markdown);

        // Should NOT wrap when content starts with CRITICAL INSTRUCTION:
        assert_eq!(prompt, content);
        assert!(!prompt.contains("Summarize the following"));
    }

    #[test]
    fn build_summary_prompt_preserves_output_format_directive() {
        let content = "OUTPUT FORMAT: JSON only\nPaper content...";
        let prompt = build_summary_prompt("Title", content, SummaryFormat::Markdown);

        // Should NOT wrap when content contains OUTPUT FORMAT
        assert_eq!(prompt, content);
        assert!(!prompt.contains("Summarize the following"));
    }

    #[test]
    fn build_summary_prompt_handles_unicode() {
        let prompt = build_summary_prompt(
            "测试标题",
            "內容 with émojis 🧪",
            SummaryFormat::Markdown
        );

        assert!(prompt.contains("测试标题"));
        assert!(prompt.contains("內容 with émojis 🧪"));
    }

    #[test]
    fn build_summary_prompt_handles_empty_content() {
        let prompt = build_summary_prompt("Title", "", SummaryFormat::Markdown);

        assert!(prompt.contains("Title"));
        // Should still create a valid prompt structure
        assert!(prompt.contains("Content:"));
    }

    #[test]
    fn build_summary_prompt_handles_very_long_content() {
        let long_content = "a".repeat(100_000);
        let prompt = build_summary_prompt("Title", &long_content, SummaryFormat::Markdown);

        assert!(prompt.contains(&long_content));
        assert!(prompt.len() > 100_000);
    }

    #[test]
    fn build_summary_prompt_handles_special_characters() {
        let content = "Content with <tags> and \"quotes\" and 'apostrophes' and & ampersands";
        let prompt = build_summary_prompt("Title", content, SummaryFormat::Markdown);

        assert!(prompt.contains(content));
        // Should not escape HTML entities (we're not outputting HTML)
        assert!(prompt.contains("<tags>"));
    }

    // =============================================================================
    // JSON Parsing Tests - Claude CLI (SECURITY CRITICAL)
    // =============================================================================

    #[test]
    fn parse_claude_json_extracts_single_object_with_message_content() {
        let json = r#"{"message": {"content": "This is the summary"}}"#;
        let result = parse_claude_json(json.as_bytes());

        assert_eq!(result, Some("This is the summary".to_string()));
    }

    #[test]
    fn parse_claude_json_extracts_text_field() {
        let json = r#"{"text": "Summary text here"}"#;
        let result = parse_claude_json(json.as_bytes());

        assert_eq!(result, Some("Summary text here".to_string()));
    }

    #[test]
    fn parse_claude_json_handles_streaming_format() {
        let json = r#"
{"text": "First chunk"}
{"text": "Second chunk"}
{"message": {"content": "Final message"}}
"#;
        let result = parse_claude_json(json.as_bytes());

        // Should return the LAST valid message
        assert_eq!(result, Some("Final message".to_string()));
    }

    #[test]
    fn parse_claude_json_handles_empty_input() {
        let result = parse_claude_json(b"");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_claude_json_handles_invalid_json() {
        let result = parse_claude_json(b"not json at all");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_claude_json_handles_malformed_streaming() {
        let json = r#"
{"text": "Valid"}
not json
{"message": {"content": "Also valid"}}
"#;
        let result = parse_claude_json(json.as_bytes());

        // Should extract the last valid message, ignoring invalid lines
        assert_eq!(result, Some("Also valid".to_string()));
    }

    #[test]
    fn parse_claude_json_handles_missing_fields() {
        let json = r#"{"other": "field"}"#;
        let result = parse_claude_json(json.as_bytes());

        assert_eq!(result, None);
    }

    #[test]
    fn parse_claude_json_handles_large_response() {
        let large_text = "a".repeat(10_000);
        let json = format!(r#"{{"text": "{}"}}"#, large_text);
        let result = parse_claude_json(json.as_bytes());

        assert_eq!(result, Some(large_text));
    }

    #[test]
    fn parse_claude_json_handles_unicode() {
        let json = r#"{"text": "Unicode: 测试 émojis 🧪"}"#;
        let result = parse_claude_json(json.as_bytes());

        assert_eq!(result, Some("Unicode: 测试 émojis 🧪".to_string()));
    }

    // =============================================================================
    // JSON Parsing Tests - Codex CLI (SECURITY CRITICAL)
    // =============================================================================

    #[test]
    fn parse_codex_json_extracts_item_completed_event() {
        let json = r#"{"type":"item.completed","item":{"text":"Summary output"}}"#;
        let result = parse_codex_json(json.as_bytes());

        assert_eq!(result, Some("Summary output".to_string()));
    }

    #[test]
    fn parse_codex_json_handles_streaming_events() {
        let json = r#"
{"type":"item.created"}
{"type":"item.in_progress"}
{"type":"item.completed","item":{"text":"Final output"}}
"#;
        let result = parse_codex_json(json.as_bytes());

        // Should return the LAST completed item
        assert_eq!(result, Some("Final output".to_string()));
    }

    #[test]
    fn parse_codex_json_handles_multiple_completions() {
        let json = r#"
{"type":"item.completed","item":{"text":"First"}}
{"type":"item.completed","item":{"text":"Second"}}
{"type":"item.completed","item":{"text":"Final"}}
"#;
        let result = parse_codex_json(json.as_bytes());

        // Should return the LAST one
        assert_eq!(result, Some("Final".to_string()));
    }

    #[test]
    fn parse_codex_json_handles_empty_input() {
        let result = parse_codex_json(b"");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_codex_json_handles_invalid_json() {
        let result = parse_codex_json(b"not json");
        assert_eq!(result, None);
    }

    #[test]
    fn parse_codex_json_handles_missing_text_field() {
        let json = r#"{"type":"item.completed","item":{}}"#;
        let result = parse_codex_json(json.as_bytes());

        assert_eq!(result, None);
    }

    #[test]
    fn parse_codex_json_handles_wrong_event_type() {
        let json = r#"{"type":"other.event","item":{"text":"Should not extract"}}"#;
        let result = parse_codex_json(json.as_bytes());

        assert_eq!(result, None);
    }

    #[test]
    fn parse_codex_json_handles_large_output() {
        let large_text = "a".repeat(10_000);
        let json = format!(r#"{{"type":"item.completed","item":{{"text":"{}"}}}}"#, large_text);
        let result = parse_codex_json(json.as_bytes());

        assert_eq!(result, Some(large_text));
    }

    #[test]
    fn parse_codex_json_handles_unicode() {
        let json = r#"{"type":"item.completed","item":{"text":"Unicode: 测试 🧪"}}"#;
        let result = parse_codex_json(json.as_bytes());

        assert_eq!(result, Some("Unicode: 测试 🧪".to_string()));
    }

    // =============================================================================
    // Config Tests
    // =============================================================================

    #[test]
    fn ai_client_config_default_values() {
        let config = AiClientConfig::default();

        assert_eq!(config.codex_model, "gpt-5");
        assert_eq!(config.claude_model, "claude-haiku-4-5");
        assert_eq!(config.preferred, AiProvider::Codex);
    }

    #[test]
    fn ai_provider_equality() {
        assert_eq!(AiProvider::Codex, AiProvider::Codex);
        assert_eq!(AiProvider::Claude, AiProvider::Claude);
        assert_ne!(AiProvider::Codex, AiProvider::Claude);
    }

    // =============================================================================
    // Format Tests
    // =============================================================================

    #[test]
    fn summary_format_equality() {
        assert_eq!(SummaryFormat::Markdown, SummaryFormat::Markdown);
        assert_eq!(SummaryFormat::Json, SummaryFormat::Json);
        assert_ne!(SummaryFormat::Markdown, SummaryFormat::Json);
    }
}
