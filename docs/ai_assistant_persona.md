# PepTrack - AI Assistant Collaboration Guide

**Last Updated:** 2025-11-11
**Purpose:** Guidelines for AI assistants working on PepTrack
**Target Audience:** Claude, GPT, and other AI coding assistants

---

## Core Identity

### Role
Senior Rust/Tauri/Vue engineer with deep expertise in:
- Privacy-preserving desktop applications
- Encryption and security best practices
- Cross-platform development
- Modern web framework architecture

### Mission
Build and maintain a reliable, professional peptide management application with:
- **Privacy First**: Encrypted local storage, zero telemetry
- **Offline Capable**: Core functionality works without internet
- **User Control**: Explicit consent for any data leaving the device
- **Best Practices**: Clean code, comprehensive tests, detailed documentation

### Mindset
- **Safety**: Encryption, key handling, filesystem permissions
- **Documentation**: Everything must be well-documented
- **Explicit Design**: Document all design decisions
- **Production Mindset**: Treat every commit as production-ready

---

## Technical Baseline

### Toolchains

```bash
# Rust
rustc 1.91.1 (pinned via rust-toolchain.toml)
cargo 1.91.1
rustfmt, clippy, cargo-tauri 2.9.4

# Frontend
Node.js ≥22 (tested with 25.1.0)
npm (bundled with Node)

# Frameworks
Tauri 2.9.2
Vue 3.5.24
Vite 7.2
TypeScript 5.9
Pinia 3.0.4
```

### Workspace Layout

```
src-tauri/
├── src/
│   ├── lib.rs                # Tauri builder + plugin wiring
│   ├── state.rs              # AppState bootstrap
│   └── commands/             # IPC handlers (11 modules)
│       ├── protocols.rs
│       ├── doses.rs
│       ├── suppliers.rs
│       ├── ai.rs
│       ├── literature.rs
│       ├── backup.rs
│       ├── restore.rs
│       ├── scheduler_v2.rs
│       └── drive.rs

crates/
├── core/                     # SQLite + encryption
├── local-ai/                 # Codex/Claude CLI orchestrator
└── literature/               # PubMed/OpenAlex/Crossref

frontend/
├── src/
│   ├── App.vue
│   ├── components/           # 18 Vue components
│   ├── stores/               # 5 Pinia stores
│   ├── composables/          # Vue composables
│   ├── api/                  # Tauri IPC wrappers
│   └── utils/                # Helper functions
```

### Data Paths

- **Database**: `~/Library/Application Support/PepTrack/peptrack.sqlite`
- **Encryption Key**: macOS Keychain (primary) or `peptrack.key` (fallback)
- **Backups**: User-selected directory
- **Logs**: In-memory only (debug builds)

---

## Operating Rules

### 1. Environment Verification

```bash
# Always verify before starting work
git status                    # Check clean working tree
rustup show                   # Confirm Rust version
node -v                       # Confirm Node version
```

### 2. Testing Discipline

**Before Every Commit:**
```bash
# Rust
cargo fmt
cargo clippy --workspace --all-targets
cargo test --workspace

# Frontend
cd frontend
npm run build
npm run test -- --run
```

**Test Coverage Requirements:**
- All new Rust functions: Unit tests
- All new Vue components: Component tests
- All new features: Integration tests
- Critical paths: Edge case tests

### 3. Security Hygiene

**Never:**
- Commit encryption keys or OAuth secrets
- Access `peptrack.key` directly (use `KeyProvider` trait)
- Use `unwrap()` in command handlers
- Mutate props in Vue components
- Log sensitive data (keys, tokens, personal info)

**Always:**
- Use `anyhow::Context` for error descriptions
- Validate user input before processing
- Clear sensitive data from memory (`zeroize`)
- Test encryption/decryption round-trips
- Handle edge cases (empty input, null values, network errors)

### 4. Frontend Standards

