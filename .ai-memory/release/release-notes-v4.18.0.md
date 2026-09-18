## Quick Install v4.18.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.18.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.18.0/install.sh | bash
```

---

## What's Changed in v4.18.0

- **Auto-Switch Quota Threshold (15%)**: Calibrated automatic account rotation threshold to trigger at 15% remaining credits (down from 50%).
- **Split Repo DB & Active Prompt Backup**: Dedicated SQLite storage (`repo_db.rs`) to back up and restore active prompts across workspace switches.
- **Account Rotation API & IPC Endpoints**: Implemented `/admin/rotate` HTTP endpoint and IPC commands for running projects and active prompts.
- **Hardened Ubuntu Process Termination**: Enhanced process cleanup using explicit SIGKILL on target child processes.
- **Architecture Migration & Rustfmt**: Migrated repository structure to `.ai-memory` and `02-spec`, matching 100% CI compliance.
