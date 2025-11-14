import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useProtocols } from '../useProtocols'
import { useProtocolStore } from '../../stores/protocols'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

describe('useProtocols Composable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  // =============================================================================
  // Reactive Refs Tests
  // =============================================================================

  it('exposes protocols from store', () => {
    const { protocols } = useProtocols()
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'Test', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(protocols.value).toHaveLength(1)
    expect(protocols.value[0]?.name).toBe('Test')
  })

  it('exposes loading state from store', () => {
    const { loading } = useProtocols()
    const store = useProtocolStore()

    expect(loading.value).toBe(false)

    store.loading = true
    expect(loading.value).toBe(true)
  })

  it('exposes protocolCount from store', () => {
    const { protocolCount } = useProtocols()
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'P1', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'P2', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(protocolCount.value).toBe(2)
  })

  // =============================================================================
  // Computed Helpers Tests
  // =============================================================================

  it('computes hasProtocols correctly', () => {
    const { hasProtocols } = useProtocols()
    const store = useProtocolStore()

    expect(hasProtocols.value).toBe(false)

    store.protocols = [
      { id: '1', name: 'Test', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasProtocols.value).toBe(true)
  })

  it('computes isEmpty correctly when not loading', () => {
    const { isEmpty } = useProtocols()
    const store = useProtocolStore()

    store.loading = false
    store.protocols = []

    expect(isEmpty.value).toBe(true)
  })

  it('isEmpty is false when loading', () => {
    const { isEmpty } = useProtocols()
    const store = useProtocolStore()

    store.loading = true
    store.protocols = []

    expect(isEmpty.value).toBe(false)
  })

  it('isEmpty is false when has protocols', () => {
    const { isEmpty } = useProtocols()
    const store = useProtocolStore()

    store.loading = false
    store.protocols = [
      { id: '1', name: 'Test', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(isEmpty.value).toBe(false)
  })

  // =============================================================================
  // Action Methods Tests
  // =============================================================================

  it('refreshProtocols calls fetchProtocols', async () => {
    const { refreshProtocols } = useProtocols()

    vi.mocked(api.listProtocols).mockResolvedValue([])

    await refreshProtocols()

    expect(api.listProtocols).toHaveBeenCalled()
  })

  it('refreshProtocols passes force flag', async () => {
    const { refreshProtocols } = useProtocols()
    const store = useProtocolStore()

    vi.mocked(api.listProtocols).mockResolvedValue([])

    // Set up cache
    store.protocols = [
      { id: '1', name: 'Cached', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]
    store.lastFetch = Date.now()

    await refreshProtocols(true)

    // Should have called API even with valid cache
    expect(api.listProtocols).toHaveBeenCalled()
  })

  it('addProtocol creates protocol with correct payload', async () => {
    const { addProtocol } = useProtocols()

    const newProtocol = {
      id: '1',
      name: 'New Protocol',
      peptide_name: 'BPC-157',
      notes: 'Test notes',
      target_concentration_mg_ml: 2.5,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.saveProtocol).mockResolvedValue(newProtocol)

    await addProtocol('New Protocol', 'BPC-157', 'Test notes', 2.5)

    expect(api.saveProtocol).toHaveBeenCalledWith({
      name: 'New Protocol',
      peptideName: 'BPC-157',
      notes: 'Test notes',
      targetConcentrationMgMl: 2.5
    })
  })

  it('addProtocol works with minimal parameters', async () => {
    const { addProtocol } = useProtocols()

    const newProtocol = {
      id: '1',
      name: 'Simple',
      peptide_name: 'TB-500',
      notes: null,
      target_concentration_mg_ml: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.saveProtocol).mockResolvedValue(newProtocol)

    await addProtocol('Simple', 'TB-500')

    expect(api.saveProtocol).toHaveBeenCalledWith({
      name: 'Simple',
      peptideName: 'TB-500',
      notes: undefined,
      targetConcentrationMgMl: undefined
    })
  })

  // =============================================================================
  // Store Method Access Tests
  // =============================================================================

  it('exposes fetchProtocols from store', async () => {
    const { fetchProtocols } = useProtocols()

    vi.mocked(api.listProtocols).mockResolvedValue([])

    await fetchProtocols()

    expect(api.listProtocols).toHaveBeenCalled()
  })

  it('exposes createProtocol from store', async () => {
    const { createProtocol } = useProtocols()

    const newProtocol = {
      id: '1',
      name: 'Test',
      peptide_name: 'BPC-157',
      notes: null,
      target_concentration_mg_ml: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.saveProtocol).mockResolvedValue(newProtocol)

    await createProtocol({ name: 'Test', peptideName: 'BPC-157' })

    expect(api.saveProtocol).toHaveBeenCalled()
  })

  it('exposes updateProtocol from store', async () => {
    const { updateProtocol } = useProtocols()
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'Original', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.saveProtocol).mockResolvedValue({
      id: '1',
      name: 'Updated',
      peptide_name: 'BPC-157',
      notes: null,
      target_concentration_mg_ml: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    })

    await updateProtocol('1', { name: 'Updated' })

    expect(store.protocols[0]?.name).toBe('Updated')
  })

  it('exposes removeProtocol from store', async () => {
    const { removeProtocol } = useProtocols()
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'To Remove', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    await removeProtocol('1')

    expect(store.protocols).toHaveLength(0)
  })

  it('exposes getProtocolById from store', () => {
    const { getProtocolById } = useProtocols()
    const store = useProtocolStore()

    const target = {
      id: '2',
      name: 'Target',
      peptide_name: 'BPC-157',
      notes: null,
      target_concentration_mg_ml: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    store.protocols = [
      { id: '1', name: 'Other', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      target
    ]

    const found = getProtocolById('2')

    expect(found).toEqual(target)
  })

  it('exposes searchProtocols from store', () => {
    const { searchProtocols } = useProtocols()
    const store = useProtocolStore()

    store.protocols = [
      { id: '1', name: 'BPC Stack', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'TB Routine', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const results = searchProtocols('BPC')

    expect(results).toHaveLength(1)
    expect(results[0]?.name).toBe('BPC Stack')
  })

  // =============================================================================
  // Reactivity Tests
  // =============================================================================

  it('protocols ref updates when store changes', () => {
    const { protocols } = useProtocols()
    const store = useProtocolStore()

    expect(protocols.value).toHaveLength(0)

    store.protocols = [
      { id: '1', name: 'New', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(protocols.value).toHaveLength(1)
  })

  it('computed properties are reactive', () => {
    const { hasProtocols, isEmpty } = useProtocols()
    const store = useProtocolStore()

    expect(hasProtocols.value).toBe(false)
    expect(isEmpty.value).toBe(true)

    store.protocols = [
      { id: '1', name: 'Test', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasProtocols.value).toBe(true)
    expect(isEmpty.value).toBe(false)
  })
})
