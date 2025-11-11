<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  getBackupSchedule,
  updateBackupSchedule,
  triggerManualBackup,
  getBackupHistory,
  getBackupProgress,
  type BackupSchedule,
  type BackupFrequency,
  type BackupDestination,
  type BackupHistoryEntry,
  type BackupProgress,
  type CleanupSettings,
} from "../api/peptrack";

const schedule = ref<BackupSchedule>({
  enabled: false,
  frequency: "Manual",
  destinations: ["Local"],
  lastBackup: null,
  nextBackup: null,
  backupOnClose: false,
  compress: true,
  cleanupSettings: {
    keepLastN: 10,
    olderThanDays: null,
  },
  maxRetries: 3,
});

const loading = ref(false);
const saving = ref(false);
const triggering = ref(false);
const message = ref<string | null>(null);
const error = ref<string | null>(null);
const history = ref<BackupHistoryEntry[]>([]);
const progress = ref<BackupProgress | null>(null);

// For DailyAt frequency
const selectedFrequencyType = ref<"Hourly" | "DailyAt" | "Weekly" | "Manual">("Manual");
const dailyAtHour = ref(9); // Default to 9 AM

const frequencies: { value: string; label: string; description: string }[] = [
  { value: "Hourly", label: "Hourly", description: "Backup every hour" },
  { value: "DailyAt", label: "Daily at specific time", description: "Backup once per day at a specific hour" },
  { value: "Weekly", label: "Weekly", description: "Backup once per week" },
  { value: "Manual", label: "Manual Only", description: "No automatic backups" },
];

const destinations: { value: BackupDestination; label: string; icon: string }[] = [
  { value: "Local", label: "Local Storage", icon: "💾" },
  { value: "GoogleDrive", label: "Google Drive", icon: "☁️" },
];

const lastBackupFormatted = computed(() => {
  if (!schedule.value.lastBackup) return "Never";
  try {
    return new Date(schedule.value.lastBackup).toLocaleString();
  } catch {
    return "Unknown";
  }
});

const nextBackupFormatted = computed(() => {
  if (!schedule.value.enabled || !schedule.value.nextBackup) return "N/A";
  try {
    return new Date(schedule.value.nextBackup).toLocaleString();
  } catch {
    return "Unknown";
  }
});

function parseFrequency(freq: BackupFrequency) {
  if (typeof freq === "string") {
    selectedFrequencyType.value = freq as any;
  } else if (typeof freq === "object" && "DailyAt" in freq) {
    selectedFrequencyType.value = "DailyAt";
    dailyAtHour.value = freq.DailyAt.hour;
  }
}

function buildFrequency(): BackupFrequency {
  if (selectedFrequencyType.value === "DailyAt") {
    return { DailyAt: { hour: dailyAtHour.value } };
  }
  return selectedFrequencyType.value;
}

async function loadSchedule() {
  loading.value = true;
  error.value = null;
  try {
    schedule.value = await getBackupSchedule();
    parseFrequency(schedule.value.frequency);

    // Set defaults if not present
    if (schedule.value.compress === undefined) {
      schedule.value.compress = true;
    }
    if (schedule.value.backupOnClose === undefined) {
      schedule.value.backupOnClose = false;
    }
    if (!schedule.value.cleanupSettings) {
      schedule.value.cleanupSettings = { keepLastN: 10, olderThanDays: null };
    }
    if (schedule.value.maxRetries === undefined) {
      schedule.value.maxRetries = 3;
    }
  } catch (err) {
    error.value = `Failed to load schedule: ${String(err)}`;
  } finally {
    loading.value = false;
  }
}

async function loadHistory() {
  try {
    history.value = await getBackupHistory();
  } catch (err) {
    console.error("Failed to load history:", err);
  }
}

async function loadProgress() {
  try {
    progress.value = await getBackupProgress();
  } catch (err) {
    console.error("Failed to load progress:", err);
  }
}

async function saveSchedule() {
  saving.value = true;
  message.value = null;
  error.value = null;
  try {
    // Build frequency from UI state
    schedule.value.frequency = buildFrequency();

    schedule.value = await updateBackupSchedule(schedule.value);
    message.value = "✅ Schedule saved successfully!";
    setTimeout(() => {
      message.value = null;
    }, 3000);
  } catch (err) {
    error.value = `Failed to save schedule: ${String(err)}`;
  } finally {
    saving.value = false;
  }
}

async function runBackupNow() {
  triggering.value = true;
  message.value = null;
  error.value = null;
  try {
    const result = await triggerManualBackup();
    message.value = `✅ Backup completed! ${result}`;
    // Reload schedule and history
    await loadSchedule();
    await loadHistory();
  } catch (err) {
    error.value = `Backup failed: ${String(err)}`;
  } finally {
    triggering.value = false;
  }
}

