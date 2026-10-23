# Specification 196: Implementing Next Five Tasks (Wildcard Model Routing, Prompt Sanitizer, Pipeline Decoupling, PR Scope Cohesion, Contributor Cost Disclosure)

## 1. Overview & Context
This specification defines the architectural contracts, linting rules, and verification test suites for 5 critical governance, pipeline architecture, and release gate standards defined in `AGENTS.md`:
1. **Wildcard Model Routing Guard** (`Imp-610`): Wildcard glob and regex patterns over brittle exact model string comparisons.
2. **Agent-Client Prompt Sanitization Regex Guard** (`Imp-611`): Regex pattern matching over static keyword replacement without stripping system prompts or user queries.
3. **Pipeline Decoupling & Protocol Normalization Guard** (`Imp-612`): Keeping adapters strictly restricted to parameter normalization while pipeline uniformly handles thinking blocks, budgeting, and sanitization.
4. **Single Problem Scope & PR Cohesion Verifier** (`Imp-613`): Validating that PRs represent a cohesive single problem class and each commit is self-contained and individually revertable.
5. **Contributor Cost Disclosure & Attribution Linter** (`Imp-614`): Enforcing disclosure of affected existing behaviors and unverified paths, along with `Co-authored-by:` attribution compliance.

---

## 2. Five Planned Tasks Breakdown

### Task 1: Wildcard Model Routing & Forward-Compatible Dispatch Guard (`Imp-610`)
- **Module**: `src-tauri/src/modules/release_gate/wildcard_model_routing_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_wildcard_model_routing.rs`
- **Specification**:
  - Enforces `AGENTS.md` rule: "Prioritize wildcard patterns (e.g., `gemini-*-flash-*`, `claude-3-5-*`, `gpt-4o-*`) to anticipate future model releases rather than exact string matches".
  - Structs & Enums: `RoutingPatternType`, `ModelRouteRule`, `ModelRoutingViolation`, `ModelMatchReport`.
  - Functions: `evaluate_model_match(model_name: &str, rules: &[ModelRouteRule]) -> Result<ModelMatchReport, ModelRoutingViolation>`, `lint_routing_code(source: &str) -> Vec<ModelRoutingViolation>`, `is_wildcard_pattern(pattern: &str) -> bool`.

### Task 2: Agent-Client Prompt Sanitization Regex Guard (`Imp-611`)
- **Module**: `src-tauri/src/modules/release_gate/agent_prompt_sanitizer_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_agent_prompt_sanitizer.rs`
- **Specification**:
  - Enforces `AGENTS.md` rule: "For agent-client prompt sanitization, prioritize regex-based pattern matching over static keyword replacement, ensuring full coverage without stripping pipeline system prompts or user queries."
  - Sanitization must identify dangerous prompt injection patterns, leaky system directives, or unauthorized role spoofing via targeted regex rules without dropping valid user queries or core pipeline instructions.
  - Structs & Enums: `SanitizationMethod`, `SanitizerRule`, `PromptInspectionReport`, `SanitizationViolation`.
  - Functions: `sanitize_prompt_text(input: &str, is_system_prompt: bool, is_user_query: bool, rules: &[SanitizerRule]) -> Result<PromptInspectionReport, SanitizationViolation>`, `lint_sanitization_implementation(source_code: &str) -> Vec<SanitizationViolation>`.

### Task 3: Pipeline Decoupling & Protocol Normalization Guard (`Imp-612`)
- **Module**: `src-tauri/src/modules/release_gate/pipeline_protocol_decoupling_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_pipeline_protocol_decoupling.rs`
- **Specification**:
  - Enforces `AGENTS.md` rule: "Pipeline First: Keep the pipeline strictly decoupled from specific protocols. The four protocols function purely as Gemini adapters. Adapters are restricted to parameter normalization, payload transformation, protocol divergence adaptation, and edge cases unsolvable within the pipeline stage. The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."
  - Validates adapter responsibility boundaries and flags protocol leaks into pipeline or pipeline-level responsibilities implemented inside adapters.
  - Structs & Enums: `ComponentRole`, `ResponsibilityDomain`, `BoundaryValidationReport`, `CouplingViolation`.
  - Functions: `validate_component_responsibilities(component: &ComponentRole, domains: &[ResponsibilityDomain]) -> BoundaryValidationReport`, `lint_adapter_pipeline_boundaries(source_path: &str, code: &str) -> Vec<CouplingViolation>`.

### Task 4: Single Problem Scope & PR Cohesion Verifier (`Imp-613`)
- **Module**: `src-tauri/src/modules/release_gate/pr_scope_cohesion_verifier.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_pr_scope_cohesion.rs`
- **Specification**:
  - Enforces `AGENTS.md` rule: "A PR represents a cohesive collection of fixes or features dedicated to a single problem class. Keep unrelated concerns (such as governance, release tooling, or documentation) in isolated PRs. Each commit must represent an independent, self-contained functional unit that is individually revertable."
  - Categorizes changed files across problem domains (`CoreEngine`, `ProxyPipeline`, `ReleaseGovernance`, `Documentation`, `UiComponents`, `DbOrm`) and detects tangled cross-domain diffs.
  - Verifies template alignment (`.github/pull_request_template.md`) requiring problem classification, behavior alterations, unverified paths, and rollback strategy.
  - Structs & Enums: `ProblemDomain`, `PrCommitUnit`, `PrScopeReport`, `ScopeViolation`.
  - Functions: `evaluate_pr_scope(commits: &[PrCommitUnit], pr_body: &str) -> PrScopeReport`.

### Task 5: Contributor Cost Disclosure & Attribution Linter (`Imp-614`)
- **Module**: `src-tauri/src/modules/release_gate/contributor_cost_disclosure_linter.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_contributor_cost_disclosure.rs`
- **Specification**:
  - Enforces `AGENTS.md` rule: "Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs. Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements. Accompany every rejection with a concrete file-by-file accept/drop breakdown and an actionable path forward."
  - Structs & Enums: `CostDisclosureReport`, `AttributionTrailer`, `DisclosureViolation`.
  - Functions: `verify_cost_disclosure(pr_body: &str) -> CostDisclosureReport`, `verify_attribution_trailers(commit_message: &str, expected_contributors: &[&str]) -> Result<Vec<AttributionTrailer>, DisclosureViolation>`, `lint_rejection_feedback(feedback_text: &str) -> Result<(), DisclosureViolation>`.

---

## 3. Invariants & Rules Upheld
- **Zero-Build & Zero-Test Invariant**: Strictly preserved; zero compilers invoked.
- **Repository-Relative Paths**: 100% compliant across all specs, tests, and code.
- **Search Exclusively via GitMap**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Anti-Collision Commit**: Enqueued and committed upstream before implementation.
