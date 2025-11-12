use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use rusqlite::{params, Connection};
use time::OffsetDateTime;
use tracing::info;

use crate::encryption::{EnvelopeEncryption, KeyProvider};
use crate::models::{
    Alert, DoseLog, InventoryItem, LiteratureEntry, PeptideProtocol,
    PriceHistory, Supplier, SummaryHistory,
};

const DEFAULT_DB_NAME: &str = "peptrack.sqlite";

pub struct StorageConfig {
    pub data_dir: Option<PathBuf>,
    pub db_file_name: Option<String>,
    pub key_provider: Arc<dyn KeyProvider>,
}

impl StorageConfig {
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

pub struct StorageManager {
    db_path: PathBuf,
    encryption: EnvelopeEncryption,
}

impl StorageManager {
    pub fn new(config: StorageConfig) -> Result<Self> {
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

            CREATE TABLE IF NOT EXISTS suppliers (
                id TEXT PRIMARY KEY,
                name TEXT NOT NULL,
                payload BLOB NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS inventory (
                id TEXT PRIMARY KEY,
                protocol_id TEXT NOT NULL REFERENCES protocols(id) ON DELETE CASCADE,
                supplier_id TEXT REFERENCES suppliers(id) ON DELETE SET NULL,
                payload BLOB NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE TABLE IF NOT EXISTS price_history (
                id TEXT PRIMARY KEY,
                supplier_id TEXT NOT NULL REFERENCES suppliers(id) ON DELETE CASCADE,
                peptide_name TEXT NOT NULL,
                payload BLOB NOT NULL,
                recorded_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_price_history_supplier_peptide
                ON price_history(supplier_id, peptide_name, recorded_at DESC);

            CREATE TABLE IF NOT EXISTS alerts (
                id TEXT PRIMARY KEY,
                alert_type TEXT NOT NULL,
                severity TEXT NOT NULL,
                payload BLOB NOT NULL,
                is_read INTEGER NOT NULL DEFAULT 0,
                is_dismissed INTEGER NOT NULL DEFAULT 0,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_alerts_not_dismissed
                ON alerts(is_dismissed, created_at DESC) WHERE is_dismissed = 0;

            CREATE TABLE IF NOT EXISTS summary_history (
                id TEXT PRIMARY KEY,
                title TEXT NOT NULL,
                payload BLOB NOT NULL,
                created_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_summary_history_created
                ON summary_history(created_at DESC);
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

    /// Lists all dose logs across all protocols
    ///
    /// Returns logs ordered by logged_at (most recent first).
    pub fn list_dose_logs(&self) -> Result<Vec<DoseLog>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM dose_logs ORDER BY logged_at DESC")?;
        let mut rows = stmt.query([]).context("Unable to run dose logs query")?;
        let mut logs = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            logs.push(self.decode_dose_log(&blob)?);
        }
        Ok(logs)
    }

    /// Lists dose logs for a specific protocol
    ///
    /// Returns logs ordered by logged_at (most recent first).
    pub fn list_dose_logs_for_protocol(&self, protocol_id: &str) -> Result<Vec<DoseLog>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM dose_logs WHERE protocol_id = ?1 ORDER BY logged_at DESC",
        )?;
        let mut rows = stmt
            .query([protocol_id])
            .context("Unable to run dose logs query")?;
        let mut logs = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            logs.push(self.decode_dose_log(&blob)?);
        }
        Ok(logs)
    }

    /// Deletes a specific dose log by ID
    pub fn delete_dose_log(&self, log_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM dose_logs WHERE id = ?1", params![log_id])
            .context("Failed to delete dose log")?;
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
        let mut stmt =
            conn.prepare("SELECT payload FROM literature_cache ORDER BY indexed_at DESC")?;
        let mut rows = stmt
            .query([])
            .context("Unable to run literature list query")?;
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

    // Supplier CRUD operations

    pub fn upsert_supplier(&self, supplier: &Supplier) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(supplier).context("Failed to serialize supplier")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO suppliers (id, name, payload, updated_at)
            VALUES (?1, ?2, ?3, ?4)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                payload = excluded.payload,
                updated_at = excluded.updated_at;
            "#,
            params![
                supplier.id,
                supplier.name,
                encrypted,
                supplier.updated_at.to_string()
            ],
        )
        .context("Failed to upsert supplier")?;

        Ok(())
    }

