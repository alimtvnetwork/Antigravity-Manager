# Plan: Enhancing Telegram UI and GitMap Integration

- **Slug:** `enhancing-telegram-ui-and-gitmap-integration`
- **Status:** `COMPLETED`
- **Canonical Spec:** [02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md](../../02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/01-architecture-spec.md)
- **Component Spec:** [02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/02-component-spec.md](../../02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/02-component-spec.md)

## User Request (Verbatim)

```text
# Enhancing Telegram UI and GitMap Integration: high priority instruction, non-negotiable task

What you have in the telegram UI is very poor make it better add more copy commands examples to make it more powerful when seeking help, add more options to the system, getting csv or file format as cmd output, add options, make it extremely easy to prompt to instances, machine and then write the prompt, check account balance easily, add options and release minor bump please

Use the technology from `gitmap` nodes to control multiple machines, instances, or accounts, forwarding and everything else, just like what we do for the deployment. Similar options we can allow using the `gitmap` a little bit, using SSH and following the other end could also receive that delegation and work with it. Craft CLI commands, steps and everything else to ensure we achieve the best, and control multiple machines from one machine. Ensure the ability to see the instances account, which is using what credits and things like that. Import SSH information using the `gitmap` to our CLI and UI.

If the `gitmap` is not installed, suggest installing `gitmap` using the command line argument, or it can itself open the PowerShell and install it. Connect to SSH and other stuff from its UI. In the settings, have SSH option using the IP and machine alias, username, and so on. Password will be asked the first time, log in and then save it as a RSA algorithm encryption, and also can be deployed to other machines with the encryption so that there is no problem. The password should not be saved as plain text anywhere.

## slug: enhancing-telegram-ui-and-gitmap-integration

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/ and enqueue plan task in .ai-memory/plans/enhancing-telegram-ui-and-gitmap-integration.md (subtasks in .ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
4. Use `gitmap` AI agents to enter data
5. Task completion includes committing and pushing to Git
6. Enhance Telegram UI with more copy command examples and options for CSV or file format outputs
7. Implement functionality to prompt instances and machines easily
8. Integrate account balance checking options
9. Use `gitmap` technology to control multiple machines and instances
10. Ensure SSH information can be imported and managed via `gitmap` in CLI and UI
11. Implement installation suggestion for `gitmap` if not present
12. Add SSH settings with RSA encryption for secure password management
```

## Screenshot Evidence

- ![Telegram Switch Alert with HTML entity issue](../../assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-01.png)
- ![Telegram Rich Inline Button Inspiration](../../assets/screenshots/enhancing-telegram-ui-and-gitmap-integration-02.png)

## Subtask Completion Summary

| Task ID | Subtask File | Owner | Owned Files | Status |
| :--- | :--- | :--- | :--- | :--- |
| `Task-01` | `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/01-spec-and-plan.md` | Spec Agents / Lead | `02-spec/21-app/enhancing-telegram-ui-and-gitmap-integration/*`, `.ai-memory/plans/enhancing-telegram-ui-and-gitmap-integration.md` | `COMPLETED` |
| `Task-02` | `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/02-telegram-bot-and-csv-export.md` | Worker 01 | `src-tauri/src/modules/notification_hub/*`, `src-tauri/src/modules/telegram_inbound/*` | `COMPLETED` |
| `Task-03` | `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/03-gitmap-ssh-rsa-and-cli.md` | Worker 01 | `src-tauri/src/modules/ssh_manager/*`, `src-tauri/src/commands/*`, `src-tauri/src/bin/agm/*` | `COMPLETED` |
| `Task-04` | `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/04-frontend-telegram-and-ssh-ui.md` | Worker 02 | `src/components/settings/*`, `src/pages/Settings/*`, `src/services/*` | `COMPLETED` |
| `Task-05` | `.ai-memory/plans/subtasks/enhancing-telegram-ui-and-gitmap-integration/05-release-minor-bump.md` | Lead | `changelog.md`, `changelog_en.md`, `readme.md`, `readme_en.md`, `package.json`, `src-tauri/Cargo.toml` | `COMPLETED` |
