# Specification 200: Implementing Next Five Tasks (Thinking Cache Invalidation, Readme Changelog Sync, Sole Contributor Attribution, Tag Heading Matcher & Prefix Stability)

## 1. Overview & Context
This specification details the architecture, compliance criteria, and verification suites for Sequence 200 (Steps 216–220) in Antigravity-Manager.
All 5 tasks implement non-negotiable architectural mandates defined directly in `AGENTS.md`:
1. **Thinking Cache Invalidation Controller (`Imp-630`)**: Manages one-time prompt dialogs and version upgrade triggers for `SUGGESTION_DELETE_THINKING_STORE` and `SUGGESTION_TARGET_VERSION` in `gui_config.json`.
2. **README Changelog Synchronizer (`Imp-631`)**: Validates that stable releases (`vX.Y.Z`) synchronize release summaries across both `README.md` (under "## 📝 更新日志") and `README_EN.md` (under "## 📝 Changelog"), while beta releases remain changelog-only.
3. **Sole Contributor Attribution Guard (`Imp-632`)**: Audits Git changelog and release notes to enforce strictly attribution to `@aukgit` (`(Thanks to @aukgit)`) without extraneous GitHub handles (`@...`).
4. **Tag Changelog Heading Matcher (`Imp-633`)**: Enforces exact character-for-character matching between Git tag names (including `v` prefix) and `CHANGELOG.md` headings, with branch gating (`main` for stable, `beta` for pre-release).
5. **Prefix Stability Pipeline Guard (`Imp-634`)**: Ensures immutable prefix stability across multi-turn LLM dialogs within the pipeline stage to maximize upstream KV prompt caching and prevent cache thrashing.

---

## 2. Five Planned Tasks Breakdown

### Task 1: Thinking Cache Invalidation Controller (`Imp-630`)
- **Target Module**: `src-tauri/src/modules/release_gate/thinking_cache_invalidation_controller.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_thinking_cache_invalidation_controller.rs`
- **Requirements**:
  - Enforces `AGENTS.md` Thinking Cache Invalidation Control:
    - Routine releases: `suggestion_delete_thinking_store = false`.
    - Architecture/schema refactor releases: `suggestion_delete_thinking_store = true`, `suggestion_target_version = "<version>"`.
  - Determines whether an upgrade prompt should be surfaced to the user based on previous action state persisted in `gui_config.json`.
  - Prevents repeated prompts when target version was already acknowledged or dismissed.
  - Structs & Enums: `ThinkingCacheConfig`, `UserCacheActionState`, `CachePromptDecision`, `CachePromptAction`.

### Task 2: README Changelog Synchronizer (`Imp-631`)
- **Target Module**: `src-tauri/src/modules/release_gate/readme_changelog_synchronizer.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_readme_changelog_synchronizer.rs`
- **Requirements**:
  - Enforces `AGENTS.md` Synchronize README Changelog (同步首页更新日志):
    - Stable releases (`vX.Y.Z`): Release summary MUST exist and match in `README.md` ("## 📝 更新日志") and `README_EN.md` ("## 📝 Changelog").
    - Pre-releases / Beta (`vX.Y.Z-beta.N`): Remain exclusively in `CHANGELOG.md` and MUST NOT update the root README summaries.
  - Structs & Enums: `ReleaseTagType`, `ReadmeSyncReport`, `ReadmeSyncViolation`.

### Task 3: Sole Contributor Attribution Guard (`Imp-632`)
- **Target Module**: `src-tauri/src/modules/release_gate/sole_contributor_attribution_guard.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_sole_contributor_attribution_guard.rs`
- **Requirements**:
  - Enforces `AGENTS.md` Documentation & Attribution Invariant:
    - Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`.
    - Intercepts any other GitHub user handles (`@...`) in release changelogs or release notes.
  - Structs & Enums: `AttributionAuditReport`, `AttributionViolation`.

### Task 4: Tag Changelog Heading Matcher (`Imp-633`)
- **Target Module**: `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher.rs`
- **Test Suite**: `src-tauri/src/modules/release_gate/tests_tag_changelog_heading_matcher.rs`
- **Requirements**:
  - Enforces `AGENTS.md` Commit, Tag & Push Invariant:
    - Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix).
    - Stable tags (`vX.Y.Z`) must target `main`. Pre-release tags (`vX.Y.Z-beta.N`) must target `beta`. Cross-branch misplacement is strictly intercepted.
  - Structs & Enums: `TagPlacementReport`, `TagMatchViolation`.

### Task 5: Prefix Stability Pipeline Guard (`Imp-634`)
- **Target Module**: `src-tauri/src/proxy/handlers/common/prefix_stability_pipeline_guard.rs`
- **Test Suite**: `src-tauri/src/proxy/handlers/common/tests_prefix_stability_pipeline_guard.rs`
- **Requirements**:
  - Enforces `AGENTS.md` Pipeline First & Prefix Stability:
    - Verifies that conversation turn history retains strict prefix stability (system instructions, user turns, and tool calls maintain deterministic ordering and content prefix).
    - Flags mutated earlier turns that would bust KV prompt caches upstream.
  - Structs & Enums: `DialogTurn`, `PrefixStabilityReport`, `PrefixStabilityViolation`.

---

## 3. Non-Negotiable Invariants
- **Zero-Build & Zero-Test Invariant (R1)**: No compilers or test runners during implementation.
- **Worker Git Ban (R7)**: Subagents never execute git commands.
- **Strictly Relative Paths**: All paths strictly relative (`02-spec/...`, `.ai-memory/...`, `src-tauri/...`).
- **Exclusively GitMap for Search**: Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- **Zero Secrets**: No secret patterns in source or test code.
