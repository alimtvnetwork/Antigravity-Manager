# Master Execution Plan: Prompt Tree and UI Bug Fixes

- **Plan ID**: 242
- **Slug**: `prompt-tree-and-ui-bug-fixes`
- **Spec Path**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/01-architecture-spec.md`
- **Status**: Completed / Released
- **Version**: `v4.286.0`

---

## 1. Task Inventory & Subtasks Completed

- **Subtask 1**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/01-subtask-queue-removal-and-false-queued-badges.md`
  - [x] Diagnose and resolve queue item removal failures in Prompt Tree View. Bounded optimistic queued turns in `usePromptTreeData.ts` to 12s TTL to prune dequeued turns and prevent circular SWR cache re-injection.
  - [x] Eliminate false positive `1 QUEUED` badge aggregation on idle projects. Removed conv 0 fallback in `tree_node_conv.rs` and reconciled `c.is_queued` in `tree_nodes.rs`.
  - [x] Added native `clear_queued_prompts` IPC command (`cmd_queue_resend.rs`, `queue.rs`) with interactive `[ Clear Queue ]` action in `DetailHeader.tsx`.
- **Subtask 2**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/02-subtask-action-buttons-dropdowns-and-compact-ui.md`
  - [x] Redesigned action buttons in `DetailHeader.tsx` to compact `rounded-md` (6px radius) capsules (`h-7`).
  - [x] Combined `[ Send ]` (Hotkey: N), `[ Enqueue ]` (Hotkey: Q), `[ Rerun ×N ]` (Hotkey: R), and dropdown chevron into a contiguous segmented capsule with 1-click execution and full popover options.
  - [x] Added interactive Next Breakdown dropdown popover with category/task selectors (`code`, `ui`, `system`, `ideas`), task batch sizes (`10`, `20`, `50`, `100`), dry run toggle, automated execution actions, and CLI parity hint.
- **Subtask 3**: `02-spec/21-app/242-prompt-tree-and-ui-bug-fixes/03-subtask-fleet-machine-detection-and-duplicate-button.md`
  - [x] Corrected local host machine `w2` detection in `useFleetMachines.ts` and `query.rs` using GitMap variables and `seen_ips` deduplication to prevent duplicate `server` entry.
  - [x] Restricted purple `[Lead]` badge strictly to `w1`.
  - [x] Removed duplicate `+ New Instance` button on Instances page in `InstancesHeader.tsx`.
  - [x] Created `PromptTreeBreakdownView.tsx` and mounted it under `[ ✨ Breakdowns (100) ]` tab in `DetailTabs.tsx`.

---

## 2. Verification & Quality Gates Passed

- [x] `cargo fmt -- --check`: Passed with exit code 0.
- [x] `npm run build`: Passed with exit code 0 (`tsc` + Vite bundle cleanly generated).
- [x] Version bump: Atomically upgraded all 14 manifests to `v4.286.0`.
- [x] Release notes: Attributed strictly `@aukgit` (`(Thanks to @aukgit)`), strictly relative Git paths.
