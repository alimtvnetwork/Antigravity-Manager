# Subtask 08 — Build + E2E + release both + CI green

Repos: Antigravity-Manager + gitmap-v28. Spec: 03-e2e-and-release-spec.md.
Status: BLOCKED until subtasks 01-07 complete. Owner: Lead.

## Steps
1. Builds (first build/test moment of the whole task): gitmap `go build ./...` + `go vet` (toolchain at ~/workspace/tools/go/bin); AGM pre-flight where Rust exists (cargo fmt --check, clippy) + `npm run build`; fix-forward anything red.
2. gitmap E2E: command-surface sweep (help + exit codes incl. typed cliexit), linters clean (boolean, newline styling, enum), FastGate.
3. AGM E2E per spec 03 scenarios 1-5, sandbox instances only (test-create/test-clean), never the caller's live IDE. The backup→quota-switch→restore→rerun scenario is the acceptance gate for spec 01.
4. Commit each repo's work via GitMap (hyphen messages), push, watch `gitmap pe -t` to green on both mains; fix-forward until green.
5. Releases: AGM minor bump via `npm run bump` (attribution `(Thanks to @aukgit)`, README + README_EN sync, tag from main, watch release workflow). gitmap minor bump via release-branch lifecycle + `03-ai-scripts/37-bump-version.py`, annotated tag, merge, push, watch Release + installer-smoke jobs.
6. Verify no release was in flight before tagging; verify release pages contain relative paths only and real changelog content.
7. Close the parent plan: move to completed with evidence links (CI runs, tags, E2E results).

## Acceptance
- `gitmap pe -t` green on both repos post-release.
- Spec 01 acceptance scenario passes live.
- Both release pages published with correct versions.
