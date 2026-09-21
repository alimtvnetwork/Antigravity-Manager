---
name: agm-multi-instance-sandboxing
description: Specialized skill for managing, provisioning, and debugging multi-instance Antigravity IDE sandboxes, directory isolation (--user-data-dir), process scoring, and instance rotation in Antigravity-Manager.
---

# AGM Multi-Instance Sandboxing & Process Management

This skill guides engineering work on the multi-instance orchestration layer of Antigravity-Manager. It enables users to launch, isolate, monitor, and rotate multiple independent Antigravity IDE (or VS Code / Cursor) processes concurrently without state or credential collisions.

## Key Source Files

- `src-tauri/src/modules/instance.rs` — Directory provisioning, registry management (`instances.json`), status detection, and process launching.
- `src-tauri/src/modules/process.rs` — Cross-platform process enumeration, window handle detection, and process termination.
- `src-tauri/src/commands/instance.rs` — Tauri IPC commands bridging frontend actions to backend instance logic.
- `src/stores/useInstanceStore.ts` — React Zustand store managing instance states, active selection, and health polling.
- `src/pages/Instances.tsx` — Management UI for creating profiles, launching instances, and viewing live metrics.

## Core Directives & Patterns

### 1. Sandbox Directory Isolation
Every instance receives a dedicated, sandboxed profile directory:
- Base Directory: `<data_dir>/instances/<profile_name>/`
- User Data: `--user-data-dir <data_dir>/instances/<profile_name>/data`
- Extensions: `--extensions-dir <data_dir>/instances/<profile_name>/extensions`
- Dedicated IPC / debug ports to prevent TCP port contention.

### 2. Candidate Scoring & Smart Rotation
When dispatching incoming tasks or prompts across instances:
- Instances are scored using process health, memory consumption, active open files, and recent response latency.
- Healthy idle instances receive highest priority.
- If an instance is unresponsive or degraded, the rotation engine initiates clean process teardown, releases socket bindings, and starts a fresh instance.

### 3. Cross-Platform Executable Resolution
Executable discovery inspects platform-specific default paths:
- Windows: `%LOCALAPPDATA%\Programs\Antigravity\Antigravity.exe` or `%APPDATA%\Antigravity`
- macOS: `/Applications/Antigravity.app/Contents/MacOS/Antigravity`
- Linux: `/usr/bin/antigravity` or custom user configurations in `AppConfig`.

### 4. Process Teardown Safety
- Always issue graceful termination signals before escalating to forceful process kills (`SIGKILL` / `TerminateProcess`).
- Ensure child process trees and background language server daemons spawned by the IDE are cleaned up to prevent orphaned zombie processes.
