# Subtask 04: Contiguous Button Capsules, High-Contrast Yellow States, Tactile 3D Depth & CLI Error Management

- **Parent Plan:** [.ai-memory/plans/pending/225-prompt-tree-view-issues.md](../../pending/225-prompt-tree-view-issues.md)
- **Parent Spec:** [02-spec/21-app/225-prompt-tree-view-issues/04-buttons-contrast-and-error-spec.md](../../../../02-spec/21-app/225-prompt-tree-view-issues/04-buttons-contrast-and-error-spec.md)
- **Task ID:** `225`
- **Subtask ID:** `04`
- **Status:** `READY_FOR_EXECUTION`
- **Owner:** Worker 03 (Media, Button & Error Architect)
- **Date:** 2026-10-24

---

## 1. Context & Objectives

During recent UI and system audits, several critical design defects and tooling gaps were highlighted:
1. **Disjoint Circular Buttons with Dashes:** Toolbar action clusters in `InstanceRowActions.tsx`, `ModalHeader.tsx`, and `DetailHeader.tsx` exhibit disconnected circular shapes separated by vertical dash spacers (`<div className="w-px h-3.5 bg-slate-300..." />`).
2. **Low-Contrast Yellow Buttons:** Yellow hover and active states (Reset, Fast-Forward, Switch) fail WCAG AA contrast rules by applying light or white text/icons against bright yellow backgrounds (`#FACC15`, `amber-400`).
3. **Missing Tactile 3D Elevation:** Reset and Refresh buttons blend indistinguishably into dark-glass modal backgrounds without tactile elevation or distinct button personalities.
4. **Non-Clickable Image Links in IDE:** Images displayed in Prompt Tree View cannot be clicked as files when exported or viewed by developers in their IDEs.
5. **CLI Error Management & Agent Skill:** Lack of a documented, readily accessible AI skill for `agm errors` to query, clear, and heal errors from the command line.

### Core Objectives:
- Refactor all toolbar action clusters into contiguous segmented pill capsules (`rounded-full`, shared border, `divide-x`, dark-glass styling) with zero internal circle rounding or disjoint dash lines.
- Enforce high-contrast dark text (`text-zinc-950` / `text-slate-900`) and bold font weight across all yellow/amber button surfaces.
- Implement physical tactile 3D elevation depth: solid bottom bevel shadows (`shadow-[0_2px_0_0_#...]`), active press translations (`active:translate-y-0.5 active:shadow-none`), and distinct color identities for Reset (Amber) and Refresh (Sky).
- Normalize IDE file links using the standard `[filename.ext](file:///D:/...)` markdown format and add an interactive "Open in IDE" button via Tauri opener.
- Author and publish the AI agent skill `.agents/skills/agm-error-inspection/skill.md` for CLI error inspection and self-healing.

---

## 2. Targeted Files & Areas

- `src/components/instances/PromptTreeView/ModalHeader.tsx` (Segmented capsule layout, Reset/Refresh 3D buttons, high-contrast dark text)
- `src/components/instances/table/InstanceRowActions.tsx` (Root-level instance table action capsule, eliminate dashes and disjoint circle artifacts)
- `src/components/accounts/AccountCard/AccountCardActions.tsx` (Yellow hover high-contrast text fix)
- `src/components/instances/PromptTreeView/PromptInstructionView.tsx` (Clickable IDE file links, "Open in IDE" action, accurate turn-level image mapping)
- `src/components/instances/PromptTreeView/clipboard.ts` (IDE clickable file formatters and path normalization)
- `src/components/instances/PromptTreeView/markdown.tsx` (Inline IDE file click handler via Tauri opener)
- `src-tauri/src/bin/agm/error_cmds.rs` (CLI error commands review and JSON formatting parity)
- `.agents/skills/agm-error-inspection/skill.md` (New canonical agent skill documentation)

---

## 3. Step-by-Step Implementation Plan

### Step 4.1: Root-Level & Header Contiguous Capsule Harmonization
- Open `src/components/instances/table/InstanceRowActions.tsx`:
  - Remove inner `rounded-l-full` wrapper and standalone vertical divider lines (`w-px h-3.5`).
  - Ensure outer container encapsulates all actions with `rounded-full`, shared border, and `divide-x`.
  - First action receives `rounded-l-full`; last action receives `rounded-r-full`; middle actions have rectangular flush edges.
- Open `src/components/instances/PromptTreeView/ModalHeader.tsx`:
  - Ensure the header toolbar uses the single contiguous segmented pill capsule without nested pill borders or broken margins.

### Step 4.2: High-Contrast Yellow Text & 3D Tactile Elevation Depth
- Update Reset button styling in `ModalHeader.tsx`:
  ```tsx
  className={cn(
      "flex items-center justify-center gap-1.5 h-7 px-3 text-[11px] font-bold transition-all duration-150 ease-out cursor-pointer disabled:opacity-50",
      "bg-amber-400 hover:bg-amber-300 text-zinc-950",
      "shadow-[0_2px_0_0_#b45309] active:translate-y-0.5 active:shadow-none"
  )}
  ```
- Update Refresh button styling in `ModalHeader.tsx`:
  ```tsx
  className={cn(
      "flex items-center justify-center gap-1.5 h-7 px-3 text-[11px] font-semibold transition-all duration-150 ease-out cursor-pointer disabled:opacity-50",
      "bg-sky-600 hover:bg-sky-500 text-white",
      "shadow-[0_2px_0_0_#0369a1] active:translate-y-0.5 active:shadow-none"
  )}
  ```
- Update yellow hover states in `AccountCardActions.tsx` and `InstanceRowActions.tsx`:
  - Ensure `hover:text-zinc-950 hover:font-bold` accompanies any `hover:from-[#FDE047] hover:to-[#EAB308]` yellow gradient.

### Step 4.3: Clickable IDE Image Formatter & 1-Click Opener
- In `src/components/instances/PromptTreeView/clipboard.ts`:
  - Implement `formatIdeClickableFileLink(filePath: string, label?: string): string`.
  - Format paths using RFC-8089 `file:///` forward-slash syntax so clicking in IDE terminals and chats opens the file.
- In `src/components/instances/PromptTreeView/PromptInstructionView.tsx`:
  - Add an "Open in IDE" quick action button on image cards that calls Tauri's `app.opener().open_path(...)`.
  - Validate image turn positioning to ensure images extracted from transcripts match their exact prompt turn step index.

### Step 4.4: Document AI Skill for AGM Error Inspection
- Author `.agents/skills/agm-error-inspection/skill.md`:
  - Document `agm errors list`, `agm errors clear`, `agm errors fix`, and `agm errors inspect`.
  - Provide machine-readable and interactive examples.
  - Register the skill in the workspace index.

---

## 4. Verification Checkpoints

1. Inspect `InstanceRowActions.tsx` in UI: Confirm smooth contiguous pill capsule with zero circular-in-between artifacts and zero floating dashes.
2. Inspect `ModalHeader.tsx`: Confirm Reset button displays deep dark text (`text-zinc-950`) over amber with physical 3D elevation; Refresh button displays crisp white text over sky blue.
3. Test image copy and markdown view: Confirm copied links appear as `[name](file:///...)` and open properly in IDE.
4. Run `agm errors` from CLI: Verify error list output format and clean JSON parsing.
