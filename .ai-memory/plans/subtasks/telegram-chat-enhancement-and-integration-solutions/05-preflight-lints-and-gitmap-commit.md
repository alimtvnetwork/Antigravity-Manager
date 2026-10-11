# Subtask 05: Preflight Quality Gates, Lints & GitMap Commit

## Metadata
- **Subtask ID:** `05`
- **Slug:** `05-preflight-lints-and-gitmap-commit`
- **Parent Plan:** `.ai-memory/plans/telegram-chat-enhancement-and-integration-solutions.md`
- **Status:** completed
- **Owner:** Antigravity AI
- **Target Operations:** Quality Gates, Linters, Hygiene Audits, GitMap Commit

---

## 1. Objectives & Scope
Execute comprehensive end-to-end quality assurance, compile gates, clippy linter passes, code formatting verification, invariant audits (positive booleans, strict relative paths, no silent errors), and conclude with an atomic GitMap commit and push to remote.

---

## 2. Target Files & Touch Points

1. **Rust Backend Crate**: `src-tauri/`
   - Formatter verification: `cargo fmt -- --check`
   - Linter pass: `cargo clippy --all-targets --all-features`
   - Unit test suites:
     - `src-tauri/src/modules/telegram_inbound/`
     - `src-tauri/src/modules/email_inbound/`
     - `src-tauri/src/modules/notification_hub/`
2. **Specification & Plan Documentation**:
   - `02-spec/21-app/telegram-chat-enhancement-and-integration-solutions/`
   - `.ai-memory/plans/telegram-chat-enhancement-and-integration-solutions.md`
   - `.ai-memory/plans/subtasks/telegram-chat-enhancement-and-integration-solutions/`

---

## 3. Preflight Execution Sequence

### Step 1: Rust Code Formatting Gate
```bash
cd src-tauri && cargo fmt -- --check
```
- **Acceptance Criterion**: Zero formatting violations. If unformatted, execute `cargo fmt` to align syntax.

### Step 2: Comprehensive Clippy Linter Gate
```bash
cd src-tauri && cargo clippy --all-targets --all-features
```
- **Acceptance Criterion**: Clean compilation with zero warnings or errors. Must satisfy all Rust idioms and borrow checker rules.

### Step 3: Targeted Subsystem Unit Tests
```bash
cd src-tauri && cargo test modules::telegram_inbound
cd src-tauri && cargo test modules::email_inbound
cd src-tauri && cargo test modules::notification_hub
```
- **Acceptance Criterion**: 100% test pass rate across all touched modules.

### Step 4: Non-Negotiable Invariants Audit
- [ ] **Strict Relative Git Paths**: Grep/verify zero absolute paths (`C:\...`, `/home/...`) introduced in spec or code files.
- [ ] **Positive Boolean Naming**: Verify all booleans use positive terms (`is_active`, `is_success`, `has_images`).
- [ ] **No Silent Errors**: Verify all fallible operations propagate through `AppError` or `crate::error::record_ignored` with justification comments.

### Step 5: Atomic GitMap Commit Ceremony
```bash
gitmap cpf -m "feat(telemetry): telegram html rich cards, free catbox media vault & standalone fleet discovery"
```
- **Acceptance Criterion**: Clean working tree, all modified and new files tracked, zero temporary or junk files committed.

---

## 4. Verification & Output Summary

| Step | Gate / Tool | Command | Expected Outcome |
| :--- | :--- | :--- | :--- |
| 1 | Formatting | `cargo fmt -- --check` | Exit code 0 |
| 2 | Clippy | `cargo clippy --all-targets --all-features` | Exit code 0 |
| 3 | Unit Tests | `cargo test modules::telegram_inbound` | All tests pass |
| 4 | Hygiene Audit | Relative paths & positive booleans check | Verified compliant |
| 5 | Git Commit | `gitmap cpf` | Clean working tree |
