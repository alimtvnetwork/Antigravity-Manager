# RCA-05 — `TestTopLevelCmdRegistryMatchesAST` failed: file split dropped the `// gitmap:cmd top-level` marker (140 registry entries with no matching AST declaration)

Repo: `alimtvnetwork/gitmap-v28` (`cli/constants/`, Go) · CI workflow step:
`go test ./...` (package `constants`) · Breaking commit: `83939b8ae` (the
300-line-cap file-split commit) · Fix commit: `55120fc07`

## Symptom

The package test `TestTopLevelCmdRegistryMatchesAST` failed in CI with:

```
topLevelCmds() registry has 140 entry/entries with no matching AST declaration
```

followed by a 140-name list. In plain language: the hand-maintained
`topLevelCmds()` registry claims 140 command names exist, but the test's AST
scan of the source tree could not find declarations for any of them. Every
name on the missing list lived in the newly created file
`cli/constants/constants_commands_extra.go`.

## Root cause

**One line:** the split moved const blocks into
`cli/constants/constants_commands_extra.go` without the
`// gitmap:cmd top-level` marker comment that the AST collector requires, so
the collector silently ignored every moved constant.

Explanation, step by step, for zero-context readers:

1. gitmap keeps its CLI command IDs as Go string constants (`CmdScan =
   "scan"`, `CmdCloneNext = "clone-next"`, plus `CmdFooAlias` variants) in
   `cli/constants/`.
2. A test — `TestTopLevelCmdRegistryMatchesAST` in
   `cli/constants/cmd_constants_parity_test.go` — is the §5 future-hardening
   guard from `02-spec/01-app/99-cli-cmd-uniqueness-ci-guard.md`. It walks
   every `constants_*.go` file with `go/parser`, and collects every `Cmd*`
   string constant **only if** it sits inside a `const` block whose doc
   comment contains the magic marker `// gitmap:cmd top-level` (constant
   `parityMarkerTopLevel` in the test). Lines tagged `// gitmap:cmd skip`
   are excluded. It then asserts the collected set is *exactly equal* to the
   manual `topLevelCmds()` registry in `cmd_constants_test.go`. Registry
   drift in either direction fails CI by design.
3. A sibling mirror guard, `TestNoUnmarkedTopLevelCmdBlocks`, rejects any
   const block declaring `Cmd*` constants *without* the marker — so the
   codebase treats the marker as load-bearing metadata, not decoration.
4. Before the split, all command constants lived in
   `cli/constants/constants_cli.go` under marked blocks. The split commit
   carved the tail of that file into the new
   `cli/constants/constants_commands_extra.go`, but gave the moved const
   block a plain descriptive comment:

   ```go
   // Additional CLI commands (extended set).
   const (
   ```

   No `// gitmap:cmd top-level` line. The parser therefore collected the
   markers from `constants_commands.go` (17 marker occurrences) and **zero**
   from `constants_commands_extra.go` — while `topLevelCmds()` still listed
   all 140 moved names. Set difference: 140 entries in the registry with no
   AST match. Exactly the failure observed.

The bug class is "metadata lost in a move": the marker is invisible to the
compiler and to human readers skimming the diff, but a test treats it as the
sole signal for "this block declares real commands". Splits must move the
signal with the code.

## How it was found (exact commands)

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28

# 1. Read the failing test to learn what it actually scans for
gitmap cat cli/constants/cmd_constants_parity_test.go
#    -> the header comment says it plainly: it collects "every Cmd* string
#       constant declared inside a const block marked `// gitmap:cmd top-level`"
#       and asserts exact equality with the topLevelCmds() registry.
#    -> constants parityMarkerTopLevel = "gitmap:cmd top-level"

# 2. Count the marker per constants file (this is the whole diagnosis)
gitmap aum search 'gitmap:cmd top-level' cli/constants/constants_commands.go
#    -> 17 hit(s)
gitmap aum search 'gitmap:cmd top-level' cli/constants/constants_commands_extra.go
#    -> 0 hit(s)

# 3. Confirm the new file really does declare the missing commands
gitmap cat cli/constants/constants_commands_extra.go | sed -n '1,10p'
#    -> shows "// Additional CLI commands (extended set)." above the const
#       block — a plain comment, no marker — followed by CmdCompletion etc.
```

Step 2 is the diagnosis: 17 vs 0. The constants are all there; the test just
cannot see the ones in the unmarked file.

## Fix

One comment line added above the moved const block in
`cli/constants/constants_commands_extra.go`:

Before (lines 1–5):

```go
package constants

// Additional CLI commands (extended set).
const (
	CmdCompletion            = "completion"
```

After (verified on disk today, line 3):

```go
package constants

// gitmap:cmd top-level
// Additional CLI commands (extended set).
const (
	CmdCompletion            = "completion"
```

The marker goes on the const block's doc comment — the exact position the
parser inspects (`gen.Doc` of the `*ast.GenDecl`). Putting it on an
individual constant line or at the bottom of the file would not work; it must
be part of the block's leading comment group.

Commit facts: fix shipped as commit `55120fc07` (one-line comment addition).

## Verification evidence

1. The affected package test was re-run with the exact filter from the
   brief: `go test ./constants/ -run TestTopLevelCmdRegistryMatchesAST` →
   **PASS**. This is the same test that failed in CI, so its green run is
   the direct proof.
2. The mirror guard `TestNoUnmarkedTopLevelCmdBlocks` (same package, same
   `go test ./constants/` run) also passes — no other const block in the
   package is missing its marker.
3. Today (2026-10-15) the marker is still at
   `cli/constants/constants_commands_extra.go:3` (verified via
   `gitmap aum search 'gitmap:cmd top-level'
   cli/constants/constants_commands_extra.go` → 1 hit at line 3), and
   `constants_commands.go` still carries its 17 markers — the marked-set and
   the registry agree on disk.

## Prevention — checklist item

**Marker-preservation rule for file splits:**

1. Before moving any code, search the moved region for magic markers:
   `gitmap aum search 'gitmap:' <donor-file>` (in this repo the marker family
   is `gitmap:cmd top-level` / `gitmap:cmd skip`; other repos have their own
   — the point is to look).
2. Move each marker *with* its declaration, in the exact position the tooling
   expects (block doc comment, not a random line nearby).
3. After the split, run the affected package's tests locally **before**
   pushing — here that is `go test ./constants/` — not just `go build`. A
   split that compiles can still break a source-scanning test, and only the
   package tests will tell you.

One-line rule for the dummy AI: **magic comments are code — a file split that
drops them is an incomplete split, even if everything compiles.**
