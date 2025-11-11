# PepTrack - Quick Reference for Contributors

**For:** Human developers and AI assistants
**Last Updated:** 2025-11-11

---

## Quick Orientation

### Repository Structure

```
PepTrack/
├── src-tauri/          # Rust Tauri backend
│   ├── src/lib.rs     # App initialization
│   ├── src/state.rs   # AppState bootstrap
│   └── src/commands/  # IPC handlers (11 modules)
├── crates/            # Rust library crates
│   ├── core/          # Storage + encryption
│   ├── local-ai/      # CLI orchestrator
│   └── literature/    # API fetchers
├── frontend/          # Vue 3 SPA
│   ├── src/App.vue
│   ├── src/components/
│   ├── src/stores/    # Pinia state management
│   └── src/api/       # Tauri IPC wrappers
└── docs/              # Technical documentation
```

### Data Storage

- **Database**: `~/Library/Application Support/PepTrack/peptrack.sqlite` (encrypted)
- **Encryption Key**: macOS Keychain (primary) or `peptrack.key` (fallback)
- **Backups**: User-selected directory

**⚠️ Never commit encryption keys or OAuth secrets!**

---

## Build & Development Commands

### Rust Backend

```bash
# Format code
cargo fmt

# Lint (must pass with zero warnings)
cargo clippy --workspace --all-targets

# Run tests
cargo test --workspace

# Run specific crate tests
cargo test -p peptrack-core

# Development mode
cargo tauri dev
```

### Frontend

```bash
cd frontend

# Install dependencies
npm install

# Build production bundle
npm run build

# Run component tests
npm run test -- --run

# Type check
vue-tsc -b
```

### Full Build

```bash
# Production release
cargo tauri build
```

---

## Coding Style & Conventions

### Rust

- **Edition**: 2021
- **Error Handling**: Use `anyhow::Result` and add `.context()` to error chains
- **No Panics**: Prefer `?` over `unwrap()` in production code
- **Naming**: `snake_case` modules/functions, `CamelCase` structs/enums
- **Documentation**: `///` comments for public APIs

```rust
use anyhow::{Context, Result};

pub fn process_data(input: &str) -> Result<Output> {
    let parsed = parse(input)
        .context("Failed to parse input")?;
    Ok(transform(parsed))
}
```

### TypeScript/Vue

- **Indentation**: 2 spaces
- **Components**: PascalCase, use `<script setup>` with TypeScript
- **Functions**: camelCase
- **Props**: Never mutate, use `emit` for updates
- **ARIA**: All interactive elements need labels

```vue
<script setup lang="ts">
interface Props {
    modelValue: string;
}

const props = defineProps<Props>();
const emit = defineEmits<{
    'update:modelValue': [value: string];
}>();
</script>

<template>
    <button
        @click="handleClick"
        :disabled="loading"
        :aria-busy="loading"
        aria-label="Submit form"
    >
        Submit
    </button>
</template>
```

### Commit Messages

```
<type>(<scope>): <subject>

<body>

<footer>
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

**Example**:
```
feat(doses): add calendar view for dose history

Implements monthly calendar with color-coded peptide indicators.
Includes navigation and click-to-view dose details.

Closes #42
```

---

## Testing & Verification

### Pre-Commit Checklist

- [ ] `cargo fmt` (format Rust code)
- [ ] `cargo clippy --workspace --all-targets` (zero warnings)
- [ ] `cargo test --workspace` (all tests pass)
- [ ] `cd frontend && npm run build` (frontend builds)
- [ ] `cd frontend && npm run test -- --run` (frontend tests pass)

### Test Coverage Goals

- **Rust**: Unit tests for all storage/encryption functions
- **Frontend**: Component tests for all major components
- **Integration**: Test critical user flows

---

## Security & Best Practices

### Never

- ❌ Commit encryption keys (`peptrack.key`)
- ❌ Commit OAuth secrets
- ❌ Access `peptrack.key` directly (use `KeyProvider` trait)
- ❌ Use `unwrap()` in command handlers
- ❌ Mutate props in Vue components
- ❌ Log sensitive data (keys, tokens, personal info)

### Always

- ✅ Use `anyhow::Context` for errors
- ✅ Validate user input
- ✅ Clear sensitive data from memory (`zeroize`)
- ✅ Test encryption round-trips
- ✅ Clean up resources in `onUnmounted` hooks
- ✅ Add ARIA labels to interactive elements

---

## Common Tasks

### Adding a New Tauri Command

1. **Create handler** in `src-tauri/src/commands/<feature>.rs`:
   ```rust
   #[tauri::command]
   pub async fn my_command(
       payload: MyPayload,
       state: State<'_, AppState>,
   ) -> Result<MyResult, String> {
       // Implementation
   }
   ```

2. **Register in** `src-tauri/src/lib.rs`:
   ```rust
   .invoke_handler(tauri::generate_handler![my_command])
   ```

3. **Add TypeScript wrapper** in `frontend/src/api/peptrack.ts`:
   ```typescript
   export async function myCommand(payload: MyPayload): Promise<MyResult> {
       return await invoke("my_command", { payload });
   }
   ```

### Adding a New Pinia Store

1. **Create store** in `frontend/src/stores/<feature>.ts`:
   ```typescript
   export const useMyStore = defineStore('myStore', () => {
       const data = ref<MyData[]>([]);
       const loading = ref(false);

       async function fetch() {
           loading.value = true;
           try {
               data.value = await myCommand();
           } catch (error) {
               showErrorToast(error);
           } finally {
               loading.value = false;
           }
       }

       return { data, loading, fetch };
   });
   ```

2. **Create composable** in `frontend/src/composables/use<Feature>.ts`:
   ```typescript
   export function useMyFeature() {
       const store = useMyStore();
       return {
           data: computed(() => store.data),
           loading: computed(() => store.loading),
           refresh: store.fetch,
       };
   }
   ```

---

## Troubleshooting

### `cargo check` fails with permission errors
```bash
sudo rm -rf target
mkdir target
```

### Frontend build fails
```bash
cd frontend
rm -rf node_modules package-lock.json
npm install
```

### "AI not available"
```bash
# Install Codex or Claude CLI
npm install -g codex-cli
# OR
curl https://code.claude.com/install.sh | bash
```

---

## Documentation

- **[README.md](README.md)** - Project overview
- **[SETUP.md](SETUP.md)** - User setup guide
- **[TESTING.md](TESTING.md)** - Testing scenarios
- **[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)** - Detailed architecture
- **[docs/future_self.md](docs/future_self.md)** - Developer onboarding
- **[docs/ai_assistant_persona.md](docs/ai_assistant_persona.md)** - AI collaboration guide

---

## Quick Reference

| Task | Command |
|------|---------|
| **Start dev server** | `cargo tauri dev` |
| **Run all tests** | `cargo test --workspace` |
| **Lint Rust** | `cargo clippy --workspace --all-targets` |
| **Format Rust** | `cargo fmt` |
| **Build frontend** | `cd frontend && npm run build` |
| **Test frontend** | `cd frontend && npm run test -- --run` |
| **Production build** | `cargo tauri build` |

---

**Need more details? Check the docs/ directory for comprehensive guides.**
