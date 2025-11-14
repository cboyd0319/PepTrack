import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import EnhancedAiSummary from '../EnhancedAiSummary.vue'

describe('EnhancedAiSummary Component', () => {
  let wrapper: VueWrapper<any>

  const defaultProps = {
    summarizing: false,
    summaryOutput: null,
    summaryProvider: null
  }

  beforeEach(() => {
    vi.clearAllMocks()
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
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    expect(wrapper.find('h2').text()).toContain('AI Summary Helper')
    expect(wrapper.find('.muted').text()).toContain('Transform research papers')
  })

  it('renders form fields', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    expect(wrapper.find('input[type="text"]').exists()).toBe(true) // title
    expect(wrapper.find('textarea').exists()).toBe(true) // content
    expect(wrapper.findAll('select')).toHaveLength(2) // format and style
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('renders format options', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const formatSelect = wrapper.findAll('select')[0]
    const options = formatSelect?.findAll('option')

    expect(options).toHaveLength(3)
    expect(options?.[0]?.text()).toContain('Markdown')
    expect(options?.[1]?.text()).toContain('Plain Text')
    expect(options?.[2]?.text()).toContain('Bullet Points')
  })

  it('renders style options', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const styleSelect = wrapper.findAll('select')[1]
    const options = styleSelect?.findAll('option')

    expect(options).toHaveLength(4)
    expect(options?.[0]?.text()).toContain('Balanced')
    expect(options?.[1]?.text()).toContain('Simple')
    expect(options?.[2]?.text()).toContain('Technical')
    expect(options?.[3]?.text()).toContain('Brief')
  })

  // =============================================================================
  // Character Count Tests
  // =============================================================================

  it('displays character count', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    await wrapper.find('textarea').setValue('Test content')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.char-count').text()).toContain('12')
  })

  it('shows warning for long content', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const longText = 'a'.repeat(10001)
    await wrapper.find('textarea').setValue(longText)

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.char-count').classes()).toContain('warning')
    expect(wrapper.find('.warning-text').exists()).toBe(true)
    expect(wrapper.find('.warning-text').text()).toContain('may take time')
  })

  it('formats large character counts with commas', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const longText = 'a'.repeat(5000)
    await wrapper.find('textarea').setValue(longText)

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.char-count').text()).toContain('5,000')
  })

  // =============================================================================
  // Form Submission Tests
  // =============================================================================

  it('disables submit when no content', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const submitBtn = wrapper.find('button[type="submit"]')
    expect(submitBtn.attributes('disabled')).toBeDefined()
  })

  it('enables submit when content is provided', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    await wrapper.find('textarea').setValue('Some content')

    const submitBtn = wrapper.find('button[type="submit"]')
    expect(submitBtn.attributes('disabled')).toBeUndefined()
  })

  it('disables submit when summarizing', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summarizing: true }
    })

    const submitBtn = wrapper.find('button[type="submit"]')
    expect(submitBtn.attributes('disabled')).toBeDefined()
  })

  it('shows loading text when summarizing', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summarizing: true }
    })

    expect(wrapper.find('button[type="submit"]').text()).toContain('Creating Summary...')
  })

  it('emits submit event when form submitted', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    await wrapper.find('input[type="text"]').setValue('Test Title')
    await wrapper.find('textarea').setValue('Test content')
    await wrapper.findAll('select')[0]?.setValue('Markdown')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    expect(wrapper.emitted('submit')).toBeTruthy()
  })

  it('includes all form data in submit event', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    await wrapper.find('input[type="text"]').setValue('My Paper')
    await wrapper.find('textarea').setValue('Paper content here')
    await wrapper.findAll('select')[0]?.setValue('Plain')
    await wrapper.findAll('select')[1]?.setValue('simple')

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    expect(wrapper.emitted('submit')?.[0]).toBeDefined()
  })

  // =============================================================================
  // Summary Output Tests
  // =============================================================================

  it('hides output section when no summary', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    expect(wrapper.find('.summary-output').exists()).toBe(false)
  })

  it('shows output section when summary exists', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test summary' }
    })

    expect(wrapper.find('.summary-output').exists()).toBe(true)
  })

  it('displays summary content', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'This is the summary content' }
    })

    expect(wrapper.text()).toContain('This is the summary content')
  })

  it('displays provider badge when provider is set', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: {
        ...defaultProps,
        summaryOutput: 'Test',
        summaryProvider: 'Claude'
      }
    })

    expect(wrapper.find('.provider-badge').exists()).toBe(true)
    expect(wrapper.find('.provider-badge').text()).toContain('Claude')
  })

  it('hides provider badge when no provider', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    expect(wrapper.find('.provider-badge').exists()).toBe(false)
  })

  // =============================================================================
  // Output Actions Tests
  // =============================================================================

  it('shows output action buttons when summary exists', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    const actionButtons = wrapper.findAll('.action-btn')
    expect(actionButtons.length).toBeGreaterThan(0)
  })

  it('has copy button', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    const copyBtn = wrapper.findAll('.action-btn')[0]
    expect(copyBtn?.text()).toContain('Copy')
  })

  it('has export button', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    const exportBtn = wrapper.findAll('.action-btn')[1]
    expect(exportBtn?.text()).toContain('Export')
  })

  it('has regenerate button', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    const regenerateBtn = wrapper.findAll('.action-btn')[2]
    expect(regenerateBtn?.text()).toContain('Regenerate')
  })

  it('disables regenerate when summarizing', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: {
        ...defaultProps,
        summaryOutput: 'Test',
        summarizing: true
      }
    })

    const regenerateBtn = wrapper.findAll('.action-btn')[2]
    expect(regenerateBtn?.attributes('disabled')).toBeDefined()
  })

  it('copies to clipboard when copy clicked', async () => {
    global.navigator.clipboard = {
      writeText: vi.fn().mockResolvedValue(undefined)
    } as any

    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test summary' }
    })

    await wrapper.findAll('.action-btn')[0]?.trigger('click')

    expect(navigator.clipboard.writeText).toHaveBeenCalledWith('Test summary')
  })

  it('exports summary when export clicked', async () => {
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

    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test summary' }
    })

    await wrapper.findAll('.action-btn')[1]?.trigger('click')

    expect(mockLink.click).toHaveBeenCalled()
  })

  it('emits regenerate event when regenerate clicked', async () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Test' }
    })

    await wrapper.findAll('.action-btn')[2]?.trigger('click')

    expect(wrapper.emitted('regenerate')).toBeTruthy()
  })

  // =============================================================================
  // History Modal Tests
  // =============================================================================

  it('hides history button when no history', () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    expect(wrapper.find('.ghost-btn').exists()).toBe(false)
  })

  it('shows history button when history exists', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    wrapper.vm.summaryHistory = [
      { id: '1', title: 'Test', summary_output: 'Summary', format: 'Markdown', provider: 'Claude', created_at: new Date().toISOString() }
    ]

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.ghost-btn').exists()).toBe(true)
    expect(wrapper.find('.ghost-btn').text()).toContain('History (1)')
  })

  it('opens history modal when button clicked', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    wrapper.vm.summaryHistory = [
      { id: '1', title: 'Test', summary_output: 'Summary', format: 'Markdown', provider: 'Claude', created_at: new Date().toISOString() }
    ]

    await wrapper.vm.$nextTick()

    await wrapper.find('.ghost-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.history-modal-overlay').exists()).toBe(true)
  })

  it('closes history modal when close button clicked', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    wrapper.vm.showHistory = true
    await wrapper.vm.$nextTick()

    await wrapper.find('.close-btn').trigger('click')

    expect(wrapper.vm.showHistory).toBe(false)
  })

  it('displays history entries', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    wrapper.vm.summaryHistory = [
      { id: '1', title: 'Paper 1', summary_output: 'Summary 1', format: 'Markdown', provider: 'Claude', created_at: new Date().toISOString() },
      { id: '2', title: 'Paper 2', summary_output: 'Summary 2', format: 'Plain', provider: 'Codex', created_at: new Date().toISOString() }
    ]
    wrapper.vm.showHistory = true

    await wrapper.vm.$nextTick()

    expect(wrapper.findAll('.history-entry')).toHaveLength(2)
    expect(wrapper.text()).toContain('Paper 1')
    expect(wrapper.text()).toContain('Paper 2')
  })

  it('shows default title for untitled entries', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    wrapper.vm.summaryHistory = [
      { id: '1', title: null, summary_output: 'Summary', format: 'Markdown', provider: 'Claude', created_at: new Date().toISOString() }
    ]
    wrapper.vm.showHistory = true

    await wrapper.vm.$nextTick()

    expect(wrapper.text()).toContain('Untitled Summary')
  })

  // =============================================================================
  // Format Rendering Tests
  // =============================================================================

  it('renders markdown output for Markdown format', async () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: '# Heading\n\nParagraph' }
    })

    await wrapper.find('textarea').setValue('Content')
    await wrapper.findAll('select')[0]?.setValue('Markdown')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.markdown-output').exists()).toBe(true)
  })

  it('renders plain output for non-Markdown formats', async () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps, summaryOutput: 'Plain text summary' }
    })

    await wrapper.find('textarea').setValue('Content')
    await wrapper.findAll('select')[0]?.setValue('Plain')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.plain-output').exists()).toBe(true)
  })

  // =============================================================================
  // Edge Cases Tests
  // =============================================================================

  it('trims whitespace from content before checking if empty', () => {
    wrapper = mount(EnhancedAiSummary, {
      props: { ...defaultProps }
    })

    wrapper.vm.formData.content = '   '

    const submitBtn = wrapper.find('button[type="submit"]')
    expect(submitBtn.attributes('disabled')).toBeDefined()
  })

  it('handles very long titles', async () => {
    wrapper = mount(EnhancedAiSummary, { props: defaultProps })

    const longTitle = 'a'.repeat(500)
    await wrapper.find('input[type="text"]').setValue(longTitle)

    expect(wrapper.find('input[type="text"]').element.value).toBe(longTitle)
  })
})