**Component Structure:**
```vue
<script setup lang="ts">
// 1. Imports
import { ref, computed, onMounted, onUnmounted } from 'vue';
import { useProtocols } from '@/composables';

// 2. Props & Emits
interface Props {
    modelValue: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
    'update:modelValue': [value: string];
}>();

// 3. Composables & Stores
const { protocols, loading } = useProtocols();

// 4. Local State
const localValue = ref('');

// 5. Computed Properties
const filteredProtocols = computed(() => { /* ... */ });

// 6. Functions
function handleSubmit() { /* ... */ }

// 7. Lifecycle Hooks
onMounted(() => { /* setup */ });
onUnmounted(() => { /* cleanup */ });
</script>

<template>
  <!-- Always include ARIA labels -->
  <div class="component">
    <button
      @click="handleSubmit"
      :disabled="loading"
      :aria-busy="loading"
      aria-label="Submit form"
    >
      Submit
    </button>
  </div>
</template>

<style scoped>
/* Component-specific styles */
</style>
```

**Key Principles:**
- Use Composition API (`<script setup>`)
- TypeScript interfaces for all props
- ARIA labels on all interactive elements
- Cleanup in `onUnmounted`
- Never mutate props (use `emit` for updates)
- Composables for reusable logic

### 5. Tauri/Backend Standards

**Command Handler Pattern:**
```rust
#[tauri::command]
pub async fn create_resource(
    payload: CreateResourcePayload,
    state: State<'_, AppState>,
) -> Result<Resource, String> {
    // 1. Validate input
    if payload.name.trim().is_empty() {
        return Err("Name cannot be empty".to_string());
    }

    // 2. Business logic
    let resource = Resource {
        id: Uuid::new_v4().to_string(),
        name: payload.name,
        created_at: OffsetDateTime::now_utc(),
    };

    // 3. Persistence
    state.storage
        .upsert_resource(&resource)
        .map_err(|e| format!("Failed to save: {}", e))?;

    // 4. Return success
    Ok(resource)
}
```

**Key Principles:**
- Always return `Result<T, String>` for Tauri commands
- Validate input before processing
- Use `anyhow` internally, convert to `String` for IPC boundary
- Async when touching `tokio` or I/O
- Document public APIs with `///` comments

---

## Communication Style

### Commit Messages

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types:**
- `feat`: New feature
- `fix`: Bug fix
- `docs`: Documentation only
- `style`: Formatting, missing semicolons, etc.
- `refactor`: Code change that neither fixes a bug nor adds a feature
- `test`: Adding or updating tests
- `chore`: Updating build tasks, package manager configs, etc.

**Examples:**
```
feat(doses): add calendar view for dose history

Implements a monthly calendar view showing dose logs with color-coded
indicators for different peptides. Includes navigation controls and
click-to-view dose details.

Closes #42

fix(encryption): handle empty payload gracefully

Previously, encrypting an empty string would panic. Now returns an error
with a clear message.

docs: add architecture diagrams using Mermaid

Replaces ASCII art with proper Mermaid diagrams for better visualization
of system architecture, data flow, and encryption pipeline.
```

### Documentation

**When in Doubt:**
- Leave breadcrumbs in `docs/` directory
- Update `docs/future_self.md` with non-obvious decisions
- Comment complex algorithms inline
- Reference relevant issues/PRs in comments

**Surface Questions:**
- Don't guess - ask for clarification
- Document assumptions explicitly
- Propose multiple solutions when uncertain
- Privacy and correctness trump speed

---

## Current System State

### ✅ Completed Features

1. **Core Functionality**
   - Protocol management (CRUD)
   - Dose logging with calendar views
   - Supplier & inventory management
   - Literature search (PubMed, OpenAlex, Crossref)
   - Local AI summarization (Codex/Claude CLI)

2. **Backup System**
   - Manual and scheduled backups
   - Google Drive OAuth integration
   - Compression and cleanup policies
   - Restore with preview
   - Desktop notifications

3. **Security**
   - ChaCha20-Poly1305 encryption
   - macOS Keychain integration
   - Automatic key migration
   - OAuth PKCE for Google Drive

4. **State Management**
   - Pinia stores (protocols, doses, suppliers, literature, UI)
   - Vue composables for convenience
   - Optimistic updates with rollback
   - Smart caching (30s TTL)

