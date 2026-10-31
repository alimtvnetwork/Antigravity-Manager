# Subtask 03: Installer Parity & E2E Test Harness Implementation

**Slug**: `224-repository-setup-and-installation-instructions`  
**Parent Spec**: `02-spec/21-app/224-repository-setup-and-installation-instructions/03-e2e-and-installer-spec.md`  
**Master Ledger**: `02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md`  
**Status**: `DONE`  
**Owner**: Worker 02 (Runners & Build Specialist)  

---

## 1. Overview & Objective

This subtask delivers the unified end-to-end testing harness:
1. Created `e2e.ps1` (Windows) and `e2e.sh` (Linux/macOS) with structured ANSI status reporting.
2. Implemented caller IDE self-preservation (`ProtectedCallerPid`), preventing test scripts from terminating the developer's working editor session.
3. Implemented transient test sandboxing under `.temp/test-sandbox-<timestamp>-<rand>/` with automated cleanup.
4. Added support for test suites: `all`, `instance`, `lifecycle`, `switcher`, `prompts`, `smoke`.

---

## 2. Target File Manifest

| Target File | Status | Action Required |
| :--- | :--- | :--- |
| `e2e.ps1` | Created | Unified Windows E2E harness with transient sandboxing and ANSI tables. |
| `e2e.sh` | Created | Unified POSIX E2E harness with transient sandboxing and exit trapping. |

---

## 3. Verification & Acceptance Criteria

- [x] `e2e.ps1 -Suite smoke` passes cleanly in <1s with 0 errors.
- [x] `e2e.sh --suite smoke` passes cleanly with 0 errors.
- [x] Caller IDE PID detected and protected from termination.
- [x] Transient sandboxes created under `.temp/test-sandbox-*` and purged in `finally` / `trap EXIT` block.
- [x] Strictly relative Git paths maintained.
