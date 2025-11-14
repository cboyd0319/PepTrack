import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useDoseStore } from '../doses'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('Dose Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useDoseStore()

    expect(store.doses).toEqual([])
    expect(store.loading).toBe(false)
  })

  it('fetches doses from API', async () => {
    const store = useDoseStore()
    const mockDoses = [
      { id: '1', protocol_id: 'p1', site: 'Left Shoulder', amount_mg: 0.5, notes: null, logged_at: new Date().toISOString() }
    ]

    vi.mocked(api.listDoseLogs).mockResolvedValue(mockDoses)

    await store.fetchDoses()

    expect(api.listDoseLogs).toHaveBeenCalled()
    expect(store.doses).toEqual(mockDoses)
  })

  it('creates new dose log', async () => {
    const store = useDoseStore()
    const newDose = {
      id: '1',
      protocol_id: 'p1',
      site: 'Right Shoulder',
      amount_mg: 0.75,
      notes: null,
      logged_at: new Date().toISOString()
    }

    vi.mocked(api.saveDoseLog).mockResolvedValue(newDose)

    await store.createDose({
      protocolId: 'p1',
      site: 'Right Shoulder',
      amountMg: 0.75
    })

    expect(store.doses).toContain(newDose)
    expect(showSuccessToast).toHaveBeenCalled()
  })

  it('deletes dose log', async () => {
    const store = useDoseStore()
    store.doses = [
      { id: '1', protocol_id: 'p1', site: 'Site', amount_mg: 0.5, notes: null, logged_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', site: 'Site 2', amount_mg: 0.6, notes: null, logged_at: new Date().toISOString() }
    ]

    vi.mocked(api.deleteDoseLog).mockResolvedValue(undefined)

    await store.deleteDose('1')

    expect(store.doses).toHaveLength(1)
    expect(store.doses[0]?.id).toBe('2')
  })

  it('filters doses by protocol', () => {
    const store = useDoseStore()
    store.doses = [
      { id: '1', protocol_id: 'p1', site: 'Site 1', amount_mg: 0.5, notes: null, logged_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p2', site: 'Site 2', amount_mg: 0.6, notes: null, logged_at: new Date().toISOString() },
      { id: '3', protocol_id: 'p1', site: 'Site 3', amount_mg: 0.7, notes: null, logged_at: new Date().toISOString() }
    ]

    const filtered = store.getDosesByProtocol('p1')

    expect(filtered).toHaveLength(2)
    expect(filtered.every(d => d.protocol_id === 'p1')).toBe(true)
  })

  it('handles fetch errors', async () => {
    const store = useDoseStore()

    vi.mocked(api.listDoseLogs).mockRejectedValue(new Error('Network error'))

    await expect(store.fetchDoses()).rejects.toThrow()
    expect(showErrorToast).toHaveBeenCalled()
  })
})
