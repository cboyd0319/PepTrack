<script setup lang="ts">
import { ref, onMounted } from "vue";
import {
  checkDriveStatus,
  startDriveOAuth,
  disconnectDrive,
  uploadToDrive,
  exportBackupData,
  type DriveOAuthConfig,
  type DriveStatus,
} from "../api/peptrack";
import { showErrorToast, showSuccessToast } from "../utils/errorHandling";

const driveStatus = ref<DriveStatus | null>(null);
const loading = ref(false);

// ═══════════════════════════════════════════════════════════════
// 🔐 OAUTH CREDENTIALS - DEVELOPER CONFIGURATION
// ═══════════════════════════════════════════════════════════════
// TODO: Replace these placeholders with YOUR Google Cloud OAuth credentials
// See docs/GOOGLE_OAUTH_SETUP.md for detailed setup instructions
//
// IMPORTANT: These credentials are for YOUR app, not the user's data
// - Users will log in with their Google account
// - Their data stays in THEIR Google Drive
// - You're just providing the app credentials for the OAuth flow
// ═══════════════════════════════════════════════════════════════

const GOOGLE_OAUTH_CONFIG: DriveOAuthConfig = {
  // Get these from: https://console.cloud.google.com/apis/credentials
  clientId: "YOUR_CLIENT_ID_HERE.apps.googleusercontent.com",
  clientSecret: "YOUR_CLIENT_SECRET_HERE",
};

// ═══════════════════════════════════════════════════════════════

async function loadDriveStatus() {
  try {
    driveStatus.value = await checkDriveStatus();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'check Google Drive status' });
  }
}

async function connectGoogleDrive() {
  loading.value = true;

  try {
    const response = await startDriveOAuth(GOOGLE_OAUTH_CONFIG);

    // Open browser for OAuth
    window.open(response.authUrl, '_blank');

    showSuccessToast(
      'Opening Browser',
      'Please complete the authorization in your browser, then return here.'
    );

    // Note: The actual connection happens when the user completes OAuth
    // and the backend receives the callback
    setTimeout(() => {
      loadDriveStatus();
    }, 3000); // Check status after a delay

  } catch (error: unknown) {
    showErrorToast(error, { operation: 'connect to Google Drive' });
  } finally {
    loading.value = false;
  }
}

async function handleDisconnect() {
  loading.value = true;

  try {
    await disconnectDrive();
    showSuccessToast('Disconnected', 'Disconnected from Google Drive successfully');
    await loadDriveStatus();
  } catch (error) {
    showErrorToast(error, { operation: 'disconnect from Google Drive' });
  } finally {
    loading.value = false;
  }
}

async function handleBackupToDrive() {
  loading.value = true;

  try {
    // Get backup data (returns JSON string)
    const backupJson = await exportBackupData();

    // Parse to get metadata for success message
    const backupData = JSON.parse(backupJson);
    const metadata = backupData.metadata || backupData;

    // Generate filename
    const now = new Date();
    const timestamp = now.toISOString().slice(0, 16).replace('T', '_').replace(':', '-');
    const filename = `peptrack_backup_${timestamp}.json`;

    // Upload to Drive
    await uploadToDrive(filename, backupJson);

    showSuccessToast(
      'Backup Uploaded',
      `Backup uploaded successfully! (${metadata.protocolsCount || 0} protocols, ${metadata.dosesCount || 0} doses, ${metadata.literatureCount || 0} papers)`
    );

  } catch (error) {
    showErrorToast(error, { operation: 'upload backup to Google Drive' });
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadDriveStatus();
});
</script>

