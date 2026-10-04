# Plan: 115 — Instance Clone Settings Sync and Card UI Polish

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

## Traceable Subtasks

- **Task-01**: Deep codebase research on instance cloning engine, security presets, and modal layout via `research` subagents.
- **Task-02**: Overhaul `copy_instance_with_options` in `src-tauri/src/modules/instance.rs` to guarantee complete duplication of settings, keybindings, state DB, themes, workspace projects, and security presets.
- **Task-03**: Redesign `src/components/instances/InstanceSettingsModal.tsx` to fix button overlap, eliminate horizontal scrollbars, and convert verbose text into concise icon capsules.
- **Task-04**: Build and wire in-app `InstanceDeleteModal.tsx` in `src/pages/Instances.tsx` and `InstanceTable.tsx`, retiring native Windows confirm boxes.
- **Task-05**: Implement card/row locking and action progress overlay during play/pause/stop/switch in `src/pages/Instances.tsx` and `InstanceTable.tsx`.
- **Task-06**: Write E2E integration test `src-tauri/tests/instance_cloning_and_sync_test.rs` asserting settings, project, and preset preservation.
- **Task-07**: Run pre-flights (`cargo fmt -- --check`, `npm run build`) and push atomic GitMap commit.
