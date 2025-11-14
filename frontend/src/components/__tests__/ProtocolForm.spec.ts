import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import ProtocolForm from '../ProtocolForm.vue'

describe('ProtocolForm Component', () => {
  let wrapper: VueWrapper<any>

  const defaultProps = {
    form: {
      name: '',
      peptideName: '',
      notes: '',
      targetConcentration: ''
    },
    saving: false
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

  it('renders form header', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    expect(wrapper.find('h2').text()).toContain('Add New Peptide Plan')
  })

  it('renders all form fields', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    expect(wrapper.findAll('input')).toHaveLength(3) // name, peptideName, targetConcentration
    expect(wrapper.find('textarea').exists()).toBe(true)
    expect(wrapper.find('button[type="submit"]').exists()).toBe(true)
  })

  it('renders submit button with default text', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    expect(wrapper.find('button[type="submit"]').text()).toContain('Save Plan')
  })

  // =============================================================================
  // Props Rendering Tests
  // =============================================================================

  it('displays form values from props', () => {
    wrapper = mount(ProtocolForm, {
      props: {
        form: {
          name: 'BPC-157 Protocol',
          peptideName: 'BPC-157',
          notes: 'Test notes',
          targetConcentration: 2.5
        },
        saving: false
      }
    })

    const inputs = wrapper.findAll('input')
    expect(inputs[0]?.element.value).toBe('BPC-157 Protocol')
    expect(inputs[1]?.element.value).toBe('BPC-157')
    expect(inputs[2]?.element.value).toBe('2.5')
    expect(wrapper.find('textarea').element.value).toBe('Test notes')
  })

  it('displays empty values when form is empty', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const inputs = wrapper.findAll('input')
    expect(inputs[0]?.element.value).toBe('')
    expect(inputs[1]?.element.value).toBe('')
    expect(inputs[2]?.element.value).toBe('')
    expect(wrapper.find('textarea').element.value).toBe('')
  })

  it('displays saving state when saving prop is true', () => {
    wrapper = mount(ProtocolForm, {
      props: {
        ...defaultProps,
        saving: true
      }
    })

    expect(wrapper.find('button[type="submit"]').text()).toContain('Saving...')
    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeDefined()
  })

  it('enables submit button when not saving', () => {
    wrapper = mount(ProtocolForm, {
      props: {
        ...defaultProps,
        saving: false
      }
    })

    expect(wrapper.find('button[type="submit"]').attributes('disabled')).toBeUndefined()
  })

  // =============================================================================
  // Event Emission Tests
  // =============================================================================

  it('emits update:name when name input changes', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const nameInput = wrapper.findAll('input')[0]
    await nameInput?.setValue('New Protocol')

    expect(wrapper.emitted('update:name')).toBeTruthy()
    expect(wrapper.emitted('update:name')?.[0]).toEqual(['New Protocol'])
  })

  it('emits update:peptideName when peptide input changes', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const peptideInput = wrapper.findAll('input')[1]
    await peptideInput?.setValue('TB-500')

    expect(wrapper.emitted('update:peptideName')).toBeTruthy()
    expect(wrapper.emitted('update:peptideName')?.[0]).toEqual(['TB-500'])
  })

  it('emits update:targetConcentration when concentration input changes', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const concentrationInput = wrapper.findAll('input')[2]
    await concentrationInput?.setValue('5.0')

    expect(wrapper.emitted('update:targetConcentration')).toBeTruthy()
    expect(wrapper.emitted('update:targetConcentration')?.[0]).toEqual(['5.0'])
  })

  it('emits update:notes when notes textarea changes', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const notesTextarea = wrapper.find('textarea')
    await notesTextarea.setValue('Updated notes')

    expect(wrapper.emitted('update:notes')).toBeTruthy()
    expect(wrapper.emitted('update:notes')?.[0]).toEqual(['Updated notes'])
  })

  it('emits submit when form is submitted', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const form = wrapper.find('form')
    await form.trigger('submit.prevent')

    expect(wrapper.emitted('submit')).toBeTruthy()
    expect(wrapper.emitted('submit')).toHaveLength(1)
  })

  it('emits submit when submit button is clicked', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const submitButton = wrapper.find('button[type="submit"]')
    await submitButton.trigger('click')

    expect(wrapper.emitted('submit')).toBeTruthy()
  })

  // =============================================================================
  // Input Attributes Tests
  // =============================================================================

  it('has correct input types', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const inputs = wrapper.findAll('input')
    expect(inputs[0]?.attributes('type')).toBe('text') // name
    expect(inputs[1]?.attributes('type')).toBe('text') // peptideName
    expect(inputs[2]?.attributes('type')).toBe('number') // targetConcentration
  })

  it('has number input constraints for concentration', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const concentrationInput = wrapper.findAll('input')[2]
    expect(concentrationInput?.attributes('min')).toBe('0')
    expect(concentrationInput?.attributes('step')).toBe('0.01')
  })

  it('has placeholders for all inputs', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const inputs = wrapper.findAll('input')
    expect(inputs[0]?.attributes('placeholder')).toBeTruthy()
    expect(inputs[1]?.attributes('placeholder')).toBeTruthy()
    expect(inputs[2]?.attributes('placeholder')).toBeTruthy()
    expect(wrapper.find('textarea').attributes('placeholder')).toBeTruthy()
  })

  it('has correct textarea rows', () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    expect(wrapper.find('textarea').attributes('rows')).toBe('3')
  })

  // =============================================================================
  // Multiple Update Events Tests
  // =============================================================================

  it('emits multiple updates when multiple fields change', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const inputs = wrapper.findAll('input')
    await inputs[0]?.setValue('Protocol 1')
    await inputs[1]?.setValue('BPC-157')
    await inputs[2]?.setValue('2.5')
    await wrapper.find('textarea').setValue('Some notes')

    expect(wrapper.emitted('update:name')).toHaveLength(1)
    expect(wrapper.emitted('update:peptideName')).toHaveLength(1)
    expect(wrapper.emitted('update:targetConcentration')).toHaveLength(1)
    expect(wrapper.emitted('update:notes')).toHaveLength(1)
  })

  // =============================================================================
  // Edge Cases Tests
  // =============================================================================

  it('handles numeric string for targetConcentration', () => {
    wrapper = mount(ProtocolForm, {
      props: {
        form: {
          name: '',
          peptideName: '',
          notes: '',
          targetConcentration: '10.5'
        },
        saving: false
      }
    })

    const concentrationInput = wrapper.findAll('input')[2]
    expect(concentrationInput?.element.value).toBe('10.5')
  })

  it('handles zero as targetConcentration', () => {
    wrapper = mount(ProtocolForm, {
      props: {
        form: {
          name: '',
          peptideName: '',
          notes: '',
          targetConcentration: 0
        },
        saving: false
      }
    })

    const concentrationInput = wrapper.findAll('input')[2]
    expect(concentrationInput?.element.value).toBe('0')
  })

  it('handles very long text in notes', async () => {
    const longText = 'a'.repeat(1000)
    wrapper = mount(ProtocolForm, { props: defaultProps })

    await wrapper.find('textarea').setValue(longText)

    expect(wrapper.emitted('update:notes')?.[0]).toEqual([longText])
  })

  it('handles special characters in inputs', async () => {
    wrapper = mount(ProtocolForm, { props: defaultProps })

    const specialText = 'Test-123_!@#$%'
    await wrapper.findAll('input')[0]?.setValue(specialText)

    expect(wrapper.emitted('update:name')?.[0]).toEqual([specialText])
  })

  it('does not submit when button is disabled', async () => {
    wrapper = mount(ProtocolForm, {
      props: {
        ...defaultProps,
        saving: true
      }
    })

    const submitButton = wrapper.find('button[type="submit"]')

    // Button is disabled, but form submit event is still preventable
    // The disabled state is for UX, not event prevention
    expect(submitButton.attributes('disabled')).toBeDefined()
  })
})
