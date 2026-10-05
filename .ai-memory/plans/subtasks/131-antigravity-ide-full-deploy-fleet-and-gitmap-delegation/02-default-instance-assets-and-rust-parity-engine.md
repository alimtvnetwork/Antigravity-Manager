# Subtask 02: Default Instance Asset Ingestion & Rust Parity Engine

## Deliverable:
1. Ingest host environment reference configuration into `C:\Users\Administrator\.antigravity_tools\instances\default\home\.gemini\`:
   - Copy `config\config.json` containing Dracula purple `#BD93F9`, `#19191C`, `#F8F8F2`, Turbo preset, and enabled plugins.
   - Copy `config\plugins\` containing the 4 official plugins (`chrome-devtools-plugin`, `data-agent-kit-plugin`, `google-antigravity-sdk`, `modern-web-guidance-plugin`).
   - Copy `antigravity\builtin\skills\` containing the 9 core builtin skills.
   - Update `instances\default\home\AppData\Roaming\Antigravity\User\settings.json` with `workbench.colorTheme` and Turbo mode settings.
2. In `src-tauri/src/modules/instance.rs`:
   - Implement `sync_instance_ide_parity(target_instance_id: &str) -> AppResult<()>` to ensure every instance inherits the 4 pillars.
   - Call `sync_instance_ide_parity` inside `copy_instance_with_options`, `create_instance_with_account`, and `ensure_default_instance_exists`.

## Verification:
- `instances/default/home/.gemini/config/config.json` exists and validates against JSON schema.
- All 4 plugins and 9 builtin skills exist inside `instances/default`.
