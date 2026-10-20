# Subtask 01: Git Sync, Environment Baseline & Preflight Discovery

**Parent Plan Reference**: [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md](../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md)  
**Parent Task**: `cli-and-ui-testing-for-antigravity-backups`  
**Subtask ID**: `01-subtask-git-sync-and-preflight`  
**Target Area**: Preflight, Git Sync Verification, Caller Session Discovery, Toolchain & Ledger  
**Status**: APPROVED SUBTASK PLAN  
**Attribution**: strictly `@aukgit`  

---

## 1. Scope & Objectives

1. Verify upstream repository synchronization and working tree baseline prior to code modification.
2. Detect and safeguard the developer's active caller IDE instance (`detect_caller_instance_id()`) to prevent termination or state corruption during subsequent CLI test runs.
3. Validate local build and linting toolchains (Rust `cargo`, Node.js `npm`, `gitmap` CLI suite) to ensure pre-flight quality gates are operational.
4. Audit Split SQLite storage structures (`repo_prompts.db`, `backup_prompts.db`, `gui_config.json`) and verify schema readiness for running prompt inspection and multimodal backup records.
5. Initialize and synchronize the multi-agent task ledger (`.ai-memory/temp-agents/cli-and-ui-testing-for-antigravity-backups/ledger.md`).

---

## 2. Invariants & Guardrails

- **Strict Relative Git Paths Invariant**: All referenced paths in plans, documentation, and telemetry MUST be repository-relative (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`). Absolute paths with drive letters (`C:\...`, `D:\...`) are strictly banned.
- **Caller IDE Self-Preservation**: Automated tests, daemons, and CLI discovery commands MUST detect and protect the caller's active IDE session (e.g., `default-copy-5597`). Termination of the active caller session is strictly prohibited.
- **Subagent Git Command Ban**: Subagents are strictly forbidden from executing mutating Git commands (`git add`, `git commit`, `git push`, `git checkout`). Only the lead orchestrator or developer manages Git state.
- **GitMap Primacy**: Codebase exploration, file listing, and symbol discovery must use `gitmap` commands (`gitmap lf`, `gitmap find`, `gitmap aum search`, `gitmap cat`). Standard unindexed grep/ripgrep commands are banned.
- **No Silent Errors Invariant**: All preflight validations, database queries, and environment probes must propagate failures or record warnings via `crate::error::record_ignored` with justification.

---

## 3. Detailed Implementation Steps

### Step 1: Upstream Sync & Working Tree State Baseline
- Confirm the repository working tree is clean or properly accounted for before starting feature work.
- Validate that upstream changes have been integrated via `gitmap pull` (or fallback `git pull`) without unmerged conflict markers (`<<<<<<<`, `=======`, `>>>>>>>`).
- Record baseline branch state (`main`) and commit hash in the execution ledger.

### Step 2: Caller Session & Process Discovery
- Locate the active developer IDE instance using `detect_caller_instance_id()` from `src-tauri/src/bin/agm/system_cmds.rs` and `src-tauri/src/modules/instance.rs`.
- Identify the caller instance identifier (e.g. `default-copy-5597`, `inst-test`) and active PID.
- Mark this instance identifier as protected in test runner parameters:
  - Any automated test or switcher script targeting the caller instance must fail-fast with exit code 2 unless `--force` is explicitly provided.
  - Test suites (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) must spawn ephemeral sandboxed test instances (e.g. `test-backup-sandbox`) rather than modifying the caller profile.

### Step 3: Toolchain Preflight & Quality Gates Validation
- Verify Rust compiler and Cargo toolchain:
  ```bash
  cargo --version
  rustc --version
  ```
- Run local rustfmt check across the workspace:
  ```bash
  cd src-tauri && cargo fmt -- --check
  ```
- Verify Node.js and frontend build environment:
  ```bash
  npm --version
  node --version
  ```
- Verify GitMap binary availability and responsive CLI interface:
  ```bash
  gitmap --version
  ```

### Step 4: Split SQLite Storage & Workspace State Audit
- Verify the integrity and writeability of Split SQLite databases:
  - `repo_prompts.db`: Check table `active_prompts` and ensure indexes on `(status, created_at)` exist.
  - `backup_prompts.db`: Check table `prompt_backups` and verify columns `id`, `instance_id`, `prompt_content`, `image_payload`, `created_at`.
- Verify `gui_config.json` existence and validate fallback handling if `prompt_preview_max_words` is absent.
- Ensure temporary directories (`.ai-memory/temp-agents/cli-and-ui-testing-for-antigravity-backups/`) exist with write permissions.

### Step 5: Multi-Agent Task Ledger Initialization
- Author and verify the task tracking ledger in `.ai-memory/temp-agents/cli-and-ui-testing-for-antigravity-backups/ledger.md`.
- Register all discrete tasks (Task-01 through Task-08), assigned owners, owned disjoint file paths, and current execution phases.
- Confirm state: Task-01 marked as `DONE` upon preflight completion.

---

## 4. Verification & Quality Gates

| Verification Item | Command / Check | Expected Result | Pass/Fail Criteria |
|:---|:---|:---|:---|
| **Git Working Tree** | `gitmap pull` / check status | No merge conflict markers | Zero conflict markers |
| **Caller Protection** | `detect_caller_instance_id()` | Identifies active caller instance | Non-empty caller ID shielded |
| **Cargo Fmt** | `cd src-tauri && cargo fmt -- --check` | Clean formatting output | Exit code 0 |
| **GitMap Accessibility** | `gitmap --version` | Prints valid version banner | Exit code 0 |
| **SQLite Schema Audit** | SQLite table probe | `active_prompts` & `prompt_backups` present | Tables accessible |
| **Ledger Verification** | Inspect `ledger.md` | Tasks Task-01 to Task-08 registered | File exists & formatted |

---

## 5. Traceability & Cross-References

- Canonical Spec: [02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md](../../../02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/01-architecture-spec.md)
- Next Subtask: [.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/02-subtask-architecture-and-specs.md](./02-subtask-architecture-and-specs.md)
- Master Plan: `.ai-memory/plans/cli-and-ui-testing-for-antigravity-backups.md`
- Execution Ledger: `.ai-memory/temp-agents/cli-and-ui-testing-for-antigravity-backups/ledger.md`
