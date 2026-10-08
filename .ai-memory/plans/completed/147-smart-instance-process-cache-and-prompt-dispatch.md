# Master Plan: 147-smart-instance-process-cache-and-prompt-dispatch

## 1. Executive Summary
This milestone resolves the critical defect where sending or queueing prompts from the instance section fails or unnecessarily spawns duplicate IDE instances. It implements a smart multi-instance process cache with two-tier PID liveness checking (cached PID verification -> fresh process table re-scan -> only reopen if dead), hardens running prompt detection to eliminate false-positive running badges on idle items, compacts unnecessary bracket tags in the prompt tree view, implements CLI parity with automated E2E tests, and finishes with a minor version bump release (`v4.165.0`).

## 2. Requirements & Traceability Matrix
1. **Prompt Dispatch & Queue RCA**: Determine root cause of `send_prompt` / `enqueue_prompt` failures and unwanted relaunch behavior.
2. **Smart Instance Process Cache**: Cache running Antigravity IDE instances with PID, command line args, and data-dir attributes. Re-verify before launch; never spawn a new instance if already running.
3. **Liveness Double-Check**: If cached PID died, scan active process table. Reopen only when confirmed completely offline.
4. **False Positive Running Elimination**: Audit `compute_project_conversation_tree` and running prompt queries to eliminate false running indicators on completed turns.
5. **Prompt View UI Compaction**: Reduce bracketed items and redundant tags in `PromptTreeViewModal.tsx`.
6. **CLI & E2E Testing**: Verify prompt dispatch, queueing, and process caching via CLI and an automated Python test script.
7. **Release Ceremony**: Minor version bump (`4.164.0 -> 4.165.0`), synchronize manifests, update changelogs/READMEs (`@aukgit` attribution), and publish release.

## 3. Subtask Breakdown
- [x] **Subtask 01:** `01-prompt-send-enqueue-rca-and-process-cache.md` (RCA, process cache, PID liveness double-check, prevent duplicate launches)
- [x] **Subtask 02:** `02-running-detection-hardening-and-tag-compaction.md` (Eliminate false running items, reduce prompt tree bracket tags)
- [x] **Subtask 03:** `03-cli-e2e-testing-and-ipc-parity.md` (CLI prompt send/queue command parity and Python E2E verification harness)
- [x] **Subtask 04:** `04-version-bump-and-release-ceremony.md` (Pre-flight checks, minor bump v4.165.0, changelogs, READMEs, atomic commit & release)

## 4. Invariants & Rules
- Minimalist & Contextual UI: Follow segmented dark-glass capsules (`rounded-full`, shared border, dark-glass backdrop) per `AGENTS.md`.
- Search codebase exclusively via GitMap (`gitmap find`, `gitmap aum search`, `gitmap cat`). Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- Strictly use relative Git paths.
- Mandatory `A = 2, H = 2` subagent spawning at every stage.
