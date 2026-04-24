//! SQLite-backed memory store with FTS5 full-text search.

use crate::config::data_dir;
use crate::error::{FerroError, Result};
use rusqlite::{Connection, params};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Memory {
    pub id: i64,
    pub key: String,
    pub content: String,
    pub created_at: String,
    pub updated_at: String,
    pub relevance: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversationEntry {
    pub id: i64,
    pub session_id: String,
    pub role: String,
    pub content: String,
    pub timestamp: String,
}

pub struct MemoryStore {
    conn: Connection,
}

impl MemoryStore {
    pub fn new(db_path: Option<PathBuf>) -> Result<Self> {
        let path = db_path.unwrap_or_else(|| data_dir().join("memory.db"));

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(&path)
            .map_err(|e| FerroError::Memory(format!("Failed to open database: {e}")))?;

        let store = Self { conn };
        store.initialize_tables()?;
        Ok(store)
    }

    /// Create an in-memory store for testing.
    pub fn in_memory() -> Result<Self> {
        let conn = Connection::open_in_memory()
            .map_err(|e| FerroError::Memory(format!("Failed to create in-memory db: {e}")))?;
        let store = Self { conn };
        store.initialize_tables()?;
        Ok(store)
    }

    fn initialize_tables(&self) -> Result<()> {
        self.conn
            .execute_batch(
                "
            CREATE TABLE IF NOT EXISTS memories (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                key TEXT NOT NULL UNIQUE,
                content TEXT NOT NULL,
                created_at TEXT NOT NULL DEFAULT (datetime('now')),
                updated_at TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS memories_fts USING fts5(
                key, content, content=memories, content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS memories_ai AFTER INSERT ON memories BEGIN
                INSERT INTO memories_fts(rowid, key, content) VALUES (new.id, new.key, new.content);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_ad AFTER DELETE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, key, content) VALUES ('delete', old.id, old.key, old.content);
            END;

            CREATE TRIGGER IF NOT EXISTS memories_au AFTER UPDATE ON memories BEGIN
                INSERT INTO memories_fts(memories_fts, rowid, key, content) VALUES ('delete', old.id, old.key, old.content);
                INSERT INTO memories_fts(rowid, key, content) VALUES (new.id, new.key, new.content);
            END;

            CREATE TABLE IF NOT EXISTS conversations (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                session_id TEXT NOT NULL,
                role TEXT NOT NULL,
                content TEXT NOT NULL,
                timestamp TEXT NOT NULL DEFAULT (datetime('now'))
            );

            CREATE INDEX IF NOT EXISTS idx_conversations_session ON conversations(session_id);
            ",
            )
            .map_err(|e| FerroError::Memory(format!("Failed to initialize tables: {e}")))?;
        Ok(())
    }

    /// Store or update a memory entry.
    pub fn insert(&self, key: &str, content: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO memories (key, content) VALUES (?1, ?2)
                 ON CONFLICT(key) DO UPDATE SET content = ?2, updated_at = datetime('now')",
                params![key, content],
            )
            .map_err(|e| FerroError::Memory(format!("Insert failed: {e}")))?;
        Ok(())
    }

    /// Search memories using FTS5 full-text search.
    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Memory>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT m.id, m.key, m.content, m.created_at, m.updated_at,
                        rank * -1.0 as relevance
                 FROM memories_fts f
                 JOIN memories m ON f.rowid = m.id
                 WHERE memories_fts MATCH ?1
                 ORDER BY rank
                 LIMIT ?2",
            )
            .map_err(|e| FerroError::Memory(format!("Search prepare failed: {e}")))?;

        let memories = stmt
            .query_map(params![query, limit as i64], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    relevance: row.get(5)?,
                })
            })
            .map_err(|e| FerroError::Memory(format!("Search failed: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(memories)
    }

    /// Get a memory by key.
    pub fn get(&self, key: &str) -> Result<Option<Memory>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, key, content, created_at, updated_at, 1.0 as relevance
                 FROM memories WHERE key = ?1",
            )
            .map_err(|e| FerroError::Memory(format!("Get prepare failed: {e}")))?;

        let result = stmt
            .query_row(params![key], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    relevance: row.get(5)?,
                })
            })
            .ok();

        Ok(result)
    }

    /// Delete a memory by key.
    pub fn forget(&self, key: &str) -> Result<bool> {
        let rows = self
            .conn
            .execute("DELETE FROM memories WHERE key = ?1", params![key])
            .map_err(|e| FerroError::Memory(format!("Delete failed: {e}")))?;
        Ok(rows > 0)
    }

    /// List all memories.
    pub fn list_all(&self) -> Result<Vec<Memory>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, key, content, created_at, updated_at, 1.0 as relevance
                 FROM memories ORDER BY updated_at DESC",
            )
            .map_err(|e| FerroError::Memory(format!("List failed: {e}")))?;

        let memories = stmt
            .query_map([], |row| {
                Ok(Memory {
                    id: row.get(0)?,
                    key: row.get(1)?,
                    content: row.get(2)?,
                    created_at: row.get(3)?,
                    updated_at: row.get(4)?,
                    relevance: row.get(5)?,
                })
            })
            .map_err(|e| FerroError::Memory(format!("List query failed: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(memories)
    }

    /// Save a conversation message.
    pub fn save_conversation(&self, session_id: &str, role: &str, content: &str) -> Result<()> {
        self.conn
            .execute(
                "INSERT INTO conversations (session_id, role, content) VALUES (?1, ?2, ?3)",
                params![session_id, role, content],
            )
            .map_err(|e| FerroError::Memory(format!("Save conversation failed: {e}")))?;
        Ok(())
    }

    /// Get conversation history for a session.
    pub fn get_conversation(&self, session_id: &str) -> Result<Vec<ConversationEntry>> {
        let mut stmt = self
            .conn
            .prepare(
                "SELECT id, session_id, role, content, timestamp
                 FROM conversations
                 WHERE session_id = ?1
                 ORDER BY id ASC",
            )
            .map_err(|e| FerroError::Memory(format!("Get conversation failed: {e}")))?;

        let entries = stmt
            .query_map(params![session_id], |row| {
                Ok(ConversationEntry {
                    id: row.get(0)?,
                    session_id: row.get(1)?,
                    role: row.get(2)?,
                    content: row.get(3)?,
                    timestamp: row.get(4)?,
                })
            })
            .map_err(|e| FerroError::Memory(format!("Conversation query failed: {e}")))?
            .filter_map(|r| r.ok())
            .collect();

        Ok(entries)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_memory_crud() {
        let store = MemoryStore::in_memory().unwrap();

        // Insert
        store.insert("user_name", "Alice").unwrap();
        store.insert("project", "Ferroclaw").unwrap();

        // Get
        let mem = store.get("user_name").unwrap().unwrap();
        assert_eq!(mem.content, "Alice");

        // Update
        store.insert("user_name", "Bob").unwrap();
        let mem = store.get("user_name").unwrap().unwrap();
        assert_eq!(mem.content, "Bob");

        // List
        let all = store.list_all().unwrap();
        assert_eq!(all.len(), 2);

        // Delete
        assert!(store.forget("user_name").unwrap());
        assert!(store.get("user_name").unwrap().is_none());
    }

    #[test]
    fn test_memory_fts_search() {
        let store = MemoryStore::in_memory().unwrap();

        store
            .insert("rust_agent", "Building a Rust-based AI agent framework")
            .unwrap();
        store
            .insert("python_tool", "dietmcp is a Python MCP bridge tool")
            .unwrap();
        store
            .insert("security", "Capability-based permission system for safety")
            .unwrap();

        let results = store.search("Rust agent", 10).unwrap();
        assert!(!results.is_empty());
        assert_eq!(results[0].key, "rust_agent");
    }

    #[test]
    fn test_conversation_persistence() {
        let store = MemoryStore::in_memory().unwrap();

        store.save_conversation("sess_1", "user", "Hello").unwrap();
        store
            .save_conversation("sess_1", "assistant", "Hi there!")
            .unwrap();
        store
            .save_conversation("sess_2", "user", "Different session")
            .unwrap();

        let history = store.get_conversation("sess_1").unwrap();
        assert_eq!(history.len(), 2);
        assert_eq!(history[0].role, "user");
        assert_eq!(history[1].role, "assistant");
    }
}
