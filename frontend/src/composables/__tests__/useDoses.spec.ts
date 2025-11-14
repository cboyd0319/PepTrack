import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useDoses } from '../useDoses'
import { useDoseStore } from '../../stores/doses'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

describe('useDoses Composable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  // =============================================================================
  // Reactive Refs Tests
  // =============================================================================

  it('exposes doses from store', () => {
    const { doses } = useDoses()
    const store = useDoseStore()

    store.doses = [
      {
        id: '1',
        protocol_id: 'p1',
        injection_site: 'deltoid',
        amount_mg: 2.5,
        notes: null,
        logged_at: new Date().toISOString()
      }
    ]

    expect(doses.value).toHaveLength(1)
    expect(doses.value[0]?.amount_mg).toBe(2.5)
  })

  it('exposes loading state from store', () => {
    const { loading } = useDoses()
    const store = useDoseStore()

    expect(loading.value).toBe(false)

    store.loading = true
    expect(loading.value).toBe(true)
  })

  it('exposes doseCount from store', () => {
    const { doseCount } = useDoses()
    const store = useDoseStore()

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', injection_site: 'quad', amount_mg: 3.0, notes: null, logged_at: new Date().toISOString() }
    ]

    expect(doseCount.value).toBe(2)
  })

  it('exposes recentDoses from store', () => {
    const { recentDoses } = useDoses()
    const store = useDoseStore()

    const now = new Date()
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: yesterday.toISOString() },
      { id: '2', protocol_id: 'p1', injection_site: 'quad', amount_mg: 3.0, notes: null, logged_at: now.toISOString() }
    ]

    expect(recentDoses.value).toHaveLength(2)
    expect(recentDoses.value[0]?.id).toBe('2') // Most recent first
  })

  it('exposes dosesThisWeek from store', () => {
    const { dosesThisWeek } = useDoses()
    const store = useDoseStore()

    const now = new Date()
    const fiveDaysAgo = new Date(now.getTime() - 5 * 24 * 60 * 60 * 1000)
    const tenDaysAgo = new Date(now.getTime() - 10 * 24 * 60 * 60 * 1000)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: tenDaysAgo.toISOString() },
      { id: '2', protocol_id: 'p1', injection_site: 'quad', amount_mg: 3.0, notes: null, logged_at: fiveDaysAgo.toISOString() }
    ]

    expect(dosesThisWeek.value).toHaveLength(1)
    expect(dosesThisWeek.value[0]?.id).toBe('2')
  })

  it('exposes dosesThisMonth from store', () => {
    const { dosesThisMonth } = useDoses()
    const store = useDoseStore()

    const now = new Date()
    const fifteenDaysAgo = new Date(now.getTime() - 15 * 24 * 60 * 60 * 1000)
    const fortyDaysAgo = new Date(now.getTime() - 40 * 24 * 60 * 60 * 1000)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: fortyDaysAgo.toISOString() },
      { id: '2', protocol_id: 'p1', injection_site: 'quad', amount_mg: 3.0, notes: null, logged_at: fifteenDaysAgo.toISOString() }
    ]

    expect(dosesThisMonth.value).toHaveLength(1)
    expect(dosesThisMonth.value[0]?.id).toBe('2')
  })

  // =============================================================================
  // Computed Helpers Tests
  // =============================================================================

  it('computes hasDoses correctly', () => {
    const { hasDoses } = useDoses()
    const store = useDoseStore()

    expect(hasDoses.value).toBe(false)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    expect(hasDoses.value).toBe(true)
  })

  // =============================================================================
  // Action Methods Tests
  // =============================================================================

  it('fetchDoses calls API without protocol ID', async () => {
    const { fetchDoses } = useDoses()

    const mockDoses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    vi.mocked(api.listDoseLogs).mockResolvedValue(mockDoses)

    await fetchDoses()

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  it('fetchDoses calls API with protocol ID', async () => {
    const { fetchDoses } = useDoses()

    const mockDoses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    vi.mocked(api.listDoseLogsForProtocol).mockResolvedValue(mockDoses)

    await fetchDoses('p1')

    expect(api.listDoseLogsForProtocol).toHaveBeenCalledWith('p1')
  })

  it('logDose creates dose with correct payload', async () => {
    const { logDose } = useDoses()

    const newDose = {
      id: '1',
      protocol_id: 'p1',
      injection_site: 'deltoid',
      amount_mg: 2.5,
      notes: 'Test notes',
      logged_at: new Date().toISOString()
    }

    vi.mocked(api.logDose).mockResolvedValue(newDose)

    await logDose('p1', 'deltoid', 2.5, 'Test notes')

    expect(api.logDose).toHaveBeenCalledWith({
      protocolId: 'p1',
      site: 'deltoid',
      amountMg: 2.5,
      notes: 'Test notes'
    })
  })

  it('logDose works without notes', async () => {
    const { logDose } = useDoses()

    const newDose = {
      id: '1',
      protocol_id: 'p1',
      injection_site: 'deltoid',
      amount_mg: 2.5,
      notes: null,
      logged_at: new Date().toISOString()
    }

    vi.mocked(api.logDose).mockResolvedValue(newDose)

    await logDose('p1', 'deltoid', 2.5)

    expect(api.logDose).toHaveBeenCalledWith({
      protocolId: 'p1',
      site: 'deltoid',
      amountMg: 2.5,
      notes: undefined
    })
  })

  it('removeDose deletes dose', async () => {
    const { removeDose } = useDoses()
    const store = useDoseStore()

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    vi.mocked(api.deleteDoseLog).mockResolvedValue(undefined)

    await removeDose('p1', '1')

    expect(api.deleteDoseLog).toHaveBeenCalledWith('1')
    expect(store.doses).toHaveLength(0)
  })

  // =============================================================================
  // Store Method Access Tests
  // =============================================================================

  it('exposes getDosesForProtocol from store', () => {
    const { getDosesForProtocol } = useDoses()
    const store = useDoseStore()

    const dose = {
      id: '1',
      protocol_id: 'p1',
      injection_site: 'deltoid',
      amount_mg: 2.5,
      notes: null,
      logged_at: new Date().toISOString()
    }

    store.dosesByProtocol.set('p1', [dose])

    const doses = getDosesForProtocol('p1')

    expect(doses).toHaveLength(1)
    expect(doses[0]?.id).toBe('1')
  })

  // =============================================================================
  // Reactivity Tests
  // =============================================================================

  it('doses ref updates when store changes', () => {
    const { doses } = useDoses()
    const store = useDoseStore()

    expect(doses.value).toHaveLength(0)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    expect(doses.value).toHaveLength(1)
  })

  it('computed properties are reactive', () => {
    const { hasDoses, doseCount } = useDoses()
    const store = useDoseStore()

    expect(hasDoses.value).toBe(false)
    expect(doseCount.value).toBe(0)

    store.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    expect(hasDoses.value).toBe(true)
    expect(doseCount.value).toBe(1)
  })
})
