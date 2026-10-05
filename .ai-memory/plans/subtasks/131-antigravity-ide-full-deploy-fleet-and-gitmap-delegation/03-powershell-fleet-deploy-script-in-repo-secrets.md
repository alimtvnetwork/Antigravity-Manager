# Subtask 03: Standalone Fleet Deployment Script in repo-secrets

## Deliverable:
Maintain and enhance the standalone PowerShell deployment script:
- `d:\work\repo-secrets\02-antigravity-manager\scripts\deploy-antigravity-ide-fleet.ps1`
- `scripts/deploy-antigravity-ide-fleet.ps1`

## Capabilities:
- Full unattended deployment across 4 pillars:
  - Theme: Dracula purple seeds and VS Code workbench settings.
  - Presets: Turbo mode, eager execution, zero-trust bypass.
  - Plugins: 4 official plugins deployed and enabled.
  - Skills: 9 builtin skills and plugin skills deployed.
- Keyring bypass markers (`antigravity-ide-keyring-unavailable`).
- Binary deployment & user PATH registration for `agy.exe`.
- Automated 7-Gate Verification Scorecard (`VG-01` to `VG-07`).

## Verification:
- Script executes locally with `-DryRun` and produces formatted scorecard.
- Script exists in both `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/`.
