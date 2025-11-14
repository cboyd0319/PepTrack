# PepTrack Code Quality Audit Report
**Date:** 2025-11-14
**Auditor:** The Rust Mac Overlord
**Scope:** Comprehensive analysis of test coverage and technical documentation

---

## Executive Summary

PepTrack has a **mixed code quality profile** with exceptional work in some areas and critical gaps in others. The encryption and backup infrastructure are exemplary, but core data persistence (`db.rs`) and application command layer lack adequate test coverage. The Vue frontend has only **3.3% component test coverage** (1 of 30+ components).

### Overall Ratings

| Area | Test Coverage | Documentation | Grade |
|------|--------------|---------------|-------|
| **Rust: Core Encryption** | ✅ Excellent (9 tests) | ✅ Excellent | A+ |
| **Rust: Backup Encryption** | ✅ Excellent (8 tests) | ✅ Excellent | A+ |
| **Rust: Database Layer** | ⚠️ Critical Gap (1 test) | ❌ None | D |
| **Rust: Models** | ❌ None | ❌ None | F |
| **Rust: Local AI** | ⚠️ Minimal (2 tests) | ⚠️ Minimal | C |
| **Rust: Literature APIs** | ⚠️ Good (7 tests) | ✅ Good | B |
| **Rust: Tauri Commands** | ⚠️ Serialization only | ❌ Minimal | C |
| **Vue: Components** | ❌ Critical Gap (1/30+) | ⚠️ Minimal | D |
| **Vue: Stores** | ❌ None (0/5) | ⚠️ Some JSDoc | D |
| **Vue: Composables** | ❌ None (0/5) | ⚠️ Some JSDoc | D |
| **Vue: Error Handling** | ✅ Excellent (19 tests) | ✅ Good | A |

---

## Detailed Findings by Module

### 1. Rust Core Crate (`crates/core/`)

#### ✅ `encryption.rs` - **GOLD STANDARD**
**Test Coverage:** 9 comprehensive tests
**Documentation:** Excellent Rustdoc with examples

**Tests include:**
- ✅ Key material validation (short keys rejected)
- ✅ Round-trip encryption (empty & 1MB plaintext)
- ✅ Tampering detection (Poly1305 MAC validation)
- ✅ Short payload rejection
- ✅ Wrong key detection
- ✅ Unique nonce generation per call
- ✅ Key provider cloning
- ✅ EnvKeyProvider error handling

**Verdict:** This module is an exemplar. Use it as a template for other modules.

---

#### ✅ `backup_encryption.rs` - **GOLD STANDARD**
**Test Coverage:** 8 comprehensive tests
**Documentation:** Excellent module-level docs with wire format

**Tests include:**
- ✅ Round-trip encryption/decryption
- ✅ Wrong password detection
- ✅ Encrypted backup detection
- ✅ Different salts/nonces produce different ciphertexts
- ✅ Empty password handling
- ✅ Large data (100KB)
- ✅ Unicode data (émojis, Cyrillic, Chinese)

**Verdict:** Another perfect example. Security-critical code with comprehensive coverage.

---

#### ❌ `db.rs` - **CRITICAL PRIORITY**
**Test Coverage:** 1 test only (protocol round-trip)
**Documentation:** None
**Lines of Code:** ~740

**Missing Tests (CRITICAL):**
- ❌ Dose log CRUD (append, list, delete, by protocol)
- ❌ Literature cache (cache, list, search)
- ❌ Suppliers (CRUD, get, delete)
- ❌ Inventory (CRUD, by protocol, get, delete)
- ❌ Price history (add, list, get latest)
- ❌ Alerts (create, list, mark read, dismiss, clear)
- ❌ Summary history (save, list, delete)
- ❌ Connection failure scenarios
- ❌ Encryption/decryption failures during save/load
- ❌ Schema initialization (foreign keys, indexes)
- ❌ Concurrent access patterns
- ❌ WAL mode verification
- ❌ Foreign key cascade behavior (DELETE CASCADE, SET NULL)

