<template>
  <div class="literature-search">
    <h2>📚 Research Papers</h2>
    <p class="subtitle">Find scientific studies about peptides</p>

    <!-- Search Form -->
    <div class="search-box">
      <input
        id="literature-search-input"
        v-model="searchQuery"
        type="text"
        placeholder="What do you want to research? (e.g., BPC-157 wound healing)"
        @keyup.enter="handleSearch"
        class="search-input"
        aria-label="Search scientific literature"
        autocomplete="off"
      />
      <button
        @click="handleSearch"
        :disabled="isSearching || !searchQuery.trim()"
        class="search-btn"
        aria-label="Search for research papers"
        :aria-busy="isSearching"
      >
        {{ isSearching ? 'Finding Papers...' : 'Find Papers' }}
      </button>
    </div>

    <!-- Error Display -->
    <div v-if="error" class="error-message">
      {{ error }}
    </div>

    <!-- Search Results -->
    <div v-if="searchResults.length > 0" class="search-results">
      <h3>Papers We Found</h3>
      <div v-for="sourceResult in searchResults" :key="sourceResult.source" class="source-section">
        <h4 class="source-header">From {{ getSourceName(sourceResult.source) }} ({{ sourceResult.results.length }} papers)</h4>
        <div v-for="(result, idx) in sourceResult.results" :key="idx" class="result-card">
          <h5>{{ result.title }}</h5>
          <p v-if="result.authors" class="authors">{{ result.authors }}</p>
          <p v-if="result.journal" class="journal">
            <em>{{ result.journal }}</em>
            <span v-if="result.published_date"> ({{ result.published_date }})</span>
          </p>
          <p v-if="result.abstract_text" class="abstract">{{ result.abstract_text }}</p>
          <div class="links">
            <a v-if="result.url" :href="result.url" target="_blank" rel="noopener noreferrer">
              View Article →
            </a>
            <span v-if="result.doi" class="doi">DOI: {{ result.doi }}</span>
          </div>
        </div>
      </div>
    </div>

    <!-- Saved Papers -->
    <div class="cached-section">
      <div class="cached-header">
        <h3>Your Saved Papers ({{ cachedLiterature.length }})</h3>
        <button
          @click="loadCachedLiterature"
          class="refresh-btn"
          aria-label="Refresh saved papers"
        >↻ Refresh</button>
      </div>

      <input
        id="cache-search-input"
        v-model="cacheSearchQuery"
        type="text"
        placeholder="Search your saved papers..."
        @input="handleCacheSearch"
        class="search-input"
        aria-label="Search saved papers"
        autocomplete="off"
      />

      <div v-if="filteredCachedLiterature.length === 0" class="no-results">
        No saved papers yet. Search for papers above and they'll be saved here automatically!
      </div>

      <div v-else class="literature-list">
        <div v-for="entry in filteredCachedLiterature" :key="entry.id" class="literature-card">
          <div class="literature-header">
            <span class="source-badge">{{ getSourceName(entry.source) }}</span>
            <span class="date">Saved {{ formatDate(entry.indexed_at) }}</span>
          </div>
          <h4>{{ entry.title }}</h4>
          <p v-if="entry.summary" class="summary">{{ entry.summary }}</p>
          <a v-if="entry.url" :href="entry.url" target="_blank" rel="noopener noreferrer" class="view-link">
            📄 Read Paper →
          </a>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted } from 'vue';
import { showErrorToast } from '../utils/errorHandling';
import {
  listLiterature,
  searchCachedLiterature,
  searchLiterature,
  type LiteratureEntry,
  type LiteratureSearchResult,
} from '../api/peptrack';

// Search state
const searchQuery = ref('');
const isSearching = ref(false);
const searchResults = ref<LiteratureSearchResult[]>([]);
const error = ref<string | null>(null);

// Always search all sources with sensible defaults
const maxResults = 10;

// Cache state
const cachedLiterature = ref<LiteratureEntry[]>([]);
const cacheSearchQuery = ref('');
const filteredCachedLiterature = ref<LiteratureEntry[]>([]);

onMounted(() => {
  loadCachedLiterature();
});

async function loadCachedLiterature() {
  try {
    cachedLiterature.value = await listLiterature();
    filteredCachedLiterature.value = cachedLiterature.value;
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load saved papers' });
  }
}

async function handleSearch() {
  if (!searchQuery.value.trim()) return;

  // Always search all sources - user doesn't need to choose
  const sources = ['pubmed', 'openalex'];

  isSearching.value = true;
  error.value = null;
  searchResults.value = [];

  try {
    const results = await searchLiterature({
      query: searchQuery.value,
      maxResults: maxResults,
      sources,
    });
    searchResults.value = results;

    // Refresh saved papers after search
    await loadCachedLiterature();
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'search literature' });
  } finally {
    isSearching.value = false;
  }
}

