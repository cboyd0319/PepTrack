import { ref, onUnmounted } from 'vue';
import { getPendingDoseReminders, type DoseSchedule } from '../api/peptrack';
import { showNotification, NotificationPresets } from '../utils/notifications';

export interface ReminderServiceConfig {
  checkIntervalMinutes?: number; // How often to check for reminders (default: 5 minutes)
  enabled?: boolean; // Whether the service is enabled (default: true)
  debug?: boolean; // Enable debug logging (default: false)
}

const DEBUG = import.meta.env.DEV; // Only log in development mode

function debugLog(...args: any[]) {
  if (DEBUG) {
    console.log('[ReminderService]', ...args);
  }
}

export function useReminderService(config: ReminderServiceConfig = {}) {
  const {
    checkIntervalMinutes = 5,
    enabled = true,
  } = config;

  const isRunning = ref(false);
  const lastCheckTime = ref<Date | null>(null);
  const notifiedSchedules = ref<Set<string>>(new Set()); // Track which schedules we've already notified for
  let intervalId: number | null = null;

  async function checkReminders() {
    if (!enabled || !isRunning.value) {
      return;
    }

    try {
      const pendingReminders = await getPendingDoseReminders();

      if (pendingReminders.length > 0) {
        debugLog(`Found ${pendingReminders.length} pending reminders`);

        for (const reminder of pendingReminders) {
          // Check if we've already notified for this schedule in this time window
          const notificationKey = `${reminder.id}-${reminder.timeOfDay}`;

          if (!notifiedSchedules.value.has(notificationKey)) {
            // Send notification
            await sendReminderNotification(reminder);

            // Mark as notified
            notifiedSchedules.value.add(notificationKey);

            // Clean up old notifications after 1 hour
            setTimeout(() => {
              notifiedSchedules.value.delete(notificationKey);
            }, 60 * 60 * 1000);
          }
        }
      }

      lastCheckTime.value = new Date();
    } catch (error) {
      console.error('[ReminderService] Error checking reminders:', error);
      // Don't show error toast for background checks to avoid annoying the user
    }
  }

  async function sendReminderNotification(schedule: DoseSchedule) {
    try {
      const notification = NotificationPresets.doseReminder(
        schedule.peptideName,
        schedule.timeOfDay,
        schedule.amountMg
      );

      await showNotification(notification);

      debugLog(`Sent notification for ${schedule.protocolName}`);
    } catch (error) {
      console.error('[ReminderService] Error sending notification:', error);
    }
  }

  function start() {
    if (isRunning.value) {
      console.warn('[ReminderService] Service already running');
      return;
    }

    if (!enabled) {
      debugLog('Service is disabled');
      return;
    }

    debugLog(`Starting reminder service (checking every ${checkIntervalMinutes} minutes)`);
    isRunning.value = true;

    // Check immediately on start
    checkReminders();

    // Then check periodically
    intervalId = window.setInterval(() => {
      checkReminders();
    }, checkIntervalMinutes * 60 * 1000);
  }

  function stop() {
    if (!isRunning.value) {
      return;
    }

    debugLog('Stopping reminder service');
    isRunning.value = false;

    if (intervalId !== null) {
      clearInterval(intervalId);
      intervalId = null;
    }

    // Clear notified schedules
    notifiedSchedules.value.clear();
  }

  function restart() {
    stop();
    start();
  }

  // Clean up on component unmount
  onUnmounted(() => {
    stop();
  });

  return {
    isRunning,
    lastCheckTime,
    start,
    stop,
    restart,
    checkReminders, // Expose for manual checks
  };
}
