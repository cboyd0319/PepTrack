use peptrack_core::models::{Alert, AlertSeverity, AlertType, PriceHistory, SummaryHistory};
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
pub struct InventoryPrediction {
    pub inventory_id: String,
    pub protocol_id: String,
    pub protocol_name: String,
    pub peptide_name: String,
    pub current_quantity_mg: f32,
    pub average_daily_usage_mg: f32,
    pub estimated_days_remaining: f32,
    pub will_run_out_soon: bool,
    pub threshold_days: i32,
}

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

/// Predict inventory depletion based on dose history
///
/// Analyzes dose logs over the past `analysis_days` to calculate average daily usage.
/// Returns predictions for all inventory items, flagging those that will run out
/// within `threshold_days`.
#[tauri::command]
pub async fn predict_inventory_depletion(
    state: State<'_, std::sync::Arc<AppState>>,
    threshold_days: Option<i32>,
    analysis_days: Option<i32>,
) -> Result<Vec<InventoryPrediction>, String> {
    let threshold = threshold_days.unwrap_or(14); // Default: warn 14 days before depletion
    let lookback = analysis_days.unwrap_or(30); // Default: analyze last 30 days

    info!("Predicting inventory depletion (threshold: {} days, lookback: {} days)", threshold, lookback);

    // Get all inventory items
    let inventory = state.storage.list_inventory().map_err(|e| {
        error!("Failed to list inventory: {:#}", e);
        format!("Failed to list inventory: {}", e)
    })?;

    // Get all protocols for name lookup
    let protocols = state.storage.list_protocols().map_err(|e| {
        error!("Failed to list protocols: {:#}", e);
        format!("Failed to list protocols: {}", e)
    })?;

    let mut predictions = Vec::new();

    for item in inventory {
        // Skip if no remaining quantity
        let current_qty = match item.quantity_remaining_mg {
            Some(qty) if qty > 0.0 => qty,
            _ => continue,
        };

        // Get protocol details
        let protocol = protocols.iter().find(|p| p.id == item.protocol_id);
        if protocol.is_none() {
            continue;
        }
        let protocol = protocol.unwrap();

        // Get dose logs for this protocol
        let dose_logs = state
            .storage
            .list_dose_logs_for_protocol(&item.protocol_id)
            .map_err(|e| {
                error!("Failed to get dose logs for protocol {}: {:#}", item.protocol_id, e);
                format!("Failed to get dose logs: {}", e)
            })?;

        // Calculate average daily usage from recent doses
        let now = time::OffsetDateTime::now_utc();
        let cutoff_date = now - time::Duration::days(lookback as i64);

        let recent_doses: Vec<_> = dose_logs
            .iter()
            .filter(|log| log.timestamp >= cutoff_date)
            .collect();

        if recent_doses.is_empty() {
            // No recent usage, skip prediction
            continue;
        }

        // Calculate total usage and days span
        let total_usage: f32 = recent_doses
            .iter()
            .filter_map(|log| log.amount_mg)
            .sum();

        // Find the date range of doses
        let oldest_dose = recent_doses.iter().map(|log| log.timestamp).min().unwrap();
        let newest_dose = recent_doses.iter().map(|log| log.timestamp).max().unwrap();
        let days_span = (newest_dose - oldest_dose).whole_days() + 1; // +1 to include both endpoints

        if days_span <= 0 {
            continue;
        }

        let average_daily_usage = total_usage / days_span as f32;

        if average_daily_usage <= 0.0 {
            continue;
        }

        // Calculate estimated days remaining
        let estimated_days_remaining = current_qty / average_daily_usage;
        let will_run_out_soon = estimated_days_remaining <= threshold as f32;

        predictions.push(InventoryPrediction {
            inventory_id: item.id.clone(),
            protocol_id: item.protocol_id.clone(),
            protocol_name: protocol.name.clone(),
            peptide_name: protocol.peptide_name.clone(),
            current_quantity_mg: current_qty,
            average_daily_usage_mg: average_daily_usage,
            estimated_days_remaining,
            will_run_out_soon,
            threshold_days: threshold,
        });
    }

    Ok(predictions)
}

/// Check inventory levels and create alerts for items running low
///
/// Automatically creates LowStock alerts for inventory items predicted to run out
/// within the threshold period.
#[tauri::command]
pub async fn check_inventory_and_create_alerts(
    state: State<'_, std::sync::Arc<AppState>>,
    threshold_days: Option<i32>,
    analysis_days: Option<i32>,
) -> Result<Vec<Alert>, String> {
    let threshold = threshold_days.unwrap_or(14);

    info!("Checking inventory and creating alerts (threshold: {} days)", threshold);

    // Get predictions
    let predictions = predict_inventory_depletion(
        state.clone(),
        Some(threshold),
        analysis_days,
    )
    .await?;

    let mut created_alerts = Vec::new();

    for prediction in predictions {
        if !prediction.will_run_out_soon {
            continue;
        }

        // Determine severity based on urgency
        let severity = if prediction.estimated_days_remaining <= 3.0 {
            AlertSeverity::Critical
        } else if prediction.estimated_days_remaining <= 7.0 {
            AlertSeverity::Warning
        } else {
            AlertSeverity::Info
        };

        let title = format!(
            "Low Stock: {} ({})",
            prediction.protocol_name, prediction.peptide_name
        );

        let message = format!(
            "Estimated {:.1} days remaining ({:.1}mg left, using ~{:.2}mg/day). Consider reordering soon.",
            prediction.estimated_days_remaining,
            prediction.current_quantity_mg,
            prediction.average_daily_usage_mg
        );

        let mut alert = Alert::new(AlertType::LowStock, severity, &title, &message);
        alert.related_id = Some(prediction.inventory_id.clone());
        alert.related_type = Some("inventory".to_string());

        // Check if similar alert already exists and is not dismissed
        let existing_alerts = state.storage.list_alerts(false).map_err(|e| {
            error!("Failed to check existing alerts: {:#}", e);
            format!("Failed to check existing alerts: {}", e)
        })?;

        let similar_alert_exists = existing_alerts.iter().any(|a| {
            a.alert_type == AlertType::LowStock
                && a.related_id.as_deref() == Some(&prediction.inventory_id)
                && !a.is_dismissed
        });

        if !similar_alert_exists {
            state.storage.create_alert(&alert).map_err(|e| {
                error!("Failed to create alert: {:#}", e);
                format!("Failed to create alert: {}", e)
            })?;

            created_alerts.push(alert);
            info!("Created low stock alert for: {}", prediction.protocol_name);
        }
    }

    info!("Created {} new inventory alerts", created_alerts.len());
    Ok(created_alerts)
}
