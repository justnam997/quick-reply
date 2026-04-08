use chrono::{DateTime, Utc};

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)]
pub enum SyncPhase {
    Idle,
    SyncingConversations,
    SyncingMessages,
    BuildingEmbeddings,
    UpdatingProfiles,
    Complete,
    Error,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct PlatformSyncProgress {
    pub phase: SyncPhase,
    pub conversations_total: usize,
    pub conversations_synced: usize,
    pub messages_total: usize,
    pub messages_synced: usize,
    pub embeddings_total: usize,
    pub embeddings_processed: usize,
    pub profiles_total: usize,
    pub profiles_updated: usize,
    pub last_sync: Option<DateTime<Utc>>,
    pub error: Option<String>,
}

impl PlatformSyncProgress {
    pub fn new() -> Self {
        Self {
            phase: SyncPhase::Idle,
            conversations_total: 0,
            conversations_synced: 0,
            messages_total: 0,
            messages_synced: 0,
            embeddings_total: 0,
            embeddings_processed: 0,
            profiles_total: 0,
            profiles_updated: 0,
            last_sync: None,
            error: None,
        }
    }

    #[allow(dead_code)]
    pub fn status_line(&self) -> String {
        match &self.phase {
            SyncPhase::Idle => "Idle".to_string(),
            SyncPhase::SyncingConversations => {
                format!(
                    "Syncing conversations ({}/{})",
                    self.conversations_synced, self.conversations_total
                )
            }
            SyncPhase::SyncingMessages => {
                format!(
                    "Syncing messages ({}/{})",
                    self.messages_synced, self.messages_total
                )
            }
            SyncPhase::BuildingEmbeddings => {
                format!(
                    "Building embeddings ({}/{})",
                    self.embeddings_processed, self.embeddings_total
                )
            }
            SyncPhase::UpdatingProfiles => {
                format!(
                    "Updating profiles ({}/{})",
                    self.profiles_updated, self.profiles_total
                )
            }
            SyncPhase::Complete => {
                if let Some(last) = &self.last_sync {
                    let ago = Utc::now().signed_duration_since(*last);
                    if ago.num_seconds() < 60 {
                        format!("Synced {}s ago", ago.num_seconds())
                    } else if ago.num_minutes() < 60 {
                        format!("Synced {}m ago", ago.num_minutes())
                    } else {
                        format!("Synced {}h ago", ago.num_hours())
                    }
                } else {
                    "Complete".to_string()
                }
            }
            SyncPhase::Error => {
                format!("Error: {}", self.error.as_deref().unwrap_or("unknown"))
            }
        }
    }

    #[allow(dead_code)]
    pub fn memory_status_line(&self) -> String {
        format!(
            "{}/{} embedded | {} profiles",
            self.embeddings_processed, self.embeddings_total, self.profiles_updated
        )
    }
}
