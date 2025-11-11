<script setup lang="ts">
import { onMounted, ref } from "vue";
import ProtocolList from "./components/ProtocolList.vue";
import ProtocolForm from "./components/ProtocolForm.vue";
import AiSummaryPanel from "./components/AiSummaryPanel.vue";
import LiteratureSearch from "./components/LiteratureSearch.vue";
import WelcomeScreen from "./components/WelcomeScreen.vue";
import DoseTracker from "./components/DoseTracker.vue";
import BackupExport from "./components/BackupExport.vue";
import GoogleDriveBackup from "./components/GoogleDriveBackup.vue";
import ScheduledBackup from "./components/ScheduledBackup.vue";

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
</script>

<template>
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

    <section v-if="errorMessage" class="banner error">
      ⚠️ {{ errorMessage }}
    </section>

    <section class="grid">
      <ProtocolList
        :protocols="protocols"
        :loading="loadingProtocols"
        @refresh="refreshProtocols"
      />

      <ProtocolForm :form="form" :saving="savingProtocol" @submit="handleCreateProtocol" />
    </section>

    <section class="dose-section">
      <DoseTracker />
    </section>

    <AiSummaryPanel
      :form="summaryForm"
      :summarizing="summarizing"
      :summary-output="summaryOutput"
      :summary-provider="summaryProvider"
      @summarize="handleSummarize"
    />

    <section class="literature-section">
      <LiteratureSearch />
    </section>

    <section class="backup-section">
      <BackupExport />
    </section>

    <section class="drive-section">
      <GoogleDriveBackup />
    </section>

    <section class="scheduled-backup-section">
      <ScheduledBackup />
    </section>
  </main>
</template>

<style scoped>
.header-content {
  display: flex;
  justify-content: space-between;
  align-items: flex-start;
  gap: 20px;
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
