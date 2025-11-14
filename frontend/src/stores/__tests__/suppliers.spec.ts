import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSupplierStore } from '../suppliers'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('Supplier Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('initializes with empty state', () => {
    const store = useSupplierStore()

    expect(store.suppliers).toEqual([])
    expect(store.loading).toBe(false)
  })

  it('fetches suppliers from API', async () => {
    const store = useSupplierStore()
    const mockSuppliers = [
      {
        id: '1',
        name: 'PeptideSource',
        contact_email: 'contact@peptidesource.com',
        contact_phone: null,
        website: 'https://peptidesource.com',
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listSuppliers).mockResolvedValue(mockSuppliers)

    await store.fetchSuppliers()

    expect(api.listSuppliers).toHaveBeenCalled()
    expect(store.suppliers).toEqual(mockSuppliers)
  })

  it('creates new supplier', async () => {
    const store = useSupplierStore()
    const newSupplier = {
      id: '1',
      name: 'New Supplier',
      contact_email: null,
      contact_phone: null,
      website: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.saveSupplier).mockResolvedValue(newSupplier)

    await store.createSupplier({ name: 'New Supplier' })

    expect(store.suppliers).toContain(newSupplier)
    expect(showSuccessToast).toHaveBeenCalled()
  })

  it('updates existing supplier', async () => {
    const store = useSupplierStore()
    const original = {
      id: '1',
      name: 'Original',
      contact_email: null,
      contact_phone: null,
      website: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    store.suppliers = [original]

    vi.mocked(api.saveSupplier).mockResolvedValue({
      ...original,
      name: 'Updated'
    })

    await store.updateSupplier('1', { name: 'Updated' })

    expect(store.suppliers[0]?.name).toBe('Updated')
  })

  it('deletes supplier', async () => {
    const store = useSupplierStore()
    store.suppliers = [
      {
        id: '1',
        name: 'To Delete',
        contact_email: null,
        contact_phone: null,
        website: null,
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.deleteSupplier).mockResolvedValue(undefined)

    await store.deleteSupplier('1')

    expect(store.suppliers).toHaveLength(0)
  })

  it('finds supplier by ID', () => {
    const store = useSupplierStore()
    const target = {
      id: '2',
      name: 'Target',
      contact_email: null,
      contact_phone: null,
      website: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    store.suppliers = [
      {
        id: '1',
        name: 'Other',
        contact_email: null,
        contact_phone: null,
        website: null,
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      target
    ]

    const found = store.getSupplierById('2')

    expect(found).toEqual(target)
  })

  it('handles errors gracefully', async () => {
    const store = useSupplierStore()

    vi.mocked(api.listSuppliers).mockRejectedValue(new Error('Network error'))

    await expect(store.fetchSuppliers()).rejects.toThrow()
    expect(showErrorToast).toHaveBeenCalled()
  })
})
