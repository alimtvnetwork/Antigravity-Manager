# Plan 117: Deep Instance Cloning, Security Presets, Settings Sync, and Card Mutex UI Overhaul

## Status: IN_PROGRESS
## Parent Task: 117-instance-clone-settings-security-presets-and-card-progress-ui

---

## User Request (Verbatim)

```text
# High Priority Instruction

Okay. A couple of serious issues. First of all, when we clone the account, the EXE from the tooling, it does not clone the settings, does not clone the projects, basically does not even have the security presets. It's very terrible. I'm not sure how you approve of this. And the settings section of the instance settings section, the UI is broken in several places. I asked you to use icons. Use proper icon, use less text. You did not do it properly. Make sure that you do it properly and respect that. Many things are broken, I really don't know. And when you are cloning, how the hell it can be cloned without the theme, settings, and everything, and without the projects? How? We have just selected it. So it's a serious bug I think you need to work on. And you need to also do the end-to-end testing and make sure the UI is proper and then apply it. Does this make sense?

Also, when we click on delete, delete should show up a confirmation dialog within the UI, not the Windows one. That's the first thing. Second is that when we click on the play or pause button or stop button, it should disable that card or that row to not to click any other item, and it should feel like something is going on as a progress. You need to improve that UI for the cards and other stuff according to this. You need to fix this part, please

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Fix the cloning process to ensure it includes settings, projects, and security presets.
4. Repair the UI in the settings section to use proper icons and less text.
5. Conduct end-to-end testing to ensure the UI is proper.
6. Implement a confirmation dialog within the UI for the delete action.
7. Update the UI to disable the card or row when play, pause, or stop buttons are clicked, indicating progress.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]
```

---

## Subtask Breakdown

- [ ] **Task-01**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/001-backend-instance-id-resolution-and-sqlite-ro-cloning.md`
  - In `src-tauri/src/modules/instance.rs`, `commands/instance.rs`, and `cli.rs`: call `resolve_instance_id` to reliably resolve aliases (`"default"`, `"Default"`, sequence numbers). Update `safe_clone_sqlite_db` with read-only URI mode and incremental backup retry loops to safely clone locked databases while Antigravity is running.
- [ ] **Task-02**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/002-backend-complete-settings-sync-and-security-presets.md`
  - Deep-copy ALL `settings.json` keys, `keybindings.json`, `snippets/`, `security_presets.json`, `antigravity_policies.json`, and `.gemini/policies` (with fallback to user home) during instance cloning and settings synchronization.
- [ ] **Task-03**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/003-backend-projects-recent-paths-and-workspace-storage.md`
  - In `merge_state_vscdb_recent_paths` and `copy_instance_projects`, open SQLite in read-only URI mode, migrate modern VS Code recent paths (`history.recentlyOpenedPathsList`), and clone `workspaceStorage/` folders with resilient error handling.
- [ ] **Task-04**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/004-frontend-instance-settings-modal-icon-capsules-polish.md`
  - Redesign `InstanceSettingsModal.tsx`: replace verbose toggle cards with compact segmented pill switches and Lucide icons, eliminate dropdown menu overlap in replication bar, and unify JSON tools into a segmented capsule.
- [ ] **Task-05**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/005-frontend-in-app-delete-dialog-and-card-mutex-overlay.md`
  - In `src/pages/Instances.tsx` and `src/components/instances/InstanceTable.tsx`: ensure in-app React modal for delete with profile details; add full card glass mutex overlay and row-level progress indicator when Play, Pause, or Stop buttons are triggered.
- [ ] **Task-06**: `subtasks/117-instance-clone-settings-security-presets-and-card-progress-ui/006-testing-preflight-and-verification.md`
  - Run frontend `npm run build`, `cargo fmt -- --check`, isolated unit/integration tests, and atomic GitMap commit.
