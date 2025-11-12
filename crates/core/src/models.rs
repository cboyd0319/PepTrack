use serde::{Deserialize, Serialize};
use time::OffsetDateTime;
use uuid::Uuid;

use crate::db::now_timestamp;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeptideProtocol {
    pub id: String,
    pub name: String,
    pub peptide_name: String,
    pub notes: Option<String>,
    pub current_vial_status: Option<String>,
    pub target_concentration_mg_ml: Option<f32>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl PeptideProtocol {
    pub fn new<S: Into<String>>(name: S, peptide_name: S) -> Self {
        let now = now_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            peptide_name: peptide_name.into(),
            notes: None,
            current_vial_status: None,
            target_concentration_mg_ml: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoseLog {
    pub id: String,
    pub protocol_id: String,
    pub site: String,
    pub amount_mg: f32,
    pub notes: Option<String>,
    pub logged_at: OffsetDateTime,
}

impl DoseLog {
    pub fn new<S: Into<String>>(protocol_id: S, site: S, amount_mg: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            protocol_id: protocol_id.into(),
            site: site.into(),
            amount_mg,
            notes: None,
            logged_at: now_timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LiteratureEntry {
    pub id: String,
    pub source: String,
    pub title: String,
    pub url: Option<String>,
    pub summary: Option<String>,
    pub relevance_score: Option<f32>,
    pub indexed_at: OffsetDateTime,
}

impl LiteratureEntry {
    pub fn new<S: Into<String>>(source: S, title: S) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            source: source.into(),
            title: title.into(),
            url: None,
            summary: None,
            relevance_score: None,
            indexed_at: now_timestamp(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Supplier {
    pub id: String,
    pub name: String,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl Supplier {
    pub fn new<S: Into<String>>(name: S) -> Self {
        let now = now_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.into(),
            contact_email: None,
            contact_phone: None,
            website: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum VialStatus {
    Sealed,
    Opened,
    Empty,
    Expired,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InventoryItem {
    pub id: String,
    pub protocol_id: String,
    pub supplier_id: Option<String>,
    pub vial_number: Option<String>,
    pub vial_status: VialStatus,
    pub purchase_date: Option<OffsetDateTime>,
    pub expiry_date: Option<OffsetDateTime>,
    pub cost_per_mg: Option<f32>,
    pub quantity_mg: Option<f32>,
    pub quantity_remaining_mg: Option<f32>, // NEW: Track remaining quantity
    pub concentration_mg_ml: Option<f32>,
    pub batch_number: Option<String>,
    pub lot_number: Option<String>,
    pub low_stock_threshold_mg: Option<f32>, // NEW: Alert when below this
    pub notes: Option<String>,
    pub created_at: OffsetDateTime,
    pub updated_at: OffsetDateTime,
}

impl InventoryItem {
    pub fn new<S: Into<String>>(protocol_id: S) -> Self {
        let now = now_timestamp();
        Self {
            id: Uuid::new_v4().to_string(),
            protocol_id: protocol_id.into(),
            supplier_id: None,
            vial_number: None,
            vial_status: VialStatus::Sealed,
            purchase_date: None,
            expiry_date: None,
            cost_per_mg: None,
            quantity_mg: None,
            quantity_remaining_mg: None,
            concentration_mg_ml: None,
            batch_number: None,
            lot_number: None,
            low_stock_threshold_mg: None,
            notes: None,
            created_at: now,
            updated_at: now,
        }
    }
}

/// Price History Entry
/// Tracks price changes for peptides from suppliers over time
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PriceHistory {
    pub id: String,
    pub supplier_id: String,
    pub peptide_name: String,
    pub cost_per_mg: f32,
    pub url: Option<String>, // Source URL if scraped
    pub in_stock: Option<bool>, // Track availability
    pub notes: Option<String>,
    pub recorded_at: OffsetDateTime,
}

impl PriceHistory {
    pub fn new<S: Into<String>>(supplier_id: S, peptide_name: S, cost_per_mg: f32) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            supplier_id: supplier_id.into(),
            peptide_name: peptide_name.into(),
            cost_per_mg,
            url: None,
            in_stock: None,
            notes: None,
            recorded_at: now_timestamp(),
        }
    }
}

/// Alert types for notifications
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum AlertType {
    LowStock,
    ExpiringSoon,
    Expired,
    PriceIncrease,
    PriceDecrease,
    OutOfStock,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// System Alert
/// Tracks alerts for low stock, expiring items, price changes, etc.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Alert {
    pub id: String,
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub related_id: Option<String>, // ID of related item (inventory_id, supplier_id, etc.)
    pub related_type: Option<String>, // Type: "inventory", "supplier", "protocol"
    pub is_read: bool,
    pub is_dismissed: bool,
    pub created_at: OffsetDateTime,
}

impl Alert {
    pub fn new<S: Into<String>>(
        alert_type: AlertType,
        severity: AlertSeverity,
        title: S,
        message: S,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            alert_type,
            severity,
            title: title.into(),
            message: message.into(),
            related_id: None,
            related_type: None,
            is_read: false,
            is_dismissed: false,
            created_at: now_timestamp(),
        }
    }
}

/// AI Summary History
/// Stores previous AI summaries for reference
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryHistory {
    pub id: String,
    pub title: String,
    pub original_content: String,
    pub summary_output: String,
    pub format: String, // "markdown", "plain", "bullets"
    pub provider: String, // "openai", "anthropic", "ollama"
    pub created_at: OffsetDateTime,
}

impl SummaryHistory {
    pub fn new<S: Into<String>>(
        title: S,
        original_content: S,
        summary_output: S,
        format: S,
        provider: S,
    ) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            original_content: original_content.into(),
            summary_output: summary_output.into(),
            format: format.into(),
            provider: provider.into(),
            created_at: now_timestamp(),
        }
    }
}
