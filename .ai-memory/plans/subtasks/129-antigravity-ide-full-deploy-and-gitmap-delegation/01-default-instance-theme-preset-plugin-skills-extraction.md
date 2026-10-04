# Subtask 01: Default Instance Theme, Presets, Plugins, and Skills Extraction

**Task ID:** `129-01-default-instance-theme-preset-plugin-skills-extraction`  
**Target Files:** `src-tauri/src/modules/instance.rs`, Default Instance Profiles  
**Owner:** Worker 01 (Backend Specialist)  
**Status:** READY  
**Prerequisites:** None  

---

## 1. Context & User Directive (Verbatim)

```text
1. Default instance theme not applied to instances
2. Preset mode not copied from default instance
3. Plugins are not applied, skills are not added
```

---

## 2. Technical Objective

Inventory, catalog, and define the extraction specification for the Default instance assets across the **4 Pillars of Antigravity IDE Parity** (Theme, Preset Mode, Official 4 Plugins, and 9 Builtin Skills + ~34 Plugin Skills). This creates the deterministic packaging blueprint required by both the backend instance parity engine (`instance.rs`) and the standalone fleet deployment script (`deploy-antigravity-ide-fleet.ps1`).

---

## 3. Implementation Blueprint

### 3.1 Pillar 1: Theme Extraction Specification

1. **Sources to Probe in Priority Order:**
   - Source A: `%APPDATA%\Antigravity\User\settings.json` (Primary active VS Code user settings)
   - Source B: `instances/default/home/AppData/Roaming/Antigravity/User/settings.json` (Default instance home profile)
   - Source C: `instances/default/data/User/settings.json` (Default instance data directory)
   - Source D: `%USERPROFILE%\.antigravity_tools\gui_config.json` (Antigravity Manager GUI config fallback)

2. **Keys to Extract:**
   - `"workbench.colorTheme"`: Extracted string value (e.g. `"Default Dark+"`, `"Tokyo Night Slate"`, `"Catppuccin Mocha"`). Default fallback if missing: `"Default Dark+"`.
   - `"workbench.colorCustomizations"`: Full JSON object containing title bar, status bar, and tab color overrides.
   - `"workbench.preferredDarkColorTheme"` & `"workbench.preferredLightColorTheme"` (if present).

3. **Database Theme Cache:**
   - In `<source>/User/globalStorage/state.vscdb`: Extract `ItemTable` rows where `key LIKE '%theme%'` or `key = 'workbench.theme'`.
   - Replicate via `safe_clone_sqlite_db` into target `<target>/User/globalStorage/state.vscdb`.

### 3.2 Pillar 2: Preset Mode & Security Policies Extraction

1. **Security Presets (`User/security_presets.json`):**
   - Extract or generate the canonical Turbo Mode definition:
   ```json
   {
     "active_preset": "turbo",
     "presets": {
       "turbo": {
         "name": "Turbo (Autonomous)",
         "description": "High-velocity execution with zero-trust bypass and auto-approvals",
         "auto_approve_commands": true,
         "auto_approve_file_edits": true,
         "bypass_command_confirmations": true,
         "zero_trust_bypass": true,
         "eager_execution": true
       }
     }
   }
   ```

2. **Antigravity Policies (`User/antigravity_policies.json` & `.gemini/policies/`):**
   - Extract policy rules granting unrestricted terminal execution and auto-approval.

3. **Workspace Trust & Settings Guardrails (`User/settings.json`):**
   - Mandatory keys to inject into target:
     - `"security.workspace.trust.enabled": false`
     - `"security.workspace.trust.startupPrompt": "never"`
     - `"security.workspace.trust.emptyWindow": false`
     - `"gemini.experimental.eagerExecution": true`
     - `"antigravity.securityPreset": "turbo"`
     - `"antigravity.autoApprove": true`
     - `"files.autoSave": "afterDelay"`

### 3.3 Pillar 3: Official 4 Plugins Inventory & Extraction

1. **Directory Location:**
   - Primary: `%USERPROFILE%\.gemini\config\plugins\`
   - Instance Secondary: `<source_home>\.gemini\config\plugins\`

2. **The 4 Official Plugins Manifest:**
   - **`chrome-devtools-plugin`**: Contains Chrome DevTools MCP server, a11y auditing, DOM inspection tools, and 5 debugging skills.
   - **`data-agent-kit-plugin`**: Contains BigQuery dataform/dbt pipelines, GCP tools, Airflow DAG migration tools, and 34 data skills.
   - **`google-antigravity-sdk`**: Contains Antigravity Agent runtime tools, SDK wrappers, and multi-agent coordination hooks.
   - **`modern-web-guidance-plugin`**: Contains modern web standards, Chrome extensions, and responsive UI guidance.

3. **Extraction Rules:**
   - Recursively copy each plugin directory into `<target_home>/.gemini/config/plugins/<plugin_name>/`.
   - Preserve all internal `skills/`, `plugin.json`, MCP manifests, and scripts.
   - Skip volatile lock files (`*.lock`, `lockfile`).

### 3.4 Pillar 4: Skills Inventory & Extraction

1. **9 Builtin Skills:**
   - Location: `%USERPROFILE%\.gemini\antigravity\builtin\skills\`
   - Catalog:
     1. `agy-customizations`: Rule loading and customization reference
     2. `antigravity_guide`: IDE documentation and workflow guide
     3. `automation`: Macro engine and script replay
     4. `generative_ui`: Interactive DOM widget rendering
     5. `migrate-workflows`: Legacy workflow migration
     6. `permissioned-github`: GitHub API operations
     7. `plugin`: Plugin management
     8. `ui-extension`: UI extension hooks
     9. `ui-plugin-navigation`: Navigation bar and toolbar extension
   - Target: `<target_home>/.gemini/antigravity/builtin/skills/`

2. **~34 Plugin Skills:**
   - Packaged directly within each plugin directory (`<target_home>/.gemini/config/plugins/<plugin_name>/skills/`).
   - Specifically verify that `data-agent-kit-plugin/skills` has all 34 skills (`bigquery_ai_ml`, `bigquery_sql`, `gcloud_auth_verification`, etc.).

---

## 4. Verification & Concrete Path Matrix

| Asset | Source Path Checked | Expected File / Directory Count | Target Destination |
|---|---|---|---|
| Theme | `%APPDATA%\Antigravity\User\settings.json` | 1 file (`workbench.colorTheme`) | Target User `settings.json` (both paths) |
| Presets | `security_presets.json` | 1 file (Turbo mode) | Target User `security_presets.json` |
| Policies | `antigravity_policies.json` | 1 file (Allow all) | Target User `antigravity_policies.json` |
| Plugins | `%USERPROFILE%\.gemini\config\plugins\` | 4 directories | Target `.gemini/config/plugins/` |
| Skills (Builtin) | `%USERPROFILE%\.gemini\antigravity\builtin\skills\` | 9 directories | Target `.gemini/antigravity/builtin/skills/` |
| Skills (Plugin) | `plugins/*/skills/` | ~42 total skill directories | Target `.gemini/config/plugins/*/skills/` |

---

## 5. Acceptance Criteria

- [x] Extraction rules account for missing files with deterministic defaults (`"Default Dark+"` theme and `"turbo"` preset).
- [x] All 4 official plugins and 9 builtin skills are explicitly named and inventoried with absolute path hierarchies.
- [x] Both user settings directories on Windows (`<data_dir>/User` and `<home_dir>/AppData/Roaming/Antigravity/User`) are targeted for deep-merge synchronization.
- [x] Subagent strictly adheres to TOTAL BAN ON GIT (no git commands executed).
