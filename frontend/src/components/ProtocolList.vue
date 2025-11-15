<script setup lang="ts">
import { ref, computed } from 'vue';
import type { PeptideProtocol } from "../api/peptrack";
import {
  populateDefaultPeptides,
  toggleProtocolFavorite,
  addProtocolTag,
  removeProtocolTag,
  bulkDeleteProtocols,
  bulkAddTagToProtocols,
  bulkToggleFavoriteProtocols
} from "../api/peptrack";
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
const tagInput = ref<Record<string, string>>({});
const processingTags = ref<Set<string>>(new Set());
const selectedTagFilter = ref<string>('');

// Bulk selection state
const selectedProtocolIds = ref<Set<string>>(new Set());
const bulkTagInput = ref<string>('');
const processingBulkOperation = ref(false);

// Get all unique tags across all protocols
const allTags = computed(() => {
  const tagsSet = new Set<string>();
  props.protocols.forEach(protocol => {
    protocol.tags?.forEach(tag => tagsSet.add(tag));
  });
  return Array.from(tagsSet).sort();
});

// Sort and filter protocols
const sortedProtocols = computed(() => {
  let filtered = [...props.protocols];

  // Filter by selected tag
  if (selectedTagFilter.value) {
    filtered = filtered.filter(p =>
      p.tags?.includes(selectedTagFilter.value)
    );
  }

  // Sort: favorites first, then by updated_at
  return filtered.sort((a, b) => {
    // Favorites first
    if (a.is_favorite && !b.is_favorite) return -1;
    if (!a.is_favorite && b.is_favorite) return 1;

    // Then by updated_at (most recent first)
    const dateA = new Date(a.updated_at).getTime();
    const dateB = new Date(b.updated_at).getTime();
    return dateB - dateA;
  });
});

// Bulk selection computed
const allSelected = computed(() => {
  if (sortedProtocols.value.length === 0) return false;
  return sortedProtocols.value.every(p => selectedProtocolIds.value.has(p.id));
});

const someSelected = computed(() => {
  return selectedProtocolIds.value.size > 0 && !allSelected.value;
});

const selectionCount = computed(() => selectedProtocolIds.value.size);

// Bulk selection functions
function toggleSelectAll() {
  if (allSelected.value) {
    selectedProtocolIds.value.clear();
  } else {
    sortedProtocols.value.forEach(p => selectedProtocolIds.value.add(p.id));
  }
}

function toggleSelectProtocol(protocolId: string) {
  if (selectedProtocolIds.value.has(protocolId)) {
    selectedProtocolIds.value.delete(protocolId);
  } else {
    selectedProtocolIds.value.add(protocolId);
  }
}

function clearSelection() {
  selectedProtocolIds.value.clear();
  bulkTagInput.value = '';
}

async function handleBulkDelete() {
  const count = selectedProtocolIds.value.size;
  if (count === 0) return;

  if (!confirm(`Are you sure you want to delete ${count} protocol${count !== 1 ? 's' : ''}? This cannot be undone.`)) {
    return;
  }

  processingBulkOperation.value = true;

  try {
    const ids = Array.from(selectedProtocolIds.value);
    const deleted = await bulkDeleteProtocols(ids);

    showSuccessToast('Success', `Deleted ${deleted} protocol${deleted !== 1 ? 's' : ''}`);
    clearSelection();
    emit('refresh');
  } catch (error) {
    showErrorToast(error, { operation: 'bulk delete protocols' });
  } finally {
    processingBulkOperation.value = false;
  }
}

async function handleBulkAddTag() {
  const tag = bulkTagInput.value.trim();
  const count = selectedProtocolIds.value.size;

  if (!tag || count === 0) return;

  processingBulkOperation.value = true;

  try {
    const ids = Array.from(selectedProtocolIds.value);
    const modified = await bulkAddTagToProtocols(ids, tag);

    showSuccessToast('Success', `Added tag "${tag}" to ${modified} protocol${modified !== 1 ? 's' : ''}`);
    bulkTagInput.value = '';
    emit('refresh');
  } catch (error) {
    showErrorToast(error, { operation: 'bulk add tag' });
  } finally {
    processingBulkOperation.value = false;
  }
}

