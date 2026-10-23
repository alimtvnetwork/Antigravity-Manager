# Specification 196: Implementing Next Five Tasks (Wildcard Model Routing, Prompt Sanitizer, Pipeline Decoupling, PR Scope Cohesion & Contributor Cost Disclosure)

## 1. Overview & Context
This specification defines the architectural contracts and compliance verifiers for the 5 continuous reliability and release engineering tasks in Sequence 196.
All tasks originate directly from the non-negotiable architectural invariants codified in `AGENTS.md`.

## 2. Five Planned Tasks Breakdown

### Task 1: Wildcard Model Routing & Forward-Compatible Dispatch Guard
- **Module**: `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_wildcard_model_routing.rs`
- **Specification**:
  - Prioritize wildcard patterns (e.g., `gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-*`) to anticipate future model releases rather than exact string matches.
  - Reject static, brittle string matches in routing rules that fail to cover future minor version releases.
  - Structs & Enums: `ModelRouteRule`, `ModelMatchReport`, `ModelRoutingViolation`.
  - Functions: `evaluate_model_match(model_name: &str, rules: &[ModelRouteRule]) -> Result<ModelMatchReport, ModelRoutingViolation>`, `is_wildcard_pattern(pattern: &str) -> bool`.

### Task 2: Agent Prompt Sanitizer & Regex Pattern Matching Guard
- **Module**: `src-tauri/src/modules/release_gate/agent_prompt_sanitizer_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_agent_prompt_sanitizer.rs`
- **Specification**:
  - For agent-client prompt sanitization, prioritize regex-based pattern matching over static keyword replacement.
  - Ensure full coverage of dangerous patterns without stripping pipeline system prompts or genuine user queries.
  - Structs & Enums: `SanitizationRule`, `SanitizationReport`, `PromptSanitizationViolation`.
  - Functions: `sanitize_agent_prompt(prompt: &str, rules: &[SanitizationRule]) -> SanitizationReport`, `validate_sanitization_rule(rule: &SanitizationRule) -> Result<(), PromptSanitizationViolation>`.

### Task 3: Pipeline Protocol Decoupling & Adapter Boundary Guard
- **Module**: `src-tauri/src/modules/release_gate/pipeline_protocol_decoupling_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_pipeline_protocol_decoupling.rs`
- **Specification**:
  - Keep the pipeline strictly decoupled from specific protocols. The four protocols (OpenAI Responses, OpenAI Chat, Anthropic Claude, Gemini) function purely as Gemini adapters.
  - Adapters are restricted to parameter normalization, payload transformation, protocol divergence adaptation, and edge cases unsolvable within the pipeline stage.
  - The pipeline stage uniformly handles converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and sanitization.
  - Prioritize protocol-agnostic, generic fixes within the pipeline rather than localized adapter modifications.
  - Structs & Enums: `PipelineStage`, `AdapterResponsibility`, `DecouplingViolation`, `DecouplingReport`.
  - Functions: `validate_pipeline_decoupling(stage: PipelineStage, responsibility: AdapterResponsibility) -> Result<(), DecouplingViolation>`.

### Task 4: PR Scope Cohesion & Individually Revertable Commits Verifier
- **Module**: `src-tauri/src/modules/release_gate/pr_scope_cohesion_verifier.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_pr_scope_cohesion.rs`
- **Specification**:
  - A PR represents a cohesive collection of fixes or features dedicated to a single problem class.
  - Keep unrelated concerns (such as governance, release tooling, or documentation) in isolated PRs.
  - Each commit must represent an independent, self-contained functional unit that is individually revertable.
  - Structs & Enums: `ProblemClass`, `CommitScope`, `PrScopeReport`, `PrScopeViolation`.
  - Functions: `validate_pr_scope_cohesion(problem_class: &ProblemClass, touched_modules: &[String]) -> PrScopeReport`.

### Task 5: Contributor Cost Disclosure & Attribution Linter
- **Module**: `src-tauri/src/modules/release_gate/contributor_cost_disclosure_linter.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_contributor_cost_disclosure.rs`
- **Specification**:
  - Preserve authorship by preferring contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs.
  - Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements.
  - Own tooling and documentation gaps directly rather than attributing downstream frictions to contributors.
  - Accompany every rejection with a concrete file-by-file accept/drop breakdown and an actionable path forward.
  - Structs & Enums: `CostDisclosureReport`, `AttributionCheck`, `ContributorCostViolation`.
  - Functions: `audit_contributor_pr_metadata(description: &str, coauthors: &[String], is_proxy_merge: bool) -> CostDisclosureReport`.

## 3. Invariants & Rules Upheld
- Zero builds and zero tests during implementation phase (Rule 1).
- Strictly repository-relative Git paths.
- Search exclusively via GitMap.
- Tree Swarm concurrency executed via A=3 subagents in disjoint file boxes.
