# Plan 27: Single Menu Button Navbar, Space Utilization, and Pure EXE Installer

> **Plan Path:** `.ai-memory/plans/pending/27-single-menu-button-navbar-and-pure-exe-installer.md`
> **Status:** In Progress
> **Task Origin & Inception:** User requested to make the middle navigation a dedicated single Menu button (removing multiple pills in the middle), utilize space in between cleanly, ensure the instance selector is permanently inside the window boundary, eliminate zip handling from the installer in favor of direct setup EXE installation, and release v4.23.0.
> **Total Steps / Loops Budget:** N = 150 (Phase 1: Steps 1..75, Phase 2: Steps 76..150).
> **Target Release:** v4.23.0

---

## User Request (Verbatim)

```text
is it done properly?


Couple of problems. Uh, first of all, the zip file that you, uh, add to the installation, that really does not work. Um, so that needs to fix actually. So probably you need to look into the installer script. Installer works. Now, the installer script, uh, should actually install the, the EXE rather than the zip. I think we should execute the zip. But we, we have to discuss about the zip later. So let's fix the installation part, part first. And you didn't fix the UI issue for the, for the instance. I actually asked you several times. The instance I cannot see that it's outside of the window. I can look into that. You didn't fix the UI issues. The UI looks much compact now, but I, I do think that you should utilize the space in between a little bit more and make the top level UI more compact. Uh, middle, too many options. Uh, let's, let's make it a menu button. Okay, make these fixes please, and make a release
```

---

## Extracted Actionable Task List

- [ ] **Task 1: Middle Navigation Single Menu Button (`NavMenu.tsx`)**
  - Replace the multi-pill horizontal navigation entirely with a single, elegant Menu button dropdown in the center of the navbar.
  - Display current active section icon and title on the Menu button (e.g. `[👥 Accounts ▾]` or `[☰ Menu ▾]`).
  - Group all navigation items cleanly inside the dropdown popover (Dashboard, Accounts, Instances, API Proxy, Transit Station, Traffic Logs, Token Stats, User Tokens, Security, Settings).
  - Maximize compact layout and provide abundant breathing room between logo, middle menu, and right-side controls.

- [ ] **Task 2: Instance Selector Boundary Guarantee (`Navbar.tsx` & `InstanceSelector.tsx`)**
  - With the middle reduced to a single compact menu button (~120px), guarantee >500px of horizontal space across all standard desktop resolutions (1024x700 and below).
  - Enforce `shrink-0` on `<InstanceSelector />`, profile dropdown trigger, quick clone button, quick add button, and `<NavSettings />`.
  - Ensure the instance profile name, live running pulse dot, and all controls can never be clipped or pushed outside the window viewport.

- [ ] **Task 3: Pure EXE Installer Hardening (`install.ps1`)**
  - Completely remove `.zip` download and archive extraction paths from `install.ps1`.
  - Exclusively download and execute the NSIS setup executable (`*setup.exe` / `*.exe`).
  - Pass `/S` and `/D=$InstallDir` cleanly to ensure unattended, silent direct setup.
  - Verify through dry run execution.

- [ ] **Task 4: Quality Gate Verification, Release v4.23.0 & Git Push**
  - Bump version to `v4.23.0` across 28 manifest and documentation files.
  - Run file-level linters and verify repository hygiene.
  - Consolidate plan into `.ai-memory/plans/completed/27-single-menu-button-navbar-and-pure-exe-installer.md`.
  - Commit and push atomic release tag `v4.23.0` to `origin/main`.
