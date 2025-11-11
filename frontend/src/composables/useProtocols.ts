/**
 * Protocol Composable
 * Convenient wrapper around protocol store with additional utilities
 */

import { computed } from 'vue'
import { useProtocolStore } from '../stores'
import { storeToRefs } from 'pinia'

export function useProtocols() {
  const store = useProtocolStore()

  // Reactive refs from store
  const {
    protocols,
    loading,
    protocolCount,
    activeProtocols,
    archivedProtocols,
    protocolsByPeptide
  } = storeToRefs(store)

  // Computed helpers
  const hasProtocols = computed(() => protocols.value.length > 0)
  const isEmpty = computed(() => !loading.value && protocols.value.length === 0)

  // Actions (directly from store)
  const {
    fetchProtocols,
    createProtocol,
    updateProtocol,
    removeProtocol,
    getProtocolById,
    searchProtocols
  } = store

  // Helper functions
  async function refreshProtocols(force = false) {
    await fetchProtocols(force)
  }

  async function addProtocol(name: string, peptideName: string, notes?: string, targetConcentrationMgMl?: number) {
    return await createProtocol({
      name,
      peptideName,
      notes,
      targetConcentrationMgMl
    })
  }

  return {
    // State
    protocols,
    loading,
    hasProtocols,
    isEmpty,

    // Getters
    protocolCount,
    activeProtocols,
    archivedProtocols,
    protocolsByPeptide,

    // Actions
    refreshProtocols,
    fetchProtocols,
    addProtocol,
    createProtocol,
    updateProtocol,
    removeProtocol,
    getProtocolById,
    searchProtocols
  }
}
