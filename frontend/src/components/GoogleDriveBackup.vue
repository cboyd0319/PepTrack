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

const driveStatus = ref<DriveStatus | null>(null);
const loading = ref(false);
const message = ref<string | null>(null);
const errorMessage = ref<string | null>(null);

// OAuth configuration
const showConfigForm = ref(false);
const oauthConfig = ref<DriveOAuthConfig>({
  clientId: "",
  clientSecret: "",
});

// OAuth flow state
const awaitingCallback = ref(false);
const oauthAuthUrl = ref<string | null>(null);

async function loadDriveStatus() {
  try {
    driveStatus.value = await checkDriveStatus();
  } catch (error) {
    console.error("Failed to check Drive status:", error);
  }
}

async function startOAuthFlow() {
  if (!oauthConfig.value.clientId || !oauthConfig.value.clientSecret) {
    errorMessage.value = "Please enter both Client ID and Client Secret";
    return;
  }

  loading.value = true;
  errorMessage.value = null;

  try {
    const response = await startDriveOAuth(oauthConfig.value);
    oauthAuthUrl.value = response.authUrl;

    // Open browser for OAuth
    window.open(response.authUrl, '_blank');

    awaitingCallback.value = true;
    message.value = "Opening browser for Google authentication... Please complete the authorization and come back here.";

    // Store config in localStorage for callback handling
    localStorage.setItem("drive_oauth_config", JSON.stringify(oauthConfig.value));
    localStorage.setItem("drive_oauth_state", response.state);

  } catch (error) {
    errorMessage.value = `Failed to start OAuth: ${String(error)}`;
  } finally {
    loading.value = false;
  }
}

async function handleDisconnect() {
  loading.value = true;
  errorMessage.value = null;

  try {
    await disconnectDrive();
    message.value = "Disconnected from Google Drive successfully";
    await loadDriveStatus();
  } catch (error) {
    errorMessage.value = `Failed to disconnect: ${String(error)}`;
  } finally {
    loading.value = false;
  }
}

async function handleBackupToDrive() {
  loading.value = true;
  errorMessage.value = null;
  message.value = null;

  try {
    // Get backup data
    const backupData = await exportBackupData();
    const jsonContent = JSON.stringify(backupData, null, 2);

    // Generate filename
    const now = new Date();
    const timestamp = now.toISOString().slice(0, 16).replace('T', '_').replace(':', '-');
    const filename = `peptrack_backup_${timestamp}.json`;

    // Upload to Drive
    await uploadToDrive(filename, jsonContent);

    message.value = `✅ Backup uploaded to Google Drive successfully! (${backupData.metadata.protocolsCount} protocols, ${backupData.metadata.dosesCount} doses, ${backupData.metadata.literatureCount} papers)`;

  } catch (error) {
    errorMessage.value = `Failed to upload backup: ${String(error)}`;
  } finally {
    loading.value = false;
  }
}

onMounted(() => {
  loadDriveStatus();

  // Check if we're returning from OAuth (URL params would be handled by a callback server)
  // For now, this is a simple implementation
});
</script>

<template>
  <div class="drive-section">
    <div class="section-header">
      <h2>☁️ Google Drive Backup</h2>
      <p class="section-description">
        Automatically backup your data to Google Drive for extra safety.
      </p>
    </div>

    <!-- Not Connected State -->
    <div v-if="!driveStatus?.connected" class="drive-content">
      <div class="status-card disconnected">
        <div class="status-icon">⚠️</div>
        <div>
          <h3>Not Connected</h3>
          <p>Connect your Google Drive to enable automatic backups.</p>
        </div>
      </div>

      <div v-if="!showConfigForm" class="connect-prompt">
        <p class="setup-note">
          🔐 <strong>Privacy First:</strong> You'll need your own Google Cloud credentials.
          This ensures your data stays completely private and under your control.
        </p>

        <button @click="showConfigForm = true" class="primary-btn">
          🔗 Connect Google Drive
        </button>

        <div class="help-section">
          <p><strong>Need help setting this up?</strong></p>
          <a href="https://console.cloud.google.com/apis/credentials" target="_blank" class="help-link">
            📚 Get Google Cloud Credentials
          </a>
        </div>
      </div>

      <!-- OAuth Configuration Form -->
      <div v-else class="config-form">
        <div class="form-header">
          <h3>Google Cloud Configuration</h3>
          <button @click="showConfigForm = false" class="close-btn">✕</button>
        </div>

        <div class="setup-instructions">
          <p><strong>Follow these steps:</strong></p>
          <ol>
            <li>Go to <a href="https://console.cloud.google.com/apis/credentials" target="_blank">Google Cloud Console</a></li>
            <li>Create a new OAuth 2.0 Client ID (Desktop application)</li>
            <li>Add redirect URI: <code>http://localhost:8080/oauth/callback</code></li>
            <li>Copy your Client ID and Client Secret below</li>
          </ol>
        </div>

        <div class="form-group">
          <label for="clientId">Client ID</label>
          <input
            id="clientId"
            v-model="oauthConfig.clientId"
            type="text"
            placeholder="Your Google OAuth Client ID"
            :disabled="loading || awaitingCallback"
          />
        </div>

        <div class="form-group">
          <label for="clientSecret">Client Secret</label>
          <input
            id="clientSecret"
            v-model="oauthConfig.clientSecret"
            type="password"
            placeholder="Your Google OAuth Client Secret"
            :disabled="loading || awaitingCallback"
          />
        </div>

        <div v-if="awaitingCallback" class="waiting-message">
          <p>⏳ Waiting for you to complete authentication in your browser...</p>
          <p class="small-note">After authorizing, you'll need to manually paste the authorization code here.</p>
        </div>

        <button
          @click="startOAuthFlow"
          :disabled="loading || awaitingCallback"
          class="primary-btn"
        >
          {{ awaitingCallback ? "Waiting for Authorization..." : "🚀 Start Connection" }}
        </button>
      </div>
    </div>

    <!-- Connected State -->
    <div v-else class="drive-content">
      <div class="status-card connected">
        <div class="status-icon">✅</div>
        <div>
          <h3>Connected to Google Drive</h3>
          <p v-if="driveStatus.email">{{ driveStatus.email }}</p>
        </div>
      </div>

      <div class="actions">
        <button
          @click="handleBackupToDrive"
          :disabled="loading"
          class="backup-btn"
        >
          {{ loading ? "⏳ Uploading..." : "☁️ Backup to Drive Now" }}
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

    <!-- Messages -->
    <div v-if="message" class="message success">
      {{ message }}
    </div>

    <div v-if="errorMessage" class="message error">
      {{ errorMessage }}
    </div>
  </div>
