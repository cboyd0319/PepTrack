use peptrack_core::{Alert, AlertSeverity, AlertType, PriceHistory, SummaryHistory};
use serde::{Deserialize, Serialize};
use tauri::State;
use tracing::{error, info};

use crate::state::AppState;

// ========== Price History Commands ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AddPricePayload {
    pub supplier_id: String,
    pub peptide_name: String,
    pub cost_per_mg: f32,
    pub url: Option<String>,
    pub in_stock: Option<bool>,
    pub notes: Option<String>,
}

#[tauri::command]
pub async fn add_price_history(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: AddPricePayload,
) -> Result<PriceHistory, String> {
    info!("Adding price history: {} @ ${}/mg", payload.peptide_name, payload.cost_per_mg);

    let mut entry = PriceHistory::new(
        &payload.supplier_id,
        &payload.peptide_name,
        payload.cost_per_mg,
    );
    entry.url = payload.url;
    entry.in_stock = payload.in_stock;
    entry.notes = payload.notes;

    state.storage.add_price_history(&entry).map_err(|e| {
        error!("Failed to add price history: {:#}", e);
        format!("Failed to add price history: {}", e)
    })?;

    Ok(entry)
}

#[tauri::command]
pub async fn list_price_history(
    state: State<'_, std::sync::Arc<AppState>>,
    supplier_id: String,
    peptide_name: Option<String>,
) -> Result<Vec<PriceHistory>, String> {
    state
        .storage
        .list_price_history_for_supplier(&supplier_id, peptide_name.as_deref())
        .map_err(|e| {
            error!("Failed to list price history: {:#}", e);
            format!("Failed to list price history: {}", e)
        })
}

#[tauri::command]
pub async fn get_latest_price(
    state: State<'_, std::sync::Arc<AppState>>,
    supplier_id: String,
    peptide_name: String,
) -> Result<Option<PriceHistory>, String> {
    state
        .storage
        .get_latest_price(&supplier_id, &peptide_name)
        .map_err(|e| {
            error!("Failed to get latest price: {:#}", e);
            format!("Failed to get latest price: {}", e)
        })
}

// ========== Alert Commands ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CreateAlertPayload {
    pub alert_type: AlertType,
    pub severity: AlertSeverity,
    pub title: String,
    pub message: String,
    pub related_id: Option<String>,
    pub related_type: Option<String>,
}

#[tauri::command]
pub async fn create_alert(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: CreateAlertPayload,
) -> Result<Alert, String> {
    info!("Creating alert: {}", payload.title);

    let mut alert = Alert::new(
        payload.alert_type,
        payload.severity,
        &payload.title,
        &payload.message,
    );
    alert.related_id = payload.related_id;
    alert.related_type = payload.related_type;

    state.storage.create_alert(&alert).map_err(|e| {
        error!("Failed to create alert: {:#}", e);
        format!("Failed to create alert: {}", e)
    })?;

    Ok(alert)
}

#[tauri::command]
pub async fn list_alerts(
    state: State<'_, std::sync::Arc<AppState>>,
    include_dismissed: Option<bool>,
) -> Result<Vec<Alert>, String> {
    state
        .storage
        .list_alerts(include_dismissed.unwrap_or(false))
        .map_err(|e| {
            error!("Failed to list alerts: {:#}", e);
            format!("Failed to list alerts: {}", e)
        })
}

#[tauri::command]
pub async fn mark_alert_read(
    state: State<'_, std::sync::Arc<AppState>>,
    alert_id: String,
) -> Result<(), String> {
    state.storage.mark_alert_read(&alert_id).map_err(|e| {
        error!("Failed to mark alert as read: {:#}", e);
        format!("Failed to mark alert as read: {}", e)
    })
}

#[tauri::command]
pub async fn dismiss_alert(
    state: State<'_, std::sync::Arc<AppState>>,
    alert_id: String,
) -> Result<(), String> {
    state.storage.dismiss_alert(&alert_id).map_err(|e| {
        error!("Failed to dismiss alert: {:#}", e);
        format!("Failed to dismiss alert: {}", e)
    })
}

