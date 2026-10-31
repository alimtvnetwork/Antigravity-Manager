# Plan: Fleet Node Navigation & Error Log Remediation (Non-Destructive Plan Mode)

## User Request (Verbatim)
> "Antigravity Manager (`Antigravity Manager (AGM)`) have lots of errors in the log. I have recorded it. Can you please look into those error logs and write into the plan mode? Because the code is going into refactor, we cannot fix it. Make sure we put this as a plan so that we can fix it later on. Also, try to tie it with the new refactors. If something changes, include that into the plan. Do not execute or fix this issue. Include the log, create a detailed AI instruction so that AI can do the fixes later on. The feed section, or when we are in the instance tab, the other machine information is still not visible. Check the gitmap, that gitmap connects with the nodes correctly. But in your case, we do not have the connection with the other machine, and we cannot get the information here in the UI level and also the CLI level. You need to have navigation command like gitmap. You need to have help checkouts, information that you could request to other Antigravity Manager (`Antigravity Manager (AGM)`) through SSH or communications like gitmap. The Antigravity Manager (`Antigravity Manager (AGM)`) should be able to read gitmap configuration aliasing and can influence itself as well. Focus on this."

## Executive Summary
This architectural master plan addresses two critical operational capabilities of Antigravity Manager:
1. **Forensic Error Log Remediation Plan (Frozen Execution Guard)**: Formulates deep root cause analysis (RCA), reproduction logs, call-site mappings, and structured future AI implementation instructions for the top 5 error patterns identified across runtime logs (`app.log.2026-10-17`, `crash_stacktrace.log`, `panic.log`):
   - **Pattern 1**: Prompt Queue Scheduler 17,000x Infinite Retry Storm & OS Error 3 (`/nonexistent/repo` test leakage).
   - **Pattern 2**: Supabase 3,440x Transport Error & Fleet Query Storm (NXDOMAIN unpublish on free-tier pause).
   - **Pattern 3**: Broken Pipe Panic (`os error 232`) on CLI output pipe closure (`cmd_tree` / `cmd_prompts_tree`).
   - **Pattern 4**: SQLite Lock Contention during sequence metadata refresh (BEGIN DEFERRED lock upgrade deadlock).
   - **Pattern 5**: Missing `storage.json` / `state.vscdb` on instance sandbox creation and cloning.
   *Crucial Directive: Code modifications are strictly frozen during the current codebase refactor; all remediations are authored as comprehensive specifications and actionable AI instructions.*

2. **GitMap Node Navigation & Multi-Tier Fleet Mesh Architecture**:
   - Bridges the gap between GitMap's working multi-node SSH cluster (`gitmap nodes --json`, 6 registered nodes: `w1`, `w2`, `w3`, `w4`, `u1`, `main`) and AGM's Instance tab.
   - Eliminates the hard block where AGM fails to display remote machines when Supabase is disabled or paused.
   - Defines a 4-Tier Discovery Ladder: Tier 1 Cloud Supabase → Tier 2 Local GitMap SSH Mesh → Tier 3 Decentralized SQLite Fleet Sync → Tier 4 Local Node Telemetry.
   - Specifies GitMap-parity CLI navigation commands (`agm fleet nodes`, `agm fleet ping`, `agm fleet exec`, `agm fleet instances`) and enriched UI Instance Matrix with live status, latency, IP copy, and instance chips.

## Architectural Boundaries & Subtasks
- **Subtask 01 (`Task-01`)**: `01-error-logs-rca-and-remediation-spec`
  - Spec: `02-spec/21-app/fleet-node-navigation-and-error-log-plan/01-error-logs-rca-and-remediation-spec.md`
  - Subtask Plan: `.ai-memory/plans/subtasks/fleet-node-navigation-and-error-log-plan/01-error-logs-remediation-plan.md`
  - Scope: 5-Part forensic RCA, exact log traces, reproduction conditions, prevention invariants, and step-by-step future AI implementation checklist.

- **Subtask 02 (`Task-02`)**: `02-gitmap-node-navigation-and-fleet-spec`
  - Spec: `02-spec/21-app/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-spec.md`
  - Subtask Plan: `.ai-memory/plans/subtasks/fleet-node-navigation-and-error-log-plan/02-gitmap-node-navigation-and-fleet-plan.md`
  - Scope: GitMap node discovery parity, 4-tier discovery ladder, `useFleetMachines` fallback, `agm fleet` navigation CLI suite, and remote machine instance cards in UI.

## Refactor Synchronization & Convergence
- This plan explicitly aligns with the active `refactor/cg-dry-constants-and-nested-if` branch:
  - Consolidates duplicated fleet query logic into a unified resolver.
  - Replaces nested error branches in `useFleetMachines.ts` and `query.rs` with clean fallback guards.
  - Standardizes constant timeouts (`PROBE_TIMEOUT_MS = 1500`, `MAX_RETRIES = 3`, `CIRCUIT_BREAKER_SECS = 300`).
