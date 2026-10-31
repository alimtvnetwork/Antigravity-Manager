# Subtask 04 - Wave-2 E0500 fix on moved tip

Date: 31-Oct-26
Status: in progress
Owned files (exclusive):
- src-tauri/src/modules/repo_db/tree_node_conv.rs
- .ai-memory/plans/subtasks/240-cicd-fix-and-release-for-multiple-packages/04-wave2-e0500-fix.md
Work:
- CI run 41202605011 on tip 608abea60 failed with the single error
  E0500 at tree_node_conv.rs:856 (double mutable borrow via
  iter_mut().find() chained with or_else(first_mut())).
- Fix: index-first selection (position under a shared borrow, fallback
  index 0, single &mut index). Semantics unchanged.
- Local fmt check passed (exit 0). Local full clippy/check cannot run
  on this VM (SIGKILL out-of-memory, zero E-code errors) - remote CI
  is the arbiter, as recorded in the spec.
Evidence: post-push CI run ID/URL must be fully green before the bump.
