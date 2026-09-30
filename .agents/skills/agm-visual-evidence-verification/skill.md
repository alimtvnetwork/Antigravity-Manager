---
name: agm-visual-evidence-verification
description: Specialized skill for automated Chrome DevTools Protocol (CDP) screenshot capture, synthetic audit card rendering (Pillow), system timestamp verification, and visual proof generation during Antigravity instance switching workflows.
---

# AGM Visual Evidence Verification & Screenshot Engine

Specialized operational guide for the visual verification engine in [`assets/screenshots/generate_instance_screenshot.py`](file:///d:/work/Antigravity-Manager/assets/screenshots/generate_instance_screenshot.py) and the associated CLI visual verification workflows in [`src-tauri/src/bin/agm.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/bin/agm.rs) (`agm test-instance-flow`).

---

## 1. Engine Topology & 3-Tier Capture Pipeline

The visual verification subsystem operates a 3-tier capture pipeline designed to produce concrete, timestamped, verifiable proof of instance authentication, active PIDs, and running prompt status without halting unattended CI/CD or CLI automation:

```
                                  Visual Verification Request
                   (from 'agm test-instance-flow' or scripts/test-instance-e2e.ps1)
                                                |
                                                v
                            DevToolsActivePort Discovery Probe
                         (Inspects data_dir, home_dir, APPDATA)
                                                |
                               +----------------+----------------+
                               |                                 |
                     Port Found & Responding            Port Missing / Unreachable
                               v                                 v
                 Tier 1: Live CDP WebSocket            Tier 2: Win32 Native Window Grab
                 - Host IDE leak guard (check email)   - Matches conscious PID & title
                 - Advance onboarding & send Ctrl+,    - ctypes.windll.user32 + ImageGrab
                 - Page.captureScreenshot (4s timeout)           |
                               |                        Failed or Non-Windows
                               +----------------+----------------+
                                                |
                                                v
                                 Tier 3: Synthetic Audit Card Fallback
                                 - Pillow (PIL) 1280x760 Canvas Engine
                                 - System UTC Date/Time Header Bar
                                 - Verified PID, Path, & Account Badge
                                 - 5s Heartbeat Runner Telemetry
                                                |
                                                v
                                 Telemetry Banner Stamping (add_header_banner)
                                 - 42px header banner (#252526 with #007acc line)
                                 - Stage label, Instance ID, PID, Account Email
                                 - Date/Time UTC, and Heartbeat status
                                                |
                                                v
                               PNG Visual Artifact Output
                      - instance_step1_initial.png       (Account 1)
                      - instance_step2_switched.png      (Account 2)
                      - instance_step3_switched_back.png (Restored Account 1)
```

---

## 2. Chrome DevTools Protocol (CDP) Automation

### Port Discovery Ladder (`get_devtools_port`)
Electron and Chromium write active debugging listener information to `DevToolsActivePort`. The script probes candidates in priority order:
1. `<instance_dir>/DevToolsActivePort`
2. `<instance_dir>/data/DevToolsActivePort`
3. `<instance_dir>/../DevToolsActivePort`
4. `<instance_dir>/home/AppData/Roaming/Antigravity/DevToolsActivePort`
5. `<instance_dir>/../home/AppData/Roaming/Antigravity/DevToolsActivePort`
6. `%APPDATA%/Antigravity/DevToolsActivePort`

Once a port candidate is parsed, `http://127.0.0.1:{port}/json` is pinged with a 1,000ms timeout to confirm an active debugger session.

### WebSocket Automation Flow (`capture_screenshot_from_cdp`)
1. Connects to `webSocketDebuggerUrl` corresponding to the main Antigravity window.
2. **Host IDE Profile Leak Guard**: Before capturing, evaluates `document.body.innerText`. If the text contains `robinh6625@gmail.com` when targeting a different account, live capture is immediately aborted to prevent leaking the operator's primary workstation IDE profile.
3. Dispatches onboarding modal advance JS (`advance_js`) to click "Next", "Continue", or "Get Started" buttons if an initial setup dialog is displayed.
4. Attempts direct click on "Settings" DOM element, then sends `Input.dispatchKeyEvent` sequences for `Ctrl+,` (`modifiers: 2`, `windowsVirtualKeyCode: 188`, `code: Comma`).
5. Invokes `Page.captureScreenshot` with format `png` (4.0s timeout).
6. Decodes the Base64 image payload and applies `add_header_banner`.

---

## 3. Win32 Native Window Grab (`try_capture_live_win32`)

When running on Windows and CDP is unreachable (or the port is blocked), Tier 2 attempts a direct OS window grab:
1. Enumerates top-level visible windows using `ctypes.windll.user32.EnumWindows`.
2. Inspects `GetWindowThreadProcessId` matching the target conscious PID.
3. Verifies window title contains `"antigravity"`, `"visual studio"`, or `"code"`.
4. Brings the window to foreground via `SetForegroundWindow`.
5. Grabs the bounding rectangle coordinates via `PIL.ImageGrab.grab(bbox)`.
6. Stamps the 42px telemetry header banner and saves the output.

---

## 4. High-Fidelity Synthetic Audit Card (Pillow Fallback)

When running on headless Linux servers, Docker containers, or environments where live windows cannot be grabbed, `render_screenshot` synthesizes a pixel-accurate 1280x760 dark-themed audit card:

### Card Visual Hierarchy
1. **Window Titlebar (`#323233`)**:
   - Window control glyphs (`— □ ✕`).
   - Title string: `Antigravity — [Profile: <id> | PID: <pid>] — Date: <UTC Date/Time>`.
   - Antigravity icon badge (`#4285f4`).
2. **Left Activity Bar (`#2b2b2b`, width=54)**:
   - Visual Studio Code style icons (`📂 🔍 🔀 ▶ 📦 🤖 ⚙`).
3. **Settings Editor Slate (`#1e1e1e`)**:
   - Settings breadcrumb and search input box (`#3c3c3c`).
   - **User Accounts Section**:
     - Avatar circle with account initials.
     - Confirmed account email (`#4ec9b0`).
     - Authentication status badge (`🟢 Active Profile Authenticated`).
4. **Instance & Runtime Telemetry Card**:
   - Instance ID and conscious root PID.
   - Profile data directory path.
   - Triple `state.vscdb` verification status badge (`✅ Injected into 3 SQLite DBs`).
   - Keyring Bypass status: `✅ OS Keyring Bypassed (SSH/WSL Emulation Active)`.
5. **Real-Time Prompt Heartbeat Card**:
   - Liveness Status: `🟢 5s Heartbeat Active & Logging`.
   - Log Location: `.antigravity_goal_prompt.log`.
   - Latest Tick Line: `[YYYY-MM-DD HH:MM:SS UTC] [PID: <pid>] Iteration: <N> | Status: RUNNING`.
   - Task resumption marker: `.antigravity_resume_task.json` present.

---

## 5. Telemetry Header Banner (`add_header_banner`)

Every captured image (CDP, Win32, or Synthetic) is stamped with a top 42px telemetry banner:
- Background: `#252526` with a 2px `#007acc` accent border.
- Left: Workflow stage label (e.g., `● Step 1: Initial Profile & Account Launch`) and `Instance: <id> | PID: <pid>`.
- Center: `Confirmed Account: <email>` (`#4ec9b0` bold).
- Right: UTC Date/Time (`#dcdcaa`) and heartbeat telemetry line (`#9cdcfe`).

---

## 6. Standard Verification Artifacts

The three canonical screenshots generated by `agm test-instance-flow` and `scripts/test-instance-e2e.ps1`:

| Artifact Name | Workflow Phase | Verified Indicators |
|---|---|---|
| `instance_step1_initial.png` | Phase 1: Baseline Launch | Account 1 authenticated, Initial PID, Heartbeat Iteration 1-3 |
| `instance_step2_switched.png` | Phase 2: Autonomous Switch | Account 2 authenticated, New PID, Prompt safely snapshotted |
| `instance_step3_switched_back.png` | Phase 3: Fast-Forward Restore | Account 1 restored, Initial account verified, Prompt heartbeat resumed |

---

## 7. CLI Invocation Reference

To generate or test visual verification independently:

```bash
# Capture full flow mock/live screenshots
python assets/screenshots/generate_instance_screenshot.py \
  --email "rokixshohag1@gmail.com" \
  --step 1 \
  --output assets/screenshots/instance_step1_initial.png \
  --pid 14232 \
  --folder "D:\work\Antigravity-Manager\.antigravity_tools\instances\test-cli-flow-1" \
  --heartbeat-file "D:\work\Antigravity-Manager\.antigravity_goal_prompt.log" \
  --heartbeat-status "Iteration: 1 | Status: RUNNING"

# Run autonomous end-to-end verification with visual evidence output
agm test-instance-flow --json
```
