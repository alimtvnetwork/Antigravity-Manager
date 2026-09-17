## Quick Install v4.13.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.13.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.13.0/install.sh | bash
```

---

## What's Changed in v4.13.0

- **Ubuntu / Linux IDE Discovery & Auto-Healing**:
  - Implemented dynamic `$PATH` lookup via `which` command, standard filesystem locations, Snap paths (`/snap/bin`, `/var/lib/snapd/snap/bin`), Flatpak packages, and `.desktop` entry file parsing to locate Antigravity, Cursor, and VS Code executables.
  - Added autonomous `storage.json` synthesis with valid telemetry fingerprints (`machineId`, `macMachineId`, `devDeviceId`, `sqmId`) and automatic SQLite `ItemTable` creation, completely eliminating `storage_json_not_found` errors on fresh Ubuntu/Linux setups.
- **Structured Universal Error Management & Interactive AI Modal**:
  - Full Universal Response Envelope serialization in Rust backend (`src-tauri/src/error.rs`) conforming to standard taxonomy (`E1001`–`E9001`).
  - Interaction tracking (last 10 user clicks) and stack trace parsing in React 19 frontend.
  - Interactive diagnostic modal with one-click **"Copy Error for AI"** button generating clean, high-signal Markdown reports.
- **Root README 100% English Overhaul & Open-Source Attribution**:
  - Overhauled root `readme.md` strictly in English according to repository guidelines (0 Chinese characters).
  - Cited and praised original creator `lbjlaq/Antigravity-Manager` and all contributors.
  - Documented ongoing governance under the MD Animal Experiments series (`Md. Alim Ul Karim`, Riseup Asia LLC) and sponsorship by Rivera.
