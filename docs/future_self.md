# PepTrack - Developer Onboarding Guide

**Last Updated:** 2025-11-11
**For:** Future developers and AI assistants working on PepTrack

---

## Quick Start

Welcome to PepTrack! This guide will help you get up to speed quickly.

### Environment Setup

1. **Prerequisites**
   ```bash
   # Rust toolchain (1.91.1 pinned via rust-toolchain.toml)
   rustup default 1.91.1
   rustup component add rustfmt clippy

   # Tauri CLI
   cargo install tauri-cli --version 2.9.4

   # Node.js (≥22, tested with 25.1.0)
   node -v  # Should be 22+
   ```

2. **Clone and Setup**
   ```bash
   git clone <repository-url>
   cd PepTrack
   cd frontend && npm install && cd ..
   ```

3. **Verify Build**
   ```bash
   cargo fmt
   cargo clippy --workspace --all-targets
   cargo test --workspace
   cd frontend && npm run build && npm run test -- --run
   ```

---

## Project Structure

```
PepTrack/
├── crates/                 # Rust library crates
│   ├── core/              # Storage, encryption, models
│   ├── local-ai/          # AI CLI orchestration
│   └── literature/        # Literature API fetchers
│
├── src-tauri/             # Tauri application
│   ├── src/
│   │   ├── lib.rs        # App initialization
│   │   ├── state.rs      # AppState bootstrap
│   │   └── commands/     # IPC handlers (11 modules)
│   └── tauri.conf.json   # Configuration
│
├── frontend/              # Vue 3 SPA
│   ├── src/
│   │   ├── App.vue       # Main application
│   │   ├── components/   # 18 Vue components
│   │   ├── stores/       # 5 Pinia stores
│   │   ├── composables/  # Vue composables
│   │   ├── api/          # Tauri IPC wrappers
│   │   └── utils/        # Helper functions
│   └── vitest.config.ts  # Test configuration
│
└── docs/                  # Documentation
    ├── ARCHITECTURE.md    # Detailed architecture
    ├── future_self.md     # This file
    └── ai_assistant_persona.md
```

---

## Technology Stack

### Backend (Rust)
- **Tauri 2.9.2** - Desktop app framework
- **SQLite** - Local encrypted database
- **ChaCha20-Poly1305** - AEAD encryption
- **Tokio 1.41.1** - Async runtime
- **Security-framework 2.11** - macOS Keychain integration

### Frontend (Vue 3)
- **Vue 3.5.24** - UI framework (Composition API)
- **Vite 7.2** - Build tool
- **TypeScript 5.9** - Type safety
- **Pinia 3.0.4** - State management
- **Vitest 2.1.4** - Component testing

---

## Key Concepts

### 1. Data Storage

**Location:**
- Database: `~/Library/Application Support/PepTrack/peptrack.sqlite`
- Encryption key: macOS Keychain (primary) or `peptrack.key` (fallback)

**Encryption:**
- Algorithm: ChaCha20-Poly1305 (AEAD)
- All domain data encrypted as BLOBs
- Unique 12-byte nonce per record
- Automatic key migration to Keychain on first launch

**Schema:**
```sql
protocols (id PK, name, payload BLOB, updated_at)
dose_logs (id PK, protocol_id FK, payload BLOB, logged_at)
suppliers (id PK, name, payload BLOB, updated_at)
inventory (id PK, protocol_id FK, supplier_id FK, payload BLOB, updated_at)
literature_cache (id PK, source, payload BLOB, indexed_at)
```

### 2. State Management

**Pinia Stores:**
- `useProtocolStore()` - Protocol CRUD with 30s caching
- `useDoseStore()` - Dose logging and history
- `useSupplierStore()` - Supplier & inventory management
- `useLiteratureStore()` - Literature search & AI summaries
- `useUIStore()` - Global UI state (loading, modals, online status)

**Patterns:**
- Optimistic updates with automatic rollback
- Smart caching (30s TTL for protocols)
- Centralized error handling via toast notifications

### 3. IPC Commands

Commands are organized by domain in `src-tauri/src/commands/`:

| Module | Key Commands |
|--------|--------------|
| `protocols.rs` | `list_protocols`, `save_protocol` |
| `doses.rs` | `log_dose`, `list_dose_logs` |
| `suppliers.rs` | CRUD for suppliers & inventory |
| `ai.rs` | `summarize_text`, `check_ai_availability` |
| `literature.rs` | `search_literature` (PubMed/OpenAlex/Crossref) |
| `backup.rs` | `export_backup_data` |
| `restore.rs` | `preview_backup`, `restore_from_backup` |
| `scheduler_v2.rs` | Scheduled backup automation |
| `drive.rs` | Google Drive OAuth & uploads |

### 4. Security

**Never:**
- Commit encryption keys or OAuth secrets
- Access `peptrack.key` directly (use `KeyProvider` abstraction)
- Use unwrap() in command handlers (propagate `Result`)
- Mutate props in Vue components

**Always:**
- Use `anyhow::Context` for error descriptions
- Clean up resources in `onUnmounted` hooks
- Validate user input before IPC calls
- Test encryption/decryption round-trips

---

## Development Workflow

### Daily Commands

```bash
# Start development server
cargo tauri dev

# Run tests
cargo test --workspace
cd frontend && npm run test -- --run

# Lint and format
cargo fmt
cargo clippy --workspace --all-targets

# Build production release
cargo tauri build
```

### Adding a New Feature

