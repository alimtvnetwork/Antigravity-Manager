# 240 - CI/CD Fix and Release for Multiple Packages - Architecture Spec

Date: 31-Oct-26
Branch: main
Starting point: commit 3659500cf (origin/main at pull time)
Current version: 4.281.2 (version.json and package.json); Cargo.toml and
tauri.conf.json lag at 4.281.0 (version drift, fixed by the bump step below).

## 1. Problem statement

CI on main is red, so the Release workflow gate refuses every release.
The last CI run on the main tip is run 41195657081
(https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/41195657081),
conclusion failure, for commit 3659500cf. The Build Frontend job passes;
every Rust job fails. The Release run 41195653954 for the same commit
failed at "Verify Branch & Release Type Compatibility" with the gate
message: CI for commit 3659500cf concluded 'failure', not 'success' -
release refused. That gate failure is a consequence of the CI failure,
not a separate defect, and no workflow file is changed in this run.

"Multiple packages" in this task means the versioned manifests that must
move together in this repo, not separate published packages: package.json,
package-lock.json, version.json, src-tauri/Cargo.toml,
src-tauri/tauri.conf.json, src-tauri/Cargo.lock, plus changelog.md and
the readme changelog sections. The repo's own bump script
(scripts/bump-version.mjs, via `npm run bump`) is the single sync
mechanism for those files.

## 2. Root causes (four compile errors, three roots)

Note: CI reported three errors because the fourth (2c) was masked by
2a - with the field missing, the compiler could not also report the
use-after-move on that same expression. It surfaced on local clippy
once the field existed, and is fixed in the same run.

### 2a. E0609 - missing field on ProfileCandidate

- Location: src-tauri/src/modules/auto_switcher/manual.rs, line 188:
  `candidate.rolling_4h_quota >= 99.5` classifies the rotation result as
  "Primary Tier" versus "Fallback Tier".
- The struct src-tauri/src/modules/auto_switcher/scoring.rs
  `ProfileCandidate` has only `instance_id`, `account_id`, `email`,
  `quota_percent`, `score`. It has no `rolling_4h_quota` field.
- The similarly named `rolling_4h_quota_percent` in
  src-tauri/src/modules/auto_switcher/fast_forward_target_resolver.rs
  belongs to a different, dead-code resolver struct and is not usable
  here; do not wire manual.rs to it.

### 2b. E0594 (x2) - immutable binding mutated

- Location: src-tauri/src/modules/repo_db/agy.rs, lines 1138-1139, inside
  `enqueue_prompt_for_instance`. After a successful live DevTools CDP
  injection into a running IDE, the code updates the queued prompt to
  `status = "dispatched"` and refreshes `updated_at`, then persists it
  with `save_or_requeue_prompt` so the background scheduler does not
  double-dispatch it.
- The binding at line 1046 is `let active_prompt = ActivePrompt { ... }`
  (immutable), so both assignments are rejected. The mutation itself is
  correct and intended; only the binding needs `mut`.

### 2c. E0382 - use of moved value (masked by 2a)

- Location: src-tauri/src/modules/auto_switcher/manual.rs, line 188
  (now the captured variable). `execute_profile_rotation_with_context`
  takes `candidate` by value (line 181), so the later read of
  `candidate.rolling_4h_quota` is a use-after-move.
- Fix: capture `candidate.rolling_4h_quota` into a local `f64` before
  the rotation call and classify the tier from that copy. The value is
  identical; no decision, message, or ordering changes.

## 3. Fix design (minimal, root-cause, no behaviour change)

1. Add `pub rolling_4h_quota: f64` to `ProfileCandidate` in scoring.rs,
   with a comment stating WHY: it carries the live 4-hour rolling-window
   quota percent used for tier classification in manual rotation.
2. Populate it at every construction site in
   src-tauri/src/modules/auto_switcher/selection.rs (seven sites: four
   in `select_candidate_profiles`, three in the verified/fallback path
   of the candidate evaluator) with the same value already assigned to
   `quota_percent` at that site. In the verified path that value is the
   freshly measured `fresh_4h_quota`, so the tier label in manual.rs
   keeps exactly the meaning its author intended. No scoring, sorting,
   threshold, or pool logic changes.
3. Change line 1046 of agy.rs to `let mut active_prompt = ...`. No other
   change in that function; the dispatched-status write and its
   best-effort justification comment stay untouched.

Explicitly out of scope: the 1234 clippy warnings (warnings, not errors;
CI fails on errors only), the pre-existing beta-branch v5.x tag line,
the v5.0.12 tag pointing at a non-main commit, workflow files
(.github/workflows/ - untouched, so no workflow-scope push blocker is
triggered), and any UI or protocol change.

## 4. Verification contract

- Local pre-flight per repo house rules: `cargo fmt -- --check` and
  `cargo clippy --all-targets --all-features` under src-tauri (clippy
  already includes compilation; no separate full build locally, and the
  Tauri GUI cannot launch on this headless VM).
- Remote: the CI run on the fix commit must be fully green (Build
  Frontend + Check Rust Code on all platforms + Build Tauri App), and
  the Release workflow run on the release tag must be green with the
  release published (not draft) and assets present.
- Version rule: tag and changelog heading must match character-for-character
  (including the `v` prefix), per repo release discipline, and the
  release must be a stable release on main (never a pre-release tag).

## 5. Release ceremony

- Bump: `npm run bump minor` (4.281.2 -> 4.282.0), which also repairs
  the Cargo.toml / tauri.conf.json drift to 4.281.0. Do not use the
  bump script's `--commit` flag; the commit is made with the gitmap
  commit suite so identity and push stay under the configured bot
  identity.
- Commit and push the fix plus the bump together only after the fix is
  verified locally; then tag `v4.282.0` and push the tag, which triggers
  the Release workflow. Confirm the tag via `git ls-remote` and the
  release via the GitHub API before reporting success.
- Attribution in changelog entries: strictly `@aukgit` only, per repo
  house rules; no other handles.
