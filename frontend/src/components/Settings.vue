<script setup lang="ts">
import { ref } from "vue";
import BackupAndRestore from "./BackupAndRestore.vue";
import NotificationPreferences from "./NotificationPreferences.vue";
import CalendarIntegration from "./CalendarIntegration.vue";
import DarkModeToggle from "./DarkModeToggle.vue";
import DashboardWidgetSettings from "./DashboardWidgetSettings.vue";
import AboutHelp from "./AboutHelp.vue";

type Tab = "backup" | "notifications" | "calendar" | "appearance" | "dashboard" | "about";

const activeTab = ref<Tab>("backup");

function setActiveTab(tab: Tab) {
  activeTab.value = tab;
}

function testNotification() {
  try {
    if ("Notification" in window) {
      if (Notification.permission === 'granted') {
        new Notification('🔔 Test Notification', {
          body: 'This is a test notification from PepTrack!',
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
  <div class="settings">
    <div class="settings-header">
      <h1>⚙️ Settings</h1>
      <p class="subtitle">Configure backups, notifications, and preferences</p>
    </div>

    <!-- Tab Navigation -->
    <div class="settings-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'backup' }]"
        @click="setActiveTab('backup')"
      >
        <span class="tab-icon">📦</span>
        <span class="tab-label">Backup & Restore</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'notifications' }]"
        @click="setActiveTab('notifications')"
      >
        <span class="tab-icon">🔔</span>
        <span class="tab-label">Notifications</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'calendar' }]"
        @click="setActiveTab('calendar')"
      >
        <span class="tab-icon">📅</span>
        <span class="tab-label">Calendar</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'appearance' }]"
        @click="setActiveTab('appearance')"
      >
        <span class="tab-icon">🎨</span>
        <span class="tab-label">Appearance</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'dashboard' }]"
        @click="setActiveTab('dashboard')"
      >
        <span class="tab-icon">📊</span>
        <span class="tab-label">Dashboard</span>
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'about' }]"
        @click="setActiveTab('about')"
      >
        <span class="tab-icon">ℹ️</span>
        <span class="tab-label">About & Help</span>
      </button>
    </div>

    <!-- Tab Content -->
    <div class="settings-content">
      <!-- Backup & Restore Tab -->
      <div v-if="activeTab === 'backup'" class="tab-panel">
        <BackupAndRestore />
      </div>

      <!-- Notifications Tab -->
      <div v-if="activeTab === 'notifications'" class="tab-panel">
        <div class="notifications-section">
          <h2>🔔 Notification Preferences</h2>
          <p class="description">Configure when you want to receive notifications</p>

          <div class="notification-content">
            <NotificationPreferences />

            <div class="test-section">
              <h3>Test Notifications</h3>
              <p>Make sure notifications are working properly</p>
              <button @click="testNotification" class="test-btn">
                Send Test Notification
              </button>
            </div>
          </div>
        </div>
      </div>

      <!-- Calendar Tab -->
      <div v-if="activeTab === 'calendar'" class="tab-panel">
        <CalendarIntegration />
      </div>

      <!-- Appearance Tab -->
      <div v-if="activeTab === 'appearance'" class="tab-panel">
        <DarkModeToggle />
      </div>

      <!-- Dashboard Tab -->
      <div v-if="activeTab === 'dashboard'" class="tab-panel">
        <DashboardWidgetSettings />
      </div>

      <!-- About & Help Tab -->
      <div v-if="activeTab === 'about'" class="tab-panel">
        <AboutHelp />
      </div>
    </div>
  </div>
</template>

<style scoped>
.settings {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fafafa;
}

.settings-header {
  padding: 24px 24px 16px 24px;
  background: white;
  border-bottom: 2px solid #e0e0e0;
}

.settings-header h1 {
  font-size: 32px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.subtitle {
  font-size: 16px;
  color: #666;
  margin: 0;
}

/* Tab Navigation */
.settings-tabs {
  display: flex;
  gap: 4px;
  padding: 0 16px;
  background: white;
  border-bottom: 2px solid #e0e0e0;
}

.tab-btn {
  display: flex;
  align-items: center;
  gap: 8px;
  padding: 14px 24px;
  background: transparent;
  border: none;
  border-bottom: 3px solid transparent;
  font-size: 15px;
  font-weight: 600;
  color: #666;
  cursor: pointer;
  transition: all 0.2s;
  position: relative;
  top: 2px;
  white-space: nowrap;
}

.tab-btn:hover {
  color: #1976d2;
  background: #f5f5f5;
  border-radius: 8px 8px 0 0;
}

.tab-btn.active {
  color: #1976d2;
  border-bottom-color: #1976d2;
  background: white;
}

.tab-icon {
  font-size: 18px;
}

.tab-label {
  font-size: 14px;
}

/* Content Area */
.settings-content {
  flex: 1;
  overflow-y: auto;
  background: #fafafa;
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

/* Notifications Section */
.notifications-section {
  padding: 24px;
  max-width: 900px;
  margin: 0 auto;
}

.notifications-section h2 {
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

.notification-content {
  display: flex;
  flex-direction: column;
  gap: 24px;
}

.test-section {
  background: white;
  padding: 24px;
  border: 2px solid #e0e0e0;
  border-radius: 12px;
}

.test-section h3 {
  font-size: 18px;
  font-weight: 600;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.test-section p {
  margin: 0 0 16px 0;
  font-size: 14px;
  color: #666;
}

.test-btn {
  padding: 10px 20px;
  background: #1976d2;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.test-btn:hover {
  background: #1565c0;
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(25, 118, 210, 0.3);
}

/* Responsive */
@media (max-width: 768px) {
  .settings-tabs {
    overflow-x: auto;
    -webkit-overflow-scrolling: touch;
  }

  .tab-label {
    display: none;
  }

  .tab-icon {
    font-size: 24px;
  }
}

@media (prefers-color-scheme: dark) {
  .settings {
    background: #1a1a1a;
  }

  .settings-header,
  .settings-tabs {
    background: #2a2a2a;
    border-bottom-color: #3a3a3a;
  }

  .settings-header h1,
  .notifications-section h2,
  .test-section h3 {
    color: #fff;
  }

  .subtitle,
  .description,
  .test-section p {
    color: #aaa;
  }

  .tab-btn {
    color: #aaa;
  }

  .tab-btn:hover {
    color: #fff;
    background: #3a3a3a;
  }

  .tab-btn.active {
    color: #64b5f6;
    border-bottom-color: #64b5f6;
    background: #1a1a1a;
  }

  .settings-content {
    background: #1a1a1a;
  }

  .test-section {
    background: #2a2a2a;
    border-color: #3a3a3a;
  }
}
</style>
