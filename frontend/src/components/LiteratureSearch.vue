<template>
  <div class="literature-search">
    <div class="search-header">
      <h2>📚 Research Papers</h2>
      <p class="subtitle">Find scientific studies about peptides</p>
    </div>

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

    <!-- Risk Matrix Button -->
    <div v-if="selectedPapers.length > 0" class="risk-matrix-banner">
      <div class="banner-content">
        <span>{{ selectedPapers.length }} paper(s) selected</span>
        <button @click="analyzeRiskMatrix" :disabled="analyzingRisks" class="analyze-btn">
          {{ analyzingRisks ? '🔍 Analyzing...' : '🎯 Analyze Risk Matrix' }}
        </button>
        <button @click="clearSelection" class="clear-btn">Clear</button>
      </div>
    </div>

    <!-- Search Results -->
    <div v-if="searchResults.length > 0" class="search-results">
      <h3>Papers We Found</h3>
      <div v-for="sourceResult in searchResults" :key="sourceResult.source" class="source-section">
        <h4 class="source-header">From {{ getSourceName(sourceResult.source) }} ({{ sourceResult.results.length }} papers)</h4>
        <div v-for="(result, idx) in sourceResult.results" :key="idx" class="result-card">
          <div class="paper-checkbox">
            <input
              type="checkbox"
              :id="`paper-${sourceResult.source}-${idx}`"
              :disabled="!isPaperSelected(result) && selectedPapers.length >= 5"
              @change="togglePaperSelection(result)"
              :checked="isPaperSelected(result)"
            />
            <label :for="`paper-${sourceResult.source}-${idx}`">Select for Risk Analysis</label>
          </div>
          <h5>{{ result.title }}</h5>
          <p v-if="result.authors" class="authors">{{ result.authors }}</p>
          <p v-if="result.journal" class="journal">
            <em>{{ result.journal }}</em>
            <span v-if="result.published_date"> ({{ result.published_date }})</span>
          </p>
          <p v-if="result.abstract_text" class="abstract">{{ result.abstract_text }}</p>
          <div class="links">
            <button type="button" class="link-btn" @click="openLink(result.url)">
              View Article →
            </button>
            <span v-if="result.doi" class="doi">DOI: {{ result.doi }}</span>
          </div>
          <div class="result-actions">
            <button type="button" class="ghost-btn" @click="summarizeResult(result)">
              🤖 Send to AI Summary
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Saved Papers -->
    <div class="cached-section">
      <div class="auto-save-note">
        🔖 Papers are saved to your library automatically every time you run a search.
      </div>
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

      <!-- Filters and Export -->
      <div v-if="cachedLiterature.length > 0" class="filters-export-bar">
        <div class="filters">
          <select v-model="sourceFilter" class="filter-select">
            <option value="all">All Sources</option>
            <option value="pubmed">Medical Database</option>
            <option value="openalex">Research Library</option>
          </select>

          <select v-model="sortBy" class="filter-select">
            <option value="date-desc">Newest First</option>
            <option value="date-asc">Oldest First</option>
            <option value="title-asc">Title (A-Z)</option>
          </select>
        </div>

        <div class="export-buttons">
          <button @click="exportBibTeX" class="export-btn" title="Export as BibTeX">
            📄 BibTeX
          </button>
          <button @click="exportCSV" class="export-btn" title="Export as CSV">
            📊 CSV
          </button>
        </div>
      </div>

      <div v-if="filteredCachedLiterature.length === 0" class="no-results">
        No saved papers yet. Search for papers above and they'll be saved here automatically!
      </div>

      <div v-else class="literature-list">
        <div v-for="entry in filteredCachedLiterature" :key="entry.id" class="literature-card">
          <div class="literature-header">
            <div class="header-left">
              <span class="source-badge">{{ getSourceName(entry.source) }}</span>
              <span class="date">Saved {{ formatDate(entry.indexed_at) }}</span>
            </div>
            <div class="paper-checkbox-inline">
              <input
                type="checkbox"
                :id="`saved-paper-${entry.id}`"
                :disabled="!isSavedPaperSelected(entry) && selectedPapers.length >= 5"
                @change="toggleSavedPaperSelection(entry)"
                :checked="isSavedPaperSelected(entry)"
              />
              <label :for="`saved-paper-${entry.id}`">Select</label>
            </div>
          </div>
          <h4>{{ entry.title }}</h4>
          <p v-if="entry.summary" class="summary">{{ entry.summary }}</p>
          <button
            type="button"
            class="view-link"
            @click="openLink(entry.url)"
          >
            📄 Read Paper →
          </button>
          <div class="result-actions">
            <button type="button" class="ghost-btn" @click="summarizeSavedEntry(entry)">
              🤖 Summarize Again
            </button>
          </div>
        </div>
      </div>
    </div>

    <!-- Risk Matrix Modal -->
    <div v-if="showRiskMatrix" class="modal-overlay" @click="closeRiskMatrix">
      <div class="modal-content risk-matrix-modal" @click.stop>
        <div class="modal-header">
          <h3>🎯 Literature Risk Matrix Analysis</h3>
          <button @click="closeRiskMatrix" class="close-btn">✕</button>
        </div>

        <div class="modal-body">
          <div class="papers-analyzed">
            <h4>📚 Papers Analyzed ({{ riskAnalysis.length }})</h4>
            <ul>
              <li v-for="(paper, idx) in riskAnalysis" :key="idx">{{ paper.title }}</li>
            </ul>
          </div>

          <div class="risk-categories">
            <div class="risk-category critical">
              <h4>🔴 Critical Risks</h4>
              <ul>
                <li v-for="(risk, idx) in criticalRisks" :key="idx">{{ risk }}</li>
                <li v-if="criticalRisks.length === 0" class="no-risks">None identified</li>
              </ul>
            </div>

            <div class="risk-category warning">
              <h4>🟡 Moderate Concerns</h4>
              <ul>
                <li v-for="(concern, idx) in moderateConcerns" :key="idx">{{ concern }}</li>
                <li v-if="moderateConcerns.length === 0" class="no-risks">None identified</li>
              </ul>
            </div>

            <div class="risk-category info">
              <h4>🔵 Study Limitations</h4>
              <ul>
                <li v-for="(limitation, idx) in studyLimitations" :key="idx">{{ limitation }}</li>
                <li v-if="studyLimitations.length === 0" class="no-risks">None identified</li>
              </ul>
            </div>
          </div>

          <div class="summary-section">
            <h4>📊 Overall Risk Assessment</h4>
            <p>{{ overallAssessment }}</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from 'vue';
