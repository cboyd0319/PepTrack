//! PepTrack Core - Encrypted local storage for peptide protocol management
//!
//! This crate provides secure, encrypted storage of peptide protocols, dose logs,
//! and literature entries using ChaCha20-Poly1305 envelope encryption and SQLite.
//!
//! # Features
//!
//! - **Envelope Encryption**: Each record is encrypted with a unique nonce before storage
//! - **Key Management**: Flexible key provider abstraction for different key sources
//! - **SQLite Storage**: Efficient local database with WAL mode and foreign key support
//! - **Type-Safe Models**: Strongly-typed domain models with UUID-based identification
//!
//! # Examples
//!
//! ```no_run
//! use std::sync::Arc;
//! use peptrack_core::{StaticKeyProvider, StorageConfig, StorageManager, PeptideProtocol};
//!
//! # fn main() -> anyhow::Result<()> {
//! // Create a key provider with a 32-byte key
//! let key_provider = Arc::new(StaticKeyProvider::new(vec![42u8; 32])?);
//!
//! // Configure storage
//! let storage = StorageManager::new(StorageConfig {
//!     data_dir: None, // Uses OS default
//!     db_file_name: None, // Uses "peptrack.sqlite"
//!     key_provider,
//! })?;
//!
//! // Initialize database schema
//! storage.initialize()?;
//!
//! // Create and save a protocol
//! let mut protocol = PeptideProtocol::new("Morning Protocol", "BPC-157");
//! protocol.notes = Some("Take with food".to_string());
//! storage.upsert_protocol(&protocol)?;
//!
//! // List all protocols
//! let protocols = storage.list_protocols()?;
//! # Ok(())
//! # }
//! ```

pub mod backup_encryption;
pub mod db;
pub mod encryption;
pub mod keychain;
pub mod models;

pub use backup_encryption::{decrypt_backup, encrypt_backup, is_encrypted_backup};
pub use db::{StorageConfig, StorageManager};
pub use encryption::{EnvelopeEncryption, KeyMaterial, KeyProvider, StaticKeyProvider};
pub use keychain::{migrate_file_key_to_keychain, KeychainKeyProvider};
pub use models::{BodyMetric, DoseLog, InventoryItem, LiteratureEntry, PeptideProtocol, SideEffect, Supplier, VialStatus};
