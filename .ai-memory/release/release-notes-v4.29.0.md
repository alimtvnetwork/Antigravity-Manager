## Quick Install v4.29.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.29.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.29.0/install.sh | bash
```

---

## What's Changed in v4.29.0

- **Compact Top Spacing & Layout**: Standardized compact top padding (14-16px) across all dashboard pages, reduced window drag handle to 16px, and eliminated vertical whitespace clutter.
- **Win32 Window Unminimize Fix**: Ensured window unminimizes cleanly before show on tray click, single-instance activation, and IPC calls, preventing frozen or hidden UI states.
- **Universal Error Management Drawer & Stack Trace Viewer**: Debounced error queue badge in header, left-side slide-over drawer, and full stack trace inspection modal with one-click copy.
- **Taskbar Branding & Accounts Polish**: Integrated official application icon in Windows taskbar/PE headers, reordered account action buttons (Refresh #1, Switch #2), and defaulted landing tab to Accounts.
- **Instance Smart Double Play**: Double-click profile rotation algorithm prioritizing longest idle profiles with SQLite profiling and candidate preview.
- **100% Green CI/CD Quality Gates**: Validated across all 28 automated quality gates.
