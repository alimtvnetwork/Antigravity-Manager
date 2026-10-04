# Application Specification: 108-prompts-restore-ui-progressbar-and-instance-sync

## User Request (Verbatim)
```text
Progressbar coloring

First color from right #12b27d
Next  #1af18d

https://prnt.sc/HwKMXHPGqKaS

follow the skills from @[.agents/skills/error-management] 

# High Priority Instruction

Okay. First of all, we have to start with the restore. Backup and restore of the prompts, enqueued prompts, and running prompts. These are very two important factors. I think you need to retest or check and do the end-to-end testing. Okay. That's the first thing. Second, coming to the point, we need to change the color of the, let's say... Yeah, these are the things we need to fix. Okay? Then we have to come to the UI. We have to work on the UI. First of all, when we have the progress bar, okay? I'm giving you a sample of the progress bar. It's not like that we have to have this type of bigger circle, but we can have a middle circle, something like this, with the feeling like it is draining water. Okay? So try to explain the spec first as how it needs to be in detail, like how a progress bar needs to be there. The progress bar does not need to have too many padding space or the, let's say, the circle. It doesn't have to be too much big. Try to keep it in line with the progress bar. Try to explain this as much as possible along with the image. Okay? So it is close to this, but not also too much space and padding like this one. Okay? So we have to reduce the space so that it fits in our UI. Okay? That's how you have to explain in details as much as possible. Okay. The progress bar is very important, and the team I mentioned, the team needs to be in between the two buttons that I'm showing you. It's between the refresh or purge and the language. It needs to be in between these two. Okay, so the teams then needs to be in between these two. Okay? That's the first thing. Second, it is in the instance setting option. Okay? We have two options to copy-paste the item. What do I mean by that? Two options, meaning that the copy teams and the copy workspace, okay, that could be in separate. That's fine. Okay. But, also there should be one button that will do both of the copy. Okay? Both copy and paste. Okay? Separate is fine, but also a both copy. That means I could copy all together both of these settings, okay, in one shot. And besides the copy, there should be a paste button. So there should be a split button concept, one is copy and other is paste. Let's say I copy the one project. So in the dropdown, it should show two things. One is the profile name, sequence and the profile name, and the IDE ending part. Okay? If the IDE have a antigravity hyphen something like this, it will have that suffix card displayed in the dropdown. Okay? Nicely, so you can use the custom dropdown, so that it looks nice. Okay. The project tree and copy from one place to another, that actually does not work because the copy remains in memory, and then if I go to the another project, I cannot paste it. It says it's invalid, something like this. So you need to look into this problem. Okay? Do you understand? Can you please walk us through? Regarding these options in the settings instance and things like that, make sure that you have the CLI commands. Also, the theme change needs to be from CLI commands. Another thing I want is the accounts menu, where all the drop-downs are. So in this menu, besides this, I want the accounts icon. And besides this, I want the instance icon. And then besides this, I want the settings icon. So these three would be very close that I click and I go there. And then rest of the item would do on a simple drop-down, hamburger drop-down menu, which shows the current drop-down, except for the ones that we already displayed in icons. Like accounts, we don't need instance, and the settings we already have. So we don't need to put this again. Does this make sense? Can you please reorganize these accounts button? So these are the tasks. So make sure that you understand the task. You write the spec first in very detail, especially for the progress bar. So I'm giving you the progress bar color link. Very important. There's a little bit more thing that's into the prompts layer. So first of all, the prompts layer Model Heading, that is a broken one. It actually goes on top of the header, and then the heading part does not show up. That is one issue. The next problem in that prompts layer section is that the display that it shows, the display is not in, let's say, markdown. So it needs to display like a markdown. We can have a raw view and a preview mode where we can also in the preview mode, we should be able to type. So we can add this feature, markdown preview, highlight, and things like that. It should be easy. Now, you have the full screen mode, which you can just say full. You don't need to have full screen inspector. It looks bad. And have a hover over tooltip for every one of them. And I need to have more options here. For example, I could say resend again. I could say enqueued again, enqueued resend. Send would be directly send, and enqueued would be if there is an item there, then it would be enqueued. If not, then send it again. So these are the things I want with all of these items, these prompts that we already sent. And another is the preview. The left-hand side, the preview where there's untitled conversation or the conversation has no text at all. No prompt preview. In those sections, I think we can skip it because we don't know what that is and why it is no prompt preview. So if it's a error, then we need to know the error logs and stack traces using our error management. But if it is only the, let's say, the prompts that is empty somehow, we don't have it, then you don't show it here. What you could do is you could say no prompt preview or untitled prompt together, and you give a number like 176 that would be at the end onto that project. So the way that it would work is the first level is the project. The second tree view would be the conversation, conversations names. If we have different conversations, that will show up. And after the conversation, we will have inside the prompts the next tree view. So three layers tree view we want. So make sure that you have it. If same thing is repeated like untitled or something like this, you combine together. And you first deal with the 500 words, not more than that, but also user have the ability to copy the whole thing and take out the whole thing. And when you display this, you create own caching for that instance using the split DB concept. Try to understand this. So this is a big concept you need to grasp. You need to write the spec, you need to plan, then and only then you can write it. So first, write the spec. And also enqueued the tasks so that if you break or if you cannot do it, the next AI can pick it up and do the remaining task. Do you understand? Is it clear?

# Actionable Items Must Follow Non-Negotiable

1. Write a plan and spec first for the backup and restore of prompts.
2. Retest or check and perform end-to-end testing.
3. Change the color of UI elements as specified.
4. Design and detail the progress bar specifications.
5. Ensure the team button is positioned correctly between the specified buttons.
6. Implement copy-paste functionality with a split button concept.
7. Address the project tree copy-paste issue.
8. Reorganize the accounts menu with specified icons.
9. Fix the prompts layer Model Heading issue and markdown display.
10. Implement full screen mode and tooltips.
11. Add resend and enqueued options for prompts.
12. Handle no prompt preview scenarios appropriately.
13. Implement a three-layer tree view for projects, conversations, and prompts.
14. Create caching for instances using the split DB concept.
15. enqueued tasks for continuity in case of failure.
```

