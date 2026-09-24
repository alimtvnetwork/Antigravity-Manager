# Spec 25: Instance Lifecycle & Delete Fix, Auto-Switcher 300s/15%/12% Dynamic Polling, Navbar Consolidation, Focus Guard, and Subject-Driven Plaintext Email Engine

- **Status:** active
- **Version:** 4.70.0
- **Author:** Antigravity Orchestrator

---

## User Request (Verbatim)

```text
https://prnt.sc/2IgVGqVvdOAo
https://prnt.sc/ydgq-Y5feY5i
https://prnt.sc/3PFE9zj0cykl
https://prnt.sc/F_11aCEaoMCb
https://prnt.sc/ncs3mfil2FAk
https://prnt.sc/GSP_AluULzuX
https://prnt.sc/FDOMrN6T63P9
https://prnt.sc/dDUKpPnBaJ37
https://prnt.sc/mDxMODvqzp-h
https://prnt.sc/VjAqwmFIFY_b

Okay. So far, I have lots of observation. Let's start with the important ones. First, the instance section, the delete button does not work, so you need to fix it immediately. The delete button is a chaotic situation, okay? And it does not work at all. Okay, so that actually gives a error trace. I am adding the error trace. And also the default or the, let's say, instance creation, has a serious issue. I'm going to explain that. The issue with the instance creation is that once the instance is created, it cannot manage itself. So it just closes and open and closes and open. So that's a terrible thing to look at, okay? So in this case, what I want from you is to test it. Test it here with end-to-end testing locally. You can run it, try to create a instance, delete it. Okay? And also, you should have these, let's say, command line commands to do that. And at the end, you show me these command line commands. Okay? Also, another point which I have, let's say, reported several times. The Anti-Gravity release page combines the installation several times, which I asked you several times not to do, and you're still doing it. Does not make any sense. Okay? There are other errors, which I will give you the screenshots and things, okay, which basically you can look at. The next problem that we have is that the account switch, auto switch section, we have the settings, right? So it should automatically check in every-- So by default, it should automatically check in every 300 milliseconds. That means five minutes, right? Only if it goes below the, let's say, 12% or 10%, then it will check every... Sorry, under 15%, it will check every 60 seconds. Under, let's say, 12%, it will check every 40 seconds. Okay? So basically, once it goes below 15%, so put that as a default, it should automatically switch to the next workspace and also push the current running prompt to send now to that project repo. Okay? So that is kind of must. We have been discussing with this several times, and you are not fixing it. It's not polite. I want you to test this. Probably what you could do is put the threshold to 90% to testing, and then you do this here in this machine. Do everything you want. No worries on this, okay? Because this is snapshot VM I could restore. So don't take any problems, but you commit. Every code you write, you must commit as a group so that before testing, we must commit. Because otherwise, any crash could be fatal. The code can be lost. Remember that. Also, here in the image, you can see Gemini Pro is primary evaluated model. No. The default one would be Gemini Flash-8 High. That would be the default model, which I will give you the screenshot as well. So these are on top of my head, these are really creating big issues. The account switching is not working. Also, there are issues if I click on the accounts and click on the instances. It just looks like colliding with each other, where the other dropdown should be closed automatically. Okay? That's kind of the standard feeling which I do not get here. I think you need to fix that. So if we go inside now, and right-hand side, we have lots of button. I don't find a reason to have this debug button where we could move all the debug things to the left-hand icon. Again, I'm going to give you the screenshot, so you look into all these screenshots, okay, so that nothing remains unsolved. All right. And also the theme color and language, both of those can be on a dropdown, okay, rather than two buttons. Only the recycle button could be there. That's all right. I appreciate that. And also here, the import/export should be combined to one dropdown button that is icon. And when I hover over, I should be seeing the icon tooltip, which I do not see. I think you have integrated, but the tooltip goes below the other stuff, which is also very wrong. You do not do the proper designing and color stuff, which is absolutely bad. You need to focus on this. Now, if we go into the email section, the email formatting and the dispatch, as I mentioned, these are not yet fixed The PowerShell section is also not improved. Okay. And also the Telegram section also has some alignment issues. Okay. And also I want you to create a Telegram that's a bot, step by step using PowerShell. So Telegram account is integrated here in desktop. You could do end-to-end testing to make sure that you can create the bot and things like that. Okay. Look into all the image so that you understand the image, make sure of that. And also Telegram needs to be connected in order to check how this will work. For example, the interactive remote control section, you don't need to pass PowerShell clone to the terminal. That would actually give an error. Okay, so you just pass the command to PowerShell and that would work. Currently, in your case, it is not happening and creating more issues. I hope you understand and respect that. So here, one important factor is the account switching that needs to be happening automatically and email needs to be sent when this happens. These are not happening yet. You need to send me all the sample email that will contain all this email formatting. How I can interact with this email. So this needs to be there, which you didn't do it yet. Okay. In your formatting, it is not there, which is also very bad. Disrespectful. Okay, one more thing. Sometimes when you're working with, let's say, Windows Explorer or other place, it immediately switch backs to Antigravity IDE. Why it happens, find the root cause, and don't do it. Okay? Probably you do it because I said sometimes Antigravity becomes blank. I think this is why you are doing it. But I think just switching to that does not solve the problem. Okay? So you could try to find the root cause of it in the future. Okay? But not this way. And I want you to write this into the documents. Okay, so these are the factors I think you need to work on. Okay, now I received the emails as a format, which is also very terrible, as I mentioned several times. In your email, there is no need to have any HTML formatting nicely. You just put the text that I can understand and reply back. Okay? So most of the things would be done in the subject. Only the prompt section would be sent into the body. Okay? So I think we have discussed this in other prompts. You can look into your specs properly. I hope it's clear, right? If you have any question, confusion, let me know.
```

