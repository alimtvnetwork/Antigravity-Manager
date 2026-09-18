## Quick Install v6.34.0

### Windows (PowerShell)
```powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.34.0/consolidated-install.ps1 -OutFile consolidated-install.ps1; .\consolidated-install.ps1
```

### Unix / macOS (Bash)
```bash
curl -sL https://raw.githubusercontent.com/alimtvnetwork/coding-guidelines-v24/v6.34.0/consolidated-install.sh | bash
```

---

## What's Changed in v6.34.0

### 🚀 Highlights & Fixes
- **Installer Syntax Fix:** Resolved JavaScript template literal syntax errors and unescaped PowerShell backtick newlines (`\`n\`n`) in `scripts/generate-bundle-installers.mjs`.
- **Pluggable Python Cache:** Implemented modular `tmp/cache/` (paths, locks, files) with safe cross-process atomic file locking and stale-lock recovery in `00-shared-engine.py`.
- **Fast AI Reader Tool:** Added `14-fast-file-reader.py` for sub-millisecond safe file reading and directory exploration.
- **CI/CD Quality Gates Expanded:** Registered `Bundle Installer Generation` and `Spec Tree Sync` into the local CI runner (`03-cicd-local-runner.py`), ensuring 10/10 automated checks pass green.
- **Prompt Specifications Hardened:** Updated prompt guidelines in `01-prompts/` to mandate DRY Python tools, top-level Enums with `*Type` suffixes, and functions under 25 lines.

---

### 📦 Artifacts & Manifests Synced
- `version.json`, `package.json`, `changelog.md`, `public/health-score.json`, `src/data/specTree.json`, and all 14 bundle installer scripts.
