---
plan: auto-switch-and-ui-functionality-issues
subtask: "02"
title: Subprocess Window Suppression via CREATE_NO_WINDOW (0x08000000) Across 14 Files and Callsites
domain: backend-rust-windows-subprocesses
depends_on: none
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/01-architecture-and-quota-autoswitch-spec.md
  master_plan: ../../auto-switch-and-ui-functionality-issues.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - src-tauri/src/modules/ssh_manager/deploy_keys.rs
  - src-tauri/src/modules/ssh_manager/exec.rs
  - src-tauri/src/modules/ssh_manager/keys.rs
  - src-tauri/src/modules/ssh_manager/mesh.rs
  - src-tauri/src/modules/ssh_manager/sync.rs
  - src-tauri/src/modules/telegram_inbound/cluster.rs
  - src-tauri/src/modules/telegram_inbound/commands.rs
  - src-tauri/src/modules/telegram_inbound/injection.rs
  - src-tauri/src/modules/telegram_inbound/open_shot.rs
  - src-tauri/src/modules/telegram_inbound/processing.rs
  - src-tauri/src/modules/telegram_inbound/reports_cluster_a.rs
  - src-tauri/src/modules/telegram_inbound/ui_remote.rs
  - src-tauri/src/modules/repo_db/tree_format.rs
  - src-tauri/src/modules/git_info.rs
status: pending
---

# Subtask 02: Subprocess Window Suppression via `CREATE_NO_WINDOW` (`0x08000000`) Across 14 Files and Callsites

## 1. Context & Motivation

When the user clicks Auto-Switch, Fast-Forward, or triggers automated background tasks (SSH key deployment, screenshot captures, prompt tree formatting, cluster queries, and git telemetry), Windows opens transient, flashing black console / PowerShell windows.

### Root Cause
In Rust on Windows, `std::process::Command::new(...)` by default inherits the parent console or requests the Win32 subsystem to allocate a standard console window via `CreateProcessW`. When executing tools like `icacls.exe`, `ssh.exe`, `powershell.exe`, `gitmap.exe`, `clip.exe`, and `git.exe`, this manifests as visible command prompt windows popping up, stealing focus and disrupting the user workflow.

### Architectural Invariant
All background subprocess invocations across the Windows backend MUST enforce console window suppression by applying `CREATE_NO_WINDOW` (`0x08000000`) or using the codebase's existing helper `creation_flags_windows()` from [`src-tauri/src/utils/command.rs`](file:///src-tauri/src/utils/command.rs):
```rust
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

// On Windows Command:
cmd.creation_flags(0x08000000);
```

---

## 2. Comprehensive Inventory: 14 Target Files & ~25 Callsites

