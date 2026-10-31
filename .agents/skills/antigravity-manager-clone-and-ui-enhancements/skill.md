---
name: antigravity-manager-clone-and-ui-enhancements
description: >-
  Fix IDE cloning freeze, revert instance card action buttons to two lines/levels, simplify clone modal (always full copy + workspace), fix profile auto-selection logic for accounts expiring soon, add IDE settings & prompt import/export (UI & CLI), add install/delete/reinstall Antigravity app (UI & CLI), and conduct end-to-end verification.
---

# Antigravity Manager Clone and UI Enhancements

## Core Intent & Capabilities
- **Clone Reliability**: Eliminate freeze/hang during IDE clone by performing async non-blocking file copies, streaming progress, and avoiding main thread starvation.
- **Cloning Modal Simplification**: Remove profile copy scope option; ensure full IDE clone and workspace copy are always enabled.
- **Card Action Buttons Two-Line Layout**: Revert instance card action buttons from single row back to 2 lines/levels.
- **Account Selection / Quota Expiration**: Fix selection/scoring logic to prioritize accounts whose quota is expiring soonest before reset.
- **IDE Settings & Prompts Import/Export**: Export/import complete IDE settings, pin projects, starter prompts, and read prompts via UI modal and CLI commands (`agm settings export/import` / `agm ide export/import`).
- **Antigravity Install, Delete & Reinstall**: Install IDE option in UI & CLI (`agm ide install`), plus clean purge and reinstall (`agm app clean-reinstall` / `agm ide purge-reinstall`) across Windows, macOS, Linux.
