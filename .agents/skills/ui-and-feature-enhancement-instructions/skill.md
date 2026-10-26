---
name: ui-and-feature-enhancement-instructions
description: Autonomously execute UI improvements, CSS3 button animations, tab alignment, release page rerun integration, strict E2E test prompts, default 24h clock, clone-from-default instance behavior, and future chat architecture specifications.
---

# UI and Feature Enhancement Instructions

## Operational Mandates
- **Active Section UI**: Eliminate visual glitches, refine borders, badges, and active state indicators to look professional and cohesive.
- **Button Visuals & CSS3 Animations**: Center detail and tab buttons, ensure icons blend smoothly, add subtle CSS3 animations (hover, active, transition) with zero slowness.
- **Rerun & Dispatch Configuration**:
  - Simplify mode button labels to "Send", "Enqueue", "Rerun" (move extra descriptions to tooltips).
  - Enforce FIFO enqueue invariant on rerun turn.
  - Widen dropdown UI.
  - Fix image attachment placement and ensure release page is accessible in rerun.
- **Strict E2E Test & Deployment Prompts**: Include strict pre-deployment verification prompt ("Have you tested these below instructions with end-to-end testing and released it?").
- **New Instance Behavior**: Clicking "+ New Instance" must duplicate/copy from the Default instance rather than a blank template, and feature polished hover states.
- **Time Formatting**: Default clock format across all UI surfaces must be 24-hour clock.
- **Settings UI Polish**: Clean, elegant theming and color palettes without disjointed styling.
- **Main Menu Rectangular Styling**: Ensure top-level menu capsules and hover highlights strictly follow uniform geometry without clipped rectangular/pill anomalies.
- **Future Chat System Architectural Spec**: Thoroughly specify dual-channel chat integration (IDE workspace bridge + Direct API client) for main menu without immediate implementation.
