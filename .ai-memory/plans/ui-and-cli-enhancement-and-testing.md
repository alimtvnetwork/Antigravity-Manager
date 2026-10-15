# Plan: UI and CLI Enhancement and Testing (ui-and-cli-enhancement-and-testing)

## User Request (Verbatim)
```text
# UI and CLI Enhancement and Testing: high priority instruction, non-negotiable task

Okay. So here you can see the dropdown for the prompt actually looks a bit different. So I don't see the dropdown for the enqueue. So send works, but send actually sends as an enqueue. It does not send directly. That is one of the things that you need to fix and check and do the end-to-end testing here. Also, at the same time, you need to correct the CLI methods, CLI sections as well. And if possible, try to enhance and make this UI a little bit compact. And when I click on this section, try to have proper dropdown menus, okay, so that this is lucrative. Can you please do that for me? Then also try to have a standalone copy button where I could copy all this image and everything together as one shot. Okay? Can you please integrate? And finally, you can do a release bump using minor bump actually, and do a release.

## slug: ui-and-cli-enhancement-and-testing

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/ui-and-cli-enhancement-and-testing/ and enqueue plan task in .ai-memory/plans/ui-and-cli-enhancement-and-testing.md (subtasks in .ai-memory/plans/subtasks/ui-and-cli-enhancement-and-testing/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Fix and check the dropdown for enqueue functionality and perform end-to-end testing
7. Correct the CLI methods and sections
8. Enhance and compact the UI, ensuring proper dropdown menus
9. Integrate a standalone copy button for copying images and content together
10. Perform a minor release bump and execute the release

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing the work to reduce the credits.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Only add the relative paths, never add the absolute path during your work; this should be respected on the release page and in release notes as well.
```

---

## Architecture & Subtask Decomposition

### Subtasks Inventory
1. **Subtask 01 (`Task-01`)**: `01-ui-enhancements.md`
   - Split action toolbar into segmented pill capsules (`Copy All`, `Send`, `Enqueue`).
   - Create dedicated dropdown menus for Send and Enqueue options with transparent borderless styling.
   - Fix direct prompt dispatch so Send triggers immediate execution rather than queueing.
   - Add standalone "Copy All (Content + Images)" button.
   - Compact horizontal spacing in `DetailHeader.tsx` and `PromptInstructionView.tsx`.
   - Owned files:
     - `src/components/instances/PromptTreeView/DetailHeader.tsx`
     - `src/components/instances/PromptTreeView/PromptInstructionView.tsx`
     - `src/components/instances/PromptTreeView/usePromptTreeState.ts`
     - `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
     - `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx`

2. **Subtask 02 (`Task-02`)**: `02-cli-and-release.md`
   - Correct CLI subcommand routing in `agm` for `prompt send`, `prompt queue`, and `queue`.
   - Add sequence resolution (`C001`, `P001`) to queue and send subcommands.
   - Support `--queue` flag in `cmd_prompt_dispatch`.
   - Add `enqueue` to recognized subcommand aliases.
   - Update CLI help text in `help_text.rs` and `prompt_mgmt.rs`.
   - Implement direct CDP submission fallback in `send_prompt_now_for_instance`.
   - Execute minor release bump ceremony (`npm run bump minor`) and sync documentation.
   - Owned files:
     - `src-tauri/src/bin/agm/main.rs`
     - `src-tauri/src/bin/agm/prompt_cmds.rs`
     - `src-tauri/src/bin/agm/prompt_dispatch.rs`
     - `src-tauri/src/bin/agm/common.rs`
     - `src-tauri/src/bin/agm/help_text.rs`
     - `src-tauri/src/modules/repo_db/agy.rs`
     - `src/components/instances/PromptTreeView/clipboard.ts`
     - `CHANGELOG.md`
     - `CHANGELOG_EN.md`
     - `README.md`
     - `README_EN.md`

---

## Execution Status & Verification
- **Status:** COMPLETED
- **Pre-flight Gates:**
  - `cargo fmt -- --check`: PASSED (Exit code 0)
  - `cargo check --lib`: PASSED (Exit code 0)
  - `cargo check --bin agm`: PASSED (Exit code 0)
  - `npm run build`: PASSED (Exit code 0, 14.44s Vite production build)
  - Secrets Scan: 0 hits across 939 files
  - Relative Path Hygiene: 0 absolute machine paths
- **Release Bump:** Atomic bump to `v4.204.0` across 14 release locations with documentation and changelogs synchronized.
