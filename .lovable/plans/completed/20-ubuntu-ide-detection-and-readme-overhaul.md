# Plan 20: Ubuntu IDE Auto-Healing, Storage JSON Repair, and Root README Overhaul

> **Version:** 1.0.0
> **Status:** Completed
> **Created:** 2026-09-17
> **Completed:** 2026-09-17
> **Scope:** Rust Backend (`src-tauri/src/modules/device.rs`, `process.rs`, `db.rs`), Root `readme.md`, Release Lifecycle

---

## Task-Specific Rule Set (3-5 Custom Rules)

1. **Rule U1 — Robust Ubuntu IDE Auto-Healing:** `device::get_storage_path` and `db::get_db_path` MUST NOT fail with `storage_json_not_found` when running on Ubuntu/Linux. If `storage.json` or its enclosing directory does not exist, the backend MUST auto-detect the configuration location, auto-create required directories (`User/globalStorage`), and synthesize a valid initial `storage.json` populated with valid telemetry identities.
2. **Rule U2 — Comprehensive Linux Process Discovery:** `process::check_standard_locations` on Linux MUST scan `$PATH` via `which`, check standard deb/snap/flatpak installation paths, and parse `.desktop` entry files to reliably locate `antigravity`, `antigravity-ide`, `cursor`, and `code` executables on Ubuntu.
3. **Rule U3 — English-Only Standard Root README:** The root `readme.md` MUST use 100% English text, adhere to `02-spec/01-spec-authoring-guide/13-root-readme-conventions.md`, cite and praise the original authors (`lbjlaq/Antigravity-Manager`), document management under the MD Animal Experiments series, and cite Rivera sponsorship.
4. **Rule U4 — Strict Relative Git Paths & Lowercase Naming:** All documentation, plans, subtasks, and files MUST use strictly lowercase filenames (Rule 4) and relative paths starting from the git root (Rule 5).
5. **Rule U5 — Coding Guidelines & Micro-Tasking:** Functions MUST be kept <= 8-15 lines, use implicit booleans (Rule 1), and avoid explicit boolean true checks or mixed polarity.

---

## Architectural Breakdown

### 1. Ubuntu / Linux IDE Detection (`src-tauri/src/modules/process.rs`)
- Expand Linux executable candidate paths:
  - System PATH lookups via `which antigravity`, `which antigravity-ide`, `which cursor`, `which code`.
  - Additional directories: `/usr/local/bin/`, `/snap/bin/`, `/var/lib/snapd/snap/bin/`, `/opt/antigravity/`, `/opt/antigravity-ide/`, `~/.local/share/antigravity/`.
  - Desktop file parsing: inspect `/usr/share/applications/` and `~/.local/share/applications/` for `Exec=` entries.

### 2. Storage JSON & Database Auto-Healing (`src-tauri/src/modules/device.rs` & `src-tauri/src/modules/db.rs`)
- Expand Linux candidate config directories:
  - `~/.config/Antigravity/User/globalStorage/`
  - `~/.config/antigravity/User/globalStorage/`
  - `~/.config/Antigravity IDE/User/globalStorage/`
  - `~/.config/antigravity-ide/User/globalStorage/`
  - `~/snap/antigravity/current/.config/Antigravity/User/globalStorage/`
  - `~/.var/app/com.antigravity.ide/config/Antigravity/User/globalStorage/`
- Implement auto-healing in `get_storage_path`:
  - If existing `storage.json` is found, return it.
  - If not found, select best candidate directory, create parent directories if missing, and initialize default `storage.json` with fresh telemetry IDs (`telemetry.machineId`, `telemetry.macMachineId`, `telemetry.devDeviceId`, `telemetry.sqmId`).
- Update `db::get_all_candidate_db_paths` to include all case variations and Snap/Flatpak paths.

### 3. Root README English Overhaul (`readme.md`)
- Overhaul `readme.md` strictly in English.
- Hero block: Brand icon, title, tagline, badge matrices, author block (`Md. Alim Ul Karim`, Riseup Asia LLC).
- Animal Experiments header section and Rivera sponsorship notice.
- Prominent citation and praise for original creators (`lbjlaq/Antigravity-Manager`), clearly stating this fork builds on their foundational work.
- Screenshot gallery, quick installation one-liners, features matrix, architecture, and documentation cross-references.

### 4. Release Ceremony & Version Sync (`v4.13.0`)
- Bump version to `4.13.0` across `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`.
- Update `CHANGELOG.md` with detailed release notes.
- Tag and trigger release.

---

## Subtask Decomposition (All Completed)

1. [Subtask 01](.lovable/plans/subtasks/20-ubuntu-ide-detection-and-readme-overhaul/01-linux-ide-detection-and-path-lookup.md) — Completed
2. [Subtask 02](.lovable/plans/subtasks/20-ubuntu-ide-detection-and-readme-overhaul/02-storage-json-auto-healing.md) — Completed
3. [Subtask 03](.lovable/plans/subtasks/20-ubuntu-ide-detection-and-readme-overhaul/03-root-readme-english-overhaul.md) — Completed
4. [Subtask 04](.lovable/plans/subtasks/20-ubuntu-ide-detection-and-readme-overhaul/04-release-v4-13-0-and-version-sync.md) — Completed
