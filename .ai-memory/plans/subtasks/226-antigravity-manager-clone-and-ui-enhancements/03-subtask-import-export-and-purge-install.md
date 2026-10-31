# Subtask 03 Engineering Plan: IDE Profile Bundle Import/Export, Clean Purge & Reinstall, Install IDE Engine, and E2E Verification

**Subtask:** `03-subtask-import-export-and-purge-install.md`  
**Parent Task:** `226-antigravity-manager-clone-and-ui-enhancements`  
**File Location:** `.ai-memory/plans/subtasks/226-antigravity-manager-clone-and-ui-enhancements/03-subtask-import-export-and-purge-install.md`  
**Target Specifications:** `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/03-migration-and-verification-spec.md`  
**Target Modules:**  
- `src-tauri/src/modules/instance/bundle_types.rs` (New: AgmIdeProfileBundle schema & models)  
- `src-tauri/src/modules/instance/bundle_export.rs` (New: Profile bundle packaging engine)  
- `src-tauri/src/modules/instance/bundle_import.rs` (New: Profile bundle restoration & merge engine)  
- `src-tauri/src/modules/instance/purge_reinstall.rs` (New: Cross-platform 4-stage purge & re-provisioning engine)  
- `src-tauri/src/modules/ide_installer/mod.rs` (New: Multi-IDE installer subsystem facade)  
- `src-tauri/src/modules/ide_installer/types.rs` (New: IDE target & platform command models)  
- `src-tauri/src/modules/ide_installer/windows.rs` (New: Windows winget & PowerShell installer ladder)  
- `src-tauri/src/modules/ide_installer/macos.rs` (New: macOS Homebrew & DMG installer ladder)  
- `src-tauri/src/modules/ide_installer/linux.rs` (New: Linux Snap, APT, and AppImage installer ladder)  
- `src-tauri/src/modules/ide_installer/verify.rs` (New: Post-installation verification & path updater)  
- `src-tauri/src/modules/instance.rs` (Facade re-exports for bundle and purge modules)  
- `src-tauri/src/modules/cli/instance.rs` (CLI subcommands: export, import, purge-reinstall, install)  
- `src-tauri/src/modules/cli/args.rs` (CLI argument routing for `agm ide ...` commands)  
- `src-tauri/src/commands/instance.rs` (Tauri IPC commands: export, import, purge, install)  
- `src-tauri/src/lib.rs` (Registration in `generate_handler!`)  
- `src/components/instances/IdeBundleModal.tsx` (New: UI modal for profile bundle export & import)  
- `src/components/instances/PurgeReinstallModal.tsx` (New: UI modal for clean purge & reinstall)  
- `src/components/instances/InstallIdeModal.tsx` (New: UI modal for Antigravity, Cursor, and VS Code installation)  
- `src/pages/instances/InstancesHeader.tsx` (Header integration for bundle, purge, and install modals)  
- `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/04-e2e-report.md` (Formal E2E verification ledger)  
**Status:** READY FOR IMPLEMENTATION  

---

## 1. Scope & Objective Breakdown

This subtask governs the complete implementation of **Tasks 07, 08, 09, and 10** from the parent plan `.ai-memory/plans/pending/226-antigravity-manager-clone-and-ui-enhancements.md`:

1. **Task 07: IDE Profile Bundle Import / Export Engine**:
   - Provide complete serialization of an IDE instance into an `.agmide` / `.json` bundle file.
   - Bundle settings (`settings.json`, `keybindings.json`, `snippets/`), projects (`workspace_uris`, `running_projects`, `agm_project_sequences`), and prompts (`active_prompts`, `prompt_suffix_templates`, `prompt_resend_as_templates`, `global_system_prompt`).
   - Implement backend engines, Tauri IPC endpoints, CLI subcommands (`agm ide export`, `agm ide import`), and the React UI modal `IdeBundleModal.tsx`.
