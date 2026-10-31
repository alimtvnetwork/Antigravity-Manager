# Plan: Fleet System and UI/UX Bug Fixes

**Slug**: `fleet-system-and-ui-ux-bug-fixes`  
**Status**: IN_PROGRESS  
**Lead**: Antigravity Orchestrator  
**Created**: 2026-10-20  

---

## User Request (Verbatim)
```text
https://prnt.sc/FnF7IMTncnd5
Markdown view
https://prnt.sc/oxgHWyUQkiWW
color
https://prnt.sc/FdL_kq6hZEDW
Fix the UI Issues
https://prnt.sc/uXKknIPwaS3Z
https://prnt.sc/6LEiq8Qfq_bV

# Fleet System and UI/UX Bug Fixes: high priority instruction, non-negotiable task

The fleet is still wrong. That means here you have selected this as IP 20. But if I put the same thing into perspective with Gitmap, which is very different, how could you be that stupid? Can you please tell me? If I run Gitmap on the same machine, the IP is different, which points to three. Does not match in here, but it should be a W1 machine, not the main machine. The IP what you are showing, this is not really correct. If I do the Gitmap nodes, you can see that your numbers and the Gitmap numbers I have been giving you a screenshot does not match at all. Why? Why you have this wrong information and why you don't have the W1 here? The fleet system, I have been asking for several times. You have not fixed it. That's not respectful. Then again, if we go to the prompt tree view, again, running prompt still cannot detect now. It's a new bug, I guess. So apparently, there are running prompts in the first instance, default instance. It still says nothing is running. If I go into the prompt tree view, and if I double-click and rename the sidebar from the conversation name to Alim, also in the top name does not change in the UI. This does not change, and also it does not affect it into the actual checklist, which I am giving you the conversation history. You can check that it did not get updated. So that is a serious bug where you have claimed that you have done it. You say you have tested, and then when I go there, I see it's not working. How can that be? Also, the copy button that we have, copy all. Do you not need to put include our images on this UI? You put it when we do the drop-down thing. Why you have to do that? I don't understand. And why the details button, why it actually went up? The detail button was there with a healthy icon. Why it went there? I don't understand. I mean, who is paying you to be that stupid? Who? Please tell me. You are wasting my time. I think in the main instance section, there should be a button to open all or start all instances. That is still missing. I think you need to work on it and make sure that you can do the all image copy paste. That means from the prompts, which is still not tested yet. I will give you the default color that you can use for the background of the markdown view. Also, you have not tested the fleet, SSH fleet with proper naming with the IPs. You have not done it. Why? It's fully messed up. Why don't you clean up? Why don't there is a reset button for the SSH cluster? The whole thing is a trash. Why? So long you have been working. Where you are breaking it? How you cannot figure out? It's very simple, right? It's very simple. In the SSH section again, you did something correct. Did something correct in the SSH section. I do appreciate that in the SSH settings section. But again, you have too much text. Where the stupidity you got too much text. These all should be icons. If I hover over, I would see the text. Fix this UI issues. Don't waste my time. Test it. Do a create a new instance for checking, open it, run the test cases, and also check your UI where the issues are, make it more incredible. Also, I have given you a screenshot for a color. So based on the color, you try to have the markdown default views. Sorry, markdown default color, markdown view inside the prompt view. Does this make sense?
```

---

## 5 Mandatory Pillars & Architectural Intent

### Pillar 1: Strategic Intent & Systems Reasoning
Antigravity-Manager serves as the unified orchestration and proxy gateway for multi-instance Antigravity environments. We resolve 8 critical defects:
1. Fleet topology desync: Local host (192.168.1.3) was missing `w1`, misattributed as `main` (192.168.1.20) or filtered out, leaving 5 nodes instead of 6 matching `gitmap nodes`.
2. Missing SSH Cluster Reset: No mechanism to flush stale caches and re-synchronize node rosters with GitMap.
3. Prompt Tree Liveness: Running prompts in `active_prompts` with status `dispatched` or empty default `instance_id` showed as `IDLE` / "No prompt running".
4. Conversation Rename Desync: Double-clicking in sidebar did not update header title, `selectedConversation` state, or persist into Antigravity IDE SQLite full-text search (`conversations_fts`).
5. Copy All Images UX: The button label and multi-MIME rich image copy was confusingly tucked behind dropdowns.
6. Details Button Misplacement: Moved up to row 1; must be restored to row 2 inline with view mode toggles.
7. Start All / Open All Instances: Missing global button on instance header to launch all stopped instances.
8. SSH Settings Action Text: Verbose text ("Test Connection", "Prompt Machine", "Delete") cluttered tables; converted to compact icon capsules with tooltips.
9. Markdown Viewer Background Color: Calibrated to `#19191c` (rgb(25, 25, 28)) per uploaded color reference.

