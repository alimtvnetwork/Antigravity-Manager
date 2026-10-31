# Execution Plan: Root Cause Analysis and End-to-End Testing

## Overview
Comprehensive plan for diagnosing and resolving Prompt Tree timestamp/turn discrepancies, restoring full attached image preview rendering, validating cross-machine Telegram prompt dispatching over GitMap SSH, and executing end-to-end testing.

## Task Breakdown
- [x] **Subtask 01**: Prompt Tree Timestamp and Running Turn Discrepancy RCA (`.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/01-prompt-tree-discrepancy-rca.md`)
- [x] **Subtask 02**: Attached Image Preview and Security Guard Remediation (`.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/02-image-loading-and-security-guard.md`)
- [x] **Subtask 03**: Cross-Machine Telegram Dispatching and GitMap SSH Mesh (`.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/03-cross-machine-telegram-gitmap-ssh.md`)
- [ ] **Subtask 04**: End-to-End Verification, Pre-flight Checks, and Release (`.ai-memory/plans/subtasks/root-cause-analysis-and-end-to-end-testing/04-e2e-verification-and-release.md`)

## Status Ledger
| Subtask | Status | Owner | Evidence |
|---|---|---|---|
| 01 | Completed | Lead | `tree_collect.rs` live active write checks & turn badges |
| 02 | Completed | Lead | `cmd_files.rs` sensitive path fix & Base64 preloading |
| 03 | Completed | Lead | `node_label.rs` GitMap IP resolution to `w1` & Telegram reports |
| 04 | In Progress | Lead | Running pre-flight checks and confirming CI/Release |
