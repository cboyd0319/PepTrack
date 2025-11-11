/**
 * UI Store
 * Global UI state management including loading states, modals, and notifications
 */

import { defineStore } from 'pinia'
import { ref, computed } from 'vue'

export interface LoadingState {
  isLoading: boolean
  message?: string
}

export const useUIStore = defineStore('ui', () => {
  // State
  const globalLoading = ref(false)
  const loadingMessage = ref<string | null>(null)
  const loadingOperations = ref<Map<string, LoadingState>>(new Map())

  // Modal state
  const activeModal = ref<string | null>(null)
  const modalData = ref<any>(null)

  // Sidebar/navigation state
  const sidebarCollapsed = ref(false)
  const currentView = ref<string>('protocols')

  // Network state
  const isOnline = ref(navigator.onLine)

  // Getters
  const hasActiveLoading = computed(() =>
    Array.from(loadingOperations.value.values()).some(op => op.isLoading)
  )

  const isModalOpen = computed(() => activeModal.value !== null)

  // Actions
  function startLoading(operation: string, message?: string) {
    loadingOperations.value.set(operation, { isLoading: true, message })
    updateGlobalLoading()
  }

  function stopLoading(operation: string) {
    loadingOperations.value.delete(operation)
    updateGlobalLoading()
  }

  function updateGlobalLoading() {
    if (hasActiveLoading.value) {
      globalLoading.value = true
      // Get first loading message if any
      const firstLoading = Array.from(loadingOperations.value.values())
        .find(op => op.message)
      loadingMessage.value = firstLoading?.message || null
    } else {
      globalLoading.value = false
      loadingMessage.value = null
    }
  }

  function openModal(modalId: string, data?: any) {
    activeModal.value = modalId
    modalData.value = data
  }

  function closeModal() {
    activeModal.value = null
    modalData.value = null
  }

  function toggleSidebar() {
    sidebarCollapsed.value = !sidebarCollapsed.value
  }

  function setView(view: string) {
    currentView.value = view
  }

  function setOnlineStatus(online: boolean) {
    isOnline.value = online
  }

  // Initialize online status listener
  if (typeof window !== 'undefined') {
    window.addEventListener('online', () => setOnlineStatus(true))
    window.addEventListener('offline', () => setOnlineStatus(false))
  }

  return {
    // State
    globalLoading,
    loadingMessage,
    activeModal,
    modalData,
    sidebarCollapsed,
    currentView,
    isOnline,

    // Getters
    hasActiveLoading,
    isModalOpen,

    // Actions
    startLoading,
    stopLoading,
    openModal,
    closeModal,
    toggleSidebar,
    setView,
    setOnlineStatus
  }
})
