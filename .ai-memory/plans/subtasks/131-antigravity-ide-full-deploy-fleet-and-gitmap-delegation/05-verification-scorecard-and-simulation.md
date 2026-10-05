# Subtask 05: 7-Gate Verification Scorecard & Deployment Simulation

## Deliverable:
Maintain and run the automated verification test suite:
- `scratch/verify_ide_deployment.py`

## Gates to Audit:
1. `VG-01`: Theme Parity Gate (`customThemeSeedsDark` #BD93F9 / `workbench.colorTheme`).
2. `VG-02`: Preset Mode Gate (`turboMode = true`, `security.workspace.trust.enabled = false`).
3. `VG-03`: Plugins Inventory Gate (all 4 official plugins present and enabled).
4. `VG-04`: Skills Completeness Gate (all 9 builtin skills present).
5. `VG-05`: Hygiene & Keyring Gate (keyring bypass markers present, `app_storage.json` populated).
6. `VG-06`: Binaries & CLI Gate (`Antigravity.exe` & `agy.exe`).
7. `VG-07`: Instances Readiness Gate (profile structure valid).

## Verification:
- Run `python scratch/verify_ide_deployment.py` and verify all gates PASS.
