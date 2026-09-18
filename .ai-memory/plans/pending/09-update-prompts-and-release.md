# Plan: Update Prompts and Release

**Status:** PENDING

## Problem Statement
The user explicitly requested to fetch the latest prompts from the prompt architect repository and subsequently execute a full version release.

## Acceptance Criteria
1. Execute scripts/update-prompts-from-architect.ps1 to pull the latest prompts.
2. Commit the prompt changes (if any).
3. Execute the release pipeline (bumping the minor version).
4. Run
pm run sync to ensure health scores, version maps, and the README are correctly synced and stamped.
5. Regenerate installer bundles using scripts/generate-bundle-installers.mjs.
6. Push the new tags and provide the installation script in the final output.

## Subtasks
- 01-update-prompts.md: Run the prompt updater script.
- 02-release.md: Run the release ceremony.
