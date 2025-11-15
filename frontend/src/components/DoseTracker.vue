<template>
  <div class="dose-tracker">
    <h2>💉 Track Your Doses</h2>
    <p class="subtitle">Log doses and set up recurring schedules</p>

    <!-- Tab Navigation -->
    <div class="dose-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'log' }]"
        @click="activeTab = 'log'"
      >
        📝 Log Dose
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'schedules' }]"
        @click="activeTab = 'schedules'"
      >
        ⏰ Schedules
      </button>
    </div>

    <!-- Duplicate Dose Warning Modal -->
    <div v-if="showDuplicateWarning" class="modal-overlay" @click="cancelDuplicateDose">
      <div class="modal-content" @click.stop>
        <div class="modal-header">
          <h3>⚠️ Duplicate Dose Detected</h3>
        </div>
        <div class="modal-body">
          <p>
            You logged a dose for <strong>{{ getDuplicateProtocolName() }}</strong>
            <strong>{{ getDuplicateTimeAgo() }}</strong> ago.
          </p>
          <p class="warning-text">
            Last dose: <strong>{{ duplicateInfo.lastAmount }} mg</strong>
            at {{ formatDate(duplicateInfo.lastTime) }}
          </p>
          <p>Are you sure you want to log another dose so soon?</p>
        </div>
        <div class="modal-footer">
          <button @click="cancelDuplicateDose" class="btn-secondary">
            ❌ Cancel
          </button>
          <button @click="confirmDuplicateDose" class="btn-primary">
            ✅ Yes, Log Dose
          </button>
        </div>
      </div>
    </div>

    <!-- Log Dose Tab -->
    <div v-show="activeTab === 'log'" class="tab-content">
      <!-- Log New Dose Form -->
    <div class="log-dose-section panel">
      <h3>➕ Log a Dose</h3>
      <div v-if="!hasProtocols" class="empty-state">
        <p>
          You need at least one peptide plan before logging doses.
          Add a plan in the <strong>Protocols</strong> tab, then come back here.
        </p>
      </div>
      <form @submit.prevent="handleLogDose" class="dose-form">
        <label for="dose-protocol-select">
          Which Peptide Plan?
          <select
            id="dose-protocol-select"
            v-model="form.protocolId"
            @change="onProtocolChange"
            required
            aria-label="Select peptide protocol"
            :disabled="!hasProtocols"
          >
            <option value="">Select a plan...</option>
            <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
              {{ protocol.name }} ({{ protocol.peptide_name }})
            </option>
          </select>
        </label>

        <!-- Recent Doses Preview -->
        <div v-if="recentProtocolDoses.length > 0" class="recent-doses-preview">
          <div class="recent-doses-header">
            <span class="recent-doses-label">📊 Recent doses for this protocol:</span>
            <button
              type="button"
              @click="useLastDoseAsTemplate"
              class="use-last-btn"
              title="Fill form with last dose details"
            >
              ↻ Use Last Dose
            </button>
          </div>
          <div class="recent-doses-list">
            <div v-for="(dose, index) in recentProtocolDoses.slice(0, 3)" :key="dose.id" class="recent-dose-item">
              <span class="recent-dose-amount">{{ dose.amount_mg }} mg</span>
              <span class="recent-dose-site">@ {{ dose.site }}</span>
              <span class="recent-dose-time">{{ formatDate(dose.logged_at) }}</span>
            </div>
          </div>
        </div>

        <div class="form-row">
          <label for="dose-amount-input">
            Amount (mg)
          </label>
          <input
            id="dose-amount-input"
            v-model.number="form.amountMg"
            type="number"
            step="0.01"
            min="0"
            placeholder="e.g., 0.5"
            required
            aria-label="Dose amount in milligrams"
            autocomplete="off"
          />

          <label for="dose-site-input">
            Injection Site
          </label>
          <input
            id="dose-site-input"
            v-model="form.site"
            type="text"
            placeholder="e.g., Left Abdomen, Right Thigh"
            required
            aria-label="Injection site location"
            autocomplete="off"
            list="injection-sites"
          />
          <datalist id="injection-sites">
            <option value="Left Abdomen">
            <option value="Right Abdomen">
            <option value="Left Thigh">
            <option value="Right Thigh">
            <option value="Left Deltoid">
            <option value="Right Deltoid">
            <option value="Left Glute">
            <option value="Right Glute">
          </datalist>
        </div>

        <label for="dose-notes-input">
          Notes (optional)
        </label>
        <textarea
          id="dose-notes-input"
          v-model="form.notes"
          rows="2"
          placeholder="How you're feeling, any side effects, etc."
          aria-label="Additional notes about dose"
        />

        <button
          type="submit"
          :disabled="isLogging || !hasProtocols"
          class="primary-btn"
          aria-label="Log dose entry"
          :aria-busy="isLogging"
        >
          {{ isLogging ? '⏳ Logging...' : '💾 Save Dose' }}
        </button>
      </form>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-message">
      ⚠️ {{ error }}
    </div>

    <!-- Success Message -->
    <div v-if="successMessage" class="success-message">
      ✅ {{ successMessage }}
    </div>

    <!-- Dose History -->
    <div class="history-section">
      <div class="history-header">
        <h3>📊 Your Dose History</h3>
        <div class="history-controls">
          <label for="filter-protocol-select">
            Filter by plan:
          </label>
          <select
            id="filter-protocol-select"
            v-model="filterProtocolId"
            @change="loadDoses"
            aria-label="Filter doses by protocol"
          >
            <option value="">All Plans</option>
            <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
              {{ protocol.name }}
            </option>
          </select>
          <button
            @click="loadDoses"
            class="refresh-btn"
            aria-label="Refresh dose history"
          >↻ Refresh</button>
        </div>
      </div>

      <div v-if="doses.length === 0" class="no-doses">
        No doses logged yet. Log your first dose above!
      </div>

      <div v-else class="dose-list">
        <div v-for="dose in doses" :key="dose.id" class="dose-card">
          <div class="dose-header">
            <div class="dose-info">
              <strong>{{ getProtocolName(dose.protocol_id) }}</strong>
              <span class="dose-amount">{{ dose.amount_mg }} mg</span>
            </div>
            <button
              @click="deleteDose(dose.id)"
              class="delete-btn"
              :aria-label="`Delete dose from ${formatDate(dose.logged_at)}`"
            >
              🗑️
            </button>
          </div>

          <div class="dose-details">
            <div class="dose-site">
              📍 {{ dose.site }}
            </div>
            <div class="dose-time">
              🕐 {{ formatDate(dose.logged_at) }}
            </div>
          </div>

          <p v-if="dose.notes" class="dose-notes">
            📝 {{ dose.notes }}
          </p>
        </div>
      </div>
    </div>
    </div>

    <!-- Schedules Tab -->
    <div v-show="activeTab === 'schedules'" class="tab-content">
      <DoseScheduleManager />
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, computed } from 'vue';
import DoseScheduleManager from './DoseScheduleManager.vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  logDose,
  listDoseLogs,
  listDoseLogsForProtocol,
  deleteDoseLog,
  listProtocols,
  type DoseLog,
  type LogDosePayload,
  type PeptideProtocol,
} from '../api/peptrack';
import { formatDate as formatDateUtil } from '../utils/dateFormatter';

