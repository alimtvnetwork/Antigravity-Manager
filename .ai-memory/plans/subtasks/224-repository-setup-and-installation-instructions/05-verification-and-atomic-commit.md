# Subtask 05: Quality Gates Verification & Atomic GitMap Commit Ceremony

**Task ID:** `224`  
**Step ID:** `05`  
**Slug:** `224-repository-setup-and-installation-instructions`  
**Parent Plan:** [.ai-memory/plans/pending/224-repository-setup-and-installation-instructions.md](../../pending/224-repository-setup-and-installation-instructions.md)  
**Master Audit Ledger:** [02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md](../../../02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md)  
**Status:** `PENDING`  
**Owner:** Root Orchestrator (Lead Agent)  
**Date:** 2026-10-24  

---

## 1. Overview & Objective

This subtask defines the final quality assurance gate and atomic commit ceremony for Task 224 (`224-repository-setup-and-installation-instructions`). It ensures that:
1. All newly introduced runner scripts (`build.ps1`, `e2e.ps1`, `build.sh`, `e2e.sh`), modified scripts (`run.ps1`, `run.sh`, `install.ps1`, `scripts/install-rust-toolchain.ps1`), and toolchain helpers pass static syntax validation.
2. The strictly relative Git paths invariant is 100% verified across all touched files.
3. The zero builds / zero tests constraint is strictly maintained.
4. The master audit ledger and pending plan are transitioned to `COMPLETED`.
5. An atomic GitMap commit is created and pushed following repo release conventions.

---

## 2. Quality Gates & Pre-Flight Verification Checklist

### Gate 1: Script Static Syntax Validation
Validate PowerShell script AST syntax without invoking commands or triggering compilation:
```powershell
$scripts = @(
    "run.ps1",
    "build.ps1",
    "e2e.ps1",
    "install.ps1",
    "scripts/install-rust-toolchain.ps1",
    "scripts/toolchain-db.ps1"
)

foreach ($script in $scripts) {
    if (Test-Path $script) {
        $content = Get-Content $script -Raw
        $errors = $null
        $tokens = $null
        [System.Management.Automation.Language.Parser]::ParseInput($content, [ref]$tokens, [ref]$errors) | Out-Null
        if ($errors.Count -gt 0) {
            Write-Error "Syntax error in $script : $($errors[0].Message)"
        } else {
            Write-Host "  [OK] Syntax clean: $script" -ForegroundColor Green
        }
    }
}
```

For POSIX shell scripts (`bash -n`):
```bash
for script in run.sh build.sh e2e.sh install.sh; do
    if [ -f "$script" ]; then
        bash -n "$script" && echo "  [OK] Syntax clean: $script"
    fi
done
```

### Gate 2: Strictly Relative Git Paths Invariant Audit
Search all newly authored and modified specification files, plans, and documentation to ensure no absolute filesystem paths were introduced:
```powershell
# Verify zero absolute drive paths in touched specs and docs
$forbiddenPaths = @("D:\", "C:\", "/home/")
# Ensure file references use repo-relative paths (02-spec/..., .ai-memory/..., scripts/...)
```
*Verification criteria*: Every path reference in documentation and comments must use repository-relative forward-slash syntax (`02-spec/...`, `.ai-memory/...`, `scripts/...`).

### Gate 3: Toolchain SQLite Schema Validation
Verify that `.ai-memory/toolchain.db` schema conforms to the specification:
```sql
CREATE TABLE IF NOT EXISTS toolchain_cache (
    component_id TEXT PRIMARY KEY,
    binary_name TEXT NOT NULL,
    detected_path TEXT NOT NULL,
    version_output TEXT NOT NULL,
    is_valid INTEGER NOT NULL DEFAULT 1,
    last_verified_at TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS toolchain_runs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    runner_name TEXT NOT NULL,
    action TEXT NOT NULL,
    status TEXT NOT NULL,
    details TEXT,
    created_at TEXT NOT NULL
);
```

### Gate 4: Zero Builds / Zero Tests Enforcement
- Verify that `cargo build`, `npm run build`, `npm run test`, and `tauri build` were NOT run during the task execution.
- Only lightweight static checks and AST parsing are permitted.

---

## 3. Plan & Ledger Completion Ceremony

Once all implementation steps (01 through 04) are verified:
1. Update `02-spec/21-app/224-repository-setup-and-installation-instructions/00-master-audit-ledger.md`:
   - Set `Status: COMPLETED`.
   - Mark all steps (01 through 05) as `[x] COMPLETED`.
2. Move pending plan:
   - Move `.ai-memory/plans/pending/224-repository-setup-and-installation-instructions.md` to `.ai-memory/plans/completed/224-repository-setup-and-installation-instructions.md`.
3. Update `.ai-memory/plans/readme.md`:
   - Add Task 224 to the "Recent Completed Tasks Register" at the top of the table.

---

## 4. Atomic GitMap Commit & Push Protocol

Stage all touched files and execute an atomic hyphen-separated commit via GitMap:

```bash
# Stage touched files
git add run.ps1 run.sh build.ps1 build.sh e2e.ps1 e2e.sh install.ps1 scripts/
git add README.md README_EN.md .ai-memory/what-to-read.md
git add 02-spec/21-app/224-repository-setup-and-installation-instructions/
git add .ai-memory/plans/

# Create atomic commit via GitMap cpf
gitmap cpf "scripts - add build.ps1 e2e.ps1 and enforce c-drive toolchain sqlite tracking"
```

### Commit Message Standards
- Must follow the hyphen-separated category convention (`scripts - <action>`).
- Must describe the final state accurately.
- Attribution must strictly attribute `@aukgit` (`(Thanks to @aukgit)`).
- Push upstream to `main` branch.

---

## 5. Summary Checklist

- [ ] All PowerShell scripts pass AST parsing with 0 errors.
- [ ] All POSIX shell scripts pass `bash -n` syntax check.
- [ ] Zero absolute paths in specifications, plans, and documentation.
- [ ] `README.md` and `README_EN.md` synchronized with Root Runner Suite section and `v4.257.2`.
- [ ] `.ai-memory/what-to-read.md` updated with Task 224 invariants.
- [ ] Master audit ledger and plan transitioned to `COMPLETED`.
- [ ] Atomic GitMap commit created and pushed cleanly.
