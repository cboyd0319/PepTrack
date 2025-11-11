# PepTrack – Professional Peptide Management & Local AI Research Notes

PepTrack is a macOS-first desktop application built with Rust, Tauri, and Vue for securely tracking peptide protocols while leveraging on-device AI CLIs for literature summarization. All domain data stays local: protocols, dose logs, and cached literature are encrypted before touching disk, and summaries are generated via developer-installed Codex/Claude CLIs—no cloud calls from the app itself.

---

## Current Capabilities (Nov 2025)
- **Protocol storage:** create/list peptide protocols with notes, vial status, and target concentrations. Records are stored in SQLite (`peptrack.sqlite`) under the user's Application Support directory and encrypted with ChaCha20-Poly1305 envelope encryption.
- **Supplier & inventory management:** track peptide suppliers with contact info, manage vial inventory with expiry dates, cost tracking, batch/lot numbers, and vial status lifecycle.
- **Dose logging:** full UI for tracking doses with dates, amounts, and notes. Calendar views and history tracking included.
- **Backup system:** comprehensive backup and restore functionality with:
  - Manual backups (compressed or uncompressed JSON)
  - Scheduled automatic backups (hourly, daily, weekly)
  - Google Drive OAuth integration for cloud backups
  - Backup preview and restore with detailed results
  - Automatic backup cleanup with configurable retention policies
- **macOS Keychain integration:** encryption keys stored in macOS Keychain by default (macOS only), with automatic migration from file-based storage and secure OS-level protection.
- **Literature search:** integrated search across PubMed, OpenAlex, and Crossref APIs with result caching.
- **Local AI summarizer:** the Vue panel calls the Tauri command `summarize_text`, which invokes `peptrack-local-ai`. The orchestrator prefers Codex CLI (`gpt-5`), falling back to Claude CLI (`claude-haiku-4-5`), and returns Markdown or JSON summaries.
- **Notifications:** desktop notifications for backup success/failure with user-configurable preferences and granular controls.
- **Error handling:** intelligent error detection with user-friendly toast notifications and contextual suggestions for resolution.
- **Zero telemetry:** no outbound requests are made unless the user's CLI tools do so.

---

## Architecture Overview
```
┌──────────────┐        ┌──────────────────────────────┐        ┌────────────────────────┐
│  Vue 3 / Vite│<--IPC-->│ Tauri shell (src-tauri)       │<--API-->│ peptrack-core (Rust)   │
│  frontend    │        │  - Commands: list/save/summarize│      │  - SQLite + encryption │
└──────────────┘        │  - App state + key mgmt       │        └────────────────────────┘
                         │                                      ┌────────────────────────┐
                         └──────────────────────────────────────>│ peptrack-local-ai crate│
                                                                │  - Codex/Claude CLI     │
                                                                └────────────────────────┘
```

---

## Tech Stack & Tooling
- **Rust** `1.91.1` (see `rust-toolchain.toml`) with `rustfmt`, `clippy`, and `cargo-tauri 2.9.4`.
- **Tauri** `2.9.2`, `tauri-build 2.5.1`, `tauri-plugin-log 2.7.1`.
- **Frontend:** Vue `3.5.24`, Vite `7.2`, TypeScript `5.9`.
- **Crypto:** `chacha20poly1305 0.11.0-rc.2`, envelope keys persisted at `~/Library/Application Support/PepTrack/peptrack.key`.
- **Local AI:** optional Codex CLI (default) and Claude Code CLI fallback, both detected via `which`.

---

## Repository Layout
```
.
├── frontend/                # Vue UI + Vite config
│   ├── src/App.vue          # Entry screen wiring feature panels together
│   ├── src/components/      # ProtocolList, ProtocolForm, AiSummaryPanel
│   ├── src/api/             # Tauri invoke helpers
│   └── vitest.config.ts     # Component/unit test config
├── src-tauri/               # Tauri shell & commands
│   ├── src/lib.rs           # Builder + plugin wiring
│   ├── src/state.rs         # AppState bootstrap + key mgmt
│   └── src/commands/        # IPC handlers (protocols.rs, ai.rs)
├── crates/
│   ├── core/                # Storage manager, models, ChaCha20-Poly1305 encryption
│   └── local-ai/            # Codex/Claude CLI orchestrator + CLI parsing
├── docs/                  # Contributor notes (future-self guide, agent handbook)
└── AGENTS.md              # Quick contributor checklist
```

---

## Getting Started
1. **Install prerequisites**
   - `rustup default 1.91.1 && rustup component add rustfmt clippy`
   - Node.js ≥ 22 (project uses 25.1.0 locally)
   - `cargo install tauri-cli --version 2.9.4`
   - Optional: Codex CLI & Claude Code CLI if you want AI summaries to work.
2. **Install frontend deps**
   ```bash
   cd frontend && npm install
   ```
3. **Run in development**
   ```bash
   cargo tauri dev
   ```
4. **Test & lint**
   ```bash
   cargo fmt && cargo clippy --workspace
   cargo test --workspace
   (cd frontend && npm run build)
   (cd frontend && npm run test -- --run)
   ```

---

## Data & Security Notes
- Database path: `~/Library/Application Support/PepTrack/peptrack.sqlite`.
- **Keys (macOS):** Encryption keys are stored in macOS Keychain by default, providing OS-level encryption and access control. Automatic migration from file-based storage on first launch.
- **Keys (other platforms):** `peptrack.key` holds 32 random bytes in hex; managed by `ensure_key_material`.
- Encryption: `peptrack-core::EnvelopeEncryption` uses ChaCha20-Poly1305 with unique nonces per record.
- Logging: Tauri `log` plugin only attaches in debug builds; production builds omit it for privacy.
- Future work includes Secure Enclave integration and enforcing per-record metadata authentication.

---

## Local AI Orchestrator
- `peptrack-local-ai` detects CLI binaries during startup.
- The orchestrator walks a priority chain (Codex → Claude by default) and streams prompts via stdin/stdout.
- Responses are parsed from CLI JSON; if parsing fails, raw stdout is returned so the UI still shows something.
- Extendable to additional providers by implementing `LocalAiClient`.

---

## Roadmap & Next Steps
1. ~~**Supplier tracking + inventory:**~~ ✅ **COMPLETE** - Full supplier and inventory management with vial tracking, expiry dates, cost tracking, and batch/lot numbers.
2. ~~**macOS Keychain integration:**~~ ✅ **COMPLETE** - Encryption keys now stored in macOS Keychain by default with automatic migration from file-based storage.
3. **Background reminders:** create a LaunchAgent or Tauri sidecar to surface dose reminders and vial-expiry notifications even when the UI is closed.
4. **Cloud restore:** add ability to restore backups directly from Google Drive without downloading first.
5. **Multi-cloud support:** extend backup system to support Dropbox, OneDrive, and other cloud providers.
6. **Backup encryption:** add optional encryption for backup files at rest with user-managed passwords.

For contributor workflows and day-to-day commands, see `AGENTS.md` and `docs/peptrack_future_self.md`.
