use std::path::Path;

use tokio_rusqlite::Connection;

use crate::error::{QuickReplyError, Result};

pub struct Database {
    conn: Connection,
}

impl Database {
    pub async fn open(path: &Path) -> Result<Self> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(path)
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Failed to open database: {e}")))?;

        // Enable WAL mode and set pragmas
        conn.call(|conn| {
            conn.execute_batch(
                "PRAGMA journal_mode=WAL;
                 PRAGMA busy_timeout=5000;
                 PRAGMA foreign_keys=ON;
                 PRAGMA synchronous=NORMAL;",
            )?;
            Ok(())
        })
        .await
        .map_err(|e| QuickReplyError::Storage(format!("Failed to set pragmas: {e}")))?;

        let db = Self { conn };
        db.run_migrations().await?;
        Ok(db)
    }

    async fn run_migrations(&self) -> Result<()> {
        self.conn
            .call(|conn| {
                conn.execute_batch(
                    "CREATE TABLE IF NOT EXISTS schema_version (
                        version INTEGER PRIMARY KEY
                    );

                    INSERT OR IGNORE INTO schema_version (version) VALUES (0);",
                )?;

                let version: i32 =
                    conn.query_row("SELECT MAX(version) FROM schema_version", [], |row| {
                        row.get(0)
                    })?;

                if version < 1 {
                    conn.execute_batch(
                        "CREATE TABLE IF NOT EXISTS messages (
                            id TEXT PRIMARY KEY,
                            conversation_id TEXT NOT NULL,
                            platform TEXT NOT NULL,
                            sender TEXT NOT NULL,
                            content TEXT NOT NULL,
                            timestamp INTEGER NOT NULL,
                            is_read BOOLEAN DEFAULT FALSE,
                            is_outgoing BOOLEAN DEFAULT FALSE
                        );
                        CREATE INDEX IF NOT EXISTS idx_messages_conv
                            ON messages(conversation_id, timestamp DESC);
                        CREATE INDEX IF NOT EXISTS idx_messages_platform
                            ON messages(platform);

                        CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts
                            USING fts5(content, content=messages, content_rowid=rowid);

                        CREATE TRIGGER IF NOT EXISTS messages_ai AFTER INSERT ON messages BEGIN
                            INSERT INTO messages_fts(rowid, content)
                            VALUES (new.rowid, new.content);
                        END;
                        CREATE TRIGGER IF NOT EXISTS messages_ad AFTER DELETE ON messages BEGIN
                            INSERT INTO messages_fts(messages_fts, rowid, content)
                            VALUES('delete', old.rowid, old.content);
                        END;

                        CREATE TABLE IF NOT EXISTS conversations (
                            id TEXT PRIMARY KEY,
                            platform TEXT NOT NULL,
                            participants TEXT NOT NULL,
                            last_active INTEGER NOT NULL,
                            last_message_preview TEXT
                        );

                        CREATE TABLE IF NOT EXISTS contacts (
                            id TEXT PRIMARY KEY,
                            name TEXT NOT NULL,
                            platform TEXT NOT NULL,
                            profile_path TEXT
                        );

                        CREATE TABLE IF NOT EXISTS embeddings (
                            id INTEGER PRIMARY KEY AUTOINCREMENT,
                            message_id TEXT NOT NULL REFERENCES messages(id),
                            vector BLOB NOT NULL,
                            model TEXT NOT NULL DEFAULT 'bge-small-en-v1.5',
                            created_at INTEGER NOT NULL
                        );
                        CREATE INDEX IF NOT EXISTS idx_embeddings_message
                            ON embeddings(message_id);

                        INSERT OR REPLACE INTO schema_version (version) VALUES (1);",
                    )?;
                }

                Ok(())
            })
            .await
            .map_err(|e| QuickReplyError::Storage(format!("Migration failed: {e}")))?;

        Ok(())
    }

    pub fn connection(&self) -> &Connection {
        &self.conn
    }
}
