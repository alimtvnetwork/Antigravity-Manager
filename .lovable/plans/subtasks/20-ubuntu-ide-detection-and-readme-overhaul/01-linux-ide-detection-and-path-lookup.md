# Subtask 01: Linux IDE Executable Detection and PATH Lookup

> **Parent Plan:** [.lovable/plans/pending/20-ubuntu-ide-detection-and-readme-overhaul.md](../../pending/20-ubuntu-ide-detection-and-readme-overhaul.md)
> **Target File:** `src-tauri/src/modules/process.rs`
> **Status:** Completed

---

## Scope & Purpose

Upgrade Linux IDE executable detection in `src-tauri/src/modules/process.rs` to comprehensively discover Antigravity, Cursor, and VS Code installations on Ubuntu and other Linux distributions.

---

## Requirements

1. **Dynamic PATH Resolution (`which`)**:
   - Query system PATH using `which <name>` for candidate binaries (`antigravity`, `antigravity-ide`, `cursor`, `code`).
   - If found and executable exists, return path.

2. **Expanded Linux Filesystem Candidates**:
   - Check standard Debian/Ubuntu locations:
     - `/usr/bin/<exe>`
     - `/usr/local/bin/<exe>`
     - `/snap/bin/<exe>`
     - `/var/lib/snapd/snap/bin/<exe>`
     - `/opt/<folder>/<exe>` (both title case and lowercase)
     - `~/.local/bin/<exe>`
     - `~/.local/share/<folder>/<exe>`
     - `~/Applications/*.AppImage`

3. **Desktop Entry Detection**:
   - Scan `/usr/share/applications/` and `~/.local/share/applications/` for `.desktop` files matching antigravity, parse `Exec=` line to extract binary path.

4. **Coding Guidelines**:
   - Keep functions <= 8-15 lines.
   - Use implicit boolean evaluation (Rule 1).
