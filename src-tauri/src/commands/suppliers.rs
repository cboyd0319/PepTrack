use peptrack_core::{InventoryItem, Supplier, VialStatus};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::OffsetDateTime;
use tracing::{error, info, warn};
use regex::Regex;

use crate::state::AppState;

// ========== Supplier Commands ==========

#[tauri::command]
pub async fn create_supplier(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: CreateSupplierPayload,
) -> Result<Supplier, String> {
    info!("Creating supplier: {}", payload.name);

    let mut supplier = Supplier::new(&payload.name);
    supplier.contact_email = payload.contact_email;
    supplier.contact_phone = payload.contact_phone;
    supplier.website = payload.website;
    supplier.notes = payload.notes;

    state.storage.upsert_supplier(&supplier).map_err(|e| {
        error!("Failed to create supplier: {:#}", e);
        format!("Failed to create supplier: {}", e)
    })?;

    Ok(supplier)
}

#[tauri::command]
pub async fn list_suppliers(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<Supplier>, String> {
    state.storage.list_suppliers().map_err(|e| {
        error!("Failed to list suppliers: {:#}", e);
        format!("Failed to list suppliers: {}", e)
    })
}

#[tauri::command]
pub async fn get_supplier(
    state: State<'_, std::sync::Arc<AppState>>,
    supplier_id: String,
) -> Result<Option<Supplier>, String> {
    state.storage.get_supplier(&supplier_id).map_err(|e| {
        error!("Failed to get supplier: {:#}", e);
        format!("Failed to get supplier: {}", e)
    })
}

#[tauri::command]
pub async fn update_supplier(
    state: State<'_, std::sync::Arc<AppState>>,
    supplier_id: String,
    payload: UpdateSupplierPayload,
) -> Result<Supplier, String> {
    info!("Updating supplier: {}", supplier_id);

    let mut supplier = state
        .storage
        .get_supplier(&supplier_id)
        .map_err(|e| format!("Failed to fetch supplier: {}", e))?
        .ok_or_else(|| "Supplier not found".to_string())?;

    if let Some(name) = payload.name {
        supplier.name = name;
    }
    supplier.contact_email = payload.contact_email.or(supplier.contact_email);
    supplier.contact_phone = payload.contact_phone.or(supplier.contact_phone);
    supplier.website = payload.website.or(supplier.website);
    supplier.notes = payload.notes.or(supplier.notes);
    supplier.updated_at = OffsetDateTime::now_utc();

    state.storage.upsert_supplier(&supplier).map_err(|e| {
        error!("Failed to update supplier: {:#}", e);
        format!("Failed to update supplier: {}", e)
    })?;

    Ok(supplier)
}

#[tauri::command]
pub async fn delete_supplier(
    state: State<'_, std::sync::Arc<AppState>>,
    supplier_id: String,
) -> Result<(), String> {
    info!("Deleting supplier: {}", supplier_id);

    state.storage.delete_supplier(&supplier_id).map_err(|e| {
        error!("Failed to delete supplier: {:#}", e);
        format!("Failed to delete supplier: {}", e)
    })
}

/// Validate URL to prevent SSRF attacks
fn validate_scraping_url(url_str: &str) -> Result<url::Url, String> {
    let url = url::Url::parse(url_str)
        .map_err(|_| "Invalid URL format".to_string())?;

    // Only allow HTTP/HTTPS
    if url.scheme() != "http" && url.scheme() != "https" {
        return Err("Only HTTP and HTTPS URLs are allowed".to_string());
    }

    // Block localhost and private IP ranges to prevent SSRF
    if let Some(host) = url.host_str() {
        let host_lower = host.to_lowercase();

        if host_lower == "localhost"
            || host_lower == "127.0.0.1"
            || host_lower.starts_with("192.168.")
            || host_lower.starts_with("10.")
            || host_lower.starts_with("172.16.")
            || host_lower.starts_with("172.17.")
            || host_lower.starts_with("172.18.")
            || host_lower.starts_with("172.19.")
            || host_lower.starts_with("172.20.")
            || host_lower.starts_with("172.21.")
            || host_lower.starts_with("172.22.")
            || host_lower.starts_with("172.23.")
            || host_lower.starts_with("172.24.")
            || host_lower.starts_with("172.25.")
            || host_lower.starts_with("172.26.")
            || host_lower.starts_with("172.27.")
            || host_lower.starts_with("172.28.")
            || host_lower.starts_with("172.29.")
            || host_lower.starts_with("172.30.")
            || host_lower.starts_with("172.31.")
            || host_lower == "169.254.169.254"  // AWS/Cloud metadata
            || host_lower.starts_with("[::1]")   // IPv6 localhost
            || host_lower.starts_with("fe80:")   // IPv6 link-local
            || host_lower.starts_with("fc00:")   // IPv6 unique local
        {
            return Err("Access to private/internal addresses is not allowed for security reasons".to_string());
        }
    }

    Ok(url)
}

/// Scrape a website for peptide prices
#[tauri::command]
pub async fn scrape_supplier_website(
    url: String,
    peptide_name: Option<String>,
) -> Result<Vec<PriceMatch>, String> {
    info!("Scraping URL: {} for peptide: {:?}", url, peptide_name);

    // Validate URL to prevent SSRF attacks
    let validated_url = validate_scraping_url(&url)?;

    // Fetch the webpage
    let response = reqwest::get(validated_url).await.map_err(|e| {
        error!("Failed to fetch URL: {:#}", e);
        format!("Failed to fetch webpage: {}", e)
    })?;

    let html = response.text().await.map_err(|e| {
        error!("Failed to read response: {:#}", e);
        format!("Failed to read webpage content: {}", e)
    })?;

    // Extract prices using multiple patterns
    let mut matches = Vec::new();

    // Pattern 1: $X.XX/mg or $X.XX per mg
    let price_per_mg_re = Regex::new(r"\$?(\d+(?:\.\d{1,2})?)\s*(?:/|per)\s*mg").unwrap();
    for cap in price_per_mg_re.captures_iter(&html) {
        if let Some(price_str) = cap.get(1) {
            if let Ok(price) = price_str.as_str().parse::<f32>() {
                matches.push(PriceMatch {
                    price_per_mg: price,
                    context: extract_context(&html, cap.get(0).unwrap().start(), 100),
                    pattern_type: "per_mg".to_string(),
                });
            }
        }
    }

    // Pattern 2: XXmg for $YY or XXmg - $YY
    let vial_price_re = Regex::new(r"(\d+(?:\.\d+)?)\s*mg\s*(?:for|-|:)?\s*\$(\d+(?:\.\d{1,2})?)").unwrap();
    for cap in vial_price_re.captures_iter(&html) {
        if let (Some(mg_str), Some(price_str)) = (cap.get(1), cap.get(2)) {
            if let (Ok(mg), Ok(total_price)) = (mg_str.as_str().parse::<f32>(), price_str.as_str().parse::<f32>()) {
                if mg > 0.0 {
                    let price_per_mg = total_price / mg;
                    matches.push(PriceMatch {
                        price_per_mg,
                        context: extract_context(&html, cap.get(0).unwrap().start(), 100),
                        pattern_type: "vial_price".to_string(),
                    });
                }
            }
        }
    }

    // Pattern 3: Generic price mentions near peptide names
    if let Some(ref peptide) = peptide_name {
        // Prevent ReDoS by limiting peptide name length
        if peptide.len() > 100 {
            warn!("Peptide name too long for regex search: {} chars", peptide.len());
            return Ok(matches);
        }

        let peptide_pattern = format!(r"(?i){}\s*(?:\w+\s*){{0,10}}\$(\d+(?:\.\d{{1,2}})?)", regex::escape(peptide));
        if let Ok(peptide_re) = Regex::new(&peptide_pattern) {
            for cap in peptide_re.captures_iter(&html) {
                if let Some(price_str) = cap.get(1) {
                    if let Ok(price) = price_str.as_str().parse::<f32>() {
                        matches.push(PriceMatch {
                            price_per_mg: price,
                            context: extract_context(&html, cap.get(0).unwrap().start(), 150),
                            pattern_type: "peptide_mention".to_string(),
                        });
                    }
                }
            }
        }
    }

    // Remove duplicates and sort by price
    matches.sort_by(|a, b| a.price_per_mg.partial_cmp(&b.price_per_mg).unwrap());
    matches.dedup_by(|a, b| (a.price_per_mg - b.price_per_mg).abs() < 0.01);

    if matches.is_empty() {
        warn!("No prices found on URL: {}", url);
    } else {
        info!("Found {} price matches", matches.len());
    }

    Ok(matches)
}

/// Extract text context around a position in HTML (strips tags)
fn extract_context(html: &str, position: usize, radius: usize) -> String {
    let start = position.saturating_sub(radius);
    let end = (position + radius).min(html.len());
    let snippet = &html[start..end];

    // Simple HTML tag removal
    let tag_re = Regex::new(r"<[^>]+>").unwrap();
    let clean = tag_re.replace_all(snippet, " ");

    // Collapse whitespace
    let ws_re = Regex::new(r"\s+").unwrap();
    ws_re.replace_all(&clean, " ").trim().to_string()
}

// ========== Inventory Commands ==========

#[tauri::command]
pub async fn create_inventory_item(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: CreateInventoryPayload,
) -> Result<InventoryItem, String> {
    info!(
        "Creating inventory item for protocol: {}",
        payload.protocol_id
    );

    let mut item = InventoryItem::new(&payload.protocol_id);
    item.supplier_id = payload.supplier_id;
    item.vial_number = payload.vial_number;
    item.vial_status = payload.vial_status.unwrap_or(VialStatus::Sealed);
    item.purchase_date = payload.purchase_date;
    item.expiry_date = payload.expiry_date;
    item.cost_per_mg = payload.cost_per_mg;
    item.quantity_mg = payload.quantity_mg;
    item.concentration_mg_ml = payload.concentration_mg_ml;
    item.batch_number = payload.batch_number;
    item.lot_number = payload.lot_number;
    item.notes = payload.notes;

    state.storage.upsert_inventory_item(&item).map_err(|e| {
        error!("Failed to create inventory item: {:#}", e);
        format!("Failed to create inventory item: {}", e)
    })?;

    Ok(item)
}

#[tauri::command]
pub async fn list_inventory(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<Vec<InventoryItem>, String> {
    state.storage.list_inventory().map_err(|e| {
        error!("Failed to list inventory: {:#}", e);
        format!("Failed to list inventory: {}", e)
    })
}

#[tauri::command]
pub async fn list_inventory_by_protocol(
    state: State<'_, std::sync::Arc<AppState>>,
    protocol_id: String,
) -> Result<Vec<InventoryItem>, String> {
    state
        .storage
        .list_inventory_by_protocol(&protocol_id)
        .map_err(|e| {
            error!("Failed to list inventory for protocol: {:#}", e);
            format!("Failed to list inventory: {}", e)
        })
}

#[tauri::command]
pub async fn get_inventory_item(
    state: State<'_, std::sync::Arc<AppState>>,
    item_id: String,
) -> Result<Option<InventoryItem>, String> {
    state.storage.get_inventory_item(&item_id).map_err(|e| {
        error!("Failed to get inventory item: {:#}", e);
        format!("Failed to get inventory item: {}", e)
    })
}

#[tauri::command]
pub async fn update_inventory_item(
    state: State<'_, std::sync::Arc<AppState>>,
    item_id: String,
    payload: UpdateInventoryPayload,
) -> Result<InventoryItem, String> {
    info!("Updating inventory item: {}", item_id);

    let mut item = state
        .storage
        .get_inventory_item(&item_id)
        .map_err(|e| format!("Failed to fetch inventory item: {}", e))?
        .ok_or_else(|| "Inventory item not found".to_string())?;

    item.supplier_id = payload.supplier_id.or(item.supplier_id);
    item.vial_number = payload.vial_number.or(item.vial_number);
    if let Some(status) = payload.vial_status {
        item.vial_status = status;
    }
    item.purchase_date = payload.purchase_date.or(item.purchase_date);
    item.expiry_date = payload.expiry_date.or(item.expiry_date);
    item.cost_per_mg = payload.cost_per_mg.or(item.cost_per_mg);
    item.quantity_mg = payload.quantity_mg.or(item.quantity_mg);
    item.concentration_mg_ml = payload.concentration_mg_ml.or(item.concentration_mg_ml);
    item.batch_number = payload.batch_number.or(item.batch_number);
    item.lot_number = payload.lot_number.or(item.lot_number);
    item.notes = payload.notes.or(item.notes);
    item.updated_at = OffsetDateTime::now_utc();

    state.storage.upsert_inventory_item(&item).map_err(|e| {
        error!("Failed to update inventory item: {:#}", e);
        format!("Failed to update inventory item: {}", e)
    })?;

    Ok(item)
}

#[tauri::command]
pub async fn delete_inventory_item(
    state: State<'_, std::sync::Arc<AppState>>,
    item_id: String,
) -> Result<(), String> {
    info!("Deleting inventory item: {}", item_id);

    state.storage.delete_inventory_item(&item_id).map_err(|e| {
        error!("Failed to delete inventory item: {:#}", e);
        format!("Failed to delete inventory item: {}", e)
    })
}

// ========== Payload Structs ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceMatch {
    pub price_per_mg: f32,
    pub context: String,
    pub pattern_type: String,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateSupplierPayload {
    pub name: String,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateSupplierPayload {
    pub name: Option<String>,
    pub contact_email: Option<String>,
    pub contact_phone: Option<String>,
    pub website: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateInventoryPayload {
    pub protocol_id: String,
    pub supplier_id: Option<String>,
    pub vial_number: Option<String>,
    pub vial_status: Option<VialStatus>,
    pub purchase_date: Option<OffsetDateTime>,
    pub expiry_date: Option<OffsetDateTime>,
    pub cost_per_mg: Option<f32>,
    pub quantity_mg: Option<f32>,
    pub concentration_mg_ml: Option<f32>,
    pub batch_number: Option<String>,
    pub lot_number: Option<String>,
    pub notes: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct UpdateInventoryPayload {
    pub supplier_id: Option<String>,
    pub vial_number: Option<String>,
    pub vial_status: Option<VialStatus>,
    pub purchase_date: Option<OffsetDateTime>,
    pub expiry_date: Option<OffsetDateTime>,
    pub cost_per_mg: Option<f32>,
    pub quantity_mg: Option<f32>,
    pub concentration_mg_ml: Option<f32>,
    pub batch_number: Option<String>,
    pub lot_number: Option<String>,
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_supplier_payload_deserialization() {
        let json = r#"{
            "name": "Peptide Sciences",
            "contactEmail": "info@peptidesciences.com",
            "website": "https://peptidesciences.com"
        }"#;

        let payload: CreateSupplierPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.name, "Peptide Sciences");
        assert_eq!(
            payload.contact_email,
            Some("info@peptidesciences.com".to_string())
        );
        assert_eq!(
            payload.website,
            Some("https://peptidesciences.com".to_string())
        );
    }

    #[test]
    fn test_create_inventory_payload_deserialization() {
        let json = r#"{
            "protocolId": "proto-123",
            "supplierId": "supplier-456",
            "vialNumber": "V001",
            "vialStatus": "sealed",
            "costPerMg": 1.25,
            "quantityMg": 10.0
        }"#;

        let payload: CreateInventoryPayload = serde_json::from_str(json).unwrap();
        assert_eq!(payload.protocol_id, "proto-123");
        assert_eq!(payload.supplier_id, Some("supplier-456".to_string()));
        assert_eq!(payload.vial_number, Some("V001".to_string()));
        assert_eq!(payload.cost_per_mg, Some(1.25));
        assert_eq!(payload.quantity_mg, Some(10.0));
    }
}