#[tauri::command]
pub async fn clear_all_alerts(
    state: State<'_, std::sync::Arc<AppState>>,
) -> Result<(), String> {
    info!("Clearing all alerts");
    state.storage.clear_all_alerts().map_err(|e| {
        error!("Failed to clear alerts: {:#}", e);
        format!("Failed to clear alerts: {}", e)
    })
}

// ========== Summary History Commands ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SaveSummaryPayload {
    pub title: String,
    pub original_content: String,
    pub summary_output: String,
    pub format: String,
    pub provider: String,
}

#[tauri::command]
pub async fn save_summary(
    state: State<'_, std::sync::Arc<AppState>>,
    payload: SaveSummaryPayload,
) -> Result<SummaryHistory, String> {
    info!("Saving summary: {}", payload.title);

    let summary = SummaryHistory::new(
        &payload.title,
        &payload.original_content,
        &payload.summary_output,
        &payload.format,
        &payload.provider,
    );

    state.storage.save_summary(&summary).map_err(|e| {
        error!("Failed to save summary: {:#}", e);
        format!("Failed to save summary: {}", e)
    })?;

    Ok(summary)
}

#[tauri::command]
pub async fn list_summary_history(
    state: State<'_, std::sync::Arc<AppState>>,
    limit: Option<usize>,
) -> Result<Vec<SummaryHistory>, String> {
    state.storage.list_summary_history(limit).map_err(|e| {
        error!("Failed to list summary history: {:#}", e);
        format!("Failed to list summary history: {}", e)
    })
}

#[tauri::command]
pub async fn delete_summary(
    state: State<'_, std::sync::Arc<AppState>>,
    summary_id: String,
) -> Result<(), String> {
    info!("Deleting summary: {}", summary_id);
    state.storage.delete_summary(&summary_id).map_err(|e| {
        error!("Failed to delete summary: {:#}", e);
        format!("Failed to delete summary: {}", e)
    })
}

// ========== Analytics & Reporting Commands ==========

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PriceComparison {
    pub peptide_name: String,
    pub suppliers: Vec<SupplierPrice>,
    pub lowest_price: f32,
    pub highest_price: f32,
    pub average_price: f32,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SupplierPrice {
    pub supplier_id: String,
    pub supplier_name: String,
    pub cost_per_mg: f32,
    pub in_stock: Option<bool>,
    pub recorded_at: String,
}

#[tauri::command]
pub async fn compare_prices(
    state: State<'_, std::sync::Arc<AppState>>,
    peptide_name: String,
) -> Result<PriceComparison, String> {
    info!("Comparing prices for: {}", peptide_name);

    // Get all suppliers
    let suppliers = state.storage.list_suppliers().map_err(|e| {
        error!("Failed to list suppliers: {:#}", e);
        format!("Failed to list suppliers: {}", e)
    })?;

    let mut supplier_prices = Vec::new();

    for supplier in suppliers {
        if let Ok(Some(price_entry)) = state
            .storage
            .get_latest_price(&supplier.id, &peptide_name)
        {
            supplier_prices.push(SupplierPrice {
                supplier_id: supplier.id.clone(),
                supplier_name: supplier.name.clone(),
                cost_per_mg: price_entry.cost_per_mg,
                in_stock: price_entry.in_stock,
                recorded_at: price_entry.recorded_at.to_string(),
            });
        }
    }

    if supplier_prices.is_empty() {
        return Err(format!("No price data found for {}", peptide_name));
    }

    let prices: Vec<f32> = supplier_prices.iter().map(|sp| sp.cost_per_mg).collect();
    let lowest_price = prices.iter().cloned().fold(f32::INFINITY, f32::min);
    let highest_price = prices.iter().cloned().fold(f32::NEG_INFINITY, f32::max);
    let average_price = prices.iter().sum::<f32>() / prices.len() as f32;

    Ok(PriceComparison {
        peptide_name,
        suppliers: supplier_prices,
        lowest_price,
        highest_price,
        average_price,
    })
}
