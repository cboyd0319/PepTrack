<script setup lang="ts">
import { ref } from "vue";
import ScheduledBackup from "./ScheduledBackup.vue";
import GoogleDriveBackup from "./GoogleDriveBackup.vue";
import BackupExport from "./BackupExport.vue";
import RestoreBackup from "./RestoreBackup.vue";
import NotificationPreferences from "./NotificationPreferences.vue";
import SupplierManagement from "./SupplierManagement.vue";
import InventoryManagement from "./InventoryManagement.vue";

type Tab = "scheduled" | "drive" | "backup" | "restore" | "notifications" | "suppliers" | "inventory";

const activeTab = ref<Tab>("scheduled");

const tabs = [
  { id: "scheduled" as Tab, label: "⏰ Scheduled Backups", description: "Automatic backup scheduling" },
  { id: "drive" as Tab, label: "☁️ Google Drive", description: "Cloud backup setup" },
  { id: "backup" as Tab, label: "💾 Manual Backup", description: "Export data manually" },
  { id: "restore" as Tab, label: "📥 Restore", description: "Restore from backup" },
  { id: "suppliers" as Tab, label: "🏢 Suppliers", description: "Manage suppliers and vendors" },
  { id: "inventory" as Tab, label: "📦 Inventory", description: "Track peptide vials and stock" },
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
      <h1>⚙️ Settings</h1>
      <p class="header-description">
        Manage your backup settings, cloud storage, and data
      </p>
    </div>

    <div class="settings-layout">
      <!-- Sidebar Navigation -->
      <aside class="sidebar">
        <div class="sidebar-section">
          <div class="section-title">Backup & Restore</div>
          <button
            @click="setActiveTab('scheduled')"
            :class="['sidebar-item', { active: activeTab === 'scheduled' }]"
          >
            <span class="item-icon">⏰</span>
            <div class="item-content">
              <div class="item-label">Scheduled Backups</div>
              <div class="item-desc">Automatic scheduling</div>
            </div>
          </button>
          <button
            @click="setActiveTab('drive')"
            :class="['sidebar-item', { active: activeTab === 'drive' }]"
          >
            <span class="item-icon">☁️</span>
            <div class="item-content">
              <div class="item-label">Google Drive</div>
              <div class="item-desc">Cloud backup setup</div>
            </div>
          </button>
          <button
            @click="setActiveTab('backup')"
            :class="['sidebar-item', { active: activeTab === 'backup' }]"
          >
            <span class="item-icon">💾</span>
            <div class="item-content">
              <div class="item-label">Manual Backup</div>
              <div class="item-desc">Export data manually</div>
            </div>
          </button>
          <button
            @click="setActiveTab('restore')"
            :class="['sidebar-item', { active: activeTab === 'restore' }]"
          >
            <span class="item-icon">📥</span>
            <div class="item-content">
              <div class="item-label">Restore</div>
              <div class="item-desc">Restore from backup</div>
            </div>
          </button>
        </div>

        <div class="sidebar-section">
          <div class="section-title">Inventory Management</div>
          <button
            @click="setActiveTab('suppliers')"
            :class="['sidebar-item', { active: activeTab === 'suppliers' }]"
          >
            <span class="item-icon">🏢</span>
            <div class="item-content">
              <div class="item-label">Suppliers</div>
              <div class="item-desc">Manage vendors</div>
            </div>
          </button>
          <button
            @click="setActiveTab('inventory')"
            :class="['sidebar-item', { active: activeTab === 'inventory' }]"
          >
            <span class="item-icon">📦</span>
            <div class="item-content">
              <div class="item-label">Inventory</div>
              <div class="item-desc">Track vials & stock</div>
            </div>
          </button>
        </div>

        <div class="sidebar-section">
          <div class="section-title">Preferences</div>
          <button
            @click="setActiveTab('notifications')"
            :class="['sidebar-item', { active: activeTab === 'notifications' }]"
          >
            <span class="item-icon">🔔</span>
            <div class="item-content">
              <div class="item-label">Notifications</div>
              <div class="item-desc">Alert preferences</div>
            </div>
          </button>
        </div>
      </aside>

      <!-- Main Content Area -->
      <main class="content-area">
        <transition name="fade" mode="out-in">
          <div v-if="activeTab === 'scheduled'" key="scheduled" class="content-panel">
            <ScheduledBackup />
          </div>

          <div v-else-if="activeTab === 'drive'" key="drive" class="content-panel">
            <GoogleDriveBackup />
          </div>

          <div v-else-if="activeTab === 'backup'" key="backup" class="content-panel">
            <BackupExport />
          </div>

          <div v-else-if="activeTab === 'restore'" key="restore" class="content-panel">
            <RestoreBackup />
          </div>

          <div v-else-if="activeTab === 'suppliers'" key="suppliers" class="content-panel">
            <SupplierManagement />
          </div>

          <div v-else-if="activeTab === 'inventory'" key="inventory" class="content-panel">
            <InventoryManagement />
          </div>

          <div v-else-if="activeTab === 'notifications'" key="notifications" class="content-panel">
            <NotificationPreferences @test-notification="handleTestNotification" />
          </div>
        </transition>
      </main>
    </div>
  </div>
</template>

<style scoped>
.settings-page {
  max-width: 100vw;
  width: 100%;
  margin: 0 auto;
  padding: 20px;
  min-height: 100vh;
  box-sizing: border-box;
  overflow-x: hidden;
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

/* Sidebar Layout */
.settings-layout {
  display: flex;
  gap: 24px;
  align-items: start;
  width: 100%;
}

/* Sidebar */
.sidebar {
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
  padding: 16px;
  position: sticky;
  top: 20px;
  max-height: calc(100vh - 100px);
  overflow-y: auto;
  width: 280px;
  min-width: 280px;
  max-width: 280px;
  flex-shrink: 0;
}

.sidebar-section {
  margin-bottom: 24px;
}

.sidebar-section:last-child {
  margin-bottom: 0;
}

.section-title {
  font-size: 11px;
  font-weight: 700;
  text-transform: uppercase;
  letter-spacing: 0.5px;
  color: #999;
  margin-bottom: 8px;
  padding: 0 12px;
}

.sidebar-item {
  width: 100%;
  display: flex;
  align-items: center;
  gap: 12px;
  padding: 12px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  text-align: left;
  margin-bottom: 4px;
}

.sidebar-item:hover {
  background: #f8f9fa;
}

.sidebar-item.active {
  background: #e7f3ff;
  border-left: 3px solid #007bff;
  padding-left: 9px;
}

.sidebar-item.active .item-label {
  color: #007bff;
  font-weight: 600;
}

.item-icon {
  font-size: 24px;
  flex-shrink: 0;
  width: 32px;
  text-align: center;
}

.item-content {
  flex: 1;
  min-width: 0;
}

.item-label {
  font-size: 14px;
  font-weight: 500;
  color: #333;
  margin-bottom: 2px;
  transition: all 0.2s;
}

.item-desc {
  font-size: 11px;
  color: #999;
  line-height: 1.3;
}

/* Content Area */
.content-area {
  flex: 1;
  min-width: 0;
  min-height: 600px;
  overflow-x: hidden;
}

.content-panel {
  animation: fadeIn 0.3s ease;
  max-width: 100%;
  overflow-x: hidden;
}

/* Force all child sections to respect the container */
.content-panel > * {
  max-width: 100%;
  box-sizing: border-box;
  margin: 0 !important;
}

/* Force tables to respect container width */
.content-panel table {
  table-layout: fixed;
  width: 100%;
  word-wrap: break-word;
}

.content-panel table td,
.content-panel table th {
  overflow-wrap: break-word;
  word-wrap: break-word;
  hyphens: auto;
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

/* Responsive: Stack on mobile */
@media (max-width: 768px) {
  .settings-page {
    max-width: 100%;
    padding: 12px;
  }

  .settings-layout {
    grid-template-columns: 1fr;
    gap: 16px;
  }

  .sidebar {
    position: static;
    max-height: none;
  }

  .content-area {
    padding: 20px;
  }

  .item-desc {
    display: none;
  }
}
</style>
