# Subtask 02: Storage JSON Auto-Healing & Database Candidate Path Expansion

> **Parent Plan:** [.lovable/plans/pending/20-ubuntu-ide-detection-and-readme-overhaul.md](../../pending/20-ubuntu-ide-detection-and-readme-overhaul.md)
> **Target Files:** `src-tauri/src/modules/device.rs`, `src-tauri/src/modules/db.rs`
> **Status:** Completed

---

## Scope & Purpose

Prevent `storage_json_not_found` errors on Ubuntu/Linux by expanding candidate configuration paths (including lowercase folder names, Snap, and Flatpak directories) and automatically initializing `storage.json` with valid telemetry identifiers if missing.

---

## Requirements

1. **Candidate Path Expansion in `device.rs` and `db.rs`**:
   - Add lowercase Linux folder names: `antigravity`, `antigravity-ide`, `Antigravity`, `Antigravity IDE`.
   - Add Snap paths: `~/snap/antigravity/current/.config/...`, `~/snap/code/current/.config/...`.
   - Add Flatpak paths: `~/.var/app/com.antigravity.ide/config/...`.

2. **Auto-Healing `get_storage_path`**:
   - If an existing `storage.json` is located in any candidate path, return it.
   - If no `storage.json` exists:
     - Identify the most appropriate config directory (e.g. `~/.config/Antigravity/User/globalStorage/` on Linux).
     - Create the directory hierarchy (`fs::create_dir_all`).
     - Synthesize a default `storage.json` with generated machine ID, mac machine ID, and device ID:
       ```json
       {
         "telemetry.machineId": "<hex-sha256>",
         "telemetry.macMachineId": "<hex-sha256>",
         "telemetry.devDeviceId": "<uuid>",
         "telemetry.sqmId": "{<uuid>}"
       }
       ```
     - Return the path to the newly initialized `storage.json`.

3. **Coding Guidelines**:
   - Keep functions <= 8-15 lines.
   - Implicit boolean evaluation only (Rule 1).
