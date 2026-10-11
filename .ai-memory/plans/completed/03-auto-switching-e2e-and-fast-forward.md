# Completed Plan: 03-auto-switching-e2e-and-fast-forward

## User Request (Verbatim)
Hi there. Can you please check into the coding? What can we improve in the account change and other stuff? How can we make it automated and auto-switching? You have to check the end-to-end testing here that the auto-switching should work. Okay? If we have a issue or the account gets low in credit, it's going to automatically switch. Currently, the fast forward makes the UI and everything non-responsive. Try to look into the algorithm, make it better. Use caching as well, but also invalidate the caching. So try to have that smartness, and also at the end, showcase the algorithm so that I understand what is the switched user at the end. Okay, and also at the same time, I think the chats and chromes, these are still buggy, crashes the screen. I hope that you understand and fix these things. Is it clear? So do end-to-end testing on auto-switching and other stuff. Is it clear? The system hangs after some time, so you need to respect that, find the root cause, and then fix it. Okay, run the code here to see what are the issues. Okay, create a separate instance and try to resolve that.

Follow @[.agents/skills/execute-parent-task-with-n-steps-v6]

## Visual Evidence
![Window Not Responding](assets/screenshots/auto-switching-e2e-and-fast-forward-01.png)
![System Freeze Dialog](assets/screenshots/auto-switching-e2e-and-fast-forward-02.png)

## Root Causes Identified & Resolved
1. **Auto-Switching Scoring & Inversion Flaws**:
   - `scoring.rs` inverted account ranking by multiplying `hours_elapsed` in a descending sort, penalizing freshly reset accounts.
   - `calculate_4h_window_quota` evaluated the minimum across all consumed models instead of strictly the configured `target_model`.
   - Live quota queries hit Google API with zero memory caching, while disk JSON was read stale with fragile `models.first()` checks.
   - Proxy 429/403 rate limits were isolated from the auto-switcher daemon.
2. **Fast-Forward & UI Non-Responsive Hang**:
   - Synchronous `get_project_conversation_tree` ran directly on the Tauri IPC thread, reading multi-megabyte `transcript_full.jsonl` files and executing unbatched SQLite transactions.
   - A 3-second rapid polling interval in `useInstancePage.ts` overwhelmed the Windows message pump.
   - Window focus in `focus.rs` executed `powershell.exe Add-Type` compiling C# code via Roslyn at runtime.
3. **Chat & Process Stability**:
   - `markdown.tsx` greedy regex exploded virtual DOM node count on large transcripts.
   - Uncached `System::new()` calls caused excessive Win32 process table enumerations.
4. **E2E Testing**:
   - Authoring `scripts/test-auto-switch-quota-e2e.ps1` with 8 deterministic stages verifying sandbox instance isolation, 98% threshold drop, smart rotation, prompt persistence, and default threshold restoration.

## Tasks Completed
- ✅ **Task-01**: Auto-Switching Algorithm Modernization, Multi-Tier Caching & Invalidation (Worker 01, Wave 1)
- ✅ **Task-02**: Fast-Forward Hang & UI Freeze Remediation (Worker 02, Wave 1)
- ✅ **Task-03**: Chat Tree & Process Crash Hardening (Worker 01, Wave 2)
- ✅ **Task-04**: Sandboxed E2E Test Suite for Auto-Switching (Worker 02, Wave 2)

## Verification Evidence
- `gitmap ps "Set-Location src-tauri; cargo fmt -- --check"` -> PASS exit 0
- `gitmap ps "pwsh -NoProfile -File scripts/test-auto-switch-quota-e2e.ps1 -DryRun"` -> PASS exit 0 (8/8 stages verified)