// State
const activeTab = ref<'log' | 'schedules'>('log');
const protocols = ref<PeptideProtocol[]>([]);
const doses = ref<DoseLog[]>([]);
const filterProtocolId = ref('');
const isLogging = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);
const hasProtocols = computed(() => protocols.value.length > 0);

// Duplicate detection state
const showDuplicateWarning = ref(false);
const duplicateInfo = ref<{
  lastTime: string;
  lastAmount: number;
  protocolId: string;
}>({
  lastTime: '',
  lastAmount: 0,
  protocolId: '',
});
const DUPLICATE_THRESHOLD_HOURS = 2; // Configurable threshold

// Form state
const form = ref<LogDosePayload>({
  protocolId: '',
  site: '',
  amountMg: 0,
  notes: '',
});

// Computed: Recent doses for selected protocol
const recentProtocolDoses = computed(() => {
  if (!form.value.protocolId) return [];

  return doses.value
    .filter(d => d.protocol_id === form.value.protocolId)
    .sort((a, b) => new Date(b.logged_at).getTime() - new Date(a.logged_at).getTime())
    .slice(0, 5); // Get last 5 doses
});

onMounted(async () => {
  await loadProtocols();
  await loadDoses();
});

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load protocols' });
  }
}

async function loadDoses() {
  try {
    error.value = null;
    if (filterProtocolId.value) {
      doses.value = await listDoseLogsForProtocol(filterProtocolId.value);
    } else {
      doses.value = await listDoseLogs();
    }
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load dose history' });
  }
}

