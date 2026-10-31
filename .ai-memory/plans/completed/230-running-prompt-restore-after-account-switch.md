# Master Execution Plan: 230-running-prompt-restore-after-account-switch

- **Parent Task Slug**: `running-prompt-restore-after-account-switch`
- **Parent Task Sequence**: `230`
- **Target Release**: Minor Version Bump (v4.271.0)
- **Status**: COMPLETED
- **Prompt Hash**: Verified
- **Created**: 2026-10-27
- **Completed**: 2026-10-27

---

## 1. Executive Summary & Intent
This task addresses a critical regression where running prompts are not restored following an account switch (both manual account switches and automated quota auto-switches).

### Core Questions to Answer & Fix:
1. **Why does the running prompt not get restored?**:
   - Where in the pipeline does account switching capture the active prompt?
   - Is `backup_running_prompts_for_switch` being called before or after instance processes are terminated?
   - How does `restore_running_prompts_after_switch` locate the backed-up prompts? Does it look up by `instance_id`, `project_id`, or `account_hash`?
   - Does `repo_db::save_or_requeue_prompt` or `restore_prompt_to_queue` fail silently, or does the restore cursor drop the prompt?
   - Does the 10-second settling delay (`SETTLING_DELAY_SECS`) block, abort early, or fail to trigger queue drain after the switch completes?
   - What happens to the transcript parsing in `discovery.rs` / `transcript.rs`? Does it find zero active prompts because the instance was killed before the backup was taken?
2. **Comprehensive 4-Part Root Cause Analysis (RCA)**:
   - Part 1: Proximate Cause
   - Part 2: Root Cause Mechanism
   - Part 3: Architecture & Invariant Breach
   - Part 4: Surgical Elimination & Permanent Safeguards
3. **End-to-End Verification**:
   - Unit tests covering prompt backup capture, settling delay countdown, and post-switch restore injection.
   - Rust pre-flight checks: `cargo fmt -- --check`, `cargo check` / `cargo clippy`.
   - Frontend build: `npm run build`.
4. **Minor Version Bump Release Ceremony**:
   - Synchronize SemVer version across `package.json`, `package-lock.json`, `Cargo.toml`.
   - Update `CHANGELOG.md` and `CHANGELOG_EN.md` with sole attribution to `@aukgit`.
   - Synchronize `README.md` and `README_EN.md` changelog sections.
   - Commit and push atomically via GitMap.

---

## 2. Work Breakdown & Subtask Waves

```text
Wave 1 (Deep Discovery & Forensic RCA Swarm, A = 3):
  ├── Research 01: Account Switch Flow (Manual & Auto-Switcher) & Backup Sequence [DONE]
  ├── Research 02: Prompt Extraction from Transcripts & SQLite Active Prompts Store [DONE]
  └── Research 03: Post-Switch Restore Pipeline, 10s Settling Delay & Queue Dispatch [DONE]

Wave 2 (Planning & Spec Authoring Swarm, A = 3):
  ├── Spec Author 01: 01-architecture-spec.md & 01-subtask-architecture-and-contracts.md [DONE]
  ├── Spec Author 02: 02-component-spec.md & 02-subtask-capture-and-restore.md [DONE]
  └── Spec Author 03: 03-e2e-and-release-spec.md & 03-subtask-e2e-and-release.md [DONE]

Wave 3 (Surgical Fix Implementation Swarm, A = 3):
  ├── Worker 01: Pre-Switch Running Prompt Capture Order & Transcript Extraction [DONE]
  ├── Worker 02: Post-Switch Restore Invariant, Settling Timer & Instant Queue Awakening [DONE]
  └── Worker 03: E2E Unit Test Suites & Frontend Restore Status Visibility [DONE]

Wave 4 (Verification & Release Ceremony):
  └── Lead: E2E Test Execution, Cargo Check, Frontend Build, Minor Version Bump & GitMap Release [DONE]
```
