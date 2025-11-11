use peptrack_core::{InventoryItem, Supplier, VialStatus};
use serde::{Deserialize, Serialize};
use tauri::State;
use time::OffsetDateTime;
use tracing::{error, info};

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

    state
        .storage
        .upsert_supplier(&supplier)
        .map_err(|e| {
            error!("Failed to create supplier: {:#}", e);
            format!("Failed to create supplier: {}", e)
        })?;

    Ok(supplier)
}

#[tauri::command]
pub async fn list_suppliers(state: State<'_, std::sync::Arc<AppState>>) -> Result<Vec<Supplier>, String> {
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
    state
        .storage
        .get_supplier(&supplier_id)
        .map_err(|e| {
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

    state
        .storage
        .upsert_supplier(&supplier)
        .map_err(|e| {
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

    state
        .storage
        .delete_supplier(&supplier_id)
        .map_err(|e| {
            error!("Failed to delete supplier: {:#}", e);
            format!("Failed to delete supplier: {}", e)
        })
}

// ========== Inventory Commands ==========

#[tauri::command]
pub async fn create_inventory_item(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: CreateInventoryPayload,
) -> Result<InventoryItem, String> {
    info!("Creating inventory item for protocol: {}", payload.protocol_id);

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

    state
        .storage
        .upsert_inventory_item(&item)
        .map_err(|e| {
            error!("Failed to create inventory item: {:#}", e);
            format!("Failed to create inventory item: {}", e)
        })?;

    Ok(item)
}

#[tauri::command]
pub async fn list_inventory(state: State<'_, std::sync::Arc<AppState>>) -> Result<Vec<InventoryItem>, String> {
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
    state
        .storage
        .get_inventory_item(&item_id)
        .map_err(|e| {
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

    state
        .storage
        .upsert_inventory_item(&item)
        .map_err(|e| {
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

    state
        .storage
        .delete_inventory_item(&item_id)
        .map_err(|e| {
            error!("Failed to delete inventory item: {:#}", e);
            format!("Failed to delete inventory item: {}", e)
        })
}

// ========== Payload Structs ==========

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
        assert_eq!(payload.contact_email, Some("info@peptidesciences.com".to_string()));
        assert_eq!(payload.website, Some("https://peptidesciences.com".to_string()));
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
