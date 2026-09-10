# Plan: Guideline Prompt and Installer Upgrade

**Status:** PENDING

## Acceptance Criteria

1. Restructure the coding style index/overview file into a "prompt" with "must follow instruction" when the AI reads it.
2. Put the rule about "method arguments in each line" in the root of the repo (or root of the coding guideline) formatted for AI.
3. Update the PS and Bash installer generation (generate-bundle-installers.mjs) to handle file removal during upgrades using a manifest in the inner JSON section (codingGuideline), ensuring it NEVER touches the root Version.
4. Ensure the installers emit a JSON summary after running so other AI/CLI tools can parse the installation results.
5. Create a file outlining 50 improvements in the 02-spec/03-error-manage folder.
6. Execute N-Step continuous loop gracefully handling all constraints (MD022, MD032, no [N], no generics).

## Subtasks

- 01-format-overview-prompt.md: Rewrite 00-overview.md (or index) as a powerful AI prompt and extract the method signature rule.
- [x] 02-upgrade-installer-scripts.md: Modify scripts/generate-bundle-installers.mjs to add deletion logic (based on manifest array), never touch root version, and output .lovable/install-summary.json. (✅ Done)
- 03-generate-02-improvements.md: Audit 02-spec/03-error-manage and draft 50 architectural/functional improvements.
- 04-release-6.29.0.md: Release script execution and readme sync.
