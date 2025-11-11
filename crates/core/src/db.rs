use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use rusqlite::{params, Connection};
use time::OffsetDateTime;
use tracing::info;

use crate::encryption::{EnvelopeEncryption, KeyProvider};
use crate::models::{DoseLog, LiteratureEntry, PeptideProtocol};

const DEFAULT_DB_NAME: &str = "peptrack.sqlite";

pub struct StorageConfig<P: KeyProvider + 'static> {
    pub data_dir: Option<PathBuf>,
    pub db_file_name: Option<String>,
    pub key_provider: Arc<P>,
}

impl<P: KeyProvider + 'static> StorageConfig<P> {
    pub fn resolve_path(&self) -> Result<PathBuf> {
        if let Some(explicit) = &self.data_dir {
            return Ok(explicit.join(self.db_file_name.as_deref().unwrap_or(DEFAULT_DB_NAME)));
        }

        let mut dir = data_dir().context("Unable to resolve OS data directory")?;
        dir.push("PepTrack");
        std::fs::create_dir_all(&dir).context("Unable to create PepTrack data directory")?;
        Ok(dir.join(self.db_file_name.as_deref().unwrap_or(DEFAULT_DB_NAME)))
    }
}

pub struct StorageManager<P: KeyProvider + 'static> {
    db_path: PathBuf,
    encryption: EnvelopeEncryption<P>,
}

impl<P: KeyProvider + 'static> StorageManager<P> {
    pub fn new(config: StorageConfig<P>) -> Result<Self> {
        let db_path = config.resolve_path()?;
        let encryption = EnvelopeEncryption::new(config.key_provider);
        Ok(Self {
            db_path,
            encryption,
        })
    }

    fn open_connection(&self) -> Result<Connection> {
        let conn = Connection::open(&self.db_path)
            .with_context(|| format!("Unable to open database at {}", self.db_path.display()))?;
        conn.execute_batch("PRAGMA journal_mode=WAL; PRAGMA foreign_keys=ON;")
            .context("Unable to configure SQLite pragmas")?;
        Ok(conn)
    }

    pub fn initialize(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute_batch(
            r#"
            CREATE TABLE IF NOT EXISTS protocols (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                payload BLOB NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS dose_logs (
                id TEXT PRIMARY KEY,
                protocol_id TEXT NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
                payload BLOB NOT NULL,
                logged_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS literature_cache (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                payload BLOB NOT NULL,
                indexed_at TEXT NOT NULL
            );
            "#,
        )
        .context("Failed to initialize database schema")?;

        info!("Database initialized at {}", self.db_path.display());
        Ok(())
    }

    pub fn upsert_protocol(&self, protocol: &PeptideProtocol) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(protocol).context("Failed to serialize protocol")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO protocols (id, name, payload, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                payload = excluded.payload,
                updated_at = excluded.updated_at;
            "#,
            params![
                protocol.id,
                protocol.name,
                encrypted,
                protocol.updated_at.to_string()
            ],
        )
        .context("Failed to upsert protocol")?;

        Ok(())
    }

    pub fn list_protocols(&self) -> Result<Vec<PeptideProtocol>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM protocols ORDER BY updated_at DESC")?;
        let mut rows = stmt.query([]).context("Unable to run list query")?;
        let mut protocols = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            protocols.push(self.decode_protocol(&blob)?);
        }
        Ok(protocols)
    }

    pub fn append_dose_log(&self, log: &DoseLog) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(log).context("Failed to serialize dose log")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO dose_logs (id, protocol_id, payload, logged_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                payload = excluded.payload,
                logged_at = excluded.logged_at;
            "#,
            params![
                log.id,
                log.protocol_id,
                encrypted,
                log.logged_at.to_string()
            ],
        )
        .context("Failed to append dose log")?;

        Ok(())
    }

    pub fn cache_literature(&self, entry: &LiteratureEntry) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(entry).context("Failed to serialize literature entry")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO literature_cache (id, source, payload, indexed_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                source = excluded.source,
                payload = excluded.payload,
                indexed_at = excluded.indexed_at;
            "#,
            params![
                entry.id,
                entry.source,
                encrypted,
                entry.indexed_at.to_string()
            ],
        )
        .context("Failed to cache literature entry")?;

        Ok(())
    }

    /// Lists all cached literature entries
    ///
    /// Returns entries ordered by indexed date (most recent first).
    pub fn list_literature(&self) -> Result<Vec<LiteratureEntry>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM literature_cache ORDER BY indexed_at DESC")?;
        let mut rows = stmt.query([]).context("Unable to run literature list query")?;
        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            entries.push(self.decode_literature(&blob)?);
        }
        Ok(entries)
    }

    /// Searches cached literature by title or source
    ///
    /// This performs a case-insensitive search on decrypted entries.
    /// For large caches, consider adding FTS (Full Text Search) support.
    pub fn search_literature(&self, query: &str) -> Result<Vec<LiteratureEntry>> {
        let all_entries = self.list_literature()?;
        let query_lower = query.to_lowercase();

        Ok(all_entries
            .into_iter()
            .filter(|entry| {
                entry.title.to_lowercase().contains(&query_lower)
                    || entry.source.to_lowercase().contains(&query_lower)
                    || entry
                        .summary
                        .as_ref()
                        .map(|s| s.to_lowercase().contains(&query_lower))
                        .unwrap_or(false)
            })
            .collect())
    }

    fn decode_protocol(&self, blob: &[u8]) -> Result<PeptideProtocol> {
        let decrypted = self.encryption.open(blob)?;
        let protocol: PeptideProtocol =
            serde_json::from_slice(&decrypted).context("Failed to deserialize protocol")?;
        Ok(protocol)
    }

    fn decode_literature(&self, blob: &[u8]) -> Result<LiteratureEntry> {
        let decrypted = self.encryption.open(blob)?;
        let entry: LiteratureEntry =
            serde_json::from_slice(&decrypted).context("Failed to deserialize literature entry")?;
        Ok(entry)
    }
}

pub fn now_timestamp() -> OffsetDateTime {
    OffsetDateTime::now_utc()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{models::PeptideProtocol, StaticKeyProvider};
    use tempfile::tempdir;

    #[test]
    fn upsert_and_list_protocols_roundtrips() {
        let tmp = tempdir().expect("tempdir");
        let key_provider =
            Arc::new(StaticKeyProvider::new(vec![7u8; 32]).expect("static key provider"));
        let storage = StorageManager::new(StorageConfig {
            data_dir: Some(tmp.path().to_path_buf()),
            db_file_name: Some("test.sqlite".into()),
            key_provider,
        })
        .expect("storage manager");
        storage.initialize().expect("init db");

        let mut protocol = PeptideProtocol::new("Protocol A", "BPC-157");
        protocol.notes = Some("store at 4C".into());

        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let fetched = storage.list_protocols().expect("list");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].name, "Protocol A");
        assert_eq!(fetched[0].notes.as_deref(), Some("store at 4C"));
    }
}
