/**
 * Dose Store
 * State management for dose logging and tracking
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'
import type { DoseLog, LogDosePayload } from '../api/peptrack'
import { listDoseLogs, listDoseLogsForProtocol, logDose as apiLogDose, deleteDoseLog } from '../api/peptrack'
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
      .sort((a, b) => new Date(b.logged_at).getTime() - new Date(a.logged_at).getTime())
      .slice(0, 10)
  )

  const dosesThisWeek = computed(() => {
    const weekAgo = new Date()
    weekAgo.setDate(weekAgo.getDate() - 7)
    return doses.value.filter(d => new Date(d.logged_at) >= weekAgo)
  })

  const dosesThisMonth = computed(() => {
    const monthAgo = new Date()
    monthAgo.setMonth(monthAgo.getMonth() - 1)
    return doses.value.filter(d => new Date(d.logged_at) >= monthAgo)
  })

  // Actions
  async function fetchDoses(protocolId?: string) {
    loading.value = true
    try {
      const data = protocolId
        ? await listDoseLogsForProtocol(protocolId)
        : await listDoseLogs()

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

  async function logDose(protocolId: string, site: string, amountMg: number, notes?: string) {
    loading.value = true
    try {
      const payload: LogDosePayload = {
        protocolId,
        site,
        amountMg,
        notes
      }
      const newDose = await apiLogDose(payload)

      // Update cache
      const protocolDoses = dosesByProtocol.value.get(protocolId) || []
      dosesByProtocol.value.set(protocolId, [newDose, ...protocolDoses])
      doses.value.unshift(newDose)

      showSuccessToast('Dose Logged', 'Dose logged successfully')
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
      await deleteDoseLog(doseId)

      // Update cache
      const protocolDoses = dosesByProtocol.value.get(protocolId) || []
      dosesByProtocol.value.set(
        protocolId,
        protocolDoses.filter(d => d.id !== doseId)
      )
      doses.value = doses.value.filter(d => d.id !== doseId)

      showSuccessToast('Dose Deleted', 'Dose deleted successfully')
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
