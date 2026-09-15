# 06. Multi-Instance Manager and UI Specification

## Overview
Antigravity-Manager enables running multiple independent, isolated Antigravity instances concurrently. Each instance maintains its own dedicated `--user-data-dir`, extensions directory, configuration, and bound account credentials.

## UI Specification

### 1. Top Navigation Bar (`Navbar.tsx`)
- **Instance Selector Dropdown (`InstanceSelector.tsx`)**:
  - Position: Right side of the top navigation bar, adjacent to theme/language toggles.
  - Active Display: Shows active instance label (e.g., `Default`, `Instance 1`, `Work`).
  - Status Indicator: Green dot when the instance has an active running process with PID; grey dot when idle.
  - Quick Actions:
    - **Copy Button (❐)**: Instantly duplicates the selected instance configuration and extensions into a new profile.
    - **Add Button (+)**: Opens a modal to create a new, clean isolated profile with a custom name.
- **Nav Menu Tab**:
  - Adds `/instances` tab ("Instances" / "多实例") with high priority in `Navbar.tsx`.

### 2. Dedicated Instances Tab (`src/pages/Instances.tsx`)
- **Header**: Search bar, total instance counter, running instance counter, and "Create New Instance" primary button.
- **Instance Cards**:
  - **Title & Badge**: Name, instance ID, running PID badge (`Running: PID 14208` or `Idle`), default badge.
  - **Account Binding**: Shows bound email address or unassigned state.
  - **Path Details**: Displays `--user-data-dir` path with an open folder button.
  - **Actions**:
    - **Launch / Bring to Front**: Starts Antigravity with `--user-data-dir` and `--password-store=basic`.
    - **Close Instance**: Selectively kills only this instance PID (`kill -15 <pid>`). Sibling instances remain unaffected.
    - **Clone / Duplicate**: Copies configuration and extensions to a new instance.
    - **Wipe Session**: Clears authentication tokens while keeping user settings and extensions intact.
    - **Delete**: Safely removes the instance folder (blocked if currently running).

### 3. Account Switching Evolution (`AccountRow.tsx`)
- Clicking the switch button (`⇄`):
  - Injects the credentials directly into the active target instance's `state.vscdb`.
  - Launches or brings the target instance to front.
  - **Zero Sibling Termination**: Other instances running other accounts stay open and active.
- Dropdown menu on switch button:
  - "Open in Active Instance"
  - "Open in [Instance Name]..."
  - "Duplicate New Instance & Open"
