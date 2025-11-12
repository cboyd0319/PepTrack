<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import Dashboard from "./components/Dashboard.vue";
import ProtocolList from "./components/ProtocolList.vue";
import ProtocolForm from "./components/ProtocolForm.vue";
import Research from "./components/Research.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import DoseTracker from "./components/DoseTracker.vue";
import Settings from "./components/Settings.vue";
import SupplierManagement from "./components/SupplierManagement.vue";
import InventoryManagement from "./components/InventoryManagement.vue";
import Toast from "./components/Toast.vue";

// Navigation
type View = "dashboard" | "doses" | "protocols" | "research" | "operations" | "settings";
const currentView = ref<View>("dashboard");

// Welcome screen ref
const welcomeScreen = ref<InstanceType<typeof WelcomeScreen> | null>(null);

// Connectivity status
const isOnline = ref(navigator.onLine);

import type { PeptideProtocol, CreateProtocolPayload } from "./api/peptrack";
import { listProtocols, saveProtocol, exportBackupData } from "./api/peptrack";

const protocols = ref<PeptideProtocol[]>([]);
const loadingProtocols = ref(false);
const savingProtocol = ref(false);
const errorMessage = ref<string | null>(null);

const form = ref({
  name: "",
  peptideName: "",
  notes: "",
  targetConcentration: "" as string | number,
});

async function refreshProtocols() {
  loadingProtocols.value = true;
  errorMessage.value = null;
  try {
    protocols.value = await listProtocols();
  } catch (error) {
    errorMessage.value = `Failed to load protocols: ${String(error)}`;
  } finally {
    loadingProtocols.value = false;
  }
}

async function handleCreateProtocol() {
  if (!form.value.name || !form.value.peptideName) {
    errorMessage.value = "Name and peptide fields are required.";
    return;
  }

  savingProtocol.value = true;
  errorMessage.value = null;
  const payload: CreateProtocolPayload = {
    name: form.value.name,
    peptideName: form.value.peptideName,
    notes: form.value.notes || undefined,
    targetConcentrationMgMl: form.value.targetConcentration
      ? Number(form.value.targetConcentration)
      : undefined,
  };
  try {
    await saveProtocol(payload);
    await refreshProtocols();
    form.value = {
      name: "",
      peptideName: "",
      notes: "",
      targetConcentration: "",
    };
  } catch (error) {
    errorMessage.value = `Failed to save protocol: ${String(error)}`;
  } finally {
    savingProtocol.value = false;
  }
}

// Navigation helpers for Dashboard quick actions
function handleNavigateToTab(tab: string) {
  currentView.value = tab as View;
}

function handleQuickLogDose() {
  currentView.value = "doses";
}

async function handleQuickBackup() {
  try {
    await exportBackupData();
    // Success message handled by the API
  } catch (error) {
    errorMessage.value = `Backup failed: ${String(error)}`;
  }
}

function showHelp() {
  welcomeScreen.value?.open();
}

// Network status detection
function updateOnlineStatus() {
  isOnline.value = navigator.onLine;
}

onMounted(() => {
  refreshProtocols();

  // Listen for connectivity changes
  window.addEventListener('online', updateOnlineStatus);
  window.addEventListener('offline', updateOnlineStatus);
});

onUnmounted(() => {
  window.removeEventListener('online', updateOnlineStatus);
  window.removeEventListener('offline', updateOnlineStatus);
});
</script>

<template>
  <Toast />
  <WelcomeScreen ref="welcomeScreen" />

  <main class="page">
    <header>
      <div class="header-content">
        <div>
          <h1>🧪 PepTrack</h1>
          <p class="subtitle">
            Track your peptides and research - all stored privately on your computer.
          </p>
        </div>
        <div class="header-actions">
          <div class="status-indicators">
            <span v-if="isOnline" class="status-badge online" title="Connected to internet">
              🌐 Online
            </span>
            <span v-else class="status-badge offline" title="No internet connection">
              ⚠️ Offline
            </span>
          </div>
          <button @click="showHelp" class="help-btn" title="Show help and welcome info">
            ❓ Help
          </button>
        </div>
      </div>
    </header>

    <!-- Main Navigation -->
    <nav class="main-nav">
      <button
        @click="currentView = 'dashboard'"
        :class="['nav-btn', { active: currentView === 'dashboard' }]"
        title="Dashboard overview"
      >
        <span class="nav-icon">🏠</span>
        <span class="nav-label">Dashboard</span>
      </button>
      <button
        @click="currentView = 'doses'"
        :class="['nav-btn', { active: currentView === 'doses' }]"
        title="Log and track doses"
      >
        <span class="nav-icon">💉</span>
        <span class="nav-label">Doses</span>
      </button>
      <button
        @click="currentView = 'protocols'"
        :class="['nav-btn', { active: currentView === 'protocols' }]"
        title="Manage peptide protocols"
      >
        <span class="nav-icon">🧪</span>
        <span class="nav-label">Protocols</span>
      </button>
      <button
        @click="currentView = 'research'"
        :class="['nav-btn', { active: currentView === 'research' }]"
        title="Literature search and AI summaries"
      >
        <span class="nav-icon">📚</span>
        <span class="nav-label">Research</span>
      </button>
      <button
        @click="currentView = 'operations'"
        :class="['nav-btn', { active: currentView === 'operations' }]"
        title="Manage suppliers and inventory"
      >
        <span class="nav-icon">🏢</span>
        <span class="nav-label">Operations</span>
      </button>
      <button
        @click="currentView = 'settings'"
        :class="['nav-btn', { active: currentView === 'settings' }]"
        title="Configure settings and backups"
      >
        <span class="nav-icon">⚙️</span>
        <span class="nav-label">Settings</span>
      </button>
    </nav>

    <section v-if="errorMessage" class="banner error">
      ⚠️ {{ errorMessage }}
    </section>

    <!-- Dashboard View (NEW) -->
    <div v-if="currentView === 'dashboard'" class="view-content full-height">
      <Dashboard
        @navigateToTab="handleNavigateToTab"
        @quickLogDose="handleQuickLogDose"
        @quickBackup="handleQuickBackup"
      />
    </div>

    <!-- Doses View -->
    <div v-if="currentView === 'doses'" class="view-content full-height">
      <DoseTracker />
    </div>

    <!-- Protocols View -->
    <div v-if="currentView === 'protocols'" class="view-content">
      <section class="grid">
        <ProtocolList
          :protocols="protocols"
          :loading="loadingProtocols"
          @refresh="refreshProtocols"
        />

        <ProtocolForm
          :form="form"
          :saving="savingProtocol"
          @submit="handleCreateProtocol"
          @update:name="form.name = $event"
          @update:peptideName="form.peptideName = $event"
          @update:notes="form.notes = $event"
          @update:targetConcentration="form.targetConcentration = $event"
        />
      </section>
    </div>

    <!-- Research View (NEW - combines AI + Literature) -->
    <div v-if="currentView === 'research'" class="view-content full-height">
      <Research />
    </div>

    <!-- Operations View -->
    <div v-if="currentView === 'operations'" class="view-content">
      <section class="grid">
        <SupplierManagement />
        <InventoryManagement />
      </section>
    </div>

    <!-- Settings View -->
    <div v-if="currentView === 'settings'" class="view-content full-height">
      <Settings />
    </div>
  </main>
