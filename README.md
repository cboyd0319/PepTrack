# PepTrack – Professional Peptide Management & Local AI Research Notes

PepTrack is a macOS-first desktop application built with Rust, Tauri, and Vue for securely tracking peptide protocols while leveraging on-device AI CLIs for literature summarization. All domain data stays local: protocols, dose logs, and cached literature are encrypted before touching disk, and summaries are generated via developer-installed Codex/Claude CLIs—no cloud calls from the app itself.

---

## Current Capabilities (Nov 2025)
- **Protocol storage:** create/list peptide protocols with notes, vial status, and target concentrations. Records are stored in SQLite (`peptrack.sqlite`) under the user’s Application Support directory and encrypted with ChaCha20-Poly1305 envelope encryption.
- **Dose & literature schemas:** backend structs and tables exist for dose logs and literature cache, ready for future UI wiring.
- **Local AI summarizer:** the Vue panel calls the Tauri command `summarize_text`, which invokes `peptrack-local-ai`. The orchestrator prefers Codex CLI (`gpt-5`), falling back to Claude CLI (`claude-haiku-4-5`), and returns Markdown or JSON summaries.
- **Zero telemetry:** no outbound requests are made unless the user’s CLI tools do so.

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
├── frontend/              # Vue UI + Vite config
│   ├── src/App.vue        # Current screens (protocol list, creator, AI summary)
│   └── src/api/           # Tauri invoke helpers
├── src-tauri/             # Tauri shell & commands
│   └── src/lib.rs         # State setup, encryption key bootstrap, IPC handlers
├── crates/
│   ├── core/              # Storage manager, models, ChaCha20-Poly1305 envelope encryption
│   └── local-ai/          # Codex/Claude CLI orchestrator
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
   cargo test -p peptrack-core
   (cd frontend && npm run build)
   ```

---

## Data & Security Notes
- Database path: `~/Library/Application Support/PepTrack/peptrack.sqlite`.
- Keys: `peptrack.key` holds 32 random bytes in hex; managed by `ensure_key_material`.
- Encryption: `peptrack-core::EnvelopeEncryption` uses ChaCha20-Poly1305 with unique nonces per record.
- Logging: Tauri `log` plugin only attaches in debug builds; production builds omit it for privacy.
- Future work includes migrating to macOS Keychain or Secure Enclave and enforcing per-record metadata authentication.

---

## Local AI Orchestrator
- `peptrack-local-ai` detects CLI binaries during startup.
- The orchestrator walks a priority chain (Codex → Claude by default) and streams prompts via stdin/stdout.
- Responses are parsed from CLI JSON; if parsing fails, raw stdout is returned so the UI still shows something.
- Extendable to additional providers by implementing `LocalAiClient`.

---

## Roadmap & Next Steps
1. **Supplier tracking + inventory:** extend the schema (protocol metadata + new tables) and build Vue UI for suppliers, cost-per-mg, and stock alerts.
2. **Dose logging UX:** connect the existing `DoseLog` model to UI forms, add calendar/timeline views, and expose Tauri commands for CRUD operations.
3. **Literature ingestion pipeline:** implement real fetchers (PubMed/OpenAlex/Crossref) with caching in `literature_cache` and expose results in the frontend.
4. **macOS Keychain integration:** replace file-based key storage with Keychain-backed secrets and provide migration tooling.
5. **Background reminders:** create a LaunchAgent or Tauri sidecar to surface dose reminders and vial-expiry notifications even when the UI is closed.

For contributor workflows and day-to-day commands, see `AGENTS.md` and `docs/peptrack_future_self.md`.
