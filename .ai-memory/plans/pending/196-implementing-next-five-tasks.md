# Parent Plan — 196 Implementing Next Five Tasks

Status: IN PROGRESS (TAKEN)
Phase: 1 (claim committed & spec authored)
Owner: Agent-196-Lead
Specs: `02-spec/21-app/196-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/196-implementing-next-five-tasks/`
Backlog Delta: 60 steps -> 55 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `wildcard-model-routing` | `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce wildcard patterns (`gemini-*-flash-*`) over brittle static string matches to anticipate future AI model releases. |
| 02 | `agent-prompt-sanitizer` | `src-tauri/src/modules/release_gate/agent_prompt_sanitizer_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce regex-based prompt sanitization over static keyword replacement without stripping pipeline system prompts or user queries. |
| 03 | `pipeline-protocol-decoupling` | `src-tauri/src/modules/release_gate/pipeline_protocol_decoupling_guard.rs` | `IN PROGRESS (TAKEN)` | Enforce pipeline decoupling where adapters only normalize payloads and pipeline uniformly handles thinking blocks and prefix stability. |
| 04 | `pr-scope-cohesion` | `src-tauri/src/modules/release_gate/pr_scope_cohesion_verifier.rs` | `IN PROGRESS (TAKEN)` | Validate that PR changesets remain confined to a single problem class and all commits are self-contained and individually revertable. |
| 05 | `contributor-cost-disclosure` | `src-tauri/src/modules/release_gate/contributor_cost_disclosure_linter.rs` | `IN PROGRESS (TAKEN)` | Enforce contributor cost disclosure (affected existing behaviors, unverified paths) and Co-authored-by trailer compliance. |

## 2. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant.
- **Anti-Collision**: Initial `IN PROGRESS (TAKEN)` commitment registered and synchronized upstream so parallel AI agents do not duplicate.