2. **Task 08: Full Antigravity Clean Delete & Reinstall Engine**:
   - Deliver a safe 4-stage purge protocol across Windows, macOS, and Linux that systematically eradicates `~/.gemini/antigravity`, brain caches, transcript databases, and Electron state.
   - Enforce **Caller IDE Self-Preservation** via `detect_caller_instance_id()`, terminating with exit code `2` (`CALLER_PROTECTION_EXIT_CODE`) if an unforced purge targets the developer's active workspace session.
   - Provide optional account credential safeguarding (`--keep-accounts`), CLI subcommand (`agm ide purge-reinstall`), and UI confirmation modal `PurgeReinstallModal.tsx`.
3. **Task 09: Install IDE Engine**:
   - Construct a unified multi-IDE installer subsystem for Google Antigravity IDE, Cursor AI, and Visual Studio Code across Windows, macOS, and Linux.
   - Expose CLI command `agm ide install [antigravity|cursor|vscode]` and UI component `InstallIdeModal.tsx` with one-click installation and verification.
4. **Task 10: Comprehensive End-to-End Verification**:
   - Construct the formal E2E report `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/04-e2e-report.md` validating all eight requirements (R1–R8).

---

## 2. Step-by-Step Implementation Instructions

### Step 1: Data Models & Schema (`bundle_types.rs`)

**File to create:** `src-tauri/src/modules/instance/bundle_types.rs`  
**Responsibility:** Define the serializable structures for the `AgmIdeProfileBundle`.

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

---

### Step 2: Bundle Export Packaging Engine (`bundle_export.rs`)

**File to create:** `src-tauri/src/modules/instance/bundle_export.rs`  
**Responsibility:** Export an instance's settings, projects, and prompts into a formatted JSON string or `.agmide` file.

Key Implementation Invariants:
1. Locates `User/settings.json`, `User/keybindings.json`, and `User/snippets/` for the instance.
2. Queries `agm_project_sequences` and `running_projects` from `repo_prompts.db`.
3. Queries `active_prompts` enforcing strict FIFO ordering `ORDER BY created_at ASC, id ASC`.
4. Ingests `prompt_suffix_templates` and `prompt_resend_as_templates`.
5. Calculates SHA-256 checksum over the assembled JSON data.
6. Writes to output file or returns serialized JSON string.

