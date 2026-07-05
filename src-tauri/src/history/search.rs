use rusqlite::params;
use super::db::HistoryDb;
use super::db::HistoryEntry;

pub struct SearchResult {
    pub entries: Vec<HistoryEntry>,
    pub total: usize,
}

impl HistoryDb {
    pub fn search(&self, query: &str, limit: i64) -> anyhow::Result<SearchResult> {
        let mut stmt = self.conn.prepare(
            "SELECT h.id, h.timestamp, h.language, h.original_text, h.translation, h.source_app
             FROM history h
             INNER JOIN history_fts fts ON h.id = fts.rowid
             WHERE history_fts MATCH ?1
             ORDER BY h.id DESC
             LIMIT ?2",
        )?;

        let entries = stmt
            .query_map(params![query, limit], |row| {
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

        let total = entries.len();
        Ok(SearchResult { entries, total })
    }
}
