# PepTrack Testing Guide

## Edge Cases & Test Scenarios

This document outlines critical edge cases and testing scenarios for PepTrack's backup and restore system.

---

## 1. OAuth Token Refresh

### Test Scenarios

**✅ Happy Path**
- [ ] Fresh OAuth connection → Token stored correctly with expires_at
- [ ] Manual backup with valid token → Upload succeeds
- [ ] Token expires → Auto-refresh → Upload succeeds

**⚠️ Edge Cases**
- [ ] Token expires during backup → Should refresh mid-operation
- [ ] Refresh token itself is invalid → Should show clear error asking to reconnect
- [ ] Network fails during token refresh → Should retry or show helpful error
- [ ] OAuth config file missing → Should prompt to reconnect Drive
- [ ] Concurrent requests while token is refreshing → Should queue properly

---

## 2. Scheduled Backups

### Test Scenarios

**✅ Happy Path**
- [ ] Enable hourly backup → Next backup calculated correctly
- [ ] Enable daily at specific hour → Runs at correct time
- [ ] Weekly backup → Runs once per week
- [ ] Manual trigger → Runs immediately

**⚠️ Edge Cases**
- [ ] App closed during scheduled backup → Backup not run (expected)
- [ ] Multiple backups triggered simultaneously → Only one runs (lock mechanism)
- [ ] Backup fails with retry → Retries with exponential backoff
- [ ] All retries exhausted → Shows notification with helpful error
- [ ] Drive disconnects mid-backup → Falls back to local only
- [ ] Disk full during backup → Shows clear error with suggestion
- [ ] Schedule updated while backup running → Doesn't interrupt current backup

---

## 3. Backup Compression

### Test Scenarios

**✅ Happy Path**
- [ ] Create compressed backup → File ends with .gz
- [ ] Create uncompressed backup → File ends with .json
- [ ] Restore compressed backup → Auto-detects and decompresses
- [ ] Restore uncompressed backup → Reads normally

**⚠️ Edge Cases**
- [ ] Corrupted gzip file → Shows clear error during preview/restore
- [ ] .gz extension but not actually gzipped → Detects and handles gracefully
- [ ] Very large dataset → Compression doesn't timeout
- [ ] Compression fails → Falls back to uncompressed or shows error

---

## 4. Backup Cleanup

### Test Scenarios

**✅ Happy Path**
- [ ] Keep last 10 backups → Deletes oldest when 11th created
- [ ] Delete backups older than 30 days → Only old ones removed
- [ ] Both rules enabled → Both applied correctly
- [ ] No cleanup rules → All backups retained

