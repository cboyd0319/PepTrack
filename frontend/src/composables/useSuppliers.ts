/**
 * Suppliers Composable
 * Convenient wrapper around supplier store
 */

import { computed } from 'vue'
import { useSupplierStore } from '../stores'
import { storeToRefs } from 'pinia'

export function useSuppliers() {
  const store = useSupplierStore()

  const {
    suppliers,
    inventory,
    loadingSuppliers,
    loadingInventory,
    supplierCount,
    inventoryCount,
    activeInventory,
    expiredInventory,
    expiringSoonInventory
  } = storeToRefs(store)

  const hasSuppliers = computed(() => suppliers.value.length > 0)
  const hasInventory = computed(() => inventory.value.length > 0)
  const hasExpiredItems = computed(() => expiredInventory.value.length > 0)
  const hasExpiringSoon = computed(() => expiringSoonInventory.value.length > 0)

  const {
    fetchSuppliers,
    addSupplier,
    modifySupplier,
    removeSupplier,
    fetchInventory,
    addInventoryItem,
    modifyInventoryItem,
    removeInventoryItem
  } = store

  return {
    // State
    suppliers,
    inventory,
    loadingSuppliers,
    loadingInventory,
    hasSuppliers,
    hasInventory,
    hasExpiredItems,
    hasExpiringSoon,

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
}
