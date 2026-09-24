# Subtask 01: Installer Version Resolution & Workflow Stamping

> Status: DONE
> Owner: Antigravity Agent
> Spec: 02-spec/21-app/45-two-bar-installer-and-dwm-blank-ui-fix.md
> RCA: 02-spec/22-app-issues/10-blank-ui-dwm-occlusion-and-versioned-installer-rca.md

## Objectives
1. Fix `.github/workflows/release.yml` so `VER="${VERSION#v}"` is defined in the `Collect release files and generate checksums` step, ensuring `sed "s/__PINNED_VERSION__/$VER/g"` stamps the actual release version into `install.ps1` and `install.sh`.
2. Expand `Get-InvocationHistoryCandidates` in `install.ps1` to query `Get-History` and `[Microsoft.PowerShell.PSConsoleReadLine]::GetHistoryItems()` to detect pinned URLs even when invoked interactively via `irm ... | iex`.
3. Expand `Resolve-PinnedVersion` regex to parse `-Version`, `--version`, `AGM_VERSION`, and release download URLs.
4. Enforce strict version candidate queue in `install.ps1`: when `$isPinned` is `$true`, NEVER query latest releases, NEVER mark manifest as stale, NEVER replenish queue with unrequested versions, and construct direct release download URLs if GitHub API is unreachable.
