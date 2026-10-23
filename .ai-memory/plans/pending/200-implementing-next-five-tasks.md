# Parent Plan — 200 Implementing Next Five Tasks (Steps 216–220)

Status: IN PROGRESS (TAKEN)
Phase: 1 (claimed & reserved)
Owner: Agent-200-Lead
Specs: `02-spec/21-app/200-implementing-next-five-tasks/spec.md`
Subtasks: `.ai-memory/plans/subtasks/200-implementing-next-five-tasks/`
Backlog Delta: 40 steps -> 35 steps remaining (-5 steps claimed)

## 1. Selected Five Tasks (Claimed & In-Progress)

| Task # | Slug | Module Target | Status | Implementation Details |
|---|---|---|---|---|
| 01 | `thinking-cache-invalidation-controller` | `src-tauri/src/modules/release_gate/thinking_cache_invalidation_controller.rs` | `IN_PROGRESS` | Manages upgrade prompt evaluation and persistent state for thinking cache cleanup. |
| 02 | `readme-changelog-synchronizer` | `src-tauri/src/modules/release_gate/readme_changelog_synchronizer.rs` | `IN_PROGRESS` | Verifies full synchronization of release summaries across `README.md` and `README_EN.md` for stable releases. |
| 03 | `sole-contributor-attribution-guard` | `src-tauri/src/modules/release_gate/sole_contributor_attribution_guard.rs` | `IN_PROGRESS` | Audits changelogs and release notes to enforce strictly attribution to `@aukgit` and intercept extraneous `@...` handles. |
| 04 | `tag-changelog-heading-matcher` | `src-tauri/src/modules/release_gate/tag_changelog_heading_matcher.rs` | `IN_PROGRESS` | Enforces exact character-for-character matching between Git tag names and changelog headings, with branch gating. |
| 05 | `prefix-stability-pipeline-guard` | `src-tauri/src/proxy/handlers/common/prefix_stability_pipeline_guard.rs` | `IN_PROGRESS` | Enforces immutable prefix stability across multi-turn LLM dialogs to preserve upstream KV prompt caching. |

## 2. Invariants & Rules
- Zero-Build & Zero-Test Invariant (R1).
- Strictly relative Git paths.
- Search exclusively via GitMap. Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
- Anti-collision commit before worker dispatch.
