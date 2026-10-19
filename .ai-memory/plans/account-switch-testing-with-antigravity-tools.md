# Account Switch Testing with Antigravity Tools - Master Execution Plan

## User Request (Verbatim)

```text
# Account Switch Testing with Antigravity Tools: high priority instruction, non-negotiable task

Hi there. Can you please test out the account switch with a new instance? That even the account switch is happening manually or automatically, it should actually run again the running prompt, which should be tested by CLI from the `Antigravity Manager or Antigravity Tools` package. It should be tested with the `gitmap`, with its command. So at the end, you should summarize like you have tested this, that you are running a prompt. Let's say you could give it a deep analysis with image. Of course, the image needs to be inserted back in when we switch the account. But also first test like you can take up with the back image and prompt and can resend back. Test that first, then check before the fast-forward switch, you can take back up the last running prompt and resend it back after switching of the account. So also wait for 10 seconds in order to see that the account actually switched properly, the IDE is open and running. And then you do this using the CLI from `Antigravity Manager or Antigravity Tools` and also the `gitmap`. You should have both CLI commands in order to do that, and also from the UI, and also you should check in the `Antigravity Manager or Antigravity Tools`. There are a lot of log errors. Try to find those log errors and try to find and fix those errors. Then for `Antigravity Manager or Antigravity Tools`, try to check also the steps into the auto-switching. Auto-switching also tell us that what it is doing. I mean, what are the steps it is performing? What are the prompts it is backing up in the audit log, the images, and is it successful or not? After it restores back, what is the status of those prompt? Things like that. So this is really, really must. So make sure that you test it properly. I think it's missed. Is it clear? Is it clear?

## task: Account Switch Testing with Antigravity Tools
## slug: account-switch-testing-with-antigravity-tools

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/account-switch-testing-with-antigravity-tools/ and enqueue plan task in .ai-memory/plans/account-switch-testing-with-antigravity-tools.md (subtasks in .ai-memory/plans/subtasks/account-switch-testing-with-antigravity-tools/) first
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Test account switch with a new instance using CLI from `Antigravity Manager or Antigravity Tools` and `gitmap`.
6. Ensure the running prompt is tested and summarized, including deep analysis with images.
7. Verify image insertion back during account switch and test backup and resend of the last running prompt.
8. Wait 10 seconds to confirm account switch, IDE status, and functionality.
9. Check for log errors in `Antigravity Manager or Antigravity Tools` and fix them.
10. Analyze auto-switching steps, audit log backups, and prompt restoration status.
```

---

## 5 Mandatory Pillars

### Pillar 1: Strategic Intent & Systems Reasoning
Validate the complete prompt resumption and account switching pipeline in Antigravity-Manager (`agm`) and `gitmap`. The primary objective is to prove that running prompts—including multimodal image payloads—are safely backed up into SQLite (`backup_prompts_db.rs`), persisted to resumption artifacts (`.antigravity_resume_task.<instance>.json`), and faithfully re-injected post-switch after a mandatory 10-second process settling period.

### Pillar 2: Explicit Scope Boundaries
- **In-Scope**:
  - Test sandbox creation via `agm instances test-create` / `agm instances create`.
  - Prompt seeding with deep analysis text and multimodal image assets.
  - Backup verification via `agm backup-prompts` and SQLite query inspection.
  - Account switch execution via `agm switch` and `gitmap switch` with 10s settling.
  - Verification of prompt & image restoration in the new session.
  - Auto-switcher steps & audit log inspection.
  - Log error diagnosis, fixing runtime faults, running `agm errors fix`, and full sandbox teardown.
- **Excluded / Deferred**:
  - Terminating or touching the developer's caller IDE session (`default-copy-5597`).
  - Production credential alterations or destructive resets.

