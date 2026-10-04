# Plan 111: Auto-Switcher Threshold Fix, Daemon Timer, 5s Prompt Restoration & Instance Card Projects

## User Request (Verbatim)
```text
https://prnt.sc/T6VdWWPcCD4f

# High Priority Instruction

When we are running multiple instances, when the threshold is lower than the, let's say, the threshold given for the account switch it still did not switch. That is one of the big issues and concern, which I asked you repeated times to test end to end to make sure that it works nicely. Okay. So first of all, the account's credit is low. It still did not switch to the new one. Okay? Even though the system has the information, so I'm not sure what it did. We are waiting for here for a long period of time. It didn't do it. So you should have something like the demon next check timer so that we understand when is the next, let's say, refresh or understanding or switch is going to happen. And also remember, during the switch, we need to back up the current running and enqueued prompts and restore back after five seconds of running the IDE instance. Okay? It needs to be injected onto that specific instance and make sure the code is efficient. Make sure the code is efficient, okay? I repeat that. That is very important. And anything you deliver, you make sure that you do the end-to-end testing here. You can write end-to-end test for local cases, but this will not run in the CI/CD. Also, this will not run later on in the local machine unless it was requested. Try to make these things, okay? Is it clear?

Also in the instance cards section, we should be able to see one or two projects, let's say three projects at max. And this can be changed from the settings, whichever is running the prompts or recent running. So we can also double click on this and open the prompts window for this project. So make sure that this enhancement is done. This will be very helpful. Okay? So this will happen right after we show the data and the exe, then there will be prompts running section. Okay? You can compact it a little bit, reduce the space a little bit. Space means the width a little bit less, or this one is also fine, not a problem. Or we can have more options like bigger cards and smaller cards in the above level. So smaller card will show more cards together. Okay? List is also nice. You can see the compact list now, which is very nice. Yeah, make things good. Now, the progress bar, I think it needs to be updated with the modern progress bar as I have described. You already have the idea, right? You can focus on the modern progress bar, I guess. Which you have the definition and everything else.

# Actionable Items Must Follow Non-Negotiable

1. Write a plan and spec first
2. Ensure the account switch occurs when the threshold is reached.
3. Implement a daemon to check the next switch timing.
4. Backup and restore prompts during the switch.
5. Conduct end-to-end testing.
6. Update the instance cards section to show up to three projects.
7. Enhance the prompts window functionality.
8. Update the progress bar to the modern version.

Must follow and spawn agent using 

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

/plan first before doing the work to reduce the credits.
```

## Visual References
- `![Model Selection in IDE](assets/screenshots/111-model-selection-in-ide.png)`
- `![Default Instance Low Quota](assets/screenshots/111-default-instance-low-quota.png)`
- `![Multi Instances Quota Status](assets/screenshots/111-multi-instances-quota-status.png)`
- `![Auto Switcher Settings Mismatch](assets/screenshots/111-auto-switcher-settings-mismatch.png)`

## Task Breakdown & Execution Waves

### Wave 1: Implementation
- **Worker 01 (Backend Specialist)**:
  - Owned Files:
    - `src-tauri/src/modules/auto_switcher.rs`
    - `src-tauri/src/modules/instance.rs`
    - `src-tauri/src/commands/instance.rs`
    - `src-tauri/src/models/config.rs`
    - `src-tauri/tests/auto_switcher_e2e_test.rs`
  - Scope:
    1. Eliminate outdated `3.1`/`3.0` model bans in `auto_switcher.rs`.
    2. Evaluate lowest active model quota against threshold, enabling automatic rotation when Gemini 3.1 Pro (or any active model) drops <= threshold.
    3. Implement `AutoSwitcherDaemonStatus`, store `next_check_timestamp` in `RUNTIME_STATE`, emit live countdown events, and expose `get_auto_switcher_daemon_status` command.
    4. Implement non-blocking 5-second asynchronous delayed prompt restoration post-launch (`tokio::time::sleep(5s)`).
    5. Write isolated local integration test `src-tauri/tests/auto_switcher_e2e_test.rs` with `#[ignore]`.

- **Worker 02 (Frontend Specialist)**:
  - Owned Files:
    - `src/pages/Instances.tsx`
    - `src/components/instances/PromptTreeViewModal.tsx`
    - `src/pages/Settings.tsx`
    - `src/components/settings/AutoSwitcherSettings.tsx`
    - `src/services/instanceService.ts`
    - `src/types/config.ts`
  - Scope:
    1. Render 1 to 3 recent/running projects on instance cards with turn count and active indicator directly below data & exe paths.
    2. Add double-click handler on project chips to open `PromptTreeViewModal` auto-focused and expanded on that project (`initialSelectedProjectId`).
    3. Add card density toggle (`Normal Cards` vs `Compact Cards`) in toolbar.
    4. Display live daemon next-check countdown timer in Instances toolbar and Auto Switcher Settings.
    5. Add `instance_card_max_projects` (1–3, default 3) setting in Settings and `AppConfig`.
    6. Ensure modern `WaterDrainProgressBar` renders with correct `#1af18d` to `#12b27d` gradient and milestone checkpoints.

### Wave 2: Verification, Pre-flight Checks & Atomic GitMap Commit
- **Lead Orchestrator**:
  - Run `cargo fmt -- --check`.
  - Run `npm run build`.
  - Validate local integration test.
  - Execute atomic GitMap commit and push: `gitmap cpf "auto-switcher - quota threshold fix, daemon timer, 5s prompt restore, and instance card projects"`.
