# Execution Plan: UI/UX Redesign and Bug Fix Instructions

## User Request (Verbatim)
# UI/UX Redesign and Bug Fix Instructions: high priority instruction, non-negotiable task

This highlighted section should be collapsed by default. And also, I do not understand what is the C something that you have here in the screenshot. Let's say you have C211. What does this mean? And also what does it mean by returns? So these are the two things I do not understand. And also, this section should be hidden by default. If I click on expand button, that should be a little bit of arrow there. The purple color would be there, a little bit of arrow there. If I click on the arrow, then it would expand. The arrow would be barely visible. The UI/UX. And whatever way that I put it, it should remember. If I kept it hidden, it would be hidden for the next one. If I kept it expanded, it would remain expanded for the next one. That's one thing. Another is, if I click on the input and output button, the expand/fold button goes away. That is a bug in the UI that you need to fix. And also in the top level input. Top level, you have the session, pool session, 30, 23 turns. I don't understand this. This should be compacted inside the detail button. And also the repetition of the header, like UI redesign and bug fix, UI redesign and bug fix with the ``Id`` repeated twice. Don't need. The ``Id``, again, should be inside the details section. If we click on it, then we see it. If we don't click, don't see. And what is this hour? Hour and time range. You can actually reduce the second, because second is meaningless if the time is very big. And anywhere you have date time, you should display that date time into the DD hyphen triple M hyphen four double Y and a space 24-hour clock based on the local time zone that is set in the Windows. And that should be configurable from the settings that how they want it to display their time format everywhere. But every time you save the time into the system, I hope that you save it in the UTC0 and then convert during the display time. Make sure the date time is displayed very nicely in a 24-hour clock, which is meaningful. And if a time is given and if it's, let's say, run five minutes ago, then try to put also below, ran five minutes ago, so that it's easier for us to understand. It ran one day ago, along with the time. This type of information is very helpful for the mental picture. And again, if I go into the prompt tree, the images are not visible anywhere. One more issue, if we don't have the IDE open, and if we click on the send button, it opens the IDE, but then again, do not send because it gets lost. So when the IDE is not open, then it would already know that IDE is not open, then it will wait for five seconds to open the IDE, and it will confirm by visiting the IDE based on PID screenshot and then send the conversation. Please make sure that you work on this, because send does not work if the IDE is not open. Probably you send it, but it does not get added. That is one big issue. So these are the things I think you need to fix right now. I'm still very concerned. I'm still very concerned that how you are displaying the tree view of the prompts, I do think it's not in the right manner. And I also don't see the queued prompts. Queued prompts means prompts which are actually queued and which are running. I don't see those two anyway. I think you have some issue how you are displaying it. Image became a big issue. I don't see the image that is working. Images I don't see. And the current running prompts also you don't have the in queue option. For example, in this case. I'll give you another screenshot. So here in the antigravity, prompts are running. Prompts are running and it shows that prompts are running. If I click on it and you see empty in the left-hand side. Why and how? I don't have any idea. Try to find that root cause. Try to fix it. Do a release bump, minor bump, and do a release. Do you understand? And also SSH connection still does not work properly. Find the root cause, fix it. In SSH, our main goal is to understand which account is used by which instance, by which machine. This is the main important information. Not their IP, not anything else. IP we can see if we click on the details. That's the only important factor. And also we should be able to send prompts to any instances. So, if a working machine has multiple instance running, we should be able to see those instances, and we should be able to see what are the emails connected to this. These are the important information. And on that instance, we should be able to send prompt from one machine to another. That is the main goal. You cannot do it. I don't understand. It's been several times we have been talking about this. Several times. Find the root cause, fix it

---

## 1. Problem Classification & Architectural Root Causes

### 1.1 Session Overview Card & Header Redundancy (UI/UX)
- The session overview card in `PromptInstructionView.tsx` occupies substantial vertical space by default.
- Header duplication: Title, sequential conversation code (`C211`), and conversation ID are displayed redundantly across the card header, `DetailHeader.tsx`, and `DetailsModal.tsx`.
- User confusion regarding technical tokens: `C211` represents Conversation sequence index #211; `turns` represents the number of prompt/response interactions in the thread.
- Disappearing Expand button: Switching between `Input` and `Output` tabs changes `previewTab` to `'results'`, unmounting the expand toggle in `DetailTabs.tsx` because of a restrictive guard.

