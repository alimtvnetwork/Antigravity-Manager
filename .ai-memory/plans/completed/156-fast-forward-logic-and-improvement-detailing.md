# Plan: Fast-Forward Logic and Improvement Detailing

## User Request (Verbatim)
```text
You need to write the fast-forward logic into the spec. That's the first thing. Second is that you have another information wrong. It is zero, you mentioned, if it is active. So that is one factor. Another is if it is actually reduced from the four-hour quota to at least 1% or 2%, unless there is any more accounts left, then that would consider as zero, which you have not mentioned, and this is where I found the bug. The current code is selecting account based on fast-forwarding to account which is already in use. Let me give you a screenshot that this account that I picked that already has been used, but that has been selected by fast-forwarding several times. I think you need to find the root cause and then fix it, and then update the app. That's one thing. Another is the improvement that you have so far listed out. I want you to detail out those improvements according to files, how an AI can follow through, how these improvements can impact, where is the place it should update the coding. More journeys you need to include as a sub-point, so that any AI can follow through easily and do implementation of these improvements. Does this make sense?
```

## Architecture Roadmap & Subtask Allocation

1. **Subtask-01: Fast-Forward Specification & System Invariants**
   - File: `.ai-memory/plans/subtasks/156-fast-forward-logic-and-improvement-detailing/01-fast-forward-spec-and-invariants.md`
   - Scope: Document formal specification for Fast-Forward auto-switching under `02-spec/21-app/156-fast-forward-logic-and-improvement-detailing/spec.md`. Detail two-pass candidate ranking, strict 100% 4-hour quota primacy invariant, dual-key (ID + Email) in-use exclusion invariant, and live API pre-switch verification probe.

2. **Subtask-02: Fix Account Selection Bugs (Reduced Quota & In-Use Collisions)**
   - File: `.ai-memory/plans/subtasks/156-fast-forward-logic-and-improvement-detailing/02-fix-account-selection-reduced-quota-and-in-use.md`
   - Scope:
     - In `src-tauri/src/modules/auto_switcher/scoring.rs`: strictly return `0.0` in `score_candidate_account` if $Q_{4\text{h}} < 100.0\%$.
     - In `src-tauri/src/modules/auto_switcher/selection.rs`: eliminate false 100% overrides via `is_account_period_finished`; enforce real quota checks; add current instance and running sibling accounts by both ID and email to exclusions.
     - In `src-tauri/src/modules/auto_switcher/manual.rs` & `instances.rs`: resolve `bound_email` to exclude current and sibling in-use accounts completely.
     - In `src/services/instanceService/scoring.ts` & `src/stores/useInstanceStore/smart.ts`: ensure primary pass strictly filters $Q_{4\text{h}} \ge 100\%$, accounts with $Q_{4\text{h}} < 100\%$ score `0.0` in Pass 1, and in-use exclusions match both ID and email.

3. **Subtask-03: Detail Cataloged Improvements with File-by-File AI Implementation Journeys**
   - File: `.ai-memory/plans/subtasks/156-fast-forward-logic-and-improvement-detailing/03-detail-improvements-file-by-file-ai-journeys.md`
   - Scope: Group all 200 cataloged system, UI, DB REST, and coding guideline improvements by target files and subsystems. Author step-by-step implementation journeys, blast radius impact analysis, and precise code anchors as executable blueprints for autonomous AI agents.

---
Closure (2026-10-23, subtask 190-05): DONE-IN-CODE per research 01 audit of task 190; fast-forward logic verified present in the codebase. Filed to completed.
