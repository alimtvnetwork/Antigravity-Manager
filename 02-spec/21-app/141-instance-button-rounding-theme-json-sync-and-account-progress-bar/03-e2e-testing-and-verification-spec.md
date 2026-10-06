# Specification 141: E2E Testing, Quality Gates & Verification Specification

## 1. Quality Gates & Pre-flight Standards

### 1.1 Backend Rust Compilation & Rustfmt Gate
- `cd src-tauri && cargo fmt -- --check` must pass with zero formatting diffs.
- `cd src-tauri && cargo clippy --all-targets --all-features` must compile cleanly with 0 warnings/errors.
- Resolved items:
  - `src-tauri/src/modules/account.rs:2329`: `QuotaData` instantiated with `subscription_tier_fetched_at: None` or `QuotaData::new()`.
  - `src-tauri/src/modules/auto_switcher.rs:2630`: `QuotaData` initialized with `subscription_tier_fetched_at: None`.
  - `src-tauri/src/modules/quota.rs`: Rustfmt multiline function arguments restored.
  - `src-tauri/src/commands/fleet.rs:84`: Rustfmt argument wrapping restored.

### 1.2 Frontend Build Gate
- `npm run build` must compile TypeScript without errors and generate the production bundle into `dist/`.

### 1.3 GitMap CI Diagnostics (`gitmap pe`)
- Verify `gitmap pe` output reports no unhandled compilation failures.

---

## 2. End-to-End Automated Verification (`scripts/test-instance-sync-e2e.ps1`)

### 2.1 Test Execution Matrix
1. **Settings Copy Verification**:
   - Verify that copying settings from Instance A to Instance B merges `settings.json` keys without corrupting `window.title`.
2. **Project / Workspace Copy Verification**:
   - Verify that copying workspaces preserves `workspaceStorage` folders and links database records in `repo_prompts.db`.
3. **JSON Export & Import Verification**:
   - Verify that settings exported to JSON can be parsed and imported back with identical key-value parity.
4. **GitMap Format Invariant**:
   - Verify that project nodes serialize with `[AGM:P001 | GM:#1]` dual sequence tags.

---

## 3. Visual & Interactive Verification Checklist
- [ ] Top action buttons in `src/pages/Instances.tsx` have 4px rounding on the left and right outer ends, and square in the middle.
- [ ] Theme middle and hover colors react properly to JSON palette configuration.
- [ ] Account progress bar width is reduced by 18%, height is increased, 11 checkpoint nodes are visible and colored, and color smoothly blends from red to vibrant prominent green.
- [ ] Email masking button tooltip and labels accurately state email visibility and masking.
- [ ] Instance Sync Settings modal provides intuitive source and target selection with move/copy/JSON actions.
