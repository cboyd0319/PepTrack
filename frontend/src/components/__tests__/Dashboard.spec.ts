import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import { setActivePinia, createPinia } from 'pinia'
import Dashboard from '../Dashboard.vue'
import { useProtocolStore } from '../../stores/protocols'
import { useDoseStore } from '../../stores/doses'
import { useSupplierStore } from '../../stores/suppliers'
import * as api from '../../api/peptrack'

// Mock all child components
vi.mock('../DoseCalendarHeatmap.vue', () => ({
  default: { name: 'DoseCalendarHeatmap', template: '<div class="dose-calendar-heatmap-stub"></div>' }
}))
vi.mock('../ProtocolProgressTracker.vue', () => ({
  default: { name: 'ProtocolProgressTracker', template: '<div class="protocol-progress-tracker-stub"></div>' }
}))
vi.mock('../CostAnalysisDashboard.vue', () => ({
  default: { name: 'CostAnalysisDashboard', template: '<div class="cost-analysis-dashboard-stub"></div>' }
}))
vi.mock('../RecentActivityTimeline.vue', () => ({
  default: { name: 'RecentActivityTimeline', template: '<div class="recent-activity-timeline-stub"></div>' }
}))

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showSuccessToast, showErrorToast } from '../../utils/errorHandling'

