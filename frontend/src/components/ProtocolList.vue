<script setup lang="ts">
import type { PeptideProtocol } from "../api/peptrack";

interface Props {
  protocols: PeptideProtocol[];
  loading: boolean;
}

const props = defineProps<Props>();
const emit = defineEmits<{
  refresh: [];
}>();

function handleRefresh() {
  emit("refresh");
}
</script>

<template>
  <article class="panel">
    <div class="panel-header">
      <h2>💊 My Peptide Plans</h2>
      <button @click="handleRefresh" :disabled="props.loading">
        {{ props.loading ? "↻ Loading..." : "↻ Refresh" }}
      </button>
    </div>
    <p v-if="!props.protocols.length && !props.loading" class="muted">
      No peptide plans yet. Create your first one below!
    </p>
    <ul v-else class="protocol-list">
      <li v-for="protocol in props.protocols" :key="protocol.id">
        <div class="protocol-title">{{ protocol.name }}</div>
        <div class="protocol-meta">
          <span>{{ protocol.peptide_name }}</span>
          <span>
            Last updated:
            {{ new Date(protocol.updated_at).toLocaleDateString() }}
          </span>
        </div>
        <p class="protocol-notes" v-if="protocol.notes">
          {{ protocol.notes }}
        </p>
      </li>
    </ul>
  </article>
</template>
