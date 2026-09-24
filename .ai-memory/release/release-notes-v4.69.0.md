## Quick Install v4.69.0

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.69.0)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version 4.69.0
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.69.0)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.69.0
```

---

-   **[Release v4.69.0: Complete IDE Copy vs Profile Copy Clone Architecture & UI Layout Perfection] IDE vs Profile Duplication Modes, Navbar Item Reordering, Split Release Code Blocks & Parity Alignment**:
    -   **Restored Complete "IDE Copy vs Profile Copy" Duplication Engine**: Re-introduced the dual-mode cloning architecture across both the navigation bar (`InstanceSelector.tsx`) and the main management dashboard (`Instances.tsx`). Users can explicitly choose between **IDE Copy (Full Environment & Sessions)** (clones the complete isolated data directory, active workspace sessions, extensions, cache, and state) and **Profile Copy (Preferences & Snippets)** (copies only user preferences, keybindings, and snippets without bulky runtime session data).
    -   **Dropdown Header & Item Row UI Layout Perfection**: Strictly aligned the profile selector dropdown with user layout specifications (`https://prnt.sc/dbDLrTd-Rfns`). Renamed header section to `INSTANCES / PROFILES` with dedicated "Duplicate Active Profile" action. Reordered action buttons in every profile row to place the Rename button (`Pencil`) before the Duplicate button (`Copy`), matching screenshot layout: `Launch / Stop` -> `Fast-Forward` -> `Rename` -> `Duplicate` -> `Set Default` -> `Delete`.
    -   **Navbar Minimalist Single Fast-Forward Button Guarantee**: Confirmed that the outer navigation bar displays exclusively the single Fast-Forward (`[ ⏩ ]`) button with active shortcut tooltip (`Ctrl+Shift+F`) and prominent sequence badges (`#1`, `#2`, `#3`...), completely eliminating redundant play buttons.
    -   **Release Code Blocks Isolation & Permanent Split Verification**: Verified and enforced distinct, separated code blocks for Direct Latest and Pinned Version installer commands across GitHub Releases and release manifests to ensure copy buttons capture single actionable commands without syntax or comment leakage.
