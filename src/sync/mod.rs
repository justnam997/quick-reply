pub mod progress;

use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::Mutex;

use crate::error::Result;
use crate::memory::archive::ConversationArchive;
use crate::platform::manager::PlatformManager;
use progress::{PlatformSyncProgress, SyncPhase};

#[derive(Debug, Clone, PartialEq)]
pub enum SyncDepth {
    Full,
    Recent(usize),
    Minimal,
    Off,
}

impl SyncDepth {
    pub fn from_config(depth: &str, recent_count: usize) -> Self {
        match depth {
            "full" => SyncDepth::Full,
            "recent" => SyncDepth::Recent(recent_count),
            "minimal" => SyncDepth::Minimal,
            "off" => SyncDepth::Off,
            _ => SyncDepth::Recent(recent_count),
        }
    }

    pub fn as_str(&self) -> &str {
        match self {
            SyncDepth::Full => "full",
            SyncDepth::Recent(_) => "recent",
            SyncDepth::Minimal => "minimal",
            SyncDepth::Off => "off",
        }
    }

    pub fn next(&self) -> SyncDepth {
        match self {
            SyncDepth::Full => SyncDepth::Recent(50),
            SyncDepth::Recent(_) => SyncDepth::Minimal,
            SyncDepth::Minimal => SyncDepth::Off,
            SyncDepth::Off => SyncDepth::Full,
        }
    }
}

pub struct SyncEngine {
    archive: ConversationArchive,
    progress: Arc<Mutex<HashMap<String, PlatformSyncProgress>>>,
}

impl SyncEngine {
    pub fn new(archive: ConversationArchive) -> Self {
        Self {
            archive,
            progress: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub async fn initial_sync(
        &self,
        manager: &PlatformManager,
        platform: &str,
        depth: SyncDepth,
    ) -> Result<()> {
        if depth == SyncDepth::Off {
            return Ok(());
        }

        // Initialize progress
        {
            let mut progress = self.progress.lock().await;
            progress.insert(platform.to_string(), PlatformSyncProgress::new());
        }

        // Phase 1: Sync conversations
        self.update_phase(platform, SyncPhase::SyncingConversations)
            .await;
        let conversations = manager.fetch_conversations(platform).await?;
        let conv_count = conversations.len();

        {
            let mut progress = self.progress.lock().await;
            if let Some(p) = progress.get_mut(platform) {
                p.conversations_total = conv_count;
            }
        }

        for conv in &conversations {
            self.archive.archive_conversation(conv).await.ok();
            let mut progress = self.progress.lock().await;
            if let Some(p) = progress.get_mut(platform) {
                p.conversations_synced += 1;
            }
        }

        // Phase 2: Sync messages (based on depth)
        let msg_limit = match &depth {
            SyncDepth::Full => usize::MAX,
            SyncDepth::Recent(n) => *n,
            SyncDepth::Minimal => 0,
            SyncDepth::Off => 0,
        };

        if msg_limit > 0 {
            self.update_phase(platform, SyncPhase::SyncingMessages)
                .await;

            for conv in &conversations {
                match manager.fetch_messages(platform, &conv.id, msg_limit).await {
                    Ok(messages) => {
                        {
                            let mut progress = self.progress.lock().await;
                            if let Some(p) = progress.get_mut(platform) {
                                p.messages_total += messages.len();
                            }
                        }
                        self.archive.archive_messages(&messages).await.ok();
                        {
                            let mut progress = self.progress.lock().await;
                            if let Some(p) = progress.get_mut(platform) {
                                p.messages_synced += messages.len();
                            }
                        }
                    }
                    Err(e) => {
                        tracing::warn!(
                            "Failed to sync messages for {}/{}: {e}",
                            platform,
                            conv.id
                        );
                    }
                }
            }
        }

        // Mark complete
        {
            let mut progress = self.progress.lock().await;
            if let Some(p) = progress.get_mut(platform) {
                p.phase = SyncPhase::Complete;
                p.last_sync = Some(chrono::Utc::now());
            }
        }

        Ok(())
    }

    async fn update_phase(&self, platform: &str, phase: SyncPhase) {
        let mut progress = self.progress.lock().await;
        if let Some(p) = progress.get_mut(platform) {
            p.phase = phase;
        }
    }

    pub async fn progress(&self) -> HashMap<String, PlatformSyncProgress> {
        self.progress.lock().await.clone()
    }

    pub async fn set_error(&self, platform: &str, error: String) {
        let mut progress = self.progress.lock().await;
        if let Some(p) = progress.get_mut(platform) {
            p.phase = SyncPhase::Error;
            p.error = Some(error);
        }
    }
}
