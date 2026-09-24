# Completed Plan 70: Training REST API & Settings UI/UX Overhaul

## 1. Overview & User Requirements
- **Settings Menu UI/UX Cleanup**:
  - Buttons shortened: "Save Settings" -> "Save", "Backup & Restore" -> "Backup".
  - Tabs shortened: "Proxy Settings" -> "Proxy", "Email & Alerts" -> "Email-Alerts", "Supabase Sync" -> "Supabase".
  - Redundant "Debug" tab removed from main tab strip, aligning with title bar Bug icon.
  - Multi-language dictionary support aligned in English (`en.json`) and Chinese (`zh.json`).
  - Added Debug Console quick-access card inside Advanced tab.
- **Machine Training REST API**:
  - Complete endpoint exposure: `/api/v1/training`, `/api/v1/status`, `/api/v1/training/telemetry`, `/api/v1/training/learn`, `/api/v1/training/machines`, `/api/v1/training/modify`, `/api/v1/machines`.
  - Machine state inspection via SQLite `training_vault.db` and live system telemetry.
  - Machine modification via full IDE credential injection: `instance::switch_account_to_instance(&target.id, Some(&inst_id))` and `auto_switcher::trigger_manual_rotation_for_instance`.
  - Service toggle in General settings: `training_api_enabled`.

## 2. Key Code Changes
- `src/pages/Settings.tsx`: Clean 7-tab bar (General, Account, Proxy, Email-Alerts, Supabase, Advanced, About) and action buttons (Backup, Save); Debug quick card in Advanced.
- `src/locales/zh.json`: Updated `"save": "保存"`, `"backup_restore": "备份"`, `"proxy": "代理"`, `"email": "邮件告警"`, `"supabase": "Supabase"`.
- `src/locales/en.json`: Aligned concise tab and action keys.
- `src-tauri/src/modules/training_api.rs`: Integrated `switch_account_to_instance` and `trigger_manual_rotation_for_instance`.
- `src-tauri/src/proxy/server.rs`: Added short aliases for REST training endpoints.

## 3. Verification & Compliance
- `cargo fmt -- --check`: Passed with zero errors.
- Coding guidelines: Strict adherence to error management, clean naming, and atomic commit discipline.
