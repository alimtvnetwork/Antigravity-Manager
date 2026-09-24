# Specification: Instance Duplicate / Clone, Default Selector, Fast-Forward Shortcut, and Security Fixes

> Spec ID: `21-app/24`
> Status: `active`
> Date: 2026-09-24
> Author: Antigravity Autonomous Agent

## 1. Executive Summary & Problem Classification

This specification addresses four interconnected feature requests, UX enhancements, and critical diagnostics:
1. **Instance Duplicate / Clone Restoration**:
   - The duplicate/clone instance action was missing from the navbar dropdown header and individual instance rows.
   - Users need to rapidly clone both from the active instance header and from individual profile entries.
2. **Simplified Navbar Outside Action**:
   - The outside navbar previously displayed both a green Play/Stop button and a Fast-Forward button, cluttering the top navigation.
   - Requirement: On the outside top navbar, keep **only one button** — the **Fast-Forward** button (`[ ⏩ ]`). All play/run actions are centralized inside the dropdown and Instances page.
3. **Sequential Numbering & Default Instance Designation**:
   - Instance sequence numbers (`#1`, `#2`, `#3`...) must be visually prominent across the dropdown and management pages.
   - Any profile can be selected and designated as the "Default" instance. Default instances are protected from accidental deletion and prominently badged.
4. **Configurable Fast-Forward Shortcut (`Ctrl+Shift+F`)**:
   - The Fast-Forward action must be triggerable via a keyboard shortcut (default `Ctrl+Shift+F`).
   - The active shortcut must be visible on the Fast-Forward button tooltip and configurable via Settings under Auto Switcher.
5. **Security Page Diagnostic Fixes**:
   - **Error 1 (`get_ip_stats`)**: SQLite aggregate queries returned `NULL` on unpopulated/empty tables for `blocked` and `today` counts, triggering `Invalid column type Null at index: 2, name: blocked`.
   - **Error 2 (`get_ip_access_logs`)**: Tauri IPC argument mismatch (`missing required key query`) caused by unnested parameter payloads from the frontend.

---

## 2. Technical Data Contracts & Invariants

### A. Instance Default Designation
- **Backend IPC**: `commands::instance::set_default_instance(instance_id: String) -> AppResult<()>`
- **Registry Invariant**: Exactly one instance retains `is_default = true` at any given time. Setting an instance as default sets all others to `false`.
- **Deletion Protection**: An instance where `is_default == true` or `id == "default"` cannot be deleted.

### B. Fast-Forward Shortcut Invariant
- Configuration stored under `config.auto_profile_switcher.fast_forward_shortcut`.
- Default: `"Ctrl+Shift+F"`.
- Tooltip displays the evaluated shortcut: `Smart Switch (Fast-Forward) (${shortcut}): Close process, pick account with longest refill runway, and switch profile`.

### C. Security IP Stats Null Safety
- Queries over `ip_access_logs` must wrap aggregates in SQL `COALESCE(..., 0)` and map fields using `Option<u64>` with `.unwrap_or(0)` fallback:
  ```sql
  SELECT
      COALESCE(COUNT(*), 0) as total,
      COALESCE(COUNT(DISTINCT client_ip), 0) as unique_ips,
      COALESCE(SUM(CASE WHEN blocked = 1 THEN 1 ELSE 0 END), 0) as blocked,
      COALESCE(SUM(CASE WHEN timestamp >= ?1 THEN 1 ELSE 0 END), 0) as today
  FROM ip_access_logs
  ```

### D. Security IP Access Logs Query Normalization
- Rust command `get_ip_access_logs` accepts both `{ query: { ... } }` and top-level query parameters (`page`, `pageSize`, `search`, `blockedOnly`), all marked optional with safe defaults.

---

## 3. Visual & UX Specifications

1. **Top Navbar (`InstanceSelector.tsx`)**:
   - Trigger button: `[ · #1 Default ˅ ]`
   - Single outside action: `[ ⏩ ]` (Fast-Forward). Green Play button removed from the outside bar.
2. **Dropdown Menu (`InstanceSelector.tsx`)**:
   - Width enlarged to `w-96` (`24rem` / `384px`) to accommodate badges and full action controls without layout overflow.
   - Header bar contains: Title (`PROFILES`), Duplicate Active Profile (`Copy` icon), Import JSON (`Upload`), Export JSON (`Download`).
   - Profile item row contains:
     - Status dot (running/idle)
     - Sequence badge (`#1`, `#2`, `#3`...) with distinct high-contrast styling
     - Profile Name & Email
     - Badges: `ACTIVE`, `DEFAULT`
     - Action icons:
       1. Play / Stop
       2. Fast-Forward (Smart Switch)
       3. Duplicate Profile (`Copy` icon)
       4. Rename Profile (`Pencil` icon)
       5. Set as Default (`Star` / `Check` toggle or action)
       6. Delete Profile (`Trash` icon, hidden if default)
3. **Instances Page (`src/pages/Instances.tsx`)**:
   - Sequence number badges enlarged and highlighted.
   - Prominent `DEFAULT` badge and "Set as Default" button on profile cards.
   - Clear "Duplicate Profile" action available on every card.
4. **Settings Page (`AutoSwitcherSettings.tsx`)**:
   - Shortcut configuration input field allowing customized fast-forward keybinding.

---

## 4. Acceptance Criteria

- **AC-INST-001 (Single Outside Navbar Button)**: The outside navbar renders only the dropdown selector and Fast-Forward button. No outside Play button is present.
- **AC-INST-002 (Duplicate Restored)**: Both dropdown header and each profile item row provide a Duplicate button that opens the copy modal with scope selection (Full Copy vs Profile Only).
- **AC-INST-003 (Default Selection)**: Clicking "Set as Default" updates the registry, assigns `is_default = true`, renders the `DEFAULT` badge, and prevents deletion.
- **AC-INST-004 (Fast-Forward Shortcut)**: Pressing `Ctrl+Shift+F` (or configured shortcut) executes smart rotation. Hovering over the button displays the shortcut in the tooltip.
- **AC-SEC-001 (Zero Null Errors in get_ip_stats)**: `/security` page loads without SQLite type conversion error even when `ip_access_logs` table has zero rows.
- **AC-SEC-002 (Zero Argument Mismatch in get_ip_access_logs)**: `/security` access log viewer retrieves logs successfully with both nested and flat payloads.