**Missing Documentation:**
- ❌ No module-level docs explaining storage architecture
- ❌ No struct documentation for `StorageManager` or `StorageConfig`
- ❌ No Rustdoc on public methods
- ❌ No examples showing safe usage patterns

**Security Concerns:**
- ⚠️ `connection()` method bypasses encryption but lacks tests showing safe usage
- ⚠️ `.unwrap_or_default()` in serialization could hide errors

**Verdict:** This is the **most critical gap**. The entire application's data persistence depends on this module, yet it has only 1 test. A single bug here could corrupt user data.

---

#### ❌ `models.rs` - **HIGH PRIORITY**
**Test Coverage:** None
**Documentation:** None
**Lines of Code:** ~286

**Missing Tests:**
- ❌ Constructor functions (`new()` for all models)
- ❌ Serialization/deserialization (especially `OffsetDateTime`, enums)
- ❌ Enum variants (`VialStatus`, `AlertType`, `AlertSeverity`)
- ❌ Default values
- ❌ Edge cases (empty strings, extreme numbers)

**Missing Documentation:**
- ❌ No struct documentation
- ❌ No field documentation
- ❌ No examples

**Verdict:** Domain models are the API contract between Rust and the database. Serialization bugs could cause data loss. Needs comprehensive tests.

---

#### ⚠️ `keychain.rs` - **GOOD, MINOR GAPS**
**Test Coverage:** 6 tests (all `#[ignore]` due to user interaction)
**Documentation:** Good Rustdoc

**Tests include:**
- ✅ Store and retrieve
- ✅ Key generation (32 bytes)
- ✅ Integration with EnvelopeEncryption
- ✅ Delete functionality
- ✅ File migration

**Missing Tests:**
- ❌ Non-macOS error paths (could add unit tests for error messages)
- ❌ `migrate_file_key_to_keychain` edge cases (invalid hex, corrupted file, empty file)

**Verdict:** Good coverage, but tests don't run in CI. Consider adding non-interactive tests for error paths.

---

### 2. Rust Literature Crate (`crates/literature/`)

#### ⚠️ `pubmed.rs`, `openalex.rs`, `crossref.rs` - **GOOD, MISSING UNIT TESTS**
**Test Coverage:** 7 network tests (gracefully handle failures)
**Documentation:** Excellent module-level docs with API links

**Tests include:**
- ✅ Search returns results (network)
- ✅ Empty query handling (network)
- ✅ Fetcher creation

**Missing Tests:**
- ❌ `search_pmids()` and `fetch_summaries()` unit tests (mock HTTP responses)
- ❌ `reconstruct_abstract()` function (OpenAlex, complex logic)
- ❌ Date parsing logic (Crossref)
- ❌ Author name formatting (Crossref)
- ❌ Error scenarios (malformed responses, timeouts, rate limits)
- ❌ API key handling (PubMed)

**Verdict:** Good documentation and integration tests, but lacks unit tests for internal logic. Network tests may fail in CI.

---

### 3. Rust Local-AI Crate (`crates/local-ai/`)

#### ⚠️ `lib.rs` - **CRITICAL GAPS (SECURITY)**
**Test Coverage:** 2 tests (provider chain logic only)
**Documentation:** Minimal
**Lines of Code:** ~387

**Tests include:**
- ✅ Provider chain prefers Codex by default
- ✅ Provider chain skips missing providers

**Missing Tests (CRITICAL - SECURITY):**
- ❌ `summarize()` method (command execution)
- ❌ `build_summary_prompt()` (prompt injection detection)
- ❌ `parse_claude_json()` (complex parsing logic)
- ❌ `parse_codex_json()` (complex parsing logic)
- ❌ Error handling for CLI failures (binary not found, non-zero exit)
- ❌ Format handling (Markdown vs JSON)
- ❌ Command injection scenarios (malicious file paths, shell escaping)
- ❌ Timeout handling

