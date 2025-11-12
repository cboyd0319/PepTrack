<template>
  <div class="alerts-dashboard">
    <div class="dashboard-header">
      <div>
        <h2>🔔 Alerts Dashboard</h2>
        <p class="subtitle">Monitor inventory, pricing, and system notifications</p>
      </div>
      <div class="header-actions">
        <button
          v-if="unreadCount > 0"
          @click="markAllAsRead"
          class="action-btn"
        >
          ✓ Mark All Read
        </button>
        <button
          v-if="alerts.length > 0"
          @click="clearAllAlerts"
          class="action-btn danger"
        >
          🗑️ Clear All
        </button>
      </div>
    </div>

    <!-- Filter Bar -->
    <div class="filter-bar">
      <div class="filter-group">
        <label>Filter by Type:</label>
        <select v-model="filterType">
          <option value="all">All Types</option>
          <option value="low_stock">📉 Low Stock</option>
          <option value="expiring_soon">⏰ Expiring Soon</option>
          <option value="expired">⚠️ Expired</option>
          <option value="price_increase">📈 Price Increase</option>
          <option value="price_decrease">📉 Price Decrease</option>
          <option value="out_of_stock">❌ Out of Stock</option>
        </select>
      </div>

      <div class="filter-group">
        <label>Filter by Severity:</label>
        <select v-model="filterSeverity">
          <option value="all">All Severities</option>
          <option value="critical">🔴 Critical</option>
          <option value="warning">🟡 Warning</option>
          <option value="info">🔵 Info</option>
        </select>
      </div>

      <div class="filter-group">
        <label>
          <input type="checkbox" v-model="showDismissed" />
          Show Dismissed
        </label>
      </div>
    </div>

    <!-- Stats Summary -->
    <div class="stats-grid">
      <div class="stat-card critical">
        <div class="stat-value">{{ criticalCount }}</div>
        <div class="stat-label">Critical</div>
      </div>
      <div class="stat-card warning">
        <div class="stat-value">{{ warningCount }}</div>
        <div class="stat-label">Warnings</div>
      </div>
      <div class="stat-card info">
        <div class="stat-value">{{ infoCount }}</div>
        <div class="stat-label">Info</div>
      </div>
      <div class="stat-card unread">
        <div class="stat-value">{{ unreadCount }}</div>
        <div class="stat-label">Unread</div>
      </div>
    </div>

    <!-- Loading State -->
    <div v-if="loading" class="loading-state">
      <div class="spinner"></div>
      <p>Loading alerts...</p>
    </div>

    <!-- Empty State -->
    <div v-else-if="filteredAlerts.length === 0" class="empty-state">
      <div class="empty-icon">🎉</div>
      <h3>{{ alerts.length === 0 ? 'No Alerts' : 'No Matching Alerts' }}</h3>
      <p>
        {{
          alerts.length === 0
            ? 'Everything looks good! No alerts at this time.'
            : 'Try adjusting your filters to see more alerts.'
        }}
      </p>
    </div>

    <!-- Alerts List -->
    <div v-else class="alerts-list">
      <div
        v-for="alert in filteredAlerts"
        :key="alert.id"
        :class="[
          'alert-card',
          alert.severity,
          { unread: !alert.is_read, dismissed: alert.is_dismissed }
        ]"
      >
        <div class="alert-icon">
          {{ getAlertIcon(alert.alert_type) }}
        </div>

        <div class="alert-content">
          <div class="alert-header">
            <h4>{{ alert.title }}</h4>
            <div class="alert-badges">
              <span :class="['badge', 'severity', alert.severity]">
                {{ getSeverityLabel(alert.severity) }}
              </span>
              <span :class="['badge', 'type']">
                {{ getTypeLabel(alert.alert_type) }}
              </span>
            </div>
          </div>

          <p class="alert-message">{{ alert.message }}</p>

          <div class="alert-meta">
            <span class="alert-time">{{ formatTime(alert.created_at) }}</span>
            <span v-if="alert.related_type" class="alert-related">
              Related: {{ alert.related_type }}
            </span>
          </div>
        </div>

        <div class="alert-actions">
          <button
            v-if="!alert.is_read"
            @click="markAsRead(alert.id)"
            class="icon-btn"
            title="Mark as read"
          >
            ✓
          </button>
          <button
            v-if="!alert.is_dismissed"
            @click="dismissAlert(alert.id)"
            class="icon-btn"
            title="Dismiss"
          >
            ✕
          </button>
          <button
            v-if="alert.related_id"
            @click="navigateToRelated(alert)"
            class="icon-btn"
            title="View related item"
          >
            →
          </button>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import type { Alert, AlertType, AlertSeverity } from '../api/peptrack';
import {
  listAlerts,
  markAlertRead,
  dismissAlert as dismissAlertApi,
  clearAllAlerts as clearAllAlertsApi,
} from '../api/peptrack';
import { showSuccessToast, showErrorToast } from '../utils/errorHandling';

const emit = defineEmits<{
  navigate: [tab: string, id?: string];
}>();

