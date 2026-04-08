use std::collections::HashMap;

use tokio::sync::mpsc;

use super::{DeliveryConfirmation, PlatformAdapter};
use crate::app::ConnectionStatus;
use crate::error::Result;
use crate::models::{Conversation, Message, Notification};

pub struct PlatformManager {
    adapters: HashMap<String, Box<dyn PlatformAdapter>>,
    notification_tx: mpsc::UnboundedSender<Notification>,
    notification_rx: Option<mpsc::UnboundedReceiver<Notification>>,
}

impl PlatformManager {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            adapters: HashMap::new(),
            notification_tx: tx,
            notification_rx: Some(rx),
        }
    }

    pub fn register(&mut self, adapter: Box<dyn PlatformAdapter>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }

    pub fn take_notification_receiver(&mut self) -> Option<mpsc::UnboundedReceiver<Notification>> {
        self.notification_rx.take()
    }

    pub async fn connect_all(&mut self) {
        for (name, adapter) in self.adapters.iter_mut() {
            if let Err(e) = adapter.connect().await {
                tracing::error!("Failed to connect {}: {}", name, e);
            }
        }
    }

    pub fn status(&self, platform: &str) -> ConnectionStatus {
        self.adapters
            .get(platform)
            .map(|a| {
                if a.is_connected() {
                    ConnectionStatus::Connected
                } else {
                    ConnectionStatus::Disconnected
                }
            })
            .unwrap_or(ConnectionStatus::Disconnected)
    }

    pub async fn fetch_conversations(&self, platform: &str) -> Result<Vec<Conversation>> {
        if let Some(adapter) = self.adapters.get(platform) {
            adapter.fetch_conversations().await
        } else {
            Ok(vec![])
        }
    }

    pub async fn fetch_messages(
        &self,
        platform: &str,
        conv_id: &str,
        limit: usize,
    ) -> Result<Vec<Message>> {
        if let Some(adapter) = self.adapters.get(platform) {
            adapter.fetch_messages(conv_id, limit).await
        } else {
            Ok(vec![])
        }
    }

    pub async fn send_message(
        &self,
        platform: &str,
        conv_id: &str,
        content: &str,
    ) -> Result<DeliveryConfirmation> {
        let adapter =
            self.adapters
                .get(platform)
                .ok_or_else(|| crate::error::QuickReplyError::Platform {
                    platform: platform.to_string(),
                    message: "Platform not registered".to_string(),
                })?;
        adapter.send_message(conv_id, content).await
    }

    pub async fn start_notification_listeners(&self) {
        for (name, adapter) in &self.adapters {
            if !adapter.is_connected() {
                continue;
            }
            let tx = self.notification_tx.clone();
            match adapter.subscribe_notifications().await {
                Ok(mut rx) => {
                    let platform_name = name.clone();
                    tokio::spawn(async move {
                        while let Some(notif) = rx.recv().await {
                            if tx.send(notif).is_err() {
                                tracing::warn!("{} notification channel closed", platform_name);
                                break;
                            }
                        }
                    });
                }
                Err(e) => {
                    tracing::error!("Failed to subscribe {} notifications: {}", name, e);
                }
            }
        }
    }

    #[allow(dead_code)]
    pub fn platform_names(&self) -> Vec<String> {
        let order = ["Telegram", "iMessage", "WhatsApp", "Slack", "Messenger"];
        order
            .iter()
            .filter(|name| self.adapters.contains_key(**name))
            .map(|s| s.to_string())
            .collect()
    }
}
