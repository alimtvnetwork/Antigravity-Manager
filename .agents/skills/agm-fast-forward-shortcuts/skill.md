---
name: agm-fast-forward-shortcuts
description: Specialized skill for managing the Fast-Forward (FF) shortcut engine, multi-mode instance cloning, default profile protection, active window restore, and crash watchdogs in Antigravity-Manager.
---

# AGM Fast-Forward Shortcut & Instance Lifecycle Engine

This skill guides development, maintenance, and debugging of the Fast-Forward (`FF`) profile rotation engine, keyboard shortcut orchestration, dual-mode instance cloning, and session crash protection in Antigravity-Manager.

---

## 1. Fast-Forward (FF) Architecture & Global Shortcuts

### 1.1 Trigger Channels
Fast-Forward (`FF`) triggers an immediate, intelligent account rotation for an Antigravity IDE instance:
1. **Navbar Button**: Dedicated `[ ⏩ ]` Fast-Forward button pinned on the main top navigation bar (`src/components/navbar/InstanceSelector.tsx`).
2. **Keyboard Shortcut**: Global keybinding (default `Ctrl+Shift+F`, configured in `AutoProfileSwitcherConfig::fast_forward_shortcut`) handled via `src/hooks/useFastForwardShortcut.ts`.
3. **Inbound Remote Command**: Email or Telegram remote execution via `CMD | FF` or `CMD | FastForward` in `src-tauri/src/modules/email_inbound.rs`.
4. **Manual Settings Action**: "Rotate Now" button in `src/components/settings/AutoSwitcherSettings.tsx`.

### 1.2 Execution Pipeline (`useInstanceStore.ts -> smartRotateProfileAccount`)
When triggered:
1. **Active Lock**: Uses an atomic ref (`isRotatingRef.current`) to discard rapid repeat keystrokes.
2. **Multiplicative Candidate Scoring**:
   $$\text{Score} = S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$$
3. **Live Quota Probe**: Before activation, probes candidate with `fetch_account_quota`. If quota fails or rate-limited, drops candidate and retries.
4. **Credential Injection**: Injects updated tokens directly into `<instance_dir>/User/globalStorage/state.vscdb`.
5. **Prompt Continuity**: Scans `<instance_dir>/User/History` and recent workspace prompts (<1h window) to auto-resume active tasks.
6. **Toast Feedback**: Emits non-intrusive toast showing the switched profile name, new account email, and resumed prompt count.

---

## 2. Multi-Mode Instance Cloning (`copy_instance`)

### 2.1 Clone Modes
In `src-tauri/src/modules/instance.rs`:
- **Full Clone (`clone_mode == "full"`)**:
  - Clones the entire isolated directory tree from source to target instance.
  - Strips volatile lockfiles: `lockfile`, `*.lock`, `code.lock`, `singleton*`.
  - Sanitizes `state.vscdb` to purge stale auth tokens and avoid account cross-talk.
- **Profile-Only Clone (`clone_mode == "profile"`)**:
  - Copies user configurations only (`User/settings.json`, keybindings, snippets).
  - Skips extensions, cache, and workspace storage, keeping disk usage minimal.

---

## 3. Default Profile Protection (`set_default_instance`)

1. **Designation**: Backend IPC command `set_default_instance` sets `is_default = true` on the selected instance and clears the flag on all other instances.
2. **Deletion Guard**: Both frontend (`useInstanceStore.deleteInstance`) and backend (`modules::instance::delete_instance`) strictly forbid deleting an instance if `instance_id == "default"` or `is_default == true`.
3. **Visual Indicator**: The default instance displays a prominent star/badge in `InstanceSelector.tsx` and `Instances.tsx`.

---

## 4. Crash Watchdog & Win32 Foreground Focus

1. **Watchdog Daemon**: Background task checking active instance PIDs every 120 seconds (`watchdog_interval_seconds`).
2. **Foreground Lock Bypass**: When restoring or focusing an IDE window on Windows, uses `force_restore_and_focus_win32()` in `src-tauri/src/lib.rs` to bypass Windows Foreground Lock Timeout via `AttachThreadInput` and `SwitchToThisWindow`.