import { showErrorToast, showSuccessToast } from '../utils/errorHandling';
import {
  listLiterature,
  searchCachedLiterature,
  searchLiterature,
  openExternalLink,
  summarizeContent,
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

// Filter and sort state
const sourceFilter = ref('all');
const sortBy = ref('date-desc');

// Risk Matrix state
interface SelectedPaper {
  title: string;
  content: string;
  source: string;
}

const selectedPapers = ref<SelectedPaper[]>([]);
const showRiskMatrix = ref(false);
const analyzingRisks = ref(false);
const riskAnalysis = ref<SelectedPaper[]>([]);
const criticalRisks = ref<string[]>([]);
const moderateConcerns = ref<string[]>([]);
const studyLimitations = ref<string[]>([]);
const overallAssessment = ref('');

function emitSummaryPrefill(title: string, content: string) {
  if (!content || !content.trim()) {
    showErrorToast('No abstract available to summarize.', {
      operation: 'prepare AI summary',
      details: 'The selected paper does not include text content.',
    });
    return;
  }

  window.dispatchEvent(
    new CustomEvent("peptrack:prefill-summary", {
      detail: {
        title,
        content,
      },
    }),
  );

  showSuccessToast('AI Summary Ready', 'We pre-filled the AI Summary tab with this paper.');
}

function summarizeResult(result: LiteratureSearchResult['results'][number]) {
  emitSummaryPrefill(result.title, result.abstract_text || '');
}

function summarizeSavedEntry(entry: LiteratureEntry) {
  emitSummaryPrefill(entry.title, entry.summary || '');
}

onMounted(() => {
  loadCachedLiterature();
});

// Watch for filter changes
watch([sourceFilter, sortBy], () => {
  handleCacheSearch();
});

async function loadCachedLiterature() {
  try {
    cachedLiterature.value = await listLiterature();
    filteredCachedLiterature.value = cachedLiterature.value;
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'load saved papers' });
  }
}