</template>

<style scoped>
.page {
  max-width: 1800px;
  margin: 0 auto;
  padding: 20px;
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, Oxygen, Ubuntu, Cantarell, sans-serif;
}

header {
  margin-bottom: 24px;
}

header h1 {
  margin: 0;
  font-size: 36px;
  font-weight: 700;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  -webkit-background-clip: text;
  -webkit-text-fill-color: transparent;
  background-clip: text;
}

.subtitle {
  margin: 8px 0 0 0;
  color: #666;
  font-size: 16px;
}

.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
}

/* Main Navigation */
.main-nav {
  display: flex;
  gap: 8px;
  margin: 20px 0;
  padding: 8px;
  background: white;
  border-radius: 12px;
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.nav-btn {
  flex: 1;
  display: flex;
  align-items: center;
  justify-content: center;
  gap: 8px;
  padding: 12px 20px;
  border: none;
  background: transparent;
  border-radius: 8px;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
  font-weight: 600;
  color: #666;
}

.nav-btn:hover {
  background: #f8f9fa;
  transform: translateY(-1px);
}

.nav-btn.active {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.nav-icon {
  font-size: 20px;
}

.nav-label {
  font-weight: 600;
}

.view-content {
  animation: fadeIn 0.3s ease;
}

.view-content.full-height {
  min-height: calc(100vh - 280px);
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

.grid {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 20px;
  margin-top: 20px;
}

.help-btn {
  padding: 8px 16px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 14px;
  font-weight: 600;
  transition: all 0.2s;
  white-space: nowrap;
}

.help-btn:hover {
  background-color: #2980b9;
  transform: translateY(-1px);
  box-shadow: 0 2px 8px rgba(52, 152, 219, 0.3);
}

.help-btn:active {
  transform: translateY(0);
}

.header-actions {
  display: flex;
  align-items: center;
  gap: 12px;
}

.status-indicators {
  display: flex;
  gap: 8px;
}

.status-badge {
  padding: 4px 10px;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  white-space: nowrap;
}

.status-badge.online {
  background-color: #d4edda;
  color: #155724;
  border: 1px solid #c3e6cb;
}

.status-badge.offline {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
  animation: pulse 2s ease-in-out infinite;
}

@keyframes pulse {
  0%, 100% {
    opacity: 1;
  }
  50% {
    opacity: 0.6;
  }
}

.banner {
  padding: 12px 16px;
  border-radius: 8px;
  margin-bottom: 16px;
  font-weight: 600;
}

.banner.error {
  background-color: #f8d7da;
  color: #721c24;
  border: 1px solid #f5c6cb;
}

@media (max-width: 1200px) {
  .grid {
    grid-template-columns: 1fr;
  }
}

@media (max-width: 768px) {
  .header-content {
    flex-direction: column;
    align-items: stretch;
  }

  .header-actions {
    justify-content: space-between;
  }

  .help-btn {
    align-self: flex-end;
  }

  .main-nav {
    flex-wrap: wrap;
  }

  .nav-label {
    display: none;
  }

  .nav-btn {
    flex: 0 0 auto;
    padding: 12px 16px;
  }

  .nav-icon {
    font-size: 24px;
  }
}

@media (prefers-color-scheme: dark) {
  .page {
    background: #1a1a1a;
    color: #e0e0e0;
  }

  .subtitle {
    color: #aaa;
  }

  .main-nav {
    background: #2a2a2a;
  }

  .nav-btn {
    color: #aaa;
  }

  .nav-btn:hover {
    background: #3a3a3a;
  }

  .banner.error {
    background-color: #4a1a1a;
    color: #ff6b6b;
    border-color: #6a2a2a;
  }
}
</style>
