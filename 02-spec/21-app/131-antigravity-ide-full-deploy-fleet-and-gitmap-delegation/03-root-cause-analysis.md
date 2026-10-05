# Root Cause Analysis: 131-antigravity-ide-full-deploy-fleet-and-gitmap-delegation

## 1. Defect Description & Observed Failure

### Observed Issues:
1. When secondary or cloned instances are launched from Antigravity Manager (or deployed to other nodes), they do NOT have the default instance's theme applied.
2. Preset modes (Turbo Mode, eager execution, auto-approvals, zero-trust workspace security bypass) are not copied.
3. Plugins (the 4 official Gemini agent plugins) are not applied and missing from secondary instance homes.
4. Skills (the 9 core builtin skills and plugin skills) are missing.
5. In the local filesystem at `C:\Users\Administrator\.antigravity_tools\instances\default\home\.gemini\`:
   - `config\config.json` was entirely missing.
   - `config\plugins\` was entirely missing.
   - `antigravity\builtin\skills\` was entirely missing.
   - `settings.json` only contained `"window.title"`, dropping `"workbench.colorTheme"`.

---

## 2. Root Cause Analysis (4 Failure Mechanisms)

### Mechanism 1: Host vs Instance Home Path Redirection
- When running the host Antigravity IDE without sandboxing, `$HOME` resolves to `C:\Users\Administrator\`. The host configuration is located at `C:\Users\Administrator\.gemini\config\config.json` (containing Dracula purple `#BD93F9`, `#19191C`, `#F8F8F2`, Turbo preset, and 4 plugins).
- When running inside an Antigravity Manager instance, `$HOME` is redirected to `C:\Users\Administrator\.antigravity_tools\instances\<id>\home`.
- The instance initialization routine created an empty `.gemini` folder with only OAuth tokens, but never synchronized `config\config.json`, `config\plugins`, or `antigravity\builtin\skills`.

### Mechanism 2: Narrow File Copy Filter in `copy_gemini_trees`
- In `src-tauri/src/modules/instance.rs`, the helper `copy_gemini_trees` only enumerated a hardcoded list of root files (`google_accounts.json`, `oauth_creds.json`, etc.). It skipped `.gemini\config\` and `.gemini\antigravity\builtin\skills\`.
- Furthermore, `copy_dir_recursive` filtered out any directory containing the word `"cache"`, which erroneously dropped certain plugin assets.

### Mechanism 3: Overwriting of `settings.json` Without Theme Preservation
- When instances are launched or cloned, `instance.rs` updated `settings.json` with the formatted `window.title`.
- However, if `settings.json` did not already have `"workbench.colorTheme"`, none was added. Additionally, `enforce_default_settings` was not called unconditionally on all creation paths.

### Mechanism 4: Absence of Automated Remote Deployment Recipe in GitMap
- Previously, deploying Antigravity IDE onto another physical or virtual machine required manual file copies and ad-hoc PowerShell commands.
- There was no integrated command bridging GitMap's cluster orchestration or SSH subsystem with a self-contained Antigravity IDE deployment script.

---

## 3. Remediation & Preventive Measures

1. **Populate Reference Instance (`instances/default`)**:
   - Immediately copy host `.gemini/config/config.json`, `.gemini/config/plugins`, and `.gemini/antigravity/builtin/skills` into `instances/default/home/.gemini/`.
   - Update `instances/default/home/AppData/Roaming/Antigravity/User/settings.json` with `workbench.colorTheme` and Turbo mode settings.

2. **Implement `sync_instance_ide_parity` in `instance.rs`**:
   - On instance creation (`create_instance_with_account`), duplication (`copy_instance_with_options`), or startup (`ensure_default_instance_exists`), call `sync_instance_ide_parity` to deep-copy the 4 pillars.

3. **Deploy Fleet Script in `repo-secrets`**:
   - Provide `deploy-antigravity-ide-fleet.ps1` in `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/`.
   - Implement `gitmap-delegate-deploy.ps1` to automate remote machine deployment via `gitmap cluster run-script` or SSH.

4. **Automated 7-Gate Scorecard (`VG-01` to `VG-07`)**:
   - Enforce programmatic verification of all 7 gates before marking any deployment complete.

---

## 4. Verification & Testing Matrix

| Gate | Target Check | Pass Criteria |
|---|---|---|
| `VG-01` | Theme Parity | `customThemeSeedsDark` in `config.json` + `workbench.colorTheme` in `settings.json` |
| `VG-02` | Presets Parity | `autoExecutionPolicy = CASCADE_COMMANDS_AUTO_EXECUTION_EAGER`, `turboMode = true`, trust disabled |
| `VG-03` | Plugins Parity | Exactly 4 official plugins present in `config/plugins/` and enabled |
| `VG-04` | Skills Parity | Exactly 9 builtin skills present in `antigravity/builtin/skills/` |
| `VG-05` | Hygiene & Keyring | Keyring bypass markers present; `app_storage.json` marked complete |
| `VG-06` | Binaries & CLI | `Antigravity.exe` and `agy.exe` present and accessible |
| `VG-07` | Instances Readiness | Profile data directory and home directory structure validated |
