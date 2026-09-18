# 14. Multi-Instance Orchestration and Ubuntu Parallelism (Completed)

> **Execution Milestone**: Completed across 200 continuous self-loop steps.
> **Scope**: Multi-Profile Creation, Terminal CLI Options, Selective Linux Process Supervision via `/proc/<pid>/cmdline`, AppImage Sanitization, GNOME Keyring Bypass (`--password-store=basic`), and Frontend Navbar / Instances Tab UI.

## Summary of Completed Implementation

### 1. Subtask 01: Backend Instance Supervisor and CLI
- **Data Models** (`src-tauri/src/models/instance.rs`):
  - Defined `InstanceConfig`, `InstanceStatus`, and `InstanceRegistry` with strict positive boolean naming (`is_default`, `is_running`).
- **Core Supervisor** (`src-tauri/src/modules/instance.rs`):
  - Manages isolated profiles stored in `<app_data>/instances/<id>/data`.
  - Supports `list_instances`, `create_instance`, `copy_instance`, `delete_instance`, `wipe_instance_session`, `launch_instance`, `close_instance`, `get_active_instance`, `set_active_instance`.
  - Selective process inspection and termination: matches `--user-data-dir` argument, selectively signals only the target PID tree (`kill -15 <pid>`). Sibling instances remain unaffected.
  - Linux AppImage environment variable sanitization via `clean_appimage_env(&mut cmd)`.
  - Keyring collision bypass via `--password-store="basic"` ensuring credentials remain inside local `state.vscdb`.
- **Terminal CLI Support** (`src-tauri/src/modules/cli.rs`):
  - Terminal flags: `--create-profile <name>`, `--list-profiles`, `--copy-profile <src> <target>`, `--delete-profile <id>`, `--run-profile <id>`, `--profile-help`.
  - Hooked directly into `src-tauri/src/lib.rs` startup before GUI initialization.
- **Tauri IPC Commands** (`src-tauri/src/commands/instance.rs`):
  - Exposes all instance and account injection operations to the frontend.

### 2. Subtask 02: Frontend Instance Store, Navbar Dropdown, and Dedicated Tabs
- **Service Layer** (`src/services/instanceService.ts`):
  - TypeScript interfaces and Tauri `invoke` bindings for all instance APIs.
- **Zustand Store** (`src/stores/useInstanceStore.ts`):
  - Reactive instance list, active instance tracker, and state mutation methods.
- **Navbar Integration** (`src/components/navbar/InstanceSelector.tsx` & `Navbar.tsx`):
  - Top navigation bar dropdown showing active instance with live PID running badge.
  - Quick clone (❐) and quick create (+) profile modals.
  - Added primary navigation item for `/instances` ("Instances" / "多实例").
- **Dedicated Management View** (`src/pages/Instances.tsx`):
  - Full management cards with status badges, PID information, data directory paths, and action controls (Launch, Close, Duplicate, Wipe Session, Delete).

### 3. Subtask 03: Account Dispatch Integration & Verification
- **Targeted Account Switch** (`src/components/accounts/AccountRow.tsx` & `AccountTable.tsx`):
  - Left-click on switch button (`⇄`) automatically targets the active instance selected in the Navbar.
  - Right-click / context menu allows immediate routing to any specific instance without changing global target.
  - Direct token injection into the instance's isolated `state.vscdb`.
  - Zero sibling termination: other instances running other accounts stay untouched.
