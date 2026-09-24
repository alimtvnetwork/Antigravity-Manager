## Quick Install v4.66.0

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.66.0)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version 4.66.0
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.66.0)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.66.0
```

---

-   **[Release v4.66.0: Canonical 5-Hour Rolling Quota Restoration, App Process Branding, Instances Numbering & Email Sample Actions] 5-Hour Rolling Quota Calculation, Instant Sample Preloading, Task Manager Brand Identity, Telegram Bot Creation Guide & Release Code Block Separation**:
    -   **Canonical 5-Hour Rolling Quota Bucket Restored**: Fixed rolling hourly quota comparator in `src-tauri/src/modules/quota.rs`. If a rolling hourly quota exists and the weekly quota is not completely exhausted (`remaining_fraction > 0.001`), the UI unconditionally selects the 5-hour rolling bucket with accurate countdowns (e.g. `4h 56m`). Weekly countdowns (e.g. `6d 9h`) are reserved strictly for genuine weekly depletion.
    -   **Accounts Rapid Multi-Click Stability & Button Accessibility**: Eliminated redundant per-card `refreshQuota` background calls in `src/pages/Accounts.tsx` to stop WebView2 reflow races. Fixed action buttons in `src/components/accounts/AccountTable.tsx` to remain permanently visible with clear interactive hover styling instead of disappearing behind `opacity-0`.
    -   **Task Manager Process Identity & Versionless Desktop Shortcuts**: Set `productName` to `"Antigravity Manager Tools"` in `src-tauri/tauri.conf.json` while maintaining executable `mainBinaryName: "agm-alim"`, ensuring Windows Task Manager lists the official title without version suffix. Verified that Start Menu and Desktop shortcuts remain clean and unversioned (`Antigravity Manager Tools.lnk`), while Windows Installed Apps preserves explicit version tracking (`Antigravity Manager Tools 4.66.0`).
    -   **Prominent Instance Sequence Badges & Quick Action**: Added `#1`, `#2`, `#3`... sequence badges to all instance cards in `src/pages/Instances.tsx` and the navigation bar `InstanceSelector.tsx`. Enlarged card spacing and padding, and added a quick `+ New Instance` shortcut button in the Instances page header.
    -   **Inbound Email CLI Prefix Stripping & Natural Language AI Interceptor**: In `src-tauri/src/commands/email.rs` and `src-tauri/src/modules/email_inbound.rs`, automatically strip shell prefixes (`powershell:`, `ps:`, `ps `, `pwsh:`, `bash:`, `sh:`, `cmd:`) defaulting directly to PowerShell on Windows. Added multi-prefix simulation interception for AI instructions (`Project:`, `Prompt:`, `AI:`, `Instruction:`, `Prompt Injection:`) to simulate receipts cleanly without PowerShell syntax errors.
    -   **Email Sample Mailbox Preloader & Quick Command Templates**: Added a direct `Load Sample Mailboxes` action to the empty mailbox state and Actions dropdown in `EmailNotificationSettings.tsx`. Modernized `MailboxExportModal.tsx` scrollbars, updated branding to "Antigravity Manager Tools", and aligned quick template buttons with clean PowerShell and lowercase `gitmap` commands.
    -   **Telegram Bot Guide & Auto-Switcher / Settings Backup**: Added step-by-step interactive setup modal for Telegram bots via `@BotFather` in `SupabaseSyncSettings.tsx` and aligned input field heights. Added Actions dropdown (JSON Export, Import, Reset to Defaults) to `AutoSwitcherSettings.tsx`. Integrated `UnifiedBackupModal` into the main Settings page toolbar, and added a Debug console button to `NavSettings.tsx`.
    -   **Release Page Split Code Blocks & Verified Pinned One-Liners**: Split Direct Latest and Pinned Version installer commands into two distinct code blocks in `readme.md`, `README_EN.md`, and release templates. Verified pinned version PowerShell execution syntax: `& ([scriptblock]::Create((irm ...))) -Version 4.66.0`.
