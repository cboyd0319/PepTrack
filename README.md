# PepTrack v0.0.1 — PepTrack macOS app built with Rust, Tauri, and Vue
# Professional Peptide Management & AI-Enhanced Literature Search

**Privacy-first, locally-run peptide protocol tracking and research management system with AI-powered literature analysis.**

---

## ✨ Key Features

### 📚 Enhanced Literature Search
- **Multi-Database Search**: PubMed, OpenAlex, Semantic Scholar, Crossref, bioRxiv, arXiv
- **Local AI Summarization**: Local Claude Code CLI / Codex CLI
- **PDF Analysis**: Extract and analyze full-text papers automatically
- **Smart Ranking**: Relevance scores based on keywords, citations, and recency
- **Safety Flagging**: Auto-detect papers with animal studies, toxicity data, clinical trials

### 💊 Peptide Protocol Management
- **Reconstitution Calculator**: Calculate concentrations, doses, and syringe units
- **Protocol Tracking**: Manage multiple active peptide vials
- **Dosage Logging**: Track injections with site rotation
- **Expiration Monitoring**: Automatic alerts for expiring peptides
- **Side Effect Tracking**: Log and monitor adverse effects
- **Body Metrics**: Track weight, measurements, and progress

### 🔔 Smart Notifications
- **Dose Reminders**: Desktop notifications with customizable timing
- **Expiration Alerts**: Warning when peptides approach expiration
- **Site Rotation**: Track injection sites to avoid overuse

### 💰 Supplier & Price Tracking
- **Price Comparison**: Track prices across multiple suppliers
- **Purchase History**: Monitor spending and find best deals
- **Cost per mg**: Automatic calculation for value comparison

### 🔒 Privacy & Security
- **100% Local**: No telemetry, all data stays on your machine
- **Encrypted Credentials**: Secure JSON storage (no pickle files)
- **Open Source**: Full transparency, audit the code yourself

### 📊 Data Export & Backup
- **PDF Reports**: Professional reports with all your data
- **Google Drive Backup**: Optional cloud backup
- **Google Calendar Sync**: Optional dose scheduling
- **Import/Export**: JSON/CSV support for data portability

---

## 🧱 Tech Stack & Versions
- **Rust** `1.91.1` (pinned via `rust-toolchain.toml`)
- **Tauri** `2.9.2` runtime with `tauri-build 2.5.1` and `tauri-plugin-log 2.7.1`
- **Vue** `3.5.24` + Vite `7.2` frontend (see `frontend/`)
- **@tauri-apps/api** `2.9.0` for IPC from Vue
- **Local AI**: Codex CLI (default, uses the regular `gpt-5` model) with Claude Code CLI (`claude-haiku-4-5`) as fallback

---

## 📂 Project Structure
```
.
├── frontend/                # Vue 3 + Vite UI
│   └── src/api              # Typed wrappers around Tauri commands
├── src-tauri/               # Tauri shell + macOS integrations
│   ├── src/lib.rs           # State wiring, commands, CLI orchestration
│   └── tauri.conf.json      # Build/dev configuration (front-end lives in /frontend)
├── crates/
│   ├── core/                # SQLite storage, models, envelope encryption
│   └── local-ai/            # Codex/Claude CLI adapters & detection logic
└── Cargo.toml               # Workspace metadata + shared dependencies
```

---

## 🚀 Getting Started
1. **Install tools**
   - Rust 1.91.1 (`rustup default 1.91.1` if needed)
   - Node.js ≥ 22 (already using v25.1.0)
   - `cargo-tauri` CLI `2.9.4` (already installed)
   - Codex CLI & Claude Code CLI from their official installers / docs
2. **Install JS deps**
   ```bash
   cd frontend && npm install
   ```
3. **Dev server**
   ```bash
   cargo tauri dev
   ```
   The builder runs `npm --prefix ../frontend run dev -- --host` automatically; Vite serves on `http://localhost:5173` for hot reload.

---

## 🔐 Data Storage & Encryption
- Persistent data lives in `~/Library/Application Support/PepTrack/peptrack.sqlite` (see `StorageConfig`).
- On first launch we generate `~/Library/Application Support/PepTrack/peptrack.key` (32-byte hex) and reuse it for ChaCha20-Poly1305 envelope encryption.
- Database layout today:
  - `protocols` table (encrypted JSON payload per peptide protocol)
  - `dose_logs` table for injections / rotation data
  - `literature_cache` for AI-ranked search notes
- Schema + helpers live in `crates/core/src/db.rs`. Swapping to SQLCipher or macOS keychain-derived keys will plug in via the `KeyProvider` trait.

---

## 🧠 Local AI Summaries
- `src-tauri` tracks `LocalAiOrchestrator` state:
  1. Prefer Codex CLI (`which codex`) and run `codex exec --json --model gpt-5 -`
  2. Fallback to Claude CLI (`claude -p --model claude-haiku-4-5 --output-format json`)
- Vue UI exposes this through the “AI Literature Summary” panel (see `frontend/src/App.vue`).
- JSON/Markdown output is streamed back without hitting remote APIs other than the CLI calls themselves.

---

## 🛣️ Next Implementation Milestones
1. Flesh out supplier tracking + notifications (likely new Tauri sidecars / LaunchAgents).
2. Wire literature fetchers with real API endpoints & caching (stubs ready in `peptrack-core`).
3. Integrate macOS Keychain for encryption keys, replacing the generated file.
4. Expand Vue screens (dose logging, charts, notifications) and add Pinia/Zustand for richer state.
