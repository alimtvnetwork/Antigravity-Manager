# Architecture Specification: Prompt Tree and UI Bug Fixes

- **Spec ID**: 242
- **Slug**: `prompt-tree-and-ui-bug-fixes`
- **Category**: Frontend UI / Multi-Instance RepoDB / Fleet Synchronization
- **Version**: `v4.285.0` -> `v4.286.0`
- **Attribution**: Strictly `@aukgit` (`(Thanks to @aukgit)`)

---

## 1. Executive Summary & Problem Scope

This specification resolves critical UI, prompt tree queue, fleet machine detection, and action control defects identified in Antigravity Manager:

1. **Queue Removal Synchronization**: When prompt items are dequeued or removed from the execution queue, the Prompt Tree View fails to remove them and continues to display stale queued turns.
2. **False Queued Badge Aggregation**: Idle projects on the left tree panel display a `1 QUEUED` badge even when they contain zero active queued prompts.
3. **Action Button Consolidation & Compact Styling**: `Copy`, `Rerun`, `Send`, and `Next Breakdown` in `DetailHeader.tsx` require unified compact styling, reduced border radiuses, and individual interactive dropdowns.
4. **Interactive Next Breakdown Dropdown & Auto-Dispatch**: The `Next Breakdown` button lacks an interactive dropdown and configuration mechanism to trigger automated improvement breakdowns directly from the Prompt Tree View.
5. **Cluster Fleet Machine Detection (`w2` vs. `server`)**: The Shared Cluster / Fleet table fails to properly detect the current local host machine (`w2`) via `gitmap` machine resolution (`variables.json`).
6. **Duplicate "New Instance" Button**: An unnecessary secondary `+ New Instance` button appears in the instances layout.
7. **Integrated Next Improvement Experience**: The 100 Improvements Flywheel must be cleanly integrated inside the Prompt Tree View with visible category breakdowns (code, ui, system, ideas).

---

## 2. Technical Architecture & File Boundaries

### Subsystem A: RepoDB & Queue State Synchronization
- `src-tauri/src/modules/repo_db/tree_nodes.rs`: Fix project-level queued count and active prompt aggregation.
- `src-tauri/src/modules/repo_db/tree_phases.rs`: Prune stale queued/dispatched in-flight items when dequeued.
- `src-tauri/src/modules/repo_db/tree_node_conv.rs`: Filter out cancelled, deleted, or completed queue turns.
- `src-tauri/src/modules/repo_db/queue.rs`: Ensure dequeue mutations broadcast cache invalidation events.
- `src/components/instances/PromptTreeView/usePromptTreeData.ts`: Synchronize local tree state upon queue mutations and verify accurate badge counts.

### Subsystem B: Header Action Controls & Next Breakdown Dropdown
- `src/components/instances/PromptTreeView/DetailHeader.tsx`: Combine Copy, Rerun, Send, and Next Breakdown into compact segmented capsules with reduced border radius and individual dropdown menus.
- `src/components/instances/PromptTreeView/ImprovementsBreakdownModal.tsx`: Provide direct modal configuration and trigger for automated improvement breakdowns with category selector (`code`, `ui`, `system`, `ideas`).

### Subsystem C: Fleet Discovery & Local Node Resolution
- `src-tauri/src/modules/supabase_sync/node.rs`: Read machine alias from `~/.gitmap/variables.json` and resolve `w2` as the local node.
- `src/components/instances/fleet/useFleetMachines.ts`: Match local node IP and alias without hardcoding.

### Subsystem D: UI Polish & Duplicate Button Removal
- `src/pages/Instances.tsx` & `src/pages/instances/InstancesHeader.tsx`: Remove the redundant duplicate `+ New Instance` button.
