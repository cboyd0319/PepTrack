/**
 * Unified Notification System
 * Supports both in-app toasts and desktop OS notifications
 */

import {
  isPermissionGranted,
  requestPermission,
  sendNotification,
} from '@tauri-apps/plugin-notification';

export interface NotificationOptions {
  title: string;
  body: string;
  icon?: string;
  /** If true, will only show in-app toast, not desktop notification */
  toastOnly?: boolean;
  /** Toast type for in-app notifications */
  toastType?: 'success' | 'error' | 'warning' | 'info';
}

let permissionGranted: boolean | null = null;

/**
 * Initialize the notification system by requesting permissions if needed
 */
export async function initializeNotifications(): Promise<boolean> {
  try {
    // Check if permission already granted
    permissionGranted = await isPermissionGranted();

    if (!permissionGranted) {
      // Request permission
      const permission = await requestPermission();
      permissionGranted = permission === 'granted';
    }

    return permissionGranted;
  } catch (error) {
    console.warn('Failed to initialize notifications:', error);
    return false;
  }
}

/**
 * Show a notification (desktop + optional toast)
 */
export async function showNotification(options: NotificationOptions): Promise<void> {
  // Always show toast if specified or if desktop notification fails
  const showToastFn = (window as any).showToast;

  if (options.toastOnly || !permissionGranted) {
    // Only show in-app toast
    if (showToastFn && options.toastType) {
      showToastFn(options.body, options.toastType);
    }
    return;
  }

  try {
    // Show desktop notification
    await sendNotification({
      title: options.title,
      body: options.body,
      icon: options.icon,
    });

    // Also show toast for immediate feedback
    if (showToastFn && options.toastType) {
      showToastFn(options.body, options.toastType);
    }
  } catch (error) {
    console.warn('Desktop notification failed, falling back to toast:', error);
    if (showToastFn) {
      showToastFn(options.body, options.toastType || 'info');
    }
  }
}

/**
 * Show success notification
 */
export async function notifySuccess(title: string, body: string, toastOnly = false): Promise<void> {
  await showNotification({
    title,
    body,
    toastOnly,
    toastType: 'success',
  });
}

/**
 * Show error notification
 */
export async function notifyError(title: string, body: string, toastOnly = false): Promise<void> {
  await showNotification({
    title,
    body,
    toastOnly,
    toastType: 'error',
  });
}

/**
 * Show warning notification
 */
export async function notifyWarning(title: string, body: string, toastOnly = false): Promise<void> {
  await showNotification({
    title,
    body,
    toastOnly,
    toastType: 'warning',
  });
}

/**
 * Show info notification
 */
export async function notifyInfo(title: string, body: string, toastOnly = false): Promise<void> {
  await showNotification({
    title,
    body,
    toastOnly,
    toastType: 'info',
  });
}

/**
 * Notification presets for common use cases
 */
export const NotificationPresets = {
  /** Dose reminder notification */
  doseReminder: (peptideName: string, time: string, amountMg?: number) => ({
    title: '💉 Dose Reminder',
    body: `Time for your ${peptideName} dose${amountMg ? ` (${amountMg}mg)` : ''} - scheduled for ${time}`,
    toastType: 'info' as const,
  }),

  /** Backup completed successfully */
  backupSuccess: () => ({
    title: '✅ Backup Complete',
    body: 'Your data has been backed up successfully',
    toastType: 'success' as const,
  }),

  /** Backup failed */
  backupFailed: (reason: string) => ({
    title: '❌ Backup Failed',
    body: `Backup failed: ${reason}`,
    toastType: 'error' as const,
  }),

  /** Vial expiring soon */
  vialExpiring: (peptideName: string, days: number) => ({
    title: '⚠️ Vial Expiring Soon',
    body: `Your ${peptideName} vial expires in ${days} day${days !== 1 ? 's' : ''}`,
    toastType: 'warning' as const,
  }),

  /** Low stock alert */
  lowStock: (peptideName: string, remaining: number) => ({
    title: '📉 Low Stock Alert',
    body: `Only ${remaining}mg of ${peptideName} remaining`,
    toastType: 'warning' as const,
  }),

  /** Price change detected */
  priceChange: (peptideName: string, change: string) => ({
    title: '💰 Price Change',
    body: `${peptideName} price ${change}`,
    toastType: 'info' as const,
  }),
};
