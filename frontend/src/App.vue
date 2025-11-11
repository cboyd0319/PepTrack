<script setup lang="ts">
import { onMounted, onUnmounted, ref } from "vue";
import ProtocolList from "./components/ProtocolList.vue";
import ProtocolForm from "./components/ProtocolForm.vue";
import AiSummaryPanel from "./components/AiSummaryPanel.vue";
import LiteratureSearch from "./components/LiteratureSearch.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import DoseTracker from "./components/DoseTracker.vue";
import Settings from "./components/Settings.vue";
import Toast from "./components/Toast.vue";

// Navigation
type View = "protocols" | "doses" | "ai" | "literature" | "settings";
const currentView = ref<View>("protocols");

// Welcome screen ref
const welcomeScreen = ref<InstanceType<typeof WelcomeScreen> | null>(null);

// Connectivity status
const isOnline = ref(navigator.onLine);

import type {
  PeptideProtocol,
  SummaryFormat,
  CreateProtocolPayload,
} from "./api/peptrack";
import {
  listProtocols,
  saveProtocol,
  summarizeContent,
} from "./api/peptrack";

const protocols = ref<PeptideProtocol[]>([]);
const loadingProtocols = ref(false);
const savingProtocol = ref(false);
const summaryOutput = ref<string | null>(null);
const summaryProvider = ref<string | null>(null);
const summarizing = ref(false);
const errorMessage = ref<string | null>(null);

const form = ref({
  name: "",
  peptideName: "",
  notes: "",
  targetConcentration: "" as string | number,
});

const summaryForm = ref({
  title: "",
  content: "",
  format: "Markdown" as SummaryFormat,
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

async function handleSummarize() {
  if (!summaryForm.value.title || !summaryForm.value.content) {
    errorMessage.value = "Provide both a title and content to summarize.";
    return;
  }
  summarizing.value = true;
  errorMessage.value = null;
  summaryOutput.value = null;
  summaryProvider.value = null;
  try {
    const result = await summarizeContent({
      title: summaryForm.value.title,
      content: summaryForm.value.content,
      format: summaryForm.value.format,
    });
    summaryProvider.value = result.provider;
    summaryOutput.value = result.output;
  } catch (error) {
    errorMessage.value = `Summarization failed: ${String(error)}`;
  } finally {
    summarizing.value = false;
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
            Keep track of your peptides and research - all stored privately on your computer.
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
        @click="currentView = 'protocols'"
        :class="['nav-btn', { active: currentView === 'protocols' }]"
      >
        <span class="nav-icon">📋</span>
        <span class="nav-label">Protocols</span>
      </button>
      <button
        @click="currentView = 'doses'"
        :class="['nav-btn', { active: currentView === 'doses' }]"
      >
        <span class="nav-icon">💉</span>
        <span class="nav-label">Doses</span>
      </button>
      <button
        @click="currentView = 'ai'"
        :class="['nav-btn', { active: currentView === 'ai' }]"
      >
        <span class="nav-icon">🤖</span>
        <span class="nav-label">AI Summary</span>
      </button>
      <button
        @click="currentView = 'literature'"
        :class="['nav-btn', { active: currentView === 'literature' }]"
      >
        <span class="nav-icon">📚</span>
        <span class="nav-label">Literature</span>
      </button>
      <button
        @click="currentView = 'settings'"
        :class="['nav-btn', { active: currentView === 'settings' }]"
      >
        <span class="nav-icon">⚙️</span>
        <span class="nav-label">Settings</span>
      </button>
    </nav>

    <section v-if="errorMessage" class="banner error">
      ⚠️ {{ errorMessage }}
    </section>

    <!-- Protocols View -->
    <div v-if="currentView === 'protocols'" class="view-content">
      <section class="grid">
        <ProtocolList
          :protocols="protocols"
          :loading="loadingProtocols"
          @refresh="refreshProtocols"
        />

        <ProtocolForm :form="form" :saving="savingProtocol" @submit="handleCreateProtocol" />
      </section>
    </div>

    <!-- Doses View -->
    <div v-if="currentView === 'doses'" class="view-content">
      <DoseTracker />
    </div>

    <!-- AI Summary View -->
    <div v-if="currentView === 'ai'" class="view-content">
      <AiSummaryPanel
        :form="summaryForm"
        :summarizing="summarizing"
        :summary-output="summaryOutput"
        :summary-provider="summaryProvider"
        @summarize="handleSummarize"
      />
    </div>

    <!-- Literature View -->
    <div v-if="currentView === 'literature'" class="view-content">
      <LiteratureSearch />
    </div>

    <!-- Settings View -->
    <div v-if="currentView === 'settings'" class="view-content">
      <Settings />
    </div>
  </main>
</template>

<style scoped>
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
}
</style>
