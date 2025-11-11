use std::path::PathBuf;
use std::process::Stdio;

use anyhow::{anyhow, Context, Result};
use async_trait::async_trait;
use regex::Regex;
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
    let fallback_regex = Regex::new(r#""text"\s*:\s*"([^"]+)""#).ok();

    for line in text.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(value) = serde_json::from_str::<Value>(line) {
            if let Some(text) = value
                .pointer("/data/text")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                last_message = Some(text);
                continue;
            }

            if let Some(text) = value
                .pointer("/data/message/content")
                .and_then(|v| v.as_str())
                .map(|s| s.to_string())
            {
                last_message = Some(text);
            }
        }
    }

    if last_message.is_none() {
        if let Some(regex) = fallback_regex {
            if let Some(caps) = regex.captures(&text) {
                last_message = Some(caps[1].to_string());
            }
        }
    }

    last_message
}
