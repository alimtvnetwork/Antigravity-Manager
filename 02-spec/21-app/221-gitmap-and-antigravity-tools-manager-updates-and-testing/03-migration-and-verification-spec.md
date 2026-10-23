# Specification: Task 221 - GitMap and Antigravity Tools Manager Updates & Testing
## Migration and Verification Specification

### 1. Database & State Migrations
- No breaking schema migrations required.
- SQLite queries against `prompt_backups` and `active_prompts` leverage existing indexes:
  - `idx_prompt_backups_instance_status`
  - `idx_prompt_backups_created_at`
- All queries strictly enforce `ORDER BY created_at ASC, id ASC`.

### 2. Zero-Build / Zero-Test Invariant During Authoring
- During Phase 1 and Phase 2 (code modifications across `src-tauri`, `src`, and `d:/work/gitmap/cli/cmdpull`), no test runs or compilers will be invoked.
- Build checks and tests are deferred strictly to Phase 3.

### 3. Phase 3 Verification Gates
1. **Rust Code Quality**:
   - `rustfmt --check --edition 2021` across touched files.
2. **Frontend Tests**:
   - `npm test` runs all unit suites including rerun randomizer tests.
3. **Frontend Build**:
   - `npm run build` succeeds with zero errors.
4. **GitMap Go Verification**:
   - `go test -v ./cmdpull` passes in `d:/work/gitmap/cli`.
5. **Master E2E Verification**:
   - Run `scripts/test-agm-gitmap-running-backup-e2e.ps1` passing all gates (28/28).