</template>

<style scoped>
.drive-section {
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

.drive-content {
  margin-top: 20px;
}

.status-card {
  display: flex;
  align-items: center;
  gap: 16px;
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 20px;
}

.status-card.disconnected {
  background: #fff3cd;
  border: 1px solid #ffc107;
}

.status-card.connected {
  background: #d4edda;
  border: 1px solid #28a745;
}

.status-icon {
  font-size: 32px;
}

.status-card h3 {
  margin: 0 0 4px 0;
  font-size: 18px;
}

.status-card p {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.connect-prompt {
  text-align: center;
}

.setup-note {
  background: #e3f2fd;
  padding: 12px;
  border-radius: 6px;
  border-left: 3px solid #2196f3;
  margin-bottom: 16px;
  font-size: 14px;
}

.primary-btn {
  padding: 12px 24px;
  background-color: #2196f3;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  width: 100%;
  max-width: 300px;
}

.primary-btn:hover:not(:disabled) {
  background-color: #1976d2;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(33, 150, 243, 0.3);
}

.primary-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.help-section {
  margin-top: 20px;
  padding-top: 20px;
  border-top: 1px solid #eee;
}

.help-link {
  color: #2196f3;
  text-decoration: none;
  font-weight: 600;
}

.help-link:hover {
  text-decoration: underline;
}

.config-form {
  background: #f8f9fa;
  padding: 20px;
  border-radius: 8px;
}

.form-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 16px;
}

.form-header h3 {
  margin: 0;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #666;
}

.close-btn:hover {
  color: #333;
}

.setup-instructions {
  background: white;
  padding: 16px;
  border-radius: 6px;
  margin-bottom: 20px;
  border-left: 3px solid #2196f3;
}

.setup-instructions ol {
  margin: 8px 0 0 0;
  padding-left: 24px;
}

.setup-instructions li {
  margin: 6px 0;
}

.setup-instructions code {
  background: #f4f4f4;
  padding: 2px 6px;
  border-radius: 3px;
  font-family: monospace;
  font-size: 12px;
}

.form-group {
  margin-bottom: 16px;
}

.form-group label {
  display: block;
  margin-bottom: 6px;
  font-weight: 600;
  color: #333;
}

.form-group input {
  width: 100%;
  padding: 10px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
}

.form-group input:disabled {
  background: #f0f0f0;
  cursor: not-allowed;
}

.waiting-message {
  background: #fff3cd;
  padding: 12px;
  border-radius: 6px;
  margin-bottom: 16px;
}

.waiting-message p {
  margin: 4px 0;
}

.small-note {
  font-size: 12px;
  color: #666;
}

.actions {
  display: flex;
  gap: 12px;
}

.backup-btn {
  flex: 1;
  padding: 12px 24px;
  background-color: #28a745;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 16px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.backup-btn:hover:not(:disabled) {
  background-color: #218838;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(40, 167, 69, 0.3);
}

.backup-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.disconnect-btn {
  padding: 12px 24px;
  background-color: #dc3545;
  color: white;
  border: none;
  border-radius: 8px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.disconnect-btn:hover:not(:disabled) {
  background-color: #c82333;
}

.disconnect-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
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
