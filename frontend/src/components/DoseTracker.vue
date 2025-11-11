<template>
  <div class="dose-tracker">
    <h2>💉 Track Your Doses</h2>
    <p class="subtitle">Log when you take your peptides</p>

    <!-- Log New Dose Form -->
    <div class="log-dose-section panel">
      <h3>➕ Log a Dose</h3>
      <form @submit.prevent="handleLogDose" class="dose-form">
        <label>
          Which Peptide Plan?
          <select v-model="form.protocolId" required>
            <option value="">Select a plan...</option>
            <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
              {{ protocol.name }} ({{ protocol.peptide_name }})
            </option>
          </select>
        </label>

        <div class="form-row">
          <label>
            Amount (mg)
            <input
              v-model.number="form.amountMg"
              type="number"
              step="0.01"
              min="0"
              placeholder="e.g., 0.5"
              required
            />
          </label>

          <label>
            Injection Site
            <input
              v-model="form.site"
              type="text"
              placeholder="e.g., Left Abdomen, Right Thigh"
              required
            />
          </label>
        </div>

        <label>
          Notes (optional)
          <textarea
            v-model="form.notes"
            rows="2"
            placeholder="How you're feeling, any side effects, etc."
          />
        </label>

        <button type="submit" :disabled="isLogging" class="primary-btn">
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
          <label>
            Filter by plan:
            <select v-model="filterProtocolId" @change="loadDoses">
              <option value="">All Plans</option>
              <option v-for="protocol in protocols" :key="protocol.id" :value="protocol.id">
                {{ protocol.name }}
              </option>
            </select>
          </label>
          <button @click="loadDoses" class="refresh-btn">↻ Refresh</button>
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
            <button @click="deleteDose(dose.id)" class="delete-btn" title="Delete this dose">
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
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
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

// State
const protocols = ref<PeptideProtocol[]>([]);
const doses = ref<DoseLog[]>([]);
const filterProtocolId = ref('');
const isLogging = ref(false);
const error = ref<string | null>(null);
const successMessage = ref<string | null>(null);

// Form state
const form = ref<LogDosePayload>({
  protocolId: '',
  site: '',
  amountMg: 0,
  notes: '',
});

onMounted(async () => {
  await loadProtocols();
  await loadDoses();
});

async function loadProtocols() {
  try {
    protocols.value = await listProtocols();
  } catch (e: any) {
    console.error('Failed to load protocols:', e);
    error.value = 'Could not load your peptide plans. Please try again.';
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
  } catch (e: any) {
    console.error('Failed to load doses:', e);
    error.value = 'Could not load dose history. Please try again.';
  }
}

async function handleLogDose() {
  if (!form.value.protocolId || !form.value.site || form.value.amountMg <= 0) {
    error.value = 'Please fill in all required fields.';
    return;
  }

  isLogging.value = true;
  error.value = null;
  successMessage.value = null;

  try {
    await logDose(form.value);
    successMessage.value = 'Dose logged successfully!';

    // Reset form
    form.value = {
      protocolId: '',
      site: '',
      amountMg: 0,
      notes: '',
    };

    // Reload doses
    await loadDoses();

    // Clear success message after 3 seconds
    setTimeout(() => {
      successMessage.value = null;
    }, 3000);
  } catch (e: any) {
    console.error('Failed to log dose:', e);
    error.value = 'Could not save dose. Please try again.';
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
    successMessage.value = 'Dose deleted.';
    setTimeout(() => {
      successMessage.value = null;
    }, 2000);
  } catch (e: any) {
    console.error('Failed to delete dose:', e);
    error.value = 'Could not delete dose. Please try again.';
  }
}

function getProtocolName(protocolId: string): string {
  const protocol = protocols.value.find((p) => p.id === protocolId);
  return protocol ? `${protocol.name} (${protocol.peptide_name})` : 'Unknown Protocol';
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleString();
  } catch {
    return dateStr;
  }
}
</script>

<style scoped>
.dose-tracker {
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

.log-dose-section h3 {
  margin-top: 0;
  margin-bottom: 15px;
  color: #2c3e50;
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
}
</style>
