use std::process::Stdio;
use std::time::Duration;

use tokio::io::AsyncWriteExt;
use tokio::process::Command;
use tokio::time::timeout;

use crate::config::AiConfig;
use crate::error::{QuickReplyError, Result};

pub struct AiSuggestionResult {
    pub text: String,
}

pub struct ClaudeCli {
    binary: String,
    timeout_secs: u64,
    available: bool,
}

impl ClaudeCli {
    pub async fn new(config: &AiConfig) -> Self {
        // Check if claude binary exists in PATH
        let available = Command::new("which")
            .arg(&config.claude_binary)
            .output()
            .await
            .map(|o| o.status.success())
            .unwrap_or(false);

        if !available {
            tracing::warn!(
                "Claude CLI '{}' not found in PATH. AI suggestions will be disabled.",
                config.claude_binary
            );
        }

        Self {
            binary: config.claude_binary.clone(),
            timeout_secs: config.suggestion_timeout_secs,
            available,
        }
    }

    pub fn is_available(&self) -> bool {
        self.available
    }

    pub async fn suggest_reply(
        &self,
        context_payload: &str,
        contact_name: &str,
    ) -> Result<AiSuggestionResult> {
        if !self.available {
            return Err(QuickReplyError::Claude(
                "Claude CLI not found. Install it or check PATH.".to_string(),
            ));
        }

        let prompt = format!(
            "You are helping me reply to a message from {contact_name}. \
             Based on the conversation context provided via stdin, \
             suggest a natural, concise reply that matches my usual tone. \
             Output ONLY the suggested reply text, nothing else."
        );

        let result = timeout(
            Duration::from_secs(self.timeout_secs),
            self.invoke(&prompt, context_payload),
        )
        .await
        .map_err(|_| {
            QuickReplyError::Claude(format!(
                "Suggestion timed out after {}s. Press r to retry.",
                self.timeout_secs
            ))
        })??;

        Ok(result)
    }

    async fn invoke(&self, prompt: &str, stdin_payload: &str) -> Result<AiSuggestionResult> {
        let mut child = Command::new(&self.binary)
            .arg("--print")
            .arg("-p")
            .arg(prompt)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .map_err(|e| {
                if e.kind() == std::io::ErrorKind::NotFound {
                    QuickReplyError::Claude(format!(
                        "Claude CLI '{}' not found. Install: https://docs.anthropic.com/claude-code",
                        self.binary
                    ))
                } else {
                    QuickReplyError::Claude(format!("Failed to spawn Claude CLI: {e}"))
                }
            })?;

        // Write context to stdin
        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(stdin_payload.as_bytes()).await?;
            // Drop stdin to signal EOF
        }

        let output = child.wait_with_output().await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);

            // Detect common error patterns
            if stderr.contains("auth") || stderr.contains("login") || stderr.contains("API key") {
                return Err(QuickReplyError::Claude(
                    "Claude CLI auth expired. Run `claude auth` in a separate terminal."
                        .to_string(),
                ));
            }
            if stderr.contains("rate") || stderr.contains("429") {
                return Err(QuickReplyError::Claude(
                    "Rate limited. Please wait before requesting another suggestion.".to_string(),
                ));
            }

            return Err(QuickReplyError::Claude(format!(
                "Claude CLI failed (exit {}): {}",
                output.status,
                stderr.chars().take(200).collect::<String>()
            )));
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();

        if text.is_empty() {
            return Err(QuickReplyError::Claude(
                "Claude returned empty response.".to_string(),
            ));
        }

        Ok(AiSuggestionResult { text })
    }
}
