/**
 * Error handling utilities for PepTrack
 * Provides user-friendly error messages and toast notifications
 */

export interface ErrorContext {
  operation: string;
  details?: string;
  suggestion?: string;
}

/**
 * Common error types and their user-friendly messages
 */
const ERROR_MESSAGES: Record<string, (context?: ErrorContext) => { title: string; message: string; suggestion?: string }> = {
  // Network errors
  network: (context) => ({
    title: "Network Error",
    message: `Unable to complete ${context?.operation || "operation"}. Please check your internet connection.`,
    suggestion: "Try again when you're back online",
  }),

  // File errors
  file_not_found: (context) => ({
    title: "File Not Found",
    message: `The ${context?.operation || "file"} could not be found.`,
    suggestion: "Make sure the file exists and try again",
  }),

  file_read: (context) => ({
    title: "File Read Error",
    message: `Failed to read ${context?.operation || "the file"}.`,
    suggestion: "Check file permissions and try again",
  }),

  file_write: (context) => ({
    title: "File Write Error",
    message: `Failed to save ${context?.operation || "the file"}.`,
    suggestion: "Check disk space and file permissions",
  }),

  // Backup errors
  backup_failed: (context) => ({
    title: "Backup Failed",
    message: context?.details || "The backup operation encountered an error.",
    suggestion: context?.suggestion || "Check your backup destination and try again",
  }),

  restore_failed: (context) => ({
    title: "Restore Failed",
    message: context?.details || "Failed to restore data from backup.",
    suggestion: "Make sure the backup file is not corrupted",
  }),

  // Drive errors
  drive_not_connected: () => ({
    title: "Google Drive Not Connected",
    message: "You need to connect to Google Drive first.",
    suggestion: "Go to Settings > Google Drive to connect",
  }),

  drive_auth_failed: () => ({
    title: "Google Drive Authentication Failed",
    message: "Failed to authenticate with Google Drive.",
    suggestion: "Try reconnecting your Google Drive account",
  }),

  drive_upload_failed: (context) => ({
    title: "Drive Upload Failed",
    message: context?.details || "Failed to upload backup to Google Drive.",
    suggestion: "Check your internet connection and Drive permissions",
  }),

  // Token errors
  token_expired: () => ({
    title: "Session Expired",
    message: "Your Google Drive session has expired.",
    suggestion: "Please reconnect your Google Drive account",
  }),

  // Database errors
  database_error: (context) => ({
    title: "Database Error",
    message: `Failed to ${context?.operation || "access the database"}.`,
    suggestion: "Try restarting the application",
  }),

  // Validation errors
  validation_failed: (context) => ({
    title: "Validation Error",
    message: context?.details || "Some required information is missing or invalid.",
    suggestion: "Please check your input and try again",
  }),

  // Generic error
  unknown: (context) => ({
    title: "Unexpected Error",
    message: context?.details || "An unexpected error occurred.",
    suggestion: "Try again or contact support if the problem persists",
  }),
};

/**
 * Detect error type from error message
 */
function detectErrorType(error: unknown): string {
  const errorStr = String(error).toLowerCase();

  if (errorStr.includes("network") || errorStr.includes("connection")) {
    return "network";
  }
  if (errorStr.includes("not found")) {
    return "file_not_found";
  }
  if (errorStr.includes("drive") && errorStr.includes("not connected")) {
    return "drive_not_connected";
  }
  if (errorStr.includes("token") && errorStr.includes("expired")) {
    return "token_expired";
  }
  if (errorStr.includes("authentication") || errorStr.includes("auth")) {
    return "drive_auth_failed";
  }
  if (errorStr.includes("backup")) {
    return "backup_failed";
  }
  if (errorStr.includes("restore")) {
    return "restore_failed";
  }
  if (errorStr.includes("validation") || errorStr.includes("required")) {
    return "validation_failed";
  }
  if (errorStr.includes("database") || errorStr.includes("storage")) {
    return "database_error";
  }

  return "unknown";
}

/**
 * Show a user-friendly error toast
 */
export function showErrorToast(error: unknown, context?: ErrorContext) {
  const errorType = detectErrorType(error);
  const errorFn = ERROR_MESSAGES[errorType] || ERROR_MESSAGES.unknown;
  const errorInfo = errorFn!(context);

  // Add error details if available
  let message = errorInfo.message;
  if (context?.details) {
    message += ` ${context.details}`;
  }
  if (errorInfo.suggestion) {
    message += ` 💡 ${errorInfo.suggestion}`;
  }

  if (window.showToast) {
    window.showToast({
      type: "error",
      title: errorInfo.title,
      message,
      duration: 7000, // Longer duration for errors
    });
  } else {
    // Fallback to console if toast not available
    console.error(errorInfo.title, message);
  }
}

/**
 * Show a success toast
 */
export function showSuccessToast(title: string, message: string, duration = 5000) {
  if (window.showToast) {
    window.showToast({
      type: "success",
      title,
      message,
      duration,
    });
  }
}

/**
 * Show a warning toast
 */
export function showWarningToast(title: string, message: string, duration = 5000) {
  if (window.showToast) {
    window.showToast({
      type: "warning",
      title,
      message,
      duration,
    });
  }
}

/**
 * Show an info toast
 */
export function showInfoToast(title: string, message: string, duration = 5000) {
  if (window.showToast) {
    window.showToast({
      type: "info",
      title,
      message,
      duration,
    });
  }
}

/**
 * Handle async operation with error handling
 */
export async function handleAsync<T>(
  operation: () => Promise<T>,
  context: ErrorContext
): Promise<T | null> {
  try {
    return await operation();
  } catch (error) {
    showErrorToast(error, context);
    return null;
  }
}

/**
 * Extract user-friendly message from error
 */
export function getErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
