## Quick Install v4.15.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.15.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.15.0/install.sh | bash
```

---

## What's Changed in v4.15.0

- **AGM by Alim Branding**: Standardized window title, Navbar title, and page titles to "AGM by Alim", and executable product name to "Anti-Gravity Tools by Alim".
- **Accounts Quota UI Compactness**: Consolidated multi-badge model quotas into unified Gemini and Claude shared buckets, streamlining vertical and horizontal density.
- **Instance Management & Auto-Rotation Guidance**: Fixed instance discovery and runtime interactions, and added clear automatic rotation documentation in Settings.
- **Attribution & Manifest Cleanup**: Removed Chinese comments from backend manifests and release tools, giving full attribution and sincere thanks to upstream repository `lbjlaq/Antigravity-Manager`.