    pub fn list_suppliers(&self) -> Result<Vec<Supplier>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM suppliers ORDER BY name ASC")?;
        let mut rows = stmt
            .query([])
            .context("Unable to run supplier list query")?;
        let mut suppliers = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            suppliers.push(self.decode_supplier(&blob)?);
        }
        Ok(suppliers)
    }

    pub fn get_supplier(&self, supplier_id: &str) -> Result<Option<Supplier>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM suppliers WHERE id = ?1")?;
        let mut rows = stmt.query(params![supplier_id])?;

        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            Ok(Some(self.decode_supplier(&blob)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_supplier(&self, supplier_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM suppliers WHERE id = ?1", params![supplier_id])
            .context("Failed to delete supplier")?;
        Ok(())
    }

    // Inventory CRUD operations

    pub fn upsert_inventory_item(&self, item: &InventoryItem) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(item).context("Failed to serialize inventory item")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO inventory (id, protocol_id, supplier_id, payload, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                protocol_id = excluded.protocol_id,
                supplier_id = excluded.supplier_id,
                payload = excluded.payload,
                updated_at = excluded.updated_at;
            "#,
            params![
                item.id,
                item.protocol_id,
                item.supplier_id,
                encrypted,
                item.updated_at.to_string()
            ],
        )
        .context("Failed to upsert inventory item")?;

        Ok(())
    }

    pub fn list_inventory(&self) -> Result<Vec<InventoryItem>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM inventory ORDER BY updated_at DESC")?;
        let mut rows = stmt
            .query([])
            .context("Unable to run inventory list query")?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            items.push(self.decode_inventory_item(&blob)?);
        }
        Ok(items)
    }

    pub fn list_inventory_by_protocol(&self, protocol_id: &str) -> Result<Vec<InventoryItem>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM inventory WHERE protocol_id = ?1 ORDER BY updated_at DESC",
        )?;
        let mut rows = stmt
            .query(params![protocol_id])
            .context("Unable to run inventory query for protocol")?;
        let mut items = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            items.push(self.decode_inventory_item(&blob)?);
        }
        Ok(items)
    }

    pub fn get_inventory_item(&self, item_id: &str) -> Result<Option<InventoryItem>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM inventory WHERE id = ?1")?;
        let mut rows = stmt.query(params![item_id])?;

        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            Ok(Some(self.decode_inventory_item(&blob)?))
        } else {
            Ok(None)
        }
    }

    pub fn delete_inventory_item(&self, item_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM inventory WHERE id = ?1", params![item_id])
            .context("Failed to delete inventory item")?;
        Ok(())
    }

    // Decode helper functions

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

    fn decode_dose_log(&self, blob: &[u8]) -> Result<DoseLog> {
        let decrypted = self.encryption.open(blob)?;
        let log: DoseLog =
            serde_json::from_slice(&decrypted).context("Failed to deserialize dose log")?;
        Ok(log)
    }

    fn decode_supplier(&self, blob: &[u8]) -> Result<Supplier> {
        let decrypted = self.encryption.open(blob)?;
        let supplier: Supplier =
            serde_json::from_slice(&decrypted).context("Failed to deserialize supplier")?;
        Ok(supplier)
    }

    fn decode_inventory_item(&self, blob: &[u8]) -> Result<InventoryItem> {
        let decrypted = self.encryption.open(blob)?;
        let item: InventoryItem =
            serde_json::from_slice(&decrypted).context("Failed to deserialize inventory item")?;
        Ok(item)
    }

    // Price History CRUD operations

    pub fn add_price_history(&self, entry: &PriceHistory) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(entry).context("Failed to serialize price history")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO price_history (id, supplier_id, peptide_name, payload, recorded_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            "#,
            params![
                entry.id,
                entry.supplier_id,
                entry.peptide_name,
                encrypted,
                entry.recorded_at.to_string()
            ],
        )
        .context("Failed to add price history")?;

        Ok(())
    }

    pub fn list_price_history_for_supplier(
        &self,
        supplier_id: &str,
        peptide_name: Option<&str>,
    ) -> Result<Vec<PriceHistory>> {
        let conn = self.open_connection()?;

        let (query, params): (String, Vec<&str>) = if let Some(peptide) = peptide_name {
            (
                "SELECT payload FROM price_history WHERE supplier_id = ?1 AND peptide_name = ?2 ORDER BY recorded_at DESC".into(),
                vec![supplier_id, peptide],
            )
        } else {
            (
                "SELECT payload FROM price_history WHERE supplier_id = ?1 ORDER BY recorded_at DESC".into(),
                vec![supplier_id],
            )
        };

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt
            .query(rusqlite::params_from_iter(params.iter()))
            .context("Unable to query price history")?;

        let mut entries = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            entries.push(self.decode_price_history(&blob)?);
        }
        Ok(entries)
    }

    pub fn get_latest_price(
        &self,
        supplier_id: &str,
        peptide_name: &str,
    ) -> Result<Option<PriceHistory>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare(
            "SELECT payload FROM price_history WHERE supplier_id = ?1 AND peptide_name = ?2 ORDER BY recorded_at DESC LIMIT 1"
        )?;
        let mut rows = stmt.query(params![supplier_id, peptide_name])?;

        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            Ok(Some(self.decode_price_history(&blob)?))
        } else {
            Ok(None)
        }
    }

    // Alert CRUD operations

    pub fn create_alert(&self, alert: &Alert) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(alert).context("Failed to serialize alert")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO alerts (id, alert_type, severity, payload, is_read, is_dismissed, created_at)
            VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
            "#,
            params![
                alert.id,
                serde_json::to_string(&alert.alert_type)?,
                serde_json::to_string(&alert.severity)?,
                encrypted,
                alert.is_read as i32,
                alert.is_dismissed as i32,
                alert.created_at.to_string()
            ],
        )
        .context("Failed to create alert")?;

        Ok(())
    }

    pub fn list_alerts(&self, include_dismissed: bool) -> Result<Vec<Alert>> {
        let conn = self.open_connection()?;

        let query = if include_dismissed {
            "SELECT payload FROM alerts ORDER BY created_at DESC"
        } else {
            "SELECT payload FROM alerts WHERE is_dismissed = 0 ORDER BY created_at DESC"
        };

        let mut stmt = conn.prepare(query)?;
        let mut rows = stmt
            .query([])
            .context("Unable to query alerts")?;

        let mut alerts = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            alerts.push(self.decode_alert(&blob)?);
        }
        Ok(alerts)
    }

    pub fn mark_alert_read(&self, alert_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute(
            "UPDATE alerts SET is_read = 1 WHERE id = ?1",
            params![alert_id],
        )
        .context("Failed to mark alert as read")?;
        Ok(())
    }

    pub fn dismiss_alert(&self, alert_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute(
            "UPDATE alerts SET is_dismissed = 1 WHERE id = ?1",
            params![alert_id],
        )
        .context("Failed to dismiss alert")?;
        Ok(())
    }

    pub fn clear_all_alerts(&self) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM alerts", [])
            .context("Failed to clear alerts")?;
        Ok(())
    }

    // Summary History CRUD operations

    pub fn save_summary(&self, summary: &SummaryHistory) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(summary).context("Failed to serialize summary")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO summary_history (id, title, payload, created_at)
            VALUES (?1, ?2, ?3, ?4)
            "#,
            params![
                summary.id,
                summary.title,
                encrypted,
                summary.created_at.to_string()
            ],
        )
        .context("Failed to save summary")?;

        Ok(())
    }

    pub fn list_summary_history(&self, limit: Option<usize>) -> Result<Vec<SummaryHistory>> {
        let conn = self.open_connection()?;

        let query = if let Some(lim) = limit {
            format!("SELECT payload FROM summary_history ORDER BY created_at DESC LIMIT {}", lim)
        } else {
            "SELECT payload FROM summary_history ORDER BY created_at DESC".into()
        };

        let mut stmt = conn.prepare(&query)?;
        let mut rows = stmt
            .query([])
            .context("Unable to query summary history")?;

        let mut summaries = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            summaries.push(self.decode_summary_history(&blob)?);
        }
        Ok(summaries)
    }

    pub fn delete_summary(&self, summary_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM summary_history WHERE id = ?1", params![summary_id])
            .context("Failed to delete summary")?;
        Ok(())
    }

    // Decoder helper functions

    fn decode_price_history(&self, blob: &[u8]) -> Result<PriceHistory> {
        let decrypted = self.encryption.open(blob)?;
        let entry: PriceHistory =
            serde_json::from_slice(&decrypted).context("Failed to deserialize price history")?;
        Ok(entry)
    }

    fn decode_alert(&self, blob: &[u8]) -> Result<Alert> {
        let decrypted = self.encryption.open(blob)?;
        let alert: Alert =
            serde_json::from_slice(&decrypted).context("Failed to deserialize alert")?;
        Ok(alert)
    }

    fn decode_summary_history(&self, blob: &[u8]) -> Result<SummaryHistory> {
        let decrypted = self.encryption.open(blob)?;
        let summary: SummaryHistory =
            serde_json::from_slice(&decrypted).context("Failed to deserialize summary history")?;
        Ok(summary)
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
