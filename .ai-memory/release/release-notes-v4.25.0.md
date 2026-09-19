## Quick Install v4.25.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.25.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.25.0/install.sh | bash
```

---

## What's Changed in v4.25.0

- **Eliminate Toolbar Rogue Z-Index**: Removed `relative z-[100]` on the Add Account trigger button in `AddAccountDialog.tsx`, fixing the UI issue where the button poked through and intercepted hover over the active instance profile menu.
- **Navbar Stacking Context Elevation**: Raised navbar sticky container to `zIndex: 60`, ensuring all top-bar menus layer cleanly over page body elements.
- **Navbar Dropdown Hardening**: Applied `z-50` and viewport width constraints across all navbar popover menus (`LanguageDropdown`, `MoreDropdown`, `InstanceSelector`, `NavMenu`).
- **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