const alerts = ref<Alert[]>([]);
const loading = ref(false);
const filterType = ref<AlertType | 'all'>('all');
const filterSeverity = ref<AlertSeverity | 'all'>('all');
const showDismissed = ref(false);

// Computed Stats
const criticalCount = computed(
  () => alerts.value.filter(a => a.severity === 'critical' && !a.is_dismissed).length
);
const warningCount = computed(
  () => alerts.value.filter(a => a.severity === 'warning' && !a.is_dismissed).length
);
const infoCount = computed(
  () => alerts.value.filter(a => a.severity === 'info' && !a.is_dismissed).length
);
const unreadCount = computed(
  () => alerts.value.filter(a => !a.is_read && !a.is_dismissed).length
);

// Filtered Alerts
const filteredAlerts = computed(() => {
  let filtered = alerts.value;

  // Filter by dismissed
  if (!showDismissed.value) {
    filtered = filtered.filter(a => !a.is_dismissed);
  }

  // Filter by type
  if (filterType.value !== 'all') {
    filtered = filtered.filter(a => a.alert_type === filterType.value);
  }

  // Filter by severity
  if (filterSeverity.value !== 'all') {
    filtered = filtered.filter(a => a.severity === filterSeverity.value);
  }

  // Sort: unread first, then by severity, then by time
  return filtered.sort((a, b) => {
    if (a.is_read !== b.is_read) return a.is_read ? 1 : -1;

    const severityOrder = { critical: 0, warning: 1, info: 2 };
    if (a.severity !== b.severity) {
      return severityOrder[a.severity] - severityOrder[b.severity];
    }

    return new Date(b.created_at).getTime() - new Date(a.created_at).getTime();
  });
});

async function loadAlerts() {
  loading.value = true;
  try {
    alerts.value = await listAlerts(showDismissed.value);
  } catch (error) {
    showErrorToast(error, { operation: 'load alerts' });
  } finally {
    loading.value = false;
  }
}

async function markAsRead(alertId: string) {
  try {
    await markAlertRead(alertId);
    const alert = alerts.value.find(a => a.id === alertId);
    if (alert) alert.is_read = true;
    showSuccessToast('Marked as Read', 'Alert marked as read');
  } catch (error) {
    showErrorToast(error, { operation: 'mark alert as read' });
  }
}

async function markAllAsRead() {
  try {
    const unreadAlerts = alerts.value.filter(a => !a.is_read && !a.is_dismissed);
    await Promise.all(unreadAlerts.map(a => markAlertRead(a.id)));
    unreadAlerts.forEach(a => a.is_read = true);
    showSuccessToast('All Read', 'All alerts marked as read');
  } catch (error) {
    showErrorToast(error, { operation: 'mark all as read' });
  }
}

async function dismissAlert(alertId: string) {
  try {
    await dismissAlertApi(alertId);
    const alert = alerts.value.find(a => a.id === alertId);
    if (alert) alert.is_dismissed = true;
    showSuccessToast('Dismissed', 'Alert dismissed');
  } catch (error) {
    showErrorToast(error, { operation: 'dismiss alert' });
  }
}

async function clearAllAlerts() {
  if (!confirm('Clear all alerts? This cannot be undone.')) return;

  try {
    await clearAllAlertsApi();
    alerts.value = [];
    showSuccessToast('Cleared', 'All alerts cleared');
  } catch (error) {
    showErrorToast(error, { operation: 'clear all alerts' });
  }
}

function navigateToRelated(alert: Alert) {
  if (!alert.related_type || !alert.related_id) return;

  const tabMap: Record<string, string> = {
    inventory: 'operations',
    supplier: 'operations',
    protocol: 'protocols',
  };

  const tab = tabMap[alert.related_type] || 'dashboard';
  emit('navigate', tab, alert.related_id);
}

function getAlertIcon(type: AlertType): string {
  const icons: Record<AlertType, string> = {
    low_stock: '📉',
    expiring_soon: '⏰',
    expired: '⚠️',
    price_increase: '📈',
    price_decrease: '📉',
    out_of_stock: '❌',
  };
  return icons[type] || '🔔';
}

function getSeverityLabel(severity: AlertSeverity): string {
  const labels: Record<AlertSeverity, string> = {
    critical: 'Critical',
    warning: 'Warning',
    info: 'Info',
  };
  return labels[severity];
}

function getTypeLabel(type: AlertType): string {
  const labels: Record<AlertType, string> = {
    low_stock: 'Low Stock',
    expiring_soon: 'Expiring Soon',
    expired: 'Expired',
    price_increase: 'Price ↑',
    price_decrease: 'Price ↓',
    out_of_stock: 'Out of Stock',
  };
  return labels[type];
}

function formatTime(dateStr: string): string {
  const date = new Date(dateStr);
  const now = new Date();
  const diffMs = now.getTime() - date.getTime();
  const diffMins = Math.floor(diffMs / 60000);
  const diffHours = Math.floor(diffMs / 3600000);
  const diffDays = Math.floor(diffMs / 86400000);

  if (diffMins < 1) return 'Just now';
  if (diffMins < 60) return `${diffMins}m ago`;
  if (diffHours < 24) return `${diffHours}h ago`;
  if (diffDays < 7) return `${diffDays}d ago`;
  return date.toLocaleDateString();
}