---

## 1. Architectural Blueprint & Invariant Specifications

### 1.1 Prompt Backup, FIFO Enqueue & Running Prompts Restore Lifecycle (E2E Verified)
- **Scope**: `src-tauri/src/modules/repo_db.rs`, `instance.rs`, `task_history_db.rs`.
- **Invariants**:
  1. *Total Prompt Conservation*: Every in-flight or enqueued prompt must be tracked through `active_prompts` and preserved across process restarts.
  2. *Cold-Boot Timing Window*: Add adaptive polling for IDE prompt channel readiness (`wait_for_instance_prompt_channel`) rather than fixed arbitrary sleep, preventing cold-boot timeouts on heavy machines.
  3. *Multi-Prompt FIFO Preservation*: When multiple queued prompts exist for a single repository, `resend_running_commands_for_instance` must NOT overwrite `.antigravity_resume_task.json` or drop older tasks; it must dispatch the head of the FIFO queue while keeping remaining prompts in status `'queued'`.
  4. *Coordinated Scheduler Status*: Fix the 120s cooldown delay in `is_prompt_running_for_project` by checking real process activity and task completion instead of a rigid timer.

### 1.2 Water-Drain Checkpoint Progress Bar Specification
![Screenshot](assets/screenshots/108-progressbar-stepper.png)

#### Color Tokens
- **Right / Peak Gradient**: `#12b27d`
- **Next / Lead Fluid Color**: `#1af18d`
- **Track Background**: `bg-slate-200/80 dark:bg-[#071a27] border border-slate-300/40 dark:border-[#15334d]`
- **Active Node Fill**: `#1af18d` with border `#12b27d` and glow `shadow-[0_0_6px_rgba(26,241,141,0.5)]`
- **Inactive Node Fill**: `rgba(18, 178, 125, 0.2)` with subtle border `#12b27d`

#### Layout & Sizing Standards
- Compact height (`h-2` / 8px) with embedded checkpoint nodes (14px diameter) centered vertically.
- Avoids excessive padding: container `h-5` flex row, perfectly aligned inline with text badges.
- Fluid shimmer animation (`animate-[shimmer_2s_infinite]`) providing a subtle water-drain effect without CPU churn.
- Integrates in `QuotaItem.tsx`, `InstanceTable.tsx`, and `Instances.tsx`.

### 1.3 Top Header Capsule & Theme Button Relocation
![Screenshot](assets/screenshots/108-navbar-theme-capsule.png)

