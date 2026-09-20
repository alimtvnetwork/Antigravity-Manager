# Consolidated Plan: 52-installer-indent-aria2c-splits

> **Initial Trigger:** User requested installer external command output formatting (tab indentation and blank-line spacing), aria2c configuration with 500KB split and at least 80 splits across Windows PowerShell, Linux Bash, and Arch Linux, inspecting gitmap pipeline errors/telemetry, and creating a final minor release bump.
> **Execution Steps:** 6 continuous self-loop steps executed cleanly across planning, Rust compiler fix, installer refactoring, verification, version synchronization, and release ceremony.

---

## 1. Objectives & Summary of Outcomes

### A. External Tool Output Tab Indentation & Blank-Line Gaps
- **PowerShell (`install.ps1`):**
  - All external commands (`aria2c`, `curl.exe`, previous uninstallers, and NSIS setup installer packages) run through `Invoke-IndentedCommand`.
  - Every external output line is prefixed with a tab (`\t`) to indent cleanly from the left margin, preventing foreign tool outputs from colliding with the installer's native formatted output.
  - Empty line gaps are emitted immediately before and after command execution to guarantee visual breathing room.
- **Bash (`install.sh` & `deploy/arch/install.sh`):**
  - All package managers (`dpkg`, `apt-get`, `dnf`, `yum`, `makepkg`) and downloaders (`aria2c`, `curl`) run through `run_indented`.
  - Piped via `sed $'s/^/\t/'` with preceding and trailing blank lines to provide uniform cross-platform indentation.

### B. aria2c 500KB Split & 80 Splits with Resilient Fallback Ladder
- Configured aria2c with `-s 80 -x 16 -j 16 -k 500K` across all installer scripts (`install.ps1`, `install.sh`, `deploy/arch/install.sh`).
- Handled aria2c's strict upstream option handler: standard aria2c binaries enforce `--min-split-size` between `1048576` (1M) and `1073741824`. If aria2c fails with exit code `28` (option validation error), the scripts catch this code and immediately re-attempt with `-k 1M` and 80 splits before falling back to cURL.

### C. GitMap Telemetry & Rust Compile Remediation
- Discovered CI build failures via `gitmap pipeline error-logs` in `agm-alim (lib)`:
  1. `src-tauri/src/modules/auto_switcher.rs`: Fixed `acc.last_used` type handling (`i64` instead of `Option<i64>`).
  2. `src-tauri/src/proxy/upstream/retry.rs`: Added `pub fn parse_legacy_retry_delay(error_text: &str) -> Option<u64>` forwarding to `parse_retry_delay(error_text, None)`.
- Verified formatting with `cargo fmt --check`.

### D. Release Ceremony & Version Bump (`v4.41.0`)
- Bumped minor version from `v4.40.0` to `v4.41.0` using `37-bump-version.py`.
- Synchronized all manifests: `version.json`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/Cargo.lock`, `src-tauri/tauri.conf.json`, `Casks/antigravity-tools.rb`, `readme.md`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `02-spec/19-main-worker-service/98-changelog.md`.
- Verified version synchronization with `14-version-sync-checker.py` (0 errors).
- Verified TypeScript types with `npx tsc --noEmit` (0 errors).
- Verified bash syntax with `bash -n` (0 errors).
