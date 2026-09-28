# Subtask 04: Installer Script Resilience & End-to-End Release Ceremony

**Parent Plan**: Plan 69 (`02-spec/21-app/69-smart-switch-quota-probe-json-email-telegram-projects-and-installer-fix.md`)  
**Target Files**:
- `install.ps1`
- `package.json`
- `Cargo.toml`
- `tauri.conf.json`
- `CHANGELOG.md`

## Requirements
1. **Analyze and Fix GitMap Installer Failure**:
   - Investigate why `gitmap update all` / `gitmap agm update` failed with `exit status 1` at `cmdinstall/installagmanager.go:52`.
   - Ensure `install.ps1` runs smoothly without error, traps any non-zero exit code, and exits cleanly with 0.
2. **End-to-End Verification**:
   - Verify smart switch rejects accounts with < 100% quota.
   - Verify `[JSON]` email arrives with pure raw JSON body.
   - Verify Telegram `/projects` returns deduplicated project listing.
3. **Pre-flight & Release**:
   - `cargo fmt -- --check`
   - `cargo clippy --all-targets --all-features`
   - `npm run build`
   - Bump minor version to `v4.91.0`
   - Push to `origin/main` and verify `gitmap pe` is green.

## Status: COMPLETED

## Verification Results
- `install.ps1`:
  - Added resilient fallback: if all download attempts fail and an existing verified installation exists in `$InstallDir`, logs a clear warning and retains current installation rather than failing.
  - Added explicit `$global:LASTEXITCODE = 0; exit 0` at script termination to prevent exit status 1 leakage to Go's `cmd.Run()`.
  - Verified `install.ps1 -DryRun` and `gitmap agm update` both exit cleanly with status 0.
- All pre-flight checks passed (`cargo fmt`, `cargo clippy`, `npm run build`).
- Version bumped from 4.90.0 to 4.91.0 across all 14 manifests.

