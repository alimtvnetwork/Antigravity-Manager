# Specification: 104 - Instance Settings Sync, Duplication Parity, Folder Copy & Header Button Consolidation

## 1. Executive Summary & Problem Classification
- **Domain**: Antigravity Manager (AGM) Multi-Instance Engine, Settings Synchronizer, CLI Automation & Frontend UI Controls.
- **Problem Classification**: Feature & Architectural Enhancement.
- **Status**: IN_PROGRESS
- **Target Release**: Minor Version Bump (`v4.127.0`)

---

## 2. User Request (Verbatim)
```text
Combine the buttons 


# High Priority Instruction

Okay. So I want to have a few options, like when you make a duplicate using the CLI or from the UI, basically it's going to call the CLI methods. So whatever the CLI route method it's going to use, it's going to use the same method from the UI so that the testing will be same. Now, when we do the, let's say, back to load the Supabase database from the repo secrets, that's another thing, okay, using the IDE. Okay. Then, I think you need to have the option from the CLI to copy these settings from one IDE to another. That would include the theme colors, the approval of the turbo mode, policy, code review, readable file, writable folder, everything from the settings, one IDE to another, and we should be able to see how many IDE instances we should have. Okay? And we can always give a path and ask it to do that. If we give an executable path and say to do that, it would know where that is and automatically put that settings in. Okay? That's one thing. Another is if we do not use the-- I mean, sorry. That is like when we don't have the copy, we can also copy this. And also we should have a default settings that we can enforce on all instances or single instance. We should have all these commands from the CLI level. Now, the way that it would work, the default instance is the default installation, which already have the settings like turbo mode on and things like that, and that should be as a by default. I could enable all turbo mode for all the instances. I could do plan review, always proceed for all instances or single instance. So try to have these things, and also copy option from one instance settings to another, and also the default settings. Default settings is the default settings that I have. Try to have this everything together, and make it to JSON import/export so that anyone can use the default settings. We should also have these options inside the settings, inside the instances. We could just go into instances and say copy settings from one to another. All kinds of things. We could see existing settings, copy settings, copy button and paste it, and it would apply to the settings. We can undo, redo the settings that we have done. And also we can see the common settings like the turbo mode, browser execution policy, things like that. So we should be able to apply all of these in all of these IDEs, things like that. And also the folder copy. Folder copy means, let's have the first IDE, whatever the projects are included, we should be able to copy those to the duplicated instances. And if we don't, we should also have the CLI option to do it manually from the CLI, also from the UI, we should have these options. These are non-negotiable. So first, list out all these tasks and then try to do it. Okay? And then finally do a release bump in the minor and make a release. Is it clear?

# Actionable Items Must Follow Non-Negotiable

1. Write a plan and spec first.
2. Implement CLI and UI methods for duplication.
3. Enable CLI options for copying settings between IDEs.
4. Ensure default settings can be enforced across instances.
5. Develop JSON import/export for settings.
6. Implement copy and paste functionality for settings.
7. Provide CLI and UI options for folder copy.
8. List all tasks and execute them.
9. Perform a minor release bump and make a release.
```

---

## 3. Architectural Specifications

### 3.1 Navbar Header Button Consolidation (UI Modernization)
- **Problem**: In `media_1791013550119.png`, action buttons (Quick Clean `↺` and Theme/Language `🌙 EN ⌵`) and window controls (Minimize `—`, Maximize `🗗`, Close `✕`) appear as separate, isolated circular buttons floating with disjointed gaps.
- **Solution**:
  - Group 1 (Quick Clean + Theme/Language): Combined into a single cohesive pill group with an outer border, subtle divider border between the clean button and dropdown, and continuous backdrop blur.
  - Group 2 (Window Controls): Combined into a single contiguous window control strip with joined borders, uniform padding, and individual hover states (including destructive red hover on the close button).

