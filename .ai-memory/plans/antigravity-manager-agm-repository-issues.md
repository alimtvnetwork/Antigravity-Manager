# Plan: antigravity-manager-agm-repository-issues

Status: COMPLETED
Branch: main
Date: 2026-10-17

## User Request (Verbatim)

```text
# Antigravity Manager (`Antigravity Manager (AGM)`) Repository Issues: high priority instruction, non-negotiable task

There are several issues in the `Antigravity Manager (`Antigravity Manager (AGM)`)` repository. First of all, in the Telegram section. When working with Telegram, it's nice these steps have been pulled off. A couple of serious issues. My machine has the older version of the Antigravity tools, which the Telegram is sending information to, v1.9.0. That is some root cause to investigate in the machine. In the `Antigravity Manager (`Antigravity Manager (AGM)`)`, there should be an other version clean command, or clean space other version, or OV. That would need CLI help to remove all other versions from the machine. Keep the account information inside the SQLite DB. Telegram can send the prompt, but it sends an ``Id`` with it, which is not necessary. It should just send the prompt given, and the screenshot is not correct. In Telegram, there is a summary of the projects and the title of the conversation, but a little bit of subpoint is needed. Prompts, let's say 200 words or 100 words, should be configurable from the settings UI and CLI. After selecting the project, previous prompts, a short version, let's say 100 words, 200 words, should be visible and resendable. In the instances, the list view goes right-hand side, which is not correct, and the data executable should be one column. The prompt button should be outside in the list view of the instance. The send button has an overlapping issue in the UI. The prompt tree does not have the updated prompt. The refresh button does not take the recent prompts to the UI. Proper synchronization is needed. The copy all with image feature is not working as expected. Closing the anti-gravity tool should end all processes. The progress bar does not have different color segments. In the log section, buttons like log should have a section called data-UI-name. Inside the log, there should be a data-UI ``Id``. The audit log should show a friendly name for buttons. The audit log summary should have a copy button. From the CLI, a signal can be sent to any AI, like `Antigravity Manager (`Antigravity Manager (AGM)`)` read logs. The CLI segment should be powerful. The send button from the prompts should be fixed. The synchronization of the prompts is not working as expected. The account switch functionality should be 15%, not 98%. Backing up the prompt, resending the prompt does not work. Enhance the UI for the prompt tree. Copy content with images should work, and buttons should be compacted. The UI should be more enticing. The updater UI needs improvement following `gitmap`. Add features for Telegram, fix the screenshot issue, add the clear issue of the other version. The latest version should be the default one. When updating to the older version, it should clean up the machine and revert to that older version. Keep the settings back in. Verify the code thoroughly. In the release note, remove the segment thanking aukgit. Check the PR from the other repo inside the code base. Provide critical feedback on changes. Confirm checking the fleet option for multiple machines. In the `Antigravity Manager (`Antigravity Manager (AGM)`)` log, filter out the system log. Find the root cause of the Telegram log issue and fix it if possible.
```

## Media & Visual References

- `assets/screenshots/antigravity-manager-agm-repository-issues-01.png`
- `assets/screenshots/antigravity-manager-agm-repository-issues-02.png`
- `assets/screenshots/antigravity-manager-agm-repository-issues-03.png`
- `assets/screenshots/antigravity-manager-agm-repository-issues-04.png`

## Subtask Breakdown

- `01-telegram-version-and-clean-other-versions.md`: Root cause Telegram `v1.9.0` issue; implement `agm clean-other-versions` / `agm ov` CLI commands preserving SQLite databases; fix prompt `Id` stripping, screenshot capture, and configurable 100/200-word summaries.
- `02-instances-table-and-prompt-button.md`: Fix list view horizontal overflow; combine Exe and Data directory into a single high-density column; move Prompts button outside next to Switch and Stop.
- `03-prompt-tree-overlap-sync-and-copy.md`: Fix Send button / view tabs overlapping defect in `DetailHeader.tsx` (Media 1); fix prompt tree refresh and disk synchronization; fix copy-all-with-images pipeline (Media 3).
- `04-progressbar-and-ui-enhancements.md`: Redesign quota progress bar with distinct color-segmented track (Media 2); updater UI following GitMap conventions; compact action buttons and styling.
- `05-log-attributes-audit-copy-and-release.md`: Add `data-ui-name` and `data-ui-id` to buttons/logs; friendly names in audit log; copy button in audit summary; system log filtering; 15% switch quota threshold; release notes without thanking segment; fleet cluster check (Media 4).
