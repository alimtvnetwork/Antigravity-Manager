# Migration & Verification Spec: IDE Settings/Projects/Prompts Import/Export, Clean Purge & Reinstall, and Install IDE Engine

**Slug:** `226-antigravity-manager-clone-and-ui-enhancements`  
**File:** `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`  
**Target Release:** v4.269.0  
**Status:** APPROVED FOR IMPLEMENTATION  
**Lead Author:** Spec Authoring Subagent 03  
**Parent Plan:** `.ai-memory/plans/pending/226-antigravity-manager-clone-and-ui-enhancements.md`  
**Sibling Specs:**  
- `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/01-clone-and-card-actions-spec.md`  
- `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/02-profile-selection-and-ui-glitches-spec.md`  
- `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/04-e2e-report.md`  
**Implementation Subtask Plan:** `.ai-memory/plans/subtasks/226-antigravity-manager-clone-and-ui-enhancements/03-subtask-import-export-and-purge-install.md`

---

## 1. Executive Summary & Architectural Scope

This specification defines the architectural contracts, data models, IPC protocols, CLI interfaces, and end-to-end verification matrices for three foundational enterprise lifecycle extensions within Antigravity-Manager (`agm`), as well as the comprehensive verification protocol covering all eight user requirements in task `226`:

1. **IDE Profile Bundle Import / Export Engine (`.agmide` / `.json`)**: Enables comprehensive portability of an IDE instance's configuration across workstations and virtual environments. Bundles editor settings (`settings.json`, `keybindings.json`, `snippets/`), registered projects (workspace URIs, running projects, pinned projects from `agm_project_sequences`), and prompt assets (active FIFO prompts, suffix templates, "Resend As" templates, and global instructions).
2. **Full Antigravity Application Deletion & Clean Reinstall Engine**: A cross-platform 4-stage purge protocol across Windows, macOS, and Linux that systematically eradicates memory fog, corrupt Electron storage, bloated SQLite transcripts, stale process locks, and `~/.gemini/antigravity` disk bloat, with mandatory **Caller IDE Self-Preservation** (`detect_caller_instance_id()`) and optional credential vault preservation.
3. **Multi-IDE Installation Engine**: A unified installer subsystem supporting Google Antigravity IDE, Cursor AI, and Visual Studio Code across Windows (winget/PowerShell), macOS (brew/DMG), and Linux (snap/apt/AppImage), accessible via CLI (`agm ide install`) and modern UI (`InstallIdeModal.tsx`).
4. **End-to-End Verification Matrix**: The formal verification specification covering all eight functional pillars of the 226 task family, defining regression safeguards, exit code invariants, and performance acceptance criteria.

---

## 2. Technical Architecture & Data Contracts

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            CLI & UI Dispatch Layer                          │
│   agm ide export / import    agm ide purge-reinstall    agm ide install    │
│   IdeBundleModal.tsx         PurgeReinstallModal.tsx    InstallIdeModal.tsx │
└──────────────────────┬──────────────────────┬──────────────────────┬────────┘
                       │                      │                      │
                       ▼                      ▼                      ▼
┌──────────────────────────────┐┌─────────────────────────┐┌──────────────────┐
│   IDE Profile Bundle Engine  ││    Safe Purge Engine    ││ IDE Install Sys  │
│ - Settings Packager          ││ - Caller PID Guard (E=2)││ - Platform Matrix│
│ - Project Sequence Packager  ││ - Credential Safeguard  ││ - Win / Mac / Lin│
│ - Prompt SQLite Packager     ││ - Deep FS Sanitizer     ││ - Verification   │
│ - Schema Validator (.agmide) ││ - Clean Provisioner     ││ - Path Discovery │
└──────────────┬───────────────┘└─────────────┬───────────┘└─────────┬────────┘
               │                              │                      │
               ▼                              ▼                      ▼