**Missing Documentation:**
- ❌ No module-level docs
- ❌ No trait documentation
- ❌ No struct documentation
- ❌ No explanation of orchestration strategy

**Security Concerns:**
- ⚠️ Executes external binaries with user input
- ⚠️ Prompt building logic is untested (injection risk)
- ⚠️ JSON parsing is complex and untested (malformed responses could crash)

**Verdict:** **Security-sensitive code with inadequate tests.** Command execution and prompt building MUST be tested thoroughly.

---

### 4. Tauri Command Layer (`src-tauri/src/commands/`)

#### ⚠️ **Pattern: Serialization Tests Only, No Behavior Tests**

**Examples Audited:**
- `protocols.rs` - No tests
- `ai.rs` - 7 serialization tests, no command tests
- `backup.rs` - 4 serialization tests, no command tests

**Missing Tests (ALL COMMANDS):**
- ❌ Command functions with mocked `AppState`
- ❌ Error handling (storage failures, network failures)
- ❌ Input validation (empty strings, invalid IDs, out-of-range numbers)
- ❌ Integration with stores (does upsert actually call storage?)
- ❌ Edge cases (concurrent requests, null/None handling)

**Missing Documentation:**
- ❌ No module-level docs explaining command architecture
- ❌ No Rustdoc on command functions
- ❌ No examples showing expected payloads

**Verdict:** Commands are the API surface between Rust and Vue. They have serialization tests but no behavior/integration tests. A bug here could break the entire UI.

---

### 5. Vue Frontend (`frontend/src/`)

#### ✅ `utils/__tests__/errorHandling.spec.ts` - **GOLD STANDARD**
**Test Coverage:** 19 comprehensive tests
**Lines:** 312

**Tests include:**
- ✅ All error type detection (network, file not found, Drive, token expiry, backup, restore, validation, database)
- ✅ Unknown error handling
- ✅ Error context details
- ✅ Suggestions in error messages
- ✅ Non-Error objects
- ✅ Success/warning/info toasts
- ✅ `handleAsync` wrapper
- ✅ `getErrorMessage` utility
- ✅ Case-insensitive detection
- ✅ Fallback behavior (no showToast)

**Verdict:** This is a **perfect example** of comprehensive utility testing. Use it as a template.

---

#### ✅ `components/__tests__/Settings.spec.ts` - **GOLD STANDARD**
**Test Coverage:** 14 comprehensive tests
**Lines:** 214

**Tests include:**
- ✅ Tab rendering
- ✅ Default tab selection
- ✅ Tab switching
- ✅ Tab content rendering
- ✅ Navigation through all tabs
- ✅ Tab state lifecycle
- ✅ Accessibility attributes
- ✅ Transition effects
- ✅ Header display
- ✅ Responsive design

**Verdict:** Another **perfect example**. Proper component mocking, comprehensive interaction tests, accessibility checks.

---

#### ⚠️ `components/__tests__/ProtocolList.spec.ts` - **MINIMAL**
**Test Coverage:** 1 test only

**Missing Tests:**
- ❌ Empty state rendering
- ❌ Loading state
- ❌ Error state
- ❌ User interactions (edit, delete)
- ❌ Sorting/filtering
- ❌ Pagination

**Verdict:** Bare minimum test. Needs significant expansion.

---

#### ❌ **Components (30+ components, only 1 tested) - CRITICAL**

**Untested Critical Components:**
- ❌ `Dashboard.vue` - Main analytics view
- ❌ `DoseTracker.vue` - Core dose logging
- ❌ `BackupExport.vue` - Data backup
- ❌ `GoogleDriveBackup.vue` - Cloud integration
- ❌ `LiteratureSearch.vue` - Research search
- ❌ `EnhancedAiSummary.vue` - AI integration
- ❌ `ProtocolForm.vue` - Protocol creation
- ❌ `SupplierManagement.vue` - Supplier CRUD
- ❌ `InventoryManagement.vue` - Inventory tracking
- ❌ `GlobalSearch.vue` - Search functionality
- ❌ `OnboardingFlow.vue` - User onboarding
- ❌ (20+ more)

