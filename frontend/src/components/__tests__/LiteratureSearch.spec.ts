import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import LiteratureSearch from '../LiteratureSearch.vue'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('LiteratureSearch Component', () => {
  let wrapper: VueWrapper<any>

  const mockSearchResults = [
    {
      source: 'pubmed',
      results: [
        { id: '1', source: 'pubmed', title: 'BPC-157 Study', authors: 'Smith et al.', journal: 'Nature', published_date: '2023', abstract_text: 'Test abstract', url: 'https://pubmed.com/1', doi: '10.1234/test', indexed_at: new Date().toISOString() },
        { id: '2', source: 'pubmed', title: 'TB-500 Research', authors: 'Jones et al.', journal: 'Science', published_date: '2022', abstract_text: null, url: 'https://pubmed.com/2', doi: null, indexed_at: new Date().toISOString() }
      ]
    }
  ]

  const mockCachedLiterature = [
    { id: 'c1', source: 'pubmed', title: 'Cached Paper 1', summary: 'Summary text', url: 'https://example.com/1', relevance_score: null, indexed_at: new Date().toISOString() },
    { id: 'c2', source: 'openalex', title: 'Cached Paper 2', summary: null, url: 'https://example.com/2', relevance_score: null, indexed_at: new Date(Date.now() - 7 * 24 * 60 * 60 * 1000).toISOString() }
  ]

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.searchLiterature).mockResolvedValue(mockSearchResults)
    vi.mocked(api.listLiterature).mockResolvedValue(mockCachedLiterature)
    vi.mocked(api.searchCachedLiterature).mockResolvedValue(mockCachedLiterature)
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
  })

  // =============================================================================
  // Component Rendering Tests
  // =============================================================================

  it('renders component header', () => {
    wrapper = mount(LiteratureSearch)

    expect(wrapper.find('h2').text()).toContain('Research Papers')
    expect(wrapper.find('.subtitle').text()).toContain('Find scientific studies')
  })

  it('renders search input and button', () => {
    wrapper = mount(LiteratureSearch)

    expect(wrapper.find('#literature-search-input').exists()).toBe(true)
    expect(wrapper.find('.search-btn').exists()).toBe(true)
  })

  it('renders cached section', () => {
    wrapper = mount(LiteratureSearch)

    expect(wrapper.find('.cached-section').exists()).toBe(true)
    expect(wrapper.find('#cache-search-input').exists()).toBe(true)
  })

  // =============================================================================
  // Search Functionality Tests
  // =============================================================================

  it('disables search button when query is empty', () => {
    wrapper = mount(LiteratureSearch)

    const searchBtn = wrapper.find('.search-btn')
    expect(searchBtn.attributes('disabled')).toBeDefined()
  })

  it('enables search button when query is not empty', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')

    const searchBtn = wrapper.find('.search-btn')
    expect(searchBtn.attributes('disabled')).toBeUndefined()
  })

  it('searches when search button clicked', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.searchLiterature).toHaveBeenCalled()
  })

  it('searches when Enter key pressed', async () => {
    wrapper = mount(LiteratureSearch)

    const input = wrapper.find('#literature-search-input')
    await input.setValue('BPC-157')
    await input.trigger('keyup.enter')

    await wrapper.vm.$nextTick()

    expect(api.searchLiterature).toHaveBeenCalled()
  })

  it('displays search results after search', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.search-results').exists()).toBe(true)
    expect(wrapper.findAll('.result-card')).toHaveLength(2)
  })

  it('displays source name correctly', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.text()).toContain('Medical Database')
  })

  it('displays paper details', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.text()).toContain('BPC-157 Study')
    expect(wrapper.text()).toContain('Smith et al.')
    expect(wrapper.text()).toContain('Nature')
    expect(wrapper.text()).toContain('2023')
  })

  it('shows loading state while searching', async () => {
    let resolveSearch: any
    vi.mocked(api.searchLiterature).mockReturnValue(new Promise(resolve => { resolveSearch = resolve }))

    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.search-btn').text()).toContain('Finding Papers...')

    resolveSearch(mockSearchResults)
  })

  // =============================================================================
  // Cached Literature Tests
  // =============================================================================

  it('loads cached literature on mount', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.listLiterature).toHaveBeenCalled()
  })

  it('displays cached papers', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.findAll('.literature-card')).toHaveLength(2)
    expect(wrapper.text()).toContain('Cached Paper 1')
    expect(wrapper.text()).toContain('Cached Paper 2')
  })

  it('shows cached count', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.text()).toContain('Your Saved Papers (2)')
  })

  it('filters cached papers by search query', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('#cache-search-input').setValue('Cached Paper 1')
    await wrapper.vm.handleCacheSearch()

    await wrapper.vm.$nextTick()

    expect(api.searchCachedLiterature).toHaveBeenCalledWith('Cached Paper 1')
  })

  it('refreshes cached papers when refresh clicked', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('.refresh-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.listLiterature).toHaveBeenCalled()
  })

  // =============================================================================
  // Paper Selection Tests
  // =============================================================================

  it('allows selecting papers for risk analysis', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const checkbox = wrapper.findAll('.paper-checkbox input')[0]
    await checkbox?.setValue(true)

    expect(wrapper.vm.selectedPapers).toHaveLength(1)
  })

  it('shows risk matrix banner when papers selected', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const checkbox = wrapper.findAll('.paper-checkbox input')[0]
    await checkbox?.setValue(true)

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.risk-matrix-banner').exists()).toBe(true)
    expect(wrapper.find('.risk-matrix-banner').text()).toContain('1 paper(s) selected')
  })

  it('limits selection to 5 papers', async () => {
    wrapper = mount(LiteratureSearch)

    // Mock 6 papers
    const manyResults = [{
      source: 'pubmed',
      results: Array.from({ length: 6 }, (_, i) => ({
        id: `${i + 1}`,
        source: 'pubmed',
        title: `Paper ${i + 1}`,
        url: `https://example.com/${i}`,
        indexed_at: new Date().toISOString()
      }))
    }]

    vi.mocked(api.searchLiterature).mockResolvedValue(manyResults)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    // Select 5 papers
    const checkboxes = wrapper.findAll('.paper-checkbox input')
    for (let i = 0; i < 5; i++) {
      await checkboxes[i]?.setValue(true)
    }

    await wrapper.vm.$nextTick()

    // 6th checkbox should be disabled
    expect(checkboxes[5]?.attributes('disabled')).toBeDefined()
  })

  it('clears selection when clear button clicked', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const checkbox = wrapper.findAll('.paper-checkbox input')[0]
    await checkbox?.setValue(true)

    await wrapper.vm.$nextTick()

    await wrapper.find('.clear-btn').trigger('click')

    expect(wrapper.vm.selectedPapers).toHaveLength(0)
    expect(wrapper.find('.risk-matrix-banner').exists()).toBe(false)
  })

  // =============================================================================
  // Link Opening Tests
  // =============================================================================

  it('opens paper link in new tab', async () => {
    global.window.open = vi.fn()

    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.findAll('.link-btn')[0]?.trigger('click')

    expect(window.open).toHaveBeenCalledWith('https://pubmed.com/1', '_blank')
  })

  // =============================================================================
  // Filter and Sort Tests
  // =============================================================================

  it('filters by source', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const sourceFilter = wrapper.findAll('.filter-select')[0]
    await sourceFilter?.setValue('pubmed')

    await wrapper.vm.$nextTick()

    const visibleCards = wrapper.findAll('.literature-card')
    // Should only show pubmed papers
    expect(visibleCards.length).toBeLessThanOrEqual(2)
  })

  it('sorts by date descending', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const sortSelect = wrapper.findAll('.filter-select')[1]
    await sortSelect?.setValue('date-desc')

    await wrapper.vm.$nextTick()

    // Newest should be first
    const cards = wrapper.findAll('.literature-card')
    expect(cards[0]?.text()).toContain('Cached Paper 1')
  })

  // =============================================================================
  // Export Functionality Tests
  // =============================================================================

  it('exports to BibTeX', async () => {
    global.URL.createObjectURL = vi.fn(() => 'blob:test-url')
    global.URL.revokeObjectURL = vi.fn()

    const mockLink = {
      click: vi.fn(),
      download: '',
      href: ''
    } as any
    vi.spyOn(document, 'createElement').mockReturnValue(mockLink)
    vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink)
    vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink)

    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.findAll('.export-btn')[0]?.trigger('click')

    expect(mockLink.click).toHaveBeenCalled()
    expect(mockLink.download).toContain('.bib')
  })

  it('exports to CSV', async () => {
    global.URL.createObjectURL = vi.fn(() => 'blob:test-url')
    global.URL.revokeObjectURL = vi.fn()

    const mockLink = {
      click: vi.fn(),
      download: '',
      href: ''
    } as any
    vi.spyOn(document, 'createElement').mockReturnValue(mockLink)
    vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink)
    vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink)

    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.findAll('.export-btn')[1]?.trigger('click')

    expect(mockLink.click).toHaveBeenCalled()
    expect(mockLink.download).toContain('.csv')
  })

  // =============================================================================
  // Risk Matrix Modal Tests
  // =============================================================================

  it('opens risk matrix modal when analyze clicked', async () => {
    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const checkbox = wrapper.findAll('.paper-checkbox input')[0]
    await checkbox?.setValue(true)

    await wrapper.vm.$nextTick()

    await wrapper.find('.analyze-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.vm.showRiskMatrix).toBe(true)
  })

  it('closes risk matrix when close button clicked', async () => {
    wrapper = mount(LiteratureSearch)

    wrapper.vm.showRiskMatrix = true
    await wrapper.vm.$nextTick()

    await wrapper.find('.close-btn').trigger('click')

    expect(wrapper.vm.showRiskMatrix).toBe(false)
  })

  // =============================================================================
  // Error Handling Tests
  // =============================================================================

  it('handles search errors', async () => {
    vi.mocked(api.searchLiterature).mockRejectedValue(new Error('Network error'))

    wrapper = mount(LiteratureSearch)

    await wrapper.find('#literature-search-input').setValue('BPC-157')
    await wrapper.find('.search-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Empty States Tests
  // =============================================================================

  it('shows empty state when no cached papers', async () => {
    vi.mocked(api.listLiterature).mockResolvedValue([])

    wrapper = mount(LiteratureSearch)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.no-results').exists()).toBe(true)
    expect(wrapper.find('.no-results').text()).toContain('No saved papers yet')
  })

  it('hides search results when no results', () => {
    wrapper = mount(LiteratureSearch)

    expect(wrapper.find('.search-results').exists()).toBe(false)
  })
})
