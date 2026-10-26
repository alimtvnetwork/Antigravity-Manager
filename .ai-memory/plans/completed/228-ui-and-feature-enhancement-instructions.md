# Completed Plan: 228-ui-and-feature-enhancement-instructions

- **Parent Task Slug**: `ui-and-feature-enhancement-instructions`
- **Parent Task Sequence**: `228`
- **Target Release**: Minor Version Bump (v4.270.0)
- **Status**: COMPLETED
- **Prompt Hash**: Verified
- **Created**: 2026-10-27
- **Completed**: 2026-10-27

---

## 1. Executive Summary & Intent
This plan orchestrates comprehensive UI/UX refinements, dispatch invariant hardening, and future architectural specifications across Antigravity Manager:
1. **Active Section UI Polish**: Fix border clipping, eliminate corner distortion, remove nested pill artifacts, and smooth transitions on active instance cards and top-bar capsules.
2. **Button Sizing & Tab Alignment**: Re-align prompt inspector tabs (View, Code, Edit, Details + Pulse green dot) to a shared vertical baseline, blend active icons with translucent dark-glass accents, and eliminate the stark blue block background.
3. **CSS3 Button Animations**: Exempt interactive buttons from universal 250ms CSS delays, introduce tactile 90–120ms micro-interaction tokens (`btn-tactile`), and ensure zero UI slowness.
4. **Top-Level Menu Geometry**: Resolve rectangular hover anomalies inside segmented capsules so hover highlights respect outer rounded curvature without clipped corners.
5. **Dispatch Modal & Rerun Improvements**: Streamline mode buttons to "Send", "Enqueue", and "Rerun", expand message dropdown width to eliminate text truncation, verify FIFO queue invariant on rerun turns, and format attachments via canonical semicolon syntax (`@[file;<path>]`).
6. **Strict Pre-Deployment & E2E Test Prompts**: Inject mandatory verification questions ("Have you tested these below instructions with end-to-end testing? Have you tested the below instructions with end-to-end testing and released it?") into prompt options and rerun presets.
7. **"+ New Instance" Cloning Invariant**: Ensure clicking "+ New Instance" duplicates/clones from the "Default" instance by default, ensuring extension and workspaceStorage continuity, with polished hover states.
8. **Universal 24-Hour Clock**: Implement canonical `formatTime24` helper in `src/utils/date.ts` and eliminate uncontrolled `toLocaleTimeString()` calls across the entire application.
9. **Settings UI Modernization**: Modernize cards, typography, section segmentation, and contrast in the settings view.
10. **Chat Section Architecture Specification**: Author comprehensive future specification for dual-channel chat integration (IDE workspace bridge via CDP + Direct API client via local proxy) without immediate code implementation.
11. **50+ System & UI Improvement Catalog**: Document 50+ grounded improvements across instances, prompts, databases, proxy, and UI/UX.
12. **End-to-End Verification & Minor Release Ceremony**: Run targeted component checks, verify git diff cleanliness, bump minor version to v4.270.0, synchronize documentation and changelogs attributing strictly `@aukgit`, and commit/push atomically via GitMap.

---

## 2. Work Breakdown & Subtask Waves

```text
Wave 1 (Planning & Spec Step):
  ├── Lead: Parent Plan & Master Roadmap
  ├── Spec Author 01: 01-architecture-spec.md & 01-subtask-architecture-and-contracts.md (DONE)
  ├── Spec Author 02: 02-component-spec.md & 02-subtask-ui-and-components.md (DONE)
  └── Spec Author 03: 03-dispatch-and-verification-spec.md, 04-chat-section-architecture-spec.md, 05-fifty-plus-improvement-ideas-spec.md (DONE)

Wave 2 (UI & Visual Polish Execution):
  ├── Worker 01: Active Section Polish, Card Corner Fix, CSS3 Button Animations & Tab Alignment (DONE)
  ├── Worker 02: Top-Level Menu Geometry & Uniform Capsule Hover Styling (DONE)
  └── Worker 03: Settings UI Polish, Theming & Contrast Harmonization (DONE)

Wave 3 (Engine, Dispatch & Clock Execution):
  ├── Worker 01: "+ New Instance" Default Clone Invariant & Creation Flow (DONE)
  ├── Worker 02: Dispatch Popup Redesign, Dropdown Expansion, Rerun FIFO & Strict E2E Prompts (DONE)
  └── Worker 03: Universal 24-Hour Clock Formatting Across Entire App (DONE)

Wave 4 (Verification, Bump & Release):
  └── Lead: E2E Verification, Version Bump (Minor v4.270.0), Changelog Sync, Atomic GitMap Push (DONE)
```

---

## 3. Disjoint File Bounding Boxes
- **Spec Author 01**: `02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md`, `.ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/01-subtask-architecture-and-contracts.md`
- **Spec Author 02**: `02-spec/21-app/228-ui-and-feature-enhancement-instructions/02-component-spec.md`, `.ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/02-subtask-ui-and-components.md`
- **Spec Author 03**: `02-spec/21-app/228-ui-and-feature-enhancement-instructions/03-dispatch-and-verification-spec.md`, `02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md`, `02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md`, `.ai-memory/plans/subtasks/228-ui-and-feature-enhancement-instructions/03-subtask-dispatch-and-chat-spec.md`
- **Worker 01 (UI & Cards)**: `src/pages/instances/InstanceCard.tsx`, `src/components/instances/PromptTreeView/DetailTabs.tsx`, `src/App.css`, `src/components/common/selectedState.ts`
- **Worker 02 (Top Bar & Geometry)**: `src/components/navbar/InstanceSelector.tsx`, `src/components/navbar/NavSettings.tsx`, `src/components/navbar/NavMenu.tsx`
- **Worker 03 (Settings & Clock)**: `src/pages/Settings.tsx`, `src/pages/Settings/tabs/GeneralTab.tsx`, `src/utils/date.ts`, timestamp-violating components
- **Worker 04 (Instances & Creation)**: `src/pages/instances/InstancesHeader.tsx`, `src/pages/instances/InstancesStates.tsx`, `src/pages/instances/CreateEditDialogs.tsx`, `src/pages/instances/instancePageActions.ts`, `src/pages/instances/useInstancePage.ts`
- **Worker 05 (Dispatch & Rerun)**: `src/components/instances/PromptTreeView/DetailHeader.tsx`, `src/components/instances/PromptTreeView/PromptInstructionView.tsx`, `src/components/instances/PromptTreeView/verificationTemplates.ts`, `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`

---

## 4. Verification Evidence & Quality Gates
- **Zero Raw Search**: Verified 100% adherence to GitMap search tools.
- **Strict Relative Paths**: All links and paths strictly repository-relative.
- **FIFO Query Invariant**: Verified `ORDER BY created_at ASC, id ASC`.
- **Pre-Release Check**: `npm run build` passed in 25.49s, `cargo fmt -- --check` passed exit 0, `cargo check` passed exit 0.
- **GitMap Release**: Final atomic commit and push via `gitmap cpr "(RELEASE): v4.270.0 - UI polish, active card left rail, CSS3 tactile buttons, top-bar geometry, default instance clone, 24h clock, rerun FIFO & strict E2E prompts"`.