1. **Backend (Rust)**
   ```rust
   // 1. Add domain model to crates/core/src/models.rs
   #[derive(Debug, Clone, Serialize, Deserialize)]
   pub struct NewModel {
       pub id: String,
       // fields...
   }

   // 2. Add storage methods to crates/core/src/db.rs
   pub fn upsert_new_model(&self, model: &NewModel) -> Result<NewModel> {
       // encryption + SQLite
   }

   // 3. Create command handler in src-tauri/src/commands/new_feature.rs
   #[tauri::command]
   pub async fn create_new_model(
       payload: CreateNewModelPayload,
       state: State<'_, AppState>,
   ) -> Result<NewModel, String> {
       // business logic
   }

   // 4. Register in src-tauri/src/lib.rs
   .invoke_handler(tauri::generate_handler![create_new_model])
   ```

2. **Frontend (Vue)**
   ```typescript
   // 1. Add TypeScript interface to frontend/src/api/peptrack.ts
   export interface NewModel {
       id: string;
       // fields...
   }

   export async function createNewModel(payload: CreatePayload): Promise<NewModel> {
       return await invoke("create_new_model", { payload });
   }

   // 2. Create Pinia store in frontend/src/stores/newFeature.ts
   export const useNewFeatureStore = defineStore('newFeature', () => {
       const items = ref<NewModel[]>([]);
       const loading = ref(false);

       async function fetchItems() {
           loading.value = true;
           try {
               items.value = await listNewModels();
           } catch (error) {
               showErrorToast(error);
           } finally {
               loading.value = false;
           }
       }

       return { items, loading, fetchItems };
   });

   // 3. Create composable in frontend/src/composables/useNewFeature.ts
   export function useNewFeature() {
       const store = useNewFeatureStore();
       return {
           items: computed(() => store.items),
           loading: computed(() => store.loading),
           refresh: store.fetchItems,
       };
   }

   // 4. Create Vue component
   <script setup lang="ts">
   import { useNewFeature } from '@/composables';

   const { items, loading, refresh } = useNewFeature();

   onMounted(() => refresh());
   </script>
   ```

3. **Testing**
   ```rust
   // Rust unit test
   #[cfg(test)]
   mod tests {
       use super::*;

       #[test]
       fn test_new_model_roundtrip() {
           let storage = create_test_storage();
           let model = NewModel { /* ... */ };

           let saved = storage.upsert_new_model(&model).unwrap();
           let loaded = storage.get_new_model(&saved.id).unwrap();

           assert_eq!(saved.id, loaded.id);
       }
   }
   ```

   ```typescript
   // Vue component test
   import { mount } from '@vue/test-utils';
   import { createPinia, setActivePinia } from 'pinia';

   describe('NewFeatureComponent', () => {
       beforeEach(() => {
           setActivePinia(createPinia());
       });

       it('renders items', async () => {
           const wrapper = mount(NewFeatureComponent);
           await wrapper.vm.$nextTick();
           expect(wrapper.find('.item').exists()).toBe(true);
       });
   });
   ```

---

## Troubleshooting

### Common Issues

**Problem:** `cargo check` fails with permission errors
```bash
# Solution: Fix target directory ownership
sudo rm -rf target
mkdir target
```

**Problem:** Frontend build fails with TypeScript errors
```bash
# Solution: Clear node_modules and reinstall
cd frontend
rm -rf node_modules package-lock.json
npm install
```

**Problem:** "AI not available" message
```bash
# Solution: Install Codex or Claude CLI
npm install -g codex-cli
# OR
curl https://code.claude.com/install.sh | bash
```

**Problem:** Keychain migration fails
```bash
# Solution: Generate new key (old data will be unreadable)
rm ~/Library/Application\ Support/PepTrack/peptrack.key
# Restart app - will generate new key
```

---

## Code Style Guidelines

### Rust
- Edition 2021
- Use `anyhow::Result` for errors
- Add `context()` to all `Result` chains
- Prefer `?` over `unwrap()`
- Run `cargo fmt` and `cargo clippy` before committing
- Document public APIs with `///` comments

### TypeScript/Vue
- 2-space indentation
- Use Composition API (`<script setup>`)
- PascalCase for components
- camelCase for functions/variables
- Always define prop types with TypeScript
- Use `const` by default, `let` only when needed

### Commit Messages
```
<type>(<scope>): <subject>

<body>

<footer>
```

Types: `feat`, `fix`, `docs`, `style`, `refactor`, `test`, `chore`

Examples:
- `feat(doses): add calendar view for dose history`
- `fix(encryption): handle empty nonce gracefully`
- `docs: update architecture diagrams`

---

## Resources

- **Architecture**: See [docs/ARCHITECTURE.md](ARCHITECTURE.md)
- **Testing**: See [../TESTING.md](../TESTING.md)
- **Setup**: See [../SETUP.md](../SETUP.md)
- **AI Persona**: See [ai_assistant_persona.md](ai_assistant_persona.md)

---

## Current Focus Areas

Based on the roadmap, these are the active development priorities:

1. **Background Reminders** - LaunchAgent for dose notifications
2. **Cloud Restore** - Restore backups directly from Google Drive
3. **Multi-Cloud Support** - Dropbox/OneDrive integration
4. **Analytics Dashboard** - Usage statistics and insights
5. **Data Export** - CSV/JSON export for all data types

---

## Getting Help

1. Check documentation in `docs/`
2. Review similar existing implementations
3. Read test files for usage examples
4. Check commit history for context: `git log --oneline`

---

**Welcome to the team! Happy coding! 🎉**
