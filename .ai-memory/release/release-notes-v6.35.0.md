## Quick Install v6.35.0

### Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.35.0/consolidated-install.ps1 -OutFile consolidated-install.ps1; .\consolidated-install.ps1
```

### Unix / macOS (Bash)
```bash
curl -sL https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.35.0/consolidated-install.sh | bash
```

---

## What's Changed in v6.35.0

### 🚀 Highlights & Fixes
- **Shell-to-Python CI Migration:** Converted `verify-codegen-determinism.sh` and `regen-codegen-fixtures.sh` into pure, cross-platform Python scripts (`verify_codegen_determinism.py` and `regen_codegen_fixtures.py`) with UTF-8 stream handling and `diff` reporting.
- **Fixture Determinism Synchronized:** Regenerated expected Go, PHP, and TypeScript inverted-field fixtures in `linters-cicd/codegen/fixtures/expected/` to match current spec paths (`03-naming-conventions.md`).
- **Prompt Specifications Updated:** Hardened prompt ecosystem (`01-prompts/08-dry-code/` and `17-ci-cd/`) to mandate cross-platform Python over `.sh` shell scripts in all CI workflows.
- **CI/CD Quality Gates Expanded:** Added `Codegen Determinism Check` into `03-ai-scripts/03-cicd-local-runner.py` (now 11 concurrent quality gates passing 100% green).
- **All Bundle Installers Updated:** Regenerated all 14 installation scripts for v6.35.0.
