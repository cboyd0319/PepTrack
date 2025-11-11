<script setup lang="ts">
import { ref, onMounted, computed } from "vue";
import {
  getBackupSchedule,
  updateBackupSchedule,
  triggerManualBackup,
  type BackupSchedule,
  type BackupFrequency,
  type BackupDestination,
} from "../api/peptrack";

const schedule = ref<BackupSchedule>({
  enabled: false,
  frequency: "Manual",
  destinations: ["Local"],
  lastBackup: null,
  nextBackup: null,
});

const loading = ref(false);
const saving = ref(false);
const triggering = ref(false);
const message = ref<string | null>(null);
const error = ref<string | null>(null);

const frequencies: { value: BackupFrequency; label: string; description: string }[] = [
  { value: "Hourly", label: "Hourly", description: "Backup every hour" },
  { value: "Daily", label: "Daily", description: "Backup once per day" },
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

async function loadSchedule() {
  loading.value = true;
  error.value = null;
  try {
    schedule.value = await getBackupSchedule();
  } catch (err) {
    error.value = `Failed to load schedule: ${String(err)}`;
  } finally {
    loading.value = false;
  }
}

async function saveSchedule() {
  saving.value = true;
  message.value = null;
  error.value = null;
  try {
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
    // Reload schedule to update last backup time
    await loadSchedule();
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

onMounted(() => {
  loadSchedule();
});
</script>

<template>
  <div class="backup-section">
    <div class="section-header">
      <h2>⏰ Scheduled Backups</h2>
      <p class="section-description">
        Automatically backup your data on a regular schedule.
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
            @click="schedule.frequency = freq.value"
            :class="['frequency-btn', { active: schedule.frequency === freq.value }]"
            :disabled="!schedule.enabled && freq.value !== 'Manual'"
          >
            <div class="frequency-label">{{ freq.label }}</div>
            <div class="frequency-desc">{{ freq.description }}</div>
          </button>
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
        <p class="helper-text">
          💡 Select one or more destinations for your backups
        </p>
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
          :disabled="triggering || schedule.destinations.length === 0"
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

      <!-- Info Box -->
      <div class="info-box">
        <p><strong>ℹ️ How it works:</strong></p>
        <ul>
          <li>Enable scheduled backups to automatically save your data</li>
          <li>Choose how often you want backups to run</li>
          <li>Select where to save backups (local files and/or Google Drive)</li>
          <li>The scheduler runs in the background while the app is open</li>
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
  grid-template-columns: repeat(auto-fit, minmax(140px, 1fr));
  gap: 12px;
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

.helper-text {
  margin-top: 8px;
  font-size: 13px;
  color: #666;
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

.info-box {
  background: #e7f3ff;
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid #007bff;
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
