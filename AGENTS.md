# Repository Guidelines

## Quick Orientation
PepTrack lives entirely under `/Users/chad/Documents/GitHub/PepTrack`. The workspace contains three crates (`src-tauri`, `crates/core`, `crates/local-ai`) plus the Vue frontend in `frontend/`. Core responsibilities: `peptrack-core` handles encrypted SQLite storage, `peptrack-local-ai` shells out to Codex or Claude CLIs, and `src-tauri` wires everything into the desktop app. Docs live in `docs/`, including the future-self handoff.

## Build & Dev Commands
- `cargo fmt && cargo clippy --workspace` — formatting + lint pass (Rust 1.91.1).
- `cargo test -p peptrack-core` — exercises storage + encryption logic.
- `cargo tauri dev` — runs the desktop shell; automatically launches the Vite dev server from `frontend/`.
- `(cd frontend && npm install && npm run build)` — install Node 22+ deps and produce the production bundle.
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
