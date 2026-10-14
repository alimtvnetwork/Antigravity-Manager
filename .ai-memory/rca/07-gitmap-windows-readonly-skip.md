# RCA-07 — `TestEffectiveDataDir_ReadOnlyFallback` failed on `windows-latest` only: POSIX `chmod 0555` is not enforceable on Windows

Repo: `alimtvnetwork/gitmap-v28` (`cli/store/`, Go) · CI workflow step:
`go test ./...` (package `store`, `windows-latest` runner only) ·
Originating change: the read-only-binary-dir fallback from spec
`02-spec/22-app-issues/73-ssh-fleet-pull-gitmap-lock-permission-and-wincredman-rca.md`
· Fix (by another stream): `t.Skip` on `runtime.GOOS == "windows"` · Fix
commit: `afb75a80e`

## Symptom

The package test `TestEffectiveDataDir_ReadOnlyFallback` failed in CI **on
`windows-latest` only** — Linux and macOS runners were green. The failure:

```
expected EffectiveDataDir to fall back away from read-only dir, got "<the read-only dir>"
```

In plain language: the test created a directory, made it "read-only" with
`chmod 0555`, and expected `EffectiveDataDir()` to refuse it and fall back to
the global user data dir. On Windows, `EffectiveDataDir()` happily returned
the "read-only" directory — the fallback never triggered.

The platform-only failure pattern is the signature clue: when one OS fails
and the others pass on the *same commit*, the test is asserting something the
OS does not guarantee. Here, that something is POSIX permission semantics.

## Root cause

**One line:** the test relies on POSIX `chmod 0555` to make a directory
unwritable, but Windows does not enforce POSIX mode bits on directories, so
the directory stays writable and the fallback condition never occurs.

Explanation, step by step, for zero-context readers:

1. Background (from the spec): gitmap co-locates its SQLite databases with
   the binary (`BinaryDataDir()`). On multi-user or system installs —
   `/usr/local/bin/` on Linux, `C:\Program Files\` on Windows — that
   directory is read-only for regular users. So `EffectiveDataDir()` (in
   `cli/store/location.go:124`) was given fallback logic:

   ```go
   // EffectiveDataDir returns BinaryDataDir if it is writable. If BinaryDataDir
   // is read-only or unwritable (e.g. system install in /usr/local/bin/gitmap-cli/data),
   // it automatically falls back to GlobalUserDataDir() (~/.gitmap/data).
   func EffectiveDataDir() string {
   	dir := BinaryDataDir()
   	if isDirWritable(dir) {
   		return dir
   	}
   	if globalDir := GlobalUserDataDir(); globalDir != "" {
   		_ = os.MkdirAll(globalDir, 0755)
   		return globalDir
   	}
   	return dir
   }
   ```

2. `isDirWritable` (`cli/store/location.go:32`) does a **behavioral** probe,
   not a mode-bit inspection: `isTestFileWritable` actually tries to create
   a temp file (`.gitmap_write_test_<pid>_<nanos>`) inside the directory and
   reports success/failure. This is the right design — it measures what the
   process can really do.
3. The test (`cli/store/location_test.go:65`) manufactures the read-only
   condition the POSIX way:

   ```go
   readOnlyDir := filepath.Join(tempDir, "readonly_data")
   if err := os.MkdirAll(readOnlyDir, 0555); err != nil { ... }
   ...
   SetBinaryDataDirForTesting(readOnlyDir)
   effective := EffectiveDataDir()
   if effective == readOnlyDir {
   	t.Errorf("expected EffectiveDataDir to fall back away from read-only dir, got %q", effective)
   }
   ```

   On Linux/macOS, mode `0555` (read+execute, no write) on a directory
   blocks file creation inside it → the probe fails → the fallback fires →
   the test passes.
4. On Windows, filesystems use ACLs, not POSIX mode bits. Go's `os.Chmod`
   on Windows maps only the owner-write bit (`0200`) to the
   `FILE_ATTRIBUTE_READONLY` flag — and crucially, Windows **ignores the
   read-only attribute on directories** when deciding whether new files may
   be created inside them (the attribute is meaningful for files, not
   containers). So the behavioral probe succeeds, `isDirWritable` returns
   true, `EffectiveDataDir()` returns the "read-only" dir, and the test's
   expectation fails.

   A telling footnote: the sibling test
   `TestOpenDefault_ReadOnlyBinaryDirFallback` (`location_test.go:40`) uses
   the identical `chmod 0555` trick but only asserts that `OpenDefault()`
   *succeeds* — which holds whether or not the fallback fires. It survived
   on Windows by accident of its weaker assertion; only the test that
   asserts the fallback *did* fire can fail on Windows.

The bug class is "platform-dependent test premise": the test is correct on
POSIX and meaningless on Windows, because the precondition it constructs
("this directory is unwritable") cannot be constructed the same way there.

## How it was found (exact commands)

```bash
export PATH="$HOME/.local/bin:$PATH"
cd ~/workspace/repos/gitmap-v28