### Pillar 2: Explicit Scope Boundaries
- **In-Scope**:
  - `src-tauri/src/modules/supabase_sync/` (node.rs, migration.rs, query.rs)
  - `src-tauri/src/modules/ssh_manager/` (sync.rs, connections.rs)
  - `src-tauri/src/commands/ssh.rs`, `commands/supabase.rs`, `commands/instance.rs`, `commands/cmd_running_up_next.rs`
  - `src-tauri/src/modules/repo_db/` (tree_collect.rs, tree_phases.rs, transcript.rs, sequences.rs)
  - `src/components/instances/` (FleetMachinesTable.tsx, fleet/*, PromptTreeView/*, InstancesHeader.tsx)
  - `src/components/settings/EmailNotifications/SshGitmapSection.tsx`
  - `src/services/` (sshService.ts, instanceService/lifecycle.ts)
- **Excluded / Deferred**:
  - Version bumping or release tagging.
  - Modifying unrelated proxy protocols.

### Pillar 3: Non-Negotiable Constraints & Negative Guardrails
- Total ban on `rg`, `ripgrep`, `grep`, `git grep`, `Select-String`, and shell `dir`/`ls`.
- Workers never run git commands (Worker Git Ban).
- No builds or full test suites locally.
- Strictly relative Git paths only.

### Pillar 4: Blast Radius & Dependency Topology
- Backend changes are backward-compatible.
- UI state updates use immutable Zustand and React state setters.
- FTS5 virtual table updates are safely guarded.

### Pillar 5: Multi-Agent Concurrency Topology
- Phase 1: Planning and Discovery (`research` subagents) [Completed].
- Phase 1B: Spec and Subtask Authoring (`self` subagents) [3 agents in parallel].
- Phase 2: Implementation Swarm (`self` subagents) [3 agents in disjoint file boundaries].
- Phase 3: Targeted Verification & Local Gates.

---

## Subtask Decomposition & Disjoint File Ownership

| Subtask ID | Title | Owner | Exclusive Owned Files |
|---|---|---|---|
| `Task-01` | Fleet & SSH Cluster Matching & Reset | Worker 01 | `src-tauri/src/modules/supabase_sync/*`, `src-tauri/src/modules/ssh_manager/*`, `src-tauri/src/commands/ssh.rs`, `src-tauri/src/commands/supabase.rs`, `src/components/instances/FleetMachinesTable.tsx`, `src/components/instances/fleet/*`, `src/services/sshService.ts` |
| `Task-02` | Prompt Tree Liveness & Conversation Rename Sync | Worker 02 | `src-tauri/src/commands/cmd_running_up_next.rs`, `src-tauri/src/modules/repo_db/tree_collect.rs`, `src-tauri/src/modules/repo_db/tree_phases.rs`, `src-tauri/src/modules/repo_db/transcript.rs`, `src-tauri/src/modules/repo_db/sequences.rs`, `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`, `src/components/instances/PromptTreeView/ConversationNode.tsx` |
| `Task-03` | UI Controls, Details Button, Start All, SSH Icons & Markdown Color | Worker 03 | `src-tauri/src/commands/instance.rs`, `src-tauri/src/lib/run.rs`, `src/services/instanceService/lifecycle.ts`, `src/pages/instances/InstancesHeader.tsx`, `src/pages/instances/instancePageTypes.ts`, `src/pages/instances/instancePageActions.ts`, `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/DetailTabs.tsx`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src/components/instances/PromptTreeView/InspectorModal.tsx`, `src/components/instances/PromptTreeView/ResultsView.tsx`, `src/components/settings/EmailNotifications/SshGitmapSection.tsx` |