function toggleDestination(dest: BackupDestination) {
  const index = schedule.value.destinations.indexOf(dest);
  if (index >= 0) {
    // Don't allow removing the last destination
    if (schedule.value.destinations.length > 1) {
      schedule.value.destinations.splice(index, 1);
    }
  } else {
    schedule.value.destinations.push(dest);
  }
}

function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "N/A";
  const kb = bytes / 1024;
  const mb = kb / 1024;
  if (mb >= 1) return `${mb.toFixed(2)} MB`;
  if (kb >= 1) return `${kb.toFixed(2)} KB`;
  return `${bytes} bytes`;
}

function formatTimestamp(timestamp: string): string {
  try {
    return new Date(timestamp).toLocaleString();
  } catch {
    return timestamp;
  }
}

onMounted(() => {
  loadSchedule();
  loadHistory();
  loadProgress();

  // Poll progress every 2 seconds when backup is running
  setInterval(async () => {
    await loadProgress();
    if (progress.value?.isRunning) {
      await loadHistory();
    }
  }, 2000);
});
</script>

<template>
  <div class="backup-section">
    <div class="section-header">
      <h2>⏰ Scheduled Backups</h2>
      <p class="section-description">
        Automatically backup your data on a regular schedule with advanced options.
      </p>
    </div>

    <div v-if="loading" class="loading">Loading schedule...</div>

    <div v-else class="backup-content">
      <!-- Enable/Disable Toggle -->
      <div class="setting-row">
        <label class="toggle-label">
          <input
            type="checkbox"
            v-model="schedule.enabled"
            class="toggle-checkbox"
          />
          <span class="toggle-text">
            <strong>{{ schedule.enabled ? "✅ Enabled" : "❌ Disabled" }}</strong>
          </span>
        </label>
      </div>

      <!-- Frequency Selection -->
      <div class="setting-row">
        <label class="setting-label">📅 Backup Frequency</label>
        <div class="frequency-grid">
          <button
            v-for="freq in frequencies"
            :key="freq.value"
            @click="selectedFrequencyType = freq.value as any"
            :class="['frequency-btn', { active: selectedFrequencyType === freq.value }]"
            :disabled="!schedule.enabled && freq.value !== 'Manual'"
          >
            <div class="frequency-label">{{ freq.label }}</div>
            <div class="frequency-desc">{{ freq.description }}</div>
          </button>
        </div>

        <!-- Hour picker for DailyAt -->
        <div v-if="selectedFrequencyType === 'DailyAt'" class="hour-picker">
          <label>
            Run at hour (0-23):
            <input
              type="number"
              v-model.number="dailyAtHour"
              min="0"
              max="23"
              class="hour-input"
            />
          </label>
          <span class="hour-preview">
            ({{ dailyAtHour.toString().padStart(2, '0') }}:00)
          </span>
        </div>
      </div>

      <!-- Destination Selection -->
      <div class="setting-row">
        <label class="setting-label">📍 Backup Destinations</label>
        <div class="destinations">
          <button
            v-for="dest in destinations"
            :key="dest.value"
            @click="toggleDestination(dest.value)"
            :class="['destination-btn', { active: schedule.destinations.includes(dest.value) }]"
          >
            <span class="dest-icon">{{ dest.icon }}</span>
            <span class="dest-label">{{ dest.label }}</span>
            <span v-if="schedule.destinations.includes(dest.value)" class="check">✓</span>
          </button>
        </div>
      </div>

      <!-- Advanced Options -->
      <div class="setting-row">
        <label class="setting-label">⚙️ Advanced Options</label>

        <div class="options-grid">
          <label class="checkbox-label">
            <input type="checkbox" v-model="schedule.compress" />
            <span>🗜️ Compress backups (gzip)</span>
          </label>

          <label class="checkbox-label">
            <input type="checkbox" v-model="schedule.backupOnClose" />
            <span>💾 Backup when app closes</span>
          </label>
        </div>

        <div class="input-row">
          <label>
            🔄 Max retries on failure:
            <input
              type="number"
              v-model.number="schedule.maxRetries"
              min="1"
              max="10"
              class="small-input"
            />
          </label>
        </div>
      </div>

      <!-- Cleanup Settings -->
      <div class="setting-row">
        <label class="setting-label">🗑️ Old Backup Cleanup</label>

        <div class="cleanup-options">
          <label>
            Keep last N backups:
            <input
              type="number"
              v-model.number="schedule.cleanupSettings!.keepLastN"
              min="1"
              max="100"
              class="small-input"
              placeholder="Leave empty to keep all"
            />
          </label>

          <label>
            Delete backups older than (days):
            <input
              type="number"
              v-model.number="schedule.cleanupSettings!.olderThanDays"
              min="1"
              max="365"
              class="small-input"
              placeholder="Leave empty to keep all"
            />
          </label>
        </div>
        <p class="helper-text">
          💡 Cleanup runs after each successful backup
        </p>
      </div>

      <!-- Backup Progress -->
      <div v-if="progress?.isRunning" class="progress-section">
        <h3>⏳ Backup in Progress</h3>
        <div class="progress-bar">
          <div class="progress-fill"></div>
        </div>
        <p class="current-step">{{ progress.currentStep }}</p>
        <ul class="step-list">
          <li v-for="step in progress.completedSteps" :key="step" class="step-completed">
            ✅ {{ step }}
          </li>
          <li v-for="step in progress.failedSteps" :key="step" class="step-failed">
            ❌ {{ step }}
          </li>
        </ul>
      </div>

      <!-- Backup Status -->
      <div class="status-section">
        <div class="status-row">
          <span class="status-label">Last Backup:</span>
          <span class="status-value">{{ lastBackupFormatted }}</span>
        </div>
        <div class="status-row" v-if="schedule.enabled">
          <span class="status-label">Next Scheduled:</span>
          <span class="status-value">{{ nextBackupFormatted }}</span>
        </div>
      </div>

      <!-- Action Buttons -->
      <div class="button-row">
        <button
          @click="saveSchedule"
          :disabled="saving"
          class="save-btn"
        >
          {{ saving ? "💾 Saving..." : "💾 Save Schedule" }}
        </button>

        <button
          @click="runBackupNow"
          :disabled="triggering || schedule.destinations.length === 0 || progress?.isRunning"
          class="trigger-btn"
        >
          {{ triggering ? "⏳ Running..." : "▶️ Run Backup Now" }}
        </button>
      </div>

      <!-- Messages -->
      <div v-if="message" class="message success">
        {{ message }}
      </div>

      <div v-if="error" class="message error">
        {{ error }}
      </div>

      <!-- Backup History -->
      <div v-if="history.length > 0" class="history-section">
        <h3>📜 Backup History</h3>
        <div class="history-table-container">
          <table class="history-table">
            <thead>
              <tr>
                <th>Status</th>
                <th>Timestamp</th>
                <th>Destinations</th>
                <th>Size</th>
                <th>Compressed</th>
              </tr>
            </thead>
            <tbody>
              <tr v-for="(entry, index) in history" :key="index" :class="{ 'failed-row': !entry.success }">
                <td>{{ entry.success ? '✅' : '❌' }}</td>
                <td>{{ formatTimestamp(entry.timestamp) }}</td>
                <td>{{ entry.destinations.join(', ') }}</td>
                <td>{{ formatBytes(entry.sizeBytes) }}</td>
                <td>{{ entry.compressed ? '🗜️' : '—' }}</td>
              </tr>
            </tbody>
          </table>
        </div>
      </div>

      <!-- Info Box -->
      <div class="info-box">
        <p><strong>ℹ️ How it works:</strong></p>
        <ul>
          <li>Enable scheduled backups to automatically save your data</li>
          <li>Choose how often you want backups to run (or set a specific time)</li>
          <li>Select where to save backups (local files and/or Google Drive)</li>
          <li>The scheduler runs in the background while the app is open</li>
          <li>Backups are automatically retried on failure with exponential backoff</li>
          <li>Old backups are cleaned up automatically based on your settings</li>
        </ul>
      </div>
    </div>
  </div>
