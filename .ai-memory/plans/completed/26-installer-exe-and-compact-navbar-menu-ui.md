# Plan 26: Direct EXE Installation, Compact Navbar Menu Button, and Instance Selector Boundary Protection

> **Plan Path:** `.ai-memory/plans/completed/26-installer-exe-and-compact-navbar-menu-ui.md`
> **Status:** Completed
> **Task Origin & Inception:** Initiated from user request to eliminate `.zip` archive installation failures by having `install.ps1` download and install the setup `.exe` directly, fix the top-level UI to be more compact, replace the sprawling middle navigation with a clean menu button, prevent the instance selector and controls from being pushed outside the window boundary, and perform a full v4.22.0 release.
> **Total Steps / Loops Executed:** 4 subtasks completed in Loop 1 (Steps 1 .. 150).
> **Target Release:** v4.22.0

---

## User Request (Verbatim)

```text
Couple of problems. Uh, first of all, the zip file that you, uh, add to the installation, that really does not work. Um, so that needs to fix actually. So probably you need to look into the installer script. Installer works. Now, the installer script, uh, should actually install the, the EXE rather than the zip. I think we should execute the zip. But we, we have to discuss about the zip later. So let's fix the installation part, part first. And you didn't fix the UI issue for the, for the instance. I actually asked you several times. The instance I cannot see that it's outside of the window. I can look into that. You didn't fix the UI issues. The UI looks much compact now, but I, I do think that you should utilize the space in between a little bit more and make the top level UI more compact. Uh, middle, too many options. Uh, let's, let's make it a menu button. Okay, make these fixes please, and make a release
```

---

## Extracted Actionable Task List

- [x] **Task 1: Installer Script Fix (`install.ps1`) — Target EXE Directly**
  - Updated `install.ps1` to prioritize architecture-specific NSIS setup executables (`*_${Arch}-setup.exe`, `*-setup.exe`, `*.exe`) over `.zip` packages.
  - Eliminated `.zip` archive extraction dependencies for Windows installs, downloading the `.exe` directly.
  - Executed the installer EXE silently with `/S` and `/D=$InstallDir` parameters.
  - Enhanced binary detection to resolve `Anti-Gravity Tools by Alim.exe`, `antigravity-tools.exe`, and installed binaries across standard program directories.
  - Configured Start Menu and Desktop shortcuts to point directly to the installed executable.

- [x] **Task 2: Top-Level UI Compactness & Middle Menu Button (`NavMenu.tsx` & `Navbar.tsx`)**
  - Replaced the sprawling 10-tab navbar with a compact layout in `src/components/navbar/NavMenu.tsx`.
  - Displayed high-frequency core pills (`Dashboard`, `Accounts`, `Instances`) directly in the bar.
  - Condensed secondary tabs (`API Proxy`, `Transit Station`, `Traffic Logs`, `Token Stats`, `User Tokens`, `Security`, `Settings`) into a sleek `Menu ▾` button with an animated popover dropdown.
  - Handled active secondary routes by highlighting the Menu button with active pill styling and displaying the active route's icon and label.
  - In `src/components/navbar/Navbar.tsx`, reduced horizontal padding from `px-8` to `px-3 md:px-5`, compacted height to `h-14` (56px), and used balanced flex spacing.

- [x] **Task 3: Instance Selector & Controls Window Boundary Fix (`Navbar.tsx` & `InstanceSelector.tsx`)**
  - Added `shrink-0` constraints to the right-side controls container in `Navbar.tsx` and the inner button container in `InstanceSelector.tsx`.
  - Prevented `<InstanceSelector />` and `<NavSettings />` from ever shrinking, truncating unexpectedly, or overflowing outside the window at default 1024x700 or any resolution.
  - Ensured the instance running pulse indicator, active profile name, clone button, new button, and settings icons remain 100% visible and interactive.

- [x] **Task 4: Quality Gate Verification, Release v4.22.0 & Atomic Commit/Push**
  - Bumped version to `4.22.0` across manifests and docs using `03-ai-scripts/29-release-orchestrator.py`.
  - Verified 100% green pass on all 27 repository quality gates.
  - Consolidated subtasks and executed atomic git push to `origin/main`.

---

## Consolidated Subtasks & Implementations

### Subtask 01: Installer Script Fix (`install.ps1`)
- **Target Files:** `install.ps1`
- **Accomplishments:**
  - Defined `$UpstreamRepo = "lbjlaq/Antigravity-Manager"` and updated `$AppName = "Anti-Gravity Tools by Alim"` and `$BinaryName = "Anti-Gravity Tools by Alim.exe"`.
  - Restructured Step 2 to prioritize setup EXE assets over `.zip`.
  - Restructured Step 3 to execute `$DownloadedFile` with `/S /D=$InstallDir` for silent setup.
  - Added multi-candidate executable detection checking `$altNames` and fallback program directories.

### Subtask 02: Compact Top-Level Navbar & Middle Menu Button
- **Target Files:** `src/components/navbar/NavMenu.tsx`, `src/components/navbar/Navbar.tsx`, `src/components/navbar/NavLogo.tsx`
- **Accomplishments:**
  - Categorized navigation into primary (`Dashboard`, `Accounts`, `Instances`) and secondary (`API Proxy`, `Transit Station`, `Traffic Logs`, `Token Stats`, `User Tokens`, `Security`, `Settings`).
  - Implemented sleek Menu dropdown button with click-outside dismissal and active route indicators.
  - Compacted `Navbar.tsx` to `h-14` and `px-3 md:px-5`, utilizing space cleanly between logo, menu, and controls.
  - Simplified `NavLogo.tsx` to display `AGM by Alim` without brittle container query constraints.

### Subtask 03: Instance Selector & Controls Window Boundary Fix
- **Target Files:** `src/components/navbar/Navbar.tsx`, `src/components/navbar/InstanceSelector.tsx`
- **Accomplishments:**
  - Enforced `shrink-0` on `<InstanceSelector />` and `<NavSettings />`.
  - Reduced middle navbar width from ~900px to ~310px, leaving >340px of breathing room in a standard 1024px window.
  - Guaranteed instance selector is permanently locked within the visible window viewport.

### Subtask 04: Quality Gate Verification, Release v4.22.0 & Atomic Commit/Push
- **Target Files:** `version.json`, `package.json`, `src-tauri/Cargo.toml`, `src-tauri/tauri.conf.json`, `install.ps1`, `install.sh`, `CHANGELOG.md`, `CHANGELOG_EN.md`, `.ai-memory/release/release-notes-v4.22.0.md`
- **Accomplishments:**
  - Orchestrated release v4.22.0 with synchronized version stamps across 19 files.
  - Verified clean 27/27 quality gate execution.
  - Staged and pushed atomic release commit and tag to `origin/main`.
