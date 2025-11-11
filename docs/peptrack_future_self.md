# PepTrack Future-Self Instructions

## Repo & Environment
- Work exclusively in `/Users/chad/Documents/GitHub/PepTrack` (real git repo synced to origin/main).
- Toolchain versions:
  - Rust 1.91.1 (`rust-toolchain.toml` pinned; install components rustfmt + clippy).
  - Node 22+/npm for the Vue frontend (already using Node 25.1.0).
  - `cargo-tauri` CLI 2.9.4 installed globally.
  - Tauri crates currently set to `tauri = 2.9.2`, `tauri-build = 2.5.1`, `tauri-plugin-log = 2.7.1`.
  - Vue 3.5.24 + Vite 7.2 (`frontend/`).
- Two workspace crates under `crates/`: `core` (SQLite + encryption) and `local-ai` (Codex/Claude CLI orchestrator). Tauri shell is `src-tauri`.

## Current Code State
- Backend (Rust): `StorageManager` encrypts SQLite blobs via ChaCha20-Poly1305 with keys stored at `~/Library/Application Support/PepTrack/peptrack.key`. CLI orchestrator prefers Codex CLI (`gpt-5`) and falls back to Claude CLI (`claude-haiku-4-5`).
- Frontend (Vue): `frontend/src/App.vue` shows protocol list, creation form, and AI summary panel using `@tauri-apps/api` invokes defined in `frontend/src/api/peptrack.ts`.
- README has updated architecture/stack notes. Vite config adjusted for Tauri dev server.

## Outstanding Issues / TODOs
1. **Encryption warnings**: `crates/core/src/encryption.rs` still uses deprecated `GenericArray::from_slice`. Need to rework using `Key::from_slice` but ensure correct borrow semantics without triggering type errors. After editing, run `cargo fmt` + `cargo check` (no warnings ideally).
2. **Filesystem permissions**: Running `cargo check` as regular user may still hit "Operation not permitted" for `target`. Ensure `target/` is owned by `chad` before building (`sudo rm -rf target && mkdir target && chown -R chad target` if necessary).
3. **Git cleanliness**: After fixes, stage from `/Users/chad/Documents/GitHub/PepTrack` only. Avoid editing inside `~/.Trash/PepTrack`.
4. **Node deps**: Reinstall frontend deps with `npm install` inside `frontend/` if `node_modules` was removed before commit.
5. **Future enhancements** (from README): supplier tracking, notifications/daemons, real literature API integrations, Keychain-backed secrets.

## Working Steps When Returning
1. `cd /Users/chad/Documents/GitHub/PepTrack` and `git pull` to ensure synced.
2. Verify Node/Rust versions (`rustup show`, `node -v`).
3. Reinstall frontend deps (`npm install` under `frontend/`) and re-create `target/` if missing by running `cargo check`.
4. Fix the ChaCha encryption warnings as top priority; confirm `cargo check` passes without warnings.
5. Once clean, continue with new tasks (e.g., implement supplier tracking or background tasks) following workspace structure.

## Reference Commands
```bash
cd /Users/chad/Documents/GitHub/PepTrack
cargo fmt
cargo check
cd frontend && npm install && npm run build
cargo tauri dev
```

Keep logs of any permission-related workarounds so we don’t need elevated commands repeatedly.
