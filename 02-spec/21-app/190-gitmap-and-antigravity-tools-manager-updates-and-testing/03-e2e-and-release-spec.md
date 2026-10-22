# 03 — End-to-End Testing + Release Spec

Slug: 190-gitmap-and-antigravity-tools-manager-updates-and-testing
Scope: Antigravity-Manager + gitmap-v28. Runs ONLY after specs 01 and 02 are
fully implemented (user instruction: no building/testing during coding).

## Build phase (end only)

- AGM: pre-flight per repo rules — `cargo fmt -- --check`,
  `cargo clippy --all-targets --all-features` (src-tauri), `npm run build`.
  This VM has no Rust toolchain: pre-flight runs where Rust exists, or CI is
  the gate and the release workflow's rustfmt check is the backstop.
- gitmap-v28: `go build ./...` + `go vet` with the toolchain at
  `~/workspace/tools/go/bin`.

## E2E — AGM (the user's priority order)

1. **Account switching**: manual GUI switch and CLI `agm instances switch`
   on sandbox instances (never the caller's live IDE; transient
   `agm instances test-create` / `test-clean` sandboxes).
2. **Prompt backup → quota auto-switch → restore → rerun** (spec 01's
   acceptance scenario): start a long prompt, exhaust/force quota switch,
   verify the incomplete prompt is swept, restored, and re-dispatched FIFO
   on the target; verify restored counts reported are real.
3. **Rerun auto-pick**: rerun fires with `randomized` by default, persisted
   option, from both the detail header and the context menu.
4. **Auto-switch strictness**: candidate scoring, cooldowns, and the
   post-switch 10s resume surviving an app restart (RC-7).
5. CLI command surface: `agm prompts` (backup, resend-queued), instances,
   paths — GUI/CLI parity on the same backup DB (RC-8).

## E2E — GitMap

- Command surface sweep: help for every top-level command; the commands the
  CG waves touched (dispatch path in `cli/cmd/root.go`) get behavioral
  checks (exit codes incl. typed cliexit codes from plan 301).
- `gitmap pe -t` clean on the final tree; linters from spec 02 all clean.
- Plans/status commands reflect the drained pending folder.

## Release phase (user instruction: release both, CI/CD green)

- Both repos: fix-forward until `gitmap pe -t` is green on main.
- AGM: `npm run bump` (minor), changelog attribution strictly
  `(Thanks to @aukgit)`, README.md + README_EN.md changelog sync, tag from
  main only, watch the release workflow to green.
- gitmap-v28: release-branch lifecycle per its release prompt — bump via
  `03-ai-scripts/37-bump-version.py` (minor), changelog entry with real
  bullets, annotated tag, merge to main, push all three refs, watch the
  Release workflow + installer-smoke jobs to green.
- Relative paths only in changelogs, release notes, and release pages.
- Never cut a release while another is in flight; verify none is in flight
  before tagging either repo.
