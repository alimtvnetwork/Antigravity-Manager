# Plan: Antigravity Manager and GitMap Command Enhancements

## User Request (Verbatim)

```text
# Antigravity Manager and GitMap Command Enhancements: high priority instruction, non-negotiable task

For Antigravity Manager and `gitmap`, both should have commands to export the Antigravity pin projects with the relative paths from the default work directory. That means default work directory can be switched. So it should work like that. So you need to have new commands where the pin projects, where not only the pin, the last running prompts, specific prompts can be compacted to a SQLite database. We can use a JSON and a SQLite DB to export with conversation and the projects with pin projects and also without pin projects. It can be absolute path or it could be a relative path from the work directory. That means we can change the work directory at any time, at any given moment, and it should work. Based on that, you prepare a system design plan, commands with example, add the help text to the UI as well, and then implement it. And also try to have this feature from the UI for the Antigravity Tools Manager, especially in the instance section. Also, we can have an import, export Antigravity section, I think from the dropdown of the settings. Not in the settings especially, but the dropdown where we have all this email and other stuff, we can put it there. And also in that dropdown for the Antigravity, we can have the update option checked. Where is the update option? And also add a search button so that we can type and filter out the menu items. And for the instance and accounts, try to use only icon. We don't need to see any text. So the space is saved. And also we can have a great tooltip when we hover over. The tooltip will tell us what that is. Add a professional tooltip. And also, the UI level, when we go to the prompts history, that section takes a long time to load. Try to investigate that. If it takes a long time, then add a spinner or something like this, and try to have a caching so that you can display something and then you can push new stuff. And mostly you need to make the right-hand side more compact. You already improved, I think, in the latest version, but I want some of the buttons to be compacted. For example, the send, enqueue, these buttons can be compacted. Try to focus on that and make sure every change that you do, you make sure that you pull first and then do the commit, and also make sure that you resolve the conflict because there are multiple agents who are working at the same time. And also at the same time, for the items, try to have as many as icon less than text. But some items are not very meaningful. For example, the copy section can have a dropdown. So one would be default the copy text, copy icon and then text, and then we can just click on the dropdown to see copy with text and image, things like that. So compact as much as possible so that it looks nice, and also make sure the backup restore works very well. You can do end-to-end testing properly with the CLI commands. Check what is working, what is not. I suspect most of the things are not working very well. You can do a deep dive, try to understand what is wrong. Do not change anything in your current instance or other running instances. What you should do is create your own instance and try to deal with that for the end-to-end testing.

## slug: antigravity-manager-and-gitmap-command-enhancements

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/antigravity-manager-and-gitmap-command-enhancements/ and enqueue plan task in .ai-memory/plans/antigravity-manager-and-gitmap-command-enhancements.md (subtasks in .ai-memory/plans/subtasks/antigravity-manager-and-gitmap-command-enhancements/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Prepare a system design plan and implement new commands for exporting Antigravity pin projects with relative paths, including UI enhancements and tooltips, and ensure end-to-end testing with CLI commands.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## 1. System Design Plan & Architectural Blueprint

### 1.1 Portable Relative Path & Switchable Work Directory Model
1. **Dynamic Active Work Directory Resolution**:
   - Query in order of priority:
     1. Command parameter `--work-dir <path>` or flag `--relative-to <path>`.
     2. Environment variable `AGM_WORK_DIR` or `WORK_DIR`.
     3. Active default work directory configured in GitMap (`%LOCALAPPDATA%\gitmap-cli\data\gitmap.db` -> `work_directories WHERE is_default = 1`).
     4. Current process working directory ancestor containing git repositories.
     5. Platform default (`D:\work` on Windows, `~/work` on Unix).
2. **Relative Path Transformation Invariant**:
   - Every workspace / project path is stored with both:
     - `relativePath`: Path relative to the active work directory (e.g. `Antigravity-Manager` or `subgroup/project-a`).
     - `pathTemplate`: Template with variable interpolation `${workDir}\Antigravity-Manager`.
     - `isRelative`: Boolean indicating relative path normalization.
   - When importing on a machine or environment with a different work directory (e.g. switched to `E:\dev`), all relative paths dynamically expand against the newly selected work directory.

### 1.2 Dual-Engine Compaction (SQLite & JSON)
1. **Compacted SQLite Database (`.db`)**:
   - Lightweight, standalone SQLite database (`meta`, `pinned_projects`, `conversations`, `running_prompts`, `prompt_history`).
   - Can be shared, inspected via standard SQLite tools, or imported directly into AGM / GitMap.
2. **Compacted JSON Envelope (`.json`)**:
   - Conforms to Antigravity Manager JSON Envelope v2.0 (`agm/compact-export`).
   - Supports 2-pass variable resolution (`${workDir}`, `${repoDir}`).
   - Contains complete metadata, CLI import/export invocation commands, and human-readable comments.

### 1.3 CLI Command Architecture & Examples

| Command Line | Mode | Description |
|---|---|---|
| `agm export-compact --file vault/export.db` | SQLite | Compact all pinned projects, running prompts, and prompt history into a portable SQLite database using relative paths. |
| `agm export-compact --file vault/export.json --work-dir D:\work` | JSON | Compact projects and prompts to JSON envelope v2.0, normalizing paths relative to specified work directory. |
| `agm export-compact --file vault/export.json --no-pinned` | JSON | Export running prompts and conversations without pinned projects. |
| `agm export-compact --file vault/export.json --absolute` | JSON | Export with absolute paths instead of relative paths if explicit static paths are requested. |
| `agm import-compact vault/export.db -y` | SQLite | Restore pinned projects, running prompts, and history into AGM databases, dynamically mapping to active work directory. |
| `agm import-compact vault/export.json --work-dir E:\repos -y` | JSON | Restore from JSON into a switched work directory `E:\repos`. |
| `agm which-format vault/export.db` | Inspect | Inspects database structure, counting pinned projects, prompts, and schema version. |

### 1.4 Frontend UI Modernization & Performance Plan
1. **Header Navigation Space Saving**:
   - Convert `Accounts` and `Instances` buttons in `src/components/navbar/NavMenu.tsx` to icon-only buttons (`Users` and `Laptop`).
   - Attach dark-glass accessible tooltip bubbles (`Accounts (Profile Vault)`, `Instances (Engine Runner)`).
2. **Dropdown Menu Search & Update Check**:
   - Add a sticky search input at the top of the dropdown menu in `NavMenu.tsx` to live-filter all navigation routes (`Email & Alerts`, `API Proxy`, `Audit`, etc.).
   - Ensure the "Check for Updates" action is prominently displayed at the top of the menu with clear version checking feedback.
3. **Prompt Tree Inspector Panel Compaction**:
   - In `DetailHeader.tsx`:
     - Combine `Copy Text`, `Copy With Images`, `Save Images`, and export options into a clean split dropdown button with default one-click `Copy Text`.
     - Compact `Send Now` with icon `<RotateCw />` + `Send` + `<kbd>N</kbd>`.
     - Compact `Enqueue` into icon-only `<ListPlus />` button with tooltip.
     - Compact `Focus IDE` into icon-only `<ExternalLink />` button.
     - Compact `Full` inspector into icon-only `<Maximize2 />` button.
4. **Prompt Tree History Loading Performance**:
   - Fix `isForce: true` bypass bug in `usePromptTreeData.ts` so that SQLite `prompt_tree_cache` is utilized.
   - Introduce an instant in-memory Stale-While-Revalidate (SWR) cache so reopening the modal renders instantly (0ms) from cache before background revalidation.
   - Replace false-empty "No Projects Found" flash in `TreePanel.tsx` with a loading spinner / skeleton.
5. **Instance Management Import/Export**:
   - Add Import / Export action pill to `src/pages/instances/InstancesHeader.tsx` triggering `UnifiedBackupModal.tsx`.
   - Add help text cards and CLI command examples in the UI.

---

## 2. Discrete Task Breakdown

- **Task-01**: Specifications, Architecture & Task Subtask Ledger (Lead + Subagents).
- **Task-02**: Core Rust Backend: Relative Workdir Engine, SQLite/JSON Compact Export & Import Modules (`src-tauri/src/modules/repo_db/compact.rs`, `src-tauri/src/modules/json_envelope/`).
- **Task-03**: CLI Commands & Dispatch: Wire `export-compact` and `import-compact` in `src-tauri/src/bin/agm/` with comprehensive terminal help text.
- **Task-04**: UI Header & Dropdown Modernization: Icon-only Accounts/Instances with rich tooltips, menu search filter, and update status visibility in `NavMenu.tsx`.
- **Task-05**: Prompt Tree Inspector Compaction & SWR History Loading Cache: Split dropdown copy button, compact action controls, instant SWR cache, and loading spinner in `DetailHeader.tsx` and `usePromptTreeData.ts`.
- **Task-06**: Instance Management Import/Export Integration & In-App Documentation: Wire `UnifiedBackupModal` in `InstancesHeader.tsx` and add CLI help guide.
- **Task-07**: Multi-Agent Git Reconcile, Targeted Linters, Isolated E2E Verification & Atomic GitMap Commit/Push.

---

## 3. Strict Invariants Checklist
- [x] Exclusive GitMap searches (`gitmap aum search`, `gitmap find`, `gitmap lf`, `gitmap cat`). Total ban on `rg`, `ripgrep`, `grep`, `Select-String`.
- [x] Strict relative Git paths across all specs, plans, code, and release notes.
- [x] Zero solo execution: spawn subagents via `invoke_subagent` (A = 2, H = 2).
- [x] Zero index lock collisions: subagents never run `git` commands.
- [x] Atomic GitMap commit with hyphen separator (`gitmap cpf "<module> - <summary>"`).