┌─────────────────────────────────────────────────────────────────────────────┐
│                             Operating System & FS                           │
│  ~/.gemini/antigravity/   AppData/Local / ~/Library   repo_prompts.db / etc. │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 3. Section 1: IDE Profile Bundle Import / Export Engine

### 3.1 Bundle Schema Contract (`AgmIdeProfileBundle`)

The bundle file is stored as a formatted JSON document with either `.agmide` (recommended) or `.json` extension. It adheres to schema version `1.0`.

```json
{
  "$schema": "https://antigravity.google.com/schemas/agm-ide-profile-bundle-v1.json",
  "agm_bundle_version": "1.0",
  "exported_at": "2026-10-25T12:00:00Z",
  "source_environment": {
    "instance_id": "inst-alpha",
    "instance_name": "Development Alpha",
    "agm_version": "v4.269.0",
    "platform": "windows",
    "arch": "x86_64"
  },
  "settings": {
    "editor_settings": {
      "antigravity.turboMode": true,
      "antigravity.planReviewAlwaysProceed": false,
      "editor.fontSize": 14,
      "editor.tabSize": 2
    },
    "keybindings": [
      {
        "key": "ctrl+alt+n",
        "command": "antigravity.sendNow"
      }
    ],
    "snippets": {
      "typescript.json": "{\n  \"Console Log\": {\n    \"prefix\": \"clg\",\n    \"body\": [\"console.log('$1');\"]\n  }\n}"
    }
  },
  "projects": {
    "workspace_uris": [
      "file:///workspace/core-project",
      "file:///workspace/service-repo"
    ],
    "pinned_project_sequences": [
      {
        "project_key": "agm_core",
        "seq_id": 1,
        "project_id": "proj_01",
        "repo_name": "core-project",
        "repo_path": "repos/core-project"
      }
    ],
    "running_projects": [
      {
        "id": "rp_01",
        "repo_name": "core-project",
        "repo_path": "repos/core-project",
        "workspace_storage_path": "repos/core-project/.vscode"
      }
    ]
  },
  "prompts": {
    "active_prompts": [
      {
        "id": "ap_001",
        "repo_path": "repos/core-project",
        "prompt_content": "Analyze and refactor memory leaks in transcript reader",
        "model": "gemini-2.5-pro",
        "status": "QUEUED",
        "created_at": 1730000000,
        "updated_at": 1730000000
      }
    ],
    "suffix_templates": [
      {
        "id": "suff_strict",
        "name": "Strict Verification",
        "category": "Quality",
        "template": "Verify zero compile errors and verify all tests pass.",
        "is_default": true,
        "created_at": "2026-10-25T10:00:00Z",
        "updated_at": "2026-10-25T10:00:00Z"
      }
    ],
    "resend_as_templates": [
      {
        "id": "resend_diag",
        "name": "Retry with Diagnostics",
        "title_pattern": "[DIAGNOSTIC] {original_title}",
        "target_mode": "new",
        "suffix_id": "suff_strict",
        "is_default": true,
        "created_at": "2026-10-25T10:00:00Z",
        "updated_at": "2026-10-25T10:00:00Z"
      }
    ],
    "global_system_prompt": "You are Antigravity, a powerful agentic AI coding assistant."
  },
  "metadata": {
    "bundle_checksum": "sha256-a1b2c3d4e5f6...",
    "item_counts": {
      "settings_keys": 4,
      "keybindings": 1,
      "snippet_files": 1,
      "workspaces": 2,
      "pinned_projects": 1,
      "running_projects": 1,
      "active_prompts": 1,
      "suffix_templates": 1,
      "resend_as_templates": 1
    }
  }
}
```

### 3.2 Rust Data Structures (`src-tauri/src/modules/instance/bundle_types.rs`)