### 1.2 Unified Timestamp Formatting & Relative Durations
- Dates and timestamps are inconsistently formatted throughout the app.
- System requires storing strictly in ISO 8601 UTC0, but rendering in `DD-MMM-YYYY HH:mm` (24-hour clock) using Windows/local timezone, with a user setting option.
- When an execution time is shown, display relative time (e.g., `ran 5 minutes ago`, `ran 1 day ago`).
- Suppress seconds in durations when hours/minutes are present (e.g., `18h 27m` instead of `18h 27m 14s`).

### 1.3 Prompt Tree Image Rendering & Queued/Running Prompt Visibility
- Images fail to render in prompt tree because Tauri v2 asset capability (`core:asset:default`, `core:asset:allow-*`) is missing in `capabilities/default.json`, and relative paths are not resolved against project `repo_path`. `PromptMarkdownPreview.tsx` bypasses `resolveImageSrc()`.
- Left panel shows empty tree when prompts are running/queued because `discover_fallback_projects` in `tree_phases.rs` misses `active_prompts`, and `isStaleOrEmptyConversation` demotes queued prompts into collapsed stale groups.

### 1.4 Closed IDE Send Delivery Ladder
- Sending a prompt when the target IDE is closed causes prompt loss because `ensure_instance_running_smart` only waits 800ms before attempting CDP/socket dispatch. Electron IDE startup takes 3-5 seconds.
- Implementation requires an adaptive 5-second process stabilization ladder that verifies PID liveliness, DevTools port generation, and HTTP readiness before dispatching the prompt.

### 1.5 SSH Multi-Node Fleet: Account-Centric Discovery & Remote Prompt Dispatch
- `get_cluster_instances_and_balances` only performs TCP port 22 pings and returns dummy data without querying real multi-instance data or bound Google accounts.
- Standalone IP address columns dominate the UI instead of the critical pairing: machine + instance + connected Google account email.
- Remote prompt dispatch has serialization and response envelope mismatches, and lacks an instance dropdown selector.

---

## 2. Decomposition into Subtasks

- **Subtask 01 (`01-ui-ux-overview-collapsible.md`):** Collapsible session overview card with purple chevron, localStorage memory, header deduplication, C211/turns compaction into detail modal, and persistent Expand Full button across tabs.
- **Subtask 02 (`02-date-and-ide-send-ladder.md`):** Global `DD-MMM-YYYY HH:mm` formatter, relative time "ran X ago", duration second suppression, UTC0 storage invariant, and closed IDE 5-second readiness ladder.
- **Subtask 03 (`03-ssh-fleet-and-prompt-dispatch.md`):** SSH instance & Google account discovery, IP demotion into popovers, and instance-targeted remote prompt dispatch.

---

## 3. Disjoint File Ownership Map

| File Path | Owner Subtask |
|---|---|
| `src/components/instances/PromptTreeView/PromptInstructionView.tsx` | Subtask 01 |
| `src/components/instances/PromptTreeView/DetailTabs.tsx` | Subtask 01 |
| `src/components/instances/PromptTreeView/DetailsModal.tsx` | Subtask 01 |
| `src/components/instances/PromptTreeView/ResultsView.tsx` | Subtask 01 |
| `src/utils/date.ts` | Subtask 02 |
| `src/components/settings/GeneralSettingsTab.tsx` | Subtask 02 |
| `src-tauri/capabilities/default.json` | Subtask 02 |
| `src/components/instances/PromptTreeView/clipboard.ts` | Subtask 02 |
| `src/components/instances/PromptTreeView/PromptMarkdownPreview.tsx` | Subtask 02 |
| `src/components/instances/PromptTreeView/conversationGuards.ts` | Subtask 02 |
| `src-tauri/src/modules/repo_db/tree_phases.rs` | Subtask 02 |
| `src-tauri/src/modules/instance/smart_run.rs` | Subtask 02 |
| `src-tauri/src/modules/repo_db/agy.rs` | Subtask 02 |
| `src/components/settings/EmailNotifications/SshGitmapSection.tsx` | Subtask 03 |
| `src/components/instances/FleetMachinesTable.tsx` | Subtask 03 |
| `src/services/sshService.ts` | Subtask 03 |
| `src-tauri/src/modules/ssh_manager/types.rs` | Subtask 03 |
| `src-tauri/src/modules/ssh_manager/exec.rs` | Subtask 03 |
