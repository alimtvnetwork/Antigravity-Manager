---
name: agm-multi-instance-sandboxing
description: Specialized skill for managing Antigravity-Manager multi-instance profile sandboxing, conscious PID matching, OS keyring bypass markers, volatile session purging, and binary cloning.
---

# AGM Multi-Instance Sandboxing & Process Management

This skill provides comprehensive architectural guidance, directory structure rules, process identification algorithms, and sandboxing procedures for the Multi-Instance Management subsystem in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

Antigravity-Manager allows users to run multiple isolated instances of the Antigravity IDE concurrently without credential cross-talk or lockfile conflicts:

```
<data_dir>/instances/
├── instances.json                      # Instance registry & active pointer
├── instances.db                        # SQLite tracking active IDE PIDs (WAL mode)
├── default/                            # Default profile directory
│   └── data/
└── <instance_id>/                      # Isolated sandbox profile
    ├── home/                           # Isolated USERPROFILE / HOME environment
    │   ├── .gemini/
    │   └── AppData/Roaming/Antigravity/
    │       ├── app_storage.json        # Wizard bypass (ide-install-wizard-shown: "true")
    │       └── User/globalStorage/state.vscdb
    └── data/                           # Isolated --user-data-dir
        ├── User/
        │   └── globalStorage/state.vscdb
        ├── AppData/Roaming/Antigravity/User/globalStorage/state.vscdb
        ├── antigravity-keyring-unavailable       # Forces local file token storage
        ├── antigravity-ide-keyring-unavailable   # Language server keyring bypass
        └── antigravity-cli-keyring-unavailable   # CLI companion keyring bypass
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/instance.rs` | Instance CRUD, profile cloning (`full` vs `profile`), directory structure creation, keyring bypass injection, launch orchestration, 3-DB credential sync, and local E2E harness. |
| `src-tauri/src/modules/process.rs` | Conscious PID matching, 10-generation process tree inspection, helper process filtering, command-line arg parsing, and graceful termination. |
| `src-tauri/src/commands/instance.rs` | Registered Tauri IPC commands for instance creation, cloning, launching, stopping, and workspace restarting. |
| `src/stores/useInstanceStore.ts` | Zustand store managing instance lists, active instance ID, rotation status, and loading states. |
| `src/pages/Instances.tsx` | UI page for profile management, status badges, instance cloning dialogs, and workspace recovery triggers. |
| `scripts/test-instance-e2e.ps1` | PowerShell driver for local-only on-demand E2E instance switching tests with dual skip-by-default safety guards. |

---

## 3. Keyring Bypass & Triple Database Credential Sync

To guarantee that secondary sandboxes do not read from or write to the operating system's global credential store:

1. **Keyring Bypass Markers (`write_keyring_bypass_markers`)**:
   - Creates three marker files inside `<instance_dir>/data`:
     - `antigravity-keyring-unavailable`
     - `antigravity-ide-keyring-unavailable`
     - `antigravity-cli-keyring-unavailable`
   - Each file contains `"1\n"`.
   - Forces the Antigravity language server to bypass Windows Credential Manager, macOS Keychain, or Linux Secret Service, storing tokens locally inside that profile's `state.vscdb`.
2. **SSH Emulation Environment Variables**:
   - Sandboxed processes are spawned with dummy SSH environment variables:
     - `SSH_CONNECTION="127.0.0.1 50000 127.0.0.1 22"`
     - `SSH_CLIENT="127.0.0.1 50000 22"`
     - `SSH_TTY="pty/0"`
     - `WSL_DISTRO_NAME="AntigravityWSL"`
   - Electron and language server routines interpret this as a remote headless session, bypassing GUI keyring calls.
3. **Triple Database Token Sync**:
   - Injected OAuth tokens and device IDs are written to `state.vscdb` across three distinct paths to cover Electron and extension host lookups:
     1. `<data_dir>/User/globalStorage/state.vscdb`
     2. `<data_dir>/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`
     3. `<home_dir>/AppData/Roaming/Antigravity/User/globalStorage/state.vscdb`
   - Also writes standalone fallback file `jetski-standalone-oauth-token`.
4. **Onboarding Wizard Suppression (`update_instance_app_storage`)**:
   - Injects `app_storage.json` inside the instance's home directory.
   - Sets `ide-install-wizard-shown: "true"`, `jetski.onboarding.lastLoginUsername`, and `jetski.onboarding.lastLoginIsGcpTos: true` to suppress setup modals on first launch.

---

## 4. Conscious PID Matching Lineage Algorithm (`instance.rs` & `process.rs`)

When identifying, stopping, or switching instances, Antigravity-Manager must distinguish the root IDE window from background helpers and other applications:

1. **10-Generation Parent Map Tree**:
   - Scans system processes via `sysinfo` and constructs a child-to-parent lookup table `parent_map: HashMap<u32, u32>`.
2. **Helper Process Exclusion (`is_helper_process`)**:
   - Excludes sub-processes containing: `--type=renderer`, `--type=gpu-process`, `--type=utility`, `node-ipc`, `crashpad`, `audio`, `sandbox`, and `language_server`.
3. **Self-Immunity**:
   - Excludes own process binaries (`agm.exe`, `antigravity-manager.exe`, `agm-alim`).
4. **Host IDE Ancestor Immunity Gate**:
   - When evaluating candidates for the `default` primary IDE:
     - Climbs the ancestor chain up to 10 generations.
     - If ANY ancestor belongs to an isolated secondary instance (`instance_root_pids`), the process is **strictly rejected**.
     - Prevents sandboxed helper tasks from being misidentified as the developer's host IDE.
5. **Command-Line & Binary Binding (`find_pids_for_data_dir` & `matches_instance`)**:
   - Matches `--user-data-dir=<path>` arguments against normalized instance paths.
   - Strictly rejects candidate PIDs whose process arguments do not match the target instance `data_dir`, logging safety filter rejections.
   - For cloned executables, matches binary name `antigravity-{instance_id}`.
   - Traverses descendant trees to capture all child worker processes of matched root PIDs.
6. **Argument Sanitization (`sanitize_restart_args`)**:
   - Strips internal runtime flags (`--standalone`, `--override_ide_name`, `--subclient_type`) before relaunching the IDE.
7. **Volatile Session Clearing (`purge_volatile_instance_sessions`)**:
   - Sweeps and purges `Local Storage` and `Session Storage` directories inside the instance directory to prevent stale Web storage state across account switches.

---

## 5. Volatile Session Purging & Workspace Cleanup

When an instance is reset or prepared for account rotation (`cleanAndRestartWorkspace` / `wipe_instance_session`):
- Safely terminates the conscious root PID and awaits process tree exit.
- Purges volatile directories: `Local Storage`, `Session Storage`, `Network`, `GPUCache`, `Cache`, `Code Cache`, `blob_storage`.
- Removes locking artifacts: `lockfile`, `code.lock`.
- Preserves user settings, keybindings, and workspace configuration files intact.

---

## 6. Local E2E Testing Invariants (Plan 83 / Spec 72)

- The local E2E test suite in `instance.rs` is protected by dual skip-by-default isolation:
  1. `#[ignore = "local_only_e2e"]` prevents automated CI/CD runners from executing it.
  2. Runtime check requires `RUN_TEMP_E2E=1` environment variable.