function normalizeUrl(url?: string | null): string | null {
  if (!url) return null;
  if (/^https?:\/\//i.test(url)) return url;
  if (url.startsWith("doi:")) {
    return `https://doi.org/${url.replace(/^doi:/i, "")}`;
  }
  if (url.startsWith("10.")) {
    return `https://doi.org/${url}`;
  }
  if (url.startsWith("www.")) {
    return `https://${url}`;
  }
  return url;
}

async function openLink(rawUrl?: string | null) {
  const url = normalizeUrl(rawUrl);
  if (!url) {
    showErrorToast('Link not available.', { operation: 'open article' });
    return;
  }

  try {
    await openExternalLink(url);
  } catch (_error) {
    // Fallback to browser default if Tauri command fails
    window.open(url, '_blank', 'noopener,noreferrer');
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
    // Add "+peptide" to the query if not already present to filter results better
    let enhancedQuery = searchQuery.value;
    if (!enhancedQuery.toLowerCase().includes('peptide')) {
      enhancedQuery += ' +peptide';
    }

    const results = await searchLiterature({
      query: enhancedQuery,
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
  let results: LiteratureEntry[];

  if (!cacheSearchQuery.value.trim()) {
    results = cachedLiterature.value;
  } else {
    try {
      results = await searchCachedLiterature(cacheSearchQuery.value);
    } catch (error: unknown) {
      showErrorToast(error, { operation: 'search cached literature' });
      return;
    }
  }

  // Apply source filter
  if (sourceFilter.value !== 'all') {
    results = results.filter(entry => entry.source === sourceFilter.value);
  }

  // Apply sorting
  results = applySorting(results);

  filteredCachedLiterature.value = results;
}

function applySorting(papers: LiteratureEntry[]): LiteratureEntry[] {
  const sorted = [...papers];

  switch (sortBy.value) {
    case 'date-desc':
      sorted.sort((a, b) => new Date(b.indexed_at).getTime() - new Date(a.indexed_at).getTime());
      break;
    case 'date-asc':
      sorted.sort((a, b) => new Date(a.indexed_at).getTime() - new Date(b.indexed_at).getTime());
      break;
    case 'title-asc':
      sorted.sort((a, b) => a.title.localeCompare(b.title));
      break;
  }

  return sorted;
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

// Risk Matrix Functions
function togglePaperSelection(result: LiteratureSearchResult['results'][number]) {
  const paper: SelectedPaper = {
    title: result.title,
    content: result.abstract_text || '',
    source: 'search',
  };

  const index = selectedPapers.value.findIndex(p => p.title === paper.title);
  if (index >= 0) {
    selectedPapers.value.splice(index, 1);
  } else if (selectedPapers.value.length < 5) {
    selectedPapers.value.push(paper);
  }
}

function toggleSavedPaperSelection(entry: LiteratureEntry) {
  const paper: SelectedPaper = {
    title: entry.title,
    content: entry.summary || '',
    source: 'saved',
  };

  const index = selectedPapers.value.findIndex(p => p.title === paper.title);
  if (index >= 0) {
    selectedPapers.value.splice(index, 1);
  } else if (selectedPapers.value.length < 5) {
    selectedPapers.value.push(paper);
  }
}

function isPaperSelected(result: LiteratureSearchResult['results'][number]): boolean {
  return selectedPapers.value.some(p => p.title === result.title);
}

function isSavedPaperSelected(entry: LiteratureEntry): boolean {
  return selectedPapers.value.some(p => p.title === entry.title);
}

function clearSelection() {
  selectedPapers.value = [];
}

async function analyzeRiskMatrix() {
  if (selectedPapers.value.length === 0) return;

  analyzingRisks.value = true;
  riskAnalysis.value = [];
  criticalRisks.value = [];
  moderateConcerns.value = [];
  studyLimitations.value = [];
  overallAssessment.value = '';

  try {
    // Combine all paper content for analysis - use available content or title as fallback
    const combinedContent = selectedPapers.value
      .map((p, idx) => {
        const content = p.content && p.content.trim().length > 0
          ? p.content
          : '(No abstract available - analysis based on title only)';
        return `\n\nPaper ${idx + 1}: ${p.title}\nContent: ${content}`;
      })
      .join('\n---');

    const prompt = `YOU MUST START YOUR RESPONSE WITH EXACTLY: "CRITICAL RISKS:"

DO NOT WRITE:
- "I'm thinking about..."
- "I wonder if..."
- "**Evaluating...**"
- Any preamble or explanation
- Any chain-of-thought reasoning

JUST WRITE THE FORMAT BELOW, NOTHING ELSE:

Papers to analyze:
${combinedContent}

NOW WRITE YOUR RESPONSE IN THIS EXACT FORMAT:

CRITICAL RISKS:
- [risk 1]
- [risk 2]

MODERATE CONCERNS:
- [concern 1]
- [concern 2]

STUDY LIMITATIONS:
- [limitation 1]
- [limitation 2]

OVERALL ASSESSMENT:
[2-3 sentences about safety and research quality]

REMEMBER: Your first word must be "CRITICAL" - nothing before it.`;

    const response = await summarizeContent({
      title: 'Literature Risk Matrix Analysis',
      content: prompt,
      format: 'Markdown',
    });

    // Parse the AI response
    parseRiskAnalysis(response.output);

    // Store analyzed papers
    riskAnalysis.value = [...selectedPapers.value];

    // Show the modal
    showRiskMatrix.value = true;
    showSuccessToast('Analysis Complete', 'Risk matrix has been generated');
  } catch (error: unknown) {
    showErrorToast(error, { operation: 'analyze risk matrix' });
  } finally {
    analyzingRisks.value = false;
  }
}

function parseRiskAnalysis(analysis: string) {
  // Strip out chain-of-thought thinking if present
  // The AI might include preamble like "**Evaluating...**" or "I'm thinking about..."
  // Find where the actual structured content starts
  let cleanedAnalysis = analysis;
  const structuredStart = analysis.indexOf('CRITICAL RISKS:');
  if (structuredStart !== -1) {
    cleanedAnalysis = analysis.substring(structuredStart);
  }

  const sections = {
    critical: /CRITICAL RISKS?:(.+?)(?=MODERATE|STUDY|OVERALL|$)/is,
    moderate: /MODERATE CONCERNS?:(.+?)(?=CRITICAL|STUDY|OVERALL|$)/is,
    limitations: /STUDY LIMITATIONS?:(.+?)(?=CRITICAL|MODERATE|OVERALL|$)/is,
    assessment: /OVERALL ASSESSMENT:(.+?)$/is,
  };

  // Extract critical risks
  const criticalMatch = cleanedAnalysis.match(sections.critical);
  if (criticalMatch && criticalMatch[1]) {
    criticalRisks.value = criticalMatch[1]
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.startsWith('-') || line.startsWith('•'))
      .map(line => line.replace(/^[-•]\s*/, '').trim())
      .filter(line => line.length > 0);
  }

  // Extract moderate concerns
  const moderateMatch = cleanedAnalysis.match(sections.moderate);
  if (moderateMatch && moderateMatch[1]) {
    moderateConcerns.value = moderateMatch[1]
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.startsWith('-') || line.startsWith('•'))
      .map(line => line.replace(/^[-•]\s*/, '').trim())
      .filter(line => line.length > 0);
  }

  // Extract study limitations
  const limitationsMatch = cleanedAnalysis.match(sections.limitations);
  if (limitationsMatch && limitationsMatch[1]) {
    studyLimitations.value = limitationsMatch[1]
      .split('\n')
      .map(line => line.trim())
      .filter(line => line.startsWith('-') || line.startsWith('•'))
      .map(line => line.replace(/^[-•]\s*/, '').trim())
      .filter(line => line.length > 0);
  }

  // Extract overall assessment
  const assessmentMatch = cleanedAnalysis.match(sections.assessment);
  if (assessmentMatch && assessmentMatch[1]) {
    overallAssessment.value = assessmentMatch[1].trim();
  }

  // Fallback if parsing failed
  if (criticalRisks.value.length === 0 && moderateConcerns.value.length === 0 && studyLimitations.value.length === 0) {
    overallAssessment.value = cleanedAnalysis;
  }
}

function closeRiskMatrix() {
  showRiskMatrix.value = false;
}

// Export Functions
function exportBibTeX() {
  const entries = filteredCachedLiterature.value.map(entry => {
    const key = entry.title.toLowerCase().replace(/[^a-z0-9]/g, '').substring(0, 20);
    const year = new Date(entry.indexed_at).getFullYear();

    return `@article{${key}${year},
  title = {${entry.title}},
  author = {Unknown},
  journal = {${entry.source}},
  year = {${year}},
  url = {${entry.url || 'N/A'}},
  abstract = {${(entry.summary || '').replace(/\n/g, ' ')}}
}`;
  }).join('\n\n');

  downloadFile(entries, 'literature.bib', 'application/x-bibtex');
  showSuccessToast('Export Complete', `${filteredCachedLiterature.value.length} papers exported as BibTeX`);
}

function exportCSV() {
  const headers = ['Title', 'Source', 'URL', 'Saved Date', 'Summary'];
  const rows = filteredCachedLiterature.value.map(entry => [
    `"${(entry.title || '').replace(/"/g, '""')}"`,
    `"${getSourceName(entry.source)}"`,
    `"${entry.url || ''}"`,
    `"${formatDate(entry.indexed_at)}"`,
    `"${(entry.summary || '').replace(/"/g, '""').replace(/\n/g, ' ')}"`,
  ]);

  const csv = [headers.join(','), ...rows.map(row => row.join(','))].join('\n');
  downloadFile(csv, 'literature.csv', 'text/csv');
  showSuccessToast('Export Complete', `${filteredCachedLiterature.value.length} papers exported as CSV`);
}

function downloadFile(content: string, filename: string, mimeType: string) {
  const blob = new Blob([content], { type: mimeType });
  const url = URL.createObjectURL(blob);
  const link = document.createElement('a');
  link.href = url;
  link.download = filename;
  document.body.appendChild(link);
  link.click();
  document.body.removeChild(link);
  URL.revokeObjectURL(url);
}
</script>

<style scoped>
.literature-search {
  padding: 20px;
  max-width: 1200px;
  margin: 0 auto;
  min-height: 500px;
}

.search-header {
  margin-bottom: 24px;
}

.search-header h2 {
  font-size: 28px;
  font-weight: 700;
  margin: 0 0 8px 0;
  color: #1a1a1a;
}

.search-header .subtitle {
  margin: 0;
  color: #666;
  font-size: 15px;
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

.result-actions {
  margin-top: 10px;
}

.ghost-btn {
  padding: 8px 14px;
  border-radius: 6px;
  border: 1px solid #d1d5db;
  background: #fff;
  color: #374151;
  font-weight: 500;
  cursor: pointer;
  transition: background 0.2s ease, box-shadow 0.2s ease;
}

.ghost-btn:hover {
  background: #f3f4f6;
  box-shadow: 0 1px 3px rgba(0, 0, 0, 0.1);
}

.links a {
  color: #42b983;
  text-decoration: none;
  font-weight: bold;
}

.links a:hover,
.link-btn:hover {
  text-decoration: underline;
}

.link-btn {
  background: none;
  border: none;
  color: #42b983;
  font-weight: bold;
  cursor: pointer;
  padding: 0;
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

.auto-save-note {
  padding: 10px 12px;
  background: #f0fdf4;
  border: 1px solid #86efac;
  border-radius: 6px;
  margin-bottom: 12px;
  font-size: 13px;
  color: #14532d;
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
  display: inline-flex;
  margin-top: 10px;
  color: #42b983;
  text-decoration: none;
  font-weight: bold;
  font-size: 14px;
  background: none;
  border: none;
  cursor: pointer;
  padding: 0;
  align-items: center;
  gap: 6px;
}

.view-link:hover {
  text-decoration: underline;
}

/* Risk Matrix Styles */
.risk-matrix-banner {
  position: sticky;
  top: 0;
  z-index: 100;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  padding: 16px;
  border-radius: 8px;
  margin-bottom: 20px;
  box-shadow: 0 4px 12px rgba(102, 126, 234, 0.3);
}

.banner-content {
  display: flex;
  align-items: center;
  gap: 16px;
  color: white;
  font-weight: 600;
}

.analyze-btn {
  padding: 10px 20px;
  background-color: #ffd700;
  color: #333;
  border: none;
  border-radius: 6px;
  font-weight: 700;
  cursor: pointer;
  transition: all 0.2s;
  font-size: 14px;
}

.analyze-btn:hover:not(:disabled) {
  background-color: #ffed4e;
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(255, 215, 0, 0.4);
}

.analyze-btn:disabled {
  opacity: 0.6;
  cursor: not-allowed;
}

.clear-btn {
  padding: 10px 16px;
  background-color: rgba(255, 255, 255, 0.2);
  color: white;
  border: 1px solid white;
  border-radius: 6px;
  font-weight: 600;
  cursor: pointer;
  transition: all 0.2s;
}

.clear-btn:hover {
  background-color: rgba(255, 255, 255, 0.3);
}

.paper-checkbox {
  margin-bottom: 12px;
  padding: 8px;
  background-color: #f8f9fa;
  border-radius: 6px;
  display: flex;
  align-items: center;
  gap: 8px;
}

.paper-checkbox input[type="checkbox"] {
  width: 18px;
  height: 18px;
  cursor: pointer;
}

.paper-checkbox input[type="checkbox"]:disabled {
  cursor: not-allowed;
  opacity: 0.5;
}

.paper-checkbox label {
  font-size: 13px;
  color: #666;
  cursor: pointer;
  user-select: none;
}

.paper-checkbox-inline {
  display: flex;
  align-items: center;
  gap: 6px;
}

.paper-checkbox-inline input[type="checkbox"] {
  width: 16px;
  height: 16px;
  cursor: pointer;
}

.paper-checkbox-inline label {
  font-size: 12px;
  color: #666;
  cursor: pointer;
  user-select: none;
}

.header-left {
  display: flex;
  gap: 12px;
  align-items: center;
}

.literature-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-bottom: 10px;
}

.modal-overlay {
  position: fixed;
  top: 0;
  left: 0;
  right: 0;
  bottom: 0;
  background-color: rgba(0, 0, 0, 0.6);
  display: flex;
  align-items: center;
  justify-content: center;
  z-index: 1000;
  padding: 20px;
}

.risk-matrix-modal {
  background: white;
  border-radius: 12px;
  max-width: 900px;
  width: 100%;
  max-height: 90vh;
  overflow-y: auto;
  box-shadow: 0 8px 32px rgba(0, 0, 0, 0.2);
}

.modal-header {
  display: flex;
  justify-content: space-between;
  align-items: center;
  padding: 20px;
  border-bottom: 2px solid #e0e0e0;
  position: sticky;
  top: 0;
  background: white;
  z-index: 10;
}

.modal-header h3 {
  margin: 0;
  color: #2c3e50;
}

.close-btn {
  background: none;
  border: none;
  font-size: 24px;
  cursor: pointer;
  color: #999;
  padding: 0;
  width: 32px;
  height: 32px;
  display: flex;
  align-items: center;
  justify-content: center;
  border-radius: 50%;
  transition: all 0.2s;
}

.close-btn:hover {
  background-color: #f0f0f0;
  color: #333;
}

.modal-body {
  padding: 20px;
}

.papers-analyzed {
  margin-bottom: 24px;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
}

.papers-analyzed h4 {
  margin-top: 0;
  margin-bottom: 12px;
  color: #2c3e50;
}

.papers-analyzed ul {
  margin: 0;
  padding-left: 20px;
}

.papers-analyzed li {
  margin-bottom: 8px;
  color: #555;
  line-height: 1.5;
}

.risk-categories {
  display: flex;
  flex-direction: column;
  gap: 20px;
  margin-bottom: 24px;
}

.risk-category {
  padding: 16px;
  border-radius: 8px;
  border-left: 4px solid;
}

.risk-category.critical {
  background-color: #fee;
  border-left-color: #e74c3c;
}

.risk-category.warning {
  background-color: #fffbf0;
  border-left-color: #f39c12;
}

.risk-category.info {
  background-color: #e3f2fd;
  border-left-color: #3498db;
}

.risk-category h4 {
  margin-top: 0;
  margin-bottom: 12px;
}

.risk-category.critical h4 {
  color: #c0392b;
}

.risk-category.warning h4 {
  color: #d68910;
}

.risk-category.info h4 {
  color: #2980b9;
}

.risk-category ul {
  margin: 0;
  padding-left: 20px;
}

.risk-category li {
  margin-bottom: 8px;
  color: #333;
  line-height: 1.6;
}

.no-risks {
  color: #999 !important;
  font-style: italic;
  list-style: none;
  margin-left: -20px;
}

.summary-section {
  padding: 20px;
  background: linear-gradient(135deg, #667eea 0%, #764ba2 100%);
  border-radius: 8px;
  color: white;
}

.summary-section h4 {
  margin-top: 0;
  margin-bottom: 12px;
  color: white;
}

.summary-section p {
  margin: 0;
  line-height: 1.7;
  font-size: 15px;
}

@media (max-width: 768px) {
  .banner-content {
    flex-direction: column;
    align-items: stretch;
  }

  .analyze-btn,
  .clear-btn {
    width: 100%;
  }

  .risk-matrix-modal {
    max-height: 95vh;
  }
}

/* Filter and Export Styles */
.filters-export-bar {
  display: flex;
  justify-content: space-between;
  align-items: center;
  gap: 16px;
  padding: 16px;
  background-color: #f8f9fa;
  border-radius: 8px;
  margin: 16px 0;
}

.filters {
  display: flex;
  gap: 12px;
  flex: 1;
}

.filter-select {
  padding: 8px 12px;
  border: 1px solid #ddd;
  border-radius: 6px;
  font-size: 14px;
  background-color: white;
  cursor: pointer;
  transition: border-color 0.2s;
}

.filter-select:hover {
  border-color: #42b983;
}

.filter-select:focus {
  outline: none;
  border-color: #42b983;
  box-shadow: 0 0 0 3px rgba(66, 185, 131, 0.1);
}

.export-buttons {
  display: flex;
  gap: 8px;
}

.export-btn {
  padding: 8px 16px;
  background: linear-gradient(135deg, #42b983, #36a371);
  color: white;
  border: none;
  border-radius: 6px;
  font-weight: 600;
  font-size: 13px;
  cursor: pointer;
  transition: all 0.2s;
  display: flex;
  align-items: center;
  gap: 6px;
}

.export-btn:hover {
  transform: translateY(-1px);
  box-shadow: 0 4px 12px rgba(66, 185, 131, 0.3);
}

@media (max-width: 768px) {
  .filters-export-bar {
    flex-direction: column;
    align-items: stretch;
  }

  .filters {
    flex-direction: column;
  }

  .export-buttons {
    width: 100%;
    justify-content: stretch;
  }

  .export-btn {
    flex: 1;
    justify-content: center;
  }
}
</style>
