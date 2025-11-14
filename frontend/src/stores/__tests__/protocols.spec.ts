import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProtocolStore } from '../protocols'
import * as api from '../../api/peptrack'

// Mock the API module
vi.mock('../../api/peptrack')

// Mock toast utilities
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('Protocol Store', () => {
  beforeEach(() => {
    // Create a fresh Pinia instance for each test
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  // =============================================================================
  // Initial State Tests
  // =============================================================================

  it('initializes with empty state', () => {
    const store = useProtocolStore()

    expect(store.protocols).toEqual([])
    expect(store.loading).toBe(false)
    expect(store.lastFetch).toBeNull()
  })

  // =============================================================================
  // Computed Properties Tests
  // =============================================================================

  it('computes protocol count correctly', () => {
    const store = useProtocolStore()

    expect(store.protocolCount).toBe(0)

    store.protocols = [
      { id: '1', name: 'Protocol 1', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'Protocol 2', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(store.protocolCount).toBe(2)
  })

  it('computes activeProtocols correctly', () => {
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'Active', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(store.activeProtocols).toHaveLength(1)
  })

  it('groups protocols by peptide', () => {
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'Morning BPC', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'Evening BPC', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '3', name: 'Morning TB', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const grouped = store.protocolsByPeptide

    expect(grouped.get('BPC-157')).toHaveLength(2)
    expect(grouped.get('TB-500')).toHaveLength(1)
  })

  it('validates cache correctly', () => {
    const store = useProtocolStore()

    // No fetch yet - cache invalid
    expect(store.isCacheValid).toBe(false)

    // Set recent fetch
    store.lastFetch = Date.now()
    expect(store.isCacheValid).toBe(true)

    // Set old fetch (> 30 seconds ago)
    store.lastFetch = Date.now() - 35000
    expect(store.isCacheValid).toBe(false)
  })

  // =============================================================================
  // fetchProtocols Tests
  // =============================================================================

  it('fetches protocols from API', async () => {
    const store = useProtocolStore()
    const mockProtocols = [
      { id: '1', name: 'Test Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.listProtocols).mockResolvedValue(mockProtocols)

    await store.fetchProtocols()

    expect(api.listProtocols).toHaveBeenCalled()
    expect(store.protocols).toEqual(mockProtocols)
    expect(store.lastFetch).toBeTruthy()
    expect(store.loading).toBe(false)
  })

  it('uses cache when valid and not forcing refresh', async () => {
    const store = useProtocolStore()
    const mockProtocols = [
      { id: '1', name: 'Cached', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    store.protocols = mockProtocols
    store.lastFetch = Date.now()

    vi.mocked(api.listProtocols).mockResolvedValue([])

    await store.fetchProtocols(false)

    // Should NOT call API when cache is valid
    expect(api.listProtocols).not.toHaveBeenCalled()
    expect(store.protocols).toEqual(mockProtocols)
  })

  it('forces refresh when explicitly requested', async () => {
    const store = useProtocolStore()
    const newProtocols = [
      { id: '2', name: 'Fresh', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    store.protocols = [
      { id: '1', name: 'Cached', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]
    store.lastFetch = Date.now()

    vi.mocked(api.listProtocols).mockResolvedValue(newProtocols)

    await store.fetchProtocols(true)

    expect(api.listProtocols).toHaveBeenCalled()
    expect(store.protocols).toEqual(newProtocols)
  })

  it('handles fetch errors gracefully', async () => {
    const store = useProtocolStore()

    vi.mocked(api.listProtocols).mockRejectedValue(new Error('Network error'))

    await expect(store.fetchProtocols()).rejects.toThrow('Network error')

    expect(showErrorToast).toHaveBeenCalled()
    expect(store.loading).toBe(false)
  })

  it('sets loading state during fetch', async () => {
    const store = useProtocolStore()
    let loadingDuringFetch = false

    vi.mocked(api.listProtocols).mockImplementation(async () => {
      loadingDuringFetch = store.loading
      return []
    })

    await store.fetchProtocols()

    expect(loadingDuringFetch).toBe(true)
    expect(store.loading).toBe(false)
  })

  // =============================================================================
  // createProtocol Tests
  // =============================================================================

  it('creates new protocol and adds to store', async () => {
    const store = useProtocolStore()
    const newProtocol = { id: '1', name: 'New Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }

    vi.mocked(api.saveProtocol).mockResolvedValue(newProtocol)

    const result = await store.createProtocol({
      name: 'New Protocol',
      peptideName: 'BPC-157'
    })

    expect(api.saveProtocol).toHaveBeenCalledWith({
      name: 'New Protocol',
      peptideName: 'BPC-157'
    })
    expect(result).toEqual(newProtocol)
    expect(store.protocols).toContain(newProtocol)
    expect(showSuccessToast).toHaveBeenCalled()
  })

  it('optimistically adds protocol to beginning of list', async () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Existing', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const newProtocol = { id: '2', name: 'New', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }

    vi.mocked(api.saveProtocol).mockResolvedValue(newProtocol)

    await store.createProtocol({
      name: 'New',
      peptideName: 'BPC-157'
    })

    expect(store.protocols[0]).toEqual(newProtocol)
    expect(store.protocols).toHaveLength(2)
  })

  it('refreshes on create error', async () => {
    const store = useProtocolStore()
    const existingProtocols = [
      { id: '1', name: 'Existing', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    store.protocols = [...existingProtocols]

    vi.mocked(api.saveProtocol).mockRejectedValue(new Error('Save failed'))
    vi.mocked(api.listProtocols).mockResolvedValue(existingProtocols)

    await expect(store.createProtocol({
      name: 'New',
      peptideName: 'BPC-157'
    })).rejects.toThrow('Save failed')

    expect(showErrorToast).toHaveBeenCalled()
    expect(api.listProtocols).toHaveBeenCalled()
  })

  // =============================================================================
  // updateProtocol Tests
  // =============================================================================

  it('updates existing protocol optimistically', async () => {
    const store = useProtocolStore()
    const original = { id: '1', name: 'Original', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }

    store.protocols = [original]

    vi.mocked(api.saveProtocol).mockResolvedValue({
      ...original,
      name: 'Updated'
    })

    await store.updateProtocol('1', { name: 'Updated' })

    expect(store.protocols[0]?.name).toBe('Updated')
  })

  it('preserves other fields when updating', async () => {
    const store = useProtocolStore()
    const original = {
      id: '1',
      name: 'Original',
      peptide_name: 'BPC-157',
      notes: 'Original notes',
      target_concentration_mg_ml: 2.5,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    store.protocols = [original]

    vi.mocked(api.saveProtocol).mockResolvedValue({
      ...original,
      name: 'Updated'
    })

    await store.updateProtocol('1', { name: 'Updated' })

    expect(store.protocols[0]?.notes).toBe('Original notes')
    expect(store.protocols[0]?.target_concentration_mg_ml).toBe(2.5)
  })

  it('rolls back on update error', async () => {
    const store = useProtocolStore()
    const original = [
      { id: '1', name: 'Original', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    store.protocols = [...original]

    vi.mocked(api.saveProtocol).mockRejectedValue(new Error('Update failed'))

    await expect(store.updateProtocol('1', { name: 'Failed Update' })).rejects.toThrow('Update failed')

    // Should rollback to original
    expect(store.protocols[0]?.name).toBe('Original')
    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // removeProtocol Tests
  // =============================================================================

  it('removes protocol from store', async () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'To Remove', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'Keep', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    await store.removeProtocol('1')

    expect(store.protocols).toHaveLength(1)
    expect(store.protocols[0]?.id).toBe('2')
  })

  // =============================================================================
  // getProtocolById Tests
  // =============================================================================

  it('finds protocol by ID', () => {
    const store = useProtocolStore()
    const target = { id: '2', name: 'Target', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }

    store.protocols = [
      { id: '1', name: 'Other', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      target,
      { id: '3', name: 'Another', peptide_name: 'GHK-Cu', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const found = store.getProtocolById('2')

    expect(found).toEqual(target)
  })

  it('returns undefined for nonexistent ID', () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const found = store.getProtocolById('nonexistent')

    expect(found).toBeUndefined()
  })

  // =============================================================================
  // searchProtocols Tests
  // =============================================================================

  it('searches protocols by name', () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Morning BPC Stack', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'Evening TB Routine', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '3', name: 'BPC Recovery', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const results = store.searchProtocols('BPC')

    expect(results).toHaveLength(2)
    expect(results.every(p => p.name.includes('BPC'))).toBe(true)
  })

  it('searches protocols by peptide name', () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Protocol 1', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'Protocol 2', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const results = store.searchProtocols('TB-500')

    expect(results).toHaveLength(1)
    expect(results[0]?.peptide_name).toBe('TB-500')
  })

  it('returns empty array for no matches', () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const results = store.searchProtocols('nonexistent')

    expect(results).toEqual([])
  })

  it('search is case-insensitive', () => {
    const store = useProtocolStore()
    store.protocols = [
      { id: '1', name: 'Morning Stack', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const results = store.searchProtocols('morning')

    expect(results).toHaveLength(1)
  })
})
