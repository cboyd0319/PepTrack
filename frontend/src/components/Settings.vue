<script setup lang="ts">
import { ref } from "vue";
import ScheduledBackup from "./ScheduledBackup.vue";
import GoogleDriveBackup from "./GoogleDriveBackup.vue";
import BackupExport from "./BackupExport.vue";
import RestoreBackup from "./RestoreBackup.vue";
import NotificationPreferences from "./NotificationPreferences.vue";

type Tab = "scheduled" | "drive" | "backup" | "restore" | "notifications";

const activeTab = ref<Tab>("scheduled");

const tabs = [
  { id: "scheduled" as Tab, label: "⏰ Scheduled Backups", description: "Automatic backup scheduling" },
  { id: "drive" as Tab, label: "☁️ Google Drive", description: "Cloud backup setup" },
  { id: "backup" as Tab, label: "💾 Manual Backup", description: "Export data manually" },
  { id: "restore" as Tab, label: "📥 Restore", description: "Restore from backup" },
  { id: "notifications" as Tab, label: "🔔 Notifications", description: "Alert preferences" },
];

function setActiveTab(tab: Tab) {
  activeTab.value = tab;
}

function handleTestNotification() {
  // Send a test notification
  try {
    if ('Notification' in window) {
      if (Notification.permission === 'granted') {
        new Notification('🔔 Test Notification', {
          body: 'This is a test notification from PepTrack!',
          icon: '/icon.png', // Optional: add your app icon
        });
      } else if (Notification.permission !== 'denied') {
        Notification.requestPermission().then((permission) => {
          if (permission === 'granted') {
            new Notification('🔔 Test Notification', {
              body: 'This is a test notification from PepTrack!',
            });
          }
        }).catch((error) => {
          console.error('Failed to request notification permission:', error);
        });
      }
    }
  } catch (error) {
    console.error('Notification API error:', error);
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <h1>⚙️ Settings & Backups</h1>
      <p class="header-description">
        Manage your backup settings, cloud storage, and data restoration
      </p>
    </div>

    <div class="tabs-container">
      <div class="tabs">
        <button
          v-for="tab in tabs"
          :key="tab.id"
          @click="setActiveTab(tab.id)"
          :class="['tab-button', { active: activeTab === tab.id }]"
        >
          <div class="tab-label">{{ tab.label }}</div>
          <div class="tab-desc">{{ tab.description }}</div>
        </button>
      </div>

      <div class="tab-content">
        <transition name="fade" mode="out-in">
          <div v-if="activeTab === 'scheduled'" key="scheduled" class="tab-panel">
            <ScheduledBackup />
          </div>

          <div v-else-if="activeTab === 'drive'" key="drive" class="tab-panel">
            <GoogleDriveBackup />
          </div>

          <div v-else-if="activeTab === 'backup'" key="backup" class="tab-panel">
            <BackupExport />
          </div>

          <div v-else-if="activeTab === 'restore'" key="restore" class="tab-panel">
            <RestoreBackup />
          </div>

          <div v-else-if="activeTab === 'notifications'" key="notifications" class="tab-panel">
            <NotificationPreferences @test-notification="handleTestNotification" />
          </div>
        </transition>
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 1200px;
  margin: 0 auto;
  padding: 20px;
}

.settings-header {
  text-align: center;
  margin-bottom: 32px;
}

.settings-header h1 {
  margin: 0 0 8px 0;
  font-size: 32px;
  color: #2c3e50;
}

.header-description {
  margin: 0;
  color: #666;
  font-size: 16px;
}

.tabs-container {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

.tabs {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
  border-bottom: 2px solid #e9ecef;
  background: #f8f9fa;
}

.tab-button {
  padding: 16px 20px;
  border: none;
  background: transparent;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  border-bottom: 3px solid transparent;
  position: relative;
}

.tab-button:hover {
  background: #e9ecef;
}

.tab-button.active {
  background: white;
  border-bottom-color: #007bff;
}

.tab-button.active .tab-label {
  color: #007bff;
}

.tab-label {
  font-size: 16px;
  font-weight: 600;
  color: #333;
  margin-bottom: 4px;
  transition: color 0.2s;
}

.tab-desc {
  font-size: 12px;
  color: #666;
}

.tab-content {
  padding: 24px;
  min-height: 400px;
}

.tab-panel {
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

.fade-enter-active,
.fade-leave-active {
  transition: opacity 0.2s ease, transform 0.2s ease;
}

.fade-enter-from {
  opacity: 0;
  transform: translateY(10px);
}

.fade-leave-to {
  opacity: 0;
  transform: translateY(-10px);
}

@media (max-width: 768px) {
  .tabs {
    grid-template-columns: 1fr 1fr;
  }

  .tab-button {
    padding: 12px 16px;
  }

  .tab-label {
    font-size: 14px;
  }

  .tab-desc {
    font-size: 11px;
  }

  .tab-content {
    padding: 16px;
  }
}
</style>
