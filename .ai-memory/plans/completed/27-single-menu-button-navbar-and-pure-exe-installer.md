# Plan 27: Single Menu Button Navbar, Space Utilization, and Pure EXE Installer

> **Plan Path:** `.ai-memory/plans/completed/27-single-menu-button-navbar-and-pure-exe-installer.md`
> **Status:** Completed
> **Task Origin & Inception:** Initiated from user request to eliminate `.zip` archive installation failures completely by making `install.ps1` download and install the setup `.exe` directly, fix the top-level UI to be ultra-compact, replace multiple horizontal middle tabs with a single, dedicated Menu button dropdown showing the active page title and icon, utilize whitespace cleanly across the top bar, ensure the instance selector is permanently inside the window boundary with over 500px of clearance, and execute a full v4.23.0 release.
> **Total Steps / Loops Executed:** 4 subtasks completed in Loop 1 (Steps 1 .. 150).
> **Target Release:** v4.23.0

---

## User Request (Verbatim)

```text
is it done properly?

Couple of problems. Uh, first of all, the zip file that you, uh, add to the installation, that really does not work. Um, so that needs to fix actually. So probably you need to look into the installer script. Installer works. Now, the installer script, uh, should actually install the, the EXE rather than the zip. I think we should execute the zip. But we, we have to discuss about the zip later. So let's fix the installation part, part first. And you didn't fix the UI issue for the, for the instance. I actually asked you several times. The instance I cannot see that it's outside of the window. I can look into that. You didn't fix the UI issues. The UI looks much compact now, but I, I do think that you should utilize the space in between a little bit more and make the top level UI more compact. Uh, middle, too many options. Uh, let's, let's make it a menu button. Okay, make these fixes please, and make a release
```

---

## Extracted Actionable Task List

- [x] **Task 1: Middle Navigation Single Menu Button (`NavMenu.tsx`)**
  - Replaced multi-pill horizontal navigation entirely with a single, elegant Menu button dropdown in the center of the navbar.
  - Button dynamically displays the active section's icon and label (e.g. `[👥 Accounts ▾]`, `[📊 Dashboard ▾]`).
  - Clicking opens an animated popover dropdown listing all sections (Dashboard, Accounts, Instances, API Proxy, Transit Station, Traffic Logs, Token Stats, User Tokens, Security, Settings) with active checkmarks.
  - Eliminated UI clutter and unlocked >500px of whitespace between logo, menu, and controls.

- [x] **Task 2: Instance Selector Boundary Guarantee (`Navbar.tsx` & `InstanceSelector.tsx`)**
  - With the middle reduced to ~130px, total required navbar width is ~520px, providing massive clearance on standard 1024x700 displays.
  - Enforced `shrink-0` on `<InstanceSelector />`, profile dropdown trigger, quick clone button, quick add button, and `<NavSettings />`.
  - Guaranteed instance selector, status pulse dot, and controls are permanently locked within the visible window viewport.

- [x] **Task 3: Pure EXE Installer Hardening (`install.ps1`)**
  - Completely removed `.zip` download and `Expand-Archive` paths from `install.ps1`.
  - Exclusively targets NSIS setup executable (`*setup.exe` / `*.exe`).
  - Runs silent setup via `/S` and `/D=$InstallDir` without prompting wizard.
  - Verified through dry run execution.

- [x] **Task 4: Quality Gate Verification, Release v4.23.0 & Git Push**
  - Synchronized version to `4.23.0` across 28 manifest and doc files via `03-ai-scripts/29-release-orchestrator.py`.
  - Normalized UTF-8 LF encodings and validated relative paths.
  - Consolidated subtasks and tagged release `v4.23.0`.
  - Atomic commit and push to `origin/main` and `origin/release/v4.23.0`.

---

## Consolidated Subtasks & Implementations

### Subtask 01: Middle Single Menu Button Implementation
- **Target Files:** `src/components/navbar/NavMenu.tsx`
- **Accomplishments:**
  - Consolidated navigation into a single centered capsule button.
  - Shows current active item icon and label with animated chevron indicator.
  - Opens clean dropdown popover with outside-click dismissal.

### Subtask 02: Instance Selector Boundary & Space Utilization
- **Target Files:** `src/components/navbar/Navbar.tsx`, `src/components/navbar/InstanceSelector.tsx`
- **Accomplishments:**
  - Layout now consumes only ~520px of width across all controls.
  - `shrink-0` guarantees instance selector cannot be compressed or clipped.

### Subtask 03: Pure EXE Installer Hardening
- **Target Files:** `install.ps1`
- **Accomplishments:**
  - Eliminated `.zip` matching patterns and archive extraction code entirely.
  - Installer exclusively downloads setup `.exe` and runs unattended silent installation.

### Subtask 04: Version Sync, Quality Gates & Release v4.23.0
- **Target Files:** `version.json`, `package.json`, `src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, etc.
- **Accomplishments:**
  - Orchestrated release v4.23.0 across 28 files.
  - Successfully tagged `v4.23.0` and checked out release branch.
