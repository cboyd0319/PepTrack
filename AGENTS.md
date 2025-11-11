# Repository Guidelines

## Quick Orientation
PepTrack lives entirely under `/Users/chad/Documents/GitHub/PepTrack`. Workspace structure:
- `src-tauri/`: `lib.rs` sets up plugins, `state.rs` boots AppState/keys, `commands/` hosts IPC handlers (`protocols.rs`, `ai.rs`).
- `crates/core`: SQLite storage, envelope encryption, and unit tests for protocol round-trips.
- `crates/local-ai`: CLI orchestrator with provider-chain tests.
- `frontend/`: Vue app composed of `App.vue` + `components/` (ProtocolList, ProtocolForm, AiSummaryPanel) plus Vitest config/tests.
Docs live in `docs/`, with persona/future-self briefs; keep them updated whenever behavior changes.

## Build & Dev Commands
- `cargo fmt && cargo clippy --workspace --all-targets` — formatting + lint pass (Rust 1.91.1).
- `cargo test --workspace` — exercises storage + encryption + orchestrator tests.
- `cargo tauri dev` — runs the desktop shell; automatically launches the Vite dev server from `frontend/`.
- `(cd frontend && npm install && npm run build)` — install Node 22+ deps and produce the production bundle.
- `(cd frontend && npm run test -- --run)` — run Vitest component/unit tests.
- Optional: install Codex CLI (`gpt-5`) and Claude Code CLI (`claude-haiku-4-5`) so the AI summarizer panel works.

## Coding Style & Conventions
- Rust edition 2021; keep modules snake_case, structs CamelCase, and surface errors with `anyhow::Context`. Avoid unwraps in command handlers—propagate `Result`.
- Vue/TS: 2-space indent, `<script setup>` components, PascalCase components, and typed API wrappers in `frontend/src/api/`.
- When touching Tauri commands, add them to `tauri::generate_handler!` and declare payload structs with `#[serde(rename_all = "camelCase")]`.

## Testing & Verification
- Always run `cargo test -p peptrack-core` after schema/encryption changes; inspect the SQLite file if migrations are added.
- Frontend verification today is manual; smoke-test protocol creation/listing and the AI summary flow before merging.
- Capture CLI output when Codex/Claude fail; their stderr is crucial for debugging.

## Security & Environment Notes
- Data lives in `~/Library/Application Support/PepTrack/peptrack.sqlite`; keys are stored next to it as `peptrack.key` (32-byte hex). Never check these files into git.
- If `cargo check` complains about `target/` permissions, ensure the directory is owned by `chad` (see future-self doc).
- Document any hacks or local scripts in `docs/` so the next agent can reproduce your setup.
