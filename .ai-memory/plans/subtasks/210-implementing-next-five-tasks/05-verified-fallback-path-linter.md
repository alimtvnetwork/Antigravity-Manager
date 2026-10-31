# Subtask 05: Verified Fallback Path Linter (`Imp-679`)
Status: COMPLETED
Parent Plan: 210-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Risky Path Replacement:
"Keep an explicit fallback to the verified path whenever you replace it (e.g. '0 matches found → full restart'). A silent no-op is worse than the failure it was meant to fix."

Implement `src-tauri/src/modules/release_gate/verified_fallback_path_linter.rs` and unit tests in `src-tauri/src/modules/release_gate/tests_verified_fallback_path_linter.rs`.

## Requirements
- Scan operational replacement logic across critical pathways (fast process kill, switcher retry, model fallback).
- Detect speculative branches that fail to provide an explicit fallback to the proven baseline execution path.
- Reject unconditional silent no-ops when replacement operations encounter zero matches or partial failures.
- Provide structured audit report tracking examined branch sites, verified fallbacks, and detected silent no-ops.
