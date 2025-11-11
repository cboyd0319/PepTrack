# Repository Guidelines

## Project Structure & Module Organization
- Root workspace houses Rust crates, frontend app, and the Tauri shell. Core logic lives in `crates/core` (SQLite + ChaCha20-Poly1305 encryption) and automation helpers in `crates/local-ai`. The Vue 3 client is under `frontend/`, while the Tauri host code sits in `src-tauri/`. Shared docs (including this file) are stored in `docs/`. Keep IDE-generated files out of version control; use `target/` and `frontend/node_modules/` for build artifacts only.

## Build, Test, and Development Commands
- `cargo fmt && cargo check` — format and compile all Rust crates; expect zero warnings in CI.
- `cargo test -p peptrack-core` — run backend unit and integration tests when touching storage or encryption layers.
- `(cd frontend && npm install && npm run build)` — install Node 22+ dependencies and produce the Vite build used by Tauri.
- `cargo tauri dev` — launch the desktop shell, serving the Vue app and invoking backend commands.

## Coding Style & Naming Conventions
- Rust: Edition 2021, `rustfmt` enforced. Favor explicit error contexts via `anyhow::Context`. Modules follow snake_case (e.g., `storage_manager.rs`), types use CamelCase, and async commands should live in `src-tauri/src/commands/`.
- Vue/TypeScript: Prettier defaults (2 spaces). Components in PascalCase, composables in `useX.ts`. Keep Tauri invoke channels in `frontend/src/api/peptrack.ts`.

## Testing Guidelines
- Rust tests rely on `cargo test`; name cases with the feature under test (`encrypt_roundtrip`). Mock filesystem access where possible. No flaky sleeps.
- Frontend tests (when added) should live in `frontend/tests/` using Vitest; mirror component names (`App.spec.ts`).
- Before merging, run `cargo check` and relevant targeted tests for files you modified.

## Commit & Pull Request Guidelines
- Follow concise, present-tense commit subjects (`Fix ChaCha nonce derivation`). Group related changes; avoid formatting-only commits unless intentional.
- Pull requests need: summary of changes, testing notes (`cargo check`, `npm run build`), and screenshots/GIFs for UI updates. Link to Jira/GitHub issues when applicable and call out security-sensitive modifications (key handling, encryption routines).

## Security & Configuration Tips
- Encryption keys live at `~/Library/Application Support/PepTrack/peptrack.key`; never commit real keys. Use `.env` or mocked providers in tests.
- When manipulating `target/`, ensure ownership stays with the `chad` user to avoid permission errors during builds.
