# PepTrack Future-Self Instructions (Updated Nov 10 2025)

## Environment & Toolchain
- Repo: `/Users/chad/Documents/GitHub/PepTrack` (real git remote → `origin/main`). Never work out of `~/.Trash` or temp copies.
- Rust `1.91.1` pinned via `rust-toolchain.toml`; install `rustfmt`, `clippy`, and `cargo-tauri 2.9.4`.
- Node ≥ 22 (local dev uses 25.1.0); install pnpm/nvm only if needed, but `npm` is sufficient today.
- Tauri crates: `tauri 2.9.2`, `tauri-build 2.5.1`, `tauri-plugin-log 2.7.1`.
- Frontend: Vue `3.5.24`, Vite `7.2`, TypeScript `5.9`.
- Crypto: `chacha20poly1305 0.11.0-rc.2` (warnings resolved); ensure `cargo update` doesn’t downgrade.

## Project Snapshot
- `crates/core`: `StorageManager` encrypts JSON blobs with ChaCha20-Poly1305 and persists them in `~/Library/Application Support/PepTrack/peptrack.sqlite`. Full schema with protocols, dose logs, and literature cache—all exposed in the UI. Includes comprehensive unit tests with `tempfile`.
- `crates/local-ai`: Detects Codex CLI (`gpt-5`) and Claude CLI (`claude-haiku-4-5`). `LocalAiOrchestrator` walks a preferred chain and exposes `provider_chain()`; tests assert fallback order.
- `src-tauri/src/commands/`: Complete command suite including:
  - `protocols.rs` - Protocol CRUD operations
  - `ai.rs` - AI summarization via local CLIs
  - `backup.rs` - Manual backup creation
  - `scheduler_v2.rs` - Scheduled automatic backups with tokio runtime
  - `drive.rs` - Google Drive OAuth and cloud backup uploads
  - `restore.rs` - Backup restoration with preview
- `frontend`: Vue 3 app with comprehensive UI:
  - Protocol management (`ProtocolList`, `ProtocolForm`)
  - Dose tracking with calendar views (`DoseTracker`)
  - Literature search across PubMed/OpenAlex/Crossref (`LiteratureSearch`)
  - AI summarization panel (`AiSummaryPanel`)
  - Unified Settings with tabs (`Settings`, `ScheduledBackup`, `GoogleDriveBackup`, `BackupExport`, `RestoreBackup`, `NotificationPreferences`)
  - Global toast notification system (`Toast`, `errorHandling.ts`)
  - Vitest test suite

## Completed Features
- ✅ **Dose logging UX:** Full UI with calendar views, history tracking, and date/amount/notes logging
- ✅ **Literature ingestion:** PubMed/OpenAlex/Crossref search with result caching in `literature_cache`
- ✅ **Backup & Restore System:**
  - Manual and scheduled backups (hourly/daily/weekly)
  - Google Drive OAuth integration with cloud uploads
  - Backup compression (gzip), cleanup policies (keep last N / delete older than)
  - Restore functionality with preview before restore
  - Desktop notifications for backup events (success/failure)
- ✅ **Error Handling:** Intelligent error detection with user-friendly toast notifications and contextual suggestions
- ✅ **Testing Documentation:** Comprehensive edge case documentation in `TESTING.md` with 80+ scenarios

## Active Priorities
1. **Supplier & Inventory tracking:** extend schema + models, add CRUD commands, and build Vue UI (cost per mg, vial state, supplier metadata).
2. **Keychain-backed secrets:** replace `peptrack.key` on disk with macOS Keychain storage; provide migration utility (see `docs/keychain_migration_plan.md`).
3. **Background reminders:** design LaunchAgent or native Tauri sidecar for dose reminders and vial-expiry notifications when app is closed.
4. **Cloud restore:** add ability to restore backups directly from Google Drive without manual download.
5. **Multi-cloud support:** extend backup system to Dropbox, OneDrive, and other providers.
6. **Backup encryption:** add optional password-based encryption for backup files at rest.

## Daily Workflow
1. `cd /Users/chad/Documents/GitHub/PepTrack && git pull --rebase` (keep local branches up to date).
2. Verify tool versions: `rustup show active-toolchain`, `node -v`.
3. Install deps if needed: `cd frontend && npm install`.
4. During dev use `cargo tauri dev`; for backend-only work run `cargo fmt && cargo clippy --workspace --all-targets && cargo test --workspace`.
5. When touching the frontend, run `(cd frontend && npm run build)` and `(cd frontend && npm run test -- --run)` before opening a PR.
6. Document non-obvious workarounds in `docs/` and keep `AGENTS.md` + the persona brief aligned.

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
