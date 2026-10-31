---
name: running-prompt-restore-after-account-switch
description: Autonomously diagnose, analyze, and resolve running prompt backup and restore failures across manual and automated account switching, enforce 10s settling delays, restore queue dispatch invariants, verify via comprehensive E2E testing, and execute minor version bump release ceremony.
---

# Running Prompt Restore After Account Switch

## Mandates & Objectives
- **Root Cause Elimination**: Diagnose why running prompts fail to restore after account switches (inspecting discovery, transcript extraction, backup capture, process termination, post-switch 10s settling timer, and queue re-injection).
- **FIFO Restore & Queue Invariant**: Ensure restored prompts are queued in strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`, waking the dispatch queue worker immediately.
- **Deep Transcript Inspection**: Ensure transcript parsing prioritizes `transcript_full.jsonl`, reverse-scanning to capture genuine user prompts and attachments rather than stale telemetry or empty strings.
- **E2E Testing & Verification**: Execute complete end-to-end verification across unit tests, cargo check, frontend build, and automated test suites.
- **Minor Version Bump Release Ceremony**: Execute version synchronization, changelog generation (solely attributed to `@aukgit`), README updates, and GitMap release commit.
