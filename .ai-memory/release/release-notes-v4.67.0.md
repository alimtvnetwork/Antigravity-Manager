## Quick Install v4.67.0

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install (v4.67.0)
```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1))) -Version 4.67.0
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

#### Pinned Version Install (v4.67.0)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.67.0
```

---

-   **[Release v4.67.0: Auto Switcher UI/UX Overhaul, Local Multithreaded Runner & Docker Binary Standardization] Auto Switcher Dual-Card Symmetry, Dark Mode Theme Fixes, Full-Core Parallel Compilation & Docker Distribution Alignment**:
    -   **Auto Switcher Settings UI/UX Complete Overhaul**: Transformed the isolated left-hand model dropdown into an official, balanced **Primary Evaluated Model** card featuring a `Cpu` icon badge, `Trigger Metric` status pill, clear subtitle, and dynamic contextual trait banners explaining model profiles (Gemini 3.8 Flash high throughput, Claude Sonnet 4.6 deep reasoning budget failover, Gemini Pro baseline). Integrated a **Rotation Cooldown Guard** slider directly into the card to achieve flawless height symmetry with the right column.
    -   **Dark Mode Recency Cutoff Area Clash Root Cause Elimination**: Eliminated DaisyUI and un-themed Tailwind background color clashes that caused stark white and gray containers in dark mode. Rebuilt the active prompt recency cutoff window panel using theme-native Slate styling (`bg-slate-100/90 dark:bg-slate-900/90 border-slate-700/80`) and unified the right column under an official **Task Continuity & Watchdog** panel.
    -   **Local Multithreaded Fast Runner Optimization (`run.ps1`)**: Automatically detects system CPU core count and configures `$env:CARGO_BUILD_JOBS` for full-thread parallel compilation. Added auto-detection and activation for `sccache` compiler cache and `lld-link` fast linker, and introduced a `-Quick` / `-Run` instant execution mode to launch pre-built binaries without rebuild delays, alongside `-OptimizeIO` for Windows Defender build directory exclusions.
    -   **Docker Build Standardization & Packaging Alignment**: Updated `docker/Dockerfile`, `Dockerfile.backend`, and `Dockerfile.backend.localdist` to build the official `agm-alim` binary with `/app/antigravity-tools` symlinks, ensuring seamless headless container builds.
