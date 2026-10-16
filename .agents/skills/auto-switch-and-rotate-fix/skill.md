---
name: auto-switch-and-rotate-fix
description: Specialized skill for managing account auto-switching, candidate scoring, profile fast-forward, instance lifecycle recovery, and CLI version downgrade/revert in Antigravity-Manager.
---

# Auto Switch and Rotate Fix Specification & Operation Guide

## Overview
This skill governs the account switching, candidate scoring, fast-forward credential swapping, instance lifecycle management, and CLI downgrade/revert facilities in Antigravity-Manager.

## Key Subsystems
1. **Auto-Switcher & Scoring Engine** (`src-tauri/src/modules/auto_switcher/`):
   - `selection.rs`: Candidate profile filtering, multi-tier quota evaluation, and profile election.
   - `scoring.rs`: Rolling 4-hour window vs 7-day weekly quota weight calculations, candidate penalty scoring, and fallback scoring.
   - `switching.rs`: Post-selection switch coordination, credential injection, and prompt backup orchestration.

2. **Instance Lifecycle & Fast-Forward** (`src-tauri/src/modules/instance/`, `src-tauri/src/modules/fast_forward/`):
   - `switch.rs`: Atomic profile swap, `state.vscdb` sqlite modification, and `storage.json` updates.
   - `launch.rs`: Headless and IDE process spawning, Windows Job Object collision avoidance, and detached process creation.
   - `lifecycle.rs`: Instance restart, verification loops, and cold-start grace periods.
   - `pid_scan.rs`: Instance data directory process discovery and conscious child process PID mapping without cross-instance leakage.

3. **CLI Updater & Downgrade Engine** (`src-tauri/src/bin/agm/update_cmds.rs`, `install.ps1`, `src-tauri/src/modules/delegate_updater/`):
   - Version argument parsing (`agm update [to] <version>`, `agm update <version>`).
   - GitHub release asset candidate probing and fallback ladder.
   - Downgrade enforcement flag passing (`-Force`, `-isPinned`) bypassing version comparison short-circuits.