**Verdict:** **3.3% component coverage is unacceptable.** Critical user flows are completely untested.

---

#### ❌ **Stores (5 stores, 0 tested) - HIGH PRIORITY**

**Untested Stores:**
- ❌ `protocols.ts` - Has JSDoc, implements caching/optimistic updates
- ❌ `doses.ts` - Dose logging state
- ❌ `suppliers.ts` - Supplier state
- ❌ `literature.ts` - Literature cache
- ❌ `ui.ts` - UI state management

**What Needs Testing:**
- ❌ Caching logic (30-second cache in `protocols.ts`)
- ❌ Optimistic updates (rollback on error)
- ❌ Error handling
- ❌ State mutations
- ❌ Computed properties

**Verdict:** Stores are the **backbone of Vue state management**. Zero test coverage is a critical gap.

---

#### ❌ **Composables (5+ composables, 0 tested) - HIGH PRIORITY**

**Untested Composables:**
- ❌ `useProtocols.ts` - Has JSDoc, wrapper around store
- ❌ `useDoses.ts`
- ❌ `useSuppliers.ts`
- ❌ `useLiterature.ts`
- ❌ `useReminderService.ts` - Critical notification logic

**What Needs Testing:**
- ❌ Helper functions
- ❌ Computed properties
- ❌ Reactivity
- ❌ Store integration

**Verdict:** Composables provide the **component API**. They must be tested to ensure components work correctly.

---

## Critical Priorities (Do These First)

### P0: Security & Data Integrity (IMMEDIATE)

| Module | Reason | Effort | Impact |
|--------|--------|--------|--------|
| **`db.rs`** | Core data persistence, only 1 test, entire app depends on it | 3-4 days | CRITICAL |
| **`local-ai/lib.rs`** | Executes external binaries with user input, security risk | 2 days | CRITICAL |
| **`models.rs`** | Serialization bugs could cause data loss | 1 day | HIGH |

### P1: Application Layer (HIGH)

| Module | Reason | Effort | Impact |
|--------|--------|--------|--------|
| **Tauri Commands** | API surface between Rust and Vue, no behavior tests | 2-3 days | HIGH |
| **Vue Stores** | State management backbone, zero coverage | 2 days | HIGH |
| **Vue Composables** | Component API, zero coverage | 1 day | HIGH |

### P2: User-Facing Features (MEDIUM)

| Module | Reason | Effort | Impact |
|--------|--------|--------|--------|
| **Critical Components** | Dashboard, DoseTracker, BackupExport, GoogleDriveBackup, LiteratureSearch | 3-4 days | HIGH |
| **Other Components** | Remaining 25+ components | 4-5 days | MEDIUM |

### P3: External Integrations (LOW)

| Module | Reason | Effort | Impact |
|--------|--------|--------|--------|
| **Literature APIs** | Already has integration tests, needs unit tests | 1-2 days | LOW |

---

## Concrete Implementation Plan

### Phase 1: Security & Data Integrity (Week 1)

#### **Day 1-3: `db.rs` Comprehensive Test Suite**

**File:** `crates/core/src/db.rs` (add `#[cfg(test)]` module)

**Test Categories:**

1. **Protocol CRUD** (expand existing test)
   - List empty database
   - Get non-existent protocol
   - Update existing protocol
   - Upsert conflict handling

2. **Dose Logs** (NEW)
   - Append dose log
   - List all dose logs
   - List dose logs for protocol
   - Delete dose log
   - Delete dose log with non-existent ID
   - Foreign key cascade (delete protocol -> doses deleted)

3. **Literature** (NEW)
   - Cache literature entry
   - List literature
   - Search literature (title, source, summary)
   - Search with no results

