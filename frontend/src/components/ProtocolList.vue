<script setup lang="ts">
import { ref, withDefaults } from 'vue';
import type { PeptideProtocol } from "../api/peptrack";
import { populateDefaultPeptides } from "../api/peptrack";
import { showSuccessToast, showErrorToast } from "../utils/errorHandling";

interface Props {
  protocols?: PeptideProtocol[];
  loading?: boolean;
}

const props = withDefaults(defineProps<Props>(), {
  protocols: () => [],
  loading: false
});
const emit = defineEmits<{
  refresh: [];
}>();

const loadingDefaults = ref(false);

function handleRefresh() {
  emit("refresh");
}

async function handleLoadDefaults() {
  loadingDefaults.value = true;
  try {
    const count = await populateDefaultPeptides();
    if (count > 0) {
      showSuccessToast(`Added ${count} popular peptide${count !== 1 ? 's' : ''} to your protocols!`);
      emit("refresh");
    } else {
      showSuccessToast("All popular peptides are already in your protocols!");
    }
  } catch (error) {
    showErrorToast(error, { operation: 'load default peptides' });
  } finally {
    loadingDefaults.value = false;
  }
}
</script>

<template>
  <article class="panel">
    <div class="panel-header">
      <h2>💊 My Peptide Plans</h2>
      <div class="header-actions">
        <button
          @click="handleLoadDefaults"
          :disabled="props.loading || loadingDefaults"
          class="btn-popular"
          title="Load 27 popular peptides"
        >
          {{ loadingDefaults ? "⏳ Loading..." : "✨ Load Popular Peptides" }}
        </button>
        <button
          @click="handleRefresh"
          :disabled="props.loading"
          aria-label="Refresh protocol list"
          :aria-busy="props.loading"
        >
          {{ props.loading ? "↻ Loading..." : "↻ Refresh" }}
        </button>
      </div>
    </div>
    <p v-if="!props.protocols.length && !props.loading" class="muted">
      No peptide plans yet. Create your first one below or load popular peptides!
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

<style scoped>
.header-actions {
  display: flex;
  gap: 8px;
  flex-wrap: wrap;
}

.btn-popular {
  padding: 8px 16px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-popular:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.4);
}

.btn-popular:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

@media (max-width: 768px) {
  .header-actions {
    flex-direction: column;
    width: 100%;
  }

  .btn-popular,
  .header-actions button {
    width: 100%;
  }
}
</style>
