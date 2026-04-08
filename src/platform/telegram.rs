use std::path::PathBuf;
use std::sync::Arc;

use async_trait::async_trait;
use grammers_client::client::UpdatesConfiguration;
use grammers_client::message::InputMessage;
use grammers_client::session::storages::SqliteSession;
use grammers_client::update::Update;
use grammers_client::{Client, SenderPool};
use tokio::sync::mpsc;

use crate::config::TelegramConfig;
use crate::error::{QuickReplyError, Result};
use crate::models::{Conversation, Message, Notification};
use crate::platform::{DeliveryConfirmation, PlatformAdapter};

pub struct TelegramAdapter {
    config: TelegramConfig,
    client: Option<Client>,
    session_path: PathBuf,
    connected: bool,
}

impl TelegramAdapter {
    pub fn new(config: TelegramConfig, data_dir: &std::path::Path) -> Self {
        Self {
            config,
            client: None,
            session_path: data_dir.join("telegram.session"),
            connected: false,
        }
    }

    fn client(&self) -> Result<&Client> {
        self.client
            .as_ref()
            .ok_or_else(|| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: "Not connected".to_string(),
            })
    }
}

#[async_trait]
impl PlatformAdapter for TelegramAdapter {
    fn name(&self) -> &str {
        "Telegram"
    }

    async fn connect(&mut self) -> Result<()> {
        let session = SqliteSession::open(&self.session_path).await.map_err(|e| {
            QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Session error: {e}"),
            }
        })?;

        let pool = SenderPool::new(Arc::new(session), self.config.api_id);
        let client = Client::new(pool.handle);

        if !client
            .is_authorized()
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Auth check failed: {e}"),
            })?
        {
            // Request login code
            let _token = client
                .request_login_code(&self.config.phone, &self.config.api_hash)
                .await
                .map_err(|e| QuickReplyError::Platform {
                    platform: "Telegram".to_string(),
                    message: format!("Login code request failed: {e}"),
                })?;

            // TODO: In production, this should use a TUI modal to get the code
            return Err(QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: "Authorization required. Please run the auth flow first.".to_string(),
            });
        }

        self.client = Some(client);
        self.connected = true;
        tracing::info!("Telegram connected successfully");
        Ok(())
    }

    async fn disconnect(&mut self) -> Result<()> {
        self.client.take();
        self.connected = false;
        Ok(())
    }

    fn is_connected(&self) -> bool {
        self.connected
    }

    async fn fetch_conversations(&self) -> Result<Vec<Conversation>> {
        let client = self.client()?;
        let mut dialogs = client.iter_dialogs();
        let mut conversations = Vec::new();

        while let Some(dialog) = dialogs
            .next()
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Dialog fetch error: {e}"),
            })?
        {
            let peer = dialog.peer();
            let last_message = dialog.last_message.as_ref();

            let peer_name = peer.name().unwrap_or("Unknown").to_string();

            // Extract unread count from raw dialog
            let unread_count = match &dialog.raw {
                grammers_client::tl::enums::Dialog::Dialog(d) => d.unread_count as u32,
                grammers_client::tl::enums::Dialog::Folder(_) => 0,
            };

            conversations.push(Conversation {
                id: dialog.peer_id().to_string(),
                platform: "Telegram".to_string(),
                participants: vec![peer_name],
                last_active: last_message
                    .map(|m| m.date())
                    .unwrap_or_else(chrono::Utc::now),
                last_message_preview: last_message.map(|m| {
                    let text = m.text();
                    if text.len() > 50 {
                        format!("{}...", &text[..47])
                    } else {
                        text.to_string()
                    }
                }),
                unread_count,
            });

            if conversations.len() >= 50 {
                break;
            }
        }

        Ok(conversations)
    }

    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>> {
        let client = self.client()?;

        // Find the peer reference from dialogs
        let peer_ref = {
            let mut dialogs = client.iter_dialogs();
            let mut found = None;
            while let Some(dialog) =
                dialogs
                    .next()
                    .await
                    .map_err(|e| QuickReplyError::Platform {
                        platform: "Telegram".to_string(),
                        message: format!("Dialog iteration error: {e}"),
                    })?
            {
                if dialog.peer_id().to_string() == conv_id {
                    found = Some(dialog.peer_ref());
                    break;
                }
            }
            found.ok_or_else(|| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Chat not found: {conv_id}"),
            })?
        };

        let mut messages_iter = client.iter_messages(peer_ref);
        let mut messages = Vec::new();

        while let Some(msg) = messages_iter
            .next()
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Message fetch error: {e}"),
            })?
        {
            let sender_name = msg
                .sender()
                .and_then(|s| s.name().map(|n| n.to_string()))
                .unwrap_or_else(|| "Unknown".to_string());

            messages.push(Message {
                id: msg.id().to_string(),
                conversation_id: conv_id.to_string(),
                platform: "Telegram".to_string(),
                sender: sender_name,
                content: msg.text().to_string(),
                timestamp: msg.date(),
                is_read: true,
                is_outgoing: msg.outgoing(),
            });

            if messages.len() >= limit {
                break;
            }
        }

        // Reverse to show oldest first
        messages.reverse();
        Ok(messages)
    }

    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation> {
        let client = self.client()?;

        // Find peer reference
        let peer_ref = {
            let mut dialogs = client.iter_dialogs();
            let mut found = None;
            while let Some(dialog) =
                dialogs
                    .next()
                    .await
                    .map_err(|e| QuickReplyError::Platform {
                        platform: "Telegram".to_string(),
                        message: format!("Dialog iteration error: {e}"),
                    })?
            {
                if dialog.peer_id().to_string() == conv_id {
                    found = Some(dialog.peer_ref());
                    break;
                }
            }
            found.ok_or_else(|| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Chat not found: {conv_id}"),
            })?
        };

        let sent = client
            .send_message(peer_ref, InputMessage::new().text(content))
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "Telegram".to_string(),
                message: format!("Send failed: {e}"),
            })?;

        Ok(DeliveryConfirmation {
            message_id: sent.id().to_string(),
            timestamp: chrono::Utc::now(),
            success: true,
        })
    }

    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>> {
        let client = self.client()?.clone();
        let (tx, rx) = mpsc::unbounded_channel();
        let (_updates_tx, updates_rx) = mpsc::unbounded_channel();

        // Start the update stream
        let mut stream = client
            .stream_updates(updates_rx, UpdatesConfiguration::default())
            .await;

        tokio::spawn(async move {
            loop {
                match stream.next().await {
                    Ok(update) => {
                        if let Update::NewMessage(msg) = update {
                            if msg.outgoing() {
                                continue;
                            }
                            let sender = msg
                                .sender()
                                .and_then(|s| s.name().map(|n| n.to_string()))
                                .unwrap_or_else(|| "Unknown".to_string());

                            let conv_id = msg.peer_id().to_string();

                            let notif = Notification {
                                platform: "Telegram".to_string(),
                                conversation_id: conv_id,
                                sender: sender.clone(),
                                preview: msg.text().chars().take(100).collect(),
                                timestamp: msg.date(),
                                is_read: false,
                            };

                            if tx.send(notif).is_err() {
                                break;
                            }
                        }
                    }
                    Err(e) => {
                        tracing::error!("Telegram update error: {}", e);
                        tokio::time::sleep(std::time::Duration::from_secs(5)).await;
                    }
                }
            }
        });

        Ok(rx)
    }
}