4. **Suppliers** (NEW)
   - Upsert supplier
   - List suppliers
   - Get supplier by ID
   - Delete supplier
   - Foreign key SET NULL (delete supplier -> inventory.supplier_id becomes NULL)

5. **Inventory** (NEW)
   - Upsert inventory item
   - List all inventory
   - List inventory by protocol
   - Get inventory item by ID
   - Delete inventory item
   - Foreign key cascade (delete protocol -> inventory deleted)

6. **Price History** (NEW)
   - Add price history
   - List price history for supplier
   - List price history for supplier + peptide
   - Get latest price

7. **Alerts** (NEW)
   - Create alert
   - List alerts (with/without dismissed)
   - Mark alert as read
   - Dismiss alert
   - Clear all alerts

8. **Summary History** (NEW)
   - Save summary
   - List summaries with limit
   - List summaries without limit
   - Delete summary

9. **Error Handling** (NEW)
   - Connection failure (invalid path)
   - Encryption failure (corrupted key)
   - Decryption failure (tampered blob)
   - Deserialization failure (invalid JSON)

10. **Schema & Constraints** (NEW)
    - Foreign key enforcement (insert with invalid protocol_id)
    - WAL mode enabled
    - Index existence (price_history, alerts, summary_history)

**Documentation Tasks:**
- Add module-level docs explaining storage architecture
- Add Rustdoc to `StorageManager`, `StorageConfig`
- Add Rustdoc to all public methods
- Add usage examples for each major operation
- Document `connection()` method with safety warnings

**Estimate:** 3 days (24 hours)

---

#### **Day 4-5: `local-ai/lib.rs` Security Tests**

**File:** `crates/local-ai/src/lib.rs` (expand `#[cfg(test)]` module)

**Test Categories:**

1. **Prompt Building** (NEW - CRITICAL)
   - Standard summarization prompt
   - Prompt with existing instructions (CRITICAL INSTRUCTION prefix)
   - Markdown format
   - JSON format
   - Prompt injection attempts (escape sequences, shell commands)
   - Unicode content

2. **JSON Parsing** (NEW)
   - `parse_claude_json` with single object
   - `parse_claude_json` with streaming (line-by-line)
   - `parse_codex_json` with item.completed events
   - Malformed JSON handling
   - Empty responses
   - Large responses (1MB+)

3. **Command Execution** (NEW - Mocked)
   - Mock successful Codex execution
   - Mock successful Claude execution
   - Mock binary not found
   - Mock non-zero exit code
   - Mock timeout
   - Mock stderr output

4. **Orchestration** (Expand existing)
   - Fallback chain (Codex fails -> Claude succeeds)
   - Both providers fail
   - Custom preferred provider

**Documentation Tasks:**
- Add module-level docs explaining orchestration strategy
- Add trait documentation (`LocalAiClient`)
- Add struct documentation (`LocalAiOrchestrator`, `AiClientConfig`)
- Add security notes about command execution
- Add examples showing safe usage

**Estimate:** 2 days (16 hours)

---

#### **Day 6: `models.rs` Serialization Tests**

**File:** `crates/core/src/models.rs` (add `#[cfg(test)]` module)

**Test Categories:**

1. **Constructors** (NEW)
   - `PeptideProtocol::new()`
   - `DoseLog::new()`
   - `LiteratureEntry::new()`
   - `Supplier::new()`
   - `InventoryItem::new()`
   - `PriceHistory::new()`
   - `Alert::new()`
   - `SummaryHistory::new()`

2. **Serialization** (NEW)
   - Protocol to JSON and back
   - DoseLog to JSON and back
   - All models round-trip serialization
   - OffsetDateTime serialization (ISO 8601 format)

3. **Enums** (NEW)
   - `VialStatus` variants serialize correctly
   - `AlertType` variants serialize correctly
   - `AlertSeverity` variants serialize correctly
   - Enum serde attributes (`rename_all = "lowercase"`)

