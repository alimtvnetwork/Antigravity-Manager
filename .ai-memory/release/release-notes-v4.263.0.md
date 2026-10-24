# Antigravity Tools v4.263.0

### Quick Install

**Windows (PowerShell):**
```powershell
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.263.0/install.ps1 | iex
```

**macOS / Linux (Bash):**
```bash
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.263.0/install.sh | bash
```

### Highlights & Fixes

- **Target Instance Prompt Snapshot Scoping**: Stage 1 prompt snapshot during account rotation now scopes strictly to the target switched instance via `prompt_backup_source_instance_id` in `src-tauri/src/modules/instance/switch.rs` instead of defaulting to global active instance (Thanks to @aukgit)
- **CLI/Headless Inline Stage-5 Restore Execution**: Added `run_post_switch_resume_after_delay` in `src-tauri/src/modules/instance/post_switch_resume.rs`. Because in-process GUI timers die when the CLI process exits, CLI and headless account switching now awaits the settle delay and executes Stage 5 restore inline, guaranteeing running prompts are cleanly re-dispatched to the next account (Thanks to @aukgit)
- **Fast-Forward Duplicate Prevention**: Fast-forward leftover handling in `src-tauri/src/bin/agm/instance_fastforward.rs` only re-dispatches prompts if backed-up records remain, eliminating duplicate prompt dispatches after Stage 5 execution (Thanks to @aukgit)
- **Real Dispatched Prompt Counts**: Accurate restore and dispatch metrics correctly propagate across `src-tauri/src/modules/instance/workspace.rs` and audit logs during switch restoration (Thanks to @aukgit)