<template>
  <div class="drive-section">
    <div class="section-header">
      <h3>🔗 Google Drive</h3>
      <p class="section-description">
        Automatically sync your backups to Google Drive
      </p>
    </div>

    <!-- Not Connected State -->
    <div v-if="!driveStatus?.connected" class="drive-content">
      <div class="status-card disconnected">
        <div class="status-icon">☁️</div>
        <div class="status-info">
          <h4>Not Connected</h4>
          <p>Sync your backups to Google Drive for safekeeping</p>
        </div>
      </div>

      <button
        @click="connectGoogleDrive"
        :disabled="loading"
        class="connect-btn"
      >
        <span v-if="!loading">🔗 Connect Google Drive</span>
        <span v-else>⏳ Connecting...</span>
      </button>

      <div class="privacy-note">
        <div class="privacy-icon">🔒</div>
        <div>
          <strong>Privacy Protected:</strong>
          PepTrack only accesses files it creates in your Drive.
          We never see or access your other files.
        </div>
      </div>
    </div>

    <!-- Connected State -->
    <div v-else class="drive-content">
      <div class="status-card connected">
        <div class="status-icon">✅</div>
        <div class="status-info">
          <h4>Connected to Google Drive</h4>
          <p v-if="driveStatus.email" class="drive-email">{{ driveStatus.email }}</p>
        </div>
      </div>

      <div class="actions">
        <button
          @click="handleBackupToDrive"
          :disabled="loading"
          class="backup-btn"
        >
          <span v-if="!loading">☁️ Backup Now</span>
          <span v-else>⏳ Uploading...</span>
        </button>

        <button
          @click="handleDisconnect"
          :disabled="loading"
          class="disconnect-btn"
        >
          Disconnect
        </button>
      </div>
    </div>
  </div>
</template>

<style scoped>
.drive-section {
  margin-bottom: 24px;
}

.section-header h3 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 6px 0;
  color: #1a1a1a;
}

.section-description {
  margin: 0 0 16px 0;
  color: #666;
  font-size: 14px;
}

.drive-content {
  display: flex;
  flex-direction: column;
  gap: 16px;
}

.status-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 20px;
  border-radius: 12px;
  border: 2px solid #e0e0e0;
}

.status-card.disconnected {
  background: #f5f5f5;
}

.status-card.connected {
  background: #e8f5e9;
  border-color: #4caf50;
}

.status-icon {
  font-size: 40px;
  flex-shrink: 0;
}

.status-info h4 {
  margin: 0 0 4px 0;
  font-size: 16px;
  font-weight: 600;
  color: #1a1a1a;
}

.status-info p {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.drive-email {
  font-family: monospace;
  font-size: 13px;
  color: #4caf50 !important;
  font-weight: 600;
}

.connect-btn {
  width: 100%;
  padding: 14px 24px;
  background-color: #1976d2;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 15px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.connect-btn:hover:not(:disabled) {
  background-color: #1565c0;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(25, 118, 210, 0.3);
}

.connect-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  transform: none;
}

.privacy-note {
  display: flex;
  gap: 12px;
  padding: 12px;
  background: #e3f2fd;
  border-radius: 8px;
  border-left: 4px solid #2196f3;
  font-size: 13px;
  color: #1565c0;
  align-items: flex-start;
}

.privacy-icon {
  font-size: 20px;
  flex-shrink: 0;
}

.privacy-note strong {
  color: #1565c0;
}

.actions {
  display: flex;
  gap: 12px;
}

.backup-btn {
  flex: 1;
  padding: 12px 24px;
  background-color: #4caf50;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.backup-btn:hover:not(:disabled) {
  background-color: #45a049;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(76, 175, 80, 0.3);
}

.backup-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
  transform: none;
}

.disconnect-btn {
  padding: 12px 24px;
  background-color: transparent;
  color: #dc3545;
  border: 2px solid #dc3545;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.disconnect-btn:hover:not(:disabled) {
  background-color: #dc3545;
  color: white;
}

.disconnect-btn:disabled {
  background-color: #f0f0f0;
  color: #ccc;
  border-color: #ccc;
  cursor: not-allowed;
}

@media (prefers-color-scheme: dark) {
  .section-header h3,
  .status-info h4 {
    color: #fff;
  }

  .section-description,
  .status-info p {
    color: #aaa;
  }

  .status-card {
    border-color: #3a3a3a;
  }

  .status-card.disconnected {
    background: #2a2a2a;
  }

  .status-card.connected {
    background: #1a3a1a;
    border-color: #4caf50;
  }

  .privacy-note {
    background: #1a2a3a;
    border-left-color: #2196f3;
    color: #64b5f6;
  }

  .privacy-note strong {
    color: #64b5f6;
  }
}
</style>
