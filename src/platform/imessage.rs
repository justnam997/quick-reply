use std::path::PathBuf;

use async_trait::async_trait;
use tokio::sync::mpsc;

use crate::config::IMessageConfig;
use crate::error::{QuickReplyError, Result};
use crate::models::{Conversation, Message, Notification};
use crate::platform::{DeliveryConfirmation, PlatformAdapter};

pub struct IMessageAdapter {
    db_path: PathBuf,
    connected: bool,
    last_seen_rowid: i64,
}

impl IMessageAdapter {
    pub fn new(config: IMessageConfig) -> Self {
        Self {
            db_path: config.db_path,
            connected: false,
            last_seen_rowid: 0,
        }
    }

    fn check_tcc_permission(&self) -> Result<()> {
        if !self.db_path.exists() {
            return Err(QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!(
                    "Database not found at {}. \
                     Ensure Full Disk Access is granted: \
                     System Settings > Privacy & Security > Full Disk Access > add your terminal app.",
                    self.db_path.display()
                ),
            });
        }

        // Try opening the file to check read permission
        std::fs::File::open(&self.db_path).map_err(|_| QuickReplyError::Platform {
            platform: "iMessage".to_string(),
            message: "Cannot read chat.db — Full Disk Access permission required. \
                     System Settings > Privacy & Security > Full Disk Access > add your terminal app."
                .to_string(),
        })?;

        Ok(())
    }
}

#[async_trait]
impl PlatformAdapter for IMessageAdapter {
    fn name(&self) -> &str {
        "iMessage"
    }

