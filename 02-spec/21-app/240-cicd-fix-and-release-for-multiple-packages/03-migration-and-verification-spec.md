# 240 - CI/CD Fix and Release - Migration and Verification Spec

Date: 31-Oct-26

## Migration

- No data migration, no schema change, no config change, no API change.
- The new struct field is populated at every construction site, so no
  caller can construct a partial `ProfileCandidate`. If a future site
  is added without the field, compilation fails loudly at that site -
  that is the intended guard, not a gap.
- The `mut` change has no persisted-format effect: the stored prompt
  row already carried the `dispatched` status on the success path in
  every build where this code compiled (i.e. before the regression
  that introduced the immutable binding); this run restores that path.

## Regression note (what introduced the red CI)

- The 4.281.x sync commits touched manual rotation and prompt enqueue
  paths (cluster-wide exclusion, CDP direct dispatch) and left the two
  defects above: a referenced-but-never-declared field, and a binding
  whose later mutation was not declared mutable. Both are compile
  errors, so no released binary from the red tip could exist; the fix
  restores compilability without changing any runtime decision.

## Verification steps (in order)

1. Local: `cargo fmt -- --check` under src-tauri - PASSED (exit 0,
   no diff) on 31-Oct-26.
2. Local: `cargo clippy --all-targets --all-features` and
   `cargo check --lib --all-features` could NOT complete on this VM:
   every run ended with the rustc/clippy-driver process killed by
   signal 9 (SIGKILL, out-of-memory on the 7.7 GiB VM, even at
   CARGO_BUILD_JOBS=1), with ZERO E-code errors in any log. This is
   an environment limit, not a code result: the touched crate is too
   large for this VM to compile. The fix is type-correct by
   construction (field added and populated at all seven sites, f64
   copy captured before the move, single `mut` binding), fmt parses
   clean, and the full clippy gate is therefore delegated to remote
   CI, which is the arbiter for this run. Warning counts seen in the
   partial local runs (~1068-1273, pre-existing unused
   variables/imports) do not fail CI and are not fixed here.
3. Push the fix commit; record the new CI run ID and URL.
4. Confirm that CI run is fully green (all jobs, including the
   experimental ubuntu-26.04 entry which is continue-on-error and
   therefore cannot by itself fail the run - the gating jobs are the
   ubuntu-24.04 and windows-2022 entries).
5. Bump minor to 4.282.0 with the repo bump script, commit, push, tag
   `v4.282.0`, push the tag.
6. Confirm the tag on the remote with `git ls-remote`, confirm the
   Release workflow run is green, confirm the GitHub release is
   published (not draft, not prerelease) and record its asset count.
7. Final check: working tree clean, local main equals origin/main
   including the tag commit, latest CI run on the release tip green.

## Known non-blockers recorded honestly

- Clippy warning count (~1234, mostly unused variables/imports) is
  pre-existing and untouched; CI does not fail on warnings.
- The beta-branch v5.x pre-release line and the v5.0.12 tag (which
  points at a commit that is not the main tip) are separate lines of
  work and are not touched, renamed, or deleted in this run.
- If any future fix in this area requires a .github/workflows/ change,
  that change must be committed locally and reported as blocked on the
  VM token lacking `workflow` scope (unblock: user runs
  `gh auth refresh -s workflow` interactively, then push). This run
  needs no such change.
