## Quick Install v4.10.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.10.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.10.0/install.sh | bash
```

---

## What's Changed in v4.10.0

### 🚀 Highlights & Fixes
- **English Language Default**: Reconfigured application-wide initial language to English across all platforms and components.
- **Rust Test Suite & SQLite Concurrency**: Resolved 19 unit test regressions across proxy mappers, cache eviction, thinking budget calculations, and model snapshot synchronization. Added thread-safe mutex isolation and WAL pragmas for SQLite test databases.
- **Rust Formatting Standardized**: Auto-formatted all Rust source files with `cargo fmt` to eliminate CI formatting drift across Ubuntu, macOS, and Windows runners.
- **GitHub Pages Workflow Provisioned**: Configured repository GitHub Pages deployment source to workflow-driven builds via GitHub API, resolving 404 setup failures.
- **Standalone One-Liner Installers**: Hardened `install.ps1` and `install.sh` for one-liner invocation with version normalization and upstream fallback. Updated release workflow to automatically prepend Quick Install one-liners to GitHub Release notes and publish installer scripts as release assets.
