## Quick Install v4.26.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.26.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.26.0/install.sh | bash
```

---

## What's Changed in v4.26.0

- **In-Place Instance Rename**: Added `rename_instance` Tauri IPC command and frontend service to rename any profile without losing data.
- **Navbar Quick Rename**: Added quick edit button and dropdown inline rename controls in `InstanceSelector.tsx` for seamless profile management.
- **Instances Management Page**: Added profile rename modal and card action button in `Instances.tsx`.
- **Full Trilingual Localization**: Added complete translations across English (`en`), Simplified Chinese (`zh`), and Traditional Chinese (`zh-TW`).
- **Verified 100% Green Quality Gates**: Validated all repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
