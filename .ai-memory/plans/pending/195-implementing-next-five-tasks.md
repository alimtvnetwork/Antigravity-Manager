# Parent Plan — 195 Implementing Next Five Tasks

Status: IN_PROGRESS (TAKEN)
Phase: 1 (claimed & assigned to subagents)
Owner: Agent-195-Lead
Specs: `02-spec/21-app/195-implementing-next-five-tasks/spec.md`
E2E Master Plan: `02-spec/21-app/195-implementing-next-five-tasks/e2e_master_plan.md`
Subtasks: `.ai-memory/plans/subtasks/195-implementing-next-five-tasks/`
Backlog Delta: 65 steps -> 60 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `wildcard-model-routing` | `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs` | `IN_PROGRESS` | Enforce model routing wildcard patterns (`gemini-*-flash-*`) and regex prompt sanitizers. |
| 02 | `payload-length-negotiator` | `src-tauri/src/proxy/handlers/common/request_payload_length_negotiator.rs` | `IN_PROGRESS` | Content-Length vs chunked transfer encoding negotiator with bounded memory limits. |
| 03 | `token-budget-decider` | `src-tauri/src/proxy/handlers/common/token_budget_decider.rs` | `IN_PROGRESS` | Allocation controller across max_output_tokens, thinking budgets, and context headroom. |
| 04 | `sse-heartbeat-generator` | `src-tauri/src/proxy/handlers/common/sse_heartbeat_generator.rs` | `IN_PROGRESS` | SSE keep-alive ping frame interval scheduler preventing idle connection aborts. |
| 05 | `quota-exhaustion-predictor` | `src-tauri/src/modules/auto_switcher/quota_exhaustion_predictor.rs` | `IN_PROGRESS` | Token burn rate velocity and quota exhaustion horizon forecaster for preemptive switching. |

## 2. Invariants & Rules
- **Zero-Build & Zero-Test Invariant**: Strictly DO NOT build code or run tests during this phase.
- **Repository-Relative Paths**: Exclusively relative paths.
- **Anti-Collision Commit**: Committed immediately as taken to prevent multi-agent collisions.
