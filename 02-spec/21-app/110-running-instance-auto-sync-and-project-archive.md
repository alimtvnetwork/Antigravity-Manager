# Specification: Task 110 - Running Instance Auto-Sync, Auto-Selection, Project Archive & Stale Prompts Demotion

## 1. Problem Classification & Scope
- **Domain**: Prompt Tree View Live Telemetry, Configurable Interval Sync, Instance-Scoped Project Archive & Prompt Organization.
- **Problem Statement**:
  1. **Modal Auto-Selection Disconnect**: When opening `PromptTreeViewModal`, it did not automatically pick the actively running project and prompt for that instance, and bulk-expanded all projects, pushing selected items off-screen.
  2. **Configurable Background Auto-Sync Interval**: Users need live auto-sync intervals (15s, 30s, 1m, 2m, Off) with a minimum 15-second floor to avoid disk thrashing, automatically updating the UI in the background as new conversations arrive.
  3. **Demotion of Stale / Empty Prompts**: Untitled or empty prompts with zero steps or empty preview text cluttered the conversation list instead of sitting at the bottom as low priority.
  4. **Per-Instance Project Archive / Less Favorite (Thumbs Down)**: Instances are typically dedicated to specific projects. Users need to hide or mark projects as "Archived / Less Favorite" for a particular instance without altering repository storage or global configurations.

---

## 2. User Request (Verbatim)
```text
If we are in a running project, then it would automatically show up whichever prompt is running. If we open the running instance, it should automatically sync with the latest changes. So if we open the prompt layer on that instance, then every, let's say one minute or let's say 30 seconds, that could be changed from the settings. 30 seconds, 15 seconds, no less than 15 seconds, by the way. 15, 30 seconds, one minute, two minutes. So it would automatically update the UI based on the latest conversation that has been picked. If there is no change, then no change, but it should automatically synchronize with the UI, try to have this flavor so that it can understand where the changes are. And also prompts which are very old, for example, have no conversation. This should be always on bottom. Low priority altogether, somewhere like archive, something like this. And archives won't also show up as archive. Also projects, we should have a, let's say, less favorite or let's say thumbs down or probably archive option here. The archive option is not going to archive the project in the system, but here we don't want to see in that instance. It would save it, mark it, and make the UI better. Do you understand? Because in most cases, instances are situated for a project, too. And I'm going to use that one for that specific project, so make sure that we follow through that. Is it clear and understood?
```

---

## 3. Architecture & Technical Design

### A. Auto-Selection & Targeted Expansion on Modal Open
- **Detection Algorithm (`findAutoSelectTarget`)**:
  - Scan all non-archived projects in `treeData`:
    1. **Priority 1**: Actively running conversation (`c.is_running === true || c.status === 'RUNNING'`). Pick the newest by `last_modified`.
    2. **Priority 2**: Actively running project (`p.is_running === true`). Pick its newest conversation.
    3. **Priority 3**: First non-archived project in prioritized order. Pick its newest conversation and latest prompt turn (`conv.step_count`).
- **Targeted Expansion**:
  - Auto-expand ONLY the target project in `expandedProjects: { [targetProject.project_id]: true }`.
  - Auto-expand target conversation in `expandedConversations: { [targetConv.conversation_id]: true }`.
  - If target conversation is in the stale/untitled group, auto-expand that stale group so the item is visible.

### B. Configurable Auto-Sync Interval Capsule (15s–2m, Off)
- **Options**: `'15s'`, `'30s'` (default), `'1m'`, `'2m'`, `'off'`.
- **Validation**: Strict minimum floor of 15 seconds (`SYNC_INTERVAL_SECONDS`).
- **Placement**: Integrated directly into modal header segmented toolbar capsule next to `[Refresh]`:
  `[Backup] | [Restore] | [Refresh] | [Sync Interval Dropdown] | [Full]`.
- **Persistence**: Saved in `localStorage` under `agm_prompt_tree_sync_interval`.
- **Execution Hook**:
  - Runs via `useEffect` with `setInterval`.
  - Calls `loadTree(false, true)` (`isInitial = false, force = true`).
  - Passes `force: true` to bypass SQLite 60s disk cache (`prompt_tree_cache`).
  - Does NOT show full-page loading spinner during interval sync.
  - Updates conversation list and active prompt in place without resetting user scroll or dirty textarea edit state.

### C. Stale / Empty Prompts Demotion to Bottom
- **Classifier (`isStaleOrEmptyConversation`)**:
  - Active running conversations (`c.is_running === true`) are immune and never marked stale.
  - Non-running conversations with empty `prompt_preview_200w`, zero steps, or generic "Untitled Conversation" are categorized as stale.
- **Rendering**:
  - Active, meaningful conversations render prominently at the top.
  - Stale conversations are clustered at the bottom of the project's conversation tree under `📁 Archived / Stale Prompts ({count})`.

### D. Per-Instance Project Archive / Less Favorite (Thumbs Down)
- **Persistence**:
  - Key: `agm_archived_projects_${instanceId || 'default'}` in `localStorage`.
- **Action Buttons in Layer 1 Project Row**:
  - `[Refresh]` (`RotateCw`), `[Pin]` (`Pin`), and `[Archive / Less Favorite]` (`Archive` / `ArchiveRestore`).
  - Archiving a project automatically unpins it to maintain consistent priority.
- **Partitioning**:
  - Projects are partitioned into `activeProjects` and `archivedProjects`.
  - Under normal view, active projects render on top, while archived projects sit collapsed at the bottom under `📁 Archived Projects ({count})` with an unarchive button.
- **Filter Pills**:
  - Added `[Archived ({count})]` pill to view and search all archived projects directly.
