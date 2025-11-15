<template>
  <div class="side-effects-tracker">
    <h2>⚠️ Side Effects Tracking</h2>
    <p class="subtitle">Track and monitor adverse reactions and side effects</p>

    <!-- Log New Side Effect Form -->
    <div class="panel log-effect-section">
      <h3>➕ Log Side Effect</h3>
      <form @submit.prevent="handleLogEffect" class="effect-form">
        <div class="form-row">
          <label for="effect-date">
            Date
            <input
              id="effect-date"
              v-model="form.date"
              type="date"
              required
              aria-label="Side effect date"
            />
          </label>

          <label for="severity">
            Severity
            <select
              id="severity"
              v-model="form.severity"
              required
              aria-label="Severity level"
            >
              <option value="">Select severity</option>
              <option value="mild">Mild</option>
              <option value="moderate">Moderate</option>
              <option value="severe">Severe</option>
            </select>
          </label>
        </div>

        <div class="form-row">
          <label for="symptom">
            Symptom
            <input
              id="symptom"
              v-model="form.symptom"
              type="text"
              required
              placeholder="e.g., nausea, headache, redness"
              aria-label="Symptom description"
            />
          </label>

          <label for="duration">
            Duration (minutes)
            <input
              id="duration"
              v-model.number="form.duration_minutes"
              type="number"
              min="0"
              placeholder="e.g., 30"
              aria-label="Duration in minutes"
            />
          </label>
        </div>

        <label for="protocol-select">
          Related Protocol (optional)
          <select
            id="protocol-select"
            v-model="form.protocol_id"
            aria-label="Select related protocol"
          >
            <option :value="null">None</option>
            <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
              {{ protocol.name }} - {{ protocol.peptide_name }}
            </option>
          </select>
        </label>

        <label for="effect-notes">
          Description (optional)
        </label>
        <textarea
          id="effect-notes"
          v-model="form.description"
          rows="3"
          placeholder="Detailed description of the side effect..."
          aria-label="Side effect description"
        />

        <div class="form-checkbox">
          <label>
            <input
              v-model="form.resolved"
              type="checkbox"
              aria-label="Mark as resolved"
            />
            Mark as resolved
          </label>
        </div>

        <button
          type="submit"
          :disabled="isLogging || !form.date || !form.severity || !form.symptom"
          class="primary-btn"
          aria-label="Log side effect"
          :aria-busy="isLogging"
        >
          {{ isLogging ? '⏳ Logging...' : '💾 Save Side Effect' }}
        </button>
      </form>
    </div>

    <!-- Side Effects History -->
    <div class="history-section">
      <div class="history-header">
        <h3>📋 Side Effects History</h3>
        <div class="filter-controls">
          <label for="severity-filter">
            <select id="severity-filter" v-model="severityFilter">
              <option value="">All Severities</option>
              <option value="mild">Mild</option>
              <option value="moderate">Moderate</option>
              <option value="severe">Severe</option>
            </select>
          </label>
          <label>
            <input v-model="showResolved" type="checkbox" />
            Show resolved
          </label>
          <button
            @click="loadEffects"
            class="refresh-btn"
            aria-label="Refresh side effects history"
          >
            ↻ Refresh
          </button>
        </div>
      </div>

      <div v-if="filteredEffects.length === 0" class="no-effects">
        {{ effects.length === 0 ? 'No side effects logged yet.' : 'No side effects match the current filters.' }}
      </div>

      <div v-else class="effects-list">
        <div
          v-for="effect in filteredEffects"
          :key="effect.id"
          class="effect-card"
          :class="{
            'severity-mild': effect.severity === 'mild',
            'severity-moderate': effect.severity === 'moderate',
            'severity-severe': effect.severity === 'severe',
            'resolved': effect.resolved
          }"
        >
          <div class="effect-header">
            <div class="effect-meta">
              <div class="effect-date">
                {{ formatDate(effect.date) }}
              </div>
              <span class="severity-badge" :class="`badge-${effect.severity}`">
                {{ effect.severity.toUpperCase() }}
              </span>
            </div>
            <div class="effect-actions">
              <button
                v-if="!effect.resolved"
                @click="toggleResolved(effect.id, true)"
                class="resolve-btn"
                :aria-label="`Mark ${effect.symptom} as resolved`"
                title="Mark as resolved"
              >
                ✓
              </button>
              <button
                v-else
                @click="toggleResolved(effect.id, false)"
                class="unresolve-btn"
                :aria-label="`Mark ${effect.symptom} as unresolved`"
                title="Mark as unresolved"
              >
                ↺
              </button>
              <button
                @click="deleteEffect(effect.id)"
                class="delete-btn"
                :aria-label="`Delete ${effect.symptom} entry`"
              >
                🗑️
              </button>
            </div>
          </div>

          <div class="effect-body">
            <h4 class="symptom-name">{{ effect.symptom }}</h4>

            <div class="effect-details">
              <div v-if="effect.duration_minutes" class="detail-item">
                <span class="detail-label">Duration:</span>
                <span class="detail-value">{{ formatDuration(effect.duration_minutes) }}</span>
              </div>
              <div v-if="effect.protocol_id" class="detail-item">
                <span class="detail-label">Protocol:</span>
                <span class="detail-value">{{ getProtocolName(effect.protocol_id) }}</span>
              </div>
            </div>

            <p v-if="effect.description" class="effect-description">
              {{ effect.description }}
            </p>
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
  logSideEffect,
  listSideEffects,
  deleteSideEffect,
  toggleSideEffectResolved,
  listProtocols,
  type SideEffect,
  type SideEffectPayload,
  type PeptideProtocol,
} from '../api/peptrack';

