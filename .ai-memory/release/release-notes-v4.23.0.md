## Quick Install v4.23.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.23.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.23.0/install.sh | bash
```

---

## What's Changed in v4.23.0

- **Direct EXE Setup Installation**: Enhanced `install.ps1` to download and install setup/standalone `.exe` assets directly with silent `/S` and `/D` parameters, eliminating `.zip` extraction issues on Windows.
- **Compact Top-Level Navbar & Middle Menu Button**: Replaced sprawling 10-tab navbar with core quick-jump pills (`Dashboard`, `Accounts`, `Instances`) and a dedicated 'Menu' button with popover dropdown for secondary pages.
- **Instance Selector Boundary Protection**: Guaranteed `<InstanceSelector />` and settings controls remain 100% visible within window boundaries across all screen sizes with `shrink-0` constraints and optimized spacing.
- **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