```rust
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const AGM_IDE_BUNDLE_SCHEMA_VERSION: &str = "1.0";
pub const DEFAULT_BUNDLE_EXTENSION: &str = "agmide";

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgmIdeProfileBundle {
    pub agm_bundle_version: String,
    pub exported_at: String,
    pub source_environment: SourceEnvironment,
    pub settings: BundleSettings,
    pub projects: BundleProjects,
    pub prompts: BundlePrompts,
    pub metadata: BundleMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEnvironment {
    pub instance_id: String,
    pub instance_name: String,
    pub agm_version: String,
    pub platform: String,
    pub arch: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleSettings {
    pub editor_settings: serde_json::Value,
    pub keybindings: Vec<serde_json::Value>,
    pub snippets: HashMap<String, String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundleProjects {
    pub workspace_uris: Vec<String>,
    pub pinned_project_sequences: Vec<BundlePinnedProject>,
    pub running_projects: Vec<BundleRunningProject>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundlePinnedProject {
    pub project_key: String,
    pub seq_id: i64,
    pub project_id: String,
    pub repo_name: String,
    pub repo_path: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleRunningProject {
    pub id: String,
    pub repo_name: String,
    pub repo_path: String,
    pub workspace_storage_path: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct BundlePrompts {
    pub active_prompts: Vec<BundleActivePrompt>,
    pub suffix_templates: Vec<BundleSuffixTemplate>,
    pub resend_as_templates: Vec<BundleResendAsTemplate>,
    pub global_system_prompt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleActivePrompt {
    pub id: String,
    pub repo_path: String,
    pub prompt_content: String,
    pub model: Option<String>,
    pub status: String,
    pub created_at: i64,
    pub updated_at: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleSuffixTemplate {
    pub id: String,
    pub name: String,
    pub category: String,
    pub template: String,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleResendAsTemplate {
    pub id: String,
    pub name: String,
    pub title_pattern: String,
    pub target_mode: String,
    pub suffix_id: Option<String>,
    pub is_default: bool,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BundleMetadata {
    pub bundle_checksum: String,
    pub item_counts: HashMap<String, usize>,
}
```

### 3.3 Export Packaging Engine (`src-tauri/src/modules/instance/bundle_export.rs`)

The export engine performs sequential extraction:
1. **Settings Extraction**:
   - Resolves target instance configuration from `load_registry()`.
   - Locates `User/settings.json`, `User/keybindings.json`, and `User/snippets/` within instance data directory.
   - Parses `settings.json` into `serde_json::Value`.
   - Ingests all snippet JSON files into `HashMap<String, String>`.
2. **Projects Extraction**:
   - Reads workspace directories associated with the instance from `get_instance_workspace_paths`.
   - Connects to `repo_prompts.db` with WAL mode and queries `agm_project_sequences` where `instance_id = ?` ordered by `seq_id ASC`.
   - Queries `running_projects` where `instance_id = ?`.
3. **Prompts Extraction**:
   - Queries `active_prompts` where `instance_id = ?` enforcing strict FIFO ordering `ORDER BY created_at ASC, id ASC`.
   - Queries `prompt_suffix_templates` and `prompt_resend_as_templates`.
   - Ingests system prompt configuration from `config::get_global_system_prompt()`.
4. **Checksum & Assembly**:
   - Calculates SHA-256 digest over the concatenated canonical JSON payload.
   - Emits formatted JSON string to target file path.

### 3.4 Import Restoring Engine (`src-tauri/src/modules/instance/bundle_import.rs`)

The import engine supports two target modes:
1. **Import into Existing Instance (`--target <id>`)**:
   - Deep-merges `editor_settings` into target instance's `settings.json`.
   - Appends missing keybindings to `keybindings.json`.
   - Writes snippet files into target instance's `User/snippets/`.
   - Inserts/updates `agm_project_sequences` and `running_projects` mapped to target `instance_id`.
   - Inserts `active_prompts` marked with `status: 'QUEUED'` for prompt resumption.
2. **Import as New Cloned Instance (`--new-name <name>`)**:
   - Creates a new instance entry in registry with generated UUID (`inst-<uuid>`).
   - Initializes fresh user data directory under `.antigravity_tools/instances/<id>/data`.
   - Provisions all settings, snippets, projects, and prompts directly into the new profile.