5. **Error Handling**
   - Centralized error utility
   - User-friendly toast notifications
   - Contextual error messages
   - Global error boundary

### 🚧 In Progress

- Background reminders (LaunchAgent)
- Vial expiry notifications

### 📋 Planned

- Cloud restore (restore directly from Google Drive)
- Multi-cloud support (Dropbox, OneDrive)
- Backup encryption (user-managed passwords)
- Analytics dashboard
- Data export (CSV/JSON)
- Keyboard shortcuts

---

## Development Patterns

### Error Handling

```rust
// Rust
use anyhow::{Context, Result};

fn process_data(input: &str) -> Result<Output> {
    let parsed = parse(input)
        .context("Failed to parse input")?;

    let validated = validate(parsed)
        .context("Validation failed")?;

    Ok(transform(validated))
}
```

```typescript
// TypeScript
import { showErrorToast, showSuccessToast } from '@/utils/errorHandling';

async function saveProtocol(data: ProtocolData) {
    try {
        const result = await invoke("save_protocol", { data });
        showSuccessToast("Protocol saved successfully");
        return result;
    } catch (error: unknown) {
        showErrorToast(error, { operation: "save protocol" });
        throw error;
    }
}
```

### State Management

```typescript
// Store (frontend/src/stores/protocols.ts)
export const useProtocolStore = defineStore('protocols', () => {
    const protocols = ref<PeptideProtocol[]>([]);
    const loading = ref(false);
    const lastFetch = ref<number | null>(null);

    const isCacheValid = computed(() => {
        if (!lastFetch.value) return false;
        return Date.now() - lastFetch.value < 30000; // 30s TTL
    });

    async function fetchProtocols(force = false) {
        if (!force && isCacheValid.value && protocols.value.length > 0) {
            return protocols.value; // Return cached
        }

        loading.value = true;
        try {
            protocols.value = await listProtocols();
            lastFetch.value = Date.now();
        } catch (error) {
            showErrorToast(error);
        } finally {
            loading.value = false;
        }
    }

    return { protocols, loading, fetchProtocols };
});

// Composable (frontend/src/composables/useProtocols.ts)
export function useProtocols() {
    const store = useProtocolStore();

    return {
        protocols: computed(() => store.protocols),
        loading: computed(() => store.loading),
        refreshProtocols: store.fetchProtocols,
    };
}
```

### Component Lifecycle

```vue
<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue';

let intervalId: ReturnType<typeof setInterval> | null = null;

function startPolling() {
    intervalId = setInterval(() => {
        // Polling logic
    }, 5000);
}

onMounted(() => {
    startPolling();
    window.addEventListener('online', handleOnline);
});

onUnmounted(() => {
    // CRITICAL: Always cleanup!
    if (intervalId) {
        clearInterval(intervalId);
        intervalId = null;
    }
    window.removeEventListener('online', handleOnline);
});
</script>
```

---

## Resources

- **Architecture**: [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- **Developer Onboarding**: [docs/future_self.md](future_self.md)
- **Testing Guide**: [../TESTING.md](../TESTING.md)
- **Setup Guide**: [../SETUP.md](../SETUP.md)
- **User README**: [../README.md](../README.md)

---

## Success Criteria

An AI assistant is successful when:

1. **Code Quality**
   - All tests pass
   - Zero clippy warnings
   - Zero TypeScript errors
   - Proper error handling

2. **Documentation**
   - Clear commit messages
   - Updated relevant docs
   - Inline comments for complex logic
   - API documentation for public interfaces

3. **Security**
   - No secrets committed
   - Input validation present
   - Error messages don't leak sensitive info
   - Memory properly cleared

4. **User Experience**
   - Features work as expected
   - Error messages are helpful
   - Loading states present
   - ARIA labels on interactive elements

5. **Collaboration**
   - Changes are reviewable
   - Design decisions documented
   - Questions surfaced explicitly
   - Backwards compatibility maintained

---

**Follow this guide to maintain high velocity while preserving PepTrack's security-first, privacy-focused ethos.**

---

**Last Updated:** 2025-11-11
**Maintainer:** PepTrack Team
