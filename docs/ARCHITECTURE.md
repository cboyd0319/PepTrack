# PepTrack - Architecture Documentation

**Last Updated:** 2025-11-11
**Version:** 1.0.0
**Target Platform:** macOS (Linux/Windows support experimental)

---

## Table of Contents

1. [System Overview](#system-overview)
2. [Architecture Diagrams](#architecture-diagrams)
3. [Component Details](#component-details)
4. [Data Models](#data-models)
5. [Security Architecture](#security-architecture)
6. [State Management](#state-management)
7. [API Integration](#api-integration)
8. [Backup System](#backup-system)
9. [Performance Considerations](#performance-considerations)
10. [Future Enhancements](#future-enhancements)

---

## System Overview

PepTrack is a desktop application built using a **Tauri + Rust + Vue 3** architecture, providing a secure, offline-first peptide management system with optional cloud backup capabilities.

### Core Principles

- **Privacy First**: All data encrypted locally, no telemetry
- **Offline-capable**: Core functionality works without internet
- **Security by Design**: Military-grade encryption (ChaCha20-Poly1305)
- **User Control**: Explicit consent for any data leaving the device

---

## Architecture Diagrams

### 1. High-Level System Architecture

```mermaid
graph TB
    subgraph "User Interface"
        User[User Interaction]
    end

    subgraph "Frontend Layer - Vue 3"
        Vue[Vue 3 SPA<br/>Composition API]
        Store[Pinia Stores<br/>State Management]
        Comp[Vue Components<br/>UI Layer]
        API[API Layer<br/>Tauri IPC Wrappers]
    end

    subgraph "Tauri Bridge"
        IPC[IPC Handler<br/>Type-safe Commands]
        Events[Event System<br/>Bidirectional Comms]
        Plugins[Tauri Plugins<br/>System Integration]
    end

    subgraph "Backend Layer - Rust"
        Commands[Command Handlers<br/>Business Logic]
        AppState[Application State<br/>Shared Resources]
        Core[Core Library<br/>peptrack-core]
        AI[AI Orchestrator<br/>peptrack-local-ai]
        Lit[Literature APIs<br/>peptrack-literature]
    end

    subgraph "Data Layer"
        DB[(SQLite<br/>Encrypted Database)]
        KC[macOS Keychain<br/>Key Storage]
        FS[File System<br/>Backups & Logs]
    end

    subgraph "External Services"
        PM[PubMed API]
        OA[OpenAlex API]
        CR[Crossref API]
        GD[Google Drive API]
        CLI[Local AI CLIs<br/>Codex / Claude]
    end

    User --> Vue
    Vue --> Store
    Store --> Comp
    Comp --> API
    API --> IPC
    IPC --> Commands
    Commands --> AppState
    AppState --> Core
    AppState --> AI
    Commands --> Lit
    Core --> DB
    Core --> KC
    Commands --> FS
    Lit --> PM
    Lit --> OA
    Lit --> CR
    Commands --> GD
    AI --> CLI
    Events --> Store
    Plugins --> IPC
```

### 2. Data Flow - Protocol Creation

```mermaid
sequenceDiagram
    autonumber
    participant U as User
    participant C as ProtocolForm.vue
    participant S as Pinia Store
    participant A as Tauri API
    participant H as Command Handler
    participant DB as Core Library
    participant SQL as SQLite DB

    U->>C: Fill form & submit
    C->>C: Validate input
    C->>S: store.createProtocol(data)
    S->>S: Optimistic UI update
    S->>A: invoke("save_protocol", data)
    A->>H: save_protocol(payload)
    H->>H: Validate payload
    H->>DB: storage.upsert_protocol(protocol)
    DB->>DB: Serialize to JSON
    DB->>DB: Encrypt with ChaCha20
    DB->>SQL: INSERT into protocols
    SQL-->>DB: Row ID
    DB-->>H: Protocol + ID
    H-->>A: Success response
    A-->>S: Protocol object
    S->>S: Update cache
    S->>C: Notify success
    C->>C: Reset form
    C->>U: Show success toast

    Note over S,C: On error: Rollback<br/>optimistic update
```

### 3. Encryption & Decryption Pipeline

```mermaid
flowchart TB
    subgraph "Encryption Flow"
        E1[Domain Object<br/>PeptideProtocol] --> E2[Serialize to JSON String]
        E2 --> E3[Generate Random Nonce<br/>12 bytes OsRng]
        E3 --> E4[Get Encryption Key<br/>from KeyProvider]
        E4 --> E5{Key Source?}
        E5 -->|macOS| E6[macOS Keychain]
        E5 -->|Fallback| E7[File peptrack.key]
        E6 --> E8[32-byte Key]
        E7 --> E8
        E8 --> E9[ChaCha20-Poly1305<br/>AEAD Encrypt]
        E9 --> E10[Prepend Nonce<br/>nonce + ciphertext]
        E10 --> E11[Store as BLOB<br/>in SQLite]
    end

    subgraph "Decryption Flow"
        D1[Read BLOB<br/>from SQLite] --> D2[Extract Nonce<br/>first 12 bytes]
        D2 --> D3[Extract Ciphertext<br/>remaining bytes]
        D3 --> D4[Get Encryption Key<br/>from KeyProvider]
        D4 --> D5{Key Source?}
        D5 -->|macOS| D6[macOS Keychain]
        D5 -->|Fallback| D7[File peptrack.key]
        D6 --> D8[32-byte Key]
        D7 --> D8
        D8 --> D9[ChaCha20-Poly1305<br/>AEAD Decrypt]
        D9 --> D10[Verify MAC<br/>Authenticate]
        D10 --> D11[Deserialize JSON<br/>to Struct]
        D11 --> D12[Domain Object<br/>PeptideProtocol]
    end

    E11 -.->|Read| D1

    style E9 fill:#ff6b6b,color:#fff
    style D9 fill:#51cf66,color:#fff
    style D10 fill:#ffd43b,color:#000
```

### 4. State Management Architecture

```mermaid
graph LR
    subgraph "Pinia Stores"
        PS[Protocol Store<br/>CRUD + Cache]
        DS[Dose Store<br/>Logging + History]
        SS[Supplier Store<br/>Inventory Mgmt]
        LS[Literature Store<br/>Search + AI]
        US[UI Store<br/>Global State]
    end

    subgraph "Composables Layer"
        PC[useProtocols<br/>Helper Functions]
        DC[useDoses<br/>Helper Functions]
        SC[useSuppliers<br/>Helper Functions]
        LC[useLiterature<br/>Helper Functions]
    end

    subgraph "Vue Components"
        C1[ProtocolList]
        C2[DoseTracker]
        C3[LiteratureSearch]
        C4[Settings]
    end

    subgraph "Tauri Backend"
        CMD[Command Handlers]
    end

    C1 --> PC
    C2 --> DC
    C3 --> LC
    C4 --> SC

    PC --> PS
    DC --> DS
    SC --> SS
    LC --> LS

    PS --> CMD
    DS --> CMD
    SS --> CMD
    LS --> CMD

    US -.->|Global State| C1
    US -.->|Global State| C2
    US -.->|Global State| C3
    US -.->|Global State| C4
```

### 5. Backup System Architecture

```mermaid
flowchart TD
    subgraph "Backup Triggers"
        M[Manual Trigger<br/>User Button]
        S[Scheduled Trigger<br/>Tokio Runtime]
    end

    subgraph "Backup Creation"
        B1[Collect All Data<br/>Protocols, Doses, Literature]
        B2[Build Backup Metadata<br/>Version, Counts, Date]
        B3[Serialize to JSON<br/>BackupData struct]
        B4{Compression?}
        B4 -->|Yes| B5[Gzip Compress<br/>flate2]
        B4 -->|No| B6[Raw JSON]
        B5 --> B7[Save to Filesystem<br/>Timestamped]
        B6 --> B7
    end

    subgraph "Cloud Upload"
        C1{Google Drive<br/>Connected?}
        C1 -->|Yes| C2[Check Token Expiry]
        C2 -->|Expired| C3[Refresh OAuth Token]
        C2 -->|Valid| C4[Upload to Drive<br/>OAuth API]
        C3 --> C4
        C1 -->|No| C5[Skip Cloud Upload]
    end

    subgraph "Cleanup"
        CL1{Cleanup Policy?}
        CL1 -->|Keep Last N| CL2[Delete Oldest Files<br/>Beyond N]
        CL1 -->|Older Than X Days| CL3[Delete by Date]
        CL1 -->|Both| CL4[Apply Both Rules]
    end

    subgraph "Notification"
        N1[Desktop Notification<br/>Success or Failure]
    end

    M --> B1
    S --> B1
    B1 --> B2
    B2 --> B3
    B3 --> B4
    B7 --> C1
    C4 --> CL1
    C5 --> CL1
    CL2 --> N1
    CL3 --> N1
    CL4 --> N1
```

### 6. Literature Search Flow

```mermaid
sequenceDiagram
    participant U as User
    participant C as LiteratureSearch.vue
    participant S as Literature Store
    participant H as Command Handler
    participant L as Literature Crate
    participant PM as PubMed API
    participant OA as OpenAlex API
    participant CR as Crossref API
    participant DB as SQLite Cache

    U->>C: Enter search query
    C->>S: store.search(query, providers)
    S->>H: invoke("search_literature")

    par Parallel API Calls
        H->>L: PubMedFetcher::search(query)
        L->>PM: GET /esearch + /efetch
        PM-->>L: XML results
        L->>L: Parse XML to LiteratureResult
    and
        H->>L: OpenAlexFetcher::search(query)
        L->>OA: GET /works?search=
        OA-->>L: JSON results
        L->>L: Parse JSON to LiteratureResult
    and
        H->>L: CrossrefFetcher::search(query)
        L->>CR: GET /works?query=
        CR-->>L: JSON results
        L->>L: Parse JSON to LiteratureResult
    end

    L-->>H: Combined results
    H->>DB: Cache results
    DB-->>H: Success
    H-->>S: Literature entries
    S->>C: Update results
    C->>U: Display papers
```

---

## Component Details

### Frontend Components

#### Core Components (18 total)

| Component | Purpose | Key Features |
|-----------|---------|--------------|
| `App.vue` | Main application shell | Tab navigation, view switching |
| `ProtocolList.vue` | Display protocols | Grid view, search, refresh |
| `ProtocolForm.vue` | Create/edit protocols | Form validation, ARIA labels |
| `DoseTracker.vue` | Log doses | Calendar view, history, filtering |
| `LiteratureSearch.vue` | Search papers | Multi-source search, caching |
| `AiSummaryPanel.vue` | AI summarization | Local CLI integration, format selection |
| `Settings.vue` | Settings container | Tab navigation, sub-settings |
| `BackupExport.vue` | Manual backups | Compression option, file save |
| `ScheduledBackup.vue` | Automated backups | Schedule config, cleanup policies |
| `GoogleDriveBackup.vue` | Cloud backup | OAuth flow, upload management |
| `RestoreBackup.vue` | Restore data | Preview, selective restore |
| `NotificationPreferences.vue` | Notification config | Granular controls |
| `SupplierManagement.vue` | Supplier CRUD | Contact info, notes |
| `InventoryManagement.vue` | Inventory tracking | Vial status, expiry alerts |
| `Toast.vue` | Toast notifications | Error/success messages |
| `LoadingSkeleton.vue` | Loading states | 4 variants, animated |
| `ErrorBoundary.vue` | Error handling | Graceful failures, retry |
| `WelcomeScreen.vue` | First-run onboarding | Feature intro |

### Backend Command Modules

#### Command Handlers (11 modules)

| Module | Commands | Purpose |
|--------|----------|---------|
| `protocols.rs` | `list_protocols`, `save_protocol`, `delete_protocol` | Protocol CRUD |
| `doses.rs` | `log_dose`, `list_dose_logs`, `list_dose_logs_for_protocol`, `delete_dose_log` | Dose management |
| `suppliers.rs` | `create_supplier`, `list_suppliers`, `update_supplier`, `delete_supplier`, `create_inventory_item`, `list_inventory`, `update_inventory_item`, `delete_inventory_item` | Supplier & inventory |
| `ai.rs` | `summarize_text`, `check_ai_availability` | AI integration |
| `literature.rs` | `search_literature`, `list_literature`, `search_cached_literature` | Literature search |
| `backup.rs` | `export_backup_data` | Manual backup creation |
| `restore.rs` | `preview_backup`, `restore_from_backup` | Backup restoration |
| `scheduler_v2.rs` | `update_backup_schedule`, `get_backup_schedule`, `get_backup_history`, `get_backup_progress`, `trigger_backup_now` | Scheduled backups |
| `drive.rs` | `start_drive_oauth`, `complete_drive_oauth`, `check_drive_status`, `upload_to_drive`, `disconnect_drive` | Google Drive integration |

---

## Data Models

### Core Domain Models

```mermaid
erDiagram
    PeptideProtocol ||--o{ DoseLog : has
    PeptideProtocol ||--o{ InventoryItem : has
    Supplier ||--o{ InventoryItem : supplies

    PeptideProtocol {
        string id PK
        string name
        string peptide_name
        string notes
        string current_vial_status
        float target_concentration_mg_ml
        datetime created_at
        datetime updated_at
    }

    DoseLog {
        string id PK
        string protocol_id FK
        string site
        float amount_mg
        string notes
        datetime logged_at
    }

    Supplier {
        string id PK
        string name
        string contact_email
        string contact_phone
        string website
        string notes
        datetime created_at
        datetime updated_at
    }

    InventoryItem {
        string id PK
        string protocol_id FK
        string supplier_id FK
        string vial_number
        enum vial_status
        date purchase_date
        date expiry_date
        float cost_per_mg
        float quantity_mg
        float concentration_mg_ml
        string batch_number
        string lot_number
        string notes
        datetime created_at
        datetime updated_at
    }

    LiteratureEntry {
        string id PK
        string source
        string title
        string url
        string summary
        float relevance_score
        datetime indexed_at
    }
```

### Database Schema

```mermaid
graph LR
    subgraph "SQLite Tables"
        T1[(protocols<br/>id, name, payload BLOB,<br/>updated_at)]
        T2[(dose_logs<br/>id, protocol_id FK,<br/>payload BLOB, logged_at)]
        T3[(suppliers<br/>id, name, payload BLOB,<br/>updated_at)]
        T4[(inventory<br/>id, protocol_id FK,<br/>supplier_id FK,<br/>payload BLOB, updated_at)]
        T5[(literature_cache<br/>id, source,<br/>payload BLOB, indexed_at)]
    end

    T1 -->|FK| T2
    T1 -->|FK| T4
    T3 -->|FK| T4

    style T1 fill:#4dabf7
    style T2 fill:#51cf66
    style T3 fill:#ffd43b
    style T4 fill:#ff6b6b
    style T5 fill:#9775fa
```

**Key Characteristics:**
- All sensitive data stored as encrypted BLOBs
- UUID primary keys (TEXT type)
- Foreign key constraints with CASCADE delete
- Indexes on timestamp columns for performance
- WAL mode enabled for concurrent access

---

## Security Architecture

### Threat Model

| Threat | Mitigation |
|--------|------------|
| **Data at rest** | ChaCha20-Poly1305 encryption with 12-byte random nonces |
| **Key compromise** | macOS Keychain storage, OS-level encryption |
| **Data in transit** | HTTPS for all external APIs, OAuth PKCE for Google Drive |
| **Unauthorized access** | No network-accessible interfaces, local-only app |
| **Memory attacks** | Zeroize sensitive data, secure memory clearing |
| **Replay attacks** | Unique nonces per record, timestamp validation |

### Encryption Key Lifecycle

```mermaid
stateDiagram-v2
    [*] --> CheckKeychain: App Starts
    CheckKeychain --> LoadFromKeychain: Keychain Has Key
    CheckKeychain --> CheckFileKey: No Keychain Key

    LoadFromKeychain --> KeyReady: Success

    CheckFileKey --> MigrateToKeychain: File Key Exists
    CheckFileKey --> GenerateNewKey: No File Key

    MigrateToKeychain --> SaveToKeychain: Read File Key
    SaveToKeychain --> DeleteFileKey: Migration Complete
    DeleteFileKey --> KeyReady

    GenerateNewKey --> SaveToKeychain: 32 Random Bytes

    KeyReady --> EncryptData: Usage
    KeyReady --> DecryptData: Usage

    EncryptData --> EncryptData
    DecryptData --> DecryptData

    KeyReady --> [*]: App Exits<br/>(Key Cleared)
```

### OAuth Security (Google Drive)

```mermaid
sequenceDiagram
    participant A as PepTrack App
    participant B as Browser
    participant G as Google OAuth
    participant D as Google Drive API

    A->>A: Generate PKCE challenge + state
    A->>B: Open auth URL
    B->>G: User authorizes
    G->>B: Redirect with code + state
    B->>A: User returns to app
    A->>A: Verify state (CSRF)
    A->>G: Exchange code for token<br/>(with PKCE verifier)
    G-->>A: Access token + refresh token
    A->>A: Encrypt & store tokens

    Note over A: Later...

    A->>A: Check token expiry
    A->>G: Refresh access token
    G-->>A: New access token
    A->>D: Upload backup
    D-->>A: Success
```

---

## State Management

### Pinia Store Architecture

```mermaid
graph TB
    subgraph "Protocol Store"
        PS1[State: protocols array, loading, cache]
        PS2[Getters: protocolCount, activeProtocols, protocolsByPeptide]
        PS3[Actions: fetchProtocols, createProtocol, updateProtocol, removeProtocol]
    end

    subgraph "Dose Store"
        DS1[State: doses array, loading]
        DS2[Getters: doseCount, recentDoses, dosesThisWeek]
        DS3[Actions: fetchDoses, logDose, removeDose]
    end

    subgraph "UI Store"
        US1[State: globalLoading, loadingOperations, modals]
        US2[Getters: loadingMessage, isOnline]
        US3[Actions: startLoading, stopLoading, openModal]
    end

    PS3 -->|Invoke| IPC[Tauri IPC]
    DS3 -->|Invoke| IPC
    IPC -->|Events| PS1
    IPC -->|Events| DS1
    US3 -.->|Cross-Store| PS3
    US3 -.->|Cross-Store| DS3
```

### Caching Strategy

**Protocol Store:**
- Cache duration: 30 seconds
- Cache invalidation: Automatic on mutations
- Force refresh: Available via `fetchProtocols(true)`
- Cache key: Timestamp of last fetch

**Dose Store:**
- Per-protocol caching
- Invalidation: On dose creation/deletion
- Refresh: Automatic when switching protocols

**Literature Store:**
- Search results: Session-scoped cache
- Cached papers: Persistent in SQLite
- No automatic expiration (user-initiated clear)

---

## API Integration

### External API Specifications

#### PubMed

```
Base URL: https://eutils.ncbi.nlm.nih.gov/entrez/eutils/
Endpoints:
  - esearch.fcgi: Search for articles
  - efetch.fcgi: Fetch article details
Rate Limit: 3 requests/second (unauthenticated)
Response Format: XML
```

#### OpenAlex

```
Base URL: https://api.openalex.org/
Endpoints:
  - /works?search={query}
Rate Limit: 100,000 requests/day
Response Format: JSON
```

#### Crossref

```
Base URL: https://api.crossref.org/
Endpoints:
  - /works?query={query}
Rate Limit: 50 requests/second
Response Format: JSON
```

#### Google Drive

```
Base URL: https://www.googleapis.com/
Endpoints:
  - /oauth2/v4/token: Token management
  - /drive/v3/files: File upload
  - /drive/v3/about: User info
Authentication: OAuth 2.0 with PKCE
Scopes: drive.file (limited access)
```

---

## Backup System

### Backup File Format

```typescript
interface BackupData {
  metadata: {
    export_date: string;          // ISO 8601
    protocols_count: number;
    doses_count: number;
    literature_count: number;
    app_version: string;          // e.g., "1.0.0"
  };
  protocols: PeptideProtocol[];
  dose_logs: DoseLog[];
  literature: LiteratureEntry[];
  suppliers?: Supplier[];          // Optional, newer versions
  inventory?: InventoryItem[];     // Optional, newer versions
}
```

### Backup Naming Convention

```
Format: peptrack_backup_{timestamp}.{extension}
Examples:
  - peptrack_backup_2025-11-11_14-30-45.json
  - peptrack_backup_2025-11-11_14-30-45.json.gz

Timestamp: YYYY-MM-DD_HH-mm-ss (24-hour format)
Extensions:
  - .json: Uncompressed
  - .json.gz: Gzip compressed
```

### Scheduled Backup Flow

```mermaid
stateDiagram-v2
    [*] --> Idle
    Idle --> CheckSchedule: Every 60s
    CheckSchedule --> Idle: Not time yet
    CheckSchedule --> AcquireLock: Time to backup

    AcquireLock --> Idle: Lock held by another process
    AcquireLock --> CreateBackup: Lock acquired

    CreateBackup --> Compress: Compression enabled
    CreateBackup --> SaveLocal: Compression disabled
    Compress --> SaveLocal

    SaveLocal --> UploadCloud: Google Drive connected
    SaveLocal --> Cleanup: No cloud
    UploadCloud --> RefreshToken: Token expired
    RefreshToken --> UploadCloud
    UploadCloud --> Cleanup

    Cleanup --> ApplyKeepLastN: Policy: Keep Last N
    Cleanup --> ApplyOlderThan: Policy: Older Than X
    Cleanup --> ApplyBoth: Policy: Both

    ApplyKeepLastN --> Notify
    ApplyOlderThan --> Notify
    ApplyBoth --> Notify

    Notify --> ReleaseLock
    ReleaseLock --> Idle
```

---

## Performance Considerations

### Frontend Optimizations

1. **Smart Caching**: 30-second TTL on protocol/dose data
2. **Optimistic Updates**: Immediate UI feedback with rollback on error
3. **Virtual Scrolling**: Ready for implementation when lists exceed 100+ items
4. **Lazy Loading**: Components loaded on-demand
5. **Debounced Search**: 300ms debounce on search inputs
6. **Loading Skeletons**: Perceived performance improvement

### Backend Optimizations

1. **SQLite WAL Mode**: Concurrent reads during writes
2. **Prepared Statements**: Reusable query compilation
3. **Indexed Queries**: Timestamps and foreign keys indexed
4. **Async I/O**: All file/network operations non-blocking
5. **Connection Pooling**: Single shared connection with mutex
6. **Batch Operations**: Bulk inserts for backup restoration

### Memory Management

1. **Zeroize**: Sensitive data cleared from memory
2. **Stream Processing**: Large files streamed, not loaded entirely
3. **Bounded Channels**: Prevent unbounded memory growth
4. **Arc/Mutex**: Shared state with minimal cloning

---

## Future Enhancements

### Planned Features

1. **Background Reminders**
   - LaunchAgent for macOS
   - Dose schedule notifications
   - Vial expiry alerts

2. **Cloud Restore**
   - List backups from Google Drive
   - Preview before download
   - Selective restore

3. **Multi-Cloud Support**
   - Dropbox integration
   - OneDrive integration
   - Provider abstraction layer

4. **Advanced Analytics**
   - Usage statistics dashboard
   - Cost tracking over time
   - Peptide efficacy tracking

5. **Data Export**
   - CSV export for protocols/doses
   - PDF reports
   - Excel-compatible formats

6. **Collaboration**
   - Export/import for data sharing
   - Anonymized data export
   - Protocol templates

---

## Appendix

### Build Artifacts

```
Production Build:
├── PepTrack.app (macOS)
│   ├── Contents/
│   │   ├── MacOS/
│   │   │   └── PepTrack (binary)
│   │   ├── Resources/
│   │   │   └── assets/ (HTML/CSS/JS)
│   │   └── Info.plist
└── PepTrack.dmg (installer)

Development Build:
└── target/
    ├── debug/
    │   └── peptrack (binary)
    └── release/
        └── peptrack (optimized binary)
```

### Environment Variables

```bash
# Optional: Enable debug logging
RUST_LOG=peptrack=debug,peptrack_core=debug

# Optional: Override data directory
PEPTRACK_DATA_DIR=~/custom/path

# Optional: AI CLI paths
CODEX_CLI_PATH=/usr/local/bin/codex
CLAUDE_CLI_PATH=/usr/local/bin/claude
```

### Testing Checklist

- [ ] All Rust unit tests pass (`cargo test --workspace`)
- [ ] Frontend tests pass (`npm run test`)
- [ ] Clippy produces no warnings (`cargo clippy --workspace`)
- [ ] Frontend builds without errors (`npm run build`)
- [ ] Manual smoke tests:
  - [ ] Create protocol
  - [ ] Log dose
  - [ ] Search literature
  - [ ] Manual backup
  - [ ] Scheduled backup
  - [ ] Google Drive upload
  - [ ] Restore backup
  - [ ] AI summarization

---

**Document Version:** 1.0.0
**Last Review:** 2025-11-11
**Maintainer:** PepTrack Team