/**
 * Handle protocol selection change - auto-fill with smart defaults
 */
function onProtocolChange() {
  if (!form.value.protocolId) return;

  // Auto-fill with last dose details for this protocol
  const lastDose = recentProtocolDoses.value[0];
  if (lastDose) {
    // Only auto-fill if the fields are currently empty
    if (!form.value.site) {
      form.value.site = lastDose.site;
    }
    if (form.value.amountMg === 0 || !form.value.amountMg) {
      form.value.amountMg = lastDose.amount_mg;
    }
  }
}

/**
 * Use last dose as template - fill all fields
 */
function useLastDoseAsTemplate() {
  const lastDose = recentProtocolDoses.value[0];
  if (!lastDose) return;

  form.value.site = lastDose.site;
  form.value.amountMg = lastDose.amount_mg;
  // Don't copy notes, as they're usually specific to that dose
}

/**
 * Check if a dose for the same protocol was logged recently
 */
function checkForDuplicateDose(): boolean {
  if (!form.value.protocolId) return false;

  // Get all doses for this protocol
  const protocolDoses = doses.value.filter(d => d.protocol_id === form.value.protocolId);

  if (protocolDoses.length === 0) return false;

  // Sort by date (most recent first)
  const sortedDoses = protocolDoses.sort(
    (a, b) => new Date(b.logged_at).getTime() - new Date(a.logged_at).getTime()
  );

  const lastDose = sortedDoses[0];
  const lastDoseTime = new Date(lastDose.logged_at);
  const now = new Date();
  const hoursSinceLastDose = (now.getTime() - lastDoseTime.getTime()) / (1000 * 60 * 60);

  // Check if within threshold
  if (hoursSinceLastDose < DUPLICATE_THRESHOLD_HOURS) {
    duplicateInfo.value = {
      lastTime: lastDose.logged_at,
      lastAmount: lastDose.amount_mg,
      protocolId: lastDose.protocol_id,
    };
    return true;
  }

  return false;
}

/**
 * Get protocol name for duplicate warning
 */
function getDuplicateProtocolName(): string {
  return getProtocolName(duplicateInfo.value.protocolId);
}

/**
 * Get human-readable time since last dose
 */
function getDuplicateTimeAgo(): string {
  const lastDoseTime = new Date(duplicateInfo.value.lastTime);
  const now = new Date();
  const minutesAgo = Math.floor((now.getTime() - lastDoseTime.getTime()) / (1000 * 60));

  if (minutesAgo < 60) {
    return `${minutesAgo} minute${minutesAgo !== 1 ? 's' : ''}`;
  }

  const hoursAgo = Math.floor(minutesAgo / 60);
  const remainingMinutes = minutesAgo % 60;

  if (remainingMinutes === 0) {
    return `${hoursAgo} hour${hoursAgo !== 1 ? 's' : ''}`;
  }

  return `${hoursAgo}h ${remainingMinutes}m`;
}

