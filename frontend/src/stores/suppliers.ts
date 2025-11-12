/**
 * Supplier Store
 * State management for suppliers and inventory
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { Supplier, InventoryItem } from '../api/peptrack'
import {
  listSuppliers,
  createSupplier,
  updateSupplier,
  deleteSupplier,
  listInventory,
  createInventoryItem,
  updateInventoryItem,
  deleteInventoryItem
} from '../api/peptrack'
import { showErrorToast, showSuccessToast } from '../utils/errorHandling'

export const useSupplierStore = defineStore('suppliers', () => {
  // State
  const suppliers = ref<Supplier[]>([])
  const inventory = ref<InventoryItem[]>([])
  const loadingSuppliers = ref(false)
  const loadingInventory = ref(false)

  // Getters
  const supplierCount = computed(() => suppliers.value.length)
  const inventoryCount = computed(() => inventory.value.length)

  const activeInventory = computed(() =>
    inventory.value.filter(item => item.vial_status === 'sealed' || item.vial_status === 'opened')
  )

  const expiredInventory = computed(() => {
    const now = new Date()
    return inventory.value.filter(item => {
      if (!item.expiry_date) return false
      return new Date(item.expiry_date) < now
    })
  })

  const expiringSoonInventory = computed(() => {
    const now = new Date()
    const thirtyDaysFromNow = new Date(now.getTime() + 30 * 24 * 60 * 60 * 1000)
    return inventory.value.filter(item => {
      if (!item.expiry_date) return false
      const expiry = new Date(item.expiry_date)
      return expiry > now && expiry <= thirtyDaysFromNow
    })
  })

  // Supplier Actions
  async function fetchSuppliers() {
    loadingSuppliers.value = true
    try {
      suppliers.value = await listSuppliers()
      return suppliers.value
    } catch (error) {
      showErrorToast(error, { operation: 'load suppliers' })
      throw error
    } finally {
      loadingSuppliers.value = false
    }
  }

  async function addSupplier(payload: any) {
    loadingSuppliers.value = true
    try {
      const newSupplier = await createSupplier(payload)
      suppliers.value.push(newSupplier)
      showSuccessToast('Supplier Added', 'Supplier added successfully')
      return newSupplier
    } catch (error) {
      showErrorToast(error, { operation: 'create supplier' })
      throw error
    } finally {
      loadingSuppliers.value = false
    }
  }

  async function modifySupplier(id: string, payload: any) {
    loadingSuppliers.value = true
    try {
      const updated = await updateSupplier(id, payload)
      const index = suppliers.value.findIndex(s => s.id === id)
      if (index !== -1) {
        suppliers.value[index] = updated
      }
      showSuccessToast('Supplier Updated', 'Supplier updated successfully')
      return updated
    } catch (error) {
      showErrorToast(error, { operation: 'update supplier' })
      throw error
    } finally {
      loadingSuppliers.value = false
    }
  }

  async function removeSupplier(id: string) {
    loadingSuppliers.value = true
    try {
      await deleteSupplier(id)
      suppliers.value = suppliers.value.filter(s => s.id !== id)
      showSuccessToast('Supplier Deleted', 'Supplier deleted successfully')
    } catch (error) {
      showErrorToast(error, { operation: 'delete supplier' })
      throw error
    } finally {
      loadingSuppliers.value = false
    }
  }

  // Inventory Actions
  async function fetchInventory() {
    loadingInventory.value = true
    try {
      inventory.value = await listInventory()
      return inventory.value
    } catch (error) {
      showErrorToast(error, { operation: 'load inventory' })
      throw error
    } finally {
      loadingInventory.value = false
    }
  }

  async function addInventoryItem(payload: any) {
    loadingInventory.value = true
    try {
      const newItem = await createInventoryItem(payload)
      inventory.value.push(newItem)
      showSuccessToast('Inventory Added', 'Inventory item added successfully')
      return newItem
    } catch (error) {
      showErrorToast(error, { operation: 'create inventory item' })
      throw error
    } finally {
      loadingInventory.value = false
    }
  }

  async function modifyInventoryItem(id: string, payload: any) {
    loadingInventory.value = true
    try {
      const updated = await updateInventoryItem(id, payload)
      const index = inventory.value.findIndex(i => i.id === id)
      if (index !== -1) {
        inventory.value[index] = updated
      }
      showSuccessToast('Inventory Updated', 'Inventory updated successfully')
      return updated
    } catch (error) {
      showErrorToast(error, { operation: 'update inventory' })
      throw error
    } finally {
      loadingInventory.value = false
    }
  }

  async function removeInventoryItem(id: string) {
    loadingInventory.value = true
    try {
      await deleteInventoryItem(id)
      inventory.value = inventory.value.filter(i => i.id !== id)
      showSuccessToast('Inventory Deleted', 'Inventory item deleted successfully')
    } catch (error) {
      showErrorToast(error, { operation: 'delete inventory item' })
      throw error
    } finally {
      loadingInventory.value = false
    }
  }

  return {
    // State
    suppliers,
    inventory,
    loadingSuppliers,
    loadingInventory,

    // Getters
    supplierCount,
    inventoryCount,
    activeInventory,
    expiredInventory,
    expiringSoonInventory,

    // Actions
    fetchSuppliers,
    addSupplier,
    modifySupplier,
    removeSupplier,
    fetchInventory,
    addInventoryItem,
    modifyInventoryItem,
    removeInventoryItem
  }
})