async function handleBulkToggleFavorite(isFavorite: boolean) {
  const count = selectedProtocolIds.value.size;
  if (count === 0) return;

  processingBulkOperation.value = true;

  try {
    const ids = Array.from(selectedProtocolIds.value);
    const modified = await bulkToggleFavoriteProtocols(ids, isFavorite);

    const action = isFavorite ? 'favorited' : 'unfavorited';
    showSuccessToast('Success', `${modified} protocol${modified !== 1 ? 's' : ''} ${action}`);
    emit('refresh');
  } catch (error) {
    showErrorToast(error, { operation: 'bulk toggle favorites' });
  } finally {
    processingBulkOperation.value = false;
  }
}

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

async function handleAddTag(protocol: PeptideProtocol) {
  const tag = tagInput.value[protocol.id]?.trim();
  if (!tag) return;

  const tagKey = `${protocol.id}-${tag}`;
  if (processingTags.value.has(tagKey)) return;

  processingTags.value.add(tagKey);

  try {
    const updatedTags = await addProtocolTag(protocol.id, tag);
    protocol.tags = updatedTags;
    tagInput.value[protocol.id] = '';
    showSuccessToast('Tag Added', `Added "${tag}" to ${protocol.name}`);
    emit("refresh");
  } catch (error) {
    showErrorToast(error, { operation: 'add tag' });
  } finally {
    processingTags.value.delete(tagKey);
  }
}

async function handleRemoveTag(protocol: PeptideProtocol, tag: string) {
  const tagKey = `${protocol.id}-${tag}`;
  if (processingTags.value.has(tagKey)) return;

  processingTags.value.add(tagKey);

  try {
    const updatedTags = await removeProtocolTag(protocol.id, tag);
    protocol.tags = updatedTags;
    showSuccessToast('Tag Removed', `Removed "${tag}" from ${protocol.name}`);
    emit("refresh");
  } catch (error) {
    showErrorToast(error, { operation: 'remove tag' });
  } finally {
    processingTags.value.delete(tagKey);
  }
}

