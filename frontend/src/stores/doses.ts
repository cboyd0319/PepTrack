/**
 * Dose Store
 * State management for dose logging and tracking
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DoseLog } from '../api/peptrack'
import { listDoseLogs, createDoseLog, deleteDoseLog } from '../api/peptrack'
import { showErrorToast, showSuccessToast } from '../utils/errorHandling'

export const useDoseStore = defineStore('doses', () => {
  // State
  const doses = ref<DoseLog[]>([])
  const loading = ref(false)
  const dosesByProtocol = ref<Map<string, DoseLog[]>>(new Map())

  // Getters
  const doseCount = computed(() => doses.value.length)

  const recentDoses = computed(() =>
    [...doses.value]
      .sort((a, b) => new Date(b.loggedAt).getTime() - new Date(a.loggedAt).getTime())
      .slice(0, 10)
  )

  const dosesThisWeek = computed(() => {
    const weekAgo = new Date()
    weekAgo.setDate(weekAgo.getDate() - 7)
    return doses.value.filter(d => new Date(d.loggedAt) >= weekAgo)
  })

  const dosesThisMonth = computed(() => {
    const monthAgo = new Date()
    monthAgo.setMonth(monthAgo.getMonth() - 1)
    return doses.value.filter(d => new Date(d.loggedAt) >= monthAgo)
  })

  // Actions
  async function fetchDoses(protocolId?: string) {
    loading.value = true
    try {
      const data = protocolId
        ? await listDoseLogs(protocolId)
        : []

      if (protocolId) {
        dosesByProtocol.value.set(protocolId, data)
      } else {
        doses.value = data
      }

      return data
    } catch (error) {
      showErrorToast(error, { operation: 'load dose logs' })
      throw error
    } finally {
      loading.value = false
    }
  }

  async function logDose(protocolId: string, dose: { amount: number; unit: string; notes?: string }) {
    loading.value = true
    try {
      const newDose = await createDoseLog(protocolId, dose)

      // Update cache
      const protocolDoses = dosesByProtocol.value.get(protocolId) || []
      dosesByProtocol.value.set(protocolId, [newDose, ...protocolDoses])
      doses.value.unshift(newDose)

      showSuccessToast('Dose logged successfully')
      return newDose
    } catch (error) {
      showErrorToast(error, { operation: 'log dose' })
      throw error
    } finally {
      loading.value = false
    }
  }

  async function removeDose(protocolId: string, doseId: string) {
    loading.value = true
    try {
      await deleteDoseLog(protocolId, doseId)

      // Update cache
      const protocolDoses = dosesByProtocol.value.get(protocolId) || []
      dosesByProtocol.value.set(
        protocolId,
        protocolDoses.filter(d => d.id !== doseId)
      )
      doses.value = doses.value.filter(d => d.id !== doseId)

      showSuccessToast('Dose deleted successfully')
    } catch (error) {
      showErrorToast(error, { operation: 'delete dose' })
      throw error
    } finally {
      loading.value = false
    }
  }

  function getDosesForProtocol(protocolId: string) {
    return dosesByProtocol.value.get(protocolId) || []
  }

  function clearCache() {
    doses.value = []
    dosesByProtocol.value.clear()
  }

  return {
    // State
    doses,
    loading,

    // Getters
    doseCount,
    recentDoses,
    dosesThisWeek,
    dosesThisMonth,

    // Actions
    fetchDoses,
    logDose,
    removeDose,
    getDosesForProtocol,
    clearCache
  }
})
