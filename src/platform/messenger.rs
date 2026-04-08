use async_trait::async_trait;
use reqwest::Client as HttpClient;
use serde::Deserialize;
use tokio::sync::mpsc;

use crate::config::MessengerConfig;
use crate::error::{QuickReplyError, Result};
use crate::models::{Conversation, Message, Notification};
use crate::platform::{DeliveryConfirmation, PlatformAdapter};

/// Messenger adapter — TIER 2 (best-effort).
/// Communicates with a local Node sidecar (puppeteer bridge) via HTTP.
/// Facebook actively blocks automation, so this adapter may break.
const MAX_RESTART_ATTEMPTS: u32 = 3;

pub struct MessengerAdapter {
    base_url: String,
    http: HttpClient,
    connected: bool,
    failure_count: u32,
}

impl MessengerAdapter {
    pub fn new(config: MessengerConfig) -> Self {
        Self {
            base_url: format!("http://127.0.0.1:{}", config.bridge_port),
            http: HttpClient::new(),
            connected: false,
            failure_count: 0,
        }
    }
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct BridgeResponse<T> {
    #[serde(default)]
    ok: bool,
    data: Option<T>,
    error: Option<String>,
}

#[derive(Deserialize)]
struct BridgeConversation {
    id: String,
    name: String,
    last_message: Option<String>,
    unread: u32,
    last_active: i64,
}

#[derive(Deserialize)]
struct BridgeMessage {
    id: String,
    sender: String,
    content: String,
    timestamp: i64,
    is_outgoing: bool,
}

#[derive(Deserialize)]
struct BridgeSendResult {
    message_id: String,
}

#[async_trait]
impl PlatformAdapter for MessengerAdapter {
    fn name(&self) -> &str {
        "Messenger"
    }

    async fn connect(&mut self) -> Result<()> {
        if self.failure_count >= MAX_RESTART_ATTEMPTS {
            return Err(QuickReplyError::Platform {
                platform: "Messenger".to_string(),
                message: format!(
                    "Bridge failed {} times. Messenger is temporarily unavailable. \
                     Reconnect manually when ready.",
                    self.failure_count
                ),
            });
        }

        let resp = self
            .http
            .get(format!("{}/health", self.base_url))
            .timeout(std::time::Duration::from_secs(5))
            .send()
            .await
            .map_err(|_| {
                self.failure_count += 1;
                QuickReplyError::Platform {
                    platform: "Messenger".to_string(),
                    message: format!(
                        "Bridge not reachable at {}. Start the messenger-bridge sidecar. \
                         (Attempt {}/{})",
                        self.base_url, self.failure_count, MAX_RESTART_ATTEMPTS
                    ),
                }
            })?;

        if !resp.status().is_success() {
            self.failure_count += 1;
            return Err(QuickReplyError::Platform {
                platform: "Messenger".to_string(),
                message:
                    "Bridge health check failed. Session cookies may have expired. Re-auth needed."
                        .to_string(),
            });
        }

        self.failure_count = 0;
        self.connected = true;
        tracing::info!("Messenger bridge connected at {}", self.base_url);
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
        let resp: BridgeResponse<Vec<BridgeConversation>> = self
            .http
            .get(format!("{}/conversations", self.base_url))
            .send()
            .await?
            .json()
            .await?;

        let convs = resp
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|c| Conversation {
                id: c.id,
                platform: "Messenger".to_string(),
                participants: vec![c.name],
                last_active: chrono::DateTime::from_timestamp(c.last_active, 0)
                    .unwrap_or_else(chrono::Utc::now),
                last_message_preview: c.last_message,
                unread_count: c.unread,
            })
            .collect();

        Ok(convs)
    }

    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>> {
        let resp: BridgeResponse<Vec<BridgeMessage>> = self
            .http
            .get(format!("{}/messages/{}", self.base_url, conv_id))
            .query(&[("limit", limit.to_string())])
            .send()
            .await?
            .json()
            .await?;

        let messages = resp
            .data
            .unwrap_or_default()
            .into_iter()
            .map(|m| Message {
                id: m.id,
                conversation_id: conv_id.to_string(),
                platform: "Messenger".to_string(),
                sender: m.sender,
                content: m.content,
                timestamp: chrono::DateTime::from_timestamp(m.timestamp, 0)
                    .unwrap_or_else(chrono::Utc::now),
                is_read: true,
                is_outgoing: m.is_outgoing,
            })
            .collect();

        Ok(messages)
    }

    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation> {
        let body = serde_json::json!({ "content": content });

        let resp: BridgeResponse<BridgeSendResult> = self
            .http
            .post(format!("{}/messages/{}", self.base_url, conv_id))
            .json(&body)
            .send()
            .await?
            .json()
            .await?;

        if let Some(err) = resp.error {
            return Err(QuickReplyError::Platform {
                platform: "Messenger".to_string(),
                message: format!("Send failed: {err}"),
            });
        }

        Ok(DeliveryConfirmation {
            message_id: resp
                .data
                .map(|d| d.message_id)
                .unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
            timestamp: chrono::Utc::now(),
            success: true,
        })
    }

    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>> {
        let (_tx, rx) = mpsc::unbounded_channel();
        // TODO: WebSocket to sidecar for real-time notifications
        tracing::warn!("Messenger real-time notifications pending WebSocket integration (Tier 2)");
        Ok(rx)
    }
}