# 1. CI log: the failure appears only under the windows-latest job.
#    Same commit, same test, green on ubuntu-latest and macos-latest.
#    -> platform-dependent premise, not a logic bug.

# 2. Read the test to see the premise being constructed
gitmap cat cli/store/location_test.go | sed -n '65,88p'
#    -> os.MkdirAll(readOnlyDir, 0555), then expects EffectiveDataDir()
#       to return something OTHER than readOnlyDir

# 3. Read the production code the test exercises
gitmap cat cli/store/location.go | sed -n '32,48p'
#    -> isDirWritable does a real file-creation probe (isTestFileWritable)
gitmap cat cli/store/location.go | sed -n '121,135p'
#    -> EffectiveDataDir returns BinaryDataDir() when the probe says writable

# 4. Read the spec for the original design intent
gitmap cat 02-spec/22-app-issues/73-ssh-fleet-pull-gitmap-lock-permission-and-wincredman-rca.md | sed -n '100,103p'
#    -> the fallback exists for read-only binary dirs (system installs);
#       the tests were added alongside that feature
```

Step 1 narrows it to the OS; steps 2–3 show the mechanism (chmod premise vs
behavioral probe); step 4 confirms the feature intent is real and the test —
not the fallback logic — is what needs the platform guard. Nobody "fixed"
`EffectiveDataDir`: on a genuinely read-only Windows dir (ACL-denied) the
probe fails and the fallback works fine. The test just cannot *manufacture*
that state with `chmod`.

## Fix

Added a platform guard at the top of the test (fix by another stream,
commit `afb75a80e`). Current on-disk state at
`cli/store/location_test.go:65-68` (verified today):

Before:

```go
func TestEffectiveDataDir_ReadOnlyFallback(t *testing.T) {
	tempDir := t.TempDir()
```

After:

```go
func TestEffectiveDataDir_ReadOnlyFallback(t *testing.T) {
	if runtime.GOOS == "windows" {
		t.Skip("POSIX mode 0555 is not enforceable on Windows; the read-only dir remains writable so the fallback never triggers")
	}
	tempDir := t.TempDir()
```

`t.Skip` (not a conditional assertion, not a weakened expectation) is the
right tool: the test's *premise* is unconstructible on Windows, so running
it there can never produce a meaningful signal — only noise. The skip reason
is a full sentence explaining *why*, so the next reader does not "fix" it
by deleting the guard.

## Verification evidence

1. CI re-ran on the fix commit: the `store` package tests went green on
   `windows-latest` (the test now reports SKIP, which is a pass for CI
   purposes), and remained green on ubuntu-latest and macos-latest where the
   test still runs and still asserts the real fallback.
2. The fallback logic itself is untouched — `EffectiveDataDir()` and
   `isDirWritable()` are byte-identical to before; only the test gained the
   guard. No production behavior changed on any platform.
3. Today (2026-10-15) the skip guard is still the first statement of
   `TestEffectiveDataDir_ReadOnlyFallback` at `cli/store/location_test.go:66`
   (verified via `gitmap cat`), with the explanatory reason string intact.

## Prevention — checklist item

**Rule for permission-based tests:**

- Any test that constructs an "unwritable" (or "unreadable", "unexecutable")
  fixture with `os.Chmod` / `os.MkdirAll(..., 0555)` **must** start with:

  ```go
  if runtime.GOOS == "windows" {
  	t.Skip("<one sentence: POSIX mode bits are not enforceable on Windows; <what> remains <capable> so <expected behavior> never triggers>")
  }
  ```

- Prefer *behavioral* assertions (like `isDirWritable`'s real file-creation
  probe) over *mode-bit* assertions in production code — the production code
  here was already correct; only the test assumed POSIX.
- When a CI failure appears on exactly one OS runner, investigate the OS
  premise before the code: read what the test *assumes* about the platform,
  then check whether that assumption holds there. A one-runner failure on a
  shared commit is a platform fact, not a code bug, until proven otherwise.

One-line rule for the dummy AI: **`chmod 0555` is a POSIX spell — it does
nothing enforceable on Windows, so any test built on it skips Windows with
the reason written down.**
