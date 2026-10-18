# Specification: Instance Management and UI/UX Synchronization

- **Slug:** `instance-management-and-ui-ux-synchronization`
- **Specification ID:** `02-spec/21-app/instance-management-and-ui-ux-synchronization/01-instance-management-and-ui-ux-synchronization.md`
- **Status:** `DRAFT`
- **Owner:** @aukgit

---

## 1. Executive Summary & Strategic Rationale

Antigravity-Manager serves as the unified desktop command and proxy gateway managing multi-account Antigravity and VSCode/Cursor instances. A critical usability and operational vulnerability exists in instance lifecycle and process management:

1. **Process Ambiguity & List View Density:** The instance list row presents raw numeric values (such as `1632`) without clear labeling. This number is indeed the Operating System Process ID (**PID**). The list view row layout unnecessarily breaks the executable trailing path and data directory trailing path into separate lines or disjointed segments rather than a concise, single-line format.
2. **Account Switching Desynchronization:** Switching an instance account fails if the underlying process is active. The system fails to terminate the running PID before attempting to reload or re-authenticate the profile.
3. **Broken Process Termination:** The "Stop Instance" action frequently fails to terminate the underlying process tree, leaving orphan processes locking `workspaceStorage` and profile databases.
4. **Disjoint Switch Account UI/UX:** The account switching dialog diverges from the core dark-glass design language, lacking segmented capsules, high-contrast states, and standard modal keyboard accessibility.
5. **Missing Instance Audit Trail:** Inspecting an instance's audit trail displays an empty state due to missing instance ID tagging in `task_history_db` and decoupled frontend query parameters.
6. **Self-Instance Safety Gate:** Automated test suites and CLI operations lack self-instance awareness, risking killing the active development IDE or agent process.

This specification defines the architectural enhancements, backend API contracts, frontend interface refactors, and CLI extensions required to resolve these defects with zero regression.

---

## 2. Process Identification (PID) Confirmation & List View Layout

### 2.1 PID Confirmation
- The numerical indicator (e.g., `1632`) is the Operating System **Process Identifier (PID)** assigned by Windows/Linux/macOS to the instance's primary window or background process.
- **Requirement:** The UI must explicitly prefix the value with `PID: ` (e.g. `PID: 1632`) inside a clean status capsule:
  - **Running:** Green pulse badge (`bg-emerald-500/10 text-emerald-400 border border-emerald-500/20`) with `● Running (PID: 1632)`.
  - **Stopped / Inactive:** Muted zinc badge (`bg-zinc-800/40 text-zinc-400 border border-zinc-700/30`) with `○ Stopped`.
  - **Orphan / Mismatched:** Amber warning badge (`bg-amber-500/10 text-amber-400 border border-amber-500/20`) with `⚠ PID Mismatch (<pid>)`.

### 2.2 List Row Inline Layout
- Currently, long executable paths and user data directories cause awkward multi-line wrapping in table rows.
- **Requirement:** Display the terminal tail of the executable (e.g., `.../antigravity.exe`) and the terminal tail of the user data directory (e.g., `.../data-profile-1`) inline on the same horizontal row, using monospace font, dark-glass pill styling, and full tooltip paths on hover.

---

## 3. Robust Process Lifecycle & Conscious Termination

### 3.1 Conscious Termination Ladder (`conscious_kill_process_tree`)
When stopping an instance or switching accounts, process termination must follow a strict 3-stage fallback ladder:
1. **Stage 1 (Soft Signal):** On Windows, dispatch `WM_CLOSE` to top-level windows of the PID. On Unix, send `SIGTERM`. Wait up to 1,500ms for graceful shutdown.
2. **Stage 2 (Tree Kill):** If the process remains alive, execute tree termination (`taskkill /T /F /PID <pid>` on Windows; `pkill -TERM -P <pid>` followed by `kill -9 <pid>` on Unix).
3. **Stage 3 (Verification & Lock Release):** Verify process absence from the OS process table. Wait for file locks on `User/workspaceStorage` and SQLite databases to release.

### 3.2 Account Switching Invariant
- **Rule:** Before any account credential switch, profile reload, or OAuth refresh on an active instance:
  1. Detect the instance's active PID.
  2. If running, execute `conscious_kill_process_tree`.
  3. Validate termination before updating database state and launching or signaling the instance.

---

## 4. Modern Switch Account Dialog UI/UX

The modal must align with the unified application design tokens:
- **Container:** Dark-glass backdrop blur (`backdrop-blur-md bg-zinc-950/80 border border-zinc-800 rounded-2xl shadow-2xl`).
- **Header:** Title with segmented icon capsule, instance name badge, and explicit `X` close button + `Escape` keybinding.
- **Account Selection:** Interactive card list of available accounts with avatar/email, quota percentage bar, and candidate score badges.
- **Actions:** Segmented pill footer with `Cancel` (subtle zinc) and `Switch & Restart` (primary gradient capsule).

---

## 5. Instance Audit Trail Correlation

### 5.1 Event Tagging in `task_history_db`
- Every instance lifecycle event must record `instance_id` in `task_history_db`:
  - `INSTANCE_START`: Process launched with PID.
  - `INSTANCE_STOP`: Process stopped cleanly.
  - `INSTANCE_KILL`: Process forcefully terminated.
  - `INSTANCE_SWITCH_ACCOUNT`: Account switched from Account A to Account B.
  - `INSTANCE_CREATE` / `INSTANCE_DELETE`: Provisioning and teardown.

### 5.2 Audit Drawer Query Parity
- When opening the instance audit drawer, the query must pass `instance_id: <id>` to retrieve all matching audit rows from the split-db SQLite history database.

---

## 6. Self-Instance Detection & Safe Test Instance CLI

### 6.1 Self-Preservation Gate (`agm instance self`)
- The CLI command `agm instance self` detects whether the current shell/terminal is running within an Antigravity IDE instance by inspecting:
  1. Process tree ancestors for `antigravity.exe` or `code.exe`.
  2. `--user-data-dir` arguments in the process hierarchy.
  3. Environment variables (`VSCODE_PID`, `ANTIGRAVITY_INSTANCE_ID`).
- **Safety Invariant:** Automated tests and termination commands MUST reject killing the current self-instance.

### 6.2 Test Instance Lifecycle
- CLI subcommands for rapid, isolated testing:
  - `agm instance test-create --name <name>`: Fast clone using template config.
  - `agm instance test-destroy --name <name>`: Clean termination of processes and atomic folder removal.

---

## 7. Verification Evidence Contract

1. **Rust Pre-flight:** `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` pass with zero warnings/errors.
2. **Frontend Build:** `npm run build` succeeds without type errors.
3. **E2E Automation:** `scripts/test-instance-management-e2e.ps1` runs end-to-end, validating test instance creation, PID detection, stop instance termination, account switch synchronization, audit trail recording, and test instance teardown.
4. **Minor Release:** Atomic version bump with updated changelogs, READMEs, strictly relative paths, and `@aukgit` attribution.
