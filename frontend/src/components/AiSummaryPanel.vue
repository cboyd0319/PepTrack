<script setup lang="ts">
import type { SummaryFormat } from "../api/peptrack";

interface SummaryFormModel {
  title: string;
  content: string;
  format: SummaryFormat;
}

const props = defineProps<{
  form: SummaryFormModel;
  summarizing: boolean;
  summaryOutput: string | null;
  summaryProvider: string | null;
}>();

const emit = defineEmits<{
  summarize: [];
  'update:title': [value: string];
  'update:content': [value: string];
}>();

function handleSubmit() {
  emit("summarize");
}
</script>

<template>
  <section class="panel">
    <div class="panel-header">
      <div>
        <h2>🤖 AI Summary Helper</h2>
        <p class="muted">
          Paste text from research papers and get a simple summary
        </p>
      </div>
    </div>
    <form class="form-stack" @submit.prevent="handleSubmit">
      <label>
        Paper Name (optional)
        <input :value="props.form.title" @input="emit('update:title', ($event.target as HTMLInputElement).value)" type="text" placeholder="e.g., BPC-157 Healing Study" />
      </label>
      <label>
        Paste Text Here
        <textarea
          :value="props.form.content"
          @input="emit('update:content', ($event.target as HTMLTextAreaElement).value)"
          rows="6"
          placeholder="Copy and paste text from the paper you want summarized..."
        />
      </label>
      <button class="primary" type="submit" :disabled="props.summarizing">
        {{ props.summarizing ? "⏳ Creating Summary..." : "✨ Summarize This" }}
      </button>
    </form>

    <div v-if="props.summaryOutput" class="summary-output">
      <h3>📝 Your Summary</h3>
      <pre>{{ props.summaryOutput }}</pre>
    </div>
  </section>
</template>