describe('Dashboard Component', () => {
  let wrapper: VueWrapper<any>

  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()

    // Mock API responses
    vi.mocked(api.listProtocols).mockResolvedValue([])
    vi.mocked(api.listDoseLogs).mockResolvedValue([])
    vi.mocked(api.listInventory).mockResolvedValue([])
    vi.mocked(api.listAlerts).mockResolvedValue([])
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  // =============================================================================
  // Component Mounting & Data Loading Tests
  // =============================================================================

  it('renders dashboard header', () => {
    wrapper = mount(Dashboard)

    expect(wrapper.find('h1').text()).toContain('Dashboard')
    expect(wrapper.find('.subtitle').text()).toContain('Welcome back')
  })

  it('loads data on mount', async () => {
    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listProtocols).toHaveBeenCalled()
    expect(api.listDoseLogs).toHaveBeenCalled()
    expect(api.listInventory).toHaveBeenCalled()
    expect(api.listAlerts).toHaveBeenCalledWith(false) // Don't include dismissed
  })

  it('loads alerts on mount', async () => {
    const mockAlerts = [
      {
        id: '1',
        alert_type: 'expiring_soon',
        severity: 'warning',
        title: 'Vial Expiring Soon',
        message: 'Vial #123 expires in 5 days',
        is_dismissed: false,
        is_read: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listAlerts).mockResolvedValue(mockAlerts)

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.alerts-widget').exists()).toBe(true)
    expect(wrapper.find('.alert-count').text()).toBe('1')
  })

  it('handles data loading errors gracefully', async () => {
    vi.mocked(api.listProtocols).mockRejectedValue(new Error('Network error'))

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    // Component should still render
    expect(wrapper.find('.dashboard').exists()).toBe(true)
  })

  // =============================================================================
  // Quick Actions Tests
  // =============================================================================

  it('displays quick action buttons', () => {
    wrapper = mount(Dashboard)

    const actionButtons = wrapper.findAll('.action-btn')
    expect(actionButtons).toHaveLength(4)

    expect(actionButtons[0]?.text()).toContain('Log Dose')
    expect(actionButtons[1]?.text()).toContain('New Protocol')
    expect(actionButtons[2]?.text()).toContain('Research')
    expect(actionButtons[3]?.text()).toContain('Quick Backup')
  })

  it('emits quickLogDose when Log Dose button clicked', async () => {
    wrapper = mount(Dashboard)

    const logDoseButton = wrapper.findAll('.action-btn')[0]
    await logDoseButton?.trigger('click')

    expect(wrapper.emitted('quickLogDose')).toBeTruthy()
    expect(wrapper.emitted('quickLogDose')).toHaveLength(1)
  })

  it('emits navigateToTab when New Protocol button clicked', async () => {
    wrapper = mount(Dashboard)

    const newProtocolButton = wrapper.findAll('.action-btn')[1]
    await newProtocolButton?.trigger('click')

    expect(wrapper.emitted('navigateToTab')).toBeTruthy()
    expect(wrapper.emitted('navigateToTab')?.[0]).toEqual(['protocols'])
  })

  it('emits navigateToTab when Research button clicked', async () => {
    wrapper = mount(Dashboard)

    const researchButton = wrapper.findAll('.action-btn')[2]
    await researchButton?.trigger('click')

    expect(wrapper.emitted('navigateToTab')).toBeTruthy()
    expect(wrapper.emitted('navigateToTab')?.[0]).toEqual(['research'])
  })

  it('emits quickBackup when Quick Backup button clicked', async () => {
    wrapper = mount(Dashboard)

    const backupButton = wrapper.findAll('.action-btn')[3]
    await backupButton?.trigger('click')

    expect(wrapper.emitted('quickBackup')).toBeTruthy()
    expect(wrapper.emitted('quickBackup')).toHaveLength(1)
  })

  // =============================================================================
  // Stats Grid Tests
  // =============================================================================

  it('displays stats grid with correct initial values', () => {
    wrapper = mount(Dashboard)

    const statCards = wrapper.findAll('.stat-card')
    expect(statCards).toHaveLength(4)

    expect(statCards[0]?.text()).toContain('Active Protocols')
    expect(statCards[0]?.text()).toContain('0')

    expect(statCards[1]?.text()).toContain('Doses This Week')
    expect(statCards[1]?.text()).toContain('0')

    expect(statCards[2]?.text()).toContain('Inventory Items')
    expect(statCards[2]?.text()).toContain('0')

    expect(statCards[3]?.text()).toContain('Expiring Soon')
    expect(statCards[3]?.text()).toContain('0')
  })

  it('updates stats when store data changes', async () => {
    wrapper = mount(Dashboard)

    const protocolStore = useProtocolStore()
    const doseStore = useDoseStore()
    const supplierStore = useSupplierStore()

    protocolStore.protocols = [
      { id: '1', name: 'Test', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    const now = new Date()
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)
    doseStore.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: yesterday.toISOString() }
    ]

    supplierStore.inventory = [
      { id: '1', protocol_id: 'p1', supplier_id: 's1', vial_status: 'sealed', amount_mg: 10.0, cost: null, received_date: null, expiry_date: null, notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    await wrapper.vm.$nextTick()

    const statCards = wrapper.findAll('.stat-card')
    expect(statCards[0]?.text()).toContain('1') // Protocols
    expect(statCards[1]?.text()).toContain('1') // Doses this week
    expect(statCards[2]?.text()).toContain('1') // Inventory
  })

  it('applies warning class to expiring soon card when count > 0', async () => {
    wrapper = mount(Dashboard)

    const supplierStore = useSupplierStore()

    const fifteenDaysFromNow = new Date()
    fifteenDaysFromNow.setDate(fifteenDaysFromNow.getDate() + 15)

    supplierStore.inventory = [
      {
        id: '1',
        protocol_id: 'p1',
        supplier_id: 's1',
        vial_status: 'sealed',
        amount_mg: 10.0,
        cost: null,
        received_date: null,
        expiry_date: fifteenDaysFromNow.toISOString().split('T')[0],
        notes: null,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    await wrapper.vm.$nextTick()

    const expiringSoonCard = wrapper.findAll('.stat-card')[3]
    expect(expiringSoonCard?.classes()).toContain('warning')
  })

  // =============================================================================
  // Alerts Widget Tests
  // =============================================================================

  it('hides alerts widget when no active alerts', async () => {
    vi.mocked(api.listAlerts).mockResolvedValue([])

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.alerts-widget').exists()).toBe(false)
  })

  it('displays alerts widget when active alerts exist', async () => {
    const mockAlerts = [
      {
        id: '1',
        alert_type: 'expiring_soon',
        severity: 'warning',
        title: 'Alert 1',
        message: 'Message 1',
        is_dismissed: false,
        is_read: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listAlerts).mockResolvedValue(mockAlerts)

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.alerts-widget').exists()).toBe(true)
    expect(wrapper.find('.alert-preview-title').text()).toBe('Alert 1')
  })

  it('limits alerts preview to 3 items', async () => {
    const mockAlerts = Array.from({ length: 10 }, (_, i) => ({
      id: `${i + 1}`,
      alert_type: 'expiring_soon',
      severity: 'warning',
      title: `Alert ${i + 1}`,
      message: `Message ${i + 1}`,
      is_dismissed: false,
      is_read: false,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }))

    vi.mocked(api.listAlerts).mockResolvedValue(mockAlerts as any)

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.findAll('.alert-preview-card')).toHaveLength(3)
  })

  it('emits viewAlerts when View All button clicked', async () => {
    const mockAlerts = [
      {
        id: '1',
        alert_type: 'expiring_soon',
        severity: 'warning',
        title: 'Alert 1',
        message: 'Message 1',
        is_dismissed: false,
        is_read: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listAlerts).mockResolvedValue(mockAlerts)

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const viewAllButton = wrapper.find('.widget-header .link-btn')
    await viewAllButton.trigger('click')

    expect(wrapper.emitted('viewAlerts')).toBeTruthy()
  })

  it('dismisses alert when dismiss button clicked', async () => {
    const mockAlerts = [
      {
        id: '1',
        alert_type: 'expiring_soon',
        severity: 'warning',
        title: 'Alert 1',
        message: 'Message 1',
        is_dismissed: false,
        is_read: false,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.listAlerts).mockResolvedValue(mockAlerts)
    vi.mocked(api.dismissAlert).mockResolvedValue(undefined)

    wrapper = mount(Dashboard)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const dismissButton = wrapper.find('.dismiss-btn')
    await dismissButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.dismissAlert).toHaveBeenCalledWith('1')
    expect(showSuccessToast).toHaveBeenCalledWith('Dismissed', 'Alert dismissed')
  })

  // =============================================================================
  // Recent Doses Card Tests
  // =============================================================================

  it('shows empty state when no doses', () => {
    wrapper = mount(Dashboard)

    const recentDosesCard = wrapper.findAll('.card')[0]
    expect(recentDosesCard?.text()).toContain('No doses logged yet')
    expect(recentDosesCard?.find('.btn-secondary').exists()).toBe(true)
  })

  it('displays recent doses when available', async () => {
    wrapper = mount(Dashboard)

    const protocolStore = useProtocolStore()
    const doseStore = useDoseStore()

    protocolStore.protocols = [
      { id: 'p1', name: 'BPC-157 Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    doseStore.doses = [
      { id: '1', protocol_id: 'p1', injection_site: 'deltoid', amount_mg: 2.5, notes: null, logged_at: new Date().toISOString() }
    ]

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.dose-protocol').text()).toContain('BPC-157 Protocol')
    expect(wrapper.find('.dose-amount').text()).toBe('2.5mg')
    expect(wrapper.find('.dose-site').text()).toBe('deltoid')
  })

  it('limits recent doses to 5 items', async () => {
    wrapper = mount(Dashboard)

    const doseStore = useDoseStore()

    doseStore.doses = Array.from({ length: 10 }, (_, i) => ({
      id: `${i + 1}`,
      protocol_id: 'p1',
      injection_site: 'deltoid',
      amount_mg: 2.5,
      notes: null,
      logged_at: new Date().toISOString()
    }))

    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.dose-item')).toHaveLength(5)
  })

  // =============================================================================
  // Active Protocols Card Tests
  // =============================================================================

  it('shows empty state when no protocols', () => {
    wrapper = mount(Dashboard)

    const protocolsCard = wrapper.findAll('.card')[1]
    expect(protocolsCard?.text()).toContain('No protocols created yet')
    expect(protocolsCard?.find('.btn-secondary').exists()).toBe(true)
  })

  it('displays active protocols when available', async () => {
    wrapper = mount(Dashboard)

    const protocolStore = useProtocolStore()
    protocolStore.protocols = [
      { id: '1', name: 'BPC-157 Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
      { id: '2', name: 'TB-500 Protocol', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.protocol-item')).toHaveLength(2)
    expect(wrapper.findAll('.protocol-name')[0]?.text()).toBe('BPC-157 Protocol')
    expect(wrapper.findAll('.protocol-peptide')[0]?.text()).toBe('BPC-157')
  })

  it('limits protocols to 5 items', async () => {
    wrapper = mount(Dashboard)

    const protocolStore = useProtocolStore()
    protocolStore.protocols = Array.from({ length: 10 }, (_, i) => ({
      id: `${i + 1}`,
      name: `Protocol ${i + 1}`,
      peptide_name: 'BPC-157',
      notes: null,
      target_concentration_mg_ml: null,
      created_at: new Date().toISOString(),
      updated_at: new Date().toISOString()
    }))

    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.protocol-item')).toHaveLength(5)
  })

  // =============================================================================
  // Inventory Alerts Card Tests
  // =============================================================================

  it('hides inventory alerts card when no expiring items', () => {
    wrapper = mount(Dashboard)

    // Find all cards and check if inventory alerts card exists
    const cards = wrapper.findAll('.card')
    const inventoryCard = cards.find(card => card.text().includes('Inventory Alerts'))
    expect(inventoryCard).toBeUndefined()
  })

  it('displays inventory alerts when items expiring soon', async () => {
    wrapper = mount(Dashboard)

    const supplierStore = useSupplierStore()

    const fifteenDaysFromNow = new Date()
    fifteenDaysFromNow.setDate(fifteenDaysFromNow.getDate() + 15)

    supplierStore.inventory = [
      {
        id: '1',
        protocol_id: 'p1',
        supplier_id: 's1',
        vial_status: 'sealed',
        amount_mg: 10.0,
        cost: null,
        received_date: null,
        expiry_date: fifteenDaysFromNow.toISOString().split('T')[0],
        notes: null,
        vial_number: '123',
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    await wrapper.vm.$nextTick()

    const cards = wrapper.findAll('.card')
    const inventoryCard = cards.find(card => card.text().includes('Inventory Alerts'))
    expect(inventoryCard).toBeDefined()
    expect(inventoryCard?.text()).toContain('Vial #123')
  })

  // =============================================================================
  // Child Components Tests
  // =============================================================================

  it('renders all analytics child components', () => {
    wrapper = mount(Dashboard)

    expect(wrapper.find('.dose-calendar-heatmap-stub').exists()).toBe(true)
    expect(wrapper.find('.protocol-progress-tracker-stub').exists()).toBe(true)
    expect(wrapper.find('.cost-analysis-dashboard-stub').exists()).toBe(true)
    expect(wrapper.find('.recent-activity-timeline-stub').exists()).toBe(true)
  })

  // =============================================================================
  // Helper Methods Tests
  // =============================================================================

  it('getAlertIcon returns correct icons for alert types', () => {
    wrapper = mount(Dashboard)

    expect(wrapper.vm.getAlertIcon('low_stock')).toBe('📉')
    expect(wrapper.vm.getAlertIcon('expiring_soon')).toBe('⏰')
    expect(wrapper.vm.getAlertIcon('expired')).toBe('⚠️')
    expect(wrapper.vm.getAlertIcon('price_increase')).toBe('📈')
    expect(wrapper.vm.getAlertIcon('price_decrease')).toBe('📉')
    expect(wrapper.vm.getAlertIcon('out_of_stock')).toBe('❌')
    expect(wrapper.vm.getAlertIcon('unknown' as any)).toBe('🔔')
  })

  it('formatDate returns relative time strings', () => {
    wrapper = mount(Dashboard)

    const now = new Date()

    // Just now (< 1 hour)
    const thirtyMinutesAgo = new Date(now.getTime() - 30 * 60 * 1000)
    expect(wrapper.vm.formatDate(thirtyMinutesAgo.toISOString())).toBe('Just now')

    // Hours ago (< 24 hours)
    const fiveHoursAgo = new Date(now.getTime() - 5 * 60 * 60 * 1000)
    expect(wrapper.vm.formatDate(fiveHoursAgo.toISOString())).toBe('5h ago')

    // Days ago (< 7 days)
    const threeDaysAgo = new Date(now.getTime() - 3 * 24 * 60 * 60 * 1000)
    expect(wrapper.vm.formatDate(threeDaysAgo.toISOString())).toBe('3d ago')
  })

  it('formatExpiryDate returns relative future time strings', () => {
    wrapper = mount(Dashboard)

    const now = new Date()

    // Today
    const today = new Date(now)
    today.setHours(23, 59, 59, 999)
    expect(wrapper.vm.formatExpiryDate(today.toISOString())).toBe('Today')

    // Tomorrow
    const tomorrow = new Date(now.getTime() + 24 * 60 * 60 * 1000)
    expect(wrapper.vm.formatExpiryDate(tomorrow.toISOString())).toBe('Tomorrow')

    // In 5 days
    const fiveDaysFromNow = new Date(now.getTime() + 5 * 24 * 60 * 60 * 1000)
    expect(wrapper.vm.formatExpiryDate(fiveDaysFromNow.toISOString())).toContain('in 5 days')

    // Expired
    const yesterday = new Date(now.getTime() - 24 * 60 * 60 * 1000)
    expect(wrapper.vm.formatExpiryDate(yesterday.toISOString())).toBe('Expired')

    // Null/undefined
    expect(wrapper.vm.formatExpiryDate(null)).toBe('Unknown')
    expect(wrapper.vm.formatExpiryDate(undefined)).toBe('Unknown')
  })

  it('getProtocolName returns correct protocol name', () => {
    wrapper = mount(Dashboard)

    const protocolStore = useProtocolStore()
    protocolStore.protocols = [
      { id: 'p1', name: 'BPC-157 Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
    ]

    expect(wrapper.vm.getProtocolName('p1')).toBe('BPC-157 Protocol')
    expect(wrapper.vm.getProtocolName('nonexistent')).toBe('Unknown Protocol')
  })
})