4. **Edge Cases** (NEW)
   - Empty strings in fields
   - Very large numbers (f32 edge cases)
   - None vs Some in Option fields
   - Unicode in text fields

**Documentation Tasks:**
- Add struct documentation for all models
- Add field documentation (purpose, constraints)
- Add enum documentation
- Add examples showing construction and serialization

**Estimate:** 1 day (8 hours)

---

### Phase 2: Application Layer (Week 2)

#### **Day 7-8: Tauri Command Tests**

**Files:** `src-tauri/src/commands/*.rs`

**Approach:** Add integration tests with mocked `AppState`

**Test Categories (per command module):**

1. **`protocols.rs`** (NEW)
   - `list_protocols()` returns data from storage
   - `list_protocols()` handles storage error
   - `save_protocol()` calls storage.upsert_protocol
   - `save_protocol()` returns created protocol
   - `save_protocol()` handles validation errors
   - `save_protocol()` handles storage errors

2. **`ai.rs`** (Expand existing)
   - `check_ai_availability()` detects providers
   - `summarize_text()` calls ai_client.summarize
   - `summarize_text()` handles missing providers
   - `summarize_text()` handles timeout
   - `summarize_text()` validates input (empty content)

3. **`backup.rs`** (Expand existing)
   - `export_backup_data()` exports all data
   - `export_backup_data()` encrypts with password
   - `export_backup_data()` handles empty password
   - `export_backup_data()` handles storage failure
   - `get_backup_file_path()` returns valid path (already tested)

4. **Repeat for all 14 command modules**

**Documentation Tasks:**
- Add module-level docs to `commands/mod.rs` explaining architecture
- Add Rustdoc to all command functions
- Add payload examples in documentation

**Estimate:** 2 days (16 hours)

---

#### **Day 9-10: Vue Store Tests**

**Files:** `frontend/src/stores/*.ts` (create `*.spec.ts` files)

**Approach:** Use Vitest + Pinia testing utilities

**Test Categories (per store):**

1. **`protocols.ts`** (NEW)
   - `fetchProtocols()` calls API and updates state
   - `fetchProtocols()` uses cache when valid
   - `fetchProtocols(true)` forces refresh
   - `createProtocol()` calls API and adds to state (optimistic)
   - `createProtocol()` rolls back on error
   - `updateProtocol()` optimistic update
   - `updateProtocol()` rolls back on error
   - `removeProtocol()` removes from state
   - `protocolsByPeptide` computed property groups correctly
   - `isCacheValid` computed property expires after 30s

2. **Repeat for `doses.ts`, `suppliers.ts`, `literature.ts`, `ui.ts`**

**Test Setup:**
```typescript
import { setActivePinia, createPinia } from 'pinia'
import { beforeEach, describe, it, expect, vi } from 'vitest'
import { useProtocolStore } from '../protocols'
import * as api from '../../api/peptrack'

// Mock API
vi.mock('../../api/peptrack')

beforeEach(() => {
  setActivePinia(createPinia())
  vi.clearAllMocks()
})
```

**Estimate:** 2 days (16 hours)

---

#### **Day 11: Vue Composable Tests**

**Files:** `frontend/src/composables/*.ts` (create `*.spec.ts` files)

**Test Categories (per composable):**

1. **`useProtocols.ts`** (NEW)
   - Returns refs from store
   - `hasProtocols` computed works correctly
   - `isEmpty` computed works correctly
   - `refreshProtocols()` calls store method
   - `addProtocol()` calls createProtocol with correct payload

2. **Repeat for all composables**

**Estimate:** 1 day (8 hours)

---

### Phase 3: User-Facing Components (Week 3)

#### **Day 12-15: Critical Component Tests**

**Files:** `frontend/src/components/*.vue` (create `__tests__/*.spec.ts`)

**Priority Components:**

1. **`Dashboard.vue`** (Day 12)
   - Renders analytics widgets
   - Shows empty state when no data
   - Loading state
   - Widget visibility toggles
   - Refresh functionality

