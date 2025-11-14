import { describe, it, expect, vi, beforeEach, afterEach } from 'vitest'
import { useReminderService } from '../useReminderService'
import * as api from '../../api/peptrack'
import * as notifications from '../../utils/notifications'

vi.mock('../../api/peptrack')
vi.mock('../../utils/notifications')

describe('useReminderService Composable', () => {
  beforeEach(() => {
    vi.clearAllMocks()
    vi.useFakeTimers()
  })

  afterEach(() => {
    vi.restoreAllMocks()
    vi.useRealTimers()
  })

  // =============================================================================
  // Initial State Tests
  // =============================================================================

  it('initializes with correct default state', () => {
    const service = useReminderService()

    expect(service.isRunning.value).toBe(false)
    expect(service.lastCheckTime.value).toBeNull()
  })

  it('accepts custom configuration', () => {
    const service = useReminderService({
      checkIntervalMinutes: 10,
      enabled: true
    })

    expect(service.isRunning.value).toBe(false)
  })

  it('respects enabled:false in config', () => {
    const service = useReminderService({ enabled: false })

    service.start()

    expect(service.isRunning.value).toBe(false)
  })

  // =============================================================================
  // Start/Stop/Restart Tests
  // =============================================================================

  it('start() begins the service', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()

    expect(service.isRunning.value).toBe(true)

    // Wait for initial check
    await vi.runAllTimersAsync()
  })

  it('start() performs immediate check on start', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()

    await vi.runAllTimersAsync()

    expect(api.getPendingDoseReminders).toHaveBeenCalled()
  })

  it('start() sets up periodic interval', async () => {
    const service = useReminderService({ checkIntervalMinutes: 5 })

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()

    await vi.runAllTimersAsync()

    // Initial call
    expect(api.getPendingDoseReminders).toHaveBeenCalledTimes(1)

    // Advance 5 minutes
    vi.advanceTimersByTime(5 * 60 * 1000)
    await vi.runAllTimersAsync()

    // Should be called again
    expect(api.getPendingDoseReminders).toHaveBeenCalledTimes(2)
  })

  it('start() does nothing if already running', async () => {
    const service = useReminderService()
    const consoleSpy = vi.spyOn(console, 'warn').mockImplementation(() => {})

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()
    await vi.runAllTimersAsync()

    const firstCallCount = vi.mocked(api.getPendingDoseReminders).mock.calls.length

    service.start()
    await vi.runAllTimersAsync()

    expect(consoleSpy).toHaveBeenCalledWith(expect.stringContaining('already running'))
    expect(vi.mocked(api.getPendingDoseReminders).mock.calls.length).toBe(firstCallCount)

    consoleSpy.mockRestore()
  })

  it('stop() halts the service', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()
    await vi.runAllTimersAsync()

    service.stop()

    expect(service.isRunning.value).toBe(false)
  })

  it('stop() clears the interval', async () => {
    const service = useReminderService({ checkIntervalMinutes: 5 })

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()
    await vi.runAllTimersAsync()

    const callCountAfterStart = vi.mocked(api.getPendingDoseReminders).mock.calls.length

    service.stop()

    // Advance time - should NOT trigger another check
    vi.advanceTimersByTime(5 * 60 * 1000)
    await vi.runAllTimersAsync()

    expect(vi.mocked(api.getPendingDoseReminders).mock.calls.length).toBe(callCountAfterStart)
  })

  it('stop() does nothing if not running', () => {
    const service = useReminderService()

    // Should not throw
    expect(() => service.stop()).not.toThrow()
  })

  it('restart() stops and starts the service', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()
    await vi.runAllTimersAsync()

    service.restart()

    expect(service.isRunning.value).toBe(true)
  })

  // =============================================================================
  // Reminder Checking Tests
  // =============================================================================

  it('checkReminders fetches pending reminders', async () => {
    const service = useReminderService()

    const mockReminders = [
      {
        id: '1',
        protocolId: 'p1',
        protocolName: 'BPC-157 Protocol',
        peptideName: 'BPC-157',
        timeOfDay: 'morning',
        amountMg: 2.5,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue(mockReminders)
    vi.mocked(notifications.NotificationPresets.doseReminder).mockReturnValue({
      title: 'Dose Reminder',
      body: 'Time for BPC-157',
      tag: 'dose-reminder-1',
      requireInteraction: true
    })
    vi.mocked(notifications.showNotification).mockResolvedValue(undefined)

    service.start()
    await vi.runAllTimersAsync()

    expect(api.getPendingDoseReminders).toHaveBeenCalled()
    expect(notifications.showNotification).toHaveBeenCalled()
  })

  it('checkReminders sends notification for each reminder', async () => {
    const service = useReminderService()

    const mockReminders = [
      {
        id: '1',
        protocolId: 'p1',
        protocolName: 'BPC-157',
        peptideName: 'BPC-157',
        timeOfDay: 'morning',
        amountMg: 2.5,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      },
      {
        id: '2',
        protocolId: 'p2',
        protocolName: 'TB-500',
        peptideName: 'TB-500',
        timeOfDay: 'evening',
        amountMg: 3.0,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue(mockReminders)
    vi.mocked(notifications.NotificationPresets.doseReminder).mockReturnValue({
      title: 'Dose Reminder',
      body: 'Test',
      tag: 'test',
      requireInteraction: true
    })
    vi.mocked(notifications.showNotification).mockResolvedValue(undefined)

    service.start()
    await vi.runAllTimersAsync()

    expect(notifications.showNotification).toHaveBeenCalledTimes(2)
  })

  it('checkReminders does not duplicate notifications', async () => {
    const service = useReminderService({ checkIntervalMinutes: 1 })

    const mockReminders = [
      {
        id: '1',
        protocolId: 'p1',
        protocolName: 'BPC-157',
        peptideName: 'BPC-157',
        timeOfDay: 'morning',
        amountMg: 2.5,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue(mockReminders)
    vi.mocked(notifications.NotificationPresets.doseReminder).mockReturnValue({
      title: 'Dose Reminder',
      body: 'Test',
      tag: 'test',
      requireInteraction: true
    })
    vi.mocked(notifications.showNotification).mockResolvedValue(undefined)

    service.start()
    await vi.runAllTimersAsync()

    // First check - should send notification
    expect(notifications.showNotification).toHaveBeenCalledTimes(1)

    // Advance 1 minute - should check again but NOT send duplicate
    vi.advanceTimersByTime(1 * 60 * 1000)
    await vi.runAllTimersAsync()

    // Should still be 1 (not duplicated)
    expect(notifications.showNotification).toHaveBeenCalledTimes(1)
  })

  it('checkReminders updates lastCheckTime', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()

    expect(service.lastCheckTime.value).toBeNull()

    await vi.runAllTimersAsync()

    expect(service.lastCheckTime.value).toBeInstanceOf(Date)
  })

  it('checkReminders does nothing when not running', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    await service.checkReminders()

    expect(api.getPendingDoseReminders).not.toHaveBeenCalled()
  })

  it('checkReminders does nothing when disabled', async () => {
    const service = useReminderService({ enabled: false })

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    await service.checkReminders()

    expect(api.getPendingDoseReminders).not.toHaveBeenCalled()
  })

  it('checkReminders handles API errors gracefully', async () => {
    const service = useReminderService()
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    vi.mocked(api.getPendingDoseReminders).mockRejectedValue(new Error('API error'))

    service.start()
    await vi.runAllTimersAsync()

    // Should log error but not crash
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('Error checking reminders'),
      expect.any(Error)
    )

    consoleSpy.mockRestore()
  })

  it('checkReminders handles notification errors gracefully', async () => {
    const service = useReminderService()
    const consoleSpy = vi.spyOn(console, 'error').mockImplementation(() => {})

    const mockReminders = [
      {
        id: '1',
        protocolId: 'p1',
        protocolName: 'BPC-157',
        peptideName: 'BPC-157',
        timeOfDay: 'morning',
        amountMg: 2.5,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue(mockReminders)
    vi.mocked(notifications.NotificationPresets.doseReminder).mockReturnValue({
      title: 'Test',
      body: 'Test',
      tag: 'test',
      requireInteraction: true
    })
    vi.mocked(notifications.showNotification).mockRejectedValue(new Error('Notification error'))

    service.start()
    await vi.runAllTimersAsync()

    // Should log error but not crash
    expect(consoleSpy).toHaveBeenCalledWith(
      expect.stringContaining('Error sending notification'),
      expect.any(Error)
    )

    consoleSpy.mockRestore()
  })

  // =============================================================================
  // Cleanup Tests
  // =============================================================================

  it('cleans up notification keys after 1 hour', async () => {
    const service = useReminderService({ checkIntervalMinutes: 1 })

    const mockReminders = [
      {
        id: '1',
        protocolId: 'p1',
        protocolName: 'BPC-157',
        peptideName: 'BPC-157',
        timeOfDay: 'morning',
        amountMg: 2.5,
        enabled: true,
        created_at: new Date().toISOString(),
        updated_at: new Date().toISOString()
      }
    ]

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue(mockReminders)
    vi.mocked(notifications.NotificationPresets.doseReminder).mockReturnValue({
      title: 'Test',
      body: 'Test',
      tag: 'test',
      requireInteraction: true
    })
    vi.mocked(notifications.showNotification).mockResolvedValue(undefined)

    service.start()
    await vi.runAllTimersAsync()

    // First notification sent
    expect(notifications.showNotification).toHaveBeenCalledTimes(1)

    // Advance 30 minutes - should NOT send duplicate
    vi.advanceTimersByTime(30 * 60 * 1000)
    await vi.runAllTimersAsync()

    expect(notifications.showNotification).toHaveBeenCalledTimes(1)

    // Advance 31 more minutes (total 61 minutes) - key should be cleaned up, notification sent again
    vi.advanceTimersByTime(31 * 60 * 1000)
    await vi.runAllTimersAsync()

    expect(notifications.showNotification).toHaveBeenCalledTimes(2)
  })

  // =============================================================================
  // Manual Check Tests
  // =============================================================================

  it('allows manual checkReminders call', async () => {
    const service = useReminderService()

    vi.mocked(api.getPendingDoseReminders).mockResolvedValue([])

    service.start()
    await vi.runAllTimersAsync()

    const callCountAfterStart = vi.mocked(api.getPendingDoseReminders).mock.calls.length

    await service.checkReminders()

    expect(vi.mocked(api.getPendingDoseReminders).mock.calls.length).toBe(callCountAfterStart + 1)
  })
})