### 3.5 CLI Interface Specification

```bash
# Export active instance profile bundle
agm ide export [instance_id] [--out <output_path>] [--json]

# Import profile bundle into existing or newly created instance
agm ide import <bundle_file.agmide> [--target <instance_id>] [--new-name <new_instance_name>] [--force]
```

Exit Codes:
- `0`: Success.
- `1`: Validation or file I/O error (invalid schema, unreadable bundle).
- `2`: Caller IDE protection triggered (attempting to overwrite caller instance without `--force`).

### 3.6 UI Component: `IdeBundleModal.tsx`

Located in `src/components/instances/IdeBundleModal.tsx`.
- Modal triggered from `InstancesHeader.tsx` popover: "IDE Profile Bundle (.agmide)".
- Dual Tab Layout: **Export Profile** and **Import Profile**.
- **Export View**:
  - Select instance source (default: active instance).
  - Toggles for bundle components: `[x] Editor Settings & Keybindings`, `[x] Pinned Projects & Workspaces`, `[x] Prompts & Suffix Templates`.
  - Preview card displaying summary counts (e.g. "12 settings keys, 3 projects, 8 prompts").
  - "Save Bundle As..." file dialog trigger (`.agmide` / `.json`).
- **Import View**:
  - Drag-and-drop file upload zone or browse button.
  - Inspection summary card rendering source version, platform, and contained items.
  - Radio options:
    - `(*) Create New Instance from Bundle` (with editable name input field).
    - `( ) Merge into Existing Instance` (with instance dropdown selector).
  - "Execute Import" button with progress indicator and toast feedback.

---

## 4. Section 2: Full Antigravity Clean Delete & Reinstall Engine

### 4.1 Cross-Platform Target Directory Inventory

The purge engine operates across all supported operating systems, identifying and sanitizing the full matrix of Antigravity assets:

| Platform | Category | Exact Path Pattern | Description |
| :--- | :--- | :--- | :--- |
| **All OS** | Gemini Core | `~/.gemini/antigravity` | Brain cache, transcripts, logs, memory store |
| **All OS** | Gemini Alt | `~/.gemini/antigravity-ide`, `~/.gemini/antigravity-cli` | Auxiliary CLI/IDE logs |
| **All OS** | AGM Data | `~/.antigravity_tools/instances/` | Multi-instance user data directories |
| **Windows** | AppData Roaming | `%APPDATA%\Antigravity` | Electron user data, session storage, GPU cache |
| **Windows** | AppData Local | `%LOCALAPPDATA%\Programs\Antigravity` | Installed application binaries |
| **Windows** | Local Cache | `%LOCALAPPDATA%\Antigravity-Updater`, `%TEMP%\antigravity*` | Update packages and staging temp files |
| **macOS** | App Support | `~/Library/Application Support/Antigravity` | Electron user data, session tokens |
| **macOS** | Application | `/Applications/Antigravity.app`, `~/Applications/Antigravity.app` | Installed app bundle |
| **macOS** | Caches & Logs | `~/Library/Caches/com.google.antigravity`, `~/Library/Logs/Antigravity` | OS-level application caches |
| **Linux** | Config | `~/.config/Antigravity`, `~/.config/antigravity` | User settings and session storage |
| **Linux** | Cache | `~/.cache/Antigravity`, `~/.cache/antigravity` | Electron caches |
| **Linux** | App Storage | `/opt/antigravity`, `~/.local/share/antigravity` | Installed binaries |

### 4.2 Safe 4-Stage Purge Protocol

