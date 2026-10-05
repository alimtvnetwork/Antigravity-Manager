# Component Specification: 131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation

## 1. PowerShell Fleet Deployment Component (`deploy-antigravity-ide-fleet.ps1`)

### Location:
- Primary: `d:\work\repo-secrets\02-antigravity-manager\scripts\deploy-antigravity-ide-fleet.ps1`
- Mirror: `scripts/deploy-antigravity-ide-fleet.ps1`

### CLI Parameter Contract:
```powershell
[CmdletBinding(SupportsShouldProcess = $true)]
param(
    [Parameter(Position = 0, HelpMessage = "Target hostname, IP address, or GitMap SSH node alias")]
    [string]$TargetHost = "localhost",

    [Parameter(Position = 1, HelpMessage = "Target username on destination system")]
    [string]$TargetUser = $env:USERNAME,

    [Parameter(HelpMessage = "Antigravity execution preset mode: turbo, eager, default")]
    [ValidateSet("turbo", "eager", "default")]
    [string]$Preset = "turbo",

    [Parameter(HelpMessage = "Workbench color theme identifier to enforce")]
    [string]$Theme = "Default Dark Modern",

    [Parameter(HelpMessage = "Deploy the 4 official Gemini agent plugins")]
    [bool]$DeployPlugins = $true,

    [Parameter(HelpMessage = "Deploy the 9 core builtin skills and plugin skills")]
    [bool]$DeploySkills = $true,

    [Parameter(HelpMessage = "Deploy or link VS Code extensions directory")]
    [bool]$DeployExtensions = $true,

    [Parameter(HelpMessage = "Preview modifications without writing to disk")]
    [switch]$DryRun,

    [Parameter(HelpMessage = "Force overwrite existing settings and configuration")]
    [switch]$Force
)
```

### Modular Functions:
- `Deploy-Pillar1Theme`: Deploys theme seeds (`#19191C`, `#F8F8F2`, `#BD93F9`), sets `themeMode = THEME_MODE_DARK` in `.gemini/config/config.json`, and sets `workbench.colorTheme` in `settings.json`.
- `Deploy-Pillar2Presets`: Configures `security_presets.json` (`active_preset = turbo`), `antigravity_policies.json` (unrestricted), sets `antigravity.turboMode = true`, and sets `security.workspace.trust.enabled = false`.
- `Deploy-Pillar3Plugins`: Copies the 4 official plugins (`chrome-devtools-plugin`, `data-agent-kit-plugin`, `google-antigravity-sdk`, `modern-web-guidance-plugin`) and enables them in `config.json`.
- `Deploy-Pillar4Skills`: Copies the 9 builtin skills and all plugin skills.
- `Deploy-KeyringAndAppStorage`: Drops keyring unavailable bypass markers (`antigravity-ide-keyring-unavailable`, etc.) and marks wizard shown in `app_storage.json`.
- `Deploy-AntigravityBinaries`: Verifies `Antigravity.exe` and deploys CLI `agy.exe` into user PATH.
- `Show-VerificationScorecard`: Evaluates `VG-01` through `VG-07` and prints formatted audit report.

---

## 2. GitMap Remote Delegation Bridge (`scripts/gitmap-delegate-deploy.ps1`)

### Location:
`scripts/gitmap-delegate-deploy.ps1`

### CLI Parameter Contract:
```powershell
param(
    [Parameter(Position = 0)]
    [string]$Target = "localhost",

    [Parameter()]
    [ValidateSet("turbo", "eager", "default")]
    [string]$Preset = "turbo",

    [Parameter()]
    [string]$Theme = "Default Dark Modern",

    [Parameter()]
    [string]$DeployScript = "scripts/deploy-antigravity-ide-fleet.ps1",

    [Parameter()]
    [ValidateSet("cluster", "ssh", "local")]
    [string]$Mode = "cluster",

    [Parameter()]
    [bool]$DeployPlugins = $true,

    [Parameter()]
    [bool]$DeploySkills = $true,

    [Parameter()]
    [bool]$DeployExtensions = $true,

    [Parameter()]
    [switch]$DryRun,

    [Parameter()]
    [switch]$Force,

    [Parameter()]
    [switch]$SkipPing
)
```

### Delegation Dispatch Modes:
1. **Cluster Mode (`gitmap cluster run-script`)**:
   Dispatches through GitMap's native cluster orchestration to fleet node targets (`w1`, `w2`, `w3`, `u1`).
2. **SSH Mode (`gitmap ssh exec`)**:
   Streams the deployment script execution over SSH to the target host.
3. **Local Mode**:
   Executes the deployment script locally to synchronize the current machine's environment or default instance.

---

## 3. AGM Rust Parity Engine (`src-tauri/src/modules/instance.rs`)

### Key Methods & Contracts:
- `sync_instance_ide_parity(target_instance_id: &str) -> AppResult<()>`:
  - Discovers host `%USERPROFILE%\.gemini\config\config.json` and copies theme seeds + presets to `instances/<id>/home/.gemini/config/config.json`.
  - Discovers host `%USERPROFILE%\.gemini\config\plugins` and synchronizes all 4 official plugins.
  - Discovers host `%USERPROFILE%\.gemini\antigravity\builtin\skills` and synchronizes all 9 builtin skills.
  - Updates `instances/<id>/home/AppData/Roaming/Antigravity/User/settings.json` with theme and Turbo settings.
- `copy_instance_with_options`:
  - Calls `sync_instance_ide_parity(&new_instance.id)` on duplication or creation.
- `ensure_default_instance_exists`:
  - Calls `sync_instance_ide_parity("default")` so the default instance always has 100% complete assets.
