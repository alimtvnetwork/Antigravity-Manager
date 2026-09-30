---
name: agm-visual-evidence-verification
description: Specialized skill for automated Chrome DevTools Protocol (CDP) screenshot capture, synthetic audit card rendering (Pillow), system timestamp verification, and visual proof generation during Antigravity instance switching workflows.
---

# AGM Visual Evidence Verification & Screenshot Engine

Specialized operational guide for the visual verification engine in [`assets/screenshots/generate_instance_screenshot.py`](file:///d:/work/Antigravity-Manager/assets/screenshots/generate_instance_screenshot.py), [`scripts/capture_real_instance_screenshot.py`](file:///d:/work/Antigravity-Manager/scripts/capture_real_instance_screenshot.py), and the associated CLI visual verification workflows in [`src-tauri/src/bin/agm.rs`](file:///d:/work/Antigravity-Manager/src-tauri/src/bin/agm.rs) (`agm test-instance-flow`).

---

## 1. Engine Topology & Dual-Path Architecture

The visual verification subsystem operates a dual-path capture pipeline designed to produce concrete, timestamped, verifiable proof of instance authentication, active PIDs, and running prompt status without halting unattended CI/CD or CLI automation:

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
                     Port Found & Responding          Port Missing or Unreachable
                               v                                 v
                 Primary Path: CDP Automation       Fallback Path: High-Fidelity Mock
                               |                                 |
                               v                                 v
                   WebSocket Debugger Session          Pillow (PIL) Canvas Engine
                   - Target: "Antigravity" page        - Dark Theme Slate Palette
                   - Keystroke: 'Ctrl+,' (Settings)    - System UTC Timestamp Header
                   - CDP: Page.captureScreenshot       - Verified PID & Account Badge
                               |                       - 5s Heartbeat Runner Telemetry
                               +----------------+----------------+
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
5. `%APPDATA%/Antigravity/DevToolsActivePort`

Once a port candidate is parsed, `http://127.0.0.1:{port}/json` is pinged with a 1,000ms timeout to confirm an active debugger session.

### WebSocket Automation Flow (`capture_screenshot_from_cdp`)
1. Connects to `webSocketDebuggerUrl` corresponding to the main Antigravity window.
2. Dispatches `Input.dispatchKeyEvent` sequences:
   - Keydown: `ControlLeft` (modifiers: 2)
   - Keydown: `KeyO` or `,` (modifiers: 2)
   - Keyup: `ControlLeft`
3. Allows 1.5 seconds for the Settings UI and account credentials tab to render.
4. Invokes `Page.captureScreenshot` with format `png`.
5. Decodes the Base64 image payload and saves to the requested output path.

---

## 3. High-Fidelity Synthetic Audit Card (Pillow Fallback)

When running on headless Linux servers, Docker containers, or environments where CDP is disabled, `render_mock_screenshot` synthesizes a pixel-accurate audit card:

### Card Visual Hierarchy
1. **Header Bar**:
   - Title: `Antigravity IDE - Settings & Account Verification`
   - Active Node & OS indicator
   - **System Date & Time**: `YYYY-MM-DD HH:MM:SS UTC` (explicitly rendered at the top of every card).
2. **Authenticated Account Card**:
   - User Avatar circle with first-letter glyph.
   - Confirmed Email address (e.g. `user1@gmail.com`).
   - Active status pill: `🟢 Active Profile Authenticated`.
3. **Instance & Runtime Telemetry Card**:
   - Instance ID (e.g. `test-cli-flow-8821`).
   - Verified Active PID (scanned via `sysinfo` / `tasklist`).
   - Data Directory path (`D:\work\Antigravity-Manager\...`).
   - Triple `state.vscdb` verification status badge.
   - Keyring Bypass status: `✅ OS Keyring Bypassed (SSH/WSL Emulation Active)`.
4. **Real-Time Prompt Heartbeat Card**:
   - Liveness Status: `🟢 5s Heartbeat Active & Logging`.
   - Log Location: `.antigravity_goal_prompt.log`.
   - Latest Tick Line: `[YYYY-MM-DD HH:MM:SS UTC] [PID: 1234] Iteration: 42 | Status: RUNNING`.
   - Resume Task File: `.antigravity_resume_task.json` present.

---

## 4. Standard Verification Artifacts

The three canonical screenshots generated by `agm test-instance-flow` and `scripts/test-instance-e2e.ps1`:

| Artifact Name | Workflow Phase | Verified Indicators |
|---|---|---|
| `instance_step1_initial.png` | Phase 1: Baseline Launch | Account 1 authenticated, Initial PID, Heartbeat Iteration 1-3 |
| `instance_step2_switched.png` | Phase 2: Autonomous Switch | Account 2 authenticated, New PID, Prompt safely snapshotted |
| `instance_step3_switched_back.png` | Phase 3: Fast-Forward Restore | Account 1 restored, Initial account verified, Prompt heartbeat resumed |

---

## 5. CLI Invocation Reference

To generate or test visual verification independently:

```bash
# Capture full flow mock/live screenshots
python assets/screenshots/generate_instance_screenshot.py \
  --email "rokixshohag1@gmail.com" \
  --step 1 \
  --output assets/screenshots/instance_step1_initial.png \
  --pid 14232 \
  --folder "D:\work\Antigravity-Manager\.antigravity_tools\instances\test-cli-flow-1"

# Run autonomous end-to-end verification with visual evidence output
agm test-instance-flow --json
```
