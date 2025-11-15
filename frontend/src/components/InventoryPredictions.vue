<template>
  <div class="inventory-predictions">
    <h2>📦 Inventory Predictions</h2>
    <p class="subtitle">Predictive alerts based on dose history and usage patterns</p>

    <!-- Controls -->
    <div class="panel controls-section">
      <div class="control-group">
        <label for="threshold-days">
          Alert Threshold (days)
          <input
            id="threshold-days"
            v-model.number="thresholdDays"
            type="number"
            min="1"
            max="90"
            @change="loadPredictions"
          />
        </label>

        <label for="analysis-days">
          Analysis Period (days)
          <input
            id="analysis-days"
            v-model.number="analysisDays"
            type="number"
            min="7"
            max="180"
            @change="loadPredictions"
          />
        </label>

        <button
          @click="loadPredictions"
          class="refresh-btn"
          :disabled="isLoading"
          aria-label="Refresh predictions"
        >
          {{ isLoading ? '⏳ Loading...' : '↻ Refresh' }}
        </button>

        <button
          @click="createAlerts"
          class="alert-btn"
          :disabled="isCreatingAlerts || predictions.length === 0"
          aria-label="Create alerts for low stock items"
        >
          {{ isCreatingAlerts ? '⏳ Creating...' : '🔔 Create Alerts' }}
        </button>
      </div>

      <div class="info-text">
        Analyzing last {{ analysisDays }} days of usage. Alerting {{ thresholdDays }} days before predicted depletion.
      </div>
    </div>

    <!-- Predictions List -->
    <div class="predictions-section">
      <div class="section-header">
        <h3>Inventory Status ({{ predictions.length }} item{{ predictions.length !== 1 ? 's' : '' }})</h3>
        <div v-if="predictions.length > 0" class="legend">
          <span class="legend-item critical">■ Critical (≤3 days)</span>
          <span class="legend-item warning">■ Warning (≤7 days)</span>
          <span class="legend-item ok">■ OK (>7 days)</span>
        </div>
      </div>

      <div v-if="predictions.length === 0 && !isLoading" class="no-predictions">
        No inventory predictions available. Make sure you have inventory items with usage history.
      </div>

      <div v-else class="predictions-list">
        <div
          v-for="prediction in sortedPredictions"
          :key="prediction.inventory_id"
          class="prediction-card"
          :class="{
            'status-critical': prediction.estimated_days_remaining <= 3,
            'status-warning': prediction.estimated_days_remaining > 3 && prediction.estimated_days_remaining <= 7,
            'status-ok': prediction.estimated_days_remaining > 7,
          }"
        >
          <div class="card-header">
            <div class="card-title">
              <h4>{{ prediction.protocol_name }}</h4>
              <span class="peptide-name">{{ prediction.peptide_name }}</span>
            </div>
            <div class="status-badge" :class="getSeverityClass(prediction.estimated_days_remaining)">
              {{ formatDaysRemaining(prediction.estimated_days_remaining) }}
            </div>
          </div>

          <div class="card-body">
            <div class="metric-grid">
              <div class="metric-item">
                <span class="metric-label">Current Stock</span>
                <span class="metric-value">{{ prediction.current_quantity_mg.toFixed(1) }} mg</span>
              </div>

              <div class="metric-item">
                <span class="metric-label">Daily Usage</span>
                <span class="metric-value">{{ prediction.average_daily_usage_mg.toFixed(2) }} mg/day</span>
              </div>

              <div class="metric-item">
                <span class="metric-label">Est. Remaining</span>
                <span class="metric-value" :class="getSeverityClass(prediction.estimated_days_remaining)">
                  {{ prediction.estimated_days_remaining.toFixed(1) }} days
                </span>
              </div>

              <div class="metric-item">
                <span class="metric-label">Status</span>
                <span class="metric-value">
                  {{ prediction.will_run_out_soon ? '⚠️ Low Stock' : '✅ Adequate' }}
                </span>
              </div>
            </div>

            <div v-if="prediction.will_run_out_soon" class="alert-message">
              ⚠️ This item is predicted to run out soon. Consider reordering.
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  predictInventoryDepletion,
  checkInventoryAndCreateAlerts,
  type InventoryPrediction,
} from '../api/peptrack';

// State
const predictions = ref<InventoryPrediction[]>([]);
const isLoading = ref(false);
const isCreatingAlerts = ref(false);
const thresholdDays = ref(14); // Default: alert 14 days before depletion
const analysisDays = ref(30); // Default: analyze last 30 days

// Computed
const sortedPredictions = computed(() => {
  return [...predictions.value].sort((a, b) => {
    // Sort by estimated days remaining (ascending)
    return a.estimated_days_remaining - b.estimated_days_remaining;
  });
});

onMounted(async () => {
  await loadPredictions();
});

async function loadPredictions() {
  isLoading.value = true;
  try {
    predictions.value = await predictInventoryDepletion(thresholdDays.value, analysisDays.value);
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load inventory predictions' });
  } finally {
    isLoading.value = false;
  }
}

