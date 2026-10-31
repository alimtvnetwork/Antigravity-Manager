# Master Plan: SSH and Prompt Management Testing

Request slug: `ssh-and-prompt-management-testing`
Status: COMPLETED
Phase: 3 (Verification & Finalization)
Capacity: A = 3 agents × H = 2 hands = 6 concurrent operations

## User Request (Verbatim)
# SSH and Prompt Management Testing: high priority instruction, non-negotiable task

Can you please confirm that you can connect with SSH? And in SSH, you are showing the admin username. I don't understand why you are showing that. In the SSH feed section, in the instance section, what I'm interested in is how many instances are running, what are the emails that those instances have? This is what I wanted to know in the Instance tab. IP is important. Aliasing is also important, but I don't want to see their password, username. I'm interested in the email. You should work on it. Also the prompt tree, you have a list of prompt views. Try to make sure that it can be hidden. Try to have the hide option, and by default, it will be hidden, like collapse and expand. Test the image section for the prompts. Can you copy the image as a single text? And also image can be passed on to the IDE, or also I could save it to OneNote. The image in the prompt is becoming very challenging. Confirm when you switch back, are you able to run the running prompt as a restore? Do you have logs on this, that the exact prompt has been backed up with the image and everything and restored back? Test this part very effectively. Focus on the Telegram thing. Add a Telegram guide, and in the drop-down, besides the instance, you have the drop-down, right? Settings and other stuff. This is where you should also add the Telegram section and take out the Telegram section from the settings. Telegram, keep it as its standalone section, have some guide. Also share some CLI commands that a user can use to achieve and do this. Focus on this type of testing, especially on the prompts backup using image and restore back. The prompts tree view, we should have an option to, by default, hide the automated prompts. It can be filtered out. By default, it will be filtered out, but for your testing purpose, you can actually make it visible, and everything also can be done using the CLI. You should be able to see all these logs, whatever happening. Enhance the `Antigravity Manager or Antigravity Tools` log section for AI. Add detail help section as in skill and other parts so that any AI can learn what to do, how to do it. How to view the logs, how to view the errors. See the system-level errors, haven't happened, and try to fix all these together. The error logs should also reside in the database, a speed Database (Database (Db)) of the errors, just like `gitmap`. Follow through `gitmap`'s path. These errors can also be seen from the terminal for you to fix it, understand it, and fix it. Save, export, these are there, but also have CLI level thing which can be used to fix it itself. These are very priority work. Make sure that you complete all of these.

## Task Inventory & Status Matrix

| Task-ID | Title | Owner | Status | Target Files |
|---|---|---|---|---|
| Task-01 | Setup, Preflight, Specs & Plans | Lead | DONE | `02-spec/21-app/ssh-and-prompt-management-testing/` |
| Task-02 | SSH Fleet Card & Telemetry Redaction (Email & Instance Count over Username/Password) | Worker 01 | DONE | `src/components/instances/fleet/`, `src/pages/instances/` |
| Task-03 | Collapsible Prompt Tree & Default-Hidden Automated Prompts Filter | Worker 01 | DONE | `PromptTreeViewModal.tsx`, `TreePanel.tsx`, `usePromptTreeState.ts` |
| Task-04 | Standalone Telegram Hub in Header Dropdown & Interactive Setup Guide | Worker 01 | DONE | `NavHeader.tsx`, `Navbar.tsx`, `TelegramGuideModal.tsx`, `SettingsModal.tsx` |
| Task-05 | Multimodal Prompt Image Clipboard (IDE & OneNote Integration) | Worker 02 | DONE | `DetailHeader.tsx`, `clipboard.ts`, `PromptInstructionView.tsx` |
| Task-06 | Prompt Backup & Restore Verification with Multimodal Image Audit Logs | Worker 02 | DONE | `src-tauri/src/modules/repo_db/`, `backup.rs`, `dispatch.rs` |
| Task-07 | Speed Error Database (`errors.db`) & Error Store Persistence | Worker 03 | DONE | `src-tauri/src/modules/error_db/`, `src/stores/error-store.ts` |
| Task-08 | CLI Error Query, Inspection & Self-Healing Commands (`agm errors`) | Worker 03 | DONE | `src-tauri/src/bin/agm/`, `error_cmds.rs` |
| Task-09 | AI Diagnostics Skill & Telemetry Help Documentation | Worker 03 | DONE | `.agents/skills/agm-ai-diagnostics-and-logs/skill.md` |
| Task-10 | End-to-End Verification, Pre-flight Gates & Atomic GitMap Commit | Lead | DONE | Repository test gate and atomic GitMap commit |
