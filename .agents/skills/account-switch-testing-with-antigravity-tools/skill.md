---
name: account-switch-testing-with-antigravity-tools
description: >-
  Specialized skill for testing manual and automatic account switching, running prompt and image backup/resumption, 10-second settling verification, audit trail inspection, and Speed Error diagnostics (errors.db) in Antigravity-Manager.
---

# Account Switch Testing with Antigravity Tools

## Overview

This skill guides automated and end-to-end testing of Antigravity-Manager account switching workflows, multimodal prompt resumption, and diagnostic telemetry:
1. **Isolated Sandbox Provisioning**: Safely provisioning transient test instances (`agm instances test-create`) with conscious PID matching and zero caller IDE interference.
2. **Multimodal Running Prompt Seeding**: Seeding active prompts with rich task descriptions and embedded images in `active_prompts` and `backup_prompts`.
3. **Pre-Switch Backup**: Ensuring atomic prompt and image capture prior to account switching (`agm backup-prompts`).
4. **Account Switch & 10-Second Settling**: Executing account switches via `agm switch` and `gitmap switch`, enforcing a mandatory 10-second process settling period, and verifying IDE process liveness.
5. **Post-Switch Prompt & Image Restoration**: Re-injecting prompt content and image attachments into the newly active instance (`.antigravity_resume_task.<inst>.json` and direct SQLite dispatch).
6. **Auto-Switcher Audit & Steps Analysis**: Decoding decision steps, candidate quota scoring, and action logs in `task_history_db.rs` and `backup_prompts_db.rs`.
7. **Speed Error Diagnostics (`errors.db`)**: Inspecting AGM diagnostic error codes (E1001–E9001), running automated self-healing (`agm errors fix`), and tearing down test sandboxes completely (`agm instances test-clean`).

## Key Invariants & Safety Rules

- **Caller IDE Self-Preservation**: Automated tests must NEVER terminate or interfere with the developer's active working session (caller instance PID 32 / `default-copy-5597`).
- **Strictly Relative Paths**: All path references in specifications, test outputs, and audit logs must be relative repository paths (`02-spec/...`, `.ai-memory/...`).
- **Zero Secrets**: Never print, log, or commit OAuth tokens, refresh credentials, or API keys.
- **Search Primacy**: Use GitMap (`gitmap aum search`, `gitmap find`) exclusively for codebase discovery.
