---
name: agm-device-profile-fingerprint
description: Specialized skill for managing synthetic hardware fingerprint generation, device profile spoofing, Windows MachineGuid registry resolution, storage.json telemetry keys, and anti-abuse machine isolation in Antigravity-Manager.
---

# AGM Device Profile & Hardware Fingerprint Architecture

This skill provides architectural guidance, telemetry key specifications, hardware ID resolution procedures, and safe modification rules for the Device Profile and Machine Fingerprint spoofing subsystem in Antigravity-Manager (`src-tauri/src/modules/device.rs`).

---

## 1. Subsystem Architecture Overview

To prevent Google anti-abuse rate limits and multi-instance collision bans, AGM maintains distinct hardware identities for each isolated IDE sandbox profile.

```
+------------------------------------------------------------------------------------+
|                         Synthetic Hardware Generation                              |
|           machine_id (SHA-256)           mac_machine_id (SHA-256)                  |
|           dev_device_id (UUID v4)        sqm_id (Windows GUID / UUID)              |
+------------------------------------------+-----------------------------------------+
                                           |
                                           v
+------------------------------------------------------------------------------------+
|                                Dual-Target Injection                               |
|   1. storage.json: "telemetry.machineId", "telemetry.macMachineId", "telemetry.sqmId"|
|   2. state.vscdb: ItemTable keys "telemetry.machineId", "telemetry.macMachineId"   |
|   3. (Windows Optional): Registry "HKLM\SOFTWARE\Microsoft\Cryptography\MachineGuid"|
+------------------------------------------------------------------------------------+
```

---

## 2. Core Telemetry Identifiers & Key Schema

| Key Name | Format | Storage Location | Generator |
|---|---|---|---|
| `telemetry.machineId` | 64-char lowercase hex (SHA-256) | `storage.json` & `state.vscdb` | `rand_hex(64)` |
| `telemetry.macMachineId` | 64-char lowercase hex (SHA-256) | `storage.json` & `state.vscdb` | `rand_hex(64)` |
| `telemetry.devDeviceId` | UUID v4 (`xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx`) | `storage.json` | `Uuid::new_v4().to_string()` |
| `telemetry.sqmId` | Upper/lower UUID enclosed in braces `{...}` | `storage.json` | `format!("{{{}}}", Uuid::new_v4()).to_uppercase()` |

---

## 3. Storage Resolution & Multi-IDE Targets

In [`src-tauri/src/modules/device.rs`](src-tauri/src/modules/device.rs), [`get_storage_path`](src-tauri/src/modules/device.rs#L18-L60) discovers `storage.json` by probing:
1. `--user-data-dir` flag from live running processes (`process::get_user_data_dir_from_process`).
2. Portable installation directory (`data/user-data/User/globalStorage/storage.json`).
3. Standard IDE roaming paths:
   - Antigravity IDE (`Antigravity IDE/User/globalStorage/storage.json`)
   - VS Code / Cursor (`Code/User/globalStorage/storage.json`, `Cursor/User/globalStorage/storage.json`)
   - Classic Antigravity (`Antigravity/User/globalStorage/storage.json`)

---

## 4. Hardware ID Resolution & Windows Registry Parity

### Baseline Preservation & Rollback
- On first execution, AGM exports `device_original.json` as an immutable baseline before altering any telemetry values.
- `restore_original_device_profile()` rolls back `storage.json` and `state.vscdb` to original machine identities.

### Windows MachineGuid Resolution
- `get_windows_machine_guid()` reads `HKEY_LOCAL_MACHINE\SOFTWARE\Microsoft\Cryptography\MachineGuid`.
- For isolated sandbox runs, AGM can randomize or virtualize this identifier, decoupling the sandboxed instance from the physical motherboard UUID.

---

## 5. Maintenance Checklist & Critical Invariants

1. **Dual Persistence Invariant**:
   Whenever a device profile is regenerated or rotated, both `storage.json` and `state.vscdb` (`ItemTable`) must be updated synchronously. Writing only to `storage.json` causes VS Code/Antigravity to overwrite the values with cached SQLite state on next boot.
2. **Process Termination Before Injection**:
   The IDE process must be fully stopped before injecting device profiles into `state.vscdb` to avoid SQLite write locks and in-memory cache clobbering.
3. **Machine ID Format Validation**:
   `machineId` and `macMachineId` must always be exact 64-character lowercase hex strings. Malformed IDs cause the telemetry service to trigger client re-registration.
