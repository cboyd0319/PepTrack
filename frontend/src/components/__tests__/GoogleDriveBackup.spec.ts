import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { mount, VueWrapper } from '@vue/test-utils'
import GoogleDriveBackup from '../GoogleDriveBackup.vue'
import * as api from '../../api/peptrack'

vi.mock('../../api/peptrack')
vi.mock('../../utils/errorHandling', () => ({
  showErrorToast: vi.fn(),
  showSuccessToast: vi.fn()
}))

import { showErrorToast, showSuccessToast } from '../../utils/errorHandling'

describe('GoogleDriveBackup Component', () => {
  let wrapper: VueWrapper<any>

  const mockDriveStatusDisconnected = {
    connected: false,
    email: null
  }

  const mockDriveStatusConnected = {
    connected: true,
    email: 'user@example.com'
  }

  const mockOAuthResponse = {
    authUrl: 'https://accounts.google.com/o/oauth2/v2/auth?...'
  }

  const mockBackupData = {
    metadata: {
      protocolsCount: 10,
      dosesCount: 50,
      literatureCount: 20
    },
    protocols: [],
    doses: [],
    literature: []
  }

  beforeEach(() => {
    vi.clearAllMocks()
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusDisconnected)
    vi.mocked(api.startDriveOAuth).mockResolvedValue(mockOAuthResponse)
    vi.mocked(api.disconnectDrive).mockResolvedValue(undefined)
    vi.mocked(api.uploadToDrive).mockResolvedValue(undefined)
    vi.mocked(api.exportBackupData).mockResolvedValue(JSON.stringify(mockBackupData))

    // Mock window.open
    global.window.open = vi.fn()

    // Use fake timers for setTimeout
    vi.useFakeTimers()
  })

  afterEach(() => {
    if (wrapper) {
      wrapper.unmount()
    }
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  // =============================================================================
  // Component Rendering Tests
  // =============================================================================

  it('renders component header', () => {
    wrapper = mount(GoogleDriveBackup)

    expect(wrapper.find('h3').text()).toContain('Google Drive')
    expect(wrapper.find('.section-description').text()).toContain('sync your backups')
  })

  // =============================================================================
  // Drive Status Loading Tests
  // =============================================================================

  it('loads drive status on mount', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()

    expect(api.checkDriveStatus).toHaveBeenCalled()
  })

  it('displays disconnected state when not connected', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.status-card.disconnected').exists()).toBe(true)
    expect(wrapper.text()).toContain('Not Connected')
  })

  it('displays connected state when connected', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.status-card.connected').exists()).toBe(true)
    expect(wrapper.text()).toContain('Connected to Google Drive')
  })

  it('displays user email when connected', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.drive-email').exists()).toBe(true)
    expect(wrapper.find('.drive-email').text()).toBe('user@example.com')
  })

  it('hides email when not provided', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue({
      connected: true,
      email: null
    })

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.drive-email').exists()).toBe(false)
  })

  // =============================================================================
  // Connect Button Tests (Disconnected State)
  // =============================================================================

  it('shows connect button when disconnected', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.connect-btn').exists()).toBe(true)
    expect(wrapper.find('.connect-btn').text()).toContain('Connect Google Drive')
  })

  it('shows privacy note when disconnected', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.privacy-note').exists()).toBe(true)
    expect(wrapper.text()).toContain('Privacy Protected')
    expect(wrapper.text()).toContain('only accesses files it creates')
  })

  it('hides backup and disconnect buttons when disconnected', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.backup-btn').exists()).toBe(false)
    expect(wrapper.find('.disconnect-btn').exists()).toBe(false)
  })

  // =============================================================================
  // Connected State Tests
  // =============================================================================

  it('shows backup and disconnect buttons when connected', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.backup-btn').exists()).toBe(true)
    expect(wrapper.find('.disconnect-btn').exists()).toBe(true)
    expect(wrapper.find('.backup-btn').text()).toContain('Backup Now')
    expect(wrapper.find('.disconnect-btn').text()).toContain('Disconnect')
  })

  it('hides connect button when connected', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.connect-btn').exists()).toBe(false)
  })

  it('hides privacy note when connected', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(wrapper.find('.privacy-note').exists()).toBe(false)
  })

  // =============================================================================
  // Connect Flow Tests
  // =============================================================================

  it('starts OAuth flow when connect button clicked', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.connect-btn').trigger('click')

    expect(api.startDriveOAuth).toHaveBeenCalledWith({
      clientId: 'YOUR_CLIENT_ID_HERE.apps.googleusercontent.com',
      clientSecret: 'YOUR_CLIENT_SECRET_HERE'
    })
  })

  it('opens browser window with auth URL', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.connect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(window.open).toHaveBeenCalledWith(mockOAuthResponse.authUrl, '_blank')
  })

  it('shows success toast after opening browser', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.connect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith(
      'Opening Browser',
      expect.stringContaining('complete the authorization')
    )
  })

  it('checks status after 3 second delay', async () => {
    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('.connect-btn').trigger('click')

    await wrapper.vm.$nextTick()

    // Should not be called yet
    expect(api.checkDriveStatus).not.toHaveBeenCalled()

    // Advance timer by 3 seconds
    vi.advanceTimersByTime(3000)

    await wrapper.vm.$nextTick()

    // Now should be called
    expect(api.checkDriveStatus).toHaveBeenCalled()
  })

  it('sets loading state during connect', async () => {
    let resolveOAuth: any
    vi.mocked(api.startDriveOAuth).mockReturnValue(new Promise(resolve => { resolveOAuth = resolve }))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.connect-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.connect-btn').text()).toContain('Connecting...')
    expect(wrapper.find('.connect-btn').attributes('disabled')).toBeDefined()

    resolveOAuth(mockOAuthResponse)
  })

  it('handles connect errors', async () => {
    vi.mocked(api.startDriveOAuth).mockRejectedValue(new Error('OAuth failed'))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.connect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Disconnect Flow Tests
  // =============================================================================

  it('disconnects when disconnect button clicked', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.disconnect-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.disconnectDrive).toHaveBeenCalled()
  })

  it('shows success toast after disconnect', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.disconnect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith(
      'Disconnected',
      expect.stringContaining('successfully')
    )
  })

  it('reloads status after disconnect', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    vi.clearAllMocks()

    await wrapper.find('.disconnect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.checkDriveStatus).toHaveBeenCalled()
  })

  it('sets loading state during disconnect', async () => {
    let resolveDisconnect: any
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.disconnectDrive).mockReturnValue(new Promise(resolve => { resolveDisconnect = resolve }))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.disconnect-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.vm.loading).toBe(true)
    expect(wrapper.find('.disconnect-btn').attributes('disabled')).toBeDefined()
    expect(wrapper.find('.backup-btn').attributes('disabled')).toBeDefined()

    resolveDisconnect(undefined)
  })

  it('handles disconnect errors', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.disconnectDrive).mockRejectedValue(new Error('Disconnect failed'))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.disconnect-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Backup to Drive Tests
  // =============================================================================

  it('exports backup data when backup button clicked', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()

    expect(api.exportBackupData).toHaveBeenCalled()
  })

  it('uploads to drive with correct filename and data', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    // Mock date for predictable filename
    const mockDate = new Date('2024-03-15T10:30:00Z')
    vi.setSystemTime(mockDate)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.uploadToDrive).toHaveBeenCalledWith(
      'peptrack_backup_2024-03-15_10-30.json',
      JSON.stringify(mockBackupData)
    )
  })

  it('shows success toast with backup counts', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith(
      'Backup Uploaded',
      expect.stringContaining('10 protocols')
    )
    expect(showSuccessToast).toHaveBeenCalledWith(
      'Backup Uploaded',
      expect.stringContaining('50 doses')
    )
    expect(showSuccessToast).toHaveBeenCalledWith(
      'Backup Uploaded',
      expect.stringContaining('20 papers')
    )
  })

  it('handles backup data without metadata', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.exportBackupData).mockResolvedValue(JSON.stringify({
      protocolsCount: 5,
      dosesCount: 15,
      literatureCount: 3
    }))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith(
      'Backup Uploaded',
      expect.stringContaining('5 protocols')
    )
  })

  it('handles backup data with missing counts', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.exportBackupData).mockResolvedValue(JSON.stringify({
      metadata: {}
    }))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showSuccessToast).toHaveBeenCalledWith(
      'Backup Uploaded',
      expect.stringContaining('0 protocols')
    )
  })

  it('sets loading state during backup', async () => {
    let resolveExport: any
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.exportBackupData).mockReturnValue(new Promise(resolve => { resolveExport = resolve }))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.backup-btn').text()).toContain('Uploading...')
    expect(wrapper.find('.backup-btn').attributes('disabled')).toBeDefined()
    expect(wrapper.find('.disconnect-btn').attributes('disabled')).toBeDefined()

    resolveExport(JSON.stringify(mockBackupData))
  })

  it('handles backup errors', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.exportBackupData).mockRejectedValue(new Error('Export failed'))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  it('handles upload errors', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)
    vi.mocked(api.uploadToDrive).mockRejectedValue(new Error('Upload failed'))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    await wrapper.find('.backup-btn').trigger('click')

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Status Check Error Handling Tests
  // =============================================================================

  it('handles status check errors on mount', async () => {
    vi.mocked(api.checkDriveStatus).mockRejectedValue(new Error('Network error'))

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(showErrorToast).toHaveBeenCalled()
  })

  // =============================================================================
  // Edge Cases Tests
  // =============================================================================

  it('disables all buttons when loading', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    wrapper.vm.loading = true
    await wrapper.vm.$nextTick()

    expect(wrapper.find('.backup-btn').attributes('disabled')).toBeDefined()
    expect(wrapper.find('.disconnect-btn').attributes('disabled')).toBeDefined()
  })

  it('generates unique filenames for each backup', async () => {
    vi.mocked(api.checkDriveStatus).mockResolvedValue(mockDriveStatusConnected)

    wrapper = mount(GoogleDriveBackup)

    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    // First backup
    vi.setSystemTime(new Date('2024-03-15T10:30:00Z'))
    await wrapper.find('.backup-btn').trigger('click')
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.uploadToDrive).toHaveBeenCalledWith(
      'peptrack_backup_2024-03-15_10-30.json',
      expect.any(String)
    )

    // Second backup with different time
    vi.setSystemTime(new Date('2024-03-15T14:45:00Z'))
    await wrapper.find('.backup-btn').trigger('click')
    await wrapper.vm.$nextTick()
    await new Promise(resolve => setTimeout(resolve, 0))

    expect(api.uploadToDrive).toHaveBeenCalledWith(
      'peptrack_backup_2024-03-15_14-45.json',
      expect.any(String)
    )
  })
})
