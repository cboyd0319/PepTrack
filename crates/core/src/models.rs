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
    #[serde(default)]
    pub is_favorite: bool,
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
            is_favorite: false,
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

#[cfg(test)]
mod tests {
    use super::*;

    // =============================================================================
    // Constructor Tests
    // =============================================================================

    #[test]
    fn peptide_protocol_new_creates_valid_protocol() {
        let protocol = PeptideProtocol::new("Morning Stack", "BPC-157");

        assert_eq!(protocol.name, "Morning Stack");
        assert_eq!(protocol.peptide_name, "BPC-157");
        assert!(!protocol.id.is_empty());
        assert!(protocol.notes.is_none());
        assert!(protocol.current_vial_status.is_none());
        assert!(protocol.target_concentration_mg_ml.is_none());
    }

    #[test]
    fn dose_log_new_creates_valid_log() {
        let dose = DoseLog::new("protocol-123", "Left Shoulder", 0.5);

        assert_eq!(dose.protocol_id, "protocol-123");
        assert_eq!(dose.site, "Left Shoulder");
        assert_eq!(dose.amount_mg, 0.5);
        assert!(!dose.id.is_empty());
        assert!(dose.notes.is_none());
    }

    #[test]
    fn literature_entry_new_creates_valid_entry() {
        let entry = LiteratureEntry::new("pubmed", "Research Paper Title");

        assert_eq!(entry.source, "pubmed");
        assert_eq!(entry.title, "Research Paper Title");
        assert!(!entry.id.is_empty());
        assert!(entry.url.is_none());
        assert!(entry.summary.is_none());
        assert!(entry.relevance_score.is_none());
    }

    #[test]
    fn supplier_new_creates_valid_supplier() {
        let supplier = Supplier::new("PeptideSource");

        assert_eq!(supplier.name, "PeptideSource");
        assert!(!supplier.id.is_empty());
        assert!(supplier.contact_email.is_none());
        assert!(supplier.contact_phone.is_none());
        assert!(supplier.website.is_none());
        assert!(supplier.notes.is_none());
    }

    #[test]
    fn inventory_item_new_creates_valid_item() {
        let item = InventoryItem::new("protocol-123");

        assert_eq!(item.protocol_id, "protocol-123");
        assert!(!item.id.is_empty());
        assert!(item.supplier_id.is_none());
        assert!(matches!(item.vial_status, VialStatus::Sealed));
    }

    #[test]
    fn price_history_new_creates_valid_entry() {
        let price = PriceHistory::new("supplier-123", "BPC-157", 2.5);

        assert_eq!(price.supplier_id, "supplier-123");
        assert_eq!(price.peptide_name, "BPC-157");
        assert_eq!(price.cost_per_mg, 2.5);
        assert!(!price.id.is_empty());
    }

    #[test]
    fn alert_new_creates_valid_alert() {
        let alert = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Low Stock Alert",
            "Vial is running low"
        );

