/**
 * Literature Store
 * State management for literature search and AI summaries
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { LiteratureEntry, LiteratureSearchResult, SummaryFormat, SearchLiteraturePayload } from '../api/peptrack'
import {
  searchLiterature,
  searchCachedLiterature,
  listLiterature,
  summarizeContent
} from '../api/peptrack'
import { showErrorToast, showSuccessToast } from '../utils/errorHandling'

export const useLiteratureStore = defineStore('literature', () => {
  // State
  const searchResults = ref<LiteratureSearchResult[]>([])
  const cachedLiterature = ref<LiteratureEntry[]>([])
  const searchLoading = ref(false)
  const summarizing = ref(false)

  // Current search params
  const lastSearchQuery = ref<string>('')
  const lastSearchSources = ref<string[]>(['PubMed'])

  // AI Summary state
  const currentSummary = ref<string | null>(null)
  const summaryProvider = ref<string | null>(null)

  // Getters
  const hasSearchResults = computed(() => searchResults.value.length > 0)
  const hasCachedLiterature = computed(() => cachedLiterature.value.length > 0)
  const hasSummary = computed(() => currentSummary.value !== null)

  const recentSearches = computed(() =>
    [...cachedLiterature.value]
      .sort((a, b) => new Date(b.indexed_at).getTime() - new Date(a.indexed_at).getTime())
      .slice(0, 10)
  )

  // Actions
  async function search(query: string, sources: string[] = ['PubMed'], maxResults?: number) {
    if (!query.trim()) {
      return []
    }

    searchLoading.value = true
    lastSearchQuery.value = query
    lastSearchSources.value = sources

    try {
      const payload: SearchLiteraturePayload = {
        query,
        sources,
        maxResults
      }
      const results = await searchLiterature(payload)
      searchResults.value = results
      const totalResults = results.reduce((sum, r) => sum + r.results.length, 0)
      showSuccessToast('Search Complete', `Found ${totalResults} results from ${sources.join(', ')}`)
      return results
    } catch (error) {
      showErrorToast(error, { operation: `search ${sources.join(', ')}` })
      searchResults.value = []
      throw error
    } finally {
      searchLoading.value = false
    }
  }

  async function fetchCachedLiterature(query?: string) {
    try {
      if (query) {
        cachedLiterature.value = await searchCachedLiterature(query)
      } else {
        cachedLiterature.value = await listLiterature()
      }
      return cachedLiterature.value
    } catch (error) {
      showErrorToast(error, { operation: 'load cached literature' })
      throw error
    }
  }

  async function summarize(
    title: string,
    content: string,
    format: SummaryFormat = 'Markdown'
  ) {
    if (!title || !content) {
      throw new Error('Title and content are required for summarization')
    }

    summarizing.value = true
    try {
      const result = await summarizeContent({ title, content, format })
      currentSummary.value = result.output
      summaryProvider.value = result.provider

      showSuccessToast('Summary Generated', `Summary generated using ${result.provider}`)
      return result
    } catch (error) {
      showErrorToast(error, { operation: 'generate AI summary' })
      throw error
    } finally {
      summarizing.value = false
    }
  }

  function clearSearch() {
    searchResults.value = []
    lastSearchQuery.value = ''
  }

  function clearSummary() {
    currentSummary.value = null
    summaryProvider.value = null
  }

  function clearAll() {
    clearSearch()
    clearSummary()
    cachedLiterature.value = []
  }

  return {
    // State
    searchResults,
    cachedLiterature,
    searchLoading,
    summarizing,
    lastSearchQuery,
    lastSearchSources,
    currentSummary,
    summaryProvider,

    // Getters
    hasSearchResults,
    hasCachedLiterature,
    hasSummary,
    recentSearches,

    // Actions
    search,
    fetchCachedLiterature,
    summarize,
    clearSearch,
    clearSummary,
    clearAll
  }
})