/**
 * Cancel duplicate dose - close modal
 */
function cancelDuplicateDose() {
  showDuplicateWarning.value = false;
  duplicateInfo.value = {
    lastTime: '',
    lastAmount: 0,
    protocolId: '',
  };
}

/**
 * Confirm duplicate dose - proceed with logging
 */
async function confirmDuplicateDose() {
  showDuplicateWarning.value = false;
  await performLogDose(true); // true = skip duplicate check
}

/**
 * Handle log dose - with duplicate detection
 */
async function handleLogDose() {
  if (!form.value.protocolId || !form.value.site || form.value.amountMg <= 0 || isNaN(form.value.amountMg)) {
    error.value = 'Please fill in all required fields with valid values.';
    return;
  }

  // Check for duplicate
  if (checkForDuplicateDose()) {
    showDuplicateWarning.value = true;
    return;
  }

  // No duplicate detected, proceed
  await performLogDose(false);
}

/**
 * Actually log the dose to the database
 */
async function performLogDose(skipDuplicateCheck: boolean) {
  isLogging.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    await logDose(form.value);
    showSuccessToast('Success', 'Dose logged successfully!');

    // Reset form
    form.value = {
      protocolId: '',
      site: '',
      amountMg: 0,
      notes: '',
    };

    // Reload doses
    await loadDoses();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'log dose' });
  } finally {
    isLogging.value = false;
  }
}

async function deleteDose(logId: string) {
  if (!confirm('Are you sure you want to delete this dose entry?')) {
    return;
  }

  try {
    await deleteDoseLog(logId);
    await loadDoses();
    showSuccessToast('Success', 'Dose deleted successfully');
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'delete dose' });
  }
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find((p) => p.id === protocolId);
  return protocol ? `${protocol.name} (${protocol.peptide_name})` : 'Unknown Protocol';
}

function formatDate(dateStr: any): string {
  return formatDateUtil(dateStr);
}
</script>

<style scoped>
.dose-tracker {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

/* Tab Navigation */
.dose-tabs {
  display: flex;
  gap: 8px;
  margin: 20px 0;
  padding: 4px;
  background: #f5f5f5;
  border-radius: 10px;
}

.tab-btn {
  flex: 1;
  padding: 12px 20px;
  border: none;
  background: transparent;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
}

.tab-btn:hover {
  background: rgba(102, 126, 234, 0.1);
  color: #667eea;
}

.tab-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.3);
}

.tab-content {
  animation: fadeIn 0.3s ease;
}

@keyframes fadeIn {
  from {
    opacity: 0;
    transform: translateY(10px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
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

.log-dose-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #2c3e50;
}

.empty-state {
  margin-bottom: 15px;
  padding: 12px;
  border-radius: 6px;
  background: #fff3cd;
  border-left: 4px solid #f1c40f;
  color: #5c4400;
  font-size: 14px;
  line-height: 1.4;
}

.dose-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.dose-form label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-weight: 500;
  color: #2c3e50;
}

.dose-form input,
.dose-form select,
.dose-form textarea {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

/* Recent Doses Preview */
.recent-doses-preview {
  background: #e8f5e9;
  border: 1px solid #a5d6a7;
  border-radius: 8px;
  padding: 12px;
  margin: 10px 0;
}

.recent-doses-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 8px;
}

.recent-doses-label {
  font-size: 13px;
  font-weight: 600;
  color: #2e7d32;
}

.use-last-btn {
  padding: 6px 12px;
  background: #4caf50;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 12px;
  font-weight: 600;
  transition: background 0.2s;
}

.use-last-btn:hover {
  background: #388e3c;
}

.recent-doses-list {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.recent-dose-item {
  display: flex;
  gap: 10px;
  align-items: center;
  font-size: 13px;
  padding: 6px 10px;
  background: white;
  border-radius: 4px;
}

.recent-dose-amount {
  font-weight: 700;
  color: #1976d2;
  min-width: 60px;
}

.recent-dose-site {
  color: #666;
  flex: 1;
}

.recent-dose-time {
  color: #999;
  font-size: 12px;
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 2fr;
  gap: 15px;
}

.primary-btn {
  padding: 12px 24px;
  background-color: #42b983;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
  transition: background-color 0.2s;
}

.primary-btn:hover:not(:disabled) {
  background-color: #359268;
}

.primary-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.error-message {
  padding: 12px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 6px;
  color: #c33;
  margin-bottom: 15px;
}

.success-message {
  padding: 12px;
  background-color: #efe;
  border: 1px solid #cfc;
  border-radius: 6px;
  color: #3a3;
  margin-bottom: 15px;
}

.history-section {
  margin-top: 30px;
}

.history-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
  flex-wrap: wrap;
  gap: 10px;
}

.history-header h3 {
  margin: 0;
  color: #2c3e50;
}

.history-controls {
  display: flex;
  gap: 10px;
  align-items: center;
  flex-wrap: wrap;
}

.history-controls label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #666;
}