```
┌────────────────────────────────────────────────────────────────────────┐
│                   STAGE 1: Process Sweep & Caller Guard                │
│  - Check detect_caller_instance_id()                                   │
│  - If caller PID in target list and !force -> ABORT WITH EXIT CODE 2   │
│  - Graceful WM_CLOSE / SIGTERM -> 1000ms delay -> TerminateTree PIDs   │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   STAGE 2: Credential Safeguard Snapshot               │
│  - If keep_accounts == true:                                           │
│    - Read accounts.json, secure_tokens.db, credentials.json            │
│    - Snapshot to temporary staging vault: %TEMP%/agm-purge-vault/      │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   STAGE 3: Deep Filesystem Wipe                        │
│  - Eradicate ~/.gemini/antigravity/ (brain, logs, transcripts)         │
│  - Remove Electron Cache, Code Cache, GPUCache, Service Worker         │
│  - Remove ~/.antigravity_tools/instances/*/data/                       │
│  - Delete binary installations if full application wipe selected       │
└───────────────────────────────────┬────────────────────────────────────┘
                                    │
                                    ▼
┌────────────────────────────────────────────────────────────────────────┐
│                   STAGE 4: Automated Clean Re-provisioning             │
│  - Re-create ~/.gemini/antigravity/ scaffolding                        │
│  - Restore preserved accounts from staging vault                       │
│  - Re-inject default settings: turboMode=true, planReview=false        │
│  - Re-detect fresh base Antigravity executable                         │
└────────────────────────────────────────────────────────────────────────┘
```

#### Stage 1: Caller IDE Self-Preservation Implementation
```rust
// In src-tauri/src/modules/instance/purge_reinstall.rs
pub fn execute_purge_protocol(options: PurgeOptions) -> Result<PurgeSummary, AppError> {
    let caller_ctx = crate::modules::instance::caller_process_ancestry_tree_scanner::detect_caller_instance_id();
    if let Some(caller_pid) = caller_ctx.caller_pid {
        if !options.force {
            eprintln!(
                "[PROTECTION] Operation targets running caller IDE session (PID {}). Aborting to prevent session destruction.",
                caller_pid
            );
            eprintln!("[PROTECTION] Rerun with '--force' if you genuinely intended to terminate the active workspace.");
            std::process::exit(crate::modules::instance::caller_ide_exit_code_enforcer::CALLER_PROTECTION_EXIT_CODE);
        }
    }
    // Proceed with process sweep...
}
```

### 4.3 CLI Subcommand Specification

```bash
# Safe purge keeping accounts intact
agm ide purge-reinstall [--keep-accounts] [--force] [--json]

# Deep complete wipe including account credentials
agm ide purge-reinstall --wipe-all --force
```

### 4.4 UI Counterpart: `PurgeReinstallModal.tsx`

Located in `src/components/instances/PurgeReinstallModal.tsx`.
- Modal triggered from Instances Header "Clean Purge & Reinstall" button.
- Warning Callout (Dark Glass Red/Amber style):
  - "This operation permanently eradicates memory fog, corrupt brain state, conversation logs, and Electron caches."
- Configuration Checkboxes:
  - `[x] Safeguard and restore authenticated accounts (Recommended)`
  - `[x] Purge multi-instance workspaces & data caches`
  - `[ ] Full binary uninstall and fresh re-provisioning`
- Destructive Confirmation Guard:
  - User must type `"PURGE"` into a confirmation input field to enable the submit action.
- "Execute Clean Purge & Reinstall" button with multi-stage progress bar showing Stage 1 to Stage 4 completion.

---

## 5. Section 3: Install IDE Engine

### 5.1 Multi-IDE Platform Installation Matrix

The engine manages discovery and automated installation for three primary developer IDE environments:

| IDE Target | Windows Installation Flow | macOS Installation Flow | Linux Installation Flow |
| :--- | :--- | :--- | :--- |
| **Antigravity IDE** (`antigravity`) | PowerShell automated downloader: fetch latest Windows installer MSI/EXE from Google release CDN, execute with `/passive /norestart` | Download DMG via curl, attach via `hdiutil attach`, copy `.app` to `/Applications/`, strip quarantine `xattr -cr` | Download official AppImage / `.deb`, install via `dpkg -i` or chmod `+x` AppImage to `~/.local/bin/` |
| **Cursor AI** (`cursor`) | `winget install Anysphere.Cursor --silent --accept-package-agreements` (fallback: direct setup exe) | `brew install --cask cursor` (fallback: direct DMG mount & install) | AppImage download / `snap install cursor --classic` |
| **Visual Studio Code** (`vscode`) | `winget install Microsoft.VisualStudioCode --silent --accept-package-agreements` | `brew install --cask visual-studio-code` | `snap install code --classic` / apt repository add & install |

