## Quick Install v4.12.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.12.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.12.0/install.sh | bash
```

---

## What's Changed in v4.12.0

- **Fork Independence & URL Migration**: Full decoupling from upstream repository. All updater endpoints, installer scripts (`install.ps1`, `install.sh`), and cask recipes now directly target `alimtvnetwork/Antigravity-Manager`.
- **Gitmap Pipeline DB Central Isolation**: Re-routed Gitmap pipeline database to CLI binary data directory (`pipeline/`), preventing repo workspace pollution.
- **Settings & Web Portal Realignment**: Updated documentation, issue links, and UI cards.