2. **`DoseTracker.vue`** (Day 12)
   - Renders dose list
   - Add dose button
   - Edit dose
   - Delete dose with confirmation
   - Empty state
   - Date filtering

3. **`BackupExport.vue`** (Day 13)
   - Renders backup form
   - Export button triggers backup
   - Password encryption toggle
   - Password strength indicator
   - Success/error handling
   - File save dialog

4. **`GoogleDriveBackup.vue`** (Day 13)
   - OAuth flow initiation
   - Connection status display
   - Upload backup button
   - Disconnect button
   - Error handling (token expiry, network)

5. **`LiteratureSearch.vue`** (Day 14)
   - Search input
   - Source selection (PubMed, OpenAlex, Crossref)
   - Search results rendering
   - Loading state
   - Empty results
   - AI summary button
   - Save to cache

6. **`EnhancedAiSummary.vue`** (Day 14)
   - Renders summary content (Markdown)
   - Risk matrix display
   - Protocol recommendations
   - Copy to clipboard
   - Export summary
   - Error handling (no AI providers)

7. **`ProtocolForm.vue`** (Day 15)
   - Create mode vs Edit mode
   - Form validation (required fields)
   - Save button triggers store action
   - Cancel button
   - Prepopulated data in edit mode
   - Success/error toasts

8. **`SupplierManagement.vue`** (Day 15)
   - Renders supplier list
   - Add supplier form
   - Edit supplier
   - Delete supplier with confirmation
   - Empty state

**Estimate:** 4 days (32 hours)

---

#### **Day 16-19: Remaining Components**

**Files:** Remaining 20+ components

**Approach:** Group similar components and batch tests

**Component Groups:**
- Analytics widgets (DoseCalendarHeatmap, ProtocolProgressTracker, CostAnalysisDashboard, RecentActivityTimeline)
- Settings tabs (BackupAndRestore, NotificationPreferences, CalendarIntegration, DarkModeToggle)
- Navigation (GlobalSearch, QuickActionsFAB, KeyboardShortcutsHelp)
- Onboarding (OnboardingFlow, WelcomeScreen)
- Inventory (InventoryManagement, ReconstitutionCalculator)
- Utility (EmptyState, LoadingSkeleton, Toast, ErrorBoundary)

**Estimate:** 4 days (32 hours)

---

### Phase 4: External Integrations (Week 4)

#### **Day 20-21: Literature API Unit Tests**

**Files:** `crates/literature/src/*.rs` (expand tests)

**Test Categories:**

1. **`pubmed.rs`** (NEW - Mocked)
   - Mock successful ESearch response
   - Mock successful ESummary response
   - Mock empty results
   - Mock malformed JSON
   - API key in URL when provided
   - Error handling (network timeout, 500 error)

2. **`openalex.rs`** (NEW - Mocked)
   - Mock successful search response
   - `reconstruct_abstract()` with valid inverted index
   - `reconstruct_abstract()` with empty index
   - `reconstruct_abstract()` with large index
   - Author display name extraction
   - DOI extraction from id

3. **`crossref.rs`** (NEW - Mocked)
   - Mock successful search response
   - Date parsing (year only, year-month, full date)
   - Author name formatting (given + family, family only)
   - Container title extraction

**Approach:** Use `mockito` or `wiremock` for HTTP mocking

**Estimate:** 2 days (16 hours)

---

## Documentation Improvement Plan

### Rust Documentation

#### **High Priority:**
1. **`crates/core/src/db.rs`** - Add module docs, struct docs, method docs, examples
2. **`crates/core/src/models.rs`** - Add struct docs, field docs, enum docs
3. **`crates/local-ai/src/lib.rs`** - Add module docs, trait docs, struct docs, security notes
4. **`src-tauri/src/commands/mod.rs`** - Add module docs explaining command architecture
5. **All command functions** - Add Rustdoc with payload examples

