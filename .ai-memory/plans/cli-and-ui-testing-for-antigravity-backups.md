# Parent Plan: CLI and UI Testing for Antigravity Backups

## User Request (Verbatim)

```text
agm running --json # list prompts , instances
gitmap agy running --json
gitmap agy ff
gitmap agy backup running-prompts (rp) # + images
gitmap ag restore running-prompts (rp)
gitmap ag restore running-prompts (rp) $instance
agm backup running $instacne
agm restore running $instacne

Make sure you release with a minor bump at the end and make sure CI CD is green

# CLI and UI Testing for Antigravity Backups: high priority instruction, non-negotiable task

When you are testing the CLI and the UI of the backups, a few things. If we run the `Antigravity Manager or Antigravity Tools` and say, "Yeah, running prompts," running, it basically would list out the running prompts. First, it would list out the instances, how many instances are there, with their number and sequence so that I can use that to call again or inject a prompt again or do anything with it. So link should give under each one of the instances, how many that's running prompts. For each one of the prompts, it would actually give 100, 200 words, and that should be configurable from settings for both of the cases. So if we run from git-map, it would be git-map AGY running, it would list out the prompts with the instances. So nice view so that I can understand and see it, not the full prompt, but 100 words. We could also do JSON as a flag, giving everything on the output as a JSON. Also, we could do git-map AGY FF for fast-forward. That would just basically going to delegate that to `Antigravity Manager or Antigravity Tools` to do the fast forwarding. And also we can do `gitmap` AGY backup. Backup will give you running prompts, and we can, running prompts with image, plus images. Then when we say running, it should also give a number, like how many images are there with that prompt as well. If we do AGY restore. These commands should be there, you know, just to implement that. And when we are running the CLI commands, it should actually show every step, which prompts are getting backed up with images, where and how, and how it is restoring that. So basically, it can give a instance name as well, if we wanted to. If we don't give, then it would try to do it for both. Instance as well. All this needs to have help, proper segment so that anyone can understand. Need to have UI help along with it. Make sure that you test it, end-to-end test it so everything works. You put images to the item so that you can add the image and check the image restore is done. And also, everything needs to have tracing that this is the images, sub-images, and things that we are dealing with, so that you can understand that everything is working fine.
```

---

## Strategic Intent & Systems Reasoning

Antigravity Manager acts as the multi-instance coordinator and backup governor for Google Antigravity workspaces. Active conversational workflows and prompt execution queues can be distributed across multiple sandboxed instances (e.g. `default`, `inst-1`, `inst-2`). Developers and automated daemons require high-observability CLI inspection (`agm running [--json]` and `gitmap agy running [--json]`) displaying instance numbering, sequences, active running prompts (100–200 words bounded preview), and exact attached image counts.

Furthermore, backup and restore commands (`agm backup running [instance]`, `agm restore running [instance]`, `gitmap agy backup running-prompts`, `gitmap ag restore running-prompts [instance]`, and fast-forward `gitmap agy ff`) must provide:
1. Real-time step progress reporting (Step 1/4 to 4/4).
2. Deep multimodal tracing for primary images and sub-images.
3. Clean JSON envelopes on `--json`.
4. Comprehensive `--help` text and contextual UI help.
5. Strict FIFO restoration order (`ORDER BY created_at ASC, id ASC`).
6. Caller session preservation during automated E2E testing.
7. Minor version bump release ceremony with green CI/CD verification.

---

## Discrete Task Breakdown

1. **Task-01: Git Pull, Preflight, Ledger & Baseline Discovery**
   - **Owner:** Lead Orchestrator
   - **Owned Files:** `.ai-memory/temp-agents/cli-and-ui-testing-for-antigravity-backups/ledger.md`
   - **Status:** DONE

2. **Task-02: Spec Authoring & Subtask Decomposition**
   - **Owner:** Spec Authors 01, 02, 03
   - **Owned Files:** `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/*.md`, `.ai-memory/plans/subtasks/cli-and-ui-testing-for-antigravity-backups/*.md`
   - **Status:** IN_PROGRESS

3. **Task-03: `agm running` & `backup/restore running` CLI Enhancements**
   - **Owner:** Worker 01
   - **Owned Files:** `src-tauri/src/bin/agm/main.rs`, `src-tauri/src/bin/agm/prompt_cmds.rs`, `src-tauri/src/bin/agm/running_prompts_cmds.rs`, `src-tauri/src/bin/agm/running_backup_cmds.rs`
   - **Status:** PENDING

4. **Task-04: GitMap AGY Delegation Bridge & Aliases**
   - **Owner:** Worker 02
   - **Owned Files:** `src-tauri/src/bin/agm/misc_cmds.rs`
   - **Status:** PENDING

5. **Task-05: Configurable Word Limits & Settings UI**
   - **Owner:** Worker 03
   - **Owned Files:** `src-tauri/src/models/config.rs`, `src/types/config.ts`, `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src/components/settings/AdvancedMaintenanceSection.tsx`
   - **Status:** PENDING

6. **Task-06: Help Segments & UI Contextual Guidance**
   - **Owner:** Worker 01
   - **Owned Files:** `src-tauri/src/bin/agm/help_text.rs`, `src-tauri/src/modules/cli/prompts_help.rs`
   - **Status:** PENDING

7. **Task-07: End-to-End Verification with Image Restoration Test**
   - **Owner:** Worker 02
   - **Owned Files:** `scripts/test-agm-gitmap-running-backup-e2e.ps1`
   - **Status:** PENDING

8. **Task-08: Preflight, Minor Version Bump & Release Lifecycle**
   - **Owner:** Lead Orchestrator
   - **Owned Files:** `package.json`, `src-tauri/Cargo.toml`, `CHANGELOG.md`, `changelog_en.md`, `README.md`, `README_EN.md`
   - **Status:** PENDING
