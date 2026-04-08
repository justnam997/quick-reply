use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub id: String,
    pub conversation_id: String,
    pub platform: String,
    pub sender: String,
    pub content: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
    pub is_outgoing: bool,
}

impl Message {
    #[allow(dead_code)]
    pub fn preview(&self, max_len: usize) -> String {
        if self.content.len() <= max_len {
            self.content.clone()
        } else {
            format!("{}...", &self.content[..max_len.saturating_sub(3)])
        }
    }
}
