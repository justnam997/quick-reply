pub mod imessage;
pub mod manager;
pub mod messenger;
pub mod slack;
pub mod telegram;
pub mod whatsapp;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::error::Result;
use crate::models::{Conversation, Message, Notification};

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DeliveryConfirmation {
    pub message_id: String,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub success: bool,
}

#[async_trait]
pub trait PlatformAdapter: Send + Sync {
    fn name(&self) -> &str;

    async fn connect(&mut self) -> Result<()>;

    #[allow(dead_code)]
    async fn disconnect(&mut self) -> Result<()>;

    fn is_connected(&self) -> bool;

    async fn fetch_conversations(&self) -> Result<Vec<Conversation>>;

    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>>;

    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation>;

    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>>;
}