### 3.2 CLI & UI Duplication Parity (`instance::clone_instance`)
- **Single Source of Truth**: Both the Tauri command `clone_instance` (used by the frontend) and the CLI command `agm instance duplicate` invoke the exact same core function in `src-tauri/src/modules/instance.rs`.
- **Project & Folder Copy**:
  - In Antigravity/VS Code, open workspace projects and recent folders reside in:
    - User storage: `<data_dir>/User/workspaceStorage/`
    - Global storage: `<data_dir>/User/globalStorage/storage.json`
    - State database: `<data_dir>/User/globalStorage/state.vscdb` (containing `history.recentlyOpenedPathsList`)
  - When folder copy is selected (or via `agm instance copy-projects`), copy these paths while rewriting relative references to the target instance directory.

### 3.3 Deep Settings Synchronization (`copy_instance_settings`)
- **Settings Target Files**:
  - `<data_dir>/User/settings.json`:
    - `workbench.colorTheme`, `workbench.preferredDarkColorTheme`, `workbench.colorCustomizations`
    - `antigravity.turboMode`, `antigravity.planReviewAlwaysProceed`
    - `antigravity.browserExecutionPolicy`, `antigravity.codeReviewPolicy`
    - `antigravity.readableFiles`, `antigravity.writableFolders`
  - Extraction: Read `settings.json` and relevant keys from source instance.
  - Injection: Deep-merge settings into destination instance `settings.json`, preserving existing user customizations unless overridden.
  - Executable Path Auto-Discovery: If an executable path is provided (`--exe <path>`), inspect registered instances and match `executable_path` or derive instance data directory.

### 3.4 Default Settings Enforcement & Fast Toggles
- **Default Reference**: The `default` instance's `settings.json` serves as the golden baseline.
- **CLI Commands**:
  - `agm instance count`: Output number of registered instances and their statuses.
  - `agm instance copy-settings --from <src> --to <dest> [--exe <path>]`: Copy settings.
  - `agm instance copy-projects --from <src> --to <dest>`: Copy recent project folders.
  - `agm instance settings enforce-defaults [--all | --instance <id>]`: Enforce baseline settings.
  - `agm instance settings set-turbo [--all | --instance <id>] [--enable/--disable]`: Fast toggle turbo mode.
  - `agm instance settings set-plan-review [--all | --instance <id>] [--always-proceed/--ask]`: Fast toggle plan review.
  - `agm instance settings export [--instance <id>] --out <file.json>`: Export settings JSON.
  - `agm instance settings import [--instance <id> | --all] --file <file.json>`: Import settings JSON.

### 3.5 Frontend Settings Modal & Clipboard Sync with Undo/Redo
- In `src/pages/Instances.tsx` and a dedicated `InstanceSettingsModal.tsx`:
  - **Copy Settings**: Serializes instance settings to JSON and copies to system clipboard.
  - **Paste Settings**: Parses JSON from clipboard and applies to target instance with confirmation.
  - **Undo / Redo**: In-memory and local storage undo/redo stack (`history: SettingsSnapshot[]`, `historyIndex: number`).
  - **Common Toggles**: Quick switches for Turbo Mode and Plan Review with "Apply to All Instances" button.
  - **JSON Import / Export**: File download / upload buttons.

---

## 4. Verification Acceptance Criteria
1. **AC-1 (Navbar UI)**: Header action buttons and window controls rendered as joined, cohesive pill button groups.
2. **AC-2 (CLI/UI Parity)**: UI duplication and CLI `agm instance duplicate` execute identical Rust core function.
3. **AC-3 (Settings Copy)**: Theme, turbo mode, browser policy, and review settings successfully copied across instances.
4. **AC-4 (Default Enforcement)**: Default template settings cleanly applied across single or all instances.
5. **AC-5 (JSON Import/Export)**: JSON export produces valid schema; import updates `settings.json` without corruption.
6. **AC-6 (Copy/Paste & Undo/Redo)**: Clipboard copy/paste works seamlessly; undo restores previous snapshot.
7. **AC-7 (Folder Copy)**: Recent projects copied from source to destination instance.
8. **AC-8 (Pre-flight & Release)**: All pre-flight gates pass; release bumped to minor version `v4.127.0`.
