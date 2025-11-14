import { describe, it, expect, beforeEach } from 'vitest'
import { setActivePinia, createPinia } from 'pinia'
import { useUiStore } from '../ui'

describe('UI Store', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
  })

  it('initializes with default state', () => {
    const store = useUiStore()

    expect(store.darkMode).toBe(false)
    expect(store.sidebarCollapsed).toBe(false)
    expect(store.currentView).toBe('dashboard')
  })

  it('toggles dark mode', () => {
    const store = useUiStore()

    expect(store.darkMode).toBe(false)

    store.toggleDarkMode()

    expect(store.darkMode).toBe(true)

    store.toggleDarkMode()

    expect(store.darkMode).toBe(false)
  })

  it('toggles sidebar', () => {
    const store = useUiStore()

    expect(store.sidebarCollapsed).toBe(false)

    store.toggleSidebar()

    expect(store.sidebarCollapsed).toBe(true)

    store.toggleSidebar()

    expect(store.sidebarCollapsed).toBe(false)
  })

  it('sets current view', () => {
    const store = useUiStore()

    store.setCurrentView('protocols')

    expect(store.currentView).toBe('protocols')

    store.setCurrentView('literature')

    expect(store.currentView).toBe('literature')
  })

  it('sets dark mode directly', () => {
    const store = useUiStore()

    store.setDarkMode(true)

    expect(store.darkMode).toBe(true)

    store.setDarkMode(false)

    expect(store.darkMode).toBe(false)
  })

  it('sets sidebar state directly', () => {
    const store = useUiStore()

    store.setSidebarCollapsed(true)

    expect(store.sidebarCollapsed).toBe(true)

    store.setSidebarCollapsed(false)

    expect(store.sidebarCollapsed).toBe(false)
  })
})