// State
const effects = ref<SideEffect[]>([]);
const protocols = ref<PeptideProtocol[]>([]);
const isLogging = ref(false);
const severityFilter = ref<string>('');
const showResolved = ref(true);

// Form state - initialize with today's date
const form = ref<SideEffectPayload & { date: string }>({
  date: new Date().toISOString().split('T')[0], // YYYY-MM-DD format
  protocol_id: null,
  dose_log_id: null,
  severity: '',
  symptom: '',
  description: '',
  duration_minutes: null,
  resolved: false,
});

// Computed
const filteredEffects = computed(() => {
  return effects.value.filter(effect => {
    // Filter by severity
    if (severityFilter.value && effect.severity !== severityFilter.value) {
      return false;
    }
    // Filter by resolved status
    if (!showResolved.value && effect.resolved) {
      return false;
    }
    return true;
  });
});

onMounted(async () => {
  await Promise.all([loadEffects(), loadProtocols()]);
});

async function loadEffects() {
  try {
    effects.value = await listSideEffects();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load side effects' });
  }
}

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load protocols' });
  }
}

async function handleLogEffect() {
  if (!form.value.date || !form.value.severity || !form.value.symptom) {
    showErrorToast(new Error('Please fill in all required fields'), { operation: 'log side effect' });
    return;
  }

  isLogging.value = true;

  try {
    // Convert date to ISO 8601 format
    const payload: SideEffectPayload = {
      protocol_id: form.value.protocol_id || null,
      dose_log_id: form.value.dose_log_id || null,
      date: new Date(form.value.date).toISOString(),
      severity: form.value.severity,
      symptom: form.value.symptom.trim(),
      description: form.value.description?.trim() || null,
      duration_minutes: form.value.duration_minutes || null,
      resolved: form.value.resolved || false,
    };

    await logSideEffect(payload);
    showSuccessToast('Success', 'Side effect logged successfully!');

    // Reset form to today's date
    form.value = {
      date: new Date().toISOString().split('T')[0],
      protocol_id: null,
      dose_log_id: null,
      severity: '',
      symptom: '',
      description: '',
      duration_minutes: null,
      resolved: false,
    };

    // Reload effects
    await loadEffects();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'log side effect' });
  } finally {
    isLogging.value = false;
  }
}

async function toggleResolved(effectId: string, resolved: boolean) {
  try {
    await toggleSideEffectResolved(effectId, resolved);
    await loadEffects();
    showSuccessToast('Success', resolved ? 'Side effect marked as resolved' : 'Side effect marked as unresolved');
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'toggle side effect status' });
  }
}

async function deleteEffect(effectId: string) {
  if (!confirm('Are you sure you want to delete this side effect entry?')) {
    return;
  }

  try {
    await deleteSideEffect(effectId);
    await loadEffects();
    showSuccessToast('Success', 'Side effect deleted successfully');
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'delete side effect' });
  }
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString(undefined, {
      year: 'numeric',
      month: 'long',
      day: 'numeric',
    });
  } catch {
    return dateStr;
  }
}

function formatDuration(minutes: number): string {
  if (minutes < 60) {
    return `${minutes} min`;
  }
  const hours = Math.floor(minutes / 60);
  const mins = minutes % 60;
  return mins > 0 ? `${hours}h ${mins}min` : `${hours}h`;
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find(p => p.id === protocolId);
  return protocol ? `${protocol.name} (${protocol.peptide_name})` : 'Unknown';
}
</script>

