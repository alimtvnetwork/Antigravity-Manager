## Quick Install v4.55.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.55.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.55.0/install.sh | bash
```

---

## What's Changed in v4.55.0

- **Resolve TypeScript instance exports, rustfmt drift, and 2-min IDE crash focus recovery**: Automated release and version synchronization across all manifests.
- **CI/CD Quality Gates**: 100% green verified across all 36 automated quality gates.
