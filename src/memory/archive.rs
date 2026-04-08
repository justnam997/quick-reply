use tokio_rusqlite::Connection;

use crate::error::{QuickReplyError, Result};
use crate::models::{Conversation, Message};

pub struct ConversationArchive {
    conn: Connection,
}

impl ConversationArchive {
    pub fn new(conn: Connection) -> Self {
        Self { conn }
    }

    pub async fn archive_messages(&self, messages: &[Message]) -> Result<()> {
        let messages = messages.to_vec();
        self.conn
            .call(move |conn| {
                let tx = conn.transaction()?;
                {
                    let mut stmt = tx.prepare_cached(
                        "INSERT OR REPLACE INTO messages
                         (id, conversation_id, platform, sender, content, timestamp, is_read, is_outgoing)
                         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
                    )?;
                    for msg in &messages {
                        stmt.execute(rusqlite::params![
                            msg.id,
                            msg.conversation_id,
                            msg.platform,
                            msg.sender,
                            msg.content,
                            msg.timestamp.timestamp(),
                            msg.is_read,
                            msg.is_outgoing,
                        ])?;
                    }
                }
                tx.commit()?;
                Ok(())
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Archive failed: {e}")))
    }

    pub async fn archive_conversation(&self, conv: &Conversation) -> Result<()> {
        let conv = conv.clone();
        self.conn
            .call(move |conn| {
                conn.execute(
                    "INSERT OR REPLACE INTO conversations
                     (id, platform, participants, last_active, last_message_preview)
                     VALUES (?1, ?2, ?3, ?4, ?5)",
                    rusqlite::params![
                        conv.id,
                        conv.platform,
                        serde_json::to_string(&conv.participants).unwrap_or_default(),
                        conv.last_active.timestamp(),
                        conv.last_message_preview,
                    ],
                )?;
                Ok(())
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Archive conversation failed: {e}")))
    }

    #[allow(dead_code)]
    pub async fn get_messages(
        &self,
        conv_id: &str,
        limit: usize,
        offset: usize,
    ) -> Result<Vec<Message>> {
        let conv_id = conv_id.to_string();
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT id, conversation_id, platform, sender, content, timestamp, is_read, is_outgoing
                     FROM messages
                     WHERE conversation_id = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2 OFFSET ?3",
                )?;

                let messages = stmt
                    .query_map(rusqlite::params![conv_id, limit, offset], |row| {
                        let timestamp_secs: i64 = row.get(5)?;
                        Ok(Message {
                            id: row.get(0)?,
                            conversation_id: row.get(1)?,
                            platform: row.get(2)?,
                            sender: row.get(3)?,
                            content: row.get(4)?,
                            timestamp: chrono::DateTime::from_timestamp(timestamp_secs, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            is_read: row.get(6)?,
                            is_outgoing: row.get(7)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                Ok(messages)
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Get messages failed: {e}")))
    }

    #[allow(dead_code)]
    pub async fn get_conversations(&self, platform: Option<&str>) -> Result<Vec<Conversation>> {
        let platform = platform.map(|s| s.to_string());
        self.conn
            .call(move |conn| {
                let (sql, params): (&str, Vec<Box<dyn rusqlite::types::ToSql>>) = match &platform {
                    Some(p) => (
                        "SELECT id, platform, participants, last_active, last_message_preview
                         FROM conversations WHERE platform = ?1
                         ORDER BY last_active DESC",
                        vec![Box::new(p.clone())],
                    ),
                    None => (
                        "SELECT id, platform, participants, last_active, last_message_preview
                         FROM conversations
                         ORDER BY last_active DESC",
                        vec![],
                    ),
                };

                let mut stmt = conn.prepare(sql)?;
                let param_refs: Vec<&dyn rusqlite::types::ToSql> =
                    params.iter().map(|p| p.as_ref()).collect();
                let convs = stmt
                    .query_map(param_refs.as_slice(), |row| {
                        let timestamp_secs: i64 = row.get(3)?;
                        let participants_json: String = row.get(2)?;
                        let participants: Vec<String> =
                            serde_json::from_str(&participants_json).unwrap_or_default();
                        Ok(Conversation {
                            id: row.get(0)?,
                            platform: row.get(1)?,
                            participants,
                            last_active: chrono::DateTime::from_timestamp(timestamp_secs, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            last_message_preview: row.get(4)?,
                            unread_count: 0,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                Ok(convs)
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Get conversations failed: {e}")))
    }

    pub async fn search_messages(&self, query: &str, limit: usize) -> Result<Vec<Message>> {
        let query = query.to_string();
        self.conn
            .call(move |conn| {
                let mut stmt = conn.prepare(
                    "SELECT m.id, m.conversation_id, m.platform, m.sender, m.content, m.timestamp, m.is_read, m.is_outgoing
                     FROM messages m
                     JOIN messages_fts fts ON m.rowid = fts.rowid
                     WHERE messages_fts MATCH ?1
                     ORDER BY m.timestamp DESC
                     LIMIT ?2",
                )?;

                let messages = stmt
                    .query_map(rusqlite::params![query, limit], |row| {
                        let timestamp_secs: i64 = row.get(5)?;
                        Ok(Message {
                            id: row.get(0)?,
                            conversation_id: row.get(1)?,
                            platform: row.get(2)?,
                            sender: row.get(3)?,
                            content: row.get(4)?,
                            timestamp: chrono::DateTime::from_timestamp(timestamp_secs, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            is_read: row.get(6)?,
                            is_outgoing: row.get(7)?,
                        })
                    })?
                    .collect::<std::result::Result<Vec<_>, _>>()?;

                Ok(messages)
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Search failed: {e}")))
    }
}