```rust
use super::bundle_types::*;
use crate::error::AppError;
use sha2::{Digest, Sha256};
use std::fs;
use std::path::Path;

pub fn export_ide_profile_bundle(
    instance_id: &str,
    output_path: Option<&Path>,
) -> Result<AgmIdeProfileBundle, AppError> {
    // 1. Resolve instance
    let resolved_id = crate::modules::instance::resolve_instance_id(instance_id)
        .map_err(|e| AppError::Config(format!("Failed to resolve instance: {}", e)))?;
    let registry = crate::modules::instance::load_registry()
        .map_err(|e| AppError::Config(format!("Failed to load registry: {}", e)))?;
    let inst = registry
        .instances
        .iter()
        .find(|i| i.id == resolved_id)
        .ok_or_else(|| AppError::NotFound(format!("Instance '{}' not found", resolved_id)))?;

    // 2. Read Settings, Keybindings, Snippets
    let settings = package_bundle_settings(inst)?;

    // 3. Read Projects & Sequences
    let projects = package_bundle_projects(&resolved_id)?;

    // 4. Read Prompts & Templates (strict FIFO)
    let prompts = package_bundle_prompts(&resolved_id)?;

    // 5. Metadata & Checksum
    let mut item_counts = std::collections::HashMap::new();
    item_counts.insert("workspaces".to_string(), projects.workspace_uris.len());
    item_counts.insert("pinned_projects".to_string(), projects.pinned_project_sequences.len());
    item_counts.insert("active_prompts".to_string(), prompts.active_prompts.len());
    item_counts.insert("suffix_templates".to_string(), prompts.suffix_templates.len());

    let bundle = AgmIdeProfileBundle {
        agm_bundle_version: AGM_IDE_BUNDLE_SCHEMA_VERSION.to_string(),
        exported_at: chrono::Utc::now().to_rfc3339(),
        source_environment: SourceEnvironment {
            instance_id: inst.id.clone(),
            instance_name: inst.name.clone(),
            agm_version: crate::modules::version::get_app_version(),
            platform: std::env::consts::OS.to_string(),
            arch: std::env::consts::ARCH.to_string(),
        },
        settings,
        projects,
        prompts,
        metadata: BundleMetadata {
            bundle_checksum: "".to_string(), // calculated below
            item_counts,
        },
    };

    let serialized = serde_json::to_string_pretty(&bundle)
        .map_err(|e| AppError::Serialization(format!("Failed to serialize bundle: {}", e)))?;
    let mut hasher = Sha256::new();
    hasher.update(serialized.as_bytes());
    let checksum = format!("{:x}", hasher.finalize());

    let mut final_bundle = bundle;
    final_bundle.metadata.bundle_checksum = checksum;

    if let Some(dest) = output_path {
        if let Some(parent) = dest.parent() {
            fs::create_dir_all(parent)
                .map_err(|e| AppError::Io(format!("Failed to create parent dir: {}", e)))?;
        }
        let pretty = serde_json::to_string_pretty(&final_bundle)
            .map_err(|e| AppError::Serialization(format!("Serialization error: {}", e)))?;
        fs::write(dest, pretty)
            .map_err(|e| AppError::Io(format!("Failed to write bundle file: {}", e)))?;
    }

    Ok(final_bundle)
}
```

---

### Step 3: Bundle Import Restoration Engine (`bundle_import.rs`)

**File to create:** `src-tauri/src/modules/instance/bundle_import.rs`  
**Responsibility:** Validate, parse, and apply an `.agmide` bundle to an existing instance or create a new cloned instance.

```rust
use super::bundle_types::*;
use crate::error::AppError;
use std::fs;
use std::path::Path;

pub struct ImportOptions {
    pub target_instance_id: Option<String>,
    pub new_instance_name: Option<String>,
    pub force: bool,
}

pub struct ImportResult {
    pub instance_id: String,
    pub instance_name: String,
    pub settings_applied: usize,
    pub projects_restored: usize,
    pub prompts_queued: usize,
}

pub fn import_ide_profile_bundle(
    bundle_path: &Path,
    options: ImportOptions,
) -> Result<ImportResult, AppError> {
    let content = fs::read_to_string(bundle_path)
        .map_err(|e| AppError::Io(format!("Failed to read bundle: {}", e)))?;
    let bundle: AgmIdeProfileBundle = serde_json::from_str(&content)
        .map_err(|e| AppError::Serialization(format!("Invalid bundle format: {}", e)))?;

    // Determine target instance
    let target_id = if let Some(ref name) = options.new_instance_name {
        // Create new instance profile
        let new_id = format!("inst-{}", &uuid::Uuid::new_v4().to_string()[..8]);
        crate::modules::instance::create_instance(&new_id, name, None, None, false)
            .map_err(|e| AppError::Config(format!("Failed to create instance: {}", e)))?;
        new_id
    } else if let Some(ref tid) = options.target_instance_id {
        crate::modules::instance::resolve_instance_id(tid)
            .map_err(|e| AppError::Config(format!("Failed to resolve target: {}", e)))?
    } else {
        return Err(AppError::Validation("Either target_instance_id or new_instance_name must be specified".to_string()));
    };

    // Apply settings deep merge
    let settings_count = apply_bundle_settings(&target_id, &bundle.settings)?;

    // Restore project sequences and workspaces
    let projects_count = apply_bundle_projects(&target_id, &bundle.projects)?;

    // Restore prompts (mark as QUEUED in FIFO order)
    let prompts_count = apply_bundle_prompts(&target_id, &bundle.prompts)?;

    Ok(ImportResult {
        instance_id: target_id.clone(),
        instance_name: options.new_instance_name.unwrap_or(target_id),
        settings_applied: settings_count,
        projects_restored: projects_count,
        prompts_queued: prompts_count,
    })
}
```

