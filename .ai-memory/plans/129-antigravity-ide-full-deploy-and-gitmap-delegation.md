# Master Plan: 129-antigravity-ide-full-deploy-and-gitmap-delegation

## User Request (Verbatim)

```text
1. Default instance theme not applied to instances
2. Preset mode not copied from default instance
3. Plugins are not applied, skills are not added
4. Standalone PowerShell deployment script in repo-secrets folder properly
5. Modify GitMap to enable future delegation for fully deploying Antigravity IDE to another machine automatically.
```

## Actionable Items (Must Follow Non-Negotiable)

1. **Write Specs & Master Plan First:** Author architecture spec, component spec, root cause analysis, master plan, and granular subtask files before applying any functional modifications.
2. **Strict Search Boundary:** Search codebase exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`, `gitmap ps`, `gitmap py`); TOTAL BAN on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`.
3. **Total Ban on Git for Subagents:** Subagents must NEVER run `git add`, `git commit`, `git status`, `git diff`, or `git checkout`. All version control operations are handled strictly at final consolidation via `gitmap cpf`.
4. **4 Pillars of Antigravity IDE Parity:** Enforce 100% parity across Theme (`workbench.colorTheme`, `workbench.colorCustomizations`), Preset Mode (Turbo Mode, Eager execution, zero-trust workspace security bypass), Plugins (official 4 plugins), and Skills (9 builtin + ~34 plugin skills).
5. **Standalone PowerShell Deployment Script:** Create a production-grade, unattended deployment script `deploy-antigravity-ide-fleet.ps1` in `d:\work\repo-secrets\02-antigravity-manager\scripts\` and mirror to `scripts/`.
6. **GitMap Remote Delegation Integration:** Extend GitMap CLI cluster orchestration (`gitmap cluster run-script`, `gitmap ssh exec`, `gitmap agy ssh`) to enable single-command remote deployment of Antigravity IDE to any fleet node (`w1`, `w2`, `w3`, `u1`, `final-network-machine`).
7. **Antigravity Manager Backend Parity Engine:** Overhaul `src-tauri/src/modules/instance.rs` (`copy_instance_with_options`, `create_instance_with_account`, `copy_source_ide_trees`, and `enforce_default_settings`) to guarantee deep-merge synchronization of settings, themes, presets, plugins, and skills for all newly created or cloned instances.
8. **End-to-End Simulation & Verification:** Verify the entire deployment pipeline using a dry-run test harness to prove that child instances and remote machines inherit complete configuration without regressions.

---

## 1. Executive Summary & Root Cause Synthesis

### 1.1 Root Cause 1: Theme Loss on Instance Duplication & Creation
- **Root Cause:** When new instances are created via `create_instance_with_account`, only the default data directory's `User/settings.json` was copied. However, the user's active theme was set either in `%APPDATA%\Antigravity\User\settings.json`, `gui_config.json`, or the Default instance home directory. Furthermore, settings copy routines frequently replaced `settings.json` with only `window.title`, dropping `"workbench.colorTheme"`.
- **Solution:** Create `sync_instance_ide_parity` in `instance.rs` to inspect all default sources for `workbench.colorTheme`. If missing, inject the configured default theme (`"Default Dark+"` or user preset), and write it to both `<data_dir>/User/settings.json` and `<home_dir>/AppData/Roaming/Antigravity/User/settings.json`.

### 1.2 Root Cause 2: Preset Mode & Security Policies Omission
- **Root Cause:** Antigravity IDE requires `User/security_presets.json` and `User/antigravity_policies.json` to enable Turbo Mode / Autonomous execution. During `create_instance_with_account`, these files were only copied if they already existed in the default directory. If absent, the new instance defaulted to strict interactive prompts. Workspace trust was also not explicitly disabled.
- **Solution:** Unconditionally generate and copy `security_presets.json` configured for Turbo mode (`"active_preset": "turbo"`, `"auto_approve_commands": true`, `"eager_execution": true`, `"bypass_command_confirmations": true`) and set `"security.workspace.trust.enabled": false` in all instance settings.

### 1.3 Root Cause 3: Plugins & Skills Missing in Child Instances
- **Root Cause:** Antigravity IDE runtime resolves plugins and builtin skills from `$HOME/.gemini/config/plugins` and `$HOME/.gemini/antigravity/builtin/skills`. In secondary instances, `$HOME` is redirected to `instances/<id>/home`. During instance creation, `copy_source_ide_trees` was never called, leaving the instance's home `.gemini` folder devoid of the 4 official plugins and 9 builtin skills.
- **Solution:** Enhance `copy_source_ide_trees` to copy the 4 official plugins (`chrome-devtools-plugin`, `data-agent-kit-plugin`, `google-antigravity-sdk`, `modern-web-guidance-plugin`) and 9 builtin skills into the target instance home directory on creation, duplication, and initial launch backfill.

### 1.4 Root Cause 4: Absence of Standalone Deployment Script
- **Root Cause:** Deploying the IDE stack onto a new node previously required running manual PowerShell snippets or ad-hoc copies from developer machines.
- **Solution:** Author `deploy-antigravity-ide-fleet.ps1` in `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/` with support for local execution and remote SSH streaming, positive boolean switches, and automated health checks.

### 1.5 Root Cause 5: Missing GitMap Remote Delegation Flow
- **Root Cause:** GitMap CLI contains `gitmap cluster exec` and `gitmap agy ssh`, but lacked a dedicated Antigravity IDE deployment recipe to package the installer, themes, presets, plugins, and skills for automatic remote machine deployment.
- **Solution:** Integrate GitMap remote script execution via `gitmap cluster run-script <target> scripts/deploy-antigravity-ide-fleet.ps1` and document standard delegation recipes.

---

## 2. Research Findings & Concrete Asset Inventory

| Asset Category | Source Location | Target Location | Contents / Details |
|---|---|---|---|
| **Official Plugins (4)** | `%USERPROFILE%\.gemini\config\plugins\` | `<instance_home>\.gemini\config\plugins\` | `chrome-devtools-plugin`, `data-agent-kit-plugin`, `google-antigravity-sdk`, `modern-web-guidance-plugin` |
| **Builtin Skills (9)** | `%USERPROFILE%\.gemini\antigravity\builtin\skills\` | `<instance_home>\.gemini\antigravity\builtin\skills\` | `agy-customizations`, `antigravity_guide`, `automation`, `generative_ui`, `migrate-workflows`, `permissioned-github`, `plugin`, `ui-extension`, `ui-plugin-navigation` |
| **Plugin Skills (~34)** | `<plugin>\skills\` | `<instance_home>\.gemini\config\plugins\*\skills\` | 34 BigQuery/GCP skills in `data-agent-kit-plugin`, 5 in `chrome-devtools-plugin`, 1 in `google-antigravity-sdk`, 2 in `modern-web-guidance-plugin` |
| **VS Code Theme** | `%APPDATA%\Antigravity\User\settings.json` | `<instance_data>\User\settings.json` & `<instance_home>\AppData\Roaming\Antigravity\User\settings.json` | `"workbench.colorTheme": "Default Dark+"` (or Tokyo Night, Catppuccin, etc.) |
| **Security Presets** | `%USERPROFILE%\.antigravity_tools\security_presets.json` | `<instance_data>\User\security_presets.json` & `<instance_home>\AppData\Roaming\Antigravity\User\` | Turbo mode, eager execution, auto-approve commands, zero-trust bypass |
| **State Database** | `<source>\User\globalStorage\state.vscdb` | `<target>\User\globalStorage\state.vscdb` | Safe-cloned via `safe_clone_sqlite_db` to preserve window state & theme seeds |

---

## 3. Phase Breakdown & Execution Roadmap

### Phase 1: Default Instance Inventory & Extraction (Task-02)
- Inspect Default instance configuration, themes, presets, plugins, and skills.
- Define the canonical manifest and extraction rules for all IDE assets.

### Phase 2: Standalone Fleet Deployment Script (Task-03)
- Author `deploy-antigravity-ide-fleet.ps1` in `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/`.
- Support parameters: `$TargetHost`, `$TargetUser`, `$Preset`, `$Theme`, `$DeployPlugins`, `$DeploySkills`, `$DryRun`, `$Force`.
- Implement positive boolean normalization, idempotent execution, and automated verification pings.

### Phase 3: GitMap Remote Delegation Integration (Task-04)
- Formulate GitMap remote delegation lifecycle using `gitmap cluster run-script`, `gitmap ssh exec`, and `gitmap agy ssh`.
- Register cluster recipes for deploying Antigravity IDE to remote fleet machines (`w1`, `w2`, `w3`, `u1`, `final-network-machine`).

### Phase 4: Antigravity Manager Backend Parity Engine (Task-05)
- Update `src-tauri/src/modules/instance.rs`:
  - Enhance `copy_instance_with_options` to ensure complete theme, preset, plugin, and skill replication.
  - Enhance `create_instance_with_account` to invoke `sync_instance_ide_parity`.
  - Add backfill migration in `load_registry` or `launch_instance_inner` to repair existing incomplete instances.
- Update frontend `src/pages/Instances.tsx` to display preset status and theme badges.

### Phase 5: Verification & End-to-End Simulation (Task-06)
- Execute syntax and compilation checks (`cargo clippy`, `cargo fmt`).
- Run test suite for instance cloning and settings sync (`src-tauri/tests/instance_cloning_and_sync_test.rs`).
- Simulate deployment script execution in staging environment.

### Phase 6: Plan Consolidation & Single Atomic GitMap Push (Task-07)
- Move plan to `.ai-memory/plans/completed/`.
- Update spec and plan indexes.
- Perform secrets gate scan.
- Lead executes single consolidated atomic commit via `gitmap cpf`.

---

## 4. Task Decomposition Matrix

| Task Code | Subtask Title | Assigned Agent | Owned Files | Status |
|---|---|---|---|:---:|
| **Task-01** | Architecture Specification, Component Spec & RCA | Lead / Spec Authors | `02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/01-architecture-spec.md`<br>`02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/02-component-spec.md`<br>`02-spec/21-app/129-antigravity-ide-full-deploy-and-gitmap-delegation/03-root-cause-analysis.md`<br>`.ai-memory/plans/129-antigravity-ide-full-deploy-and-gitmap-delegation.md` | IN PROGRESS |
| **Task-02** | Default Instance Theme, Presets, Plugins & Skills Extraction | Worker 01 | `.ai-memory/plans/subtasks/129-antigravity-ide-full-deploy-and-gitmap-delegation/01-default-instance-theme-preset-plugin-skills-extraction.md`<br>`src-tauri/src/modules/instance.rs` | READY |
| **Task-03** | Standalone PowerShell Fleet Deployment Script in repo-secrets | Worker 01 | `.ai-memory/plans/subtasks/129-antigravity-ide-full-deploy-and-gitmap-delegation/02-powershell-fleet-deploy-script-in-repo-secrets.md`<br>`d:\work\repo-secrets\02-antigravity-manager\scripts\deploy-antigravity-ide-fleet.ps1`<br>`scripts/deploy-antigravity-ide-fleet.ps1` | READY |
| **Task-04** | GitMap CLI Remote Delegation Command & Recipe Integration | Worker 02 | `src-tauri/src/modules/gitmap.rs`<br>GitMap cluster delegation scripts | QUEUED |
| **Task-05** | Antigravity Manager Copy Instance Theme, Presets, Plugins & Skills Parity | Worker 02 | `src-tauri/src/modules/instance.rs`<br>`src/pages/Instances.tsx` | QUEUED |
| **Task-06** | Verification & End-to-End Deployment Simulation Suite | Worker 02 | `scratch/verify_ide_deployment.py`<br>`src-tauri/tests/` | QUEUED |
| **Task-07** | Plan Consolidation & Single Atomic GitMap Push | Lead | `.ai-memory/plans/completed/129-antigravity-ide-full-deploy-and-gitmap-delegation.md` | QUEUED |

---

## 5. Success Verification Checklist

- [ ] `01-architecture-spec.md` captures all 5 user requirements verbatim with 0 ambiguity.
- [ ] 4 Pillars of Parity (Theme, Preset, Plugins, Skills) are documented with exact file paths.
- [ ] `deploy-antigravity-ide-fleet.ps1` is deployed to both `d:\work\repo-secrets\02-antigravity-manager\scripts\` and `scripts/`.
- [ ] GitMap remote delegation lifecycle (`cluster run-script`, `ssh exec`, `agy ssh`) is fully articulated.
- [ ] `copy_instance_with_options` and `create_instance_with_account` copy themes, presets, plugins, and skills without manual intervention.
- [ ] Cargo clippy, cargo fmt, and frontend build pass with zero warnings/errors.
