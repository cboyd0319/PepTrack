<template>
  <div class="dashboard">
    <div class="dashboard-header">
      <h1>🏠 Dashboard</h1>
      <p class="subtitle">Welcome back! Here's your overview.</p>
    </div>

    <!-- Quick Actions -->
    <div class="quick-actions">
      <h2>⚡ Quick Actions</h2>
      <div class="action-buttons">
        <button @click="handleQuickLogDose" class="action-btn primary">
          <span class="icon">💉</span>
          <span class="label">Log Dose</span>
        </button>
        <button @click="navigateToTab('protocols')" class="action-btn">
          <span class="icon">🧪</span>
          <span class="label">New Protocol</span>
        </button>
        <button @click="navigateToTab('research')" class="action-btn">
          <span class="icon">📚</span>
          <span class="label">Research</span>
        </button>
        <button @click="handleQuickBackup" class="action-btn">
          <span class="icon">💾</span>
          <span class="label">Quick Backup</span>
        </button>
      </div>
    </div>

    <!-- Stats Overview -->
    <div class="stats-grid">
      <div class="stat-card">
        <div class="stat-icon">🧪</div>
        <div class="stat-info">
          <div class="stat-label">Active Protocols</div>
          <div class="stat-value">{{ protocolCount }}</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">💉</div>
        <div class="stat-info">
          <div class="stat-label">Doses This Week</div>
          <div class="stat-value">{{ dosesThisWeekCount }}</div>
        </div>
      </div>
      <div class="stat-card">
        <div class="stat-icon">📦</div>
        <div class="stat-info">
          <div class="stat-label">Inventory Items</div>
          <div class="stat-value">{{ inventoryCount }}</div>
        </div>
      </div>
      <div class="stat-card" :class="{ warning: expiringSoonCount > 0 }">
        <div class="stat-icon">⚠️</div>
        <div class="stat-info">
          <div class="stat-label">Expiring Soon</div>
          <div class="stat-value">{{ expiringSoonCount }}</div>
        </div>
      </div>
    </div>

    <!-- Main Content Grid -->
    <div class="content-grid">
      <!-- Recent Doses -->
      <div class="card">
        <div class="card-header">
          <h3>📊 Recent Doses</h3>
          <button @click="navigateToTab('doses')" class="link-btn">View All →</button>
        </div>
        <div class="card-content">
          <div v-if="recentDoses.length === 0" class="empty-state">
            <p>No doses logged yet</p>
            <button @click="handleQuickLogDose" class="btn-secondary">Log Your First Dose</button>
          </div>
          <div v-else class="dose-list">
            <div v-for="dose in recentDoses.slice(0, 5)" :key="dose.id" class="dose-item">
              <div class="dose-protocol">{{ getProtocolName(dose.protocol_id) }}</div>
              <div class="dose-details">
                <span class="dose-amount">{{ dose.amount_mg }}mg</span>
                <span class="dose-site">{{ dose.site }}</span>
                <span class="dose-date">{{ formatDate(dose.logged_at) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Active Protocols -->
      <div class="card">
        <div class="card-header">
          <h3>🧪 Active Protocols</h3>
          <button @click="navigateToTab('protocols')" class="link-btn">Manage →</button>
        </div>
        <div class="card-content">
          <div v-if="protocols.length === 0" class="empty-state">
            <p>No protocols created yet</p>
            <button @click="navigateToTab('protocols')" class="btn-secondary">Create Protocol</button>
          </div>
          <div v-else class="protocol-list">
            <div v-for="protocol in protocols.slice(0, 5)" :key="protocol.id" class="protocol-item">
              <div class="protocol-name">{{ protocol.name }}</div>
              <div class="protocol-peptide">{{ protocol.peptide_name }}</div>
            </div>
          </div>
        </div>
      </div>

      <!-- Inventory Alerts -->
      <div class="card" v-if="expiringSoonItems.length > 0">
        <div class="card-header alert">
          <h3>⚠️ Inventory Alerts</h3>
          <button @click="navigateToTab('operations')" class="link-btn">View All →</button>
        </div>
        <div class="card-content">
          <div class="alert-list">
            <div v-for="item in expiringSoonItems.slice(0, 3)" :key="item.id" class="alert-item">
              <div class="alert-icon">⏰</div>
              <div class="alert-info">
                <div class="alert-title">Vial #{{ item.vial_number || 'Unknown' }}</div>
                <div class="alert-message">Expires {{ formatExpiryDate(item.expiry_date) }}</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Backup Status -->
      <div class="card">
        <div class="card-header">
          <h3>💾 Backup Status</h3>
          <button @click="navigateToTab('settings')" class="link-btn">Configure →</button>
        </div>
        <div class="card-content">
          <div class="backup-status">
            <div class="status-item">
              <span class="status-label">Last Backup:</span>
              <span class="status-value">{{ lastBackupTime || 'Never' }}</span>
            </div>
            <div class="status-item">
              <span class="status-label">Cloud Sync:</span>
              <span class="status-value" :class="{ connected: cloudConnected }">
                {{ cloudConnected ? '✅ Connected' : '⚠️ Not Connected' }}
              </span>
            </div>
            <button @click="handleQuickBackup" class="btn-primary backup-btn">
              Create Backup Now
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue';
import { useProtocolStore } from '../stores/protocols';
import { useDoseStore } from '../stores/doses';
import { useSupplierStore } from '../stores/suppliers';
import type { PeptideProtocol } from '../api/peptrack';

const emit = defineEmits<{
  navigateToTab: [tab: string];
  quickLogDose: [];
  quickBackup: [];
}>();

// Stores
const protocolStore = useProtocolStore();
const doseStore = useDoseStore();
const supplierStore = useSupplierStore();

// Data
const cloudConnected = ref(false);
const lastBackupTime = ref<string | null>(null);

// Computed
const protocols = computed(() => protocolStore.protocols);
const protocolCount = computed(() => protocolStore.protocolCount);
const recentDoses = computed(() => doseStore.recentDoses);
const dosesThisWeekCount = computed(() => doseStore.dosesThisWeek.length);
const inventoryCount = computed(() => supplierStore.inventoryCount);
const expiringSoonItems = computed(() => supplierStore.expiringSoonInventory);
const expiringSoonCount = computed(() => expiringSoonItems.value.length);

// Methods
function navigateToTab(tab: string) {
  emit('navigateToTab', tab);
}

function handleQuickLogDose() {
  emit('quickLogDose');
}

function handleQuickBackup() {
  emit('quickBackup');
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find((p: PeptideProtocol) => p.id === protocolId);
  return protocol?.name || 'Unknown Protocol';
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = now.getTime() - date.getTime();
    const diffHours = Math.floor(diffMs / (1000 * 60 * 60));
    const diffDays = Math.floor(diffHours / 24);

    if (diffHours < 1) return 'Just now';
    if (diffHours < 24) return `${diffHours}h ago`;
    if (diffDays < 7) return `${diffDays}d ago`;

    return date.toLocaleDateString();
  } catch {
    return 'Unknown';
  }
}

function formatExpiryDate(dateStr?: string | null): string {
  if (!dateStr) return 'Unknown';
  try {
    const date = new Date(dateStr);
    const now = new Date();
    const diffMs = date.getTime() - now.getTime();
    const diffDays = Math.ceil(diffMs / (1000 * 60 * 60 * 24));

    if (diffDays < 0) return 'Expired';
    if (diffDays === 0) return 'Today';
    if (diffDays === 1) return 'Tomorrow';
    if (diffDays < 7) return `in ${diffDays} days`;
    if (diffDays < 30) return `in ${Math.ceil(diffDays / 7)} weeks`;

    return date.toLocaleDateString();
  } catch {
    return 'Unknown';
  }
}

async function loadData() {
  try {
    await Promise.all([
      protocolStore.fetchProtocols(),
      doseStore.fetchDoses(),
      supplierStore.fetchInventory(),
    ]);
  } catch (error) {
    // Errors are handled by individual stores
  }
}

onMounted(() => {
  loadData();
});
</script>

<style scoped>
.dashboard {
  padding: 24px;
  max-width: 1400px;
  margin: 0 auto;
}

.dashboard-header {
  margin-bottom: 32px;
}

.dashboard-header h1 {
  font-size: 32px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.subtitle {
  font-size: 16px;
  color: #666;
  margin: 0;
}

/* Quick Actions */
.quick-actions {
  margin-bottom: 32px;
}

.quick-actions h2 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: #1a1a1a;
}

.action-buttons {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 12px;
}

.action-btn {
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: 8px;
  padding: 20px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 600;
}

.action-btn:hover {
  border-color: #1976d2;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.1);
}

.action-btn.primary {
  background: #1976d2;
  color: white;
  border-color: #1976d2;
}

.action-btn.primary:hover {
  background: #1565c0;
}

.action-btn .icon {
  font-size: 32px;
}

/* Stats Grid */
.stats-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  gap: 16px;
  margin-bottom: 32px;
}

