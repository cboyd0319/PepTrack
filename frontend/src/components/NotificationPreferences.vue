<script setup lang="ts">
import { ref, onMounted } from "vue";

// For now, this is a UI-only component
// In the future, we could add backend support for storing these preferences
const notificationsEnabled = ref(true);
const notifyOnSuccess = ref(true);
const notifyOnFailure = ref(true);
const notifyOnScheduledBackup = ref(true);
const notifyOnManualBackup = ref(true);

// Load preferences from localStorage
function loadPreferences() {
  const stored = localStorage.getItem("notification_preferences");
  if (stored) {
    try {
      const prefs = JSON.parse(stored);
      notificationsEnabled.value = prefs.enabled ?? true;
      notifyOnSuccess.value = prefs.onSuccess ?? true;
      notifyOnFailure.value = prefs.onFailure ?? true;
      notifyOnScheduledBackup.value = prefs.onScheduledBackup ?? true;
      notifyOnManualBackup.value = prefs.onManualBackup ?? true;
    } catch {
      // Ignore parse errors
    }
  }
}

// Save preferences to localStorage
function savePreferences() {
  try {
    const prefs = {
      enabled: notificationsEnabled.value,
      onSuccess: notifyOnSuccess.value,
      onFailure: notifyOnFailure.value,
      onScheduledBackup: notifyOnScheduledBackup.value,
      onManualBackup: notifyOnManualBackup.value,
    };
    localStorage.setItem("notification_preferences", JSON.stringify(prefs));
  } catch (error) {
    // Silently fail if localStorage is unavailable
  }
}

// Watch for changes and save
function handleChange() {
  savePreferences();
}

onMounted(() => {
  loadPreferences();
});
</script>

<template>
  <div class="notifications-section">
    <div class="section-header">
      <h2>🔔 Notification Preferences</h2>
      <p class="section-description">
        Control when you receive desktop notifications for backup operations.
      </p>
    </div>

    <div class="preferences-content">
      <!-- Master Toggle -->
      <div class="setting-row master-toggle">
        <label class="toggle-label">
          <input
            type="checkbox"
            v-model="notificationsEnabled"
            @change="handleChange"
            class="toggle-checkbox"
          />
          <span class="toggle-text">
            <strong>{{ notificationsEnabled ? "🔔 Notifications Enabled" : "🔕 Notifications Disabled" }}</strong>
            <span class="toggle-desc">{{ notificationsEnabled ? "You will receive desktop notifications" : "All notifications are silenced" }}</span>
          </span>
        </label>
      </div>

      <!-- Detailed Settings -->
      <div v-if="notificationsEnabled" class="detailed-settings">
        <h3>Notify me when:</h3>

        <div class="setting-group">
          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="notifyOnSuccess"
              @change="handleChange"
              class="checkbox"
            />
            <span class="label-content">
              <span class="label-title">✅ Backups succeed</span>
              <span class="label-desc">Get notified when backups complete successfully</span>
            </span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="notifyOnFailure"
              @change="handleChange"
              class="checkbox"
            />
            <span class="label-content">
              <span class="label-title">❌ Backups fail</span>
              <span class="label-desc">Get notified when backups encounter errors</span>
            </span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="notifyOnScheduledBackup"
              @change="handleChange"
              class="checkbox"
            />
            <span class="label-content">
              <span class="label-title">⏰ Scheduled backups run</span>
              <span class="label-desc">Get notified for automatic scheduled backups</span>
            </span>
          </label>

          <label class="checkbox-label">
            <input
              type="checkbox"
              v-model="notifyOnManualBackup"
              @change="handleChange"
              class="checkbox"
            />
            <span class="label-content">
              <span class="label-title">🖱️ Manual backups complete</span>
              <span class="label-desc">Get notified when you trigger backups manually</span>
            </span>
          </label>
        </div>
      </div>

      <!-- Info Box -->
      <div class="info-box">
        <p><strong>ℹ️ About Notifications:</strong></p>
        <ul>
          <li>Notifications appear as desktop alerts from your operating system</li>
          <li>Make sure you've granted PepTrack notification permissions in your OS settings</li>
          <li>Notifications help you stay informed about backup status without checking the app</li>
          <li>You can customize which types of events trigger notifications</li>
        </ul>
      </div>

      <!-- Test Notification Button -->
      <div class="test-section">
        <button
          @click="$emit('test-notification')"
          class="test-btn"
          :disabled="!notificationsEnabled"
        >
          🔔 Test Notification
        </button>
        <p class="helper-text">Click to send a test notification and verify your settings</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.notifications-section {
  background: white;
  border-radius: 12px;
  padding: 24px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
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

.preferences-content {
  margin-top: 20px;
}

.setting-row {
  margin-bottom: 24px;
}

.master-toggle {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
  border: 2px solid #007bff;
}

.toggle-label {
  display: flex;
  align-items: flex-start;
  cursor: pointer;
  gap: 16px;
}

.toggle-checkbox {
  width: 24px;
  height: 24px;
  cursor: pointer;
  flex-shrink: 0;
  margin-top: 2px;
}

.toggle-text {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.toggle-text strong {
  font-size: 18px;
  color: #333;
}

.toggle-desc {
  font-size: 14px;
  color: #666;
}

.detailed-settings {
  margin-top: 24px;
  padding-top: 24px;
  border-top: 1px solid #e9ecef;
}

.detailed-settings h3 {
  margin: 0 0 16px 0;
  color: #333;
  font-size: 16px;
}

.setting-group {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.checkbox-label {
  display: flex;
  align-items: flex-start;
  gap: 12px;
  cursor: pointer;
  padding: 12px;
  border-radius: 8px;
  transition: background 0.2s;
}

.checkbox-label:hover {
  background: #f8f9fa;
}

.checkbox {
  width: 20px;
  height: 20px;
  cursor: pointer;
  flex-shrink: 0;
  margin-top: 2px;
}

.label-content {
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.label-title {
  font-size: 15px;
  font-weight: 600;
  color: #333;
}

.label-desc {
  font-size: 13px;
  color: #666;
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

.test-section {
  margin-top: 24px;
  text-align: center;
  padding: 20px;
  background: #f8f9fa;
  border-radius: 8px;
}

.test-btn {
  padding: 12px 24px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.test-btn:hover:not(:disabled) {
  background-color: #0056b3;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.3);
}

.test-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.helper-text {
  margin-top: 8px;
  font-size: 13px;
  color: #666;
}
</style>
