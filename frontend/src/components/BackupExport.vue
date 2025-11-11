<script setup lang="ts">
import { ref } from "vue";
import { exportBackupData } from "../api/peptrack";

const exporting = ref(false);
const exportMessage = ref<string | null>(null);
const exportError = ref<string | null>(null);

async function handleExport() {
  exporting.value = true;
  exportMessage.value = null;
  exportError.value = null;

  try {
    // Get the backup data
    const backupData = await exportBackupData();

    // Create JSON blob
    const jsonContent = JSON.stringify(backupData, null, 2);
    const blob = new Blob([jsonContent], { type: "application/json" });

    // Create download link
    const url = URL.createObjectURL(blob);
    const link = document.createElement("a");

    // Generate filename with timestamp
    const now = new Date();
    const timestamp = now.toISOString().slice(0, 16).replace('T', '_').replace(':', '-');
    link.download = `peptrack_backup_${timestamp}.json`;
    link.href = url;

    // Trigger download
    document.body.appendChild(link);
    link.click();
    document.body.removeChild(link);

    // Clean up
    URL.revokeObjectURL(url);

    exportMessage.value = `✅ Backup downloaded successfully! (${backupData.metadata.protocolsCount} protocols, ${backupData.metadata.dosesCount} doses, ${backupData.metadata.literatureCount} papers)`;

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

      <button
        @click="handleExport"
        :disabled="exporting"
        class="export-btn"
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
