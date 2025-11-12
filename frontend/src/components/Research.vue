<template>
  <div class="research">
    <!-- Tab Navigation -->
    <div class="research-tabs">
      <button
        :class="['tab-btn', { active: activeTab === 'literature' }]"
        @click="activeTab = 'literature'"
      >
        📚 Literature Search
      </button>
      <button
        :class="['tab-btn', { active: activeTab === 'ai' }]"
        @click="activeTab = 'ai'"
      >
        🤖 AI Summary
      </button>
    </div>

    <!-- Tab Content -->
    <div class="tab-content">
      <!-- Literature Search Tab -->
      <div v-show="activeTab === 'literature'" class="tab-panel">
        <LiteratureSearch @request-summary="handleRequestSummary" />
      </div>

      <!-- AI Summary Tab -->
      <div v-show="activeTab === 'ai'" class="tab-panel">
        <AiSummaryPanel
          :form="form"
          :summarizing="summarizing"
          :summary-output="currentSummary"
          :summary-provider="summaryProvider"
          @summarize="handleSummarize"
          @update:title="updateTitle"
          @update:content="updateContent"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, reactive } from 'vue';
import LiteratureSearch from './LiteratureSearch.vue';
import AiSummaryPanel from './AiSummaryPanel.vue';
import { useLiterature } from '../composables/useLiterature';
import type { SummaryFormat } from '../api/peptrack';

interface SummaryFormModel {
  title: string;
  content: string;
  format: SummaryFormat;
}

const activeTab = ref<'literature' | 'ai'>('literature');

// Use the literature composable for summary functionality
const { summarizing, currentSummary, summaryProvider, summarize } = useLiterature();

// Form state
const form = reactive<SummaryFormModel>({
  title: '',
  content: '',
  format: 'Markdown'
});

function handleRequestSummary(payload: { title: string; content: string }) {
  // Switch to AI tab and prefill the form
  form.title = payload.title;
  form.content = payload.content;
  activeTab.value = 'ai';
}

async function handleSummarize() {
  await summarize(form.title, form.content, form.format);
}

function updateTitle(value: string) {
  form.title = value;
}

function updateContent(value: string) {
  form.content = value;
}
</script>

<style scoped>
.research {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: #fafafa;
}

.research-tabs {
  display: flex;
  gap: 4px;
  padding: 16px 16px 0 16px;
  background: white;
  border-bottom: 2px solid #e0e0e0;
}

.tab-btn {
  padding: 12px 24px;
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

.tab-content {
  flex: 1;
  overflow: hidden;
}

.tab-panel {
  height: 100%;
  overflow-y: auto;
}

@media (prefers-color-scheme: dark) {
  .research {
    background: #1a1a1a;
  }

  .research-tabs {
    background: #2a2a2a;
    border-bottom-color: #3a3a3a;
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
}
</style>
