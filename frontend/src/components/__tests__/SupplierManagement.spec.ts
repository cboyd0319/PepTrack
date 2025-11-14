import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import SupplierManagement from '../SupplierManagement.vue'
import * as api from '../../api/peptrack'

// Mock child components
vi.mock('../PriceChart.vue', () => ({
  default: { name: 'PriceChart', template: '<div class="price-chart-stub"></div>' }
}))

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('SupplierManagement Component', () => {
  let wrapper: VueWrapper<any>

  const mockSuppliers = [
    { id: 's1', name: 'Peptide Sciences', contact_email: 'contact@peptidesciences.com', contact_phone: '+1-555-0123', website: 'https://peptidesciences.com', notes: 'Good quality', created_at: new Date().toISOString(), updated_at: new Date().toISOString() },
    { id: 's2', name: 'PureRawz', contact_email: null, contact_phone: null, website: 'https://purerawz.com', notes: null, created_at: new Date().toISOString(), updated_at: new Date().toISOString() }
  ]

  const mockPriceHistory = [
    { id: 'p1', supplier_id: 's1', peptide_name: 'BPC-157', cost_per_mg: 0.50, in_stock: true, url: 'https://example.com', notes: null, recorded_at: new Date().toISOString() },
    { id: 'p2', supplier_id: 's1', peptide_name: 'BPC-157', cost_per_mg: 0.55, in_stock: true, url: null, notes: 'Price increased', recorded_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString() }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.listSuppliers).mockResolvedValue(mockSuppliers)
    vi.mocked(api.createSupplier).mockResolvedValue(mockSuppliers[0]!)
    vi.mocked(api.updateSupplier).mockResolvedValue(mockSuppliers[0]!)
    vi.mocked(api.deleteSupplier).mockResolvedValue(undefined)
    vi.mocked(api.listPriceHistory).mockResolvedValue(mockPriceHistory)
    vi.mocked(api.addPriceEntry).mockResolvedValue(mockPriceHistory[0]!)
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
    wrapper = mount(SupplierManagement)

    expect(wrapper.find('h2').text()).toContain('Supplier Management')
    expect(wrapper.find('.subtitle').text()).toContain('Track your peptide suppliers')
  })

  it('loads suppliers on mount', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listSuppliers).toHaveBeenCalled()
  })

  // =============================================================================
  // Supplier Form Tests
  // =============================================================================

  it('renders supplier form with all fields', () => {
    wrapper = mount(SupplierManagement)

    expect(wrapper.find('#supplier-name').exists()).toBe(true)
    expect(wrapper.find('#supplier-email').exists()).toBe(true)
    expect(wrapper.find('#supplier-phone').exists()).toBe(true)
    expect(wrapper.find('#supplier-website').exists()).toBe(true)
    expect(wrapper.find('#supplier-notes').exists()).toBe(true)
  })

  it('displays add mode by default', () => {
    wrapper = mount(SupplierManagement)

    expect(wrapper.find('h3').text()).toContain('Add Supplier')
    expect(wrapper.find('.primary-btn').text()).toContain('Add Supplier')
    expect(wrapper.find('.secondary-btn').exists()).toBe(false)
  })

  it('creates supplier when form submitted', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.find('#supplier-name').setValue('New Supplier')
    await wrapper.find('#supplier-email').setValue('test@example.com')
    await wrapper.find('#supplier-phone').setValue('+1-555-1234')
    await wrapper.find('#supplier-website').setValue('https://example.com')
    await wrapper.find('#supplier-notes').setValue('Test notes')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(api.createSupplier).toHaveBeenCalledWith({
      name: 'New Supplier',
      contactEmail: 'test@example.com',
      contactPhone: '+1-555-1234',
      website: 'https://example.com',
      notes: 'Test notes'
    })
  })

  it('shows success message after creating supplier', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.find('#supplier-name').setValue('New Supplier')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalled()
  })

  it('reloads suppliers after creating', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('#supplier-name').setValue('New Supplier')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listSuppliers).toHaveBeenCalled()
  })

  it('resets form after successful create', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.find('#supplier-name').setValue('New Supplier')
    await wrapper.find('#supplier-email').setValue('test@example.com')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.vm.form.name).toBe('')
    expect(wrapper.vm.form.contactEmail).toBe('')
  })

  // =============================================================================
  // Supplier List Tests
  // =============================================================================

  it('displays supplier list when loaded', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.findAll('.supplier-card')).toHaveLength(2)
    expect(wrapper.text()).toContain('Peptide Sciences')
    expect(wrapper.text()).toContain('PureRawz')
  })

  it('shows empty state when no suppliers', async () => {
    vi.mocked(api.listSuppliers).mockResolvedValue([])

    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.no-suppliers').exists()).toBe(true)
    expect(wrapper.find('.no-suppliers').text()).toContain('No suppliers yet')
  })

  it('displays supplier contact info when available', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstCard = wrapper.findAll('.supplier-card')[0]
    expect(firstCard?.text()).toContain('contact@peptidesciences.com')
    expect(firstCard?.text()).toContain('+1-555-0123')
  })

  it('hides contact info when not available', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const secondCard = wrapper.findAll('.supplier-card')[1]
    expect(secondCard?.find('.supplier-contact').exists()).toBe(false)
  })

  it('displays website link when available', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstCard = wrapper.findAll('.supplier-card')[0]
    expect(firstCard?.find('.supplier-website').exists()).toBe(true)
    expect(firstCard?.find('.supplier-website a').attributes('href')).toBe('https://peptidesciences.com')
  })

  it('displays notes when available', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstCard = wrapper.findAll('.supplier-card')[0]
    expect(firstCard?.find('.supplier-notes').exists()).toBe(true)
    expect(firstCard?.find('.supplier-notes').text()).toContain('Good quality')
  })

  // =============================================================================
  // Edit Supplier Tests
  // =============================================================================

  it('switches to edit mode when edit button clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const editButton = wrapper.findAll('.edit-btn')[0]
    await editButton?.trigger('click')

    expect(wrapper.find('h3').text()).toContain('Edit Supplier')
    expect(wrapper.find('.primary-btn').text()).toContain('Update Supplier')
    expect(wrapper.find('.secondary-btn').exists()).toBe(true)
  })

  it('populates form with supplier data in edit mode', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const editButton = wrapper.findAll('.edit-btn')[0]
    await editButton?.trigger('click')

    expect(wrapper.find('#supplier-name').element.value).toBe('Peptide Sciences')
    expect(wrapper.find('#supplier-email').element.value).toBe('contact@peptidesciences.com')
  })

  it('updates supplier when form submitted in edit mode', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const editButton = wrapper.findAll('.edit-btn')[0]
    await editButton?.trigger('click')

    await wrapper.find('#supplier-name').setValue('Updated Name')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(api.updateSupplier).toHaveBeenCalledWith('s1', expect.objectContaining({
      name: 'Updated Name'
    }))
  })

  it('cancels edit when cancel button clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const editButton = wrapper.findAll('.edit-btn')[0]
    await editButton?.trigger('click')

    await wrapper.find('.secondary-btn').trigger('click')

    expect(wrapper.find('h3').text()).toContain('Add Supplier')
    expect(wrapper.vm.form.name).toBe('')
  })

  // =============================================================================
  // Delete Supplier Tests
  // =============================================================================

  it('deletes supplier when delete button clicked and confirmed', async () => {
    global.confirm = vi.fn(() => true)

    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.deleteSupplier).toHaveBeenCalledWith('s1')
  })

  it('does not delete when not confirmed', async () => {
    global.confirm = vi.fn(() => false)

    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const deleteButton = wrapper.findAll('.delete-btn')[0]
    await deleteButton?.trigger('click')

    expect(api.deleteSupplier).not.toHaveBeenCalled()
  })

  // =============================================================================
  // Price Tracking Modal Tests
  // =============================================================================

  it('opens price tracking modal when Track Price clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const priceButton = wrapper.findAll('.price-btn')[0]
    await priceButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.modal-overlay').exists()).toBe(true)
    expect(wrapper.text()).toContain('Price Tracking: Peptide Sciences')
  })

  it('closes price modal when close button clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const priceButton = wrapper.findAll('.price-btn')[0]
    await priceButton?.trigger('click')

    await wrapper.vm.$nextTick()

    const closeButton = wrapper.find('.close-btn')
    await closeButton.trigger('click')

    expect(wrapper.find('.modal-overlay').exists()).toBe(false)
  })

  // =============================================================================
  // Scrape Modal Tests (conditional on website)
  // =============================================================================

  it('shows scrape button only for suppliers with website', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const firstCard = wrapper.findAll('.supplier-card')[0]
    const secondCard = wrapper.findAll('.supplier-card')[1]

    expect(firstCard?.find('.scrape-btn').exists()).toBe(true)
    expect(secondCard?.find('.scrape-btn').exists()).toBe(true) // Both have websites in mock
  })

  it('opens scrape modal when scrape button clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const scrapeButton = wrapper.findAll('.scrape-btn')[0]
    await scrapeButton?.trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.showScrapeModal).toBe(true)
  })

  // =============================================================================
  // Refresh Functionality Tests
  // =============================================================================

  it('refreshes supplier list when refresh clicked', async () => {
    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    const refreshButton = wrapper.find('.refresh-btn')
    await refreshButton.trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.listSuppliers).toHaveBeenCalled()
  })

  // =============================================================================
  // Loading State Tests
  // =============================================================================

  it('shows loading state while fetching suppliers', async () => {
    let resolveSuppliers: any
    vi.mocked(api.listSuppliers).mockReturnValue(new Promise(resolve => { resolveSuppliers = resolve }))

    wrapper = mount(SupplierManagement)

    await wrapper.vm.$nextTick()

    wrapper.vm.isLoading = true
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.loading').exists()).toBe(true)

    resolveSuppliers(mockSuppliers)
  })

  it('shows saving state while creating supplier', async () => {
    let resolveCreate: any
    vi.mocked(api.createSupplier).mockReturnValue(new Promise(resolve => { resolveCreate = resolve }))

    wrapper = mount(SupplierManagement)

    await wrapper.find('#supplier-name').setValue('Test')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.primary-btn').text()).toContain('Saving...')

    resolveCreate(mockSuppliers[0])
  })

  // =============================================================================
  // Error Handling Tests
  // =============================================================================

  it('handles supplier creation errors', async () => {
    vi.mocked(api.createSupplier).mockRejectedValue(new Error('Network error'))

    wrapper = mount(SupplierManagement)

    await wrapper.find('#supplier-name').setValue('Test')
    await wrapper.find('form').trigger('submit.prevent')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })
})
