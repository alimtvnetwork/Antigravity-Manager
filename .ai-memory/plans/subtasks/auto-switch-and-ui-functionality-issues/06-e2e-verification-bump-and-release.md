---
plan: auto-switch-and-ui-functionality-issues
subtask: "06"
title: Comprehensive E2E Verification, Quality Gates, and Release Ceremony (v4.208.0)
domain: e2e-testing-and-release
depends_on:
  - "01"
  - "02"
  - "03"
  - "04"
  - "05"
citations:
  architecture_spec: ../../../../02-spec/21-app/auto-switch-and-ui-functionality-issues/00-master-audit-ledger.md
  coding_guidelines: ../../../coding-guidelines.md
  strictly_avoid: ../../../strictly-avoid.md
target_files:
  - package.json
  - src-tauri/Cargo.toml
  - src-tauri/tauri.conf.json
  - README.md
  - README_EN.md
  - CHANGELOG.md
  - CHANGELOG_EN.md
status: pending
---

# Subtask 06: Comprehensive E2E Verification, Quality Gates, and Release Ceremony (v4.208.0)

## 1. Context & Motivation

The user explicitly reprimanded past incomplete verifications:
> *"I asked to retest for you and other AI to verify this by using end-to-end testing by running the cargo, and build, and testing, and opening new instances, putting it to 98, putting some prompt in to check whether it is working or not. That is something that we requested several times. It's looking very bad now. You need to put your full focus on why the auto-switch does not work, why the fast forward manual click also does not work."*

To satisfy this mandate, Wave 5 (Verification & Release) must execute rigorous local end-to-end simulations, confirm all 12 traceable tasks in `00-master-audit-ledger.md`, execute strict pre-flight gates, and conduct an atomic minor version bump to `v4.208.0` with full documentation synchronization.

---

## 2. Comprehensive E2E Verification Protocol

Execute an exhaustive 6-phase verification ladder covering all user complaints:

### Phase 1: Local Pre-flight Verification Gate
1. **Rust Code Formatting**: `cd src-tauri && cargo fmt -- --check`
2. **Rust Compiler & Clippy Gate**: `cd src-tauri && cargo clippy --all-targets --all-features`
3. **Frontend Compilation & Types Gate**: `npm run build`
4. **Targeted Unit Tests**: `cd src-tauri && cargo test --lib modules::auto_switcher modules::audit modules::backup_prompts_db`

### Phase 2: Live Quota Sync & 403 Fallback Verification
1. Verify that accounts encountering `403 Forbidden` on Google project quota endpoints safely fall back to the account-wide quota summary without zeroing out or displaying stale 100% false data.
2. Verify that the 30s-50s background quota refresh daemon continuously refreshes quotas and emits `accounts://refreshed` events to the React frontend.

### Phase 3: Auto-Switch 98% Threshold & Fast-Forward Simulation
1. Simulate an active instance running a prompt with quota dropping to 98% exhaustion (2% remaining).
2. Confirm that `evaluate_instances` detects the breach and automatically initiates candidate evaluation.
3. Confirm that the highest-scoring candidate is chosen and `execute_profile_rotation_with_context` successfully binds the new account.
4. Verify that manual Fast-Forward trigger (`fast_forward_instance`) unblocks immediately and does not hang on stale locks.

### Phase 4: Windows Subprocess Window Suppression (`CREATE_NO_WINDOW`)
1. Audit all 14 Windows subprocess spawn sites in Rust.
2. Confirm that triggering auto-switch, Git operations, or prompt dispatches does NOT pop up brief flash command prompt / PowerShell terminal windows on the desktop.

### Phase 5: Interaction Audit Log & Call Stack Verification
1. Open the Interaction Audit Log modal (`AuditPanel.tsx`).
2. Verify that `METHOD` and `TARGET` cells are completely populated and not blank.
3. Expand a row: confirm that caller function name and call stack trace are visible.
4. Click `JSON`: confirm that formatted JSON is copied to the clipboard, the button switches to `Copied!` with check icon, and a success toast appears.
5. Click `JSONL`: confirm newline-delimited JSON is written to clipboard.
6. Verify that auto-switch candidate evaluations and rotation executions appear as `system` interaction rows.

### Phase 6: Privacy, Backup Retention & Updater Verification
1. Inspect Instance Card account badge: verify hover does NOT unmask email, and DOM `title` contains masked representation.
2. Verify that SQLite prompt backup table retains at most `max_backup_prompts` (100) records adhering to FIFO pruning.
3. Inspect NavMenu update icon: confirm smooth upward hover transition without `animate-bounce`.
4. Verify updater CLI console opens with 18pt font and clean ANSI text.

---

## 3. Atomic Release Ceremony (v4.208.0)

### 3.1 Version Synchronization
Execute:
```bash
npm run bump minor
```
This atomically bumps the SemVer version from `4.207.0` to `4.208.0` across 14 synchronized project manifests:
- `package.json`
- `package-lock.json`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/tauri.conf.json`

### 3.2 Documentation Synchronization & Attribution Invariant
Per the project maintenance guidelines:
1. **Strict Attribution Invariant**: Attribution in `CHANGELOG.md` and `CHANGELOG_EN.md` must attribute strictly `@aukgit` (`(Thanks to @aukgit)`). Do NOT include any other GitHub user handles (`@...`).
2. **README Changelog Synchronization**: Update the release summary in both `README.md` (under "## 📝 更新日志") and `README_EN.md` (under "## 📝 Changelog"). Never update only `CHANGELOG.md` while leaving README files outdated.

#### Changelog Summary Template:
```markdown
## [v4.208.0] - 2026-10-16

### 🚀 Highlights & Features
- **Auto-Switch & Live Quota Synchronization**: Fixed Google project 403 quota drop fallback, implemented active 30s-50s background quota polling with `accounts://refreshed` events, and unblocked manual Fast-Forward execution.
- **Process Window Suppression**: Applied `CREATE_NO_WINDOW` (`0x08000000`) across all 14 Windows subprocess spawn sites to eliminate command prompt window popups.
- **Interaction Audit Log Direct Clipboard Copy**: Replaced failing `<a download>` DOM link synthesis with genuine OS clipboard copying (`copyToClipboard`) for JSON and JSONL exports, complete with visual `Copied!` button states and toast feedback.
- **Audit Metadata & Call Stack Trace Capture**: Resolved serde camelCase mismatch restoring Method and Target columns, fixed Tauri IPC parameter mapping, implemented DOM hierarchy climbing, and enriched expanded rows with call stack traces.
- **Auto-Switch Audit Logging**: Integrated `audit::record_if_active` into candidate scoring and rotation pipelines for full operational transparency.
- **Prompt Backup Retention Cap**: Enforced strict FIFO 100-prompt backup cap in SQLite database with user-configurable retention settings.
- **Strict Email Privacy**: Eliminated hover email leaks in instance cards, standardized email masking across backend logs and Telegram alerts.
- **Desktop Updater Polish**: Replaced jerky bouncing update icon in navigation menu with smooth transitions, configured Windows updater console with 18pt font and ANSI virtual terminal processing.

(Thanks to @aukgit)
```

---

## 4. Final Quality Gates Checklist

- [ ] `cd src-tauri && cargo fmt -- --check` passes cleanly.
- [ ] `cd src-tauri && cargo clippy --all-targets --all-features` passes with zero warnings or errors.
- [ ] `npm run build` succeeds without TypeScript or Vite errors.
- [ ] Working tree is clean (`git status` reports zero untracked or modified files).
- [ ] Release tags match `v4.208.0` exactly.