---

### Step 4: Full Purge & Clean Reinstall Engine (`purge_reinstall.rs`)

**File to create:** `src-tauri/src/modules/instance/purge_reinstall.rs`  
**Responsibility:** Execute the 4-stage purge protocol with caller IDE protection.

```rust
use crate::error::AppError;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PurgeOptions {
    pub keep_accounts: bool,
    pub purge_instances: bool,
    pub uninstall_binaries: bool,
    pub force: bool,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct PurgeSummary {
    pub processes_terminated: usize,
    pub directories_purged: Vec<String>,
    pub bytes_reclaimed: u64,
    pub accounts_safeguarded: bool,
}

pub fn execute_purge_protocol(options: PurgeOptions) -> Result<PurgeSummary, AppError> {
    // Stage 1: Caller IDE Protection
    let caller_ctx = crate::modules::instance::caller_process_ancestry_tree_scanner::detect_caller_instance_id();
    if let Some(caller_pid) = caller_ctx.caller_pid {
        if !options.force {
            eprintln!(
                "[PROTECTION] Operation targets running caller IDE session (PID {}). Aborting.",
                caller_pid
            );
            eprintln!("[PROTECTION] Rerun with '--force' to proceed.");
            std::process::exit(crate::modules::instance::caller_ide_exit_code_enforcer::CALLER_PROTECTION_EXIT_CODE);
        }
    }

    // Sweep all Antigravity processes
    let term_count = sweep_antigravity_processes(caller_ctx.caller_pid, options.force)?;

    // Stage 2: Credential Safeguard Snapshot
    let staging_dir = std::env::temp_dir().join("agm-purge-safeguard");
    let mut accounts_safeguarded = false;
    if options.keep_accounts {
        accounts_safeguarded = snapshot_credentials(&staging_dir)?;
    }

    // Stage 3: Deep Filesystem Wipe
    let (purged_dirs, bytes_reclaimed) = wipe_target_directories(&options)?;

    // Stage 4: Clean Re-provisioning
    reprovision_clean_scaffolding(&staging_dir, accounts_safeguarded)?;

    Ok(PurgeSummary {
        processes_terminated: term_count,
        directories_purged: purged_dirs,
        bytes_reclaimed,
        accounts_safeguarded,
    })
}
```

---

### Step 5: Multi-IDE Installer Engine (`src-tauri/src/modules/ide_installer/`)

**Files to create:**
1. `src-tauri/src/modules/ide_installer/mod.rs`
2. `src-tauri/src/modules/ide_installer/types.rs`
3. `src-tauri/src/modules/ide_installer/windows.rs`
4. `src-tauri/src/modules/ide_installer/macos.rs`
5. `src-tauri/src/modules/ide_installer/linux.rs`
6. `src-tauri/src/modules/ide_installer/verify.rs`

Key Features:
- Supported IDEs: `Antigravity`, `Cursor`, `VsCode`.
- Executes platform-specific silent install:
  - Windows: `winget install ... --silent --accept-package-agreements` or PowerShell direct CDN installer.
  - macOS: `brew install --cask ...` or curl DMG mount.
  - Linux: `snap install ... --classic` or AppImage fetch.
- Re-verifies executable path and refreshes `locations.rs` detection cache.

---

### Step 6: Facade & CLI Wiring

1. **Update `src-tauri/src/modules/instance.rs`**:
   - Add submodules:
     ```rust
     #[path = "instance/bundle_types.rs"]
     pub mod bundle_types;
     #[path = "instance/bundle_export.rs"]
     pub mod bundle_export;
     #[path = "instance/bundle_import.rs"]
     pub mod bundle_import;
     #[path = "instance/purge_reinstall.rs"]
     pub mod purge_reinstall;
     ```
   - Re-export `export_ide_profile_bundle`, `import_ide_profile_bundle`, `execute_purge_protocol`.