onMounted(() => {
  loadAlerts();
});

// Auto-refresh every 2 minutes
setInterval(() => {
  if (!loading.value) loadAlerts();
}, 120000);
</script>

<style scoped>
.alerts-dashboard {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

.dashboard-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 20px;
}

.dashboard-header h2 {
  margin: 0 0 4px 0;
  color: #2c3e50;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin: 0;
}

.header-actions {
  display: flex;
  gap: 10px;
}

.action-btn {
  padding: 8px 16px;
  background: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.action-btn:hover {
  background: #2980b9;
  transform: translateY(-1px);
}

.action-btn.danger {
  background: #e74c3c;
}

.action-btn.danger:hover {
  background: #c0392b;
}

.filter-bar {
  display: flex;
  gap: 20px;
  padding: 16px;
  background: #f8f9fa;
  border-radius: 8px;
  margin-bottom: 20px;
  flex-wrap: wrap;
}

.filter-group {
  display: flex;
  align-items: center;
  gap: 8px;
}

.filter-group label {
  font-size: 14px;
  font-weight: 600;
  color: #555;
}

.filter-group select {
  padding: 6px 12px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
}

.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
  margin-bottom: 24px;
}

.stat-card {
  padding: 20px;
  border-radius: 8px;
  text-align: center;
  border-left: 4px solid;
}

.stat-card.critical {
  background: #fee;
  border-left-color: #e74c3c;
}

.stat-card.warning {
  background: #fff8e1;
  border-left-color: #f39c12;
}

.stat-card.info {
  background: #e3f2fd;
  border-left-color: #3498db;
}

.stat-card.unread {
  background: #f3e5f5;
  border-left-color: #9c27b0;
}

.stat-value {
  font-size: 32px;
  font-weight: bold;
  color: #2c3e50;
}

.stat-label {
  font-size: 14px;
  color: #666;
  margin-top: 4px;
}

.loading-state {
  text-align: center;
  padding: 60px 20px;
  color: #999;
}

.spinner {
  width: 40px;
  height: 40px;
  border: 4px solid #f3f3f3;
  border-top: 4px solid #3498db;
  border-radius: 50%;
  animation: spin 1s linear infinite;
  margin: 0 auto 16px;
}

@keyframes spin {
  0% { transform: rotate(0deg); }
  100% { transform: rotate(360deg); }
}

.empty-state {
  text-align: center;
  padding: 60px 20px;
}

.empty-icon {
  font-size: 64px;
  margin-bottom: 16px;
}

.empty-state h3 {
  color: #2c3e50;
  margin-bottom: 8px;
}

.empty-state p {
  color: #666;
  font-size: 14px;
}

.alerts-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.alert-card {
  display: flex;
  gap: 16px;
  padding: 16px;
  background: white;
  border: 2px solid #e0e0e0;
  border-left: 4px solid;
  border-radius: 8px;
  transition: all 0.2s;
}

.alert-card:hover {
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.alert-card.critical {
  border-left-color: #e74c3c;
}

.alert-card.warning {
  border-left-color: #f39c12;
}

.alert-card.info {
  border-left-color: #3498db;
}

.alert-card.unread {
  background: #f8f9ff;
  border-color: #3498db;
}

.alert-card.dismissed {
  opacity: 0.6;
}

.alert-icon {
  font-size: 32px;
  flex-shrink: 0;
}

.alert-content {
  flex: 1;
}

.alert-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 8px;
  gap: 12px;
}

.alert-header h4 {
  margin: 0;
  color: #2c3e50;
  font-size: 16px;
}

.alert-badges {
  display: flex;
  gap: 6px;
  flex-shrink: 0;
}

.badge {
  padding: 3px 8px;
  border-radius: 4px;
  font-size: 11px;
  font-weight: 600;
  text-transform: uppercase;
}

.badge.severity.critical {
  background: #fee;
  color: #c0392b;
}

.badge.severity.warning {
  background: #fff8e1;
  color: #d68910;
}

.badge.severity.info {
  background: #e3f2fd;
  color: #1976d2;
}

.badge.type {
  background: #f5f5f5;
  color: #666;
}

.alert-message {
  margin: 0 0 12px 0;
  color: #555;
  font-size: 14px;
  line-height: 1.5;
}

.alert-meta {
  display: flex;
  gap: 16px;
  font-size: 12px;
  color: #999;
}

.alert-actions {
  display: flex;
  flex-direction: column;
  gap: 8px;
  flex-shrink: 0;
}

.icon-btn {
  width: 32px;
  height: 32px;
  padding: 0;
  background: #f5f5f5;
  border: 1px solid #ddd;
  border-radius: 4px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 16px;
}

.icon-btn:hover {
  background: #3498db;
  color: white;
  border-color: #3498db;
}

@media (max-width: 768px) {
  .filter-bar {
    flex-direction: column;
  }

  .stats-grid {
    grid-template-columns: repeat(2, 1fr);
  }

  .alert-card {
    flex-direction: column;
  }

  .alert-actions {
    flex-direction: row;
    justify-content: flex-end;
  }
}
</style>