#### **Medium Priority:**
6. **`crates/literature/src/models.rs`** - Add trait docs with examples
7. **`crates/core/src/keychain.rs`** - Expand docs with more examples

#### **Low Priority:**
8. **Internal modules** - Add inline comments where logic is complex

### Vue Documentation

#### **High Priority:**
1. **All stores** (`stores/*.ts`) - Add JSDoc (follow `protocols.ts` example)
2. **All composables** (`composables/*.ts`) - Add JSDoc (follow `useProtocols.ts` example)
3. **Critical components** - Add JSDoc to props, emits, methods

#### **Medium Priority:**
4. **Remaining components** - Add JSDoc

---

## Estimated Total Effort

| Phase | Days | Hours |
|-------|------|-------|
| **Phase 1: Security & Data Integrity** | 6 | 48 |
| **Phase 2: Application Layer** | 5 | 40 |
| **Phase 3: User-Facing Components** | 8 | 64 |
| **Phase 4: External Integrations** | 2 | 16 |
| **Documentation** | Ongoing | 16 |
| **TOTAL** | 21 days | 184 hours |

---

## Success Metrics

### Coverage Targets (End State)

| Area | Current | Target |
|------|---------|--------|
| **Rust: Core (`db.rs`)** | 1.3% (1 test) | 90% (50+ tests) |
| **Rust: Core (`models.rs`)** | 0% | 90% (30+ tests) |
| **Rust: Local AI** | 5% (2 tests) | 80% (20+ tests) |
| **Rust: Literature** | 20% (integration only) | 80% (50+ tests) |
| **Rust: Tauri Commands** | 10% (serialization) | 70% (100+ tests) |
| **Vue: Components** | 3.3% (1/30) | 80% (25/30) |
| **Vue: Stores** | 0% | 90% (100+ tests) |
| **Vue: Composables** | 0% | 90% (50+ tests) |

### Documentation Targets

| Area | Current | Target |
|------|---------|--------|
| **Rust: Public API** | 50% | 100% |
| **Vue: Stores** | 20% | 100% |
| **Vue: Composables** | 20% | 100% |
| **Vue: Components** | 10% | 80% |

---

## Recommendations

### Immediate Actions (This Week)
1. ✅ **READ THIS REPORT** - Understand the gaps
2. 🔥 **START WITH `db.rs`** - Most critical module
3. 🔥 **TEST `local-ai/lib.rs`** - Security risk
4. 🔥 **TEST `models.rs`** - Data integrity

### Process Improvements
1. **Add CI Test Requirements** - PRs must not decrease coverage
2. **Adopt TDD** - Write tests before code for new features
3. **Code Review Checklist** - "Does this PR include tests?" must be YES
4. **Use Gold Standards** - Point new contributors to `encryption.rs`, `Settings.spec.ts`, `errorHandling.spec.ts`

### Tooling
1. **Enable `cargo tarpaulin`** - Track Rust coverage in CI
2. **Enable Vitest coverage** - Track Vue coverage in CI
3. **Add coverage badges** - Display in README.md
4. **Set up pre-commit hooks** - Run tests before commit

---

## Appendix: Testing Best Practices (Observed from Gold Standards)

### From `encryption.rs`:
- ✅ Test happy path AND edge cases (empty, large, invalid)
- ✅ Test error paths (tampering, wrong key, short payload)
- ✅ Test randomness (unique nonces)
- ✅ Name tests descriptively (`envelope_encryption_detects_tampering`)

### From `Settings.spec.ts`:
- ✅ Mock child components to isolate tests
- ✅ Test interactions (`click`, `trigger`)
- ✅ Test state persistence (`$forceUpdate`)
- ✅ Test accessibility (`<button>` tags)

### From `errorHandling.spec.ts`:
- ✅ Group related tests (`describe` blocks)
- ✅ Test all error types exhaustively
- ✅ Test edge cases (case-insensitive, prioritization)
- ✅ Test fallback behavior

---

**END OF REPORT**
