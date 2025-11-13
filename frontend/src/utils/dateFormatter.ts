/**
 * Parse Rust time array format: [year, ordinal_day, hour, minute, second, nanosecond, ...]
 */
function parseRustTimeArray(arr: any): Date | null {
  try {
    // Unwrap Vue reactive proxy if needed by converting to plain array
    let unwrapped: number[];
    if (Array.isArray(arr)) {
      unwrapped = [...arr];
    } else if (typeof arr === 'object' && arr !== null && 'length' in arr) {
      // It's an array-like object (possibly a Proxy), convert to array
      unwrapped = Array.from({ length: arr.length }, (_, i) => arr[i]);
    } else {
      return null;
    }

    if (!Array.isArray(unwrapped) || unwrapped.length < 6) return null;

    const [year, ordinal, hour, minute, second, nanosecond] = unwrapped;

    // Validate the values
    if (typeof year !== 'number' || typeof ordinal !== 'number' ||
        typeof hour !== 'number' || typeof minute !== 'number' ||
        typeof second !== 'number' || typeof nanosecond !== 'number') {
      return null;
    }

    // Convert ordinal day to month/day
    const date = new Date(Date.UTC(year, 0, ordinal));
    date.setUTCHours(hour, minute, second, Math.floor(nanosecond / 1000000));

    // Verify the date is valid
    if (isNaN(date.getTime())) {
      return null;
    }

    return date;
  } catch (e) {
    console.warn('Error parsing Rust time array:', e, arr);
    return null;
  }
}

/**
 * Safely format dates that may come from Rust as OffsetDateTime, Date objects, or strings
 */
export function formatDate(dateValue: any): string {
  if (!dateValue) return "Unknown";

  // Handle Date objects directly
  if (dateValue instanceof Date) {
    return dateValue.toLocaleString();
  }

  // Handle Rust time array format [year, ordinal, hour, min, sec, nano, ...]
  // Check if it looks like a time array (first element is a year-like number)
  if ((Array.isArray(dateValue) || (typeof dateValue === 'object' && dateValue !== null && 'length' in dateValue))
      && dateValue.length >= 6
      && typeof dateValue[0] === 'number'
      && dateValue[0] > 2000 && dateValue[0] < 2100) {
    const date = parseRustTimeArray(dateValue);
    if (date && !isNaN(date.getTime())) {
      return date.toLocaleString();
    }
  }

  // Handle OffsetDateTime objects from Rust (has secs_since_epoch)
  if (typeof dateValue === 'object' && 'secs_since_epoch' in dateValue) {
    return new Date(dateValue.secs_since_epoch * 1000).toLocaleString();
  }

  // Handle string dates
  if (typeof dateValue !== 'string') {
    console.warn('Unexpected date format:', dateValue);
    return "Invalid Date";
  }

  const normalized = dateValue.replace(" ", "T");
  const parsed = Date.parse(normalized);

  if (!Number.isNaN(parsed)) {
    return new Date(parsed).toLocaleString();
  }

  // Some drivers return microsecond precision like 2025-11-12T01:15:00.123456+00:00
  const truncated = normalized.split(".")[0];
  const retry = Date.parse(`${truncated}Z`);
  if (!Number.isNaN(retry)) {
    return new Date(retry).toLocaleString();
  }

  return dateValue;
}

/**
 * Convert any date value to ISO string
 */
export function toISOString(dateValue: any): string {
  if (!dateValue) return new Date().toISOString();

  // Handle Date objects
  if (dateValue instanceof Date) {
    return dateValue.toISOString();
  }

  // Handle Rust time array format (including Vue Proxy wrapped arrays)
  if ((Array.isArray(dateValue) || (typeof dateValue === 'object' && dateValue !== null && 'length' in dateValue))
      && dateValue.length >= 6
      && typeof dateValue[0] === 'number'
      && dateValue[0] > 2000 && dateValue[0] < 2100) {
    const date = parseRustTimeArray(dateValue);
    if (date && !isNaN(date.getTime())) {
      return date.toISOString();
    }
  }

  // Handle OffsetDateTime from Rust
  if (typeof dateValue === 'object' && 'secs_since_epoch' in dateValue) {
    return new Date(dateValue.secs_since_epoch * 1000).toISOString();
  }

  // Handle strings
  if (typeof dateValue === 'string') {
    const normalized = dateValue.replace(" ", "T");
    const date = new Date(normalized);
    if (!isNaN(date.getTime())) {
      return date.toISOString();
    }
  }

  console.warn('Could not convert to ISO string:', dateValue);
  return new Date().toISOString();
}

/**
 * Get date string for calendar (YYYY-MM-DD)
 */
export function toDateString(dateValue: any): string {
  if (!dateValue) return new Date().toISOString().split('T')[0];

  // Handle Date objects
  if (dateValue instanceof Date) {
    return dateValue.toISOString().split('T')[0];
  }

  // Handle Rust time array format (including Vue Proxy wrapped arrays)
  if ((Array.isArray(dateValue) || (typeof dateValue === 'object' && dateValue !== null && 'length' in dateValue))
      && dateValue.length >= 6
      && typeof dateValue[0] === 'number'
      && dateValue[0] > 2000 && dateValue[0] < 2100) {
    const date = parseRustTimeArray(dateValue);
    if (date && !isNaN(date.getTime())) {
      try {
        return date.toISOString().split('T')[0];
      } catch (e) {
        console.warn('Error formatting date to string:', e, dateValue);
      }
    }
  }

  // Handle OffsetDateTime from Rust
  if (typeof dateValue === 'object' && 'secs_since_epoch' in dateValue) {
    return new Date(dateValue.secs_since_epoch * 1000).toISOString().split('T')[0];
  }

  // Handle strings
  if (typeof dateValue === 'string') {
    const normalized = dateValue.replace(" ", "T");
    const date = new Date(normalized);
    if (!isNaN(date.getTime())) {
      return date.toISOString().split('T')[0];
    }
    // Maybe it's already in YYYY-MM-DD format
    if (/^\d{4}-\d{2}-\d{2}/.test(dateValue)) {
      return dateValue.split('T')[0];
    }
  }

  console.warn('Could not convert to date string:', dateValue);
  return new Date().toISOString().split('T')[0];
}