.stat-card {
  background: white;
  padding: 20px;
  border-radius: 12px;
  border: 2px solid #e0e0e0;
  display: flex;
  align-items: center;
  gap: 16px;
}

.stat-card.warning {
  border-color: #ff9800;
  background: #fff3e0;
}

.stat-icon {
  font-size: 40px;
  flex-shrink: 0;
}

.stat-info {
  flex: 1;
}

.stat-label {
  font-size: 13px;
  color: #666;
  margin-bottom: 4px;
}

.stat-value {
  font-size: 28px;
  font-weight: 700;
  color: #1a1a1a;
}

/* Content Grid */
.content-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(350px, 1fr));
  gap: 24px;
}

.card {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  overflow: hidden;
}

.card-header {
  padding: 16px 20px;
  border-bottom: 2px solid #e0e0e0;
  display: flex;
  justify-content: space-between;
  align-items: center;
}

.card-header.alert {
  background: #fff3e0;
  border-bottom-color: #ff9800;
}

.card-header h3 {
  font-size: 18px;
  font-weight: 600;
  margin: 0;
  color: #1a1a1a;
}

.link-btn {
  background: none;
  border: none;
  color: #1976d2;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  padding: 4px 8px;
  border-radius: 4px;
  transition: background 0.2s;
}

