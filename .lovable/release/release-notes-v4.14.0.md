## Quick Install v4.14.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.14.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.14.0/install.sh | bash
```

---

## What's Changed in v4.14.0

- **Ubuntu IDE Discovery Diagnostics**: Expanded multi-path scanner covering standard paths, Snap, Flatpak, ~/.local/bin, desktop entries, and AppImages; fixed .desktop quoting and field-code parsing.
- **Structured Error Model & Stack Trace Capture**: Introduced `AppError::IdeNotFound` (E7002, 404) with std::backtrace and audit diagnostics for 1-click troubleshooting.
- **Multi-Instance Isolation & Concurrency**: Spawns isolated user data directories with separate window contexts and macOS `open -n -a` support.
- **Cross-Platform Executable Cloning**: `clone_instance_executable` enables instances to run custom binaries or hardlinks regardless of OS.
