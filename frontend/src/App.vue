<script setup lang="ts">
import { onMounted, ref } from "vue";
import type {
  PeptideProtocol,
  SummaryFormat,
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
  try {
    await saveProtocol({
      name: form.value.name,
      peptideName: form.value.peptideName,
      notes: form.value.notes || undefined,
      targetConcentrationMgMl: form.value.targetConcentration
        ? Number(form.value.targetConcentration)
        : undefined,
    });
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
      <article class="panel">
        <div class="panel-header">
          <h2>Active Protocols</h2>
          <button @click="refreshProtocols" :disabled="loadingProtocols">
            {{ loadingProtocols ? "Refreshing..." : "Refresh" }}
          </button>
        </div>
        <p v-if="!protocols.length && !loadingProtocols" class="muted">
          No protocols yet. Create one to get started.
        </p>
        <ul v-else class="protocol-list">
          <li v-for="protocol in protocols" :key="protocol.id">
            <div class="protocol-title">{{ protocol.name }}</div>
            <div class="protocol-meta">
              <span>{{ protocol.peptide_name }}</span>
              <span>
                Updated:
                {{ new Date(protocol.updated_at).toLocaleString() }}
              </span>
            </div>
            <p class="protocol-notes" v-if="protocol.notes">
              {{ protocol.notes }}
            </p>
          </li>
        </ul>
      </article>

      <article class="panel">
        <div class="panel-header">
          <h2>New Protocol</h2>
        </div>
        <form class="form-stack" @submit.prevent="handleCreateProtocol">
          <label>
            Name
            <input v-model="form.name" type="text" placeholder="Protocol name" />
          </label>
          <label>
            Peptide
            <input
              v-model="form.peptideName"
              type="text"
              placeholder="BPC-157, TB-500..."
            />
          </label>
          <label>
            Target concentration (mg/mL)
            <input
              v-model="form.targetConcentration"
              type="number"
              min="0"
              step="0.01"
            />
          </label>
          <label>
            Notes
            <textarea
              v-model="form.notes"
              rows="3"
              placeholder="Reconstitution details, supplier info..."
            />
          </label>
          <button class="primary" type="submit" :disabled="savingProtocol">
            {{ savingProtocol ? "Saving..." : "Save protocol" }}
          </button>
        </form>
      </article>
    </section>

    <section class="panel">
      <div class="panel-header">
        <div>
          <h2>AI Literature Summary</h2>
          <p class="muted">
            Uses local Codex CLI by default; falls back to Claude CLI (claude-haiku-4-5).
          </p>
        </div>
      </div>
      <form class="form-stack" @submit.prevent="handleSummarize">
        <label>
          Title
          <input v-model="summaryForm.title" type="text" placeholder="Paper title" />
        </label>
        <label>
          Format
          <select v-model="summaryForm.format">
            <option value="Markdown">Markdown</option>
            <option value="Json">JSON</option>
          </select>
        </label>
        <label>
          Content
          <textarea
            v-model="summaryForm.content"
            rows="6"
            placeholder="Paste abstract, key findings, or raw notes"
          />
        </label>
        <button class="primary" type="submit" :disabled="summarizing">
          {{ summarizing ? "Summarizing..." : "Summarize" }}
        </button>
      </form>

      <div v-if="summaryOutput" class="summary-output">
        <div class="summary-meta">
          Generated by: {{ summaryProvider }}
        </div>
        <pre>{{ summaryOutput }}</pre>
      </div>
    </section>
  </main>
</template>
