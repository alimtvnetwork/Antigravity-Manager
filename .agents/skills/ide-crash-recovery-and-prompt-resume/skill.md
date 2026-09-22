---
name: ide-crash-recovery-and-prompt-resume
description: Autonomous 2-minute IDE crash watchdog, PID foreground focus management, fast-forward account rotation failover, and recent project active prompt auto-resume (<1 hour window).
---

# IDE Crash Recovery & Recent Prompt Auto-Resume Skill

This skill governs the autonomous recovery pipeline when an Antigravity IDE instance crashes, disappears, or loses focus:

1. **Watchdog Verification (2 Minutes):**
   - Polls active instance PID every 120 seconds.
   - If PID is running, verifies window visibility and focus.
   - If PID has crashed or disappeared, initiates fast-forward profile rotation.

2. **Fast-Forward Multiplicative Rotation:**
   - Evaluates eligible accounts using $S_{\text{active}} \times M_{\text{tier}} \times Q_{\text{weekly}}$.
   - Runs pre-activation live quota verification loop.
   - Switches account and relaunches Antigravity with clean lockfiles.

3. **Active Prompt Auto-Resume Engine:**
   - Filters projects executed strictly within the last 1 hour (< 3600 seconds).
   - Extracts last executed prompt text and image attachments.
   - Auto-dispatches to workspace sessions to restore the developer's boot process.