    async fn connect(&mut self) -> Result<()> {
        self.check_tcc_permission()?;

        // Verify we can open the SQLite database
        let db_path = self.db_path.clone();
        let conn = tokio_rusqlite::Connection::open(&db_path)
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("Failed to open chat.db: {e}"),
            })?;

        // Get the latest rowid for notification tracking
        let max_rowid = conn
            .call(|conn| {
                let rowid: i64 =
                    conn.query_row("SELECT COALESCE(MAX(ROWID), 0) FROM message", [], |row| {
                        row.get(0)
                    })?;
                Ok(rowid)
            })
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("Failed to query chat.db: {e}"),
            })?;

        self.last_seen_rowid = max_rowid;
        self.connected = true;
        tracing::info!("iMessage connected (chat.db accessible)");
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
        let db_path = self.db_path.clone();
        let conn = tokio_rusqlite::Connection::open(&db_path)
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("DB open error: {e}"),
            })?;

        conn.call(|conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    c.ROWID,
                    c.chat_identifier,
                    c.display_name,
                    m.text,
                    m.date / 1000000000 + 978307200 as timestamp
                 FROM chat c
                 LEFT JOIN chat_message_join cmj ON cmj.chat_id = c.ROWID
                 LEFT JOIN message m ON m.ROWID = cmj.message_id
                 WHERE m.ROWID = (
                     SELECT MAX(cmj2.message_id)
                     FROM chat_message_join cmj2
                     WHERE cmj2.chat_id = c.ROWID
                 )
                 ORDER BY timestamp DESC
                 LIMIT 50",
            )?;

            let convs = stmt
                .query_map([], |row| {
                    let rowid: i64 = row.get(0)?;
                    let chat_id: String = row.get(1)?;
                    let display_name: Option<String> = row.get(2)?;
                    let last_text: Option<String> = row.get(3)?;
                    let timestamp: Option<i64> = row.get(4)?;

                    let name = display_name
                        .filter(|n| !n.is_empty())
                        .unwrap_or_else(|| chat_id.clone());

                    Ok(Conversation {
                        id: rowid.to_string(),
                        platform: "iMessage".to_string(),
                        participants: vec![name],
                        last_active: timestamp
                            .and_then(|t| chrono::DateTime::from_timestamp(t, 0))
                            .unwrap_or_else(chrono::Utc::now),
                        last_message_preview: last_text.map(|t| {
                            if t.len() > 50 {
                                format!("{}...", &t[..47])
                            } else {
                                t
                            }
                        }),
                        unread_count: 0,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            Ok(convs)
        })
        .await
        .map_err(|e| QuickReplyError::Platform {
            platform: "iMessage".to_string(),
            message: format!("Fetch conversations failed: {e}"),
        })
    }

    async fn fetch_messages(&self, conv_id: &str, limit: usize) -> Result<Vec<Message>> {
        let db_path = self.db_path.clone();
        let conv_id = conv_id.to_string();
        let conn = tokio_rusqlite::Connection::open(&db_path)
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("DB open error: {e}"),
            })?;

        conn.call(move |conn| {
            let mut stmt = conn.prepare(
                "SELECT
                    m.ROWID,
                    m.text,
                    m.is_from_me,
                    m.date / 1000000000 + 978307200 as timestamp,
                    COALESCE(h.id, 'Unknown') as sender
                 FROM message m
                 JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
                 LEFT JOIN handle h ON m.handle_id = h.ROWID
                 WHERE cmj.chat_id = ?1
                 ORDER BY m.date DESC
                 LIMIT ?2",
            )?;

            let chat_rowid: i64 = conv_id.parse().map_err(|_| {
                rusqlite::Error::InvalidParameterName(format!("Invalid chat ID: {conv_id}"))
            })?;
            let mut messages = stmt
                .query_map(rusqlite::params![chat_rowid, limit], |row| {
                    let rowid: i64 = row.get(0)?;
                    let text: Option<String> = row.get(1)?;
                    let is_from_me: bool = row.get(2)?;
                    let timestamp: i64 = row.get(3)?;
                    let sender: String = row.get(4)?;

                    Ok(Message {
                        id: format!("imsg_{rowid}"),
                        conversation_id: conv_id.clone(),
                        platform: "iMessage".to_string(),
                        sender: if is_from_me {
                            "You".to_string()
                        } else {
                            sender
                        },
                        content: text.unwrap_or_default(),
                        timestamp: chrono::DateTime::from_timestamp(timestamp, 0)
                            .unwrap_or_else(chrono::Utc::now),
                        is_read: true,
                        is_outgoing: is_from_me,
                    })
                })?
                .collect::<std::result::Result<Vec<_>, _>>()?;

            messages.reverse();
            Ok(messages)
        })
        .await
        .map_err(|e| QuickReplyError::Platform {
            platform: "iMessage".to_string(),
            message: format!("Fetch messages failed: {e}"),
        })
    }

    async fn send_message(&self, conv_id: &str, content: &str) -> Result<DeliveryConfirmation> {
        // Resolve chat identifier from conv_id (ROWID)
        let db_path = self.db_path.clone();
        let conv_id = conv_id.to_string();
        let conn = tokio_rusqlite::Connection::open(&db_path)
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("DB open error: {e}"),
            })?;

        let chat_identifier: String = conn
            .call(move |conn| {
                conn.query_row(
                    "SELECT chat_identifier FROM chat WHERE ROWID = ?1",
                    [&conv_id],
                    |row| row.get(0),
                )
                .map_err(|e| e.into())
            })
            .await
            .map_err(|e| QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("Chat lookup failed: {e}"),
            })?;

        // Send via osascript — sanitize both content and chat_identifier
        // to prevent AppleScript injection
        let escaped_content = escape_applescript(content);
        let escaped_chat_id = escape_applescript(&chat_identifier);
        let script = format!(
            r#"tell application "Messages" to send "{escaped_content}" to buddy "{escaped_chat_id}" of service "iMessage""#
        );

        let output = tokio::process::Command::new("osascript")
            .arg("-e")
            .arg(&script)
            .output()
            .await?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(QuickReplyError::Platform {
                platform: "iMessage".to_string(),
                message: format!("osascript failed: {stderr}"),
            });
        }

        Ok(DeliveryConfirmation {
            message_id: uuid::Uuid::new_v4().to_string(),
            timestamp: chrono::Utc::now(),
            success: true,
        })
    }

    async fn subscribe_notifications(&self) -> Result<mpsc::UnboundedReceiver<Notification>> {
        let (tx, rx) = mpsc::unbounded_channel();
        let db_path = self.db_path.clone();
        let mut last_rowid = self.last_seen_rowid;

        // Poll chat.db for new messages every 3 seconds
        tokio::spawn(async move {
            loop {
                tokio::time::sleep(std::time::Duration::from_secs(3)).await;

                let Ok(conn) = tokio_rusqlite::Connection::open(&db_path).await else {
                    continue;
                };

                let current_last = last_rowid;
                let result = conn
                    .call(move |conn| {
                        let mut stmt = conn.prepare(
                            "SELECT m.ROWID, m.text, m.is_from_me,
                                    m.date / 1000000000 + 978307200,
                                    COALESCE(h.id, 'Unknown'),
                                    cmj.chat_id
                             FROM message m
                             JOIN chat_message_join cmj ON cmj.message_id = m.ROWID
                             LEFT JOIN handle h ON m.handle_id = h.ROWID
                             WHERE m.ROWID > ?1 AND m.is_from_me = 0
                             ORDER BY m.ROWID ASC",
                        )?;

                        let results = stmt
                            .query_map([current_last], |row| {
                                Ok((
                                    row.get::<_, i64>(0)?,
                                    row.get::<_, Option<String>>(1)?,
                                    row.get::<_, i64>(3)?,
                                    row.get::<_, String>(4)?,
                                    row.get::<_, i64>(5)?,
                                ))
                            })?
                            .collect::<std::result::Result<Vec<_>, _>>()?;

                        Ok(results)
                    })
                    .await;

                if let Ok(new_messages) = result {
                    for (rowid, text, timestamp, sender, chat_id) in new_messages {
                        last_rowid = last_rowid.max(rowid);
                        let notif = Notification {
                            platform: "iMessage".to_string(),
                            conversation_id: chat_id.to_string(),
                            sender: sender.clone(),
                            preview: text.unwrap_or_default().chars().take(100).collect(),
                            timestamp: chrono::DateTime::from_timestamp(timestamp, 0)
                                .unwrap_or_else(chrono::Utc::now),
                            is_read: false,
                        };
                        if tx.send(notif).is_err() {
                            return;
                        }
                    }
                }
            }
        });

        Ok(rx)
    }
}

/// Escape a string for safe embedding in AppleScript double-quoted strings.
/// Prevents injection by escaping backslashes, double quotes, and
/// stripping control characters that could break out of the string context.
fn escape_applescript(s: &str) -> String {
    s.chars()
        .filter(|c| !c.is_control() || *c == '\n')
        .collect::<String>()
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