async function handleCacheSearch() {
  if (!cacheSearchQuery.value.trim()) {
    filteredCachedLiterature.value = cachedLiterature.value;
    return;
  }

  try {
    filteredCachedLiterature.value = await searchCachedLiterature(cacheSearchQuery.value);
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'search cached literature' });
  }
}

function formatDate(dateStr: string): string {
  try {
    const date = new Date(dateStr);
    return date.toLocaleDateString();
  } catch {
    return dateStr;
  }
}

function getSourceName(source: string): string {
  const names: Record<string, string> = {
    'pubmed': 'Medical Database',
    'openalex': 'Research Library',
    'crossref': 'Scientific Journal Index',
  };
  return names[source.toLowerCase()] || source;
}
</script>

<style scoped>
.literature-search {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
}

h2 {
  margin-bottom: 8px;
  color: #2c3e50;
}

.subtitle {
  color: #666;
  font-size: 14px;
  margin-bottom: 20px;
  margin-top: 0;
}

.search-box {
  display: flex;
  gap: 10px;
  margin-bottom: 15px;
}

.search-input {
  flex: 1;
  padding: 12px;
  border: 2px solid #ddd;
  border-radius: 6px;
  font-size: 16px;
}

.search-btn {
  padding: 12px 24px;
  background-color: #42b983;
  color: white;
  border: none;
  border-radius: 6px;
  cursor: pointer;
  font-size: 16px;
  font-weight: bold;
}

.search-btn:hover:not(:disabled) {
  background-color: #359268;
}

.search-btn:disabled {
  background-color: #ccc;
  cursor: not-allowed;
}

.search-options {
  display: flex;
  gap: 20px;
  margin-bottom: 20px;
  align-items: center;
}

.search-options label {
  display: flex;
  align-items: center;
  gap: 5px;
}

.max-results-input {
  width: 60px;
  padding: 4px;
  border: 1px solid #ddd;
  border-radius: 4px;
}

.error-message {
  padding: 12px;
  background-color: #fee;
  border: 1px solid #fcc;
  border-radius: 6px;
  color: #c33;
  margin-bottom: 20px;
}

.search-results {
  margin-bottom: 40px;
}

.source-section {
  margin-bottom: 30px;
}

.source-section h4 {
  color: #2c3e50;
  margin-bottom: 15px;
  text-transform: capitalize;
}

.result-card {
  background: #f9f9f9;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
  margin-bottom: 15px;
}

.result-card h5 {
  margin: 0 0 10px 0;
  color: #2c3e50;
}

.authors, .journal {
  font-size: 14px;
  color: #666;
  margin: 5px 0;
}

.abstract {
  font-size: 14px;
  color: #555;
  margin: 10px 0;
  line-height: 1.5;
}

.links {
  display: flex;
  gap: 15px;
  margin-top: 10px;
  font-size: 14px;
}

.links a {
  color: #42b983;
  text-decoration: none;
  font-weight: bold;
}

.links a:hover {
  text-decoration: underline;
}

.doi {
  color: #999;
  font-size: 13px;
}

.cached-section {
  margin-top: 40px;
  border-top: 2px solid #eee;
  padding-top: 20px;
}

.cached-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 15px;
}

.refresh-btn {
  padding: 8px 16px;
  background-color: #3498db;
  color: white;
  border: none;
  border-radius: 4px;
  cursor: pointer;
  font-size: 14px;
}

.refresh-btn:hover {
  background-color: #2980b9;
}

.no-results {
  padding: 40px;
  text-align: center;
  color: #999;
  font-style: italic;
}

.literature-list {
  display: grid;
  gap: 15px;
}

.literature-card {
  background: white;
  border: 1px solid #ddd;
  border-radius: 8px;
  padding: 15px;
  transition: box-shadow 0.2s;
}

.literature-card:hover {
  box-shadow: 0 2px 8px rgba(0, 0, 0, 0.1);
}

.literature-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.source-badge {
  display: inline-block;
  padding: 4px 8px;
  background-color: #42b983;
  color: white;
  border-radius: 4px;
  font-size: 12px;
  font-weight: bold;
  text-transform: uppercase;
}

.date {
  font-size: 13px;
  color: #999;
}

.literature-card h4 {
  margin: 0 0 10px 0;
  color: #2c3e50;
  font-size: 16px;
}

.summary {
  font-size: 14px;
  color: #666;
  line-height: 1.5;
  margin: 10px 0;
}

.view-link {
  display: inline-block;
  margin-top: 10px;
  color: #42b983;
  text-decoration: none;
  font-weight: bold;
  font-size: 14px;
}

.view-link:hover {
  text-decoration: underline;
}
</style>
