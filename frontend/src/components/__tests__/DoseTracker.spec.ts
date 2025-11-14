import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import DoseTracker from '../DoseTracker.vue'
import * as api from '../../api/peptrack'

// Mock child component
vi.mock('../DoseScheduleManager.vue', () => ({
  default: { name: 'DoseScheduleManager', template: '<div class="dose-schedule-manager-stub"></div>' }
}))

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))
vi.mock('../../utils/dateFormatter', () => ({
  formatDate: vi.fn((date) => new Date(date).toLocaleDateString())
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('DoseTracker Component', () => {
  let wrapper: VueWrapper<any>

  const mockProtocols = [
    { id: 'p1', name: 'BPC-157 Protocol', peptide_name: 'BPC-157', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    { id: 'p2', name: 'TB-500 Protocol', peptide_name: 'TB-500', notes: null, target_concentration_mg_ml: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
  ]

  const mockDoses = [
    { id: 'd1', protocol_id: 'p1', injection_site: 'Deltoid', amount_mg: 2.5, notes: 'Feeling good', logged_at: new Date().toISOString() },
    { id: 'd2', protocol_id: 'p2', injection_site: 'Quad', amount_mg: 3.0, notes: null, logged_at: new Date().toISOString() }
  ]

  beforeEach(() => {
    vi.clearAllMocks()

    // Default mocks
    vi.mocked(api.listProtocols).mockResolvedValue(mockProtocols)
    vi.mocked(api.listDoseLogs).mockResolvedValue(mockDoses)
    vi.mocked(api.listDoseLogsForProtocol).mockResolvedValue([mockDoses[0]!])
    vi.mocked(api.logDose).mockResolvedValue(mockDoses[0]!)
    vi.mocked(api.deleteDoseLog).mockResolvedValue(undefined)
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  // =============================================================================
  // Component Mounting & Data Loading Tests
  // =============================================================================

  it('renders component header', () => {
    wrapper = mount(DoseTracker)

    expect(wrapper.find('h2').text()).toContain('Track Your Doses')
    expect(wrapper.find('.subtitle').text()).toContain('Log doses and set up recurring schedules')
  })

  it('loads protocols on mount', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listProtocols).toHaveBeenCalled()
  })

  it('loads doses on mount', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  // =============================================================================
  // Tab Navigation Tests
  // =============================================================================

  it('renders tab navigation', () => {
    wrapper = mount(DoseTracker)

    const tabs = wrapper.findAll('.tab-btn')
    expect(tabs).toHaveLength(2)
    expect(tabs[0]?.text()).toContain('Log Dose')
    expect(tabs[1]?.text()).toContain('Schedules')
  })

  it('defaults to log tab', () => {
    wrapper = mount(DoseTracker)

    const logTab = wrapper.findAll('.tab-btn')[0]
    expect(logTab?.classes()).toContain('active')
  })

  it('switches to schedules tab when clicked', async () => {
    wrapper = mount(DoseTracker)

    const schedulesTab = wrapper.findAll('.tab-btn')[1]
    await schedulesTab?.trigger('click')

    expect(schedulesTab?.classes()).toContain('active')
    expect(wrapper.find('.dose-schedule-manager-stub').exists()).toBe(true)
  })

  it('shows log tab content when log tab active', () => {
    wrapper = mount(DoseTracker)

    expect(wrapper.find('.log-dose-section').isVisible()).toBe(true)
  })

  // =============================================================================
  // Form Rendering Tests
  // =============================================================================

  it('renders dose form with all fields', () => {
    wrapper = mount(DoseTracker)

    expect(wrapper.find('#dose-protocol-select').exists()).toBe(true)
    expect(wrapper.find('#dose-amount-input').exists()).toBe(true)
    expect(wrapper.find('#dose-site-input').exists()).toBe(true)
    expect(wrapper.find('#dose-notes-input').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('populates protocol dropdown with loaded protocols', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const options = wrapper.find('#dose-protocol-select').findAll('option')
    // +1 for the "Select a plan..." option
    expect(options.length).toBe(mockProtocols.length + 1)
    expect(options[1]?.text()).toContain('BPC-157 Protocol')
    expect(options[2]?.text()).toContain('TB-500 Protocol')
  })

  it('shows empty state when no protocols', async () => {
    vi.mocked(api.listProtocols).mockResolvedValue([])

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.empty-state').exists()).toBe(true)
    expect(wrapper.find('.empty-state').text()).toContain('at least one peptide plan')
  })

  it('disables form when no protocols', async () => {
    vi.mocked(api.listProtocols).mockResolvedValue([])

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('#dose-protocol-select').attributes('disabled')).toBeDefined()
    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeDefined()
  })

  // =============================================================================
  // Form Submission Tests
  // =============================================================================

  it('logs dose when form submitted with valid data', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')
    await wrapper.find('#dose-notes-input').setValue('Test notes')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(api.logDose).toHaveBeenCalledWith({
      protocolId: 'p1',
      site: 'Deltoid',
      amountMg: 2.5,
      notes: 'Test notes'
    })
  })

  it('shows success message after logging dose', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith('Success', 'Dose logged successfully!')
  })

  it('resets form after successful submission', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')
    await wrapper.find('#dose-notes-input').setValue('Test notes')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.vm.form.protocolId).toBe('')
    expect(wrapper.vm.form.site).toBe('')
    expect(wrapper.vm.form.amountMg).toBe(0)
    expect(wrapper.vm.form.notes).toBe('')
  })

  it('reloads doses after successful submission', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  it('shows loading state while submitting', async () => {
    let resolveLogDose: any
    vi.mocked(api.logDose).mockReturnValue(new Promise(resolve => { resolveLogDose = resolve }))

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('button[type="submit"]').text()).toContain('Logging...')
    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeDefined()

    resolveLogDose(mockDoses[0])
  })

  it('handles submission errors gracefully', async () => {
    vi.mocked(api.logDose).mockRejectedValue(new Error('Network error'))

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Form Validation Tests
  // =============================================================================

  it('shows error for invalid amount (0)', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('0')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.error).toContain('valid values')
    expect(api.logDose).not.toHaveBeenCalled()
  })

  it('shows error for invalid amount (NaN)', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    wrapper.vm.form.protocolId = 'p1'
    wrapper.vm.form.site = 'Deltoid'
    wrapper.vm.form.amountMg = NaN

    await wrapper.vm.handleLogDose()

    expect(wrapper.vm.error).toContain('valid values')
    expect(api.logDose).not.toHaveBeenCalled()
  })

  it('shows error for missing protocol', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-amount-input').setValue('2.5')
    await wrapper.find('#dose-site-input').setValue('Deltoid')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.error).toContain('valid values')
    expect(api.logDose).not.toHaveBeenCalled()
  })

  it('shows error for missing site', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#dose-protocol-select').setValue('p1')
    await wrapper.find('#dose-amount-input').setValue('2.5')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.error).toContain('valid values')
    expect(api.logDose).not.toHaveBeenCalled()
  })

  // =============================================================================
  // Dose History Tests
  // =============================================================================

  it('displays dose history when doses exist', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.findAll('.dose-card')).toHaveLength(2)
  })

  it('shows empty state when no doses', async () => {
    vi.mocked(api.listDoseLogs).mockResolvedValue([])

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.no-doses').exists()).toBe(true)
    expect(wrapper.find('.no-doses').text()).toContain('No doses logged yet')
  })

  it('displays dose details correctly', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstDose = wrapper.findAll('.dose-card')[0]
    expect(firstDose?.text()).toContain('BPC-157 Protocol')
    expect(firstDose?.text()).toContain('2.5 mg')
    expect(firstDose?.text()).toContain('Deltoid')
    expect(firstDose?.text()).toContain('Feeling good')
  })

  it('displays dose notes when present', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstDose = wrapper.findAll('.dose-card')[0]
    expect(firstDose?.find('.dose-notes').exists()).toBe(true)
  })

  it('hides dose notes when not present', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const secondDose = wrapper.findAll('.dose-card')[1]
    expect(secondDose?.find('.dose-notes').exists()).toBe(false)
  })

  // =============================================================================
  // Filter & Refresh Tests
  // =============================================================================

  it('filters doses by protocol when filter selected', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('#filter-protocol-select').setValue('p1')

    await wrapper.vm.$nextTick()

    expect(api.listDoseLogsForProtocol).toHaveBeenCalledWith('p1')
  })

  it('loads all doses when filter cleared', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    // Set filter
    await wrapper.find('#filter-protocol-select').setValue('p1')
    await wrapper.vm.$nextTick()

    vi.clearAllMocks()

    // Clear filter
    await wrapper.find('#filter-protocol-select').setValue('')
    await wrapper.vm.$nextTick()

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  it('refreshes doses when refresh button clicked', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    const refreshButton = wrapper.find('.refresh-btn')
    await refreshButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  // =============================================================================
  // Delete Tests
  // =============================================================================

  it('deletes dose when delete button clicked and confirmed', async () => {
    global.confirm = vi.fn(() => true)

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.deleteDoseLog).toHaveBeenCalledWith('d1')
  })

  it('does not delete dose when not confirmed', async () => {
    global.confirm = vi.fn(() => false)

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.deleteDoseLog).not.toHaveBeenCalled()
  })

  it('shows success message after deleting dose', async () => {
    global.confirm = vi.fn(() => true)

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith('Success', 'Dose deleted successfully')
  })

  it('reloads doses after deleting', async () => {
    global.confirm = vi.fn(() => true)

    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listDoseLogs).toHaveBeenCalled()
  })

  // =============================================================================
  // Helper Methods Tests
  // =============================================================================

  it('getProtocolName returns correct protocol name', async () => {
    wrapper = mount(DoseTracker)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.vm.getProtocolName('p1')).toBe('BPC-157 Protocol (BPC-157)')
    expect(wrapper.vm.getProtocolName('p2')).toBe('TB-500 Protocol (TB-500)')
    expect(wrapper.vm.getProtocolName('nonexistent')).toBe('Unknown Protocol')
  })
})