2. **Update `src-tauri/src/modules/cli/args.rs`**:
   - Add `"ide"` command routing to `handle_ide_subcommand`.
3. **Update `src-tauri/src/modules/cli/instance.rs`**:
   - Implement `handle_ide_subcommand` supporting:
     - `agm ide export [spec] [--out <path>]`
     - `agm ide import <bundle> [--target <spec>] [--new-name <name>]`
     - `agm ide purge-reinstall [--keep-accounts] [--force]`
     - `agm ide install [antigravity|cursor|vscode] [--dry-run]`
4. **Update `src-tauri/src/commands/instance.rs` & `lib.rs`**:
   - Register IPC handlers:
     - `export_ide_bundle`
     - `import_ide_bundle`
     - `execute_purge_reinstall`
     - `install_ide_target`
     - `detect_ide_installation_status`

---

### Step 7: Frontend Modals & Header Integration

1. **`src/components/instances/IdeBundleModal.tsx`**:
   - Export and Import tabs with file picker, item summary, checksum display, and progress bar.
2. **`src/components/instances/PurgeReinstallModal.tsx`**:
   - Dark-glass caution banner, account preservation toggle, confirmation text input ("PURGE"), and 4-stage progress tracker.
3. **`src/components/instances/InstallIdeModal.tsx`**:
   - 3-card grid (Antigravity IDE, Cursor AI, VS Code) with installed status badges, live install log drawer, and verify button.
4. **`src/pages/instances/InstancesHeader.tsx`**:
   - Add "IDE Profile Bundle (.agmide)" option in Import/Export popover menu.
   - Add "Clean Purge & Reinstall" option in maintenance actions.
   - Add "Install IDE" segmented CTA button in toolbar.

---

### Step 8: Comprehensive End-to-End Verification (Task 10)

Author `02-spec/21-app/226-antigravity-manager-clone-and-ui-enhancements/04-e2e-report.md` documenting:
- Verification of R1 (Async clone, no ghost window).
- Verification of R2 (No profile copy option, full copy locked).
- Verification of R3 (Two-level card actions restored).
- Verification of R4 (Urgency multiplier applied to expiring accounts).
- Verification of R5 (Bundle export and import round-trip).
- Verification of R6 (Purge protocol with caller IDE exit code 2 guard).
- Verification of R7 (IDE installer execution and path discovery).
- Verification of R8 (UI glitch remediation).

---

## 3. Concrete Verification Criteria & Invariants

| Action / Command | Expected Exit Code | Expected Output / State |
| :--- | :--- | :--- |
| `agm ide export default --out test.agmide --json` | `0` | File `test.agmide` created with valid JSON schema and SHA-256 checksum |
| `agm ide import test.agmide --new-name "VerifyInst"` | `0` | New instance created in registry, settings and sequences restored |
| `agm ide purge-reinstall` (from caller IDE without `--force`) | `2` | Fails immediately with `[PROTECTION]` warning, caller IDE stays alive |
| `agm ide purge-reinstall --keep-accounts --force` | `0` | Purges caches and logs, preserves and restores `accounts.json` |
| `agm ide install antigravity --dry-run` | `0` | Displays platform-specific installation command without executing |

---

## 4. Non-Negotiable Invariants & Guardrails

1. **Zero Builds / Zero Tests Rule**: No compilation (`cargo build`, `npm run build`) or unit tests during authoring.
2. **Strictly Relative Git Paths**: All paths must use relative paths (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`, `src/...`).
3. **Caller IDE Self-Preservation**: Exit code `2` strictly enforced on unforced caller termination.
4. **No Silent Errors Rule**: All errors must route to `AppError` or `useErrorStore`.
