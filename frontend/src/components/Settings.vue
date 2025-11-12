<script setup lang="ts">
import { ref } from "vue";
import ScheduledBackup from "./ScheduledBackup.vue";
import GoogleDriveBackup from "./GoogleDriveBackup.vue";
import BackupExport from "./BackupExport.vue";
import RestoreBackup from "./RestoreBackup.vue";
import NotificationPreferences from "./NotificationPreferences.vue";
import SupplierManagement from "./SupplierManagement.vue";
import InventoryManagement from "./InventoryManagement.vue";

type Tab =
  | "scheduled"
  | "drive"
  | "backup"
  | "restore"
  | "notifications"
  | "suppliers"
  | "inventory";

interface TabConfig {
  id: Tab;
  label: string;
  description: string;
}

const activeTab = ref<Tab>("scheduled");

const tabGroups: Array<{ label: string; tabs: TabConfig[] }> = [
  {
    label: "Backup & Restore",
    tabs: [
      { id: "scheduled", label: "⏰ Scheduled Backups", description: "Automatic backup scheduling" },
      { id: "drive", label: "☁️ Google Drive", description: "Cloud backup setup" },
      { id: "backup", label: "💾 Manual Backup", description: "Export data manually" },
      { id: "restore", label: "📥 Restore", description: "Restore from backup" },
    ],
  },
  {
    label: "Operations & Alerts",
    tabs: [
      { id: "suppliers", label: "🏢 Suppliers", description: "Manage suppliers and vendors" },
      { id: "inventory", label: "📦 Inventory", description: "Track peptide vials and stock" },
      { id: "notifications", label: "🔔 Notifications", description: "Alert preferences" },
    ],
  },
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
        }).catch((_error) => {
          // Notification permission request failed, user will need to enable manually
        });
      }
    }
  } catch (_error) {
    // Notification API not available or blocked
  }
}
</script>

<template>
  <div class="settings-page">
    <div class="settings-header">
      <h1>⚙️ Settings</h1>
      <p class="header-description">
        Manage your backup settings, cloud storage, and data
      </p>
    </div>

    <div class="tabs-container">
      <div class="tabs-scroll-wrapper">
        <div class="tabs">
          <div
            v-for="group in tabGroups"
            :key="group.label"
            class="tab-group"
          >
            <p class="tab-group-label">{{ group.label }}</p>
            <div class="tab-group-buttons">
              <button
                v-for="tab in group.tabs"
                :key="tab.id"
                @click="setActiveTab(tab.id)"
                :class="['tab-button', { active: activeTab === tab.id }]"
              >
                <span class="tab-icon">{{ tab.label.split(' ')[0] }}</span>
                <div class="tab-info">
                  <div class="tab-label">{{ tab.label.substring(tab.label.indexOf(' ') + 1) }}</div>
                  <div class="tab-desc">{{ tab.description }}</div>
                </div>
              </button>
            </div>
          </div>
        </div>
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

          <div v-else-if="activeTab === 'suppliers'" key="suppliers" class="tab-panel">
            <SupplierManagement />
          </div>

          <div v-else-if="activeTab === 'inventory'" key="inventory" class="tab-panel">
            <InventoryManagement />
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
  max-width: 1400px;
  margin: 0 auto;
  padding: 20px;
  min-height: 100vh;
}

.settings-header {
  margin-bottom: 24px;
}

.settings-header h1 {
  margin: 0 0 4px 0;
  font-size: 32px;
  color: #2c3e50;
}

.header-description {
  margin: 0;
  color: #666;
  font-size: 14px;
}

.tabs-container {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  overflow: hidden;
}

/* Horizontal scrollable tabs */
.tabs-scroll-wrapper {
  overflow-x: auto;
  overflow-y: hidden;
  border-bottom: 2px solid #e0e0e0;
  background: #f8f9fa;
}

.tabs {
  display: flex;
  flex-direction: column;
  min-width: min-content;
  padding: 12px;
  gap: 16px;
}

.tab-group {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

.tab-group-label {
  font-size: 12px;
  font-weight: 700;
  text-transform: uppercase;
  color: #6b7280;
  letter-spacing: 0.05em;
}

.tab-group-buttons {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.tab-button {
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px 20px;
  border: none;
  background: white;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
  flex-shrink: 0;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.tab-button:hover {
  transform: translateY(-2px);
  box-shadow: 0 4px 8px rgba(0, 0, 0, 0.15);
}

.tab-button.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.tab-button.active .tab-label,
.tab-button.active .tab-desc {
  color: white;
}

.tab-icon {
  font-size: 24px;
  flex-shrink: 0;
}

.tab-info {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  gap: 2px;
}

.tab-label {
  font-size: 14px;
  font-weight: 600;
  color: #333;
}

.tab-desc {
  font-size: 11px;
  color: #666;
}

/* Content area */
.tab-content {
  padding: 24px;
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

/* Responsive */
@media (max-width: 768px) {
  .settings-page {
    padding: 12px;
  }

  .tabs {
    padding: 8px;
  }

  .tab-button {
    padding: 10px 16px;
  }

  .tab-desc {
    display: none;
  }

  .tab-content {
    padding: 16px;
  }
}
</style>