| Index | Target File | Line(s) | Binary / Command | Invocations Description |
| :---: | :--- | :---: | :--- | :--- |
| **01** | [`src-tauri/src/modules/ssh_manager/deploy_keys.rs`](file:///src-tauri/src/modules/ssh_manager/deploy_keys.rs) | L125, L139 | `icacls` | Key file NTFS ACL permissions granting (`/inheritance:r /grant SYSTEM:(F)`). 2 callsites. |
| **02** | [`src-tauri/src/modules/ssh_manager/exec.rs`](file:///src-tauri/src/modules/ssh_manager/exec.rs) | L224, L273, L291 | `ssh`, `gitmap`, `where.exe` | Cluster command execution, `gitmap --version` probe, and `where.exe gitmap` resolution. 3 callsites. |
| **03** | [`src-tauri/src/modules/ssh_manager/keys.rs`](file:///src-tauri/src/modules/ssh_manager/keys.rs) | L88, L150, L234 | `ssh-keygen`, `clip` | Public key fingerprint computation (`-lf`), key generation (`-t ed25519`), and clipboard copy. 3 callsites. |
| **04** | [`src-tauri/src/modules/ssh_manager/mesh.rs`](file:///src-tauri/src/modules/ssh_manager/mesh.rs) | L103, L106, L208, L300 | `ssh` | Remote key deployment, remote auth verification (`AGM_SSH_OK`), public key gathering, and batch sync. 4 callsites. |
| **05** | [`src-tauri/src/modules/ssh_manager/sync.rs`](file:///src-tauri/src/modules/ssh_manager/sync.rs) | L177 | `gitmap` | CLI node discovery (`gitmap ssh nodes export-json`). 1 callsite. |
| **06** | [`src-tauri/src/modules/telegram_inbound/cluster.rs`](file:///src-tauri/src/modules/telegram_inbound/cluster.rs) | L58 | `gitmap` | Cluster status query (`gitmap cluster status`). 1 callsite. |
| **07** | [`src-tauri/src/modules/telegram_inbound/commands.rs`](file:///src-tauri/src/modules/telegram_inbound/commands.rs) | L23, L98 | `gitmap`, `agm` | Execution of GitMap subcommands and AGM binary dispatch. 2 callsites. |
| **08** | [`src-tauri/src/modules/telegram_inbound/injection.rs`](file:///src-tauri/src/modules/telegram_inbound/injection.rs) | L302, L306 | `gitmap` | Prompt delegation to remote cluster node (`gitmap cluster exec` / `gitmap ssh exec`). 2 callsites. |
| **09** | [`src-tauri/src/modules/telegram_inbound/open_shot.rs`](file:///src-tauri/src/modules/telegram_inbound/open_shot.rs) | L157, L263, L337, L429, L447 | `powershell.exe`, `gitmap` | Window focus script, remote UI opening, screenshot window capture script, and remote PNG capture. 5 callsites. |
| **10** | [`src-tauri/src/modules/telegram_inbound/processing.rs`](file:///src-tauri/src/modules/telegram_inbound/processing.rs) | L399 | `gitmap` | Inbound Telegram message delegation to remote node (`gitmap cluster exec`). 1 callsite. |
| **11** | [`src-tauri/src/modules/telegram_inbound/reports_cluster_a.rs`](file:///src-tauri/src/modules/telegram_inbound/reports_cluster_a.rs) | L90, L247, L267 | `gitmap` | Remote prompt queries, `gitmap agy active`, and `gitmap agy queues`. 3 callsites. |
| **12** | [`src-tauri/src/modules/telegram_inbound/ui_remote.rs`](file:///src-tauri/src/modules/telegram_inbound/ui_remote.rs) | L933 | `powershell.exe` | Remote UI window capture helper script execution. 1 callsite. |
| **13** | [`src-tauri/src/modules/repo_db/tree_format.rs`](file:///src-tauri/src/modules/repo_db/tree_format.rs) | L303, L307 | `gitmap` | Prompt tree formatting and SSH node delegation. 2 callsites. |
| **14** | [`src-tauri/src/modules/git_info.rs`](file:///src-tauri/src/modules/git_info.rs) | L19, L37, L55, L78 | `git` | Git hash (`rev-parse --short`), branch (`rev-parse --abbrev-ref`), last tag (`describe --tags`), and status probe. 4 callsites. |

**Total Callsites**: 2 + 3 + 3 + 4 + 1 + 1 + 2 + 2 + 5 + 1 + 3 + 1 + 2 + 4 = **34 callsites across 14 files**.

---

## 3. Surgical Code Modification Details

### 3.1 Pattern Standard
For standard `std::process::Command`:
```rust
#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

let mut cmd = Command::new("binary_name");
#[cfg(target_os = "windows")]
cmd.creation_flags(0x08000000); // CREATE_NO_WINDOW
```
Alternatively, with `crate::utils::command::CommandExtWrapper`:
```rust
use crate::utils::command::CommandExtWrapper;

let mut cmd = Command::new("binary_name");
cmd.creation_flags_windows();
```

---

### 3.2 File-by-File Remediation Matrix

#### 1. [`src-tauri/src/modules/ssh_manager/deploy_keys.rs`](file:///src-tauri/src/modules/ssh_manager/deploy_keys.rs)
- **Lines 125 & 139**:
  ```rust
  #[cfg(target_os = "windows")]
  use std::os::windows::process::CommandExt;

  let mut cmd = Command::new("icacls");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  cmd.args([...]).output()
  ```

#### 2. [`src-tauri/src/modules/ssh_manager/exec.rs`](file:///src-tauri/src/modules/ssh_manager/exec.rs)
- **Line 224**:
  ```rust
  let mut cmd = Command::new("ssh");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  match cmd.args(&ssh_args).output() { ... }
  ```
- **Line 273**:
  ```rust
  let mut cmd = Command::new("gitmap");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  if let Ok(out) = cmd.arg("--version").output() { ... }
  ```
- **Line 291**:
  ```rust
  let mut cmd = Command::new(which_cmd.0);
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  if let Ok(out) = cmd.arg(which_cmd.1).output() { ... }
  ```

#### 3. [`src-tauri/src/modules/ssh_manager/keys.rs`](file:///src-tauri/src/modules/ssh_manager/keys.rs)
- **Line 88**:
  ```rust
  let mut cmd = Command::new("ssh-keygen");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  if let Ok(out) = cmd.args(["-lf", &pub_path.to_string_lossy()]).output() { ... }
  ```
- **Line 150**:
  ```rust
  let mut cmd = Command::new("ssh-keygen");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  let out = cmd.args([...]).output()?;
  ```
- **Line 234**:
  ```rust
  let mut cmd = Command::new("clip");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  if let Ok(mut child) = cmd.stdin(Stdio::piped()).spawn() { ... }
  ```

#### 4. [`src-tauri/src/modules/ssh_manager/mesh.rs`](file:///src-tauri/src/modules/ssh_manager/mesh.rs)
- **Lines 103, 106, 208, 300**:
  Wrap all `Command::new("ssh")` with `.creation_flags(0x08000000)` on Windows.

#### 5. [`src-tauri/src/modules/ssh_manager/sync.rs`](file:///src-tauri/src/modules/ssh_manager/sync.rs)
- **Line 177**:
  ```rust
  let mut cmd = std::process::Command::new("gitmap");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  if let Ok(out) = cmd.args(args).output() { ... }
  ```

#### 6. [`src-tauri/src/modules/telegram_inbound/cluster.rs`](file:///src-tauri/src/modules/telegram_inbound/cluster.rs)
- **Line 58**:
  ```rust
  let mut cmd = Command::new("gitmap");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  let output_res = cmd.args(["cluster", "status"]).output();
  ```

#### 7. [`src-tauri/src/modules/telegram_inbound/commands.rs`](file:///src-tauri/src/modules/telegram_inbound/commands.rs)
- **Line 23**:
  ```rust
  let mut cmd = Command::new("gitmap");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  let output_res = cmd.args(&parts).output();
  ```
- **Line 98**:
  ```rust
  let mut cmd = Command::new(&bin);
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  match cmd.args(&parts).output() { ... }
  ```

#### 8. [`src-tauri/src/modules/telegram_inbound/injection.rs`](file:///src-tauri/src/modules/telegram_inbound/injection.rs)
- **Lines 302 & 306**:
  Apply `cmd.creation_flags(0x08000000)` to both `gitmap cluster exec` and fallback `gitmap ssh exec`.

#### 9. [`src-tauri/src/modules/telegram_inbound/open_shot.rs`](file:///src-tauri/src/modules/telegram_inbound/open_shot.rs)
- **Line 157 & Line 337**:
  ```rust
  let mut cmd = Command::new("powershell.exe");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  let output = cmd.args([...]).output()?;
  ```
- **Lines 263, 429, 447**:
  Apply `cmd.creation_flags(0x08000000)` on all `Command::new("gitmap")` callsites.

#### 10. [`src-tauri/src/modules/telegram_inbound/processing.rs`](file:///src-tauri/src/modules/telegram_inbound/processing.rs)
- **Line 399**:
  Apply `cmd.creation_flags(0x08000000)` on `Command::new("gitmap")`.

#### 11. [`src-tauri/src/modules/telegram_inbound/reports_cluster_a.rs`](file:///src-tauri/src/modules/telegram_inbound/reports_cluster_a.rs)
- **Lines 90, 247, 267**:
  Apply `cmd.creation_flags(0x08000000)` on all 3 `Command::new("gitmap")` callsites.

#### 12. [`src-tauri/src/modules/telegram_inbound/ui_remote.rs`](file:///src-tauri/src/modules/telegram_inbound/ui_remote.rs)
- **Line 933**:
  ```rust
  let mut cmd = Command::new("powershell.exe");
  #[cfg(target_os = "windows")]
  cmd.creation_flags(0x08000000);
  let output = cmd.args([...]).output()?;
  ```

#### 13. [`src-tauri/src/modules/repo_db/tree_format.rs`](file:///src-tauri/src/modules/repo_db/tree_format.rs)
- **Lines 303 & 307**:
  ```rust
  let mut cmd1 = std::process::Command::new("gitmap");
  #[cfg(target_os = "windows")]
  cmd1.creation_flags(0x08000000);
  let output = cmd1
      .args(["ssh", "exec", &remote_cmd, "--node", node])
      .output()
      .or_else(|_| {
          let mut cmd2 = std::process::Command::new("gitmap");
          #[cfg(target_os = "windows")]
          cmd2.creation_flags(0x08000000);
          cmd2.args(["ssh", "exec", &remote_cmd]).output()
      });
  ```

#### 14. [`src-tauri/src/modules/git_info.rs`](file:///src-tauri/src/modules/git_info.rs)
- **Lines 19, 37, 55, 78**:
  Add `cmd.creation_flags(0x08000000)` across all 4 `Command::new("git")` callsites.

---

## 4. Verification Protocol

1. **Static Analysis & Compilation Gate**:
   - `cd src-tauri && cargo clippy --all-targets --all-features`
   - `cd src-tauri && cargo fmt -- --check`
2. **Runtime Execution Verification**:
   - Run AGM on Windows.
   - Trigger Auto-Switch, Fast-Forward, SSH key scan, and screenshot captures.
   - Assert zero flashing black terminal / PowerShell windows appear during execution.
