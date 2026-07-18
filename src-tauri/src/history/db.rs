use rusqlite::{params, Connection};
use serde::Serialize;
use std::path::PathBuf;
use tracing::info;

#[derive(Debug, Clone, Serialize, PartialEq)]
pub struct HistoryEntry {
    pub id: i64,
    pub timestamp: String,
    pub language: String,
    pub original_text: String,
    pub translation: Option<String>,
    pub source_app: Option<String>,
}

pub struct HistoryDb {
    pub(crate) conn: Connection,
}

impl HistoryDb {
    pub fn new(db_path: &PathBuf) -> anyhow::Result<Self> {
        if let Some(parent) = db_path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let conn = Connection::open(db_path)?;

        conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                timestamp TEXT NOT NULL,
                language TEXT NOT NULL,
                original_text TEXT NOT NULL,
                translation TEXT,
                source_app TEXT
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS history_fts USING fts5(
                original_text,
                translation,
                content=history,
                content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS history_ai AFTER INSERT ON history BEGIN
                INSERT INTO history_fts(rowid, original_text, translation)
                VALUES (new.id, new.original_text, new.translation);
            END;

            CREATE TRIGGER IF NOT EXISTS history_ad AFTER DELETE ON history BEGIN
                INSERT INTO history_fts(history_fts, rowid, original_text, translation)
                VALUES ('delete', old.id, old.original_text, old.translation);
            END;

            CREATE TRIGGER IF NOT EXISTS history_au AFTER UPDATE ON history BEGIN
                INSERT INTO history_fts(history_fts, rowid, original_text, translation)
                VALUES ('delete', old.id, old.original_text, old.translation);
                INSERT INTO history_fts(rowid, original_text, translation)
                VALUES (new.id, new.original_text, new.translation);
            END;
            "
        )?;

        info!("History database initialized at {:?}", db_path);
        Ok(Self { conn })
    }

