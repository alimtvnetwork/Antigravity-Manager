# Plan 49: Installer Output Tab Indentation, 80-Split 500KB aria2c Download Acceleration & Minor Release v4.38.0

## Status: Completed
- **Started At:** 2026-09-20T13:19:15+08:00
- **Completed At:** 2026-09-20T13:24:30+08:00

## User Request (Verbatim)
```text
When we use the installer script, we have to make sure that anything other than ourself is printed also having the tab space from the left-hand side and have a gap between them when any additional things get into running. Okay? And when you run using the Area 2C, make sure that you split it to 500 KB split for 80 splits at least. Okay, so that would be very faster to download. Can you please do that in the configuration for the shell, cURL, and also the Windows shell as well? Can you please do that for me and make a final bump and release?

also check gitmap pipeline errors or pipeline ai status , check with gitmap llm to learn
```

## Summary of Completed Subtasks

### Subtask 01: PowerShell Installer Indentation & 80-Split aria2c
- **File:** `install.ps1`
- **Accomplishments:**
  - Added `Invoke-IndentedCommand` helper that captures stdout/stderr, prepends tab indentation (`\t`), and surrounds external command execution with visual vertical gaps (blank lines).
  - Configured `aria2c` with `-s 80` (80 parallel split connections) and `-k 500K` (500 KB chunk split) alongside `-x 16` and `-j 16`.
  - Routed `aria2c`, `curl.exe`, previous uninstaller (`uninstall.exe`), and NSIS setup installer through `Invoke-IndentedCommand`.
  - Verified cleanly via `.\install.ps1 -DryRun`.

### Subtask 02: Shell Installer Indentation & 80-Split aria2c
- **Files:** `install.sh`, `deploy/arch/install.sh`
- **Accomplishments:**
  - Implemented `run_indented()` in `install.sh` and `deploy/arch/install.sh` piping output through `sed $'s/^/\t/'` with blank lines before and after.
  - Configured `aria2c` with `--disable-ipv6=true -x 16 -s 80 -j 16 -k 500K`.
  - Hardened cURL fallback options (`-fSL --progress-bar --connect-timeout 10 --retry 3`) formatted through `run_indented()`.
  - Routed package manager installations (`dpkg`, `apt-get`, `dnf`, `yum`, `makepkg`) through `run_indented()`.
  - Verified syntax with `bash -n` on both shell scripts.

### Subtask 03: GitMap Pipeline Status & Learning Verification
- **Command:** `gitmap pipeline-ai status --json`
- **Accomplishments:**
  - Ran dynamic status check on `main`. Confirmed active workflows completed successfully (`lastStatus: completed`, `lastConclusion: success`, Run ID: 35491152571).
  - Verified GitMap LLM guidance and zero pending pipeline errors.

### Subtask 04: Minor Release v4.38.0 Ceremony
- **Accomplishments:**
  - Synchronized version `4.38.0` across manifests (`version.json`, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, `Casks/antigravity-tools.rb`, `readme.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`, and `02-spec/19-main-worker-service/98-changelog.md`).
  - Verified with `python 03-ai-scripts/14-version-sync-checker.py` (Passed in 9.82ms).
  - Verified TypeScript compilation with `npx tsc --noEmit` (Passed with 0 errors).
