# Plan: Instance Clone Settings & Projects, Audit Dark Theme, and UI Polish

## Overview
Implement deep copy for instance cloning (IDE settings, themes, read folders, browser settings, and project rows), full dark theme support in Audit view, compact redesign of Settings About tab, upgraded CSS3 animated buttons and UI for duplicate profile modal, high-contrast account row selection styling, and expanded email metrics display.

---

## Subtasks
1. `01-prompt-resumption-rca-memory.md` — RCA documentation in AI memory & spec (Done).
2. `02-instance-cloning-settings-and-projects.md` — Deep copy of `settings.json`, themes, roaming/appdata, and `repo_prompts.db` project rows.
3. `03-audit-dark-theme-fix.md` — Complete dark mode classes across `src/pages/Audit.tsx`.
4. `04-settings-about-compact-redesign.md` — Compact, modern card redesign of About tab in `src/pages/Settings.tsx`.
5. `05-duplicate-modal-and-css3-buttons.md` — Redesign Duplicate modal with compact, CSS3 animated buttons in `InstanceSelector.tsx` & `Instances.tsx`.
6. `06-account-row-selection-highlight.md` — Dark blackish background with yellow/gold distinguished text in `AccountTable.tsx`.
7. `07-email-section-display-enhancement.md` — Rich connection badges and status telemetry in `src/pages/Email.tsx`.
8. `08-verification-and-release.md` — Pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`), minor version bump, changelog sync with `@aukgit`, commit, and CI/CD verification.
