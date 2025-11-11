<script setup lang="ts">
import { ref } from "vue";
import { exportBackupData } from "../api/peptrack";

const exporting = ref(false);
const exportMessage = ref<string | null>(null);
const exportError = ref<string | null>(null);
const useEncryption = ref(false);
const password = ref("");
const confirmPassword = ref("");

async function handleExport() {
  exporting.value = true;
  exportMessage.value = null;
  exportError.value = null;

  try {
    // Validate password if encryption is enabled
    if (useEncryption.value) {
      if (!password.value) {
        exportError.value = "Please enter a password for encryption";
        return;
      }
      if (password.value !== confirmPassword.value) {
        exportError.value = "Passwords do not match";
        return;
      }
      if (password.value.length < 8) {
        exportError.value = "Password must be at least 8 characters";
        return;
      }
    }

    // Get the backup data (JSON string)
    const backupJson = await exportBackupData(useEncryption.value ? password.value : undefined);

    // Parse to get metadata for success message
    const backupData = JSON.parse(backupJson);
    const metadata = backupData.metadata || backupData;

    // Create blob from JSON string
    const blob = new Blob([backupJson], { type: "application/json" });

    // Create download link
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");

    // Generate filename with timestamp
    const now = new Date();
    const timestamp = now.toISOString().slice(0, 16).replace('T', '_').replace(':', '-');
    const suffix = useEncryption.value ? '_encrypted' : '';
    link.download = `peptrack_backup_${timestamp}${suffix}.json`;
    link.href = url;

    // Trigger download
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    // Clean up
    URL.revokeObjectURL(url);

    const encryptionNote = useEncryption.value ? " 🔒 (encrypted)" : "";
    exportMessage.value = `✅ Backup downloaded successfully${encryptionNote}! (${metadata.protocolsCount || 0} protocols, ${metadata.dosesCount || 0} doses, ${metadata.literatureCount || 0} papers)`;

    // Clear password fields after successful export
    if (useEncryption.value) {
      password.value = "";
      confirmPassword.value = "";
    }

  } catch (error) {
    exportError.value = `Failed to export backup: ${String(error)}`;
  } finally {
    exporting.value = false;
  }
}
</script>

<template>
  <div class="backup-section">
    <div class="section-header">
      <h2>💾 Backup Your Data</h2>
      <p class="section-description">
        Save all your protocols, doses, and research papers to a file on your computer.
      </p>
    </div>

    <div class="backup-content">
      <div class="backup-info">
        <p>📦 <strong>What gets backed up:</strong></p>
        <ul>
          <li>All your protocols</li>
          <li>All dose logs</li>
          <li>All saved research papers</li>
        </ul>
        <p class="backup-note">
          💡 The backup file is saved as JSON and can be kept safe for restoring your data later.
        </p>
      </div>

      <div class="encryption-section">
        <label class="encryption-checkbox">
          <input
            type="checkbox"
            v-model="useEncryption"
            :disabled="exporting"
          />
          <span>🔒 Encrypt backup with password (optional)</span>
        </label>

        <div v-if="useEncryption" class="password-inputs">
          <div class="critical-warning">
            <div class="warning-icon">⚠️</div>
            <div class="warning-content">
              <strong>CRITICAL: NO PASSWORD RECOVERY POSSIBLE</strong>
              <p>
                If you forget this password, <strong>your backup will be permanently unrecoverable</strong>.
                There is no password reset, no recovery mechanism, and no way to decrypt the backup without the exact password.
              </p>
              <p>
                💡 <strong>Recommendation:</strong> Store this password in a secure password manager or write it down and keep it in a safe place.
              </p>
            </div>
          </div>

          <div class="input-group">
            <label for="password">Password:</label>
            <input
              id="password"
              type="password"
              v-model="password"
              placeholder="Enter password (min 8 characters)"
              :disabled="exporting"
              autocomplete="new-password"
            />
          </div>
          <div class="input-group">
            <label for="confirmPassword">Confirm Password:</label>
            <input
              id="confirmPassword"
              type="password"
              v-model="confirmPassword"
              placeholder="Re-enter password"
              :disabled="exporting"
              autocomplete="new-password"
            />
          </div>
        </div>
      </div>

      <button
        @click="handleExport"
        :disabled="exporting"
        class="export-btn"
        aria-label="Export backup data to file"
        :aria-busy="exporting"
      >
        {{ exporting ? "⏳ Creating Backup..." : "📥 Export Backup Now" }}
      </button>

      <div v-if="exportMessage" class="message success">
        {{ exportMessage }}
      </div>

      <div v-if="exportError" class="message error">
        {{ exportError }}
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

.backup-info {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.backup-info p {
  margin: 0 0 8px 0;
  color: #333;
}

.backup-info ul {
  margin: 8px 0 12px 0;
  padding-left: 24px;
}

.backup-info li {
  margin: 4px 0;
  color: #555;
}

.backup-note {
  background: #fff3cd;
  padding: 10px;
  border-radius: 6px;
  border-left: 3px solid #ffc107;
  font-size: 13px;
  color: #856404;
  margin-top: 12px !important;
}

.encryption-section {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
  border: 1px solid #dee2e6;
}

.encryption-checkbox {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  font-size: 15px;
  font-weight: 500;
  color: #333;
}

.encryption-checkbox input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.encryption-checkbox input[type="checkbox"]:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.password-inputs {
  margin-top: 16px;
  padding-top: 16px;
  border-top: 1px solid #dee2e6;
}

.critical-warning {
  background: linear-gradient(135deg, #dc3545 0%, #c82333 100%);
  color: white;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
  border: 3px solid #bd2130;
  box-shadow: 0 4px 12px rgba(220, 53, 69, 0.3);
  display: flex;
  gap: 12px;
  align-items: flex-start;
}

.warning-icon {
  font-size: 32px;
  flex-shrink: 0;
  animation: pulse 2s infinite;
}

@keyframes pulse {
  0%, 100% {
    transform: scale(1);
  }
  50% {
    transform: scale(1.1);
  }
}

.warning-content {
  flex: 1;
}

.warning-content strong {
  font-size: 16px;
  display: block;
  margin-bottom: 8px;
  text-transform: uppercase;
  letter-spacing: 0.5px;
}

.warning-content p {
  margin: 8px 0;
  font-size: 14px;
  line-height: 1.5;
}

.warning-content p:last-child {
  margin-bottom: 0;
  background: rgba(255, 255, 255, 0.15);
  padding: 8px 10px;
  border-radius: 4px;
  border-left: 3px solid #ffc107;
}

.input-group {
  margin-bottom: 12px;
}

.input-group label {
  display: block;
  margin-bottom: 6px;
  font-size: 14px;
  font-weight: 500;
  color: #495057;
}

.input-group input[type="password"] {
  width: 100%;
  padding: 10px 12px;
  border: 1px solid #ced4da;
  border-radius: 6px;
  font-size: 14px;
  transition: border-color 0.2s;
}

.input-group input[type="password"]:focus {
  outline: none;
  border-color: #80bdff;
  box-shadow: 0 0 0 3px rgba(0, 123, 255, 0.1);
}

.input-group input[type="password"]:disabled {
  background-color: #e9ecef;
  cursor: not-allowed;
}

.export-btn {
  width: 100%;
  padding: 14px 24px;
  background-color: #28a745;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.export-btn:hover:not(:disabled) {
  background-color: #218838;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.export-btn:active:not(:disabled) {
  transform: translateY(0);
}

.export-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.message {
  margin-top: 16px;
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
</style>