- **Target Container**: Segmented pill capsule in `src/components/navbar/NavSettings.tsx`.
- **Exact Layout Order**:
  1. **Button 1 (Left)**: Quick Clean (`RotateCcw`, `w-7 h-7 rounded-l-full`).
  2. **Button 2 (Center)**: **Theme Switcher** (`Palette` icon + active gradient swatch dot + popup menu, `h-7 px-2 border-r border-gray-200/50 dark:border-slate-700/60`).
  3. **Button 3 (Right)**: Language & Appearance (`Sun/Moon` + uppercase `EN` + `ChevronDown`, `h-7 px-2 rounded-r-full`).
- **Cleanup**: Remove redundant standalone theme switcher from the center of `Navbar.tsx`.

### 1.4 Center Route Navigation Restructuring
![Screenshot](assets/screenshots/108-navbar-accounts-menu.png)

- **Direct Route Icons** (mounted directly side-by-side in center nav bar):
  1. `[Users / Accounts]` (`/accounts`)
  2. `[Laptop / Instances]` (`/instances`)
  3. `[Settings / Settings]` (`/settings`)
- **Overflow Hamburger Menu**:
  - `Menu` icon (`h-8 px-2.5 rounded-full`) opening a clean dropdown containing only the remaining non-direct routes:
    - Dashboard (`/dashboard`)
    - Proxy & Relays (`/api-proxy`, `/apikey-fun`)
    - Monitor & Metrics (`/monitor`, `/token-stats`, `/user-token`)
    - Security & Auth (`/security`)
    - Email Management (`/email`)
    - Supabase Cloud (`/supabase`)
    - Audit Trail (`/audit`)
  - No duplicate entries for Accounts, Instances, or Settings.

### 1.5 Instance Deep Sync: Combined Copy/Paste, Suffix ID Dropdown & CLI Commands
![Screenshot](assets/screenshots/108-instance-settings-modal.png)

- **Combined Copy & Paste**:
  - Add **"Copy Both (Settings & Workspaces)"** action button in `InstanceSettingsModal.tsx`.
  - Add persistent clipboard buffer in `localStorage` (`agm_instance_clipboard_buffer`) so copying a source instance retains both settings JSON and workspace folder state across target profile switches and dialog reopenings.
  - Add split **"Paste"** button: Primary action pastes both settings & workspaces; dropdown allows "Paste Settings Only" or "Paste Workspaces Only".
- **Target Profile Dropdown Formatting**:
  - Render: `#{seq} {name} (antigravity-{id}) [Default]`.
  - If instance ID contains a suffix (e.g. `antigravity-8136`), cleanly display the suffix badge.
- **CLI Parity**:
  - `agm theme set <theme-id>`: sets theme in instance configuration.
  - `agm instance sync-settings <src> <target>`: copies settings between instances from command line.

### 1.6 Prompt Tree View Modal Overhaul
![Screenshot](assets/screenshots/108-prompt-tree-modal.png)

- **Bug Fix**: Elevate modal `z-index` to `z-[200]` and add top margin/padding to prevent sticky navbar (`z-100`) from overlapping and clipping modal title "Project & Prompt Tree View".
- **Three-Layer Tree View**:
  - Layer 1: **Project / Workspace** (Folder icon, project name, total prompt count).
  - Layer 2: **Conversations** (MessageSquare icon, conversation title or short ID, status badge).
  - Layer 3: **Prompts** (FileText icon, prompt #1, #2, word count).
- **Untitled & Empty Prompts Consolidation**:
  - Group all empty or untitled conversations under a single collapsible group node: `📁 Untitled Conversations (176)`.
- **Markdown Display & Edit Mode**:
  - Segmented toggle: `[Preview] | [Raw] | [Edit]`.
  - Preview renders rich Markdown, code blocks with syntax highlighting, and 500-word truncation with "Show Full" toggle.
  - Edit mode allows editing the prompt prior to dispatch.
- **Actions**:
  - "Full" button (renamed from "Full-Screen Inspector") with hover tooltip.
  - "Resend" button (immediately injects prompt into live instance).
  - "Enqueue" button (places prompt into FIFO scheduler queue).
- **Split-DB Caching**:
  - Create table `prompt_tree_cache` in `repo_prompts.db` to cache parsed conversation trees, reducing tree load time from seconds to <5ms.
