## Quick Install v4.68.0

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.68.0)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version 4.68.0
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.68.0)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.68.0
```

---

-   **[Release v4.68.0: Instance Profile Duplication, Default Selection, Fast-Forward Shortcut & Security Diagnostic Fixes] Instance Clone Actions Restoration, Minimalist Single Fast-Forward Button, Configurable Ctrl+Shift+F Global Shortcut, Default Profile Locking & SQLite Null Safe Aggregates**:
    -   **Restored Instance Profile Duplication (Clone / Duplicate)**: Re-introduced the "Duplicate Active Profile" action in the `InstanceSelector.tsx` dropdown header bar next to import/export, and added an inline duplicate (`Copy`) button to every individual instance row beside the rename button. Restored full cloning capabilities on the `Instances.tsx` dashboard page.
    -   **Minimalist Single Outer Fast-Forward Button & Enlarged Sequence Badges**: Cleaned up the top navbar by removing the redundant green launch play button, keeping exclusively the Fast-Forward (`[ ⏩ ]`) button with active shortcut tooltip. Enlarged the dropdown container to `w-96 md:w-[420px]` and styled high-contrast `#1`, `#2`, `#3` sequence badges.
    -   **Default Profile Selection & Deletion Protection (`set_default_instance`)**: Added the backend IPC command `set_default_instance` to atomically mark any profile as `DEFAULT` with dedicated badge indicators. Updated `delete_instance` to strictly intercept and prevent deleting default profiles.
    -   **Configurable Fast-Forward Shortcut & Global Listener**: Introduced `fast_forward_shortcut` in auto-switcher configuration (defaulting to `Ctrl+Shift+F`), provided pre-set selection pills and custom binding in `AutoSwitcherSettings.tsx`, and wired `useFastForwardShortcut.ts` for frictionless keyboard-driven instance rotation with toast feedback.
    -   **Security Diagnostic Null Aggregate & IPC Argument Fixes**: Wrapped SQLite IP statistics queries in `COALESCE(SUM(...), 0)` and deserialized safely via `Option<u64>` to eliminate `Invalid column type Null at index: 2, name: blocked`. Updated `get_ip_access_logs` to support both nested query objects and flat fallback parameters, fixing `missing required key query`.
