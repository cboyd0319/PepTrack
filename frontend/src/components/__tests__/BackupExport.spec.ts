import { describe, it, expect, vi, beforeEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import BackupExport from '../BackupExport.vue'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')

describe('BackupExport Component', () => {
  let wrapper: VueWrapper<any>

  const mockBackupData = JSON.stringify({
    metadata: {
      protocolsCount: 5,
      dosesCount: 20,
      literatureCount: 10,
      timestamp: new Date().toISOString()
    },
    protocols: [],
    doses: [],
    literature: []
  })

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.exportBackupData).mockResolvedValue(mockBackupData)

    // Mock DOM APIs
    global.URL.createObjectURL = vi.fn(() => 'blob:test-url')
    global.URL.revokeObjectURL = vi.fn()

    // Mock document methods
    const mockLink = {
      click: vi.fn(),
      download: '',
      href: ''
    } as any
    vi.spyOn(document, 'createElement').mockReturnValue(mockLink)
    vi.spyOn(document.body, 'appendChild').mockImplementation(() => mockLink)
    vi.spyOn(document.body, 'removeChild').mockImplementation(() => mockLink)
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    vi.restoreAllMocks()
  })

  // =============================================================================
  // Component Rendering Tests
  // =============================================================================

  it('renders component header', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('h2').text()).toContain('Backup Your Data')
    expect(wrapper.find('.section-description').text()).toContain('Save all your protocols')
  })

  it('renders backup info section', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('.backup-info').exists()).toBe(true)
    expect(wrapper.text()).toContain('What gets backed up')
    expect(wrapper.text()).toContain('All your protocols')
    expect(wrapper.text()).toContain('All dose logs')
    expect(wrapper.text()).toContain('All saved research papers')
  })

  it('renders encryption checkbox', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('.encryption-checkbox input').exists()).toBe(true)
    expect(wrapper.find('.encryption-checkbox span').text()).toContain('Encrypt backup')
  })

  it('renders export button', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('.export-btn').exists()).toBe(true)
    expect(wrapper.find('.export-btn').text()).toContain('Export Backup Now')
  })

  // =============================================================================
  // Encryption UI Tests
  // =============================================================================

  it('hides password inputs when encryption disabled', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('.password-inputs').exists()).toBe(false)
  })

  it('shows password inputs when encryption enabled', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)

    expect(wrapper.find('.password-inputs').exists()).toBe(true)
    expect(wrapper.find('#password').exists()).toBe(true)
    expect(wrapper.find('#confirmPassword').exists()).toBe(true)
  })

  it('shows critical warning when encryption enabled', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)

    expect(wrapper.find('.critical-warning').exists()).toBe(true)
    expect(wrapper.text()).toContain('NO PASSWORD RECOVERY POSSIBLE')
    expect(wrapper.text()).toContain('permanently unrecoverable')
  })

  // =============================================================================
  // Export Without Encryption Tests
  // =============================================================================

  it('exports backup without encryption', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.exportBackupData).toHaveBeenCalledWith(undefined)
  })

  it('creates blob and triggers download', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const mockLink = document.createElement('a')
    expect(mockLink.click).toHaveBeenCalled()
    expect(URL.createObjectURL).toHaveBeenCalled()
    expect(URL.revokeObjectURL).toHaveBeenCalled()
  })

  it('sets correct filename without encryption', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const mockLink = document.createElement('a')
    expect(mockLink.download).toMatch(/peptrack_backup_\d{4}-\d{2}-\d{2}_\d{2}-\d{2}\.json/)
    expect(mockLink.download).not.toContain('_encrypted')
  })

  it('shows success message after export', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.message.success').exists()).toBe(true)
    expect(wrapper.find('.message.success').text()).toContain('downloaded successfully')
    expect(wrapper.find('.message.success').text()).toContain('5 protocols')
    expect(wrapper.find('.message.success').text()).toContain('20 doses')
    expect(wrapper.find('.message.success').text()).toContain('10 papers')
  })

  // =============================================================================
  // Export With Encryption Tests
  // =============================================================================

  it('exports backup with encryption when password provided', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('securepassword123')
    await wrapper.find('#confirmPassword').setValue('securepassword123')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.exportBackupData).toHaveBeenCalledWith('securepassword123')
  })

  it('sets correct filename with encryption', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('securepassword123')
    await wrapper.find('#confirmPassword').setValue('securepassword123')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    const mockLink = document.createElement('a')
    expect(mockLink.download).toContain('_encrypted')
  })

  it('shows encrypted note in success message', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('securepassword123')
    await wrapper.find('#confirmPassword').setValue('securepassword123')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.message.success').text()).toContain('🔒 (encrypted)')
  })

  it('clears password fields after successful export', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('securepassword123')
    await wrapper.find('#confirmPassword').setValue('securepassword123')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.vm.password).toBe('')
    expect(wrapper.vm.confirmPassword).toBe('')
  })

  // =============================================================================
  // Password Validation Tests
  // =============================================================================

  it('shows error when password is empty', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.message.error').exists()).toBe(true)
    expect(wrapper.find('.message.error').text()).toContain('Please enter a password')
    expect(api.exportBackupData).not.toHaveBeenCalled()
  })

  it('shows error when passwords do not match', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('password123')
    await wrapper.find('#confirmPassword').setValue('different123')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.message.error').text()).toContain('Passwords do not match')
    expect(api.exportBackupData).not.toHaveBeenCalled()
  })

  it('shows error when password is too short', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('short')
    await wrapper.find('#confirmPassword').setValue('short')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(wrapper.find('.message.error').text()).toContain('at least 8 characters')
    expect(api.exportBackupData).not.toHaveBeenCalled()
  })

  it('accepts password with exactly 8 characters', async () => {
    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('#password').setValue('12345678')
    await wrapper.find('#confirmPassword').setValue('12345678')
    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.exportBackupData).toHaveBeenCalled()
  })

  // =============================================================================
  // Loading State Tests
  // =============================================================================

  it('shows loading state while exporting', async () => {
    let resolveExport: any
    vi.mocked(api.exportBackupData).mockReturnValue(new Promise(resolve => { resolveExport = resolve }))

    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.export-btn').text()).toContain('Creating Backup...')
    expect(wrapper.find('.export-btn').attributes('disabled')).toBeDefined()

    resolveExport(mockBackupData)
  })

  it('disables encryption checkbox while exporting', async () => {
    let resolveExport: any
    vi.mocked(api.exportBackupData).mockReturnValue(new Promise(resolve => { resolveExport = resolve }))

    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.encryption-checkbox input').attributes('disabled')).toBeDefined()

    resolveExport(mockBackupData)
  })

  it('disables password inputs while exporting', async () => {
    let resolveExport: any
    vi.mocked(api.exportBackupData).mockReturnValue(new Promise(resolve => { resolveExport = resolve }))

    wrapper = mount(BackupExport)

    await wrapper.find('.encryption-checkbox input').setValue(true)
    await wrapper.find('.export-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('#password').attributes('disabled')).toBeDefined()
    expect(wrapper.find('#confirmPassword').attributes('disabled')).toBeDefined()

    resolveExport(mockBackupData)
  })

  // =============================================================================
  // Error Handling Tests
  // =============================================================================

  it('shows error message when export fails', async () => {
    vi.mocked(api.exportBackupData).mockRejectedValue(new Error('Network error'))

    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.message.error').exists()).toBe(true)
    expect(wrapper.find('.message.error').text()).toContain('Failed to export backup')
    expect(wrapper.find('.message.error').text()).toContain('Network error')
  })

  it('clears previous messages when starting new export', async () => {
    wrapper = mount(BackupExport)

    wrapper.vm.exportMessage = 'Previous success'
    wrapper.vm.exportError = 'Previous error'

    await wrapper.find('.export-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.exportMessage).toBeNull()
    expect(wrapper.vm.exportError).toBeNull()
  })

  // =============================================================================
  // Accessibility Tests
  // =============================================================================

  it('has proper aria labels on export button', () => {
    wrapper = mount(BackupExport)

    expect(wrapper.find('.export-btn').attributes('aria-label')).toBe('Export backup data to file')
  })

  it('sets aria-busy during export', async () => {
    let resolveExport: any
    vi.mocked(api.exportBackupData).mockReturnValue(new Promise(resolve => { resolveExport = resolve }))

    wrapper = mount(BackupExport)

    await wrapper.find('.export-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.export-btn').attributes('aria-busy')).toBe('true')

    resolveExport(mockBackupData)
  })
})
