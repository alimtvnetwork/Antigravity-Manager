# 240 - CI/CD Fix and Release - Component Spec

Date: 31-Oct-26

## Component 1 - ProfileCandidate (scoring.rs)

- File: src-tauri/src/modules/auto_switcher/scoring.rs
- Change: add one public field to the struct, directly after
  `quota_percent`:
  `pub rolling_4h_quota: f64`
- Comment on the field (WHY): the live 4-hour rolling-window quota
  percent for the candidate, used by manual rotation to classify the
  selection tier. It mirrors `quota_percent` at construction time;
  the verified path overwrites both with the freshly measured 4-hour
  quota, so the two stay in sync by construction.
- No change to scoring functions, thresholds, or sort order.

## Component 2 - Candidate construction (selection.rs)

- File: src-tauri/src/modules/auto_switcher/selection.rs
- Seven construction sites, all receiving
  `rolling_4h_quota: <same value as quota_percent at that site>`:
  1. Bound-instance primary pool (quota from
     `calculate_candidate_quota`, gated at >= 100.0).
  2. Unbound-account primary pool (same gate).
  3. Fallback pool (quota > threshold).
  4. Emergency fallback pool (quota > 5.0).
  5. Verified 100% candidate return (`fresh_4h_quota`).
  6. Best fallback staging (`fresh_4h_quota`).
  7. Best emergency fallback staging (`fresh_4h_quota`).
- No other field, condition, or log line changes in this file.

## Component 3 - Prompt dispatch status (agy.rs)

- File: src-tauri/src/modules/repo_db/agy.rs
- Change: exactly one token - `let active_prompt` becomes
  `let mut active_prompt` at the `ActivePrompt` construction in
  `enqueue_prompt_for_instance`.
- WHY: the binding is mutated later in the same function after a
  successful CDP injection (status to `dispatched`, `updated_at`
  refreshed) and re-persisted to prevent double-dispatch by the
  background scheduler. The mutation is pre-existing intended
  behaviour; the missing `mut` is the defect.
- FIFO invariant untouched: no query ordering change in this file.

## Component 4 - Version sync and release (repo root + scripts)

- Files touched by `npm run bump minor`: package.json,
  package-lock.json, version.json, src-tauri/Cargo.toml,
  src-tauri/tauri.conf.json, src-tauri/Cargo.lock, changelog.md,
  readme.md, readme_en.md (stable-only sections), and any other file
  the bump script lists.
- Expected result: every manifest reads 4.282.0; changelog gains a
  `v4.282.0` heading dated 31-Oct-26 describing this CI fix and
  release; the release tag is `v4.282.0` on main.
- No workflow file (.github/workflows/) is modified in this run.