<style scoped>
.side-effects-tracker {
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

.log-effect-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #2c3e50;
}

.effect-form {
  display: flex;
  flex-direction: column;
  gap: 15px;
}

.effect-form label {
  display: flex;
  flex-direction: column;
  gap: 5px;
  font-weight: 500;
  color: #2c3e50;
}

.effect-form input,
.effect-form select,
.effect-form textarea {
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.effect-form input:focus,
.effect-form select:focus,
.effect-form textarea:focus {
  outline: none;
  border-color: #e74c3c;
  box-shadow: 0 0 0 3px rgba(231, 76, 60, 0.1);
}

.form-row {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 15px;
}

.form-checkbox {
  display: flex;
  align-items: center;
  gap: 8px;
}

.form-checkbox label {
  flex-direction: row;
  align-items: center;
  gap: 8px;
  font-weight: 500;
  cursor: pointer;
}

.form-checkbox input[type="checkbox"] {
  width: auto;
  cursor: pointer;
}

.primary-btn {
  padding: 12px 24px;
  background-color: #e74c3c;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
  transition: background-color 0.2s;
}

.primary-btn:hover:not(:disabled) {
  background-color: #c0392b;
}

.primary-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
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

.filter-controls {
  display: flex;
  align-items: center;
  gap: 12px;
  flex-wrap: wrap;
}

.filter-controls label {
  display: flex;
  align-items: center;
  gap: 6px;
  font-size: 14px;
}

.filter-controls select {
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  font-size: 14px;
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

.no-effects {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
  background: #f9f9f9;
  border-radius: 8px;
}

.effects-list {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.effect-card {
  background: white;
  border: 1px solid #ddd;
  border-left: 4px solid #95a5a6;
  border-radius: 8px;
  padding: 15px;
  transition: all 0.2s;
}

.effect-card.severity-mild {
  border-left-color: #f39c12;
}

.effect-card.severity-moderate {
  border-left-color: #e67e22;
}

.effect-card.severity-severe {
  border-left-color: #e74c3c;
}

.effect-card.resolved {
  opacity: 0.7;
  background: #f8f9fa;
}

.effect-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.effect-header {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  margin-bottom: 12px;
}

.effect-meta {
  display: flex;
  flex-direction: column;
  gap: 6px;
}

.effect-date {
  font-size: 14px;
  font-weight: 600;
  color: #2c3e50;
}

.severity-badge {
  display: inline-block;
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 11px;
  font-weight: bold;
  letter-spacing: 0.5px;
}

.badge-mild {
  background-color: #fff3cd;
  color: #856404;
}

.badge-moderate {
  background-color: #ffe5d0;
  color: #d35400;
}

.badge-severe {
  background-color: #f8d7da;
  color: #c0392b;
}

.effect-actions {
  display: flex;
  gap: 8px;
}

.resolve-btn,
.unresolve-btn,
.delete-btn {
  background: transparent;
  border: none;
  cursor: pointer;
  font-size: 18px;
  padding: 4px 8px;
  opacity: 0.6;
  transition: all 0.2s;
  border-radius: 4px;
}

.resolve-btn:hover {
  opacity: 1;
  background-color: #d4edda;
}

.unresolve-btn:hover {
  opacity: 1;
  background-color: #fff3cd;
}

.delete-btn:hover {
  opacity: 1;
  background-color: #f8d7da;
}

.effect-body {
  display: flex;
  flex-direction: column;
  gap: 10px;
}

.symptom-name {
  margin: 0;
  font-size: 18px;
  font-weight: 600;
  color: #e74c3c;
}

.effect-details {
  display: flex;
  flex-wrap: wrap;
  gap: 15px;
}

.detail-item {
  display: flex;
  gap: 6px;
  font-size: 14px;
}

.detail-label {
  font-weight: 600;
  color: #666;
}

.detail-value {
  color: #2c3e50;
}

.effect-description {
  margin: 0;
  padding: 10px;
  background: #f9f9f9;
  border-radius: 4px;
  font-size: 14px;
  color: #555;
  line-height: 1.5;
}

@media (max-width: 768px) {
  .form-row {
    grid-template-columns: 1fr;
  }

  .history-header {
    flex-direction: column;
    align-items: flex-start;
  }

  .filter-controls {
    width: 100%;
    justify-content: space-between;
  }

  .refresh-btn {
    width: 100%;
  }
}
</style>
