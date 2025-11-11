<script setup lang="ts">
import { ref } from "vue";
import { open } from "@tauri-apps/plugin-dialog";
import {
  previewBackup,
  restoreFromBackup,
  type BackupPreview,
  type RestoreResult,
} from "../api/peptrack";

const selectedFile = ref<string | null>(null);
const preview = ref<BackupPreview | null>(null);
const restoreResult = ref<RestoreResult | null>(null);
const loading = ref(false);
const restoring = ref(false);
const error = ref<string | null>(null);
const showConfirmDialog = ref(false);

async function selectBackupFile() {
  loading.value = true;
  error.value = null;
  preview.value = null;
  restoreResult.value = null;

  try {
    const selected = await open({
      multiple: false,
      filters: [
        {
          name: "Backup Files",
          extensions: ["json", "gz"],
        },
      ],
    });

    if (selected && typeof selected === "string") {
      selectedFile.value = selected;
      await loadPreview(selected);
    }
  } catch (err) {
    error.value = `Failed to select file: ${String(err)}`;
  } finally {
    loading.value = false;
  }
}

async function loadPreview(filePath: string) {
  loading.value = true;
  error.value = null;

  try {
    preview.value = await previewBackup(filePath);
  } catch (err) {
    error.value = `Failed to preview backup: ${String(err)}`;
    selectedFile.value = null;
  } finally {
    loading.value = false;
  }
}

function confirmRestore() {
  showConfirmDialog.value = true;
}

function cancelRestore() {
  showConfirmDialog.value = false;
}

async function performRestore() {
  if (!selectedFile.value) return;

  showConfirmDialog.value = false;
  restoring.value = true;
  error.value = null;
  restoreResult.value = null;

  try {
    restoreResult.value = await restoreFromBackup(selectedFile.value);
  } catch (err) {
    error.value = `Restore failed: ${String(err)}`;
  } finally {
    restoring.value = false;
  }
}

function reset() {
  selectedFile.value = null;
  preview.value = null;
  restoreResult.value = null;
  error.value = null;
  showConfirmDialog.value = false;
}

function formatDate(dateStr: string): string {
  try {
    return new Date(dateStr).toLocaleString();
  } catch {
    return dateStr;
  }
}
</script>