async function createAlerts() {
  isCreatingAlerts.value = true;
  try {
    const alerts = await checkInventoryAndCreateAlerts(thresholdDays.value, analysisDays.value);

    if (alerts.length === 0) {
      showSuccessToast('No Alerts Needed', 'All inventory levels are adequate');
    } else {
      showSuccessToast(
        'Alerts Created',
        `Created ${alerts.length} alert${alerts.length !== 1 ? 's' : ''} for low stock items`
      );
    }
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'create inventory alerts' });
  } finally {
    isCreatingAlerts.value = false;
  }
}

function formatDaysRemaining(days: number): string {
  if (days <= 1) {
    return '< 1 day';
  } else if (days <= 3) {
    return `${days.toFixed(0)} days`;
  } else if (days <= 7) {
    return `${days.toFixed(0)} days`;
  } else if (days <= 30) {
    return `${days.toFixed(0)} days`;
  } else {
    const weeks = Math.floor(days / 7);
    return `~${weeks} week${weeks !== 1 ? 's' : ''}`;
  }
}

function getSeverityClass(days: number): string {
  if (days <= 3) return 'severity-critical';
  if (days <= 7) return 'severity-warning';
  return 'severity-ok';
}
</script>

<style scoped>
.inventory-predictions {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  margin-bottom: 8px;
  color: #2c3e50;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin-bottom: 20px;
  margin-top: 0;
}

.panel {
  background: #f9f9f9;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
}

.controls-section {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.control-group {
  display: flex;
  gap: 15px;
  flex-wrap: wrap;
  align-items: flex-end;
}

.control-group label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-weight: 500;
  color: #2c3e50;
  font-size: 14px;
}

.control-group input {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  width: 120px;
}

.control-group input:focus {
  outline: none;
  border-color: #3498db;
  box-shadow: 0 0 0 3px rgba(52, 152, 219, 0.1);
}

.refresh-btn,
.alert-btn {
  padding: 8px 16px;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
}

.refresh-btn {
  background-color: #3498db;
  color: white;
}

.refresh-btn:hover:not(:disabled) {
  background-color: #2980b9;
}

.alert-btn {
  background-color: #e67e22;
  color: white;
}

.alert-btn:hover:not(:disabled) {
  background-color: #d35400;
}

.refresh-btn:disabled,
.alert-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.info-text {
  font-size: 13px;
  color: #666;
  font-style: italic;
}

.predictions-section {
  margin-top: 20px;
}

.section-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  flex-wrap: wrap;
  gap: 10px;
}

.section-header h3 {
  margin: 0;
  color: #2c3e50;
}

.legend {
  display: flex;
  gap: 15px;
  font-size: 13px;
  flex-wrap: wrap;
}

.legend-item {
  display: flex;
  align-items: center;
  gap: 5px;
}

.legend-item.critical {
  color: #e74c3c;
  font-weight: 600;
}

.legend-item.warning {
  color: #e67e22;
  font-weight: 600;
}

.legend-item.ok {
  color: #27ae60;
  font-weight: 600;
}

.no-predictions {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
  background: #f9f9f9;
  border-radius: 8px;
}

.predictions-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.prediction-card {
  background: white;
  border: 1px solid #ddd;
  border-left: 5px solid #95a5a6;
  border-radius: 8px;
  padding: 16px;
  transition: all 0.2s;
}

.prediction-card.status-critical {
  border-left-color: #e74c3c;
  background-color: #fef5f5;
}

.prediction-card.status-warning {
  border-left-color: #e67e22;
  background-color: #fef9f5;
}

.prediction-card.status-ok {
  border-left-color: #27ae60;
}

.prediction-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.card-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 15px;
}

.card-title h4 {
  margin: 0 0 4px 0;
  color: #2c3e50;
  font-size: 18px;
}

.peptide-name {
  font-size: 14px;
  color: #7f8c8d;
  font-weight: normal;
}

.status-badge {
  padding: 6px 14px;
  border-radius: 16px;
  font-size: 13px;
  font-weight: bold;
}

.status-badge.severity-critical {
  background-color: #e74c3c;
  color: white;
}

.status-badge.severity-warning {
  background-color: #e67e22;
  color: white;
}

.status-badge.severity-ok {
  background-color: #27ae60;
  color: white;
}

.card-body {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.metric-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
}

.metric-item {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.metric-label {
  font-size: 12px;
  color: #7f8c8d;
  font-weight: 500;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.metric-value {
  font-size: 16px;
  font-weight: 600;
  color: #2c3e50;
}

.metric-value.severity-critical {
  color: #e74c3c;
}

.metric-value.severity-warning {
  color: #e67e22;
}

.metric-value.severity-ok {
  color: #27ae60;
}

.alert-message {
  background-color: #fff3cd;
  border: 1px solid #ffc107;
  border-radius: 6px;
  padding: 10px 12px;
  color: #856404;
  font-size: 14px;
  font-weight: 500;
}

@media (max-width: 768px) {
  .control-group {
    flex-direction: column;
    align-items: stretch;
  }

  .control-group input {
    width: 100%;
  }

  .section-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .metric-grid {
    grid-template-columns: 1fr 1fr;
  }
}
</style>