.history-controls select {
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.refresh-btn {
  padding: 6px 12px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.refresh-btn:hover {
  background-color: #2980b9;
}

.no-doses {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
  background: #f9f9f9;
  border-radius: 8px;
}

.dose-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.dose-card {
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
  transition: box-shadow 0.2s;
}

.dose-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.dose-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 10px;
}

.dose-info {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.dose-info strong {
  color: #2c3e50;
  font-size: 16px;
}

.dose-amount {
  color: #42b983;
  font-weight: bold;
  font-size: 18px;
}

.delete-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 18px;
  padding: 4px;
  opacity: 0.6;
  transition: opacity 0.2s;
}

.delete-btn:hover {
  opacity: 1;
}

.dose-details {
  display: flex;
  gap: 20px;
  margin-bottom: 8px;
  font-size: 14px;
  color: #666;
}

.dose-notes {
  margin: 10px 0 0 0;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 4px;
  font-size: 14px;
  color: #555;
  line-height: 1.5;
}

/* Duplicate Warning Modal */
.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  justify-content: center;
  align-items: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

.modal-content {
  background: white;
  border-radius: 12px;
  padding: 0;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 10px 40px rgba(0, 0, 0, 0.3);
  animation: slideUp 0.3s ease;
}

@keyframes slideUp {
  from {
    opacity: 0;
    transform: translateY(20px);
  }
  to {
    opacity: 1;
    transform: translateY(0);
  }
}

.modal-header {
  padding: 20px;
  border-bottom: 1px solid #eee;
}

.modal-header h3 {
  margin: 0;
  color: #e67e22;
  font-size: 20px;
}

.modal-body {
  padding: 20px;
}

.modal-body p {
  margin: 10px 0;
  color: #2c3e50;
  line-height: 1.6;
}

.warning-text {
  background: #fff3cd;
  padding: 12px;
  border-radius: 6px;
  border-left: 4px solid #f1c40f;
  color: #856404;
  font-size: 14px;
}

.modal-footer {
  padding: 15px 20px;
  border-top: 1px solid #eee;
  display: flex;
  gap: 10px;
  justify-content: flex-end;
}

.btn-secondary {
  padding: 10px 20px;
  background: #95a5a6;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: background 0.2s;
}

.btn-secondary:hover {
  background: #7f8c8d;
}

.btn-primary {
  padding: 10px 20px;
  background: #e67e22;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: background 0.2s;
}

.btn-primary:hover {
  background: #d35400;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .history-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .history-controls {
    width: 100%;
    flex-direction: column;
    align-items: stretch;
  }

  .history-controls select {
    width: 100%;
  }

  .modal-content {
    width: 95%;
    margin: 10px;
  }

  .modal-footer {
    flex-direction: column;
  }

  .btn-secondary,
  .btn-primary {
    width: 100%;
  }
}
</style>
