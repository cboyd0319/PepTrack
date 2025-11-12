/**
 * Literature Composable
 * Convenient wrapper around literature store
 */

import { useLiteratureStore } from '../stores'
import { storeToRefs } from 'pinia'

export function useLiterature() {
  const store = useLiteratureStore()

  const {
    searchResults,
    cachedLiterature,
    searchLoading,
    summarizing,
    lastSearchQuery,
    lastSearchSources,
    currentSummary,
    summaryProvider,
    hasSearchResults,
    hasCachedLiterature,
    hasSummary,
    recentSearches
  } = storeToRefs(store)

  const {
    search,
    fetchCachedLiterature,
    summarize,
    clearSearch,
    clearSummary,
    clearAll
  } = store

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
}
