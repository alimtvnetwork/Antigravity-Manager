## Quick Install v4.104.0

### Windows (PowerShell)

```powershell
Invoke-WebRequest -Uri https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.104.0/install.ps1 -OutFile install.ps1; .\install.ps1 -TargetDir ".ai-memory/prompts" -Version "v4.104.0"
```

### Unix / Linux / macOS (Bash)

```bash
curl -sL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.104.0/install.sh | bash -s -- ".ai-memory/prompts" "v4.104.0"
```

---

## What's Changed in v4.104.0

### Added
- enforce relative JSON envelope commands, root workDir/repoDir variables, and relative path resolution
