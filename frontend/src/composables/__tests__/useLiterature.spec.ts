import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useLiterature } from '../useLiterature'
import { useLiteratureStore } from '../../stores/literature'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

describe('useLiterature Composable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  // =============================================================================
  // Reactive Refs Tests
  // =============================================================================

  it('exposes searchResults from store', () => {
    const { searchResults } = useLiterature()
    const store = useLiteratureStore()

    store.searchResults = [
      {
        source: 'PubMed',
        results: [
          {
            id: '1',
            source: 'PubMed',
            title: 'Test Paper',
            url: 'https://pubmed.ncbi.nlm.nih.gov/12345/',
            summary: null,
            relevance_score: null,
            indexed_at: new Date().toISOString()
          }
        ]
      }
    ]

    expect(searchResults.value).toHaveLength(1)
    expect(searchResults.value[0]?.source).toBe('PubMed')
  })

  it('exposes cachedLiterature from store', () => {
    const { cachedLiterature } = useLiterature()
    const store = useLiteratureStore()

    store.cachedLiterature = [
      {
        id: '1',
        source: 'PubMed',
        title: 'Cached Paper',
        url: 'https://example.com',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    expect(cachedLiterature.value).toHaveLength(1)
    expect(cachedLiterature.value[0]?.title).toBe('Cached Paper')
  })

  it('exposes searchLoading from store', () => {
    const { searchLoading } = useLiterature()
    const store = useLiteratureStore()

    expect(searchLoading.value).toBe(false)

    store.searchLoading = true
    expect(searchLoading.value).toBe(true)
  })

  it('exposes summarizing from store', () => {
    const { summarizing } = useLiterature()
    const store = useLiteratureStore()

    expect(summarizing.value).toBe(false)

    store.summarizing = true
    expect(summarizing.value).toBe(true)
  })

  it('exposes lastSearchQuery from store', () => {
    const { lastSearchQuery } = useLiterature()
    const store = useLiteratureStore()

    expect(lastSearchQuery.value).toBe('')

    store.lastSearchQuery = 'BPC-157'
    expect(lastSearchQuery.value).toBe('BPC-157')
  })

  it('exposes lastSearchSources from store', () => {
    const { lastSearchSources } = useLiterature()
    const store = useLiteratureStore()

    expect(lastSearchSources.value).toEqual(['PubMed'])

    store.lastSearchSources = ['PubMed', 'OpenAlex']
    expect(lastSearchSources.value).toEqual(['PubMed', 'OpenAlex'])
  })

  it('exposes currentSummary from store', () => {
    const { currentSummary } = useLiterature()
    const store = useLiteratureStore()

    expect(currentSummary.value).toBeNull()

    store.currentSummary = 'Test summary'
    expect(currentSummary.value).toBe('Test summary')
  })

  it('exposes summaryProvider from store', () => {
    const { summaryProvider } = useLiterature()
    const store = useLiteratureStore()

    expect(summaryProvider.value).toBeNull()

    store.summaryProvider = 'Claude'
    expect(summaryProvider.value).toBe('Claude')
  })

  // =============================================================================
  // Computed Getters Tests
  // =============================================================================

  it('exposes hasSearchResults from store', () => {
    const { hasSearchResults } = useLiterature()
    const store = useLiteratureStore()

    expect(hasSearchResults.value).toBe(false)

    store.searchResults = [
      {
        source: 'PubMed',
        results: [
          { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
        ]
      }
    ]

    expect(hasSearchResults.value).toBe(true)
  })

  it('exposes hasCachedLiterature from store', () => {
    const { hasCachedLiterature } = useLiterature()
    const store = useLiteratureStore()

    expect(hasCachedLiterature.value).toBe(false)

    store.cachedLiterature = [
      { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
    ]

    expect(hasCachedLiterature.value).toBe(true)
  })

  it('exposes hasSummary from store', () => {
    const { hasSummary } = useLiterature()
    const store = useLiteratureStore()

    expect(hasSummary.value).toBe(false)

    store.currentSummary = 'Test summary'

    expect(hasSummary.value).toBe(true)
  })

  it('exposes recentSearches from store', () => {
    const { recentSearches } = useLiterature()
    const store = useLiteratureStore()

    const now = new Date()
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)

    store.cachedLiterature = [
      { id: '1', source: 'PubMed', title: 'Old', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: yesterday.toISOString() },
      { id: '2', source: 'PubMed', title: 'New', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: now.toISOString() }
    ]

    expect(recentSearches.value).toHaveLength(2)
    expect(recentSearches.value[0]?.title).toBe('New') // Most recent first
  })

  // =============================================================================
  // Action Methods Tests
  // =============================================================================

  it('search calls API with correct payload', async () => {
    const { search } = useLiterature()

    const mockResults = [
      {
        source: 'PubMed',
        results: [
          { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
        ]
      }
    ]

    vi.mocked(api.searchLiterature).mockResolvedValue(mockResults)

    await search('BPC-157', ['PubMed'], 10)

    expect(api.searchLiterature).toHaveBeenCalledWith({
      query: 'BPC-157',
      sources: ['PubMed'],
      maxResults: 10
    })
  })

  it('search uses default sources when not specified', async () => {
    const { search } = useLiterature()

    vi.mocked(api.searchLiterature).mockResolvedValue([])

    await search('BPC-157')

    expect(api.searchLiterature).toHaveBeenCalledWith({
      query: 'BPC-157',
      sources: ['PubMed'],
      maxResults: undefined
    })
  })

  it('search returns empty array for empty query', async () => {
    const { search } = useLiterature()

    const results = await search('   ')

    expect(results).toEqual([])
    expect(api.searchLiterature).not.toHaveBeenCalled()
  })

  it('search updates store state', async () => {
    const { search, lastSearchQuery, lastSearchSources } = useLiterature()

    const mockResults = [
      {
        source: 'OpenAlex',
        results: []
      }
    ]

    vi.mocked(api.searchLiterature).mockResolvedValue(mockResults)

    await search('TB-500', ['OpenAlex'])

    expect(lastSearchQuery.value).toBe('TB-500')
    expect(lastSearchSources.value).toEqual(['OpenAlex'])
  })

  it('fetchCachedLiterature calls listLiterature when no query', async () => {
    const { fetchCachedLiterature } = useLiterature()

    const mockEntries = [
      { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
    ]

    vi.mocked(api.listLiterature).mockResolvedValue(mockEntries)

    await fetchCachedLiterature()

    expect(api.listLiterature).toHaveBeenCalled()
  })

  it('fetchCachedLiterature calls searchCachedLiterature with query', async () => {
    const { fetchCachedLiterature } = useLiterature()

    const mockEntries = [
      { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
    ]

    vi.mocked(api.searchCachedLiterature).mockResolvedValue(mockEntries)

    await fetchCachedLiterature('BPC-157')

    expect(api.searchCachedLiterature).toHaveBeenCalledWith('BPC-157')
  })

  it('summarize calls API with correct payload', async () => {
    const { summarize } = useLiterature()

    const mockResult = {
      output: 'Summary text',
      provider: 'Claude'
    }

    vi.mocked(api.summarizeContent).mockResolvedValue(mockResult)

    await summarize('Test Title', 'Test content', 'Markdown')

    expect(api.summarizeContent).toHaveBeenCalledWith({
      title: 'Test Title',
      content: 'Test content',
      format: 'Markdown'
    })
  })

  it('summarize uses default format when not specified', async () => {
    const { summarize } = useLiterature()

    const mockResult = {
      output: 'Summary text',
      provider: 'Claude'
    }

    vi.mocked(api.summarizeContent).mockResolvedValue(mockResult)

    await summarize('Test Title', 'Test content')

    expect(api.summarizeContent).toHaveBeenCalledWith({
      title: 'Test Title',
      content: 'Test content',
      format: 'Markdown'
    })
  })

  it('summarize updates store state', async () => {
    const { summarize, currentSummary, summaryProvider } = useLiterature()

    const mockResult = {
      output: 'Summary text',
      provider: 'Codex'
    }

    vi.mocked(api.summarizeContent).mockResolvedValue(mockResult)

    await summarize('Test Title', 'Test content')

    expect(currentSummary.value).toBe('Summary text')
    expect(summaryProvider.value).toBe('Codex')
  })

  it('summarize throws error for empty title', async () => {
    const { summarize } = useLiterature()

    await expect(summarize('', 'Test content')).rejects.toThrow('Title and content are required')
  })

  it('summarize throws error for empty content', async () => {
    const { summarize } = useLiterature()

    await expect(summarize('Test Title', '')).rejects.toThrow('Title and content are required')
  })

  it('clearSearch clears search state', () => {
    const { clearSearch } = useLiterature()
    const store = useLiteratureStore()

    store.searchResults = [
      {
        source: 'PubMed',
        results: [
          { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
        ]
      }
    ]
    store.lastSearchQuery = 'BPC-157'

    clearSearch()

    expect(store.searchResults).toEqual([])
    expect(store.lastSearchQuery).toBe('')
  })

  it('clearSummary clears summary state', () => {
    const { clearSummary } = useLiterature()
    const store = useLiteratureStore()

    store.currentSummary = 'Test summary'
    store.summaryProvider = 'Claude'

    clearSummary()

    expect(store.currentSummary).toBeNull()
    expect(store.summaryProvider).toBeNull()
  })

  it('clearAll clears all state', () => {
    const { clearAll } = useLiterature()
    const store = useLiteratureStore()

    store.searchResults = [{ source: 'PubMed', results: [] }]
    store.currentSummary = 'Summary'
    store.cachedLiterature = [
      { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
    ]

    clearAll()

    expect(store.searchResults).toEqual([])
    expect(store.currentSummary).toBeNull()
    expect(store.cachedLiterature).toEqual([])
  })

  // =============================================================================
  // Reactivity Tests
  // =============================================================================

  it('searchResults ref updates when store changes', () => {
    const { searchResults } = useLiterature()
    const store = useLiteratureStore()

    expect(searchResults.value).toEqual([])

    store.searchResults = [
      {
        source: 'PubMed',
        results: [
          { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
        ]
      }
    ]

    expect(searchResults.value).toHaveLength(1)
  })

  it('computed properties are reactive', () => {
    const { hasSearchResults, hasSummary } = useLiterature()
    const store = useLiteratureStore()

    expect(hasSearchResults.value).toBe(false)
    expect(hasSummary.value).toBe(false)

    store.searchResults = [
      {
        source: 'PubMed',
        results: [
          { id: '1', source: 'PubMed', title: 'Test', url: 'https://example.com', summary: null, relevance_score: null, indexed_at: new Date().toISOString() }
        ]
      }
    ]
    store.currentSummary = 'Summary'

    expect(hasSearchResults.value).toBe(true)
    expect(hasSummary.value).toBe(true)
  })
})
