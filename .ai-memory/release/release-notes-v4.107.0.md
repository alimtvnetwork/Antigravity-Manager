## Quick Install v4.107.0

### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.107.0/install.ps1 -OutFile install.ps1; .\install.ps1 -TargetDir ".ai-memory/prompts" -Version "v4.107.0"
```

### Unix / Linux / macOS (Bash)

```bash
curl -sL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.107.0/install.sh | bash -s -- ".ai-memory/prompts" "v4.107.0"
```

---

## What's Changed in v4.107.0

### Added
- exact rustfmt compliance and gitignore agm command