        assert_eq!(alert.alert_type, AlertType::LowStock);
        assert_eq!(alert.severity, AlertSeverity::Warning);
        assert_eq!(alert.title, "Low Stock Alert");
        assert_eq!(alert.message, "Vial is running low");
        assert!(!alert.is_read);
        assert!(!alert.is_dismissed);
    }

    #[test]
    fn summary_history_new_creates_valid_summary() {
        let summary = SummaryHistory::new(
            "Paper Title",
            "Original content",
            "Summary output",
            "markdown",
            "claude"
        );

        assert_eq!(summary.title, "Paper Title");
        assert_eq!(summary.format, "markdown");
        assert_eq!(summary.provider, "claude");
    }

    // =============================================================================
    // Serialization Tests
    // =============================================================================

    #[test]
    fn peptide_protocol_serialization_roundtrip() {
        let mut protocol = PeptideProtocol::new("Test Protocol", "TB-500");
        protocol.notes = Some("Test notes".to_string());
        protocol.target_concentration_mg_ml = Some(2.5);

        let json = serde_json::to_string(&protocol).expect("serialize");
        let deserialized: PeptideProtocol = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, protocol.id);
        assert_eq!(deserialized.name, protocol.name);
        assert_eq!(deserialized.peptide_name, protocol.peptide_name);
        assert_eq!(deserialized.notes, protocol.notes);
        assert_eq!(deserialized.target_concentration_mg_ml, protocol.target_concentration_mg_ml);
    }

    #[test]
    fn dose_log_serialization_roundtrip() {
        let mut dose = DoseLog::new("protocol-123", "Site", 0.75);
        dose.notes = Some("Morning dose".to_string());

        let json = serde_json::to_string(&dose).expect("serialize");
        let deserialized: DoseLog = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.id, dose.id);
        assert_eq!(deserialized.protocol_id, dose.protocol_id);
        assert_eq!(deserialized.site, dose.site);
        assert_eq!(deserialized.amount_mg, dose.amount_mg);
        assert_eq!(deserialized.notes, dose.notes);
    }

    #[test]
    fn literature_entry_serialization_roundtrip() {
        let mut entry = LiteratureEntry::new("openalex", "Paper");
        entry.url = Some("https://example.com".to_string());
        entry.summary = Some("Summary text".to_string());
        entry.relevance_score = Some(0.95);

        let json = serde_json::to_string(&entry).expect("serialize");
        let deserialized: LiteratureEntry = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.title, entry.title);
        assert_eq!(deserialized.url, entry.url);
        assert_eq!(deserialized.relevance_score, entry.relevance_score);
    }

    #[test]
    fn supplier_serialization_roundtrip() {
        let mut supplier = Supplier::new("TestSupplier");
        supplier.contact_email = Some("test@example.com".to_string());
        supplier.website = Some("https://test.com".to_string());

        let json = serde_json::to_string(&supplier).expect("serialize");
        let deserialized: Supplier = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.name, supplier.name);
        assert_eq!(deserialized.contact_email, supplier.contact_email);
        assert_eq!(deserialized.website, supplier.website);
    }

    #[test]
    fn inventory_item_serialization_roundtrip() {
        let mut item = InventoryItem::new("protocol-123");
        item.vial_status = VialStatus::Opened;
        item.quantity_mg = Some(10.0);
        item.batch_number = Some("BATCH123".to_string());

        let json = serde_json::to_string(&item).expect("serialize");
        let deserialized: InventoryItem = serde_json::from_str(&json).expect("deserialize");

        assert_eq!(deserialized.protocol_id, item.protocol_id);
        assert!(matches!(deserialized.vial_status, VialStatus::Opened));
        assert_eq!(deserialized.quantity_mg, item.quantity_mg);
        assert_eq!(deserialized.batch_number, item.batch_number);
    }

    // =============================================================================
    // Enum Serialization Tests
    // =============================================================================

    #[test]
    fn vial_status_serializes_correctly() {
        assert_eq!(serde_json::to_string(&VialStatus::Sealed).unwrap(), r#""sealed""#);
        assert_eq!(serde_json::to_string(&VialStatus::Opened).unwrap(), r#""opened""#);
        assert_eq!(serde_json::to_string(&VialStatus::Empty).unwrap(), r#""empty""#);
        assert_eq!(serde_json::to_string(&VialStatus::Expired).unwrap(), r#""expired""#);
    }

    #[test]
    fn vial_status_deserializes_correctly() {
        assert!(matches!(
            serde_json::from_str::<VialStatus>(r#""sealed""#).unwrap(),
            VialStatus::Sealed
        ));
        assert!(matches!(
            serde_json::from_str::<VialStatus>(r#""opened""#).unwrap(),
            VialStatus::Opened
        ));
        assert!(matches!(
            serde_json::from_str::<VialStatus>(r#""empty""#).unwrap(),
            VialStatus::Empty
        ));
        assert!(matches!(
            serde_json::from_str::<VialStatus>(r#""expired""#).unwrap(),
            VialStatus::Expired
        ));
    }

    #[test]
    fn alert_type_serializes_correctly() {
        assert_eq!(serde_json::to_string(&AlertType::LowStock).unwrap(), r#""low_stock""#);
        assert_eq!(serde_json::to_string(&AlertType::ExpiringSoon).unwrap(), r#""expiring_soon""#);
        assert_eq!(serde_json::to_string(&AlertType::Expired).unwrap(), r#""expired""#);
        assert_eq!(serde_json::to_string(&AlertType::PriceIncrease).unwrap(), r#""price_increase""#);
        assert_eq!(serde_json::to_string(&AlertType::PriceDecrease).unwrap(), r#""price_decrease""#);
        assert_eq!(serde_json::to_string(&AlertType::OutOfStock).unwrap(), r#""out_of_stock""#);
    }

    #[test]
    fn alert_severity_serializes_correctly() {
        assert_eq!(serde_json::to_string(&AlertSeverity::Info).unwrap(), r#""info""#);
        assert_eq!(serde_json::to_string(&AlertSeverity::Warning).unwrap(), r#""warning""#);
        assert_eq!(serde_json::to_string(&AlertSeverity::Critical).unwrap(), r#""critical""#);
    }

    // =============================================================================
    // Edge Case Tests
    // =============================================================================

    #[test]
    fn protocol_handles_empty_strings() {
        let protocol = PeptideProtocol::new("", "");
        assert_eq!(protocol.name, "");
        assert_eq!(protocol.peptide_name, "");
    }

    #[test]
    fn dose_log_handles_zero_amount() {
        let dose = DoseLog::new("protocol", "site", 0.0);
        assert_eq!(dose.amount_mg, 0.0);
    }

    #[test]
    fn dose_log_handles_large_amount() {
        let dose = DoseLog::new("protocol", "site", 999999.99);
        assert_eq!(dose.amount_mg, 999999.99);
    }

    #[test]
    fn protocol_handles_unicode_in_name() {
        let protocol = PeptideProtocol::new("测试协议", "BPC-157 🧪");
        assert_eq!(protocol.name, "测试协议");
        assert_eq!(protocol.peptide_name, "BPC-157 🧪");

        let json = serde_json::to_string(&protocol).expect("serialize");
        let deserialized: PeptideProtocol = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.name, "测试协议");
    }

    #[test]
    fn supplier_handles_special_characters_in_email() {
        let mut supplier = Supplier::new("Test");
        supplier.contact_email = Some("test+tag@example.com".to_string());

        let json = serde_json::to_string(&supplier).expect("serialize");
        let deserialized: Supplier = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.contact_email, Some("test+tag@example.com".to_string()));
    }

    #[test]
    fn inventory_item_handles_none_fields() {
        let item = InventoryItem::new("protocol");

        let json = serde_json::to_string(&item).expect("serialize");
        let deserialized: InventoryItem = serde_json::from_str(&json).expect("deserialize");

        assert!(deserialized.supplier_id.is_none());
        assert!(deserialized.quantity_mg.is_none());
        assert!(deserialized.expiry_date.is_none());
    }

    #[test]
    fn alert_handles_long_messages() {
        let long_message = "a".repeat(10000);
        let alert = Alert::new(
            AlertType::LowStock,
            AlertSeverity::Warning,
            "Title",
            &long_message
        );

        assert_eq!(alert.message.len(), 10000);

        let json = serde_json::to_string(&alert).expect("serialize");
        let deserialized: Alert = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(deserialized.message.len(), 10000);
    }

    #[test]
    fn price_history_handles_extreme_costs() {
        let cheap = PriceHistory::new("supplier", "peptide", 0.01);
        assert_eq!(cheap.cost_per_mg, 0.01);

        let expensive = PriceHistory::new("supplier", "peptide", 999.99);
        assert_eq!(expensive.cost_per_mg, 999.99);
    }

    // =============================================================================
    // OffsetDateTime Serialization Tests
    // =============================================================================

    #[test]
    fn offsetdatetime_serializes_to_iso8601() {
        let protocol = PeptideProtocol::new("Test", "BPC-157");
        let json = serde_json::to_string(&protocol).expect("serialize");

        // Should contain ISO 8601 formatted timestamps
        assert!(json.contains("created_at"));
        assert!(json.contains("updated_at"));
        assert!(json.contains('T')); // ISO 8601 has T separator
        assert!(json.contains('Z') || json.contains('+')); // UTC indicator
    }

    #[test]
    fn offsetdatetime_deserializes_from_iso8601() {
        let json = r#"{
            "id": "test-id",
            "name": "Test",
            "peptide_name": "BPC-157",
            "notes": null,
            "current_vial_status": null,
            "target_concentration_mg_ml": null,
            "created_at": "2024-01-15T10:30:00Z",
            "updated_at": "2024-01-15T10:30:00Z"
        }"#;

        let protocol: PeptideProtocol = serde_json::from_str(json).expect("deserialize");
        assert_eq!(protocol.name, "Test");
    }
}
