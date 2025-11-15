<script setup lang="ts">
import { ref, computed } from 'vue';
import type { PeptideProtocol } from "../api/peptrack";
import { populateDefaultPeptides, toggleProtocolFavorite } from "../api/peptrack";
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
const togglingFavorites = ref<Set<string>>(new Set());

// Sort protocols: favorites first, then by updated_at
const sortedProtocols = computed(() => {
  return [...props.protocols].sort((a, b) => {
    // Favorites first
    if (a.is_favorite && !b.is_favorite) return -1;
    if (!a.is_favorite && b.is_favorite) return 1;

    // Then by updated_at (most recent first)
    const dateA = new Date(a.updated_at).getTime();
    const dateB = new Date(b.updated_at).getTime();
    return dateB - dateA;
  });
});

function handleRefresh() {
  emit("refresh");
}

async function handleLoadDefaults() {
  loadingDefaults.value = true;
  try {
    const count = await populateDefaultPeptides();
    if (count > 0) {
      showSuccessToast('Peptides Added', `Added ${count} popular peptide${count !== 1 ? 's' : ''} to your protocols!`);
      emit("refresh");
    } else {
      showSuccessToast('Already Populated', 'All popular peptides are already in your protocols!');
    }
  } catch (error) {
    showErrorToast(error, { operation: 'load default peptides' });
  } finally {
    loadingDefaults.value = false;
  }
}

async function handleToggleFavorite(protocol: PeptideProtocol) {
  if (togglingFavorites.value.has(protocol.id)) return;

  togglingFavorites.value.add(protocol.id);

  try {
    const newFavoriteStatus = await toggleProtocolFavorite(protocol.id);

    // Update local state optimistically
    protocol.is_favorite = newFavoriteStatus;

    const message = newFavoriteStatus
      ? `⭐ ${protocol.name} added to favorites`
      : `Removed ${protocol.name} from favorites`;

    showSuccessToast('Success', message);

    // Refresh to ensure proper sorting
    emit("refresh");
  } catch (error) {
    showErrorToast(error, { operation: 'toggle favorite' });
  } finally {
    togglingFavorites.value.delete(protocol.id);
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
      <li
        v-for="protocol in sortedProtocols"
        :key="protocol.id"
        :class="{ 'is-favorite': protocol.is_favorite }"
      >
        <button
          @click="handleToggleFavorite(protocol)"
          :disabled="togglingFavorites.has(protocol.id)"
          class="favorite-btn"
          :class="{ 'is-favorite': protocol.is_favorite }"
          :title="protocol.is_favorite ? 'Remove from favorites' : 'Add to favorites'"
          :aria-label="protocol.is_favorite ? 'Remove from favorites' : 'Add to favorites'"
        >
          {{ protocol.is_favorite ? '⭐' : '☆' }}
        </button>

        <div class="protocol-content">
          <div class="protocol-title">{{ protocol.name }}</div>
          <div class="protocol-meta">
            <span>{{ protocol.peptide_name }}</span>
            <span>
              Last updated:
              {{ protocol.updated_at ? new Date(protocol.updated_at).toLocaleDateString() : 'N/A' }}
            </span>
          </div>
          <p class="protocol-notes" v-if="protocol.notes">
            {{ protocol.notes }}
          </p>
        </div>
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

/* Protocol list item styling */
.protocol-list li {
  display: flex;
  gap: 12px;
  align-items: flex-start;
  transition: all 0.2s ease;
}

.protocol-list li.is-favorite {
  background: linear-gradient(to right, #fff9e6 0%, #ffffff 100%);
  border-left: 3px solid #ffd700;
  padding-left: 12px;
}

/* Favorite button */
.favorite-btn {
  flex-shrink: 0;
  width: 32px;
  height: 32px;
  padding: 0;
  background: transparent;
  border: 1px solid #ddd;
  border-radius: 50%;
  cursor: pointer;
  font-size: 18px;
  line-height: 1;
  transition: all 0.2s ease;
  display: flex;
  align-items: center;
  justify-content: center;
}

.favorite-btn:hover:not(:disabled) {
  border-color: #ffd700;
  background: #fffbf0;
  transform: scale(1.1);
}

.favorite-btn.is-favorite {
  border-color: #ffd700;
  background: #fff9e6;
}

.favorite-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.favorite-btn:active:not(:disabled) {
  transform: scale(0.95);
}

/* Protocol content area */
.protocol-content {
  flex: 1;
  min-width: 0; /* Allow text truncation */
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

  .protocol-list li {
    gap: 8px;
  }

  .favorite-btn {
    width: 28px;
    height: 28px;
    font-size: 16px;
  }
}
</style>