    pub fn insert(
        &self,
        language: &str,
        original_text: &str,
        translation: Option<&str>,
        source_app: Option<&str>,
    ) -> anyhow::Result<i64> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO history (timestamp, language, original_text, translation, source_app)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![timestamp, language, original_text, translation, source_app],
        )?;

        let id = self.conn.last_insert_rowid();
        info!("Inserted history entry with id: {}", id);
        Ok(id)
    }

    pub fn get_all(&self, limit: i64) -> anyhow::Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, timestamp, language, original_text, translation, source_app
             FROM history ORDER BY id DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![limit], |row| {
                Ok(HistoryEntry {
                    id: row.get(0)?,
                    timestamp: row.get(1)?,
                    language: row.get(2)?,
                    original_text: row.get(3)?,
                    translation: row.get(4)?,
                    source_app: row.get(5)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn delete(&self, id: i64) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM history WHERE id = ?1", params![id])?;
        info!("Deleted history entry with id: {}", id);
        Ok(())
    }

    pub fn clear(&self) -> anyhow::Result<()> {
        self.conn.execute("DELETE FROM history", params![])?;
        info!("History cleared");
        Ok(())
    }

    // ── Video Transcriptions ──────────────────────────────────────

    pub fn init_video_transcriptions_table(&self) -> anyhow::Result<()> {
        self.conn.execute_batch(
            "
            CREATE TABLE IF NOT EXISTS video_transcriptions (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                video_path TEXT NOT NULL,
                video_name TEXT NOT NULL,
                duration_seconds REAL,
                language TEXT NOT NULL,
                full_text TEXT NOT NULL,
                translated_text TEXT,
                target_language TEXT,
                summary TEXT,
                segments TEXT NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE VIRTUAL TABLE IF NOT EXISTS video_fts USING fts5(
                full_text,
                summary,
                content=video_transcriptions,
                content_rowid=id
            );

            CREATE TRIGGER IF NOT EXISTS video_transcriptions_ai AFTER INSERT ON video_transcriptions BEGIN
                INSERT INTO video_fts(rowid, full_text, summary)
                VALUES (new.id, new.full_text, new.summary);
            END;

            CREATE TRIGGER IF NOT EXISTS video_transcriptions_ad AFTER DELETE ON video_transcriptions BEGIN
                INSERT INTO video_fts(video_fts, rowid, full_text, summary)
                VALUES ('delete', old.id, old.full_text, old.summary);
            END;

            CREATE TRIGGER IF NOT EXISTS video_transcriptions_au AFTER UPDATE ON video_transcriptions BEGIN
                INSERT INTO video_fts(video_fts, rowid, full_text, summary)
                VALUES ('delete', old.id, old.full_text, old.summary);
                INSERT INTO video_fts(rowid, full_text, summary)
                VALUES (new.id, new.full_text, new.summary);
            END;
            "
        )?;

        // Add columns if they don't exist (for existing databases)
        let columns: Vec<String> = {
            let mut stmt = self.conn.prepare("PRAGMA table_info(video_transcriptions)")?;
            let rows = stmt.query_map([], |row| row.get::<_, String>(1))?;
            rows.filter_map(|r| r.ok()).collect()
        };

        if !columns.contains(&"translated_text".to_string()) {
            self.conn.execute_batch("ALTER TABLE video_transcriptions ADD COLUMN translated_text TEXT")?;
        }
        if !columns.contains(&"target_language".to_string()) {
            self.conn.execute_batch("ALTER TABLE video_transcriptions ADD COLUMN target_language TEXT")?;
        }

        info!("Video transcriptions table initialized");
        Ok(())
    }

    pub fn insert_video_transcription(
        &self,
        video_path: &str,
        video_name: &str,
        duration_seconds: f64,
        language: &str,
        full_text: &str,
        translated_text: Option<&str>,
        target_language: Option<&str>,
        segments: &str,
    ) -> anyhow::Result<i64> {
        let timestamp = chrono::Utc::now().to_rfc3339();

        self.conn.execute(
            "INSERT INTO video_transcriptions (video_path, video_name, duration_seconds, language, full_text, translated_text, target_language, segments, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![video_path, video_name, duration_seconds, language, full_text, translated_text, target_language, segments, timestamp],
        )?;

        let id = self.conn.last_insert_rowid();
        info!("Inserted video transcription with id: {}", id);
        Ok(id)
    }

    pub fn get_video_transcriptions(&self, limit: i64) -> anyhow::Result<Vec<crate::commands::video_transcription::VideoTranscriptionEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, video_path, video_name, duration_seconds, language, full_text, translated_text, target_language, summary, segments, created_at
             FROM video_transcriptions ORDER BY id DESC LIMIT ?1",
        )?;

        let entries = stmt
            .query_map(params![limit], |row| {
                let segments_str: String = row.get(9)?;
                let segments: Vec<crate::commands::video_transcription::DiarizedSegment> =
                    serde_json::from_str(&segments_str).unwrap_or_default();

                Ok(crate::commands::video_transcription::VideoTranscriptionEntry {
                    id: row.get(0)?,
                    video_path: row.get(1)?,
                    video_name: row.get(2)?,
                    duration_seconds: row.get(3)?,
                    language: row.get(4)?,
                    full_text: row.get(5)?,
                    translated_text: row.get(6)?,
                    target_language: row.get(7)?,
                    summary: row.get(8)?,
                    segments,
                    created_at: row.get(10)?,
                })
            })?
            .collect::<Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    pub fn get_video_transcription(&self, id: i64) -> anyhow::Result<crate::commands::video_transcription::VideoTranscriptionEntry> {
        let mut stmt = self.conn.prepare(
            "SELECT id, video_path, video_name, duration_seconds, language, full_text, translated_text, target_language, summary, segments, created_at
             FROM video_transcriptions WHERE id = ?1",
        )?;

        let entry = stmt.query_row(params![id], |row| {
            let segments_str: String = row.get(9)?;
            let segments: Vec<crate::commands::video_transcription::DiarizedSegment> =
                serde_json::from_str(&segments_str).unwrap_or_default();

            Ok(crate::commands::video_transcription::VideoTranscriptionEntry {
                id: row.get(0)?,
                video_path: row.get(1)?,
                video_name: row.get(2)?,
                duration_seconds: row.get(3)?,
                language: row.get(4)?,
                full_text: row.get(5)?,
                translated_text: row.get(6)?,
                target_language: row.get(7)?,
                summary: row.get(8)?,
                segments,
                created_at: row.get(10)?,
            })
        })?;

        Ok(entry)
    }

    pub fn delete_video_transcription(&self, id: i64) -> anyhow::Result<()> {
        self.conn
            .execute("DELETE FROM video_transcriptions WHERE id = ?1", params![id])?;
        info!("Deleted video transcription with id: {}", id);
        Ok(())
    }

    pub fn update_video_transcription_summary(&self, id: i64, summary: &str) -> anyhow::Result<()> {
        self.conn.execute(
            "UPDATE video_transcriptions SET summary = ?1 WHERE id = ?2",
            params![summary, id],
        )?;
        info!("Updated summary for video transcription id: {}", id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU64, Ordering};

    static COUNTER: AtomicU64 = AtomicU64::new(0);

    fn test_db() -> (HistoryDb, PathBuf) {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "livetext_history_test_{}_{:?}",
            std::process::id(),
            id
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let db_path = dir.join("test_history.db");
        let db = HistoryDb::new(&db_path).unwrap();
        (db, dir)
    }

    fn cleanup(dir: &PathBuf) {
        std::fs::remove_dir_all(dir).ok();
    }

    // ── Insert ────────────────────────────────────────────────────

    #[test]
    fn test_insert_entry() {
        let (db, dir) = test_db();
        let id = db.insert("en", "Hello world", None, None).unwrap();
        assert!(id > 0);
        cleanup(&dir);
    }

    #[test]
    fn test_insert_with_translation() {
        let (db, dir) = test_db();
        let id = db.insert("en", "Hello", Some("Hola"), None).unwrap();
        assert!(id > 0);
        let entries = db.get_all(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].translation.as_deref(), Some("Hola"));
        cleanup(&dir);
    }

    #[test]
    fn test_insert_with_source_app() {
        let (db, dir) = test_db();
        let id = db.insert("en", "Test", None, Some("firefox")).unwrap();
        assert!(id > 0);
        let entries = db.get_all(10).unwrap();
        assert_eq!(entries[0].source_app.as_deref(), Some("firefox"));
        cleanup(&dir);
    }

    #[test]
    fn test_insert_multiple_entries() {
        let (db, dir) = test_db();
        let id1 = db.insert("en", "First", None, None).unwrap();
        let id2 = db.insert("es", "Segundo", None, None).unwrap();
        let id3 = db.insert("fr", "Troisième", None, None).unwrap();
        assert!(id1 < id2);
        assert!(id2 < id3);
        assert_eq!(db.get_all(100).unwrap().len(), 3);
        cleanup(&dir);
    }

    // ── Get all ───────────────────────────────────────────────────

    #[test]
    fn test_get_all_empty() {
        let (db, dir) = test_db();
        let entries = db.get_all(100).unwrap();
        assert!(entries.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_get_all_ordered_desc() {
        let (db, dir) = test_db();
        db.insert("en", "First", None, None).unwrap();
        db.insert("en", "Second", None, None).unwrap();
        db.insert("en", "Third", None, None).unwrap();

        let entries = db.get_all(100).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].original_text, "Third");
        assert_eq!(entries[1].original_text, "Second");
        assert_eq!(entries[2].original_text, "First");
        cleanup(&dir);
    }

    #[test]
    fn test_get_all_with_limit() {
        let (db, dir) = test_db();
        for i in 0..10 {
            db.insert("en", &format!("Entry {}", i), None, None).unwrap();
        }

        let entries = db.get_all(3).unwrap();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].original_text, "Entry 9");
        assert_eq!(entries[1].original_text, "Entry 8");
        assert_eq!(entries[2].original_text, "Entry 7");
        cleanup(&dir);
    }

    #[test]
    fn test_get_all_entry_fields() {
        let (db, dir) = test_db();
        db.insert("es", "Hola mundo", Some("Hello world"), Some("chrome")).unwrap();

        let entries = db.get_all(10).unwrap();
        assert_eq!(entries.len(), 1);
        let entry = &entries[0];
        assert!(entry.id > 0);
        assert!(!entry.timestamp.is_empty());
        assert_eq!(entry.language, "es");
        assert_eq!(entry.original_text, "Hola mundo");
        assert_eq!(entry.translation.as_deref(), Some("Hello world"));
        assert_eq!(entry.source_app.as_deref(), Some("chrome"));
        cleanup(&dir);
    }

    // ── Delete ────────────────────────────────────────────────────

    #[test]
    fn test_delete_entry() {
        let (db, dir) = test_db();
        let id = db.insert("en", "To delete", None, None).unwrap();
        db.delete(id).unwrap();
        let entries = db.get_all(10).unwrap();
        assert!(entries.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_delete_nonexistent_entry() {
        let (db, dir) = test_db();
        db.delete(99999).unwrap();
        cleanup(&dir);
    }

    #[test]
    fn test_delete_only_one() {
        let (db, dir) = test_db();
        let id1 = db.insert("en", "Keep", None, None).unwrap();
        let id2 = db.insert("en", "Delete", None, None).unwrap();
        db.delete(id2).unwrap();

        let entries = db.get_all(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].id, id1);
        cleanup(&dir);
    }

    // ── Clear ─────────────────────────────────────────────────────

    #[test]
    fn test_clear_all() {
        let (db, dir) = test_db();
        db.insert("en", "One", None, None).unwrap();
        db.insert("es", "Dos", None, None).unwrap();
        db.insert("fr", "Trois", None, None).unwrap();

        db.clear().unwrap();
        let entries = db.get_all(100).unwrap();
        assert!(entries.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_clear_empty_db() {
        let (db, dir) = test_db();
        db.clear().unwrap();
        let entries = db.get_all(10).unwrap();
        assert!(entries.is_empty());
        cleanup(&dir);
    }

    // ── FTS Search ────────────────────────────────────────────────

    #[test]
    fn test_search_finds_match() {
        let (db, dir) = test_db();
        db.insert("en", "Hello world", None, None).unwrap();
        db.insert("en", "Goodbye world", None, None).unwrap();

        let result = db.search("Hello", 10).unwrap();
        assert_eq!(result.total, 1);
        assert_eq!(result.entries[0].original_text, "Hello world");
        cleanup(&dir);
    }

    #[test]
    fn test_search_no_match() {
        let (db, dir) = test_db();
        db.insert("en", "Hello world", None, None).unwrap();

        let result = db.search("nonexistent", 10).unwrap();
        assert_eq!(result.total, 0);
        assert!(result.entries.is_empty());
        cleanup(&dir);
    }

    #[test]
    fn test_search_in_translation() {
        let (db, dir) = test_db();
        db.insert("en", "Hello", Some("Hola"), None).unwrap();

        let result = db.search("Hola", 10).unwrap();
        assert_eq!(result.total, 1);
        cleanup(&dir);
    }

    #[test]
    fn test_search_limit() {
        let (db, dir) = test_db();
        for i in 0..10 {
            db.insert("en", &format!("test entry {}", i), None, None).unwrap();
        }

        let result = db.search("test", 3).unwrap();
        assert_eq!(result.total, 3);
        cleanup(&dir);
    }

    // ── Database initialization ───────────────────────────────────

    #[test]
    fn test_new_creates_parent_dirs() {
        let id = COUNTER.fetch_add(1, Ordering::SeqCst);
        let dir = std::env::temp_dir().join(format!(
            "livetext_nested_test_{}_{:?}/sub/dir",
            std::process::id(),
            id
        ));
        let db_path = dir.join("test.db");
        let _db = HistoryDb::new(&db_path).unwrap();
        assert!(db_path.exists());
        // Cleanup parent
        let parent = std::env::temp_dir().join(format!(
            "livetext_nested_test_{}_{:?}",
            std::process::id(),
            id
        ));
        std::fs::remove_dir_all(parent).ok();
    }

    #[test]
    fn test_reopen_existing_db() {
        let (db, dir) = test_db();
        let db_path = dir.join("test_history.db");

        {
            db.insert("en", "Persistent data", None, None).unwrap();
        }

        {
            let db2 = HistoryDb::new(&db_path).unwrap();
            let entries = db2.get_all(10).unwrap();
            assert_eq!(entries.len(), 1);
            assert_eq!(entries[0].original_text, "Persistent data");
        }

        cleanup(&dir);
    }
}
