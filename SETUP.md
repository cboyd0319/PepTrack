# PepTrack Setup Guide

Welcome to PepTrack! This guide will help you set up all the optional features to get the most out of your peptide research tracking.

## Table of Contents

1. [Quick Start](#quick-start)
2. [AI Summary Helper Setup (Optional)](#ai-summary-helper-setup-optional)
3. [Google Drive Backup Setup (Optional)](#google-drive-backup-setup-optional)
4. [Backup Options](#backup-options)
5. [Troubleshooting](#troubleshooting)

---

## Quick Start

**PepTrack works perfectly without any setup!** All core features work immediately:

- ✅ Track your peptide protocols
- ✅ Log doses with dates and notes
- ✅ Search research papers (PubMed, OpenAlex, Crossref)
- ✅ Local manual backups
- ✅ All data encrypted and stored locally on your computer

The setup steps below are **completely optional** and only needed if you want:
- AI-powered research summaries
- Automatic cloud backups to Google Drive

---

## AI Summary Helper Setup (Optional)

The AI Summary Helper can create readable summaries of research papers for you. It's completely optional!

### What You Need

PepTrack supports two AI command-line tools:
1. **Codex CLI** (recommended) - Uses GPT-5 model
2. **Claude CLI** - Uses Claude Haiku 4.5 model

Codex is tried first, and if it's not available, Claude is used as a fallback.

### Option 1: Install Codex CLI (Recommended)

**Model Used:** GPT-5

**Installation:**
```bash
# Install Codex CLI from OpenAI
npm install -g codex-cli
```

**Configuration:**
Make sure Codex is configured to use the GPT-5 model:
```bash
# Check your Codex configuration
codex config show

# Set model to GPT-5 if needed
codex config set model gpt-5
```

**Documentation:** [https://developers.openai.com/codex/cli/reference](https://developers.openai.com/codex/cli/reference)

### Option 2: Install Claude CLI (Fallback)

**Model Used:** claude-haiku-4-5

**Installation:**
```bash
# Install Claude Code CLI
curl https://code.claude.com/install.sh | bash
```

**Configuration:**
Ensure Claude is using the Haiku model:
```bash
# Check configuration
claude config show

# The default model should be claude-haiku-4-5
```

**Documentation:** [https://code.claude.com/docs/en/cli-reference](https://code.claude.com/docs/en/cli-reference)

### Verify AI Setup

1. Open PepTrack
2. You should see a message on startup indicating which AI provider is available
3. Try the AI Summary Helper with a research paper to test it

**Still don't want AI?** No problem! Just skip this section entirely. Everything else works great without it.

---

## Google Drive Backup Setup (Optional)

Automatically backup your data to Google Drive for extra safety and access from anywhere.

### Prerequisites

- A Google account
- 5 minutes to set up Google Cloud credentials

### Step 1: Create Google Cloud Project

1. Go to [Google Cloud Console](https://console.cloud.google.com/)
2. Sign in with your Google account
3. Click "Select a Project" → "New Project"
4. Name it "PepTrack" (or anything you like)
5. Click "Create"

### Step 2: Enable Google Drive API

1. In your project, go to "APIs & Services" → "Library"
2. Search for "Google Drive API"
3. Click on it and press "Enable"

### Step 3: Create OAuth Credentials

1. Go to "APIs & Services" → "Credentials"
2. Click "Create Credentials" → "OAuth client ID"
3. If prompted, configure the OAuth consent screen:
   - User Type: **External**
   - App name: **PepTrack**
   - User support email: (your email)
   - Developer contact: (your email)
   - Click "Save and Continue" through the rest
4. Back in "Create OAuth client ID":
   - Application type: **Desktop app**
   - Name: **PepTrack Desktop**
   - Click "Create"
5. **IMPORTANT:** Add this redirect URI:
   ```
   http://localhost:8080/oauth/callback
   ```
6. Copy your **Client ID** and **Client Secret** - you'll need these!

### Step 4: Connect PepTrack to Google Drive

1. Open PepTrack
2. Scroll to "☁️ Google Drive Backup" section
3. Click "🔗 Connect Google Drive"
4. Follow the setup instructions
5. Paste your Client ID and Client Secret
6. Click "🚀 Start Connection"
7. Your browser will open - sign in and authorize PepTrack
8. Come back to PepTrack - you're connected!

### Step 5: Backup to Drive

Once connected:
1. Click "☁️ Backup to Drive Now" button
2. Your data will upload to a "PepTrack Backups" folder in your Google Drive
3. Backups are timestamped: `peptrack_backup_2024-01-15_14-30.json`

### Privacy & Security

- **You control the credentials:** Your Google OAuth credentials stay on your computer
- **Your data stays yours:** Backups go directly from your computer to YOUR Google Drive
- **No middleman:** PepTrack never sees or stores your Google credentials or data
- **Open source:** You can review all the code to verify this

---

## Backup Options

PepTrack offers two backup methods:

### 1. Manual Local Backup (Always Available)

**Location:** "💾 Backup Your Data" section

**How it works:**
1. Click "📥 Export Backup Now"
2. JSON file downloads to your computer
3. Save it wherever you want

**What's included:**
- All protocols
- All dose logs
- All saved research papers
- Metadata (export date, counts, app version)

**File format:** Human-readable JSON
**Filename:** `peptrack_backup_2024-01-15_14-30.json`

**When to use:**
- Quick backups before major changes
- Sharing data with yourself on another device
- Keeping local archives
- No internet required

### 2. Google Drive Backup (Requires Setup)

**Location:** "☁️ Google Drive Backup" section

**How it works:**
1. Click "☁️ Backup to Drive Now"
2. Data uploads directly to your Google Drive
3. Saved in "PepTrack Backups" folder

**What's included:** Same as manual backup

**When to use:**
- Regular off-site backups
- Access backups from any device
- Automatic cloud storage
- Protection against computer loss/damage

**Pro tip:** Use both! Local backups are instant, Drive backups are safe off-site.

---

## Troubleshooting

### AI Summary Helper Issues

**Problem:** "AI not available" message on startup

**Solutions:**
1. Check if Codex or Claude CLI is installed:
   ```bash
   codex --version
   # or
   claude --version
   ```
2. Ensure the CLI is in your system PATH
3. Try restarting PepTrack
4. Remember: AI is optional! Skip it if you don't need it

**Problem:** "Summarization failed" error

**Solutions:**
1. Check your internet connection
2. Verify your AI CLI API key is valid
3. Check AI CLI logs for errors
4. The paper text might be too long - try a shorter abstract

### Google Drive Issues

**Problem:** Can't connect to Google Drive

**Solutions:**
1. Verify your Client ID and Client Secret are correct
2. Check the redirect URI is exactly: `http://localhost:8080/oauth/callback`
3. Make sure Google Drive API is enabled in your Google Cloud project
4. Try disconnecting and reconnecting

**Problem:** Upload fails

**Solutions:**
1. Check your internet connection
2. Verify you're still connected (connection status at top of section)
3. Try disconnecting and reconnecting
4. Check your Google Drive storage quota

**Problem:** "Invalid OAuth state" error

**Solutions:**
1. This is a security error - start the connection process fresh
2. Click "Connect Google Drive" again
3. Enter your credentials again
4. Complete the authorization in the same browser session

### General Issues

**Problem:** Connection status always shows offline

**Check:**
1. Your actual internet connection
2. Firewall settings blocking the app
3. VPN interfering with connections

**Problem:** Backup file is huge

**This is normal!** The JSON format is verbose but human-readable. A backup with:
- 100 protocols
- 500 dose logs
- 50 research papers

Typically results in a 1-3 MB file, which is tiny by modern standards.

---

## Getting Help

**Found a bug?** Please report it at: [https://github.com/anthropics/PepTrack/issues](https://github.com/anthropics/PepTrack/issues)

**Need help?** Check the [PepTrack Discussions](https://github.com/anthropics/PepTrack/discussions)

**Feature request?** We'd love to hear it! Open an issue with the "enhancement" label.

---

## Privacy & Data

**Where is my data stored?**
- Locally: `~/Library/Application Support/PepTrack/` (macOS)
- Encrypted using ChaCha20-Poly1305
- Encryption key stored in macOS Keychain (or as fallback file)

**What data leaves my computer?**
- **Research paper searches:** Search queries sent to PubMed, OpenAlex, Crossref APIs
- **AI summaries:** Paper text sent to OpenAI or Anthropic (via your CLI)
- **Google Drive backups:** Your backup JSON sent to YOUR Google Drive
- **Nothing else!** No analytics, no tracking, no phone-home

**Can you see my data?**
- **No!** All data stays on your computer or goes directly to services YOU control
- We don't run any servers
- We don't collect any data
- We can't see your protocols, doses, or research

---

## What's Next?

Now that you're all set up:

1. **Create your first protocol** - Add a peptide you're researching
2. **Log some doses** - Track your usage over time
3. **Search research** - Find papers about your peptides
4. **Make a backup** - Protect your data!

Enjoy using PepTrack! 🧪
