# PepTrack Future-Self Instructions (Updated Nov 10 2025)

## Environment & Toolchain
- Repo: `/Users/chad/Documents/GitHub/PepTrack` (real git remote → `origin/main`). Never work out of `~/.Trash` or temp copies.
- Rust `1.91.1` pinned via `rust-toolchain.toml`; install `rustfmt`, `clippy`, and `cargo-tauri 2.9.4`.
- Node ≥ 22 (local dev uses 25.1.0); install pnpm/nvm only if needed, but `npm` is sufficient today.
- Tauri crates: `tauri 2.9.2`, `tauri-build 2.5.1`, `tauri-plugin-log 2.7.1`.
- Frontend: Vue `3.5.24`, Vite `7.2`, TypeScript `5.9`.
- Crypto: `chacha20poly1305 0.11.0-rc.2` (warnings resolved); ensure `cargo update` doesn’t downgrade.

## Project Snapshot
- `crates/core`: `StorageManager` encrypts JSON blobs with ChaCha20-Poly1305 and persists them in `~/Library/Application Support/PepTrack/peptrack.sqlite`. Tables exist for protocols, dose logs, and literature cache—even though only protocols are exposed in the UI.
- `crates/local-ai`: Detects Codex CLI (`gpt-5`) and Claude CLI (`claude-haiku-4-5`). `LocalAiOrchestrator` walks a preferred chain and parses JSON responses.
- `src-tauri`: Boots the data dir, creates/loads `peptrack.key`, registers Tauri commands (`list_protocols`, `save_protocol`, `summarize_text`), and exposes state to the frontend.
- `frontend`: Single-page Vue app with protocol list/form and an AI summary panel that surfaces orchestrator output.

## Active Priorities
1. **Supplier & Inventory tracking:** extend schema + models, add CRUD commands, and build Vue UI (cost per mg, vial state, supplier metadata).
2. **Dose logging UX:** wire `DoseLog` into Tauri commands, design timeline/history components, and display aggregated dosage metrics.
3. **Literature ingestion:** implement real fetchers (PubMed/OpenAlex/Crossref). Cache normalized results in `literature_cache` and surface them in the UI before invoking AI summarization.
4. **Keychain-backed secrets:** replace `peptrack.key` on disk with macOS Keychain storage; provide migration utility.
5. **Notifications/background tasks:** design LaunchAgent or native Tauri sidecar for reminders (dose schedules, vial expiry, supplier restock).
6. **Testing & telemetry hardening:** add unit/integration tests around storage + orchestrator, and document how to inspect CLI stderr when summaries fail.

## Daily Workflow
1. `cd /Users/chad/Documents/GitHub/PepTrack && git pull --rebase` (keep local branches up to date).
2. Verify tool versions: `rustup show active-toolchain`, `node -v`.
3. Install deps if needed: `cd frontend && npm install`.
4. During dev use `cargo tauri dev`; for API/backend-only work run `cargo fmt && cargo clippy --workspace && cargo test -p peptrack-core`.
5. When touching the frontend, run `(cd frontend && npm run build)` before opening a PR.
6. Document non-obvious workarounds in `docs/` and keep `AGENTS.md` aligned.

## Useful Commands
```bash
cargo fmt
cargo clippy --workspace --all-targets
cargo test -p peptrack-core
cargo tauri dev
(cd frontend && npm install && npm run build)
```

## Troubleshooting Notes
- If `cargo check` complains about `target/` permissions, ensure the directory exists and is owned by `chad` (`sudo rm -rf target && mkdir target && chown -R chad target`).
- Missing Codex/Claude CLIs will cause the AI panel to show “No available local AI providers.” Install those tools or stub responses before demos.
- Keep an eye on `Cargo.lock` after dependency bumps—`chacha20poly1305` is currently on an RC; don’t downgrade without checking release notes.
