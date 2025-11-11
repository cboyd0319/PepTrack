<script setup lang="ts">
import { onMounted, ref } from "vue";
import ProtocolList from "./components/ProtocolList.vue";
import ProtocolForm from "./components/ProtocolForm.vue";
import AiSummaryPanel from "./components/AiSummaryPanel.vue";
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

onMounted(() => {
  refreshProtocols();
});
</script>

<template>
  <main class="page">
    <header>
      <h1>PepTrack</h1>
      <p class="subtitle">
        Privacy-first peptide protocol management with local AI research assistance.
      </p>
    </header>

    <section v-if="errorMessage" class="banner error">
      {{ errorMessage }}
    </section>

    <section class="grid">
      <ProtocolList
        :protocols="protocols"
        :loading="loadingProtocols"
        @refresh="refreshProtocols"
      />

      <ProtocolForm :form="form" :saving="savingProtocol" @submit="handleCreateProtocol" />
    </section>

    <AiSummaryPanel
      :form="summaryForm"
      :summarizing="summarizing"
      :summary-output="summaryOutput"
      :summary-provider="summaryProvider"
      @summarize="handleSummarize"
    />
  </main>
</template>
