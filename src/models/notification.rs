use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Notification {
    pub platform: String,
    pub conversation_id: String,
    pub sender: String,
    pub preview: String,
    pub timestamp: DateTime<Utc>,
    pub is_read: bool,
}
