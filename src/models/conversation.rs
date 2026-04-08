use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Conversation {
    pub id: String,
    pub platform: String,
    pub participants: Vec<String>,
    pub last_active: DateTime<Utc>,
    pub last_message_preview: Option<String>,
    pub unread_count: u32,
}

impl Conversation {
    pub fn display_name(&self) -> &str {
        self.participants
            .first()
            .map(|s| s.as_str())
            .unwrap_or("Unknown")
    }
}