### 5.2 Implementation Module: `src-tauri/src/modules/ide_installer/`

Structure:
- `src-tauri/src/modules/ide_installer/mod.rs`: Public API facade.
- `src-tauri/src/modules/ide_installer/types.rs`: `IdeTarget`, `InstallOptions`, `InstallStatus`, `PlatformInstallCommand`.
- `src-tauri/src/modules/ide_installer/windows.rs`: PowerShell & Winget dispatch ladder.
- `src-tauri/src/modules/ide_installer/macos.rs`: Homebrew & DMG mount execution ladder.
- `src-tauri/src/modules/ide_installer/linux.rs`: Snap, APT, and AppImage installation ladder.
- `src-tauri/src/modules/ide_installer/verify.rs`: Post-installation executable detection and validation.

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum IdeTarget {
    Antigravity,
    Cursor,
    VsCode,
}

impl IdeTarget {
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().as_str() {
            "antigravity" | "agy" | "google-antigravity" => Some(Self::Antigravity),
            "cursor" | "cursor-ai" => Some(Self::Cursor),
            "vscode" | "code" | "vs-code" => Some(Self::VsCode),
            _ => None,
        }
    }
}
```

### 5.3 CLI Interface Specification

```bash
# Install specific IDE
agm ide install [antigravity|cursor|vscode] [--dry-run] [--verbose]

# Example outputs:
# [AGM] Detecting platform: Windows x86_64
# [AGM] Resolving installer package for 'Antigravity IDE'...
# [AGM] Executing silent installation: winget install Google.Antigravity ...
# [SUCCESS] Antigravity IDE installed successfully at %LOCALAPPDATA%\Programs\Antigravity\Antigravity.exe
```

### 5.4 UI Component: `InstallIdeModal.tsx`

Located in `src/components/instances/InstallIdeModal.tsx`.
- Modal triggered from Instances Header CTA button: "Install IDE" (with Sparkles / Download icon).
- 3-Card Grid Selection:
  - **Antigravity IDE**: Official Google AI pair-programming environment. Shows status: `Installed (v1.12.0)` or `Not Detected`.
  - **Cursor AI**: AI-first code editor with Claude 3.5 & GPT-4o integration. Shows status: `Installed` or `Not Detected`.
  - **Visual Studio Code**: Core extensible editor. Shows status: `Installed` or `Not Detected`.
- Installation Log Drawer: Real-time stdout stream during package installation.
- "Verify & Register Path" button immediately refreshing `locations.rs` detection cache.

---

## 6. Section 4: Comprehensive E2E Verification Plan

This section provides the exhaustive verification protocol covering all eight requirements of task `226`.

### 6.1 Requirements Verification Matrix

| Req # | Requirement Description | Target Area / Module | Verification Check & Acceptance Criteria |
| :--- | :--- | :--- | :--- |
| **R1** | **Non-Blocking Async Clone Engine** | `src-tauri/src/commands/instance.rs`, `src-tauri/src/modules/instance/clone.rs` | Window title never displays `(Not Responding)` during 5GB+ instance cloning. `tokio::task::spawn_blocking` decouples disk copying from Win32 message pump. Real-time progress events stream to UI. |
| **R2** | **Clone Modal Simplification** | `src/pages/instances/CopyInstanceDialog.tsx`, `CopyInstanceModal.tsx` | Profile copy radio option completely removed. Duplication locked to full IDE copy. `copy_projects: true` invariant visibly badged in emerald. |
| **R3** | **Two-Level Card Actions Reversion** | `src/pages/instances/InstanceCardActions.tsx` | Single crammed 10-button pill capsule decomposed into two structured rows: **Level 1 (Lifecycle - 5 buttons)** and **Level 2 (Utilities/Tools - 6 buttons)**. |
| **R4** | **Expiration-Urgency Auto-Selection** | `src-tauri/src/modules/auto_switcher/scoring.rs`, `src/services/instanceService/scoring.ts` | Multiplier $E_{\text{urgency}}$ prioritizes accounts with $\le 24\text{h}$ or $\le 48\text{h}$ remaining quota ($\ge 15\%$). Accounts with 4h remaining score higher than 168h accounts. |
| **R5** | **IDE Profile Bundle Import / Export** | `bundle_export.rs`, `bundle_import.rs`, `IdeBundleModal.tsx`, `agm ide export/import` | CLI and UI successfully export complete bundle (`.agmide` / `.json`) containing settings, projects, and prompts, and import back into new or existing instances. |
| **R6** | **Full Clean Purge & Reinstall** | `purge_reinstall.rs`, `PurgeReinstallModal.tsx`, `agm ide purge-reinstall` | Deep wipe of `~/.gemini/antigravity`, brain, logs, and Electron caches. Caller IDE self-preservation enforces exit code 2 without `--force`. Safeguards accounts when requested. |
| **R7** | **Multi-IDE Installation Engine** | `ide_installer/`, `InstallIdeModal.tsx`, `agm ide install` | CLI and UI reliably execute silent installs for Antigravity, Cursor, and VS Code across Windows, macOS, and Linux, updating discovery cache. |
| **R8** | **UI Glitches & Performance Remediation** | `PromptInstructionView.tsx`, `PromptTreeViewModal.tsx`, `InstancesHeader.tsx` | Nested `backdrop-blur-2xl` removed, base64 image reads throttled to concurrency 3, `loadTree` debounced, preventing UI lockups. |

### 6.2 Formal Verification Scripts & CLI Commands

```bash
# 1. Test Profile Bundle Export
agm ide export default --out ./test-bundle.agmide --json
# Verification: Assert exit code 0, file exists, valid JSON, item_counts > 0