function clearTagFilter() {
  selectedTagFilter.value = '';
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

    <!-- Bulk Selection Header -->
    <div v-if="sortedProtocols.length > 0" class="bulk-selection-header">
      <label class="select-all-container">
        <input
          type="checkbox"
          :checked="allSelected"
          :indeterminate="someSelected"
          @change="toggleSelectAll"
          class="select-all-checkbox"
        />
        <span class="select-all-label">
          {{ selectionCount > 0 ? `${selectionCount} selected` : 'Select all' }}
        </span>
      </label>
    </div>

    <!-- Bulk Actions Toolbar -->
    <div v-if="selectionCount > 0" class="bulk-actions-toolbar">
      <div class="bulk-actions-info">
        <span class="selection-count">{{ selectionCount }} protocol{{ selectionCount !== 1 ? 's' : '' }} selected</span>
        <button @click="clearSelection" class="btn-clear-selection">✕ Clear</button>
      </div>
      <div class="bulk-actions-buttons">
        <button
          @click="handleBulkDelete"
          :disabled="processingBulkOperation"
          class="bulk-btn bulk-btn-delete"
          title="Delete selected protocols"
        >
          🗑️ Delete
        </button>
        <button
          @click="handleBulkToggleFavorite(true)"
          :disabled="processingBulkOperation"
          class="bulk-btn bulk-btn-favorite"
          title="Mark selected as favorites"
        >
          ⭐ Favorite
        </button>
        <button
          @click="handleBulkToggleFavorite(false)"
          :disabled="processingBulkOperation"
          class="bulk-btn bulk-btn-unfavorite"
          title="Remove from favorites"
        >
          ☆ Unfavorite
        </button>
        <div class="bulk-tag-input-group">
          <input
            v-model="bulkTagInput"
            type="text"
            placeholder="Add tag to selected..."
            class="bulk-tag-input"
            maxlength="20"
            @keydown.enter.prevent="handleBulkAddTag"
          />
          <button
            @click="handleBulkAddTag"
            :disabled="!bulkTagInput.trim() || processingBulkOperation"
            class="bulk-btn bulk-btn-tag"
            title="Add tag to selected"
          >
            🏷️ Tag
          </button>
        </div>
      </div>
    </div>

    <!-- Tag Filter -->
    <div v-if="allTags.length > 0" class="tag-filter">
      <div class="tag-filter-label">🏷️ Filter by tag:</div>
      <div class="tag-filter-chips">
        <button
          v-for="tag in allTags"
          :key="tag"
          @click="selectedTagFilter = selectedTagFilter === tag ? '' : tag"
          :class="['filter-chip', { active: selectedTagFilter === tag }]"
        >
          {{ tag }}
        </button>
        <button
          v-if="selectedTagFilter"
          @click="clearTagFilter"
          class="filter-chip clear-filter"
        >
          ✕ Clear
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
        :class="{ 'is-favorite': protocol.is_favorite, 'selected': selectedProtocolIds.has(protocol.id) }"
      >
        <input
          type="checkbox"
          :checked="selectedProtocolIds.has(protocol.id)"
          @change="toggleSelectProtocol(protocol.id)"
          class="protocol-checkbox"
        />
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

          <!-- Tags Display -->
          <div v-if="protocol.tags && protocol.tags.length > 0" class="protocol-tags">
            <button
              v-for="tag in protocol.tags"
              :key="tag"
              @click="handleRemoveTag(protocol, tag)"
              :disabled="processingTags.has(`${protocol.id}-${tag}`)"
              class="tag-badge"
              :title="`Click to remove tag: ${tag}`"
            >
              🏷️ {{ tag }} <span class="tag-remove">✕</span>
            </button>
          </div>

          <!-- Add Tag Input -->
          <form @submit.prevent="handleAddTag(protocol)" class="tag-input-form">
            <input
              v-model="tagInput[protocol.id]"
              type="text"
              placeholder="Add tag (e.g., morning, recovery)..."
              class="tag-input"
              maxlength="20"
              @keydown.enter.prevent="handleAddTag(protocol)"
            />
            <button
              type="submit"
              :disabled="!tagInput[protocol.id]?.trim()"
              class="tag-add-btn"
              title="Add tag"
            >
              + Add
            </button>
          </form>
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

/* Tag Filter */
.tag-filter {
  background: #f8f9fa;
  border-radius: 8px;
  padding: 12px;
  margin-bottom: 16px;
}

.tag-filter-label {
  font-size: 13px;
  font-weight: 600;
  color: #495057;
  margin-bottom: 8px;
}

.tag-filter-chips {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
}

.filter-chip {
  padding: 6px 12px;
  background: white;
  border: 2px solid #dee2e6;
  border-radius: 16px;
  font-size: 13px;
  font-weight: 500;
  cursor: pointer;
  transition: all 0.2s;
  color: #495057;
}

.filter-chip:hover {
  border-color: #667eea;
  background: #f0f2ff;
  transform: translateY(-1px);
}

.filter-chip.active {
  background: #667eea;
  border-color: #667eea;
  color: white;
  font-weight: 600;
}

.filter-chip.clear-filter {
  background: #e9ecef;
  border-color: #adb5bd;
  color: #495057;
}

.filter-chip.clear-filter:hover {
  background: #dc3545;
  border-color: #dc3545;
  color: white;
}

/* Protocol Tags */
.protocol-tags {
  display: flex;
  flex-wrap: wrap;
  gap: 6px;
  margin-top: 8px;
}

.tag-badge {
  display: inline-flex;
  align-items: center;
  gap: 4px;
  padding: 4px 10px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  color: white;
  border: none;
  border-radius: 12px;
  font-size: 12px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.tag-badge:hover:not(:disabled) {
  transform: scale(1.05);
  box-shadow: 0 2px 8px rgba(102, 126, 234, 0.4);
}

.tag-badge:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.tag-remove {
  margin-left: 2px;
  font-weight: 700;
  opacity: 0.8;
}

.tag-badge:hover .tag-remove {
  opacity: 1;
}

/* Tag Input Form */
.tag-input-form {
  display: flex;
  gap: 8px;
  margin-top: 10px;
  align-items: center;
}

.tag-input {
  flex: 1;
  padding: 6px 10px;
  border: 2px solid #e0e0e0;
  border-radius: 6px;
  font-size: 13px;
  transition: all 0.2s;
}

.tag-input:focus {
  outline: none;
  border-color: #667eea;
  box-shadow: 0 0 0 3px rgba(102, 126, 234, 0.1);
}

.tag-add-btn {
  padding: 6px 12px;
  background: #28a745;
  color: white;
  border: none;
  border-radius: 6px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  white-space: nowrap;
}

.tag-add-btn:hover:not(:disabled) {
  background: #218838;
  transform: translateY(-1px);
  box-shadow: 0 2px 6px rgba(40, 167, 69, 0.3);
}

.tag-add-btn:disabled {
  background: #6c757d;
  opacity: 0.5;
  cursor: not-allowed;
}

/* Bulk Selection */
.bulk-selection-header {
  background: #f0f2ff;
  border-radius: 8px;
  padding: 12px 16px;
  margin-bottom: 12px;
  border: 2px solid #667eea;
}

.select-all-container {
  display: flex;
  align-items: center;
  gap: 10px;
  cursor: pointer;
  user-select: none;
}

.select-all-checkbox {
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #667eea;
}

.select-all-label {
  font-size: 14px;
  font-weight: 600;
  color: #667eea;
}

.protocol-checkbox {
  flex-shrink: 0;
  width: 18px;
  height: 18px;
  cursor: pointer;
  accent-color: #667eea;
  margin-right: 8px;
}

.protocol-list li.selected {
  background: linear-gradient(to right, #f0f2ff 0%, #ffffff 100%);
  border-left: 3px solid #667eea;
}

/* Bulk Actions Toolbar */
.bulk-actions-toolbar {
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 10px;
  padding: 16px;
  margin-bottom: 16px;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.bulk-actions-info {
  display: flex;
  align-items: center;
  justify-content: space-between;
  margin-bottom: 12px;
  padding-bottom: 12px;
  border-bottom: 1px solid rgba(255, 255, 255, 0.2);
}

.selection-count {
  color: white;
  font-weight: 600;
  font-size: 14px;
}

.btn-clear-selection {
  background: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid rgba(255, 255, 255, 0.3);
  border-radius: 6px;
  padding: 6px 12px;
  font-size: 13px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.btn-clear-selection:hover {
  background: rgba(255, 255, 255, 0.3);
  transform: translateY(-1px);
}

.bulk-actions-buttons {
  display: flex;
  flex-wrap: wrap;
  gap: 10px;
  align-items: center;
}

.bulk-btn {
  padding: 8px 16px;
  border: 2px solid white;
  border-radius: 6px;
  font-size: 14px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
  background: white;
  color: #667eea;
}

.bulk-btn:hover:not(:disabled) {
  transform: translateY(-2px);
  box-shadow: 0 4px 12px rgba(0, 0, 0, 0.2);
}

.bulk-btn:disabled {
  opacity: 0.5;
  cursor: not-allowed;
}

.bulk-btn-delete {
  color: #dc3545;
  border-color: white;
}

.bulk-btn-delete:hover:not(:disabled) {
  background: #dc3545;
  color: white;
}

.bulk-btn-favorite {
  color: #ffc107;
  border-color: white;
}

.bulk-btn-favorite:hover:not(:disabled) {
  background: #ffc107;
  color: white;
}

.bulk-btn-unfavorite {
  color: #6c757d;
  border-color: white;
}

.bulk-btn-unfavorite:hover:not(:disabled) {
  background: #6c757d;
  color: white;
}

.bulk-btn-tag {
  color: #28a745;
  border-color: white;
}

.bulk-btn-tag:hover:not(:disabled) {
  background: #28a745;
  color: white;
}

.bulk-tag-input-group {
  display: flex;
  gap: 8px;
  flex: 1;
  max-width: 300px;
}

.bulk-tag-input {
  flex: 1;
  padding: 8px 12px;
  border: 2px solid white;
  border-radius: 6px;
  font-size: 14px;
  background: rgba(255, 255, 255, 0.95);
  color: #333;
}

.bulk-tag-input:focus {
  outline: none;
  background: white;
  box-shadow: 0 0 0 3px rgba(255, 255, 255, 0.3);
}

.bulk-tag-input::placeholder {
  color: #999;
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

  .tag-filter-chips {
    gap: 4px;
  }

  .filter-chip {
    font-size: 12px;
    padding: 5px 10px;
  }

  .tag-input-form {
    flex-direction: column;
    align-items: stretch;
  }

  .tag-add-btn {
    width: 100%;
  }

  /* Bulk operations mobile */
  .bulk-actions-buttons {
    flex-direction: column;
  }

  .bulk-btn {
    width: 100%;
  }

  .bulk-tag-input-group {
    max-width: 100%;
    width: 100%;
  }
}
</style>
