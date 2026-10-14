# RCA-06 — `TestEveryCmdIDHasHelpFile` failed: source-parsing test hardcoded a path the file split deleted (`parsed zero Cmd* constants — locator broke?`)

Repo: `alimtvnetwork/gitmap-v28` (`cli/helpdoc/`, Go) · CI workflow step:
`go test ./...` (package `helpdoc`) · Breaking commit: `83939b8ae` (the
300-line-cap file-split commit) · Fix commit: SHA not recorded in the task
brief (the locator rewrite described below; verify via the file's history if
needed)

## Symptom

The package test `TestEveryCmdIDHasHelpFile` failed in CI with:

```
parsed zero Cmd* constants — locator broke?
```

That message is the test's own words (a `t.Fatal` in
`cli/helpdoc/coverage_test.go:32`). It is a deliberate tripwire: the test
parses the command-constant source files to learn every command ID, then
asserts each ID has a matching `helptext/<id>.md` file. Parsing *zero*
constants means the locator — the function that finds the source file to
parse — is pointing at a file that no longer holds the constants.

## Root cause

**One line:** `locateCLIConstantsFile()` in `cli/helpdoc/coverage_test.go`
hardcoded the path `constants/constants_cli.go`, and the split commit gutted
that file into a 6-line stub, so the parser read a file containing zero
`Cmd*` constants.

Explanation, step by step, for zero-context readers:

1. Before the split, **all** CLI command constants lived in one file,
   `cli/constants/constants_cli.go`. The helpdoc coverage test parsed that
   single file with `go/parser`, extracted every `CmdFoo = "foo"` declaration
   (`collectFromDecl`), filtered down to primaries (skipping
   `CmdFooAlias*` and the documented `helptextExemptConstants` list), and
   required a `helptext/<value>.md` for each. Adding `CmdFoo = "foo"` without
   a help file fails CI — that is the test's purpose.
2. The split commit moved the ~392 `Cmd*` lines out of `constants_cli.go`
   into two new files, `cli/constants/constants_commands.go` and
   `cli/constants/constants_commands_extra.go`. What remained in
   `constants_cli.go` is a 6-line stub (verified on disk today):

   ```go
   package constants

   const UsageHeaderFmt = "\n  " + ColorMagenta + "🗺  " + ColorWhite + "gitmap" + ColorReset +
   	"  " + ColorCyan + "v%s" + ColorReset +
   	"  " + ColorDim + "— Git repo discovery, cloning & release toolkit" + ColorReset + "\n" +
   	"  " + ColorMagenta + "────────────────────────────────────────────────────────────" + ColorReset + "\n\n"
   ```

   Zero `Cmd*` constants. The constant the test collects simply is not there
   anymore.
3. The test's locator still returned the old single path, so
   `parsePrimaryCmdIDs()` parsed the stub, `collectFromDecl` found nothing,
   `len(ids) == 0`, and the tripwire fired. Note the failure is *honest*: the
   test told you exactly what broke ("locator broke?") instead of silently
   passing with an empty command list — if it had skipped the check on zero
   IDs, the split would have silently disabled helptext coverage forever.

The bug class is "test coupled to a refactorable path": production code moved
on, but a test's hardcoded filesystem assumption did not. The constants were
never lost — the two new files hold the exact union of the old file's
`Cmd*` lines (verified at fix time: `diff` of old vs union came back empty) —
the test just stopped looking in the right place.

## How it was found (exact commands)

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28

# 1. Get the CI failure (downloaded via the API per the incident brief)
#    -> "parsed zero Cmd* constants — locator broke?" in cli/helpdoc

# 2. Read the locator and the parse loop
gitmap cat cli/helpdoc/coverage_test.go | sed -n '26,35p'
#    -> the t.Fatal("parsed zero Cmd* constants — locator broke?") tripwire
gitmap aum search 'locateCLIConstantsFile' cli/helpdoc/coverage_test.go
#    -> pre-fix: a single function returning the one hardcoded path
#       constants/constants_cli.go

# 3. Look at what that path points to now
gitmap cat cli/constants/constants_cli.go
#    -> the 6-line UsageHeaderFmt stub quoted under "Root cause": zero Cmd*
#       constants, as the tripwire claimed