</template>

<style scoped>
.backup-section {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  margin: 20px 0;
}

.section-header h2 {
  margin: 0 0 8px 0;
  font-size: 24px;
  color: #2c3e50;
}

.section-description {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.backup-content {
  margin-top: 20px;
}

.loading {
  text-align: center;
  padding: 40px;
  color: #666;
}

.setting-row {
  margin-bottom: 24px;
}

.toggle-label {
  display: flex;
  align-items: center;
  cursor: pointer;
  gap: 12px;
}

.toggle-checkbox {
  width: 20px;
  height: 20px;
  cursor: pointer;
}

.toggle-text {
  font-size: 16px;
  color: #333;
}

.setting-label {
  display: block;
  font-weight: 600;
  font-size: 16px;
  color: #333;
  margin-bottom: 12px;
}

.frequency-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(180px, 1fr));
  gap: 12px;
  margin-bottom: 12px;
}

.frequency-btn {
  padding: 12px 16px;
  border: 2px solid #ddd;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
}

.frequency-btn:hover:not(:disabled) {
  border-color: #007bff;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.2);
}

.frequency-btn.active {
  border-color: #007bff;
  background-color: #e7f3ff;
}

.frequency-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.frequency-label {
  font-weight: 600;
  color: #333;
  margin-bottom: 4px;
}

