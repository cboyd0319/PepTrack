import { describe, it, expect, vi, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useSuppliers } from '../useSuppliers'
import { useSupplierStore } from '../../stores/suppliers'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

describe('useSuppliers Composable', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  // =============================================================================
  // Reactive Refs Tests - Suppliers
  // =============================================================================

  it('exposes suppliers from store', () => {
    const { suppliers } = useSuppliers()
    const store = useSupplierStore()

    store.suppliers = [
      {
        id: '1',
        name: 'PeptideSource',
        contact_email: 'contact@example.com',
        contact_phone: null,
        website: null,
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    expect(suppliers.value).toHaveLength(1)
    expect(suppliers.value[0]?.name).toBe('PeptideSource')
  })

  it('exposes loadingSuppliers from store', () => {
    const { loadingSuppliers } = useSuppliers()
    const store = useSupplierStore()

    expect(loadingSuppliers.value).toBe(false)

    store.loadingSuppliers = true
    expect(loadingSuppliers.value).toBe(true)
  })

  it('exposes supplierCount from store', () => {
    const { supplierCount } = useSuppliers()
    const store = useSupplierStore()

    store.suppliers = [
      { id: '1', name: 'S1', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'S2', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(supplierCount.value).toBe(2)
  })

  // =============================================================================
  // Reactive Refs Tests - Inventory
  // =============================================================================

  it('exposes inventory from store', () => {
    const { inventory } = useSuppliers()
    const store = useSupplierStore()

    store.inventory = [
      {
        id: '1',
        protocol_id: 'p1',
        supplier_id: 's1',
        vial_status: 'sealed',
        amount_mg: 10.0,
        cost: null,
        received_date: null,
        expiry_date: null,
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    expect(inventory.value).toHaveLength(1)
    expect(inventory.value[0]?.amount_mg).toBe(10.0)
  })

  it('exposes loadingInventory from store', () => {
    const { loadingInventory } = useSuppliers()
    const store = useSupplierStore()

    expect(loadingInventory.value).toBe(false)

    store.loadingInventory = true
    expect(loadingInventory.value).toBe(true)
  })

  it('exposes inventoryCount from store', () => {
    const { inventoryCount } = useSuppliers()
    const store = useSupplierStore()

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', supplier_id: 's1', vial_status: 'opened', amount_mg: 5.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(inventoryCount.value).toBe(2)
  })

  it('exposes activeInventory from store', () => {
    const { activeInventory } = useSuppliers()
    const store = useSupplierStore()

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', supplier_id: 's1', vial_status: 'empty', amount_mg: 0.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '3', protocol_id: 'p1', supplier_id: 's1', vial_status: 'opened', amount_mg: 5.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(activeInventory.value).toHaveLength(2)
    expect(activeInventory.value.every(i => i.vial_status === 'sealed' || i.vial_status === 'opened')).toBe(true)
  })

  it('exposes expiredInventory from store', () => {
    const { expiredInventory } = useSuppliers()
    const store = useSupplierStore()

    const yesterday = new Date()
    yesterday.setDate(yesterday.getDate() - 1)

    const tomorrow = new Date()
    tomorrow.setDate(tomorrow.getDate() + 1)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: yesterday.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 5.0, cost: null, received_date: null, expiry_date: tomorrow.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(expiredInventory.value).toHaveLength(1)
    expect(expiredInventory.value[0]?.id).toBe('1')
  })

  it('exposes expiringSoonInventory from store', () => {
    const { expiringSoonInventory } = useSuppliers()
    const store = useSupplierStore()

    const fifteenDaysFromNow = new Date()
    fifteenDaysFromNow.setDate(fifteenDaysFromNow.getDate() + 15)

    const fortyDaysFromNow = new Date()
    fortyDaysFromNow.setDate(fortyDaysFromNow.getDate() + 40)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: fifteenDaysFromNow.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 5.0, cost: null, received_date: null, expiry_date: fortyDaysFromNow.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(expiringSoonInventory.value).toHaveLength(1)
    expect(expiringSoonInventory.value[0]?.id).toBe('1')
  })

  // =============================================================================
  // Computed Helpers Tests
  // =============================================================================

  it('computes hasSuppliers correctly', () => {
    const { hasSuppliers } = useSuppliers()
    const store = useSupplierStore()

    expect(hasSuppliers.value).toBe(false)

    store.suppliers = [
      { id: '1', name: 'Test', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasSuppliers.value).toBe(true)
  })

  it('computes hasInventory correctly', () => {
    const { hasInventory } = useSuppliers()
    const store = useSupplierStore()

    expect(hasInventory.value).toBe(false)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasInventory.value).toBe(true)
  })

  it('computes hasExpiredItems correctly', () => {
    const { hasExpiredItems } = useSuppliers()
    const store = useSupplierStore()

    expect(hasExpiredItems.value).toBe(false)

    const yesterday = new Date()
    yesterday.setDate(yesterday.getDate() - 1)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: yesterday.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasExpiredItems.value).toBe(true)
  })

  it('computes hasExpiringSoon correctly', () => {
    const { hasExpiringSoon } = useSuppliers()
    const store = useSupplierStore()

    expect(hasExpiringSoon.value).toBe(false)

    const fifteenDaysFromNow = new Date()
    fifteenDaysFromNow.setDate(fifteenDaysFromNow.getDate() + 15)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: fifteenDaysFromNow.toISOString().split('T')[0], notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasExpiringSoon.value).toBe(true)
  })

  // =============================================================================
  // Supplier Action Methods Tests
  // =============================================================================

  it('fetchSuppliers calls API', async () => {
    const { fetchSuppliers } = useSuppliers()

    const mockSuppliers = [
      { id: '1', name: 'Test', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.listSuppliers).mockResolvedValue(mockSuppliers)

    await fetchSuppliers()

    expect(api.listSuppliers).toHaveBeenCalled()
  })

  it('addSupplier creates supplier', async () => {
    const { addSupplier } = useSuppliers()

    const newSupplier = {
      id: '1',
      name: 'New Supplier',
      contact_email: 'test@example.com',
      contact_phone: null,
      website: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.createSupplier).mockResolvedValue(newSupplier)

    await addSupplier({ name: 'New Supplier', contactEmail: 'test@example.com' })

    expect(api.createSupplier).toHaveBeenCalled()
  })

  it('modifySupplier updates supplier', async () => {
    const { modifySupplier } = useSuppliers()
    const store = useSupplierStore()

    store.suppliers = [
      { id: '1', name: 'Original', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.updateSupplier).mockResolvedValue({
      id: '1',
      name: 'Updated',
      contact_email: null,
      contact_phone: null,
      website: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    })

    await modifySupplier('1', { name: 'Updated' })

    expect(store.suppliers[0]?.name).toBe('Updated')
  })

  it('removeSupplier deletes supplier', async () => {
    const { removeSupplier } = useSuppliers()
    const store = useSupplierStore()

    store.suppliers = [
      { id: '1', name: 'To Delete', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.deleteSupplier).mockResolvedValue(undefined)

    await removeSupplier('1')

    expect(store.suppliers).toHaveLength(0)
  })

  // =============================================================================
  // Inventory Action Methods Tests
  // =============================================================================

  it('fetchInventory calls API', async () => {
    const { fetchInventory } = useSuppliers()

    const mockInventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.listInventory).mockResolvedValue(mockInventory)

    await fetchInventory()

    expect(api.listInventory).toHaveBeenCalled()
  })

  it('addInventoryItem creates item', async () => {
    const { addInventoryItem } = useSuppliers()

    const newItem = {
      id: '1',
      protocol_id: 'p1',
      supplier_id: 's1',
      vial_status: 'sealed',
      amount_mg: 10.0,
      cost: null,
      received_date: null,
      expiry_date: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }

    vi.mocked(api.createInventoryItem).mockResolvedValue(newItem)

    await addInventoryItem({ protocolId: 'p1', supplierId: 's1', amountMg: 10.0 })

    expect(api.createInventoryItem).toHaveBeenCalled()
  })

  it('modifyInventoryItem updates item', async () => {
    const { modifyInventoryItem } = useSuppliers()
    const store = useSupplierStore()

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.updateInventoryItem).mockResolvedValue({
      id: '1',
      protocol_id: 'p1',
      supplier_id: 's1',
      vial_status: 'opened',
      amount_mg: 10.0,
      cost: null,
      received_date: null,
      expiry_date: null,
      notes: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    })

    await modifyInventoryItem('1', { vialStatus: 'opened' })

    expect(store.inventory[0]?.vial_status).toBe('opened')
  })

  it('removeInventoryItem deletes item', async () => {
    const { removeInventoryItem } = useSuppliers()
    const store = useSupplierStore()

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    vi.mocked(api.deleteInventoryItem).mockResolvedValue(undefined)

    await removeInventoryItem('1')

    expect(store.inventory).toHaveLength(0)
  })

  // =============================================================================
  // Reactivity Tests
  // =============================================================================

  it('suppliers ref updates when store changes', () => {
    const { suppliers } = useSuppliers()
    const store = useSupplierStore()

    expect(suppliers.value).toHaveLength(0)

    store.suppliers = [
      { id: '1', name: 'New', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(suppliers.value).toHaveLength(1)
  })

  it('inventory ref updates when store changes', () => {
    const { inventory } = useSuppliers()
    const store = useSupplierStore()

    expect(inventory.value).toHaveLength(0)

    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(inventory.value).toHaveLength(1)
  })

  it('computed properties are reactive', () => {
    const { hasSuppliers, hasInventory } = useSuppliers()
    const store = useSupplierStore()

    expect(hasSuppliers.value).toBe(false)
    expect(hasInventory.value).toBe(false)

    store.suppliers = [
      { id: '1', name: 'Test', contact_email: null, contact_phone: null, website: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]
    store.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(hasSuppliers.value).toBe(true)
    expect(hasInventory.value).toBe(true)
  })
})
