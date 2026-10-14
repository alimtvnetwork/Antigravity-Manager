# Task-02: RCA batch 2 — gitmap file-split incidents

## Status
DONE

## Objective
Write detailed root-cause analyses for the four gitmap file-split incidents
into `.ai-memory/rca/`.

## Incidents to document

### 04-gitmap-split-imports.md
- Symptom: CI `Build (...)` failed on all 6 platform jobs:
  `cmdagent/agent_claim_collision.go:207:15: undefined: os`,
  `cmdinstall/install_archive_magic.go:55:15: undefined: apperror`,
  `:59:16: undefined: io`, plus `install_archive_download.go:4:2: "bytes"
  imported and not used`.
- Root cause: commit `83939b8ae` split three files to satisfy the 300-line
  cap but did not carry all imports to the new files (and left a now-unused
  import in the old one).
- How found: `gitmap pe` → `gh run view --job <build-job> --log-failed`.
- Fix: added `"os"` to `cli/cmdagent/agent_claim_collision.go`; added `"io"`
  and `"github.com/alimtvnetwork/gitmap-v28/cli/apperror"` (unaliased, per
  `cmdinstall` convention) to `cli/cmdinstall/install_archive_magic.go`;
  removed unused `"bytes"` from `cli/cmdinstall/install_archive_download.go`;
  `gofmt -w`; verified `go build ./...`, `go vet`, `gofmt -l` clean.
  Commit `31806e142`.
- Prevention: after any file split, `go build ./...` + `go vet ./...` +
  `gofmt -l` before push; check both sides of the split for import drift.

### 05-gitmap-split-marker.md
- Symptom: `TestTopLevelCmdRegistryMatchesAST` failed:
  "topLevelCmds() registry has 140 entry/entries with no matching AST
  declaration".
- Root cause: the split moved const blocks to
  `cli/constants/constants_commands_extra.go` without the
  `// gitmap:cmd top-level` marker the AST collector requires.
- How found: CI log → read `cmd_constants_parity_test.go` → counted markers
  per file (17 vs 0).
- Fix: added `// gitmap:cmd top-level` above the const block in
  `constants_commands_extra.go`; verified with
  `go test ./constants/ -run TestTopLevelCmdRegistryMatchesAST`. Commit
  `55120fc07`.
- Prevention: file splits must preserve magic marker comments; run the
  affected package tests locally before push.

### 06-gitmap-coverage-test-locator.md
- Symptom: `TestEveryCmdIDHasHelpFile` failed:
  "parsed zero Cmd* constants — locator broke?" in `cli/helpdoc`.
- Root cause: `coverage_test.go:locateCLIConstantsFile()` hardcoded
  `constants/constants_cli.go`, which the split reduced to a 6-line stub.
- How found: downloaded CI job log via API; verified old file had 392 Cmd*
  lines and the two new files hold the exact union (`diff` empty).
- Fix: replaced single-file locator with `locateCLIConstantsFiles()`
  returning `constants_commands.go` + `constants_commands_extra.go`;
  `parsePrimaryCmdIDs()` iterates both. Verified
  `go test ./helpdoc/ -run TestEveryCmdIDHasHelpFile` passes.
- Prevention: tests that parse source files must not hardcode paths that
  refactors can invalidate; prefer globs or explicit file lists with a
  comment naming the split.

### 07-gitmap-windows-readonly-skip.md
- Symptom: `TestEffectiveDataDir_ReadOnlyFallback` failed on
  `windows-latest` only: expected fallback, got the read-only dir.
- Root cause: POSIX mode `0555` is not enforceable on Windows — the dir
  remains writable so the fallback never triggers.
- How found: CI log; test is inherently platform-dependent.
- Fix (by another stream): `t.Skip` on `runtime.GOOS == "windows"` with
  reason comment. Commit `afb75a80e`.
- Prevention: any test relying on POSIX permission bits must skip on
  Windows with an explanatory comment.

## Format per RCA
Symptom (CI log excerpt) → Root cause → How found (exact commands) → Fix
(file:line, old → new) → Verification evidence → Prevention checklist item.

## Constraints
- GitMap search only. Relative paths only. No secrets.
