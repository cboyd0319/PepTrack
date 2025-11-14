import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useLiteratureStore } from '../literature'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast } from '../../utils/errorHandling'

describe('Literature Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useLiteratureStore()

    expect(store.entries).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.searchResults).toEqual([])
  })

  it('fetches cached literature from API', async () => {
    const store = useLiteratureStore()
    const mockEntries = [
      {
        id: '1',
        source: 'pubmed',
        title: 'BPC-157 Research',
        url: 'https://pubmed.ncbi.nlm.nih.gov/12345/',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listLiterature).mockResolvedValue(mockEntries)

    await store.fetchLiterature()

    expect(api.listLiterature).toHaveBeenCalled()
    expect(store.entries).toEqual(mockEntries)
  })

  it('searches PubMed', async () => {
    const store = useLiteratureStore()
    const mockResults = [
      {
        id: '1',
        source: 'pubmed',
        title: 'BPC-157 Study',
        url: 'https://pubmed.ncbi.nlm.nih.gov/12345/',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.searchPubMed).mockResolvedValue(mockResults)

    await store.searchPubMed('BPC-157', 10)

    expect(api.searchPubMed).toHaveBeenCalledWith('BPC-157', 10)
    expect(store.searchResults).toEqual(mockResults)
  })

  it('searches OpenAlex', async () => {
    const store = useLiteratureStore()
    const mockResults = [
      {
        id: '1',
        source: 'openalex',
        title: 'Peptide Study',
        url: 'https://openalex.org/W12345',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.searchOpenAlex).mockResolvedValue(mockResults)

    await store.searchOpenAlex('peptides', 10)

    expect(api.searchOpenAlex).toHaveBeenCalledWith('peptides', 10)
    expect(store.searchResults).toEqual(mockResults)
  })

  it('searches Crossref', async () => {
    const store = useLiteratureStore()
    const mockResults = [
      {
        id: '1',
        source: 'crossref',
        title: 'Research Paper',
        url: 'https://doi.org/10.1000/xyz',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.searchCrossref).mockResolvedValue(mockResults)

    await store.searchCrossref('research', 10)

    expect(api.searchCrossref).toHaveBeenCalledWith('research', 10)
    expect(store.searchResults).toEqual(mockResults)
  })

  it('caches literature entry', async () => {
    const store = useLiteratureStore()
    const entry = {
      id: '1',
      source: 'pubmed',
      title: 'New Entry',
      url: 'https://example.com',
      summary: null,
      relevance_score: null,
      indexed_at: new Date().toISOString()
    }

    vi.mocked(api.cacheLiterature).mockResolvedValue(entry)

    await store.cacheEntry(entry)

    expect(api.cacheLiterature).toHaveBeenCalledWith(entry)
    expect(store.entries).toContain(entry)
  })

  it('clears search results', () => {
    const store = useLiteratureStore()
    store.searchResults = [
      {
        id: '1',
        source: 'pubmed',
        title: 'Result',
        url: 'https://example.com',
        summary: null,
        relevance_score: null,
        indexed_at: new Date().toISOString()
      }
    ]

    store.clearSearchResults()

    expect(store.searchResults).toEqual([])
  })

  it('handles search errors', async () => {
    const store = useLiteratureStore()

    vi.mocked(api.searchPubMed).mockRejectedValue(new Error('API error'))

    await expect(store.searchPubMed('query', 10)).rejects.toThrow()
    expect(showErrorToast).toHaveBeenCalled()
  })
})
