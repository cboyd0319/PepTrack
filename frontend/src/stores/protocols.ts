/**
 * Protocol Store
 * Centralized state management for peptide protocols with caching and optimistic updates
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { PeptideProtocol, CreateProtocolPayload } from '../api/peptrack'
import { listProtocols, saveProtocol } from '../api/peptrack'
import { showErrorToast, showSuccessToast } from '../utils/errorHandling'

export const useProtocolStore = defineStore('protocols', () => {
  // State
  const protocols = ref<PeptideProtocol[]>([])
  const loading = ref(false)
  const lastFetch = ref<number | null>(null)
  const cacheDuration = 30000 // 30 seconds

  // Getters (computed properties)
  const protocolCount = computed(() => protocols.value.length)

  const activeProtocols = computed(() => protocols.value)

  const archivedProtocols = computed(() => [] as PeptideProtocol[])

  const protocolsByPeptide = computed(() => {
    const map = new Map<string, PeptideProtocol[]>()
    protocols.value.forEach(protocol => {
      const peptide = protocol.peptide_name || 'Unknown'
      if (!map.has(peptide)) {
        map.set(peptide, [])
      }
      map.get(peptide)!.push(protocol)
    })
    return map
  })

  const isCacheValid = computed(() => {
    if (!lastFetch.value) return false
    return Date.now() - lastFetch.value < cacheDuration
  })

  // Actions
  async function fetchProtocols(force = false) {
    // Use cache if valid and not forcing refresh
    if (!force && isCacheValid.value && protocols.value.length > 0) {
      return protocols.value
    }

    loading.value = true
    try {
      const data = await listProtocols()
      protocols.value = data
      lastFetch.value = Date.now()
      return data
    } catch (error) {
      showErrorToast(error, { operation: 'load protocols' })
      throw error
    } finally {
      loading.value = false
    }
  }

  async function createProtocol(payload: CreateProtocolPayload) {
    loading.value = true
    try {
      const newProtocol = await saveProtocol(payload)

      // Optimistic update
      protocols.value.unshift(newProtocol)

      showSuccessToast('Protocol Created', 'Protocol created successfully')
      return newProtocol
    } catch (error) {
      showErrorToast(error, { operation: 'create protocol' })
      // Refresh to ensure consistency
      await fetchProtocols(true)
      throw error
    } finally {
      loading.value = false
    }
  }

  async function updateProtocol(id: string, payload: Partial<CreateProtocolPayload>) {
    loading.value = true
    const originalProtocols = [...protocols.value]

    try {
      // Optimistic update
      const index = protocols.value.findIndex(p => p.id === id)
      if (index !== -1) {
        const current = protocols.value[index]
        if (current) {
          protocols.value[index] = {
            ...current,
            name: payload.name || current.name,
            peptide_name: payload.peptideName || current.peptide_name,
            notes: payload.notes ?? current.notes,
            target_concentration_mg_ml: payload.targetConcentrationMgMl ?? current.target_concentration_mg_ml,
            updated_at: new Date().toISOString()
          }
        }
      }

      const updated = await saveProtocol({ ...payload, name: payload.name || '', peptideName: payload.peptideName || '' })

      showSuccessToast('Protocol Updated', 'Protocol updated successfully')
      return updated
    } catch (error) {
      // Rollback on error
      protocols.value = originalProtocols
      showErrorToast(error, { operation: 'update protocol' })
      throw error
    } finally {
      loading.value = false
    }
  }

  async function removeProtocol(_id: string) {
    // Note: No delete API available yet
    console.warn('Protocol deletion not yet implemented in backend')
    showErrorToast(new Error('Protocol deletion not yet available'), { operation: 'delete protocol' })
  }

  function getProtocolById(id: string) {
    return protocols.value.find(p => p.id === id)
  }

  function searchProtocols(query: string) {
    const lowerQuery = query.toLowerCase()
    return protocols.value.filter(p =>
      p.name?.toLowerCase().includes(lowerQuery) ||
      p.peptide_name?.toLowerCase().includes(lowerQuery) ||
      p.notes?.toLowerCase().includes(lowerQuery)
    )
  }

  function clearCache() {
    protocols.value = []
    lastFetch.value = null
  }

  return {
    // State
    protocols,
    loading,

    // Getters
    protocolCount,
    activeProtocols,
    archivedProtocols,
    protocolsByPeptide,
    isCacheValid,

    // Actions
    fetchProtocols,
    createProtocol,
    updateProtocol,
    removeProtocol,
    getProtocolById,
    searchProtocols,
    clearCache
  }
})
