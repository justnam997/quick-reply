use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::config::SlackConfig;
use crate::error::{QuickReplyError, Result};
use crate::models::{Conversation, Message, Notification};
use crate::platform::{DeliveryConfirmation, PlatformAdapter};

pub struct SlackAdapter {
    token: String,
    http: HttpClient,
    connected: bool,
}

impl SlackAdapter {
    pub fn new(config: SlackConfig) -> Self {
        Self {
            token: config.user_token,
            http: HttpClient::new(),
            connected: false,
        }
    }

    async fn api_get<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {
        let url = format!("https://slack.com/api/{endpoint}");
        let resp = self
            .http
            .get(&url)
            .bearer_auth(&self.token)
            .query(params)
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(resp)
    }

    async fn api_post<T: serde::de::DeserializeOwned>(
        &self,
        endpoint: &str,
        body: &serde_json::Value,
    ) -> Result<T> {
        let url = format!("https://slack.com/api/{endpoint}");
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.token)
            .json(body)
            .send()
            .await?
            .json::<T>()
            .await?;
        Ok(resp)
    }
}

#[derive(Deserialize)]
struct SlackResponse<T> {
    ok: bool,
    #[serde(flatten)]
    data: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct ConversationsData {
    channels: Vec<SlackChannel>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct SlackChannel {
    id: String,
    name: Option<String>,
    user: Option<String>,
    is_im: Option<bool>,
    #[serde(default)]
    unread_count_display: u32,
}

#[derive(Deserialize)]
struct HistoryData {
    messages: Vec<SlackMessage>,
}

#[derive(Deserialize)]
struct SlackMessage {
    ts: String,
    text: Option<String>,
    user: Option<String>,
    #[serde(default)]
    subtype: Option<String>,
}

#[derive(Deserialize)]
struct PostMessageData {
    ts: Option<String>,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct AuthTestData {
    user_id: Option<String>,
}

#[async_trait]
impl PlatformAdapter for SlackAdapter {
    fn name(&self) -> &str {
        "Slack"
    }

    async fn connect(&mut self) -> Result<()> {
        let resp: SlackResponse<AuthTestData> = self.api_get("auth.test", &[]).await?;
        if !resp.ok {
            return Err(QuickReplyError::Platform {
                platform: "Slack".to_string(),
                message: format!(
                    "Auth failed: {}",
                    resp.error.unwrap_or_else(|| "unknown".to_string())
                ),
            });
        }
        self.connected = true;
        tracing::info!("Slack connected");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn fetch_conversations(&self) -> Result<Vec<Conversation>> {
        let resp: SlackResponse<ConversationsData> = self
            .api_get(
                "conversations.list",
                &[
                    ("types", "im,mpim,public_channel,private_channel"),
                    ("limit", "50"),
                ],
            )
            .await?;

        if !resp.ok {
            return Err(QuickReplyError::Platform {
                platform: "Slack".to_string(),
                message: resp.error.unwrap_or_else(|| "API error".to_string()),
            });
        }

        let channels = resp.data.map(|d| d.channels).unwrap_or_default();

        let conversations = channels
            .into_iter()
            .map(|ch| {
                let name = ch.name.or(ch.user).unwrap_or_else(|| ch.id.clone());

                Conversation {
                    id: ch.id,
                    platform: "Slack".to_string(),
                    participants: vec![name],
                    last_active: chrono::Utc::now(),
                    last_message_preview: None,
                    unread_count: ch.unread_count_display,
                }
            })
            .collect();

        Ok(conversations)
    }

    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>> {
        let resp: SlackResponse<HistoryData> = self
            .api_get(
                "conversations.history",
                &[("channel", conv_id), ("limit", &limit.to_string())],
            )
            .await?;

        if !resp.ok {
            return Err(QuickReplyError::Platform {
                platform: "Slack".to_string(),
                message: resp.error.unwrap_or_else(|| "API error".to_string()),
            });
        }

        let slack_messages = resp.data.map(|d| d.messages).unwrap_or_default();

        let mut messages: Vec<Message> = slack_messages
            .into_iter()
            .filter(|m| m.subtype.is_none())
            .map(|m| {
                let ts_f64: f64 = m.ts.parse().unwrap_or(0.0);
                let timestamp = chrono::DateTime::from_timestamp(ts_f64 as i64, 0)
                    .unwrap_or_else(chrono::Utc::now);

                Message {
                    id: format!("slack_{}", m.ts),
                    conversation_id: conv_id.to_string(),
                    platform: "Slack".to_string(),
                    sender: m.user.unwrap_or_else(|| "Unknown".to_string()),
                    content: m.text.unwrap_or_default(),
                    timestamp,
                    is_read: true,
                    is_outgoing: false, // TODO: compare with authed user ID
                }
            })
            .collect();

        messages.reverse();
        Ok(messages)
    }

    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation> {
        let body = serde_json::json!({
            "channel": conv_id,
            "text": content,
        });

        let resp: SlackResponse<PostMessageData> = self.api_post("chat.postMessage", &body).await?;

        if !resp.ok {
            return Err(QuickReplyError::Platform {
                platform: "Slack".to_string(),
                message: format!(
                    "Send failed: {}",
                    resp.error.unwrap_or_else(|| "unknown".to_string())
                ),
            });
        }

        Ok(DeliveryConfirmation {
            message_id: resp
                .data
                .and_then(|d| d.ts)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            timestamp: chrono::Utc::now(),
            success: true,
        })
    }

    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>> {
        let (_tx, rx) = mpsc::unbounded_channel();
        // TODO: Implement Slack Socket Mode WebSocket for real-time notifications
        // For now, return an empty channel — Slack notifications require WebSocket setup
        tracing::warn!("Slack real-time notifications not yet implemented (requires Socket Mode)");
        Ok(rx)
    }
}