<template>
  <div class="restore-section">
    <div class="section-header">
      <h2>📥 Restore from Backup</h2>
      <p class="section-description">
        Restore your protocols, dose logs, and literature from a backup file.
      </p>
    </div>

    <div class="restore-content">
      <!-- File Selection -->
      <div v-if="!selectedFile" class="file-selection">
        <button @click="selectBackupFile" :disabled="loading" class="select-btn">
          {{ loading ? "⏳ Loading..." : "📂 Select Backup File" }}
        </button>
        <p class="helper-text">
          💡 Choose a .json or .json.gz backup file to preview and restore
        </p>
      </div>

      <!-- Preview Section -->
      <div v-if="preview && !restoreResult" class="preview-section">
        <div class="preview-header">
          <h3>📋 Backup Preview</h3>
          <button @click="reset" class="reset-btn" title="Select a different file">
            ↻ Select Different File
          </button>
        </div>

        <div class="preview-card">
          <div class="preview-row">
            <span class="preview-label">📁 File:</span>
            <span class="preview-value filename">{{ selectedFile?.split('/').pop() || selectedFile }}</span>
          </div>
          <div class="preview-row">
            <span class="preview-label">📅 Backup Date:</span>
            <span class="preview-value">{{ formatDate(preview.metadata.exportDate) }}</span>
          </div>
          <div class="preview-row">
            <span class="preview-label">🔢 App Version:</span>
            <span class="preview-value">{{ preview.metadata.appVersion }}</span>
          </div>
        </div>

        <div class="contents-summary">
          <h4>📦 Contents</h4>
          <div class="contents-grid">
            <div class="content-item">
              <div class="content-icon">🧪</div>
              <div class="content-info">
                <div class="content-count">{{ preview.protocolsCount }}</div>
                <div class="content-label">Protocols</div>
              </div>
            </div>
            <div class="content-item">
              <div class="content-icon">💉</div>
              <div class="content-info">
                <div class="content-count">{{ preview.doseLogsCount }}</div>
                <div class="content-label">Dose Logs</div>
              </div>
            </div>
            <div class="content-item">
              <div class="content-icon">📚</div>
              <div class="content-info">
                <div class="content-count">{{ preview.literatureCount }}</div>
                <div class="content-label">Literature Entries</div>
              </div>
            </div>
          </div>
        </div>

        <div class="warning-box">
          <p><strong>⚠️ Important:</strong></p>
          <ul>
            <li>This will merge the backup data with your current data</li>
            <li>Existing items with the same ID will be updated</li>
            <li>New items will be added</li>
            <li>No data will be deleted from your current database</li>
          </ul>
        </div>

        <div class="action-buttons">
          <button @click="confirmRestore" :disabled="restoring" class="restore-btn">
            🔄 Restore from This Backup
          </button>
        </div>
      </div>

      <!-- Confirmation Dialog -->
      <div v-if="showConfirmDialog" class="modal-overlay" @click="cancelRestore">
        <div class="modal-content" @click.stop>
          <h3>⚠️ Confirm Restore</h3>
          <p>Are you sure you want to restore from this backup?</p>
          <p class="modal-detail">
            This will add {{ preview?.protocolsCount }} protocols,
            {{ preview?.doseLogsCount }} dose logs, and
            {{ preview?.literatureCount }} literature entries to your current data.
          </p>
          <div class="modal-buttons">
            <button @click="cancelRestore" class="cancel-btn">Cancel</button>
            <button @click="performRestore" class="confirm-btn" :disabled="restoring">
              {{ restoring ? "⏳ Restoring..." : "✅ Yes, Restore" }}
            </button>
          </div>
        </div>
      </div>

      <!-- Restore Result -->
      <div v-if="restoreResult" class="result-section">
        <div class="success-header">
          <h3>✅ Restore Complete!</h3>
          <button @click="reset" class="reset-btn">↻ Restore Another Backup</button>
        </div>

        <div class="result-card">
          <h4>📊 Restored Items</h4>
          <div class="result-grid">
            <div class="result-item">
              <div class="result-icon">🧪</div>
              <div class="result-info">
                <div class="result-count">{{ restoreResult.counts.protocols }}</div>
                <div class="result-label">Protocols</div>
              </div>
            </div>
            <div class="result-item">
              <div class="result-icon">💉</div>
              <div class="result-info">
                <div class="result-count">{{ restoreResult.counts.doseLogs }}</div>
                <div class="result-label">Dose Logs</div>
              </div>
            </div>
            <div class="result-item">
              <div class="result-icon">📚</div>
              <div class="result-info">
                <div class="result-count">{{ restoreResult.counts.literature }}</div>
                <div class="result-label">Literature Entries</div>
              </div>
            </div>
          </div>
        </div>

        <div class="info-box">
          <p><strong>✨ Next Steps:</strong></p>
          <ul>
            <li>Refresh your protocol list to see the restored items</li>
            <li>Check your dose tracker for restored dose logs</li>
            <li>Visit the literature section to see restored papers</li>
          </ul>
        </div>
      </div>

      <!-- Error Message -->
      <div v-if="error" class="message error">
        {{ error }}
      </div>

      <!-- Loading State -->
      <div v-if="restoring && !restoreResult" class="restoring-overlay">
        <div class="restoring-spinner">⏳</div>
        <p class="restoring-text">Restoring backup data...</p>
        <p class="restoring-subtext">This may take a moment</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.restore-section {
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

.restore-content {
  margin-top: 20px;
  position: relative;
}

.file-selection {
  text-align: center;
  padding: 40px 20px;
  border: 2px dashed #ddd;
  border-radius: 12px;
  background: #f8f9fa;
}

.select-btn {
  padding: 16px 32px;
  background-color: #007bff;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.select-btn:hover:not(:disabled) {
  background-color: #0056b3;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 123, 255, 0.3);
}

.select-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.helper-text {
  margin-top: 12px;
  font-size: 13px;
  color: #666;
}

.preview-section {
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

.preview-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.preview-header h3 {
  margin: 0;
  color: #333;
  font-size: 20px;
}

.reset-btn {
  padding: 8px 16px;
  background-color: #6c757d;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.reset-btn:hover {
  background-color: #5a6268;
  transform: translateY(-1px);
}

.preview-card {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 16px;
  margin-bottom: 20px;
}

.preview-row {
  display: flex;
  justify-content: space-between;
  margin-bottom: 12px;
  font-size: 14px;
}

.preview-row:last-child {
  margin-bottom: 0;
}

.preview-label {
  font-weight: 600;
  color: #555;
}

.preview-value {
  color: #333;
}

.preview-value.filename {
  font-family: monospace;
  background: #fff;
  padding: 2px 8px;
  border-radius: 4px;
  font-size: 12px;
}

.contents-summary {
  margin-bottom: 20px;
}

.contents-summary h4 {
  margin: 0 0 12px 0;
  color: #333;
  font-size: 16px;
}

.contents-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.content-item {
  background: #e7f3ff;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
  border: 2px solid #007bff;
}

.content-icon {
  font-size: 32px;
}

.content-info {
  flex: 1;
}

.content-count {
  font-size: 24px;
  font-weight: 700;
  color: #007bff;
}

.content-label {
  font-size: 12px;
  color: #555;
  font-weight: 600;
}

.warning-box {
  background: #fff3cd;
  border-radius: 8px;
  padding: 16px;
  border-left: 4px solid #ffc107;
  margin-bottom: 20px;
}

.warning-box p {
  margin: 0 0 8px 0;
  color: #333;
}

.warning-box ul {
  margin: 8px 0 0 0;
  padding-left: 24px;
}

.warning-box li {
  margin: 4px 0;
  color: #555;
  font-size: 14px;
}

.action-buttons {
  display: flex;
  justify-content: center;
}

.restore-btn {
  padding: 14px 32px;
  background-color: #28a745;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.restore-btn:hover:not(:disabled) {
  background-color: #218838;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.restore-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(0, 0, 0, 0.5);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  animation: fadeIn 0.2s ease;
}

.modal-content {
  background: white;
  border-radius: 12px;
  padding: 24px;
  max-width: 500px;
  width: 90%;
  box-shadow: 0 8px 24px rgba(0, 0, 0, 0.2);
}

.modal-content h3 {
  margin: 0 0 12px 0;
  color: #333;
  font-size: 20px;
}

.modal-content p {
  margin: 0 0 12px 0;
  color: #555;
  font-size: 14px;
}

.modal-detail {
  background: #f8f9fa;
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #ffc107;
}

.modal-buttons {
  display: flex;
  gap: 12px;
  margin-top: 20px;
}

.cancel-btn,
.confirm-btn {
  flex: 1;
  padding: 12px 24px;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.cancel-btn {
  background-color: #6c757d;
  color: white;
}

.cancel-btn:hover {
  background-color: #5a6268;
}

.confirm-btn {
  background-color: #28a745;
  color: white;
}

.confirm-btn:hover:not(:disabled) {
  background-color: #218838;
}

.confirm-btn:disabled {
  background-color: #6c757d;
  cursor: not-allowed;
  opacity: 0.7;
}

.result-section {
  animation: fadeIn 0.3s ease;
}

.success-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.success-header h3 {
  margin: 0;
  color: #28a745;
  font-size: 20px;
}

.result-card {
  background: #d4edda;
  border-radius: 8px;
  padding: 20px;
  margin-bottom: 20px;
  border: 2px solid #28a745;
}

.result-card h4 {
  margin: 0 0 16px 0;
  color: #155724;
  font-size: 16px;
}

.result-grid {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
  gap: 16px;
}

.result-item {
  background: white;
  border-radius: 8px;
  padding: 16px;
  display: flex;
  align-items: center;
  gap: 12px;
}

.result-icon {
  font-size: 32px;
}

.result-info {
  flex: 1;
}

.result-count {
  font-size: 24px;
  font-weight: 700;
  color: #28a745;
}

.result-label {
  font-size: 12px;
  color: #555;
  font-weight: 600;
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

.message {
  margin-top: 16px;
  padding: 12px 16px;
  border-radius: 8px;
  font-size: 14px;
}

.message.error {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

.restoring-overlay {
  position: absolute;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background: rgba(255, 255, 255, 0.95);
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  border-radius: 12px;
  z-index: 10;
}

.restoring-spinner {
  font-size: 48px;
  animation: spin 2s linear infinite;
}

@keyframes spin {
  from {
    transform: rotate(0deg);
  }
  to {
    transform: rotate(360deg);
  }
}

.restoring-text {
  margin: 16px 0 0 0;
  font-size: 18px;
  font-weight: 600;
  color: #333;
}

.restoring-subtext {
  margin: 4px 0 0 0;
  font-size: 14px;
  color: #666;
}
</style>