### Pillar 3: Non-Negotiable Constraints & Negative Guardrails
- **Search Ban**: TOTAL BAN on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`. Use GitMap exclusively.
- **Strictly Relative Paths**: All paths in specs, plans, commits, and logs must be relative (`02-spec/...`, `.ai-memory/...`).
- **Caller IDE Self-Preservation**: Automated tests must fail-fast if targeting the caller instance unless `--force` is specified.
- **Concurrency & Subagents**: $A = 3$ concurrent subagents at all stages (planning, spec, execution).

### Pillar 4: Blast Radius & Dependency Topology
- Isolated entirely to transient test instances (`test-*` or `inst-test-*`).
- Core prompt database records are backed up with clear test instance scoping.

### Pillar 5: Multi-Agent Concurrency Topology
- **Capacity**: $A = 3$, $H = 2$, System Capacity = 6.
- **Waves**: 3-Phase execution lifecycle across 8 granular subtasks.

---

## Master Task Ledger

| Task ID | Description | State | Owner | Owned Files | Verification Evidence |
|---|---|---|---|---|---|
| **Task-01** | Git Pull Preflight & Baseline Hygiene | `COMPLETED` | Lead Orchestrator | `src-tauri/src/modules/repo_db/text_utils.rs` | `gitmap pull` exit code 0 |
| **Task-02** | Comprehensive Specification Authoring | `COMPLETED` | Spec Authoring Wave | `02-spec/21-app/account-switch-testing-with-antigravity-tools/*` | 3 Spec documents & 6 subtasks authored |
| **Task-03** | Transient Test Sandbox Creation | `COMPLETED` | Worker 1 | Transient test profile | Sandbox created with ID `test-switch-test-1792431608-1608` |
| **Task-04** | Multimodal Image Prompt Seeding & Backup | `COMPLETED` | Worker 2 | `repo_prompts.db`, `backup_prompts.db` | Backed up prompt with image (`has_images: 1`, payload: 182 bytes) |
| **Task-05** | Account Switch Execution & 10s Settling | `COMPLETED` | Worker 1 | Process manager, `accounts.json` | Switched to `riseup.asia.team@gmail.com`, 10s wait, PID active |
| **Task-06** | Prompt & Multimodal Image Restoration Verification | `COMPLETED` | Worker 2 | `repo_prompts.db`, `backup-prompts.db` | Restored prompt with image, `is_restored: 1`, status: `dispatched` |
| **Task-07** | Auto-Switcher Steps & Audit Trail Analysis | `COMPLETED` | Worker 3 | `task_history_db.rs`, audit trail | Queried `hot_tasks_cache` (`LaunchInstance`, `SchedulePrompt`, `StopInstance`) |
| **Task-08** | Error Diagnostics, Self-Healing & Sandbox Cleanup | `COMPLETED` | Worker Swarm | `errors.db`, test sandboxes | `agm errors fix` 8/8 PASSED, fixed SQLite constraint in `create.rs` and added `remove_dir_all_force` |

---

## Discrete Subtask Definitions

### Task-01: Git Pull Preflight & Baseline Hygiene
- **State**: `COMPLETED`
- **Output**: Verified repository up to date via `gitmap pull`.

### Task-02: Comprehensive Specification Authoring
- **State**: `COMPLETED`
- **Files**:
  - `02-spec/21-app/account-switch-testing-with-antigravity-tools/01-architecture-spec.md`
  - `02-spec/21-app/account-switch-testing-with-antigravity-tools/02-switch-and-prompt-backup-spec.md`
  - `02-spec/21-app/account-switch-testing-with-antigravity-tools/03-verification-and-audit-spec.md`

### Task-03: Transient Test Sandbox Provisioning & Image Prompt Seeding
- **State**: `COMPLETED`
- **Goal**: Provisioned isolated test instance `test-switch-test-1792431608-1608` via `agm instances test-create switch-test --json`.

### Task-04: Running Prompt & Multimodal Image Backup Verification
- **State**: `COMPLETED`
- **Goal**: Seeded prompt with deep analysis text and Base64 PNG image (`has_images: 1`, 182 bytes), verified in `backup_prompts.db:prompt_backups` and `.antigravity_resume_task.<inst>.json`.

### Task-05: Account Switch Execution with 10-Second Liveness Verification
- **State**: `COMPLETED`
- **Goal**: Switched account to `riseup.asia.team@gmail.com` via `agm instances switch`, tested `gitmap switch` command parity, and observed 10s settling delay while protecting caller IDE `default-copy-5597`.

### Task-06: Prompt & Multimodal Image Resend / Restoration Verification
- **State**: `COMPLETED`
- **Goal**: Restored prompt post-switch; verified `is_restored = 1`, `restored_at = 1792432020`, status: `dispatched`, and image payload intact.

### Task-07: Auto-Switcher Steps, Audit Log, & Telemetry Analysis
- **State**: `COMPLETED`
- **Goal**: Queried `hot_tasks_cache` in `task_index.db`, verifying `LaunchInstance` (code 6), `StopInstance` (code 7), and `SchedulePrompt` (code 4) audit records.

### Task-08: Log Error Audit, Self-Healing, and Test Sandbox Cleanup
- **State**: `COMPLETED`
- **Goal**: Ran `agm errors fix` (8/8 PASSED, [HEALTHY]), resolved SQLite `NOT NULL constraint failed: instance_processes.pid` in `src-tauri/src/modules/instance/create.rs`, added `remove_dir_all_force` in `src-tauri/src/utils/fs.rs`, purged all test instances via `agm instances test-clean --json`, and passed `cargo fmt -- --check`, `cargo check --bin agm`, and `npm run build`.
