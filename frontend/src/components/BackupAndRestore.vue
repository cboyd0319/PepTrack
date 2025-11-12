<template>
  <div class="backup-restore">
    <h2>📦 Backup & Restore</h2>
    <p class="description">Protect your data with automated backups and cloud sync</p>

    <!-- Sub-tabs for different backup sections -->
    <div class="backup-tabs">
      <button
        :class="['backup-tab-btn', { active: activeSection === 'quick' }]"
        @click="activeSection = 'quick'"
      >
        ⚡ Quick Backup
      </button>
      <button
        :class="['backup-tab-btn', { active: activeSection === 'cloud' }]"
        @click="activeSection = 'cloud'"
      >
        ☁️ Cloud Sync
      </button>
      <button
        :class="['backup-tab-btn', { active: activeSection === 'scheduled' }]"
        @click="activeSection = 'scheduled'"
      >
        🕐 Scheduled
      </button>
      <button
        :class="['backup-tab-btn', { active: activeSection === 'restore' }]"
        @click="activeSection = 'restore'"
      >
        📥 Restore
      </button>
    </div>

    <!-- Section Content -->
    <div class="section-content">
      <!-- Quick Backup -->
      <div v-if="activeSection === 'quick'" class="section">
        <BackupExport />
      </div>

      <!-- Cloud Sync -->
      <div v-if="activeSection === 'cloud'" class="section">
        <div class="cloud-sync-section">
          <h3>☁️ Cloud Storage Options</h3>
          <p class="section-description">Automatically sync your backups to the cloud</p>

          <GoogleDriveBackup />

          <!-- Future cloud providers -->
          <div class="coming-soon-section">
            <h4>Coming Soon</h4>
            <div class="provider-grid">
              <div class="provider-card disabled">
                <div class="provider-icon">📧</div>
                <div class="provider-name">Email Backup</div>
                <div class="provider-status">Coming Soon</div>
              </div>
              <div class="provider-card disabled">
                <div class="provider-icon">📁</div>
                <div class="provider-name">Dropbox</div>
                <div class="provider-status">Coming Soon</div>
              </div>
              <div class="provider-card disabled">
                <div class="provider-icon">☁️</div>
                <div class="provider-name">OneDrive</div>
                <div class="provider-status">Coming Soon</div>
              </div>
            </div>
          </div>
        </div>
      </div>

      <!-- Scheduled Backups -->
      <div v-if="activeSection === 'scheduled'" class="section">
        <ScheduledBackup />
      </div>

      <!-- Restore -->
      <div v-if="activeSection === 'restore'" class="section">
        <RestoreBackup />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue';
import BackupExport from './BackupExport.vue';
import GoogleDriveBackup from './GoogleDriveBackup.vue';
import ScheduledBackup from './ScheduledBackup.vue';
import RestoreBackup from './RestoreBackup.vue';

const activeSection = ref<'quick' | 'cloud' | 'scheduled' | 'restore'>('quick');
</script>

<style scoped>
.backup-restore {
  padding: 24px;
  max-width: 1200px;
}

.backup-restore h2 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.description {
  font-size: 15px;
  color: #666;
  margin: 0 0 24px 0;
}

/* Backup Tabs */
.backup-tabs {
  display: flex;
  gap: 8px;
  margin-bottom: 24px;
  border-bottom: 2px solid #e0e0e0;
  flex-wrap: wrap;
}

.backup-tab-btn {
  padding: 12px 20px;
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  font-size: 14px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
  top: 2px;
  white-space: nowrap;
}

.backup-tab-btn:hover {
  color: #1976d2;
  background: #f5f5f5;
  border-radius: 8px 8px 0 0;
}

.backup-tab-btn.active {
  color: #1976d2;
  border-bottom-color: #1976d2;
}

.section-content {
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  padding: 24px;
}

.section {
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

/* Cloud Sync Section */
.cloud-sync-section h3 {
  font-size: 20px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.section-description {
  font-size: 14px;
  color: #666;
  margin: 0 0 24px 0;
}

.coming-soon-section {
  margin-top: 40px;
  padding-top: 24px;
  border-top: 2px solid #e0e0e0;
}

.coming-soon-section h4 {
  font-size: 16px;
  font-weight: 600;
  margin: 0 0 16px 0;
  color: #666;
}

.provider-grid {
  display: grid;
  grid-template-columns: repeat(auto-fill, minmax(180px, 1fr));
  gap: 16px;
}

.provider-card {
  padding: 20px;
  background: white;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
  text-align: center;
  transition: all 0.2s;
}

.provider-card.disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.provider-icon {
  font-size: 40px;
  margin-bottom: 12px;
}

.provider-name {
  font-weight: 600;
  margin-bottom: 4px;
  color: #1a1a1a;
}

.provider-status {
  font-size: 12px;
  color: #ff9800;
  font-weight: 600;
}

@media (prefers-color-scheme: dark) {
  .backup-restore h2,
  .cloud-sync-section h3,
  .provider-name {
    color: #fff;
  }

  .description,
  .section-description {
    color: #aaa;
  }

  .backup-tabs {
    border-bottom-color: #3a3a3a;
  }

  .backup-tab-btn {
    color: #aaa;
  }

  .backup-tab-btn:hover {
    color: #fff;
    background: #3a3a3a;
  }

  .backup-tab-btn.active {
    color: #64b5f6;
    border-bottom-color: #64b5f6;
  }

  .section-content,
  .provider-card {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }

  .coming-soon-section {
    border-top-color: #3a3a3a;
  }

  .coming-soon-section h4 {
    color: #aaa;
  }
}
</style>