# 4. Prove the constants were moved, not deleted: the old file held 392 Cmd*
#    lines; the two new files hold the exact union (diff empty at fix time).
#    Current-state spot check (includes doc-comment mentions, so the raw
#    counts are higher than the 392 pure-declaration count):
gitmap cat cli/constants/constants_commands.go | awk '/Cmd[A-Z]/{c++} END{print c+0}'
#    -> 274
gitmap cat cli/constants/constants_commands_extra.go | awk '/Cmd[A-Z]/{c++} END{print c+0}'
#    -> 200
```

Step 3 is the diagnosis: the locator's target file exists but is now a stub.
Step 4 rules out the scarier hypothesis (constants accidentally deleted in
the split).

## Fix

Replaced the single-file locator with a multi-file locator, and made the
parser iterate the returned list. Both changes live in
`cli/helpdoc/coverage_test.go`.

Before — `locateCLIConstantsFile()` (singular) returned one hardcoded path:

```go
// (pre-fix shape) resolve the single command-registry source file
func locateCLIConstantsFile() string {
	...
	return filepath.Join(constantsDir, "constants_cli.go")
}
```

After — `locateCLIConstantsFiles()` (plural), verified on disk today at
`cli/helpdoc/coverage_test.go:145-159`:

```go
// locateCLIConstantsFiles resolves the command-registry source files
// (constants/constants_commands.go and constants/constants_commands_extra.go,
// the split successors of the old constants_cli.go) relative to this test
// file (works regardless of `go test` cwd).
func locateCLIConstantsFiles() []string {
	_, thisFile, _, _ := runtime.Caller(0)
	pkgDir := filepath.Dir(thisFile)
	constantsDir := filepath.Join(pkgDir, "..", "constants")

	return []string{
		filepath.Join(constantsDir, "constants_commands.go"),
		filepath.Join(constantsDir, "constants_commands_extra.go"),
	}
}
```

And `parsePrimaryCmdIDs()` (verified on disk today at
`cli/helpdoc/coverage_test.go:58-72`) now loops over both paths:

```go
// parsePrimaryCmdIDs returns every primary command constant from
// constants/constants_commands.go and constants/constants_commands_extra.go
// (aliases and exempt entries skipped).
func parsePrimaryCmdIDs() ([]cmdID, error) {
	var out []cmdID
	for _, path := range locateCLIConstantsFiles() {
		fset := token.NewFileSet()
		file, err := parser.ParseFile(fset, path, nil, parser.AllErrors)
		if err != nil {
			return nil, err
		}

		for _, decl := range file.Decls {
			out = append(out, collectFromDecl(decl)...)
		}
	}

	return filterPrimaries(out), nil
}
```

Two design details worth copying:

- Paths resolve via `runtime.Caller(0)` (the test file's own location), not
  the process working directory — `go test` can run from anywhere, and a
  relative path would break depending on invocation.
- The comment on the locator **names the split** ("the split successors of
  the old constants_cli.go"). That one sentence is what stops the next
  refactorer from re-introducing a single hardcoded path.

## Verification evidence

1. The affected package test was re-run with the exact filter from the
   brief: `go test ./helpdoc/ -run TestEveryCmdIDHasHelpFile` → **PASS**.
   This is the same test that failed in CI, so its green run is the direct
   proof — and because it parses both files, it now covers the full command
   set again (the union verified equal to the pre-split file).
2. Today (2026-10-15) the fixed locator and the iterating parser are the
   current on-disk state (verified via `gitmap cat
   cli/helpdoc/coverage_test.go`, lines 55–72 and 145–159), and
   `cli/constants/constants_cli.go` is still the 6-line stub — i.e. the old
   hardcoded path would *still* parse zero constants today, confirming the
   locator rewrite is what keeps the test meaningful.

## Prevention — checklist item

**Rule for tests that parse source files:**

- Never hardcode a single source path that a refactor can invalidate. Prefer
  a `filepath.Glob` over the source directory (e.g.
  `constants/constants_*.go`) or — if the file set must be explicit — an
  explicit list **with a comment naming the split** that created it, exactly
  like the current `locateCLIConstantsFiles` comment does.
- Keep the `len(ids) == 0 → t.Fatal("...locator broke?")` tripwire pattern:
  a source-scanning test that silently passes on zero inputs is worse than
  useless — it lets a future refactor quietly disable the coverage it was
  built to enforce. Fail loud when the scan finds nothing.
- After any file split, run the test packages that *read* the moved files,
  not just the package that *owns* them. `go build ./...` would never have
  caught this; only `go test ./helpdoc/` could.

One-line rule for the dummy AI: **if your test opens a source file by name,
that filename is a dependency — treat a file move as a breaking change to the
test.**
