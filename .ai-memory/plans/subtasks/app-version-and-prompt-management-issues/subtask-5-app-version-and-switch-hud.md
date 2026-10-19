# Subtask 5: App Version Publishing Gate & Post-Restore Account Switch Timer HUD

## Objective
1. Diagnose why installed app version is `v4.234.0` when repository is at `v4.236.0` and why update installation fails.
2. Ensure UI update notifications only declare a version available when verified against GitHub published releases.
3. Surface a visible account switch waiting time countdown and next action when account rotation occurs.

## Key Files
- `src-tauri/src/modules/updater.rs`
- `src/stores/use-update-store.ts`
- `src/components/instances/AutoSwitcher/`

## Verification
- Version reporting adheres strictly to published release assets.
- Switch countdown HUD renders active cooldown and target action.