---

## Visual Evidence & Captured Screenshots

1. `![Instance Delete Button & Toolbar](../../assets/screenshots/prnt-2IgVGqVvdOAo.png)`
2. `![GitHub Release Combined Install Block](../../assets/screenshots/prnt-ydgq-Y5feY5i.png)`
3. `![Settings Proxy View](../../assets/screenshots/prnt-3PFE9zj0cykl.png)`
4. `![Navbar Dropdown Collision](../../assets/screenshots/prnt-F_11aCEaoMCb.png)`
5. `![Navbar Debug Icon on Right](../../assets/screenshots/prnt-ncs3mfil2FAk.png)`
6. `![Accounts Toolbar Import/Export & Row Tooltips](../../assets/screenshots/prnt-GSP_AluULzuX.png)`
7. `![Telegram Bot Input Alignment](../../assets/screenshots/prnt-FDOMrN6T63P9.png)`
8. `![Interactive Remote Command PowerShell Prefix](../../assets/screenshots/prnt-dDUKpPnBaJ37.png)`
9. `![Developer Task Quick Dispatch Dropdown](../../assets/screenshots/prnt-mDxMODvqzp-h.png)`
10. `![Raw HTML Tags in Plaintext Email](../../assets/screenshots/prnt-VjAqwmFIFY_b.png)`
11. `![Auto-Switcher Settings Current State](../../assets/screenshots/v470-obs-04.png)`
12. `![Primary Evaluated Model Target](../../assets/screenshots/v470-obs-05.png)`

---

## Architectural Specifications

### 1. Instance Delete & Open/Close Loop Remediation (`Task-01`)
- **Delete Resilience (`delete_instance`)**:
  - Before removing an instance from `instances.json` and `instances.db`, check if it is currently running (`is_instance_running(pid)` or matching `--user-data-dir`). If running, invoke `close_instance(&instance_id)` and allow 500ms for OS file handles (`state.vscdb`, `Cookies`, `lockfile`) to close.
  - If `active_instance_id == instance_id`, reset `active_instance_id` to `"default"` (or the first remaining instance).
  - Perform directory removal with fallback retry (`remove_dir_all_resilient`): even if a stubborn OS lock remains on a child file, remove unlocked files and always purge the instance record from `instances.json` and `instances.db` so the UI never errors out or gets stuck.
- **Stop Open/Close Loop (`launch_instance` & `auto_switcher.rs`)**:
  - When an Electron/Chromium IDE (`Antigravity.exe`) launches with `--user-data-dir=<path>`, the initial parent launcher process often spawns the main window process and exits within 1–3 seconds, or reuses an existing process.
  - `refresh_running_statuses` and the crash watchdog in `auto_switcher.rs` MUST detect running instances by scanning process command lines (`--user-data-dir=<instance_dir>`) in addition to the initial spawned PID, updating the stored `pid` to the live process holding that `--user-data-dir`.
  - Disable automatic crash watchdog re-launching of instances unless the instance was explicitly marked `user_launched = true` and had a verified stable uptime > 30s, preventing newly created or closed instances from entering an infinite open/close loop.

### 2. GitHub Release Page Split Installation Blocks (`Task-02`)
- In `.github/workflows/release.yml`, `03-ai-scripts/29-release-orchestrator.py`, and `.ai-memory/release/release-notes-v4.69.0.md`:
  - Render **4 distinct fenced code blocks**:
    1. **Windows (PowerShell 5.1+) — Direct Latest**:
       ```powershell
       irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
       ```
    2. **Windows (PowerShell 5.1+) — Pinned Version**:
       ```powershell
       irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/vX.Y.Z/install.ps1 | iex
       ```
    3. **Linux / macOS (Bash) — Direct Latest**:
       ```bash
       curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
       ```
    4. **Linux / macOS (Bash) — Pinned Version**:
       ```bash
       curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/vX.Y.Z/install.sh | bash
       ```
  - Immediately patch the live `v4.69.0` release on GitHub via `gh release edit v4.69.0`.

