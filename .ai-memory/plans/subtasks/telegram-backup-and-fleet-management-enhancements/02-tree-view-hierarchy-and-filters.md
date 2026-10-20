# Subtask 02: Tree View Spacing, Terminal Indentation, Image Filter & Modernized Dropdown

**Subtask ID**: `sub-02-tree-ux`  
**Parent Plan**: `.ai-memory/plans/telegram-backup-and-fleet-management-enhancements.md`  
**Status**: PENDING  

## Objectives
1. Tree view project spacing & terminal indentation:
   - Expand project card vertical margins (`my-2.5`, card background styling).
   - Indent child items to the right-hand side (`ml-5 pl-3 border-l-2`) with terminal branch glyphs (`├──`, `└──`).
   - Add global Expand/Collapse All toggle button in `TreePanel.tsx`.
2. Filter bar polish:
   - Add `[ 🖼️ Images ]` filter toggle in `TreePanel.tsx` to filter projects and conversations with attachments.
   - Sizing normalization: Restyle `Hide Auto` to `px-2 py-0.5 text-[10px]` matching adjacent status pills.
3. Modernized Sync dropdown:
   - Replace native unstyled browser `<select>` in `ModalHeader.tsx` with a custom dark-glass dropdown menu with hover states and active indicators.