# 2. Test Profile Bundle Import to New Instance
agm ide import ./test-bundle.agmide --new-name "Import-Test-Instance"
# Verification: Assert exit code 0, instance registered in agm instance ls

# 3. Test Caller IDE Self-Preservation during Purge
# (From within active IDE terminal without --force)
agm ide purge-reinstall
# Verification: Assert process fails-fast with EXIT CODE 2 and outputs [PROTECTION] warning

# 4. Test Purge with Force and Account Safeguard
agm ide purge-reinstall --keep-accounts --force
# Verification: Assert exit code 0, ~/.gemini/antigravity emptied, accounts.json preserved

# 5. Test IDE Installer Discovery & Dry Run
agm ide install antigravity --dry-run
agm ide install cursor --dry-run
agm ide install vscode --dry-run
# Verification: Assert exit code 0, valid platform command generated
```

---

## 7. Non-Negotiable Invariants & Guardrails

1. **Strictly Relative Git Paths Invariant**: All file references in specifications, subtasks, commit messages, and documentation must use repository-relative paths (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`, `src/...`). No absolute paths permitted.
2. **Caller IDE Self-Preservation Invariant**: Any destructive purge, kill, or stop operation targeting the active developer IDE session must abort with exit code `2` (`CALLER_PROTECTION_EXIT_CODE`) unless `--force` is explicitly provided.
3. **FIFO Queue & Prompt Order Invariant**: Active prompt exports, dispatches, and restores must query SQLite databases strictly using `ORDER BY created_at ASC, id ASC`.
4. **No Silent Errors Rule**: Every fallible Rust operation must propagate via `AppError` or record via `crate::error::record_ignored` with an explicit justification comment. Every TypeScript catch block must route to `useErrorStore.getState().trackWarning`.
5. **Zero Builds / Zero Tests Rule**: No compilation or test execution during specification authoring.