### 3. Auto-Switcher Dynamic Polling, `Gemini 3.8 Flash High` Default, Auto Workspace Switch & Email Notification (`Task-03`)
- **Defaults & Adaptive Polling**:
  - `check_interval_seconds` default: `300` (5 minutes).
  - `low_quota_threshold_percent` default: `15.0` (15%).
  - Caution Polling (`< 15%` credits): `60` seconds.
  - Critical Polling (`<= 12%` credits): `40` seconds.
  - Primary Evaluated Model (`primary_model`): Default `gemini-3.8-flash-high` (`Gemini 3.8 Flash High (Primary, Recommended)`). Automatically migrate legacy `"Gemini Pro"` / `"gemini-pro"` config values to `"gemini-3.8-flash-high"`.
- **Automatic Switch & Active Prompt Dispatch**:
  - When evaluated model quota drops below `low_quota_threshold_percent` (15% default, testable at 90%), automatically rotate to the highest-scoring candidate account/workspace, push the current running prompt to the target project repository (`resume_recent_project_prompts` / `send_now`), and dispatch a plaintext email alert via `email_sender::send_account_switch_notification`.

### 4. Navbar Dropdown Mutual Exclusion & Top-Right Toolbar Consolidation (`Task-04`)
- **Mutual Exclusion**:
  - Broadcast custom DOM event `agm:dropdown-open` with `{ id: string }` whenever `NavMenu`, `InstanceSelector`, `NavSettings` (Theme/Language), or `ImportExportDropdown` opens. All other dropdown listeners immediately set `isOpen = false`.
- **Left-Side Debug Icon**:
  - Move the Debug Console toggle button (`Bug` icon) from `NavSettings.tsx` (right side) to `Navbar.tsx` left brand section (right next to the version/error pill).
- **Right-Side Combined Theme + Language Dropdown**:
  - Replace the separate Sun/Moon button and `EN` button in `NavSettings.tsx` with a single unified Appearance & Language dropdown button, leaving only `[Recycle (Quick Clean)]` + `[Theme & Language Dropdown]` + Window Controls (`—`, `❐`, `✕`).
- **Combined Import/Export Icon Dropdown & High Z-Index Tooltips**:
  - Combine separate `Import` and `Export` buttons in `InstanceSelector.tsx`, `Instances.tsx`, and `Accounts.tsx` into a single icon dropdown button (`Import / Export`) with high `z-[9999]` tooltips that never render behind sibling containers.

### 5. Unwanted Window Focus Stealing Root Cause Fix (`Task-05`)
- **Root Cause**:
  - Background auto-switcher and window-restored handlers invoked `force_restore_and_focus_win32` / `SetForegroundWindow` / `SwitchToThisWindow` or spawned `antigravity.exe` without `--background` / minimized state when `auto_focus_window` was `true`.
- **Fix**:
  - Set `auto_focus_window` default to `false` in `src-tauri/src/models/config.rs` and `AutoSwitcherSettings.tsx`.
  - Remove `force_restore_and_focus_win32` from automatic timers/watchdogs; only invoke foreground focus when the user explicitly clicks the system tray icon or triggers a manual window restore.

### 6. Plaintext Subject-Driven Email Engine, Direct PowerShell Execution & Telegram Integration (`Task-06`)
- **100% Plaintext Subject-Driven Emails**:
  - Strip all HTML `<div>`, `<h2>`, `<p>`, `<pre>` tags from `commands/email.rs`, `email_inbound.rs`, and `email_sender.rs`.
  - Commands and metadata belong in the **Subject** line (e.g., `Subject: CMD | STATUS`, `Subject: CMD | PS | Get-Process`, `Subject: PROMPT | Antigravity-Manager`).
  - The **Body** contains only pure plaintext (for `PROMPT`, only the prompt text + plaintext reply cheatsheet).
- **Direct PowerShell Execution**:
  - Remove `powershell:`, `ps:`, `cmd:`, `prompt:` prefixes from UI templates and Quick Dispatch values so PowerShell commands (`Get-Process | Select-Object -First 5`) run directly in `powershell.exe -NoProfile -NonInteractive -Command`.
- **Telegram Alignment & PowerShell Bot Script**:
  - Align `Telegram Bot Token` and `Allowed Chat ID (Numeric)` inputs horizontally in `EmailNotificationSettings.tsx`.
  - Provide `03-ai-scripts/telegram-bot-helper.ps1` for step-by-step Telegram bot verification, `getUpdates` Chat ID discovery, and test alert transmission.