**⚠️ Edge Cases**
- [ ] Cleanup runs while backup in progress → Doesn't delete current backup
- [ ] Files locked by another process → Cleanup logs warning but continues
- [ ] Backup directory doesn't exist → Cleanup handles gracefully
- [ ] keepLastN = 0 → Validates and prevents (shouldn't delete everything)
- [ ] Manual backup file manually renamed → Cleanup skips non-matching files

---

## 5. Google Drive Integration

### Test Scenarios

**✅ Happy Path**
- [ ] Connect Drive → OAuth flow completes
- [ ] Upload backup → File appears in "PepTrack Backups" folder
- [ ] Disconnect Drive → Tokens removed
- [ ] Reconnect Drive → New tokens stored

**⚠️ Edge Cases**
- [ ] User denies OAuth permissions → Clear error message
- [ ] Drive folder already exists → Reuses existing folder
- [ ] Drive quota exceeded → Shows meaningful error
- [ ] Network drops during upload → Shows error with retry suggestion
- [ ] Upload very large file → Doesn't timeout
- [ ] Multiple rapid uploads → Queues properly
- [ ] Drive API rate limit hit → Shows appropriate error

---

## 6. Restore Functionality

### Test Scenarios

**✅ Happy Path**
- [ ] Preview valid backup → Shows correct counts
- [ ] Restore backup → Items appear in database
- [ ] Restore merges with existing data → No duplicates created
- [ ] Restore shows success counts → Accurate numbers

**⚠️ Edge Cases**
- [ ] Restore empty backup file → Shows error "appears to be empty"
- [ ] Restore corrupted backup → Shows clear parsing error
- [ ] Restore backup with missing metadata → Handles gracefully
- [ ] Restore during active dose logging → Doesn't cause conflicts
- [ ] Very large backup file → Doesn't freeze UI
- [ ] Restore same backup twice → Items updated, not duplicated
- [ ] File selected but doesn't exist → Shows file not found error

---

## 7. Notifications

### Test Scenarios

**✅ Happy Path**
- [ ] Manual backup succeeds → Shows success notification
- [ ] Scheduled backup succeeds → Shows success notification
- [ ] Backup fails → Shows failure notification with details
- [ ] Test notification button → Sends test notification

**⚠️ Edge Cases**
- [ ] OS notification permissions denied → Shows warning in UI
- [ ] Notification preferences disabled → No notifications sent
- [ ] Multiple rapid notifications → Stack properly, don't overlap
- [ ] Notification clicked → Dismisses properly
- [ ] App in background → Notifications still appear

---

## 8. Error Handling

### Test Scenarios

**✅ Happy Path**
- [ ] Network error → Shows user-friendly toast with suggestion
- [ ] File not found → Shows specific error with context
- [ ] Validation error → Shows clear message about what's wrong

**⚠️ Edge Cases**
- [ ] Unrecognized error → Falls back to generic error handler
- [ ] Error during error handling → Doesn't crash, logs to console
- [ ] Multiple errors at once → Shows multiple toasts
- [ ] Very long error message → Truncates or wraps properly

---

## 9. UI/UX Edge Cases

### Test Scenarios

**⚠️ Edge Cases**
- [ ] Settings tab switched during backup → Progress updates continue
- [ ] Settings tab switched during restore → Operation completes
- [ ] Very long protocol/file names → UI doesn't break
- [ ] Empty database → All views handle gracefully
- [ ] Thousands of dose logs → History table paginated or performs well
- [ ] Rapid clicking of backup button → Disabled during operation
- [ ] Browser window resized → Responsive layout adjusts

---

## 10. Cross-Platform

### Test Scenarios (if applicable)

**⚠️ Platform-Specific**
- [ ] macOS notifications → Work correctly
- [ ] Windows notifications → Work correctly
- [ ] Linux notifications → Work correctly
- [ ] File paths with spaces → Handled correctly on all platforms
- [ ] File paths with special characters → Handled correctly

---

## Testing Checklist

### Before Each Release

1. **Backup System**
   - [ ] Create manual backup (uncompressed)
   - [ ] Create manual backup (compressed)
   - [ ] Upload to Google Drive
   - [ ] Enable scheduled backup (hourly)
   - [ ] Wait for scheduled backup to run
   - [ ] Check backup history

2. **Restore System**
   - [ ] Preview backup file
   - [ ] Restore from local backup
   - [ ] Verify all data restored correctly

3. **Google Drive**
   - [ ] Connect new Drive account
   - [ ] Upload backup
   - [ ] Check Drive for "PepTrack Backups" folder
   - [ ] Disconnect Drive
   - [ ] Verify tokens removed

4. **Notifications**
   - [ ] Enable notifications
   - [ ] Trigger manual backup
   - [ ] Trigger successful scheduled backup
   - [ ] Cause backup failure (disconnect Drive)
   - [ ] Test notification button

5. **Error Scenarios**
   - [ ] Try to restore invalid file
   - [ ] Try to backup with Drive disconnected
   - [ ] Try to backup with disk full
   - [ ] Try to restore corrupted backup

---

## Automated Testing

### Unit Tests Needed

```rust
// Rust backend tests
#[test]
fn test_token_expiry_detection()
#[test]
fn test_token_refresh_flow()
#[test]
fn test_backup_retry_logic()
#[test]
fn test_cleanup_keeps_newest()
#[test]
fn test_compression_decompression()
#[test]
fn test_concurrent_backup_prevention()
```

### Integration Tests Needed

```typescript
// Frontend integration tests
describe('Backup System', () => {
  it('should create and restore backup')
  it('should handle compressed backups')
  it('should show error for corrupted files')
})

describe('Settings', () => {
  it('should save schedule preferences')
  it('should navigate between tabs')
  it('should validate user inputs')
})
```

---

## Recent Bug Fixes (2025-11-12)

### Runtime Error Fixes Validated
- ✅ **Date Handling**: Fixed "Invalid Date" errors from Rust OffsetDateTime arrays
  - Test: Create dose log → View in Recent Activity → No console errors
  - Test: View Dose Calendar → Dates render correctly

- ✅ **Division by Zero**: Fixed cost analysis calculations
  - Test: View Cost Analysis with no data → No Infinity/NaN values
  - Test: Add single supplier → Rating calculation works

- ✅ **NaN Validation**: All numeric inputs validated
  - Test: Clear numeric input → Form validation prevents submission
  - Test: Enter non-numeric value → Shows clear error message

- ✅ **Memory Leaks**: Timeout cleanup implemented
  - Test: Rapidly save/delete inventory items → No timeout overlap
  - Test: Navigate away during save → Cleanup executes properly

### Testing Recommendations
1. **Date Edge Cases**
   - [ ] Create dose and verify it appears in timeline with correct date
   - [ ] Check calendar heatmap shows doses on correct days
   - [ ] Verify protocol "Last updated" field displays correctly

2. **Cost Analysis**
   - [ ] Test with zero inventory items → No division errors
   - [ ] Test with single supplier → Ratings display correctly
   - [ ] Test trend display with limited data → Shows real trends

3. **Form Validation**
   - [ ] Try to submit forms with cleared numeric fields
   - [ ] Enter letters in numeric fields → Validation catches it
   - [ ] Rapid form submissions → No double-saves

---

## Known Limitations

1. **Scheduled backups only run while app is open** - This is a limitation of the current architecture. Consider adding system-level scheduling in the future.

2. **Single Drive account** - Currently supports one Google Drive connection at a time.

3. **No cloud restore** - Can only restore from local files, not directly from Drive. Consider adding in future.

4. **No backup encryption** - Backups are not encrypted at rest. Consider adding for sensitive data.

5. **No differential backups** - Each backup is full, not incremental. Could optimize for large datasets.

---

## Performance Benchmarks

### Target Performance

- **Manual backup** (1000 protocols, 10000 doses): < 2 seconds
- **Compressed backup**: < 5 seconds
- **Drive upload** (5MB file): < 10 seconds
- **Restore operation**: < 3 seconds
- **Backup preview**: < 500ms
- **UI responsiveness during backup**: 60fps

Test with large datasets to ensure performance targets are met.

---

## Security Considerations

### What to Test

- [ ] OAuth tokens stored securely (not in plain text environment variables)
- [ ] Drive OAuth config not exposed to frontend unnecessarily
- [ ] Backup files don't contain sensitive tokens
- [ ] File paths sanitized to prevent directory traversal
- [ ] User can't trigger multiple simultaneous operations that could corrupt data

---

## Future Testing Needs

As features are added, test:

1. **Multi-cloud support** - Test with Dropbox, OneDrive, etc.
2. **Backup encryption** - Test encryption/decryption flow
3. **Cloud restore** - Test restoring directly from cloud
4. **Differential backups** - Test incremental backup logic
5. **Backup on app close** - Test exit handler
