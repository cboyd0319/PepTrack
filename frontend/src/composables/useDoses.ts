/**
 * Doses Composable
 * Convenient wrapper around dose store
 */

import { computed } from 'vue'
import { useDoseStore } from '../stores'
import { storeToRefs } from 'pinia'

export function useDoses() {
  const store = useDoseStore()

  const {
    doses,
    loading,
    doseCount,
    recentDoses,
    dosesThisWeek,
    dosesThisMonth
  } = storeToRefs(store)

  const hasDoses = computed(() => doses.value.length > 0)

  const {
    fetchDoses,
    logDose,
    removeDose,
    getDosesForProtocol
  } = store

  return {
    // State
    doses,
    loading,
    hasDoses,

    // Getters
    doseCount,
    recentDoses,
    dosesThisWeek,
    dosesThisMonth,

    // Actions
    fetchDoses,
    logDose,
    removeDose,
    getDosesForProtocol
  }
}
