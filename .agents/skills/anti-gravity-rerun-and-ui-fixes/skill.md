---
name: anti-gravity-rerun-and-ui-fixes
description: >-
  Autonomously diagnose and fix Anti-gravity rerun target ID queueing, eliminate unwanted refresh bars on enqueue, repair clipped right-hand tree view filter buttons, render enqueued items immediately in chat/tree without delay, document 4-part RCA, and update coding guidelines with tree view UI/UX design specifications.
---

# Anti-gravity Rerun and UI Fixes

## Overview
This skill guides the investigation and resolution of rerun targeting, enqueue UX refresh bar suppression, tree view filter row compaction, immediate chat queue visibility, and design system documentation.

## Core Capabilities & Directives
1. **Rerun Target ID Enqueueing**:
   - Ensure the rerun action captures the exact target conversation/turn sequence ID and enqueues directly onto that destination.
   - Enqueue immediately as active/queued prompts without 30-second backoff delays (e.g. batch-adding all 4 rerun prompts into the queue instantly).
2. **Enqueue Refresh Bar Suppression**:
   - Prevent disruptive full-tree reloading and visual refresh banners when enqueuing prompts.
   - Update client-side tree and queue states optimistically without triggering background flash.
3. **Tree View Filter Row Compaction & Clipping Repair**:
   - Resolve clipping of the rightmost filter button (e.g. `Hide Archived` / `Hide Audio`) in `TreePanel.tsx` by replacing long text with sleek tooltipped icons and responsive flex wrapping.
4. **Immediate Enqueued Visibility**:
   - Ensure enqueued items appear immediately in the active chat and tree view panels.
5. **Coding Guideline & Design System Documentation**:
   - Document canonical tree view design specifications in `02-spec/02-coding-guidelines/` and `02-spec/04-ui-ux-design-system/`.
6. **4-Part Root Cause Analysis (RCA)**:
   - Persist findings in `.ai-memory/rca/` and `.ai-memory/issues/`.