.link-btn:hover {
  background: #f0f0f0;
}

.card-content {
  padding: 20px;
}

.empty-state {
  text-align: center;
  padding: 40px 20px;
  color: #999;
}

.empty-state p {
  margin: 0 0 16px 0;
}

/* Dose List */
.dose-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dose-item {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
  border-left: 4px solid #1976d2;
}

.dose-protocol {
  font-weight: 600;
  margin-bottom: 4px;
  color: #1a1a1a;
}

.dose-details {
  display: flex;
  gap: 12px;
  font-size: 13px;
  color: #666;
}

.dose-amount {
  font-weight: 600;
  color: #1976d2;
}

/* Protocol List */
.protocol-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.protocol-item {
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
  border-left: 4px solid #4caf50;
}

.protocol-name {
  font-weight: 600;
  margin-bottom: 4px;
  color: #1a1a1a;
}

.protocol-peptide {
  font-size: 13px;
  color: #666;
}

/* Alert List */
.alert-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.alert-item {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  background: #fff3e0;
  border-radius: 8px;
  border-left: 4px solid #ff9800;
}

.alert-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.alert-title {
  font-weight: 600;
  margin-bottom: 2px;
  color: #1a1a1a;
}

.alert-message {
  font-size: 13px;
  color: #666;
}

/* Backup Status */
.backup-status {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-item {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 12px;
  background: #f5f5f5;
  border-radius: 8px;
}

.status-label {
  font-weight: 600;
  color: #666;
}

.status-value {
  color: #1a1a1a;
}

.status-value.connected {
  color: #4caf50;
}

.backup-btn {
  width: 100%;
  margin-top: 8px;
}

/* Buttons */
.btn-primary,
.btn-secondary {
  padding: 10px 20px;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  border: none;
  font-size: 14px;
  transition: all 0.2s;
}

.btn-primary {
  background: #1976d2;
  color: white;
}

.btn-primary:hover {
  background: #1565c0;
}

.btn-secondary {
  background: #f5f5f5;
  color: #333;
}

.btn-secondary:hover {
  background: #e0e0e0;
}

/* Dark Mode */
@media (prefers-color-scheme: dark) {
  .dashboard-header h1,
  .stat-value,
  .card-header h3,
  .dose-protocol,
  .protocol-name,
  .alert-title,
  .status-value {
    color: #fff;
  }

  .subtitle,
  .stat-label,
  .dose-details,
  .protocol-peptide,
  .alert-message,
  .status-label {
    color: #aaa;
  }

  .stat-card,
  .card,
  .action-btn {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }

  .dose-item,
  .protocol-item,
  .status-item {
    background: #1a1a1a;
  }

  .btn-secondary {
    background: #3a3a3a;
    color: #fff;
  }

  .btn-secondary:hover {
    background: #4a4a4a;
  }
}
</style>
