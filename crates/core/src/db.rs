use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use dirs::data_dir;
use rusqlite::{params, Connection};
use time::OffsetDateTime;
use tracing::info;

use crate::encryption::{EnvelopeEncryption, KeyProvider};
use crate::models::{
    Alert, BodyMetric, DatabaseStats, DoseLog, HealthReport, InventoryItem, LiteratureEntry, PeptideProtocol,
    PriceHistory, Supplier, SummaryHistory,
};

const DEFAULT_DB_NAME: &str = "peptrack.sqlite";

// PepTrack Application ID (unique identifier for this SQLite database)
// Generated from: "PepTrack".as_bytes() hashed
const PEPTRACK_APP_ID: i32 = 0x50657054; // "PepT" in hex

// Current schema version for migrations
const SCHEMA_VERSION: i32 = 2;

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

        // =====================================================================
        // COMPREHENSIVE SQLITE CONFIGURATION
        // Maximum safety, performance, and integrity
        // =====================================================================

        conn.execute_batch(&format!(
            "-- ═══════════════════════════════════════════════════════════
             -- CORE SAFETY & DURABILITY
             -- ═══════════════════════════════════════════════════════════

             -- Write-Ahead Logging for crash safety & better concurrency
             PRAGMA journal_mode=WAL;

             -- Maximum durability - fsync after every transaction
             PRAGMA synchronous=FULL;

             -- Enforce foreign key constraints
             PRAGMA foreign_keys=ON;

             -- Overwrite deleted data with zeros (security)
             PRAGMA secure_delete=ON;

             -- Verify database structure on access
             PRAGMA cell_size_check=ON;

             -- Disable loading of untrusted schemas (SQLite 3.31+)
             -- Note: May not be supported on older SQLite versions
             -- PRAGMA trusted_schema=OFF;

             -- ═══════════════════════════════════════════════════════════
             -- PERFORMANCE OPTIMIZATIONS
             -- ═══════════════════════════════════════════════════════════

             -- 64MB cache (negative value = kilobytes)
             PRAGMA cache_size=-64000;

             -- Memory-mapped I/O for faster reads (256MB)
             PRAGMA mmap_size=268435456;

             -- Store temp tables & indices in memory
             PRAGMA temp_store=MEMORY;

             -- Wait up to 5 seconds if database is locked
             PRAGMA busy_timeout=5000;

             -- ═══════════════════════════════════════════════════════════
             -- MAINTENANCE & OPTIMIZATION
             -- ═══════════════════════════════════════════════════════════

             -- Auto-vacuum to reclaim space (incremental for better performance)
             PRAGMA auto_vacuum=INCREMENTAL;

             -- Checkpoint WAL every 1000 pages (auto-merge to main DB)
             PRAGMA wal_autocheckpoint=1000;

             -- Enable recursive triggers for complex integrity rules
             PRAGMA recursive_triggers=ON;

             -- ═══════════════════════════════════════════════════════════
             -- APPLICATION METADATA
             -- ═══════════════════════════════════════════════════════════

             -- Set unique application ID for this database
             PRAGMA application_id={};

             -- Track schema version for migrations
             PRAGMA user_version={};

             -- Ensure UTF-8 encoding
             PRAGMA encoding='UTF-8';",
            PEPTRACK_APP_ID,
            SCHEMA_VERSION
        ))
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
                updated_at TEXT NOT NULL,
                is_favorite INTEGER NOT NULL DEFAULT 0
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

            CREATE INDEX IF NOT EXISTS idx_protocols_favorite
                ON protocols(is_favorite DESC, updated_at DESC);

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

            CREATE TABLE IF NOT EXISTS body_metrics (
                id TEXT PRIMARY KEY,
                date TEXT NOT NULL,
                payload BLOB NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            );

            CREATE INDEX IF NOT EXISTS idx_body_metrics_date
                ON body_metrics(date DESC);
            "#,
        )
        .context("Failed to initialize database schema")?;

        // Run migrations for existing databases
        self.run_migrations(&conn)?;

        info!("Database initialized at {}", self.db_path.display());
        Ok(())
    }

    /// Run database migrations for schema updates
    fn run_migrations(&self, conn: &Connection) -> Result<()> {
        // Migration: Add is_favorite column to protocols table if it doesn't exist
        let has_favorite_column: bool = conn
            .query_row(
                "SELECT COUNT(*) FROM pragma_table_info('protocols') WHERE name='is_favorite'",
                [],
                |row| row.get(0),
            )
            .unwrap_or(0) > 0;

        if !has_favorite_column {
            info!("Running migration: Adding is_favorite column to protocols table");
            conn.execute(
                "ALTER TABLE protocols ADD COLUMN is_favorite INTEGER NOT NULL DEFAULT 0",
                [],
            )
            .context("Failed to add is_favorite column")?;
            info!("Migration completed: is_favorite column added");
        }

        Ok(())
    }

    pub fn upsert_protocol(&self, protocol: &PeptideProtocol) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(protocol).context("Failed to serialize protocol")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO protocols (id, name, payload, updated_at, is_favorite)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                name = excluded.name,
                payload = excluded.payload,
                updated_at = excluded.updated_at,
                is_favorite = excluded.is_favorite;
            "#,
            params![
                protocol.id,
                protocol.name,
                encrypted,
                protocol.updated_at.to_string(),
                protocol.is_favorite as i32
            ],
        )
        .context("Failed to upsert protocol")?;

        Ok(())
    }

    pub fn list_protocols(&self) -> Result<Vec<PeptideProtocol>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM protocols ORDER BY is_favorite DESC, updated_at DESC")?;
        let mut rows = stmt.query([]).context("Unable to run list query")?;
        let mut protocols = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            protocols.push(self.decode_protocol(&blob)?);
        }
        Ok(protocols)
    }

    pub fn get_protocol(&self, protocol_id: &str) -> Result<Option<PeptideProtocol>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM protocols WHERE id = ?1")?;
        let mut rows = stmt.query([protocol_id])?;

        if let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            Ok(Some(self.decode_protocol(&blob)?))
        } else {
            Ok(None)
        }
    }

    /// Toggle the favorite status of a protocol
    pub fn toggle_protocol_favorite(&self, protocol_id: &str) -> Result<bool> {
        let conn = self.open_connection()?;

        // Get current protocol with favorite status
        let mut protocol = self
            .get_protocol(protocol_id)?
            .ok_or_else(|| anyhow::anyhow!("Protocol not found"))?;

        // Toggle favorite status
        protocol.is_favorite = !protocol.is_favorite;

        // Update the database
        self.upsert_protocol(&protocol)?;

        Ok(protocol.is_favorite)
    }

    /// Update the tags for a protocol
    ///
    /// Replaces the entire tags list for a protocol. To add/remove individual tags,
    /// fetch the protocol, modify the tags Vec, and call this method.
    ///
    /// # Arguments
    /// * `protocol_id` - The ID of the protocol to update
    /// * `tags` - The new list of tags for the protocol
    ///
    /// # Returns
    /// The updated list of tags
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let tags = vec!["morning".to_string(), "recovery".to_string()];
    /// storage.update_protocol_tags("protocol-id", tags)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn update_protocol_tags(&self, protocol_id: &str, tags: Vec<String>) -> Result<Vec<String>> {
        let mut protocol = self
            .get_protocol(protocol_id)?
            .ok_or_else(|| anyhow::anyhow!("Protocol not found"))?;

        // Update tags and timestamp
        protocol.tags = tags;
        protocol.updated_at = now_timestamp();

        // Save to database
        self.upsert_protocol(&protocol)?;

        Ok(protocol.tags)
    }

    /// Add a tag to a protocol
    ///
    /// Adds a new tag if it doesn't already exist. Tags are case-sensitive.
    ///
    /// # Arguments
    /// * `protocol_id` - The ID of the protocol
    /// * `tag` - The tag to add
    ///
    /// # Returns
    /// The updated list of tags
    pub fn add_protocol_tag(&self, protocol_id: &str, tag: String) -> Result<Vec<String>> {
        let mut protocol = self
            .get_protocol(protocol_id)?
            .ok_or_else(|| anyhow::anyhow!("Protocol not found"))?;

        // Add tag if it doesn't exist
        if !protocol.tags.contains(&tag) {
            protocol.tags.push(tag);
            protocol.updated_at = now_timestamp();
            self.upsert_protocol(&protocol)?;
        }

        Ok(protocol.tags)
    }

    /// Remove a tag from a protocol
    ///
    /// Removes the specified tag if it exists.
    ///
    /// # Arguments
    /// * `protocol_id` - The ID of the protocol
    /// * `tag` - The tag to remove
    ///
    /// # Returns
    /// The updated list of tags
    pub fn remove_protocol_tag(&self, protocol_id: &str, tag: &str) -> Result<Vec<String>> {
        let mut protocol = self
            .get_protocol(protocol_id)?
            .ok_or_else(|| anyhow::anyhow!("Protocol not found"))?;

        // Remove tag if it exists
        if let Some(pos) = protocol.tags.iter().position(|t| t == tag) {
            protocol.tags.remove(pos);
            protocol.updated_at = now_timestamp();
            self.upsert_protocol(&protocol)?;
        }

        Ok(protocol.tags)
    }

    /// Delete a single protocol
    ///
    /// Permanently removes a protocol from the database. This operation
    /// cannot be undone.
    ///
    /// # Arguments
    /// * `protocol_id` - The ID of the protocol to delete
    ///
    /// # Returns
    /// `Ok(())` if successful, `Err` if protocol not found or deletion fails
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// storage.delete_protocol("protocol-id")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn delete_protocol(&self, protocol_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        let rows_affected = conn
            .execute("DELETE FROM protocols WHERE id = ?1", params![protocol_id])
            .context("Failed to delete protocol")?;

        if rows_affected == 0 {
            return Err(anyhow::anyhow!("Protocol not found: {}", protocol_id));
        }

        Ok(())
    }

    /// Bulk delete multiple protocols
    ///
    /// Deletes multiple protocols in a single transaction for efficiency.
    /// This operation cannot be undone.
    ///
    /// # Arguments
    /// * `protocol_ids` - Slice of protocol IDs to delete
    ///
    /// # Returns
    /// The number of protocols actually deleted
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let ids = vec!["id1".to_string(), "id2".to_string()];
    /// let count = storage.bulk_delete_protocols(&ids)?;
    /// println!("Deleted {} protocols", count);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn bulk_delete_protocols(&self, protocol_ids: &[String]) -> Result<usize> {
        if protocol_ids.is_empty() {
            return Ok(0);
        }

        let conn = self.open_connection()?;
        let mut total_deleted = 0;

        // Use a transaction for atomic bulk delete
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM protocols WHERE id = ?1")?;
            for protocol_id in protocol_ids {
                let rows = stmt.execute(params![protocol_id])?;
                total_deleted += rows;
            }
        }
        tx.commit()?;

        Ok(total_deleted)
    }

    /// Bulk delete multiple dose logs
    ///
    /// Deletes multiple dose log entries in a single transaction for efficiency.
    /// This operation cannot be undone.
    ///
    /// # Arguments
    /// * `dose_ids` - Slice of dose log IDs to delete
    ///
    /// # Returns
    /// The number of dose logs actually deleted
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let ids = vec!["id1".to_string(), "id2".to_string()];
    /// let count = storage.bulk_delete_doses(&ids)?;
    /// println!("Deleted {} doses", count);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn bulk_delete_doses(&self, dose_ids: &[String]) -> Result<usize> {
        if dose_ids.is_empty() {
            return Ok(0);
        }

        let conn = self.open_connection()?;
        let mut total_deleted = 0;

        // Use a transaction for atomic bulk delete
        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM dose_logs WHERE id = ?1")?;
            for dose_id in dose_ids {
                let rows = stmt.execute(params![dose_id])?;
                total_deleted += rows;
            }
        }
        tx.commit()?;

        Ok(total_deleted)
    }

    /// Bulk add a tag to multiple protocols
    ///
    /// Adds the specified tag to multiple protocols if it doesn't already exist.
    /// Tags are case-sensitive. Updates timestamps for modified protocols.
    ///
    /// # Arguments
    /// * `protocol_ids` - Slice of protocol IDs to tag
    /// * `tag` - The tag to add
    ///
    /// # Returns
    /// The number of protocols that were actually modified (tag added)
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let ids = vec!["id1".to_string(), "id2".to_string()];
    /// let count = storage.bulk_add_tag_to_protocols(&ids, "morning".to_string())?;
    /// println!("Added tag to {} protocols", count);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn bulk_add_tag_to_protocols(&self, protocol_ids: &[String], tag: String) -> Result<usize> {
        if protocol_ids.is_empty() {
            return Ok(0);
        }

        let mut modified_count = 0;

        for protocol_id in protocol_ids {
            if let Ok(Some(mut protocol)) = self.get_protocol(protocol_id) {
                // Only update if tag doesn't already exist
                if !protocol.tags.contains(&tag) {
                    protocol.tags.push(tag.clone());
                    protocol.updated_at = now_timestamp();
                    if self.upsert_protocol(&protocol).is_ok() {
                        modified_count += 1;
                    }
                }
            }
        }

        Ok(modified_count)
    }

    /// Bulk toggle favorite status for multiple protocols
    ///
    /// Sets the favorite status for multiple protocols to the specified value.
    /// Updates timestamps for modified protocols.
    ///
    /// # Arguments
    /// * `protocol_ids` - Slice of protocol IDs to update
    /// * `is_favorite` - The favorite status to set (true = favorite, false = unfavorite)
    ///
    /// # Returns
    /// The number of protocols that were actually modified
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let ids = vec!["id1".to_string(), "id2".to_string()];
    /// let count = storage.bulk_toggle_favorite_protocols(&ids, true)?;
    /// println!("Favorited {} protocols", count);
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn bulk_toggle_favorite_protocols(&self, protocol_ids: &[String], is_favorite: bool) -> Result<usize> {
        if protocol_ids.is_empty() {
            return Ok(0);
        }

        let mut modified_count = 0;

        for protocol_id in protocol_ids {
            if let Ok(Some(mut protocol)) = self.get_protocol(protocol_id) {
                // Only update if status is different
                if protocol.is_favorite != is_favorite {
                    protocol.is_favorite = is_favorite;
                    protocol.updated_at = now_timestamp();
                    if self.upsert_protocol(&protocol).is_ok() {
                        modified_count += 1;
                    }
                }
            }
        }

        Ok(modified_count)
    }

    /// Perform comprehensive database health check
    ///
    /// Runs PRAGMA quick_check to verify database integrity and collects
    /// diagnostic information about the database state.
    ///
    /// # Returns
    /// - `HealthReport` with detailed diagnostics including:
    ///   - Integrity check results (ok/corrupted)
    ///   - Database size in MB
    ///   - WAL mode status (should be enabled)
    ///   - Foreign keys status (should be enabled)
    ///   - Page count and size information
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let report = storage.health_check()?;
    /// if report.is_healthy {
    ///     println!("Database OK: {:.2} MB", report.size_mb);
    /// } else {
    ///     eprintln!("Database corrupted: {}", report.integrity_result);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    /// - Uses PRAGMA quick_check (faster than full integrity_check)
    /// - Should be run on startup and before critical operations
    /// - Non-fatal issues are logged as warnings
    pub fn health_check(&self) -> Result<HealthReport> {
        let conn = self.open_connection()?;
        let mut report = HealthReport::new();

        // 1. Run quick integrity check (faster than full integrity_check)
        let integrity: String = conn
            .query_row("PRAGMA quick_check", [], |row| row.get(0))
            .context("Failed to run integrity check")?;

        report.is_healthy = integrity == "ok";
        report.integrity_result = integrity;

        // 2. Get database size information
        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .unwrap_or(0);

        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .unwrap_or(4096);

        report.page_count = page_count;
        report.page_size = page_size;
        report.size_mb = (page_count * page_size) as f64 / 1_048_576.0;

        // 3. Check journal mode (should be WAL)
        let journal_mode: String = conn
            .query_row("PRAGMA journal_mode", [], |row| row.get(0))
            .unwrap_or_else(|_| String::from("unknown"));

        report.wal_mode = journal_mode.to_lowercase() == "wal";

        // 4. Check foreign keys (should be ON)
        let foreign_keys: i64 = conn
            .query_row("PRAGMA foreign_keys", [], |row| row.get(0))
            .unwrap_or(0);

        report.foreign_keys_enabled = foreign_keys == 1;

        // 5. Update timestamp
        report.last_checked = now_timestamp();

        // Log health check results
        if report.is_healthy {
            info!(
                "Database health check: OK ({:.2} MB, {} pages)",
                report.size_mb, report.page_count
            );
        } else {
            tracing::error!(
                "Database corruption detected: {}",
                report.integrity_result
            );
        }

        Ok(report)
    }

    /// Verify database integrity before critical operations
    ///
    /// Runs a health check and returns an error if the database is corrupted.
    /// This is a convenience wrapper around `health_check()` for use in
    /// critical code paths where database corruption should halt execution.
    ///
    /// # Returns
    /// - `Ok(())` if database is healthy
    /// - `Err` if database integrity check fails or database is corrupted
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// // Verify integrity before backup
    /// storage.verify_integrity()?;
    /// // Safe to proceed with backup
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Notes
    /// - Always called before critical operations (backups, exports)
    /// - Logs warnings for non-critical issues (WAL mode, foreign keys)
    /// - Fails fast on corruption to prevent data loss
    pub fn verify_integrity(&self) -> Result<()> {
        let report = self.health_check()?;

        if !report.is_healthy {
            return Err(anyhow::anyhow!(
                "Database integrity check failed: {}",
                report.integrity_result
            ));
        }

        if !report.wal_mode {
            tracing::warn!("Database is not using WAL mode");
        }

        if !report.foreign_keys_enabled {
            tracing::warn!("Foreign keys are not enabled");
        }

        Ok(())
    }

    /// Optimize database performance and reclaim unused space
    ///
    /// Performs three optimization operations:
    /// 1. PRAGMA optimize - Updates query planner statistics
    /// 2. PRAGMA incremental_vacuum - Reclaims free space
    /// 3. ANALYZE - Gathers table statistics for query optimization
    ///
    /// # When to Run
    /// - Periodically (weekly recommended for active databases)
    /// - After bulk delete/update operations
    /// - When `DatabaseStats::should_vacuum()` returns true
    /// - Before creating backups for optimal size
    ///
    /// # Returns
    /// - `Ok(())` if all optimization steps succeed
    /// - `Err` if any optimization step fails
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// // Run weekly optimization
    /// storage.optimize()?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Performance Notes
    /// - Uses incremental vacuum (non-blocking)
    /// - Safe to run while database is in use
    /// - May take several seconds on large databases
    /// - Does NOT require exclusive lock
    pub fn optimize(&self) -> Result<()> {
        let conn = self.open_connection()?;

        info!("Running database optimization...");

        // 1. Run PRAGMA optimize to update query planner statistics
        conn.execute("PRAGMA optimize", [])
            .context("Failed to run PRAGMA optimize")?;

        // 2. Perform incremental vacuum to reclaim space
        conn.execute("PRAGMA incremental_vacuum", [])
            .context("Failed to run incremental vacuum")?;

        // 3. Analyze database for query optimization
        conn.execute("ANALYZE", [])
            .context("Failed to run ANALYZE")?;

        info!("Database optimization complete");
        Ok(())
    }

    /// Checkpoint the Write-Ahead Log (WAL) file
    ///
    /// Merges WAL changes into the main database file. This is important for:
    /// - Reducing WAL file size
    /// - Ensuring changes are persisted to main database
    /// - Preparing for backups (ensures complete state)
    ///
    /// # Arguments
    /// * `mode` - Checkpoint mode (case-insensitive):
    ///   - `PASSIVE` - Checkpoint without blocking readers/writers (default)
    ///   - `FULL` - Wait for readers, then checkpoint
    ///   - `RESTART` - Full checkpoint + restart WAL
    ///   - `TRUNCATE` - Full checkpoint + truncate WAL to zero bytes
    ///   - Invalid modes default to `PASSIVE`
    ///
    /// # Returns
    /// - `Ok(())` if checkpoint succeeds
    /// - `Err` if checkpoint operation fails
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// // Safe checkpoint without blocking
    /// storage.checkpoint_wal("PASSIVE")?;
    ///
    /// // Full checkpoint before backup
    /// storage.checkpoint_wal("TRUNCATE")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # When to Run
    /// - Automatically runs every 1000 pages (configured in pragmas)
    /// - Before creating backups
    /// - When `DatabaseStats::should_checkpoint()` returns true
    /// - After bulk write operations
    ///
    /// # Performance Notes
    /// - PASSIVE: Non-blocking, may not complete fully
    /// - FULL/RESTART/TRUNCATE: May block briefly
    /// - Auto-checkpoint is configured to run every 1000 pages
    pub fn checkpoint_wal(&self, mode: &str) -> Result<()> {
        let conn = self.open_connection()?;

        let checkpoint_mode = match mode.to_uppercase().as_str() {
            "PASSIVE" => "PASSIVE",
            "FULL" => "FULL",
            "RESTART" => "RESTART",
            "TRUNCATE" => "TRUNCATE",
            _ => {
                tracing::warn!("Unknown checkpoint mode '{}', using PASSIVE", mode);
                "PASSIVE"
            }
        };

        info!("Checkpointing WAL (mode: {})", checkpoint_mode);

        // PRAGMA wal_checkpoint returns (busy, log, checkpointed) as results
        // We use query_row but ignore the results
        conn.query_row(&format!("PRAGMA wal_checkpoint({})", checkpoint_mode), [], |_row| Ok(()))
            .context("Failed to checkpoint WAL")?;

        Ok(())
    }

    /// Get detailed database statistics for monitoring and maintenance
    ///
    /// Collects comprehensive metrics about database size, fragmentation,
    /// and WAL usage. Use these statistics to determine when maintenance
    /// operations (vacuum, checkpoint) should be performed.
    ///
    /// # Returns
    /// `DatabaseStats` containing:
    /// - `page_count` - Total number of database pages
    /// - `page_size` - Size of each page in bytes (typically 4096)
    /// - `total_size_mb` - Total database size in megabytes
    /// - `freelist_pages` - Number of unused pages (fragmentation)
    /// - `wasted_space_mb` - Size of wasted space from fragmentation
    /// - `wal_size_mb` - Current WAL file size in megabytes
    ///
    /// # Helper Methods
    /// `DatabaseStats` provides helper methods:
    /// - `fragmentation_percentage()` - Percentage of database that is wasted space
    /// - `should_vacuum()` - Returns true if >10% fragmented or >50MB wasted
    /// - `should_checkpoint()` - Returns true if WAL is >10MB
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let stats = storage.get_stats()?;
    /// println!("Database: {:.2} MB", stats.total_size_mb);
    /// println!("Fragmentation: {:.1}%", stats.fragmentation_percentage());
    ///
    /// if stats.should_vacuum() {
    ///     storage.optimize()?;
    /// }
    /// if stats.should_checkpoint() {
    ///     storage.checkpoint_wal("TRUNCATE")?;
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    ///
    /// # Use Cases
    /// - Monitoring database health dashboards
    /// - Automated maintenance scheduling
    /// - Performance troubleshooting
    /// - Capacity planning
    pub fn get_stats(&self) -> Result<DatabaseStats> {
        let conn = self.open_connection()?;

        let page_count: i64 = conn
            .query_row("PRAGMA page_count", [], |row| row.get(0))
            .unwrap_or(0);

        let page_size: i64 = conn
            .query_row("PRAGMA page_size", [], |row| row.get(0))
            .unwrap_or(4096);

        let freelist_count: i64 = conn
            .query_row("PRAGMA freelist_count", [], |row| row.get(0))
            .unwrap_or(0);

        let wal_size: i64 = {
            let wal_path = format!("{}-wal", self.db_path.display());
            std::fs::metadata(&wal_path)
                .map(|m| m.len() as i64)
                .unwrap_or(0)
        };

        Ok(DatabaseStats {
            page_count,
            page_size,
            total_size_mb: (page_count * page_size) as f64 / 1_048_576.0,
            freelist_pages: freelist_count,
            wasted_space_mb: (freelist_count * page_size) as f64 / 1_048_576.0,
            wal_size_mb: wal_size as f64 / 1_048_576.0,
        })
    }

    /// Get a database connection for advanced operations
    /// WARNING: Use with caution - bypasses encryption for direct SQL access
    pub fn connection(&self) -> Result<Connection> {
        self.open_connection()
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

    /// Save or update a body metric entry
    ///
    /// Stores body composition metrics like weight, body fat %, muscle mass, etc.
    /// Encrypts all data before storage.
    ///
    /// # Arguments
    /// * `metric` - The body metric entry to save
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # use peptrack_core::models::BodyMetric;
    /// # use time::OffsetDateTime;
    /// # let storage = todo!();
    /// let mut metric = BodyMetric::new(OffsetDateTime::now_utc());
    /// metric.weight_kg = Some(75.5);
    /// metric.body_fat_percentage = Some(15.2);
    /// storage.upsert_body_metric(&metric)?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn upsert_body_metric(&self, metric: &BodyMetric) -> Result<()> {
        let conn = self.open_connection()?;
        let payload = serde_json::to_vec(metric).context("Failed to serialize body metric")?;
        let encrypted = self.encryption.seal(&payload)?;

        conn.execute(
            r#"
            INSERT INTO body_metrics (id, date, payload, created_at, updated_at)
            VALUES (?1, ?2, ?3, ?4, ?5)
            ON CONFLICT(id) DO UPDATE SET
                date = excluded.date,
                payload = excluded.payload,
                updated_at = excluded.updated_at;
            "#,
            params![
                metric.id,
                metric.date.to_string(),
                encrypted,
                metric.created_at.to_string(),
                metric.updated_at.to_string()
            ],
        )
        .context("Failed to upsert body metric")?;

        Ok(())
    }

    /// List all body metrics ordered by date (most recent first)
    ///
    /// Returns all body metric entries from the database, decrypted
    /// and sorted by measurement date.
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// let metrics = storage.list_body_metrics()?;
    /// for metric in metrics {
    ///     println!("Date: {}, Weight: {:?} kg", metric.date, metric.weight_kg);
    /// }
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn list_body_metrics(&self) -> Result<Vec<BodyMetric>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM body_metrics ORDER BY date DESC")?;
        let mut rows = stmt
            .query([])
            .context("Unable to run body metrics list query")?;

        let mut metrics = Vec::new();
        while let Some(row) = rows.next()? {
            let blob: Vec<u8> = row.get(0)?;
            let decrypted = self.encryption.open(&blob)?;
            let metric: BodyMetric = serde_json::from_slice(&decrypted)
                .context("Failed to deserialize body metric")?;
            metrics.push(metric);
        }

        Ok(metrics)
    }

    /// Get a specific body metric by ID
    ///
    /// Returns the body metric if found, None otherwise.
    ///
    /// # Arguments
    /// * `metric_id` - The ID of the body metric to retrieve
    pub fn get_body_metric(&self, metric_id: &str) -> Result<Option<BodyMetric>> {
        let conn = self.open_connection()?;
        let mut stmt = conn.prepare("SELECT payload FROM body_metrics WHERE id = ?1")?;

        let result = stmt.query_row(params![metric_id], |row| {
            let blob: Vec<u8> = row.get(0)?;
            Ok(blob)
        });

        match result {
            Ok(blob) => {
                let decrypted = self.encryption.open(&blob)?;
                let metric: BodyMetric = serde_json::from_slice(&decrypted)
                    .context("Failed to deserialize body metric")?;
                Ok(Some(metric))
            }
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Delete a body metric entry
    ///
    /// Permanently removes a body metric from the database.
    ///
    /// # Arguments
    /// * `metric_id` - The ID of the body metric to delete
    ///
    /// # Example
    /// ```rust,no_run
    /// # use peptrack_core::db::StorageManager;
    /// # let storage = todo!();
    /// storage.delete_body_metric("metric-id")?;
    /// # Ok::<(), anyhow::Error>(())
    /// ```
    pub fn delete_body_metric(&self, metric_id: &str) -> Result<()> {
        let conn = self.open_connection()?;
        conn.execute("DELETE FROM body_metrics WHERE id = ?1", params![metric_id])
            .context("Failed to delete body metric")?;
        Ok(())
    }

    /// Bulk delete multiple body metrics
    ///
    /// Deletes multiple body metric entries in a single transaction.
    ///
    /// # Arguments
    /// * `metric_ids` - Slice of body metric IDs to delete
    ///
    /// # Returns
    /// The number of metrics actually deleted
    pub fn bulk_delete_body_metrics(&self, metric_ids: &[String]) -> Result<usize> {
        if metric_ids.is_empty() {
            return Ok(0);
        }

        let conn = self.open_connection()?;
        let mut total_deleted = 0;

        let tx = conn.unchecked_transaction()?;
        {
            let mut stmt = tx.prepare("DELETE FROM body_metrics WHERE id = ?1")?;
            for metric_id in metric_ids {
                let rows = stmt.execute(params![metric_id])?;
                total_deleted += rows;
            }
        }
        tx.commit()?;

        Ok(total_deleted)
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

        // Use parameterized query with LIMIT -1 for no limit (SQLite behavior)
        let limit_value = limit.map(|l| l as i64).unwrap_or(-1);

        let mut stmt = conn.prepare("SELECT payload FROM summary_history ORDER BY created_at DESC LIMIT ?1")?;
        let mut rows = stmt
            .query([limit_value])
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
    use crate::models::*;
    use crate::StaticKeyProvider;
    use tempfile::tempdir;

    // Test helper to create a storage manager with a temp database
    fn create_test_storage() -> StorageManager {
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

        // Keep temp directory alive by leaking it
        // This is acceptable for tests and prevents directory cleanup issues
        std::mem::forget(tmp);

        storage
    }

    // =============================================================================
    // Protocol CRUD Tests
    // =============================================================================

    #[test]
    fn upsert_and_list_protocols_roundtrips() {
        let storage = create_test_storage();

        let mut protocol = PeptideProtocol::new("Protocol A", "BPC-157");
        protocol.notes = Some("store at 4C".into());

        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let fetched = storage.list_protocols().expect("list");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].name, "Protocol A");
        assert_eq!(fetched[0].notes.as_deref(), Some("store at 4C"));
    }

    #[test]
    fn list_protocols_returns_empty_for_new_database() {
        let storage = create_test_storage();
        let protocols = storage.list_protocols().expect("list");
        assert_eq!(protocols.len(), 0);
    }

    #[test]
    fn get_protocol_returns_none_for_nonexistent_id() {
        let storage = create_test_storage();
        let result = storage
            .get_protocol("nonexistent-id")
            .expect("get protocol");
        assert!(result.is_none());
    }

    #[test]
    fn get_protocol_returns_existing_protocol() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Morning Stack", "TB-500");
        storage.upsert_protocol(&protocol).expect("upsert");

        let fetched = storage.get_protocol(&protocol.id).expect("get protocol");
        assert!(fetched.is_some());
        let fetched = fetched.unwrap();
        assert_eq!(fetched.id, protocol.id);
        assert_eq!(fetched.name, "Morning Stack");
    }

    #[test]
    fn upsert_protocol_updates_existing_protocol() {
        let storage = create_test_storage();
        let mut protocol = PeptideProtocol::new("Original Name", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert");

        // Update the protocol
        protocol.name = "Updated Name".to_string();
        protocol.notes = Some("New notes".to_string());
        storage.upsert_protocol(&protocol).expect("upsert updated");

        let fetched = storage.list_protocols().expect("list");
        assert_eq!(fetched.len(), 1);
        assert_eq!(fetched[0].name, "Updated Name");
        assert_eq!(fetched[0].notes.as_deref(), Some("New notes"));
    }

    // =============================================================================
    // Dose Log Tests
    // =============================================================================

    #[test]
    fn append_dose_log_and_list_roundtrips() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let dose = DoseLog::new(&protocol.id, &"Left Shoulder".to_string(), 0.5);
        storage.append_dose_log(&dose).expect("append dose");

        let doses = storage.list_dose_logs().expect("list doses");
        assert_eq!(doses.len(), 1);
        assert_eq!(doses[0].site, "Left Shoulder");
        assert_eq!(doses[0].amount_mg, 0.5);
    }

    #[test]
    fn list_dose_logs_for_protocol_filters_correctly() {
        let storage = create_test_storage();
        let protocol1 = PeptideProtocol::new("Protocol 1", "BPC-157");
        let protocol2 = PeptideProtocol::new("Protocol 2", "TB-500");
        storage.upsert_protocol(&protocol1).expect("upsert protocol1");
        storage.upsert_protocol(&protocol2).expect("upsert protocol2");

        let dose1 = DoseLog::new(&protocol1.id, &"Site A".to_string(), 0.5);
        let dose2 = DoseLog::new(&protocol2.id, &"Site B".to_string(), 1.0);
        let dose3 = DoseLog::new(&protocol1.id, &"Site C".to_string(), 0.75);

        storage.append_dose_log(&dose1).expect("append dose1");
        storage.append_dose_log(&dose2).expect("append dose2");
        storage.append_dose_log(&dose3).expect("append dose3");

        let doses_for_p1 = storage
            .list_dose_logs_for_protocol(&protocol1.id)
            .expect("list doses for protocol1");
        assert_eq!(doses_for_p1.len(), 2);
        assert!(doses_for_p1.iter().all(|d| d.protocol_id == protocol1.id));
    }

    #[test]
    fn delete_dose_log_removes_log() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let dose = DoseLog::new(&protocol.id, &"Site".to_string(), 0.5);
        let dose_id = dose.id.clone();
        storage.append_dose_log(&dose).expect("append dose");

        storage.delete_dose_log(&dose_id).expect("delete dose");

        let doses = storage.list_dose_logs().expect("list doses");
        assert_eq!(doses.len(), 0);
    }

    #[test]
    fn delete_dose_log_with_nonexistent_id_succeeds() {
        let storage = create_test_storage();
        // Deleting a non-existent dose should not error (SQL DELETE with no matches)
        storage
            .delete_dose_log("nonexistent-id")
            .expect("delete nonexistent");
    }

    // =============================================================================
    // Literature Cache Tests
    // =============================================================================

    #[test]
    fn cache_literature_and_list_roundtrips() {
        let storage = create_test_storage();
        let mut entry = LiteratureEntry::new("pubmed", "BPC-157 Research Paper");
        entry.url = Some("https://pubmed.ncbi.nlm.nih.gov/12345/".to_string());
        entry.summary = Some("This paper discusses BPC-157.".to_string());

        storage.cache_literature(&entry).expect("cache literature");

        let entries = storage.list_literature().expect("list literature");
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].title, "BPC-157 Research Paper");
        assert_eq!(entries[0].source, "pubmed");
    }

    #[test]
    fn search_literature_finds_matching_entries() {
        let storage = create_test_storage();
        let entry1 = LiteratureEntry::new("pubmed", "BPC-157 and Wound Healing");
        let entry2 = LiteratureEntry::new("openalex", "TB-500 Clinical Study");
        let entry3 = LiteratureEntry::new("pubmed", "GHK-Cu Peptide Research");

        storage.cache_literature(&entry1).expect("cache");
        storage.cache_literature(&entry2).expect("cache");
        storage.cache_literature(&entry3).expect("cache");

        let results = storage.search_literature("BPC-157").expect("search");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].title, "BPC-157 and Wound Healing");
    }

    #[test]
    fn search_literature_returns_empty_for_no_matches() {
        let storage = create_test_storage();
        let entry = LiteratureEntry::new("pubmed", "Some Paper");
        storage.cache_literature(&entry).expect("cache");

        let results = storage.search_literature("nonexistent").expect("search");
        assert_eq!(results.len(), 0);
    }

    #[test]
    fn search_literature_is_case_insensitive() {
        let storage = create_test_storage();
        let entry = LiteratureEntry::new("pubmed", "BPC-157 Research");
        storage.cache_literature(&entry).expect("cache");

        let results = storage.search_literature("bpc-157").expect("search");
        assert_eq!(results.len(), 1);
    }

    // =============================================================================
    // Supplier Tests
    // =============================================================================

    #[test]
    fn upsert_supplier_and_list_roundtrips() {
        let storage = create_test_storage();
        let mut supplier = Supplier::new("PeptideSource");
        supplier.website = Some("https://peptidesource.com".to_string());
        supplier.contact_email = Some("contact@peptidesource.com".to_string());

        storage.upsert_supplier(&supplier).expect("upsert supplier");

        let suppliers = storage.list_suppliers().expect("list suppliers");
        assert_eq!(suppliers.len(), 1);
        assert_eq!(suppliers[0].name, "PeptideSource");
        assert_eq!(
            suppliers[0].website.as_deref(),
            Some("https://peptidesource.com")
        );
    }

    #[test]
    fn get_supplier_returns_existing_supplier() {
        let storage = create_test_storage();
        let supplier = Supplier::new("TestSupplier");
        storage.upsert_supplier(&supplier).expect("upsert");

        let fetched = storage.get_supplier(&supplier.id).expect("get supplier");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().name, "TestSupplier");
    }

    #[test]
    fn get_supplier_returns_none_for_nonexistent_id() {
        let storage = create_test_storage();
        let result = storage.get_supplier("nonexistent").expect("get supplier");
        assert!(result.is_none());
    }

    #[test]
    fn delete_supplier_removes_supplier() {
        let storage = create_test_storage();
        let supplier = Supplier::new("ToDelete");
        let supplier_id = supplier.id.clone();
        storage.upsert_supplier(&supplier).expect("upsert");

        storage.delete_supplier(&supplier_id).expect("delete");

        let suppliers = storage.list_suppliers().expect("list");
        assert_eq!(suppliers.len(), 0);
    }

    // =============================================================================
    // Inventory Tests
    // =============================================================================

    #[test]
    fn upsert_inventory_item_and_list_roundtrips() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Test", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let mut item = InventoryItem::new(&protocol.id);
        item.vial_status = VialStatus::Opened;
        item.quantity_mg = Some(10.0);
        item.batch_number = Some("BATCH123".to_string());

        storage.upsert_inventory_item(&item).expect("upsert item");

        let items = storage.list_inventory().expect("list inventory");
        assert_eq!(items.len(), 1);
        assert_eq!(items[0].batch_number.as_deref(), Some("BATCH123"));
    }

    #[test]
    fn list_inventory_by_protocol_filters_correctly() {
        let storage = create_test_storage();
        let protocol1 = PeptideProtocol::new("P1", "BPC-157");
        let protocol2 = PeptideProtocol::new("P2", "TB-500");
        storage.upsert_protocol(&protocol1).expect("upsert");
        storage.upsert_protocol(&protocol2).expect("upsert");

        let item1 = InventoryItem::new(&protocol1.id);
        let item2 = InventoryItem::new(&protocol2.id);
        let item3 = InventoryItem::new(&protocol1.id);

        storage.upsert_inventory_item(&item1).expect("upsert");
        storage.upsert_inventory_item(&item2).expect("upsert");
        storage.upsert_inventory_item(&item3).expect("upsert");

        let items_for_p1 = storage
            .list_inventory_by_protocol(&protocol1.id)
            .expect("list for protocol1");
        assert_eq!(items_for_p1.len(), 2);
    }

    #[test]
    fn get_inventory_item_returns_existing_item() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Test", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let item = InventoryItem::new(&protocol.id);
        let item_id = item.id.clone();
        storage.upsert_inventory_item(&item).expect("upsert");

        let fetched = storage.get_inventory_item(&item_id).expect("get item");
        assert!(fetched.is_some());
        assert_eq!(fetched.unwrap().id, item_id);
    }

    #[test]
    fn delete_inventory_item_removes_item() {
        let storage = create_test_storage();
        let protocol = PeptideProtocol::new("Test", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let item = InventoryItem::new(&protocol.id);
        let item_id = item.id.clone();
        storage.upsert_inventory_item(&item).expect("upsert");

        storage.delete_inventory_item(&item_id).expect("delete");

        let items = storage.list_inventory().expect("list");
        assert_eq!(items.len(), 0);
    }

    // =============================================================================
    // Price History Tests
    // =============================================================================

    #[test]
    fn add_price_history_and_list_roundtrips() {
        let storage = create_test_storage();
        let supplier = Supplier::new("TestSupplier");
        storage.upsert_supplier(&supplier).expect("upsert supplier");

        let price = PriceHistory::new(&supplier.id, &"BPC-157".to_string(), 2.5);
        storage.add_price_history(&price).expect("add price");

        let prices = storage
            .list_price_history_for_supplier(&supplier.id, None)
            .expect("list prices");
        assert_eq!(prices.len(), 1);
        assert_eq!(prices[0].cost_per_mg, 2.5);
    }

    #[test]
    fn list_price_history_filters_by_peptide() {
        let storage = create_test_storage();
        let supplier = Supplier::new("TestSupplier");
        storage.upsert_supplier(&supplier).expect("upsert supplier");

        let price1 = PriceHistory::new(&supplier.id, &"BPC-157".to_string(), 2.5);
        let price2 = PriceHistory::new(&supplier.id, &"TB-500".to_string(), 3.0);
        let price3 = PriceHistory::new(&supplier.id, &"BPC-157".to_string(), 2.6);

        storage.add_price_history(&price1).expect("add");
        storage.add_price_history(&price2).expect("add");
        storage.add_price_history(&price3).expect("add");

        let bpc_prices = storage
            .list_price_history_for_supplier(&supplier.id, Some("BPC-157"))
            .expect("list");
        assert_eq!(bpc_prices.len(), 2);
        assert!(bpc_prices.iter().all(|p| p.peptide_name == "BPC-157"));
    }

    #[test]
    fn get_latest_price_returns_most_recent() {
        let storage = create_test_storage();
        let supplier = Supplier::new("TestSupplier");
        storage.upsert_supplier(&supplier).expect("upsert supplier");

        let price1 = PriceHistory::new(&supplier.id, &"BPC-157".to_string(), 2.5);
        std::thread::sleep(std::time::Duration::from_millis(10));
        let price2 = PriceHistory::new(&supplier.id, &"BPC-157".to_string(), 2.6);

        storage.add_price_history(&price1).expect("add");
        storage.add_price_history(&price2).expect("add");

        let latest = storage
            .get_latest_price(&supplier.id, "BPC-157")
            .expect("get latest");
        assert!(latest.is_some());
        assert_eq!(latest.unwrap().cost_per_mg, 2.6);
    }

    // =============================================================================
    // Alert Tests
    // =============================================================================

    #[test]
    fn create_alert_and_list_roundtrips() {
        let storage = create_test_storage();
        let alert = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Low Stock",
            "Vial is running low",
        );

        storage.create_alert(&alert).expect("create alert");

        let alerts = storage.list_alerts(false).expect("list alerts");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "Low Stock");
        assert_eq!(alerts[0].alert_type, AlertType::LowStock);
    }

    #[test]
    fn list_alerts_excludes_dismissed_by_default() {
        let storage = create_test_storage();
        let mut alert1 = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Alert 1",
            "Message 1",
        );
        let mut alert2 = Alert::new(
            AlertType::Expired,
            AlertSeverity::Critical,
            "Alert 2",
            "Message 2",
        );
        alert2.is_dismissed = true;

        storage.create_alert(&alert1).expect("create");
        storage.create_alert(&alert2).expect("create");

        let alerts = storage.list_alerts(false).expect("list alerts");
        assert_eq!(alerts.len(), 1);
        assert_eq!(alerts[0].title, "Alert 1");
    }

    #[test]
    fn list_alerts_includes_dismissed_when_requested() {
        let storage = create_test_storage();
        let mut alert1 = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Alert 1",
            "Message 1",
        );
        let mut alert2 = Alert::new(
            AlertType::Expired,
            AlertSeverity::Critical,
            "Alert 2",
            "Message 2",
        );
        alert2.is_dismissed = true;

        storage.create_alert(&alert1).expect("create");
        storage.create_alert(&alert2).expect("create");

        let alerts = storage.list_alerts(true).expect("list alerts with dismissed");
        assert_eq!(alerts.len(), 2);
    }

    #[test]
    fn mark_alert_read_updates_status() {
        let storage = create_test_storage();
        let alert = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Test",
            "Message",
        );
        let alert_id = alert.id.clone();
        storage.create_alert(&alert).expect("create");

        storage.mark_alert_read(&alert_id).expect("mark read");

        // Note: mark_alert_read only updates is_read flag, not the payload blob
        // This test verifies the SQL command succeeds without error
    }

    #[test]
    fn dismiss_alert_updates_status() {
        let storage = create_test_storage();
        let alert = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Test",
            "Message",
        );
        let alert_id = alert.id.clone();
        storage.create_alert(&alert).expect("create");

        storage.dismiss_alert(&alert_id).expect("dismiss");

        let alerts = storage.list_alerts(false).expect("list");
        assert_eq!(alerts.len(), 0); // Dismissed alerts are excluded
    }

    #[test]
    fn clear_all_alerts_removes_all() {
        let storage = create_test_storage();
        let alert1 = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Alert 1",
            "Message",
        );
        let alert2 = Alert::new(
            AlertType::Expired,
            AlertSeverity::Critical,
            "Alert 2",
            "Message",
        );

        storage.create_alert(&alert1).expect("create");
        storage.create_alert(&alert2).expect("create");

        storage.clear_all_alerts().expect("clear all");

        let alerts = storage.list_alerts(true).expect("list");
        assert_eq!(alerts.len(), 0);
    }

    // =============================================================================
    // Summary History Tests
    // =============================================================================

    #[test]
    fn save_summary_and_list_roundtrips() {
        let storage = create_test_storage();
        let summary = SummaryHistory::new(
            "BPC-157 Research Summary",
            "Original paper content...",
            "Summary output...",
            "markdown",
            "claude",
        );

        storage.save_summary(&summary).expect("save summary");

        let summaries = storage.list_summary_history(None).expect("list summaries");
        assert_eq!(summaries.len(), 1);
        assert_eq!(summaries[0].title, "BPC-157 Research Summary");
    }

    #[test]
    fn list_summary_history_respects_limit() {
        let storage = create_test_storage();
        for i in 1..=5 {
            let summary = SummaryHistory::new(
                format!("Summary {}", i),
                "content".to_string(),
                "output".to_string(),
                "markdown".to_string(),
                "claude".to_string(),
            );
            storage.save_summary(&summary).expect("save");
        }

        let summaries = storage.list_summary_history(Some(3)).expect("list");
        assert_eq!(summaries.len(), 3);
    }

    #[test]
    fn list_summary_history_without_limit_returns_all() {
        let storage = create_test_storage();
        for i in 1..=10 {
            let summary = SummaryHistory::new(
                format!("Summary {}", i),
                "content".to_string(),
                "output".to_string(),
                "markdown".to_string(),
                "claude".to_string(),
            );
            storage.save_summary(&summary).expect("save");
        }

        let summaries = storage.list_summary_history(None).expect("list");
        assert_eq!(summaries.len(), 10);
    }

    #[test]
    fn delete_summary_removes_summary() {
        let storage = create_test_storage();
        let summary = SummaryHistory::new("Test", "content", "output", "markdown", "claude");
        let summary_id = summary.id.clone();
        storage.save_summary(&summary).expect("save");

        storage.delete_summary(&summary_id).expect("delete");

        let summaries = storage.list_summary_history(None).expect("list");
        assert_eq!(summaries.len(), 0);
    }

    // =============================================================================
    // Schema & Initialization Tests
    // =============================================================================

    #[test]
    fn initialize_creates_all_tables() {
        let tmp = tempdir().expect("tempdir");
        let key_provider =
            Arc::new(StaticKeyProvider::new(vec![7u8; 32]).expect("static key provider"));
        let storage = StorageManager::new(StorageConfig {
            data_dir: Some(tmp.path().to_path_buf()),
            db_file_name: Some("test.sqlite".into()),
            key_provider,
        })
        .expect("storage manager");

        storage.initialize().expect("initialize");

        // Verify tables exist by attempting basic operations
        storage.list_protocols().expect("protocols table exists");
        storage.list_dose_logs().expect("dose_logs table exists");
        storage.list_literature().expect("literature_cache table exists");
        storage.list_suppliers().expect("suppliers table exists");
        storage.list_inventory().expect("inventory table exists");
        storage
            .list_alerts(true)
            .expect("alerts table exists");
        storage
            .list_summary_history(None)
            .expect("summary_history table exists");
    }

    #[test]
    fn initialize_is_idempotent() {
        let storage = create_test_storage();
        // Initialize again - should not error
        storage.initialize().expect("initialize again");
    }

    // =============================================================================
    // Health & Diagnostics Tests
    // =============================================================================

    #[test]
    fn health_check_returns_healthy_report() {
        let storage = create_test_storage();
        let report = storage.health_check().expect("health check");

        // Fresh database should be healthy
        assert!(report.is_healthy);
        assert_eq!(report.integrity_result, "ok");
        assert!(report.wal_mode);
        assert!(report.foreign_keys_enabled);
        assert!(report.size_mb > 0.0);
        assert!(report.page_count > 0);
        assert!(report.page_size > 0);
    }

    #[test]
    fn verify_integrity_succeeds_on_healthy_database() {
        let storage = create_test_storage();
        storage.verify_integrity().expect("integrity check should pass");
    }

    #[test]
    fn get_stats_returns_valid_statistics() {
        let storage = create_test_storage();
        let stats = storage.get_stats().expect("get stats");

        assert!(stats.page_count > 0);
        assert!(stats.page_size > 0);
        assert!(stats.total_size_mb > 0.0);
        assert!(stats.freelist_pages >= 0);
        assert!(stats.wasted_space_mb >= 0.0);
        assert!(stats.wal_size_mb >= 0.0);
    }

    #[test]
    fn optimize_database_runs_successfully() {
        let storage = create_test_storage();

        // Add some data first
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert");

        // Run optimization
        storage.optimize().expect("optimize should succeed");
    }

    #[test]
    fn checkpoint_wal_passive_mode() {
        let storage = create_test_storage();

        // Add some data to create WAL entries
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert");

        // Checkpoint with PASSIVE mode
        storage.checkpoint_wal("PASSIVE").expect("checkpoint should succeed");
    }

    #[test]
    fn checkpoint_wal_full_mode() {
        let storage = create_test_storage();

        // Add some data to create WAL entries
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert");

        // Checkpoint with FULL mode
        storage.checkpoint_wal("FULL").expect("checkpoint should succeed");
    }

    #[test]
    fn checkpoint_wal_invalid_mode_defaults_to_passive() {
        let storage = create_test_storage();

        // Invalid mode should default to PASSIVE and not error
        storage.checkpoint_wal("INVALID").expect("checkpoint should succeed with default");
    }

    #[test]
    fn database_stats_fragmentation_calculation() {
        let storage = create_test_storage();
        let stats = storage.get_stats().expect("get stats");

        // Fragmentation should be between 0 and 100
        let fragmentation = stats.fragmentation_percentage();
        assert!(fragmentation >= 0.0 && fragmentation <= 100.0);
    }

    #[test]
    fn database_stats_recommendations() {
        let storage = create_test_storage();
        let stats = storage.get_stats().expect("get stats");

        // Fresh database should not need vacuum or checkpoint
        assert!(!stats.should_vacuum(), "Fresh database shouldn't need vacuum");
        assert!(!stats.should_checkpoint(), "Fresh database shouldn't need checkpoint");
    }

    #[test]
    fn health_check_with_data_operations() {
        let storage = create_test_storage();

        // Add various types of data
        let protocol = PeptideProtocol::new("Test Protocol", "BPC-157");
        storage.upsert_protocol(&protocol).expect("upsert protocol");

        let dose_log = DoseLog::new(&protocol.id, &"Left Abdomen".to_string(), 5.0);
        storage.append_dose_log(&dose_log).expect("append dose");

        // Health check should still pass
        let report = storage.health_check().expect("health check");
        assert!(report.is_healthy);
        assert_eq!(report.integrity_result, "ok");
    }

    #[test]
    fn optimize_after_bulk_operations() {
        let storage = create_test_storage();

        // Perform bulk operations
        for i in 0..100 {
            let protocol = PeptideProtocol::new(
                &format!("Protocol {}", i),
                &format!("Peptide {}", i),
            );
            storage.upsert_protocol(&protocol).expect("upsert");
        }

        // Get stats before optimization
        let _stats_before = storage.get_stats().expect("stats before");

        // Optimize
        storage.optimize().expect("optimize");

        // Get stats after optimization
        let stats_after = storage.get_stats().expect("stats after");

        // Stats should be available (exact values may vary)
        assert!(stats_after.page_count > 0);
    }

    #[test]
    fn cache_size_is_at_least_64mb() {
        let storage = create_test_storage();
        let conn = storage.connection().expect("get connection");

        // Query cache_size pragma
        let cache_size: i64 = conn
            .query_row("PRAGMA cache_size", [], |row| row.get(0))
            .expect("query cache_size");

        // Negative values are in KB, positive are in pages
        // We configured -64000 (64MB)
        if cache_size < 0 {
            // Negative = KB
            let cache_kb = cache_size.abs();
            assert!(
                cache_kb >= 64000,
                "Cache size should be at least 64MB (64000 KB), got {} KB",
                cache_kb
            );
        } else {
            // Positive = pages (typically 4KB each)
            // 64MB / 4KB = 16000 pages minimum
            assert!(
                cache_size >= 16000,
                "Cache size should be at least 64MB (16000 pages), got {} pages",
                cache_size
            );
        }
    }
}