.frequency-desc {
  font-size: 12px;
  color: #666;
}

.hour-picker {
  margin-top: 12px;
  padding: 12px;
  background: #f8f9fa;
  border-radius: 8px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.hour-input {
  width: 80px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-left: 8px;
}

.hour-preview {
  color: #666;
  font-style: italic;
}

.destinations {
  display: flex;
  gap: 12px;
  flex-wrap: wrap;
}

.destination-btn {
  padding: 12px 20px;
  border: 2px solid #ddd;
  border-radius: 8px;
  background: white;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
}

.destination-btn:hover {
  border-color: #28a745;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.2);
}

.destination-btn.active {
  border-color: #28a745;
  background-color: #d4edda;
}

.dest-icon {
  font-size: 18px;
}

.dest-label {
  font-weight: 500;
  color: #333;
}

.check {
  color: #28a745;
  font-weight: bold;
  font-size: 16px;
}

.options-grid {
  display: flex;
  flex-direction: column;
  gap: 12px;
  margin-bottom: 12px;
}

.checkbox-label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #333;
}

.checkbox-label input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.input-row {
  margin-top: 12px;
}

.input-row label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #333;
}

.small-input {
  width: 100px;
  padding: 6px 10px;
  border: 1px solid #ddd;
  border-radius: 4px;
  margin-left: 8px;
}

.cleanup-options {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.cleanup-options label {
  display: flex;
  align-items: center;
  gap: 8px;
  font-size: 14px;
  color: #333;
}

.helper-text {
  margin-top: 8px;
  font-size: 13px;
  color: #666;
}

.progress-section {
  background: #fff3cd;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
  border: 2px solid #ffc107;
}

.progress-section h3 {
  margin: 0 0 12px 0;
  color: #333;
  font-size: 18px;
}

.progress-bar {
  height: 8px;
  background: #e9ecef;
  border-radius: 4px;
  overflow: hidden;
  margin-bottom: 12px;
}

.progress-fill {
  height: 100%;
  background: linear-gradient(90deg, #007bff, #0056b3);
  animation: progress-animation 1.5s infinite;
}

@keyframes progress-animation {
  0% { width: 10%; }
  50% { width: 90%; }
  100% { width: 10%; }
}

.current-step {
  font-weight: 600;
  color: #333;
  margin: 8px 0;
}

.step-list {
  list-style: none;
  padding: 0;
  margin: 8px 0 0 0;
}

.step-list li {
  padding: 4px 0;
  font-size: 13px;
}

.step-completed {
  color: #28a745;
}

.step-failed {
  color: #dc3545;
}

.status-section {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.status-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 8px;
  font-size: 14px;
}

.status-row:last-child {
  margin-bottom: 0;
}

.status-label {
  font-weight: 600;
  color: #555;
}

.status-value {
  color: #333;
}

.button-row {
  display: flex;
  gap: 12px;
  margin-bottom: 16px;
}

.save-btn,
.trigger-btn {
  flex: 1;
  padding: 14px 24px;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.save-btn {
  background-color: #007bff;
  color: white;
}

.save-btn:hover:not(:disabled) {
  background-color: #0056b3;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.3);
}

.trigger-btn {
  background-color: #28a745;
  color: white;
}

.trigger-btn:hover:not(:disabled) {
  background-color: #218838;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.save-btn:disabled,
.trigger-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.message {
  margin-bottom: 16px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
}

.message.success {
  background-color: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.message.error {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.history-section {
  margin-top: 32px;
}

.history-section h3 {
  margin: 0 0 16px 0;
  color: #333;
  font-size: 20px;
}

.history-table-container {
  overflow-x: auto;
}

.history-table {
  width: 100%;
  border-collapse: collapse;
  font-size: 14px;
}

.history-table thead {
  background: #f8f9fa;
}

.history-table th {
  padding: 12px;
  text-align: left;
  font-weight: 600;
  color: #333;
  border-bottom: 2px solid #dee2e6;
}

.history-table td {
  padding: 12px;
  border-bottom: 1px solid #dee2e6;
  color: #555;
}

.history-table tbody tr:hover {
  background: #f8f9fa;
}

.failed-row {
  background: #fff5f5;
}

.info-box {
  background: #e7f3ff;
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid #007bff;
  margin-top: 24px;
}

.info-box p {
  margin: 0 0 8px 0;
  color: #333;
}

.info-box ul {
  margin: 8px 0 0 0;
  padding-left: 24px;
}

.info-box li {
  margin: 4px 0;
  color: #555;
  font-size: 14px;
}
</style>
