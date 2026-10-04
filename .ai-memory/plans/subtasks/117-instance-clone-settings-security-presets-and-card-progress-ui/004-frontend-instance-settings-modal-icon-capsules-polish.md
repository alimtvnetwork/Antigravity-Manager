---
plan: 117-instance-clone-settings-security-presets-and-card-progress-ui
subtask: "004"
title: Frontend instance settings modal redesign with icon capsules and layout fix
domain: frontend/ui
target_files:
  - src/components/instances/InstanceSettingsModal.tsx
status: pending
---

# 004 — Frontend Instance Settings Modal Redesign with Icon Capsules and Layout Fix

## Scope
1. In `src/components/instances/InstanceSettingsModal.tsx`:
   - Redesign quick toggles (Turbo Mode, Plan Review, Baseline):
     - Replace large text buttons with compact segmented switch capsules and semantic Lucide icons (`Zap`, `CheckCircle2`, `Sliders`, `Sparkles`).
     - Use concise labels ("Auto-Confirm", "Plan Review") with tooltips (`title="..."`).
   - Fix replication bar dropdown positioning:
     - Prevent the "Split Paste" menu from covering the "Workspaces & Folders" card below.
     - In the 2-column cards, stack or space `<select>` and copy buttons cleanly so instance names are not squashed.
   - Group JSON tools into a contiguous segmented pill capsule (`[Copy | Paste | Export | Import | Raw Editor]`).
