## Quick Install v4.24.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.24.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.24.0/install.sh | bash
```

---

## What's Changed in v4.24.0

- **Dedicated Email Setup Page**: Added `/email` route and standalone `Email.tsx` view, directly surfaced in top-level navigation as `Email & Alerts`.
- **Navbar Dropdown Boundary Protection**: Constrained all navigation and instance selector popovers with `max-w-[calc(100vw-32px)]` to prevent clipping beyond screen boundaries.
- **Dynamic Release Asset Discovery**: Hardened `install.ps1` to query `/releases` and dynamically select releases with verified uploaded assets, avoiding stale release downloads.
- **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
