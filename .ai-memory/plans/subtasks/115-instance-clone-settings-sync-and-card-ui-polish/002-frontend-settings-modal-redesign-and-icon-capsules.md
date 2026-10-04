# Subtask 002: Frontend Settings Modal Redesign and Icon Capsules

## Objective
Overhaul `src/components/instances/InstanceSettingsModal.tsx` to resolve visual glitches:
1. Fix overlapping "Copy Now" and "Copy Folders" buttons with source dropdowns.
2. Eliminate the horizontal scrollbar caused by `overflow-y-auto` + unbounded width.
3. Replace verbose text with sleek, compact Lucide icon capsules and concise labels.
4. Ensure clean responsive layout across all profile name lengths.

## Target Files
- `src/components/instances/InstanceSettingsModal.tsx`

## Root Causes Addressed
1. Container had `overflow-y-auto` without `overflow-x-hidden`, and inner replication bar exceeded modal client width, causing horizontal scrollbar.
2. Select elements lacked `min-w-0` and `truncate`. With long profile names, dropdown intrinsic min-width expanded beyond column bounds, pushing adjacent buttons and causing overlap.
3. Buttons lacked `shrink-0` and `whitespace-nowrap`.
4. Overly wordy button labels like "Copy Both (Settings & Workspaces)", "Copy Folders & Projects into Target" caused visual clutter.

## Design Improvements
- Add `overflow-x-hidden` to modal scroll container.
- Use `min-w-0 flex-1 truncate` on select inputs.
- Convert replication buttons into compact segmented icon pill capsules (`rounded-lg bg-slate-100 dark:bg-[#0c2438] border border-gray-200 dark:border-[#15334d]` with `Copy`, `FolderSync`, `Sliders`, `Sparkles`).
- Use clear tooltips and concise text badges instead of long paragraphs.

## Acceptance Criteria
- [x] No horizontal scrollbars appear under any modal tab or resolution.
- [x] Dropdowns and buttons never overlap, even with 50+ character profile names.
- [x] Streamlined icon capsules provide one-click actions for settings, workspaces, and both.
