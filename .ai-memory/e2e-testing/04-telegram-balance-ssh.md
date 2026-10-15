# 04 — Telegram, balance, SSH (v4.203.0)

Release `v4.203.0`, commit `876dc50b`. These are the checks the AI executed on that change. GUI click-through of every new settings control was not a live browser pass. `npm run build` passed. Rust tests for crypto, telegram inbound, and ssh manager passed.

## Unit checks (run before CLI)

- [ ] `cargo test --lib utils::crypto::tests` — RSA encrypt then decrypt returns the same password. Cipher text does not contain the password. Prefix is `ag_rsa_v1_`.
- [ ] `cargo test --lib modules::telegram_inbound::tests`
- [ ] `cargo test --lib modules::ssh_manager::tests`
- [ ] `cargo fmt -- --check` and `cargo clippy --all-targets --all-features` exit 0.

## Balance and CSV

- [ ] `agm balance`
  - Pass: a table with columns MACHINE, IP, INSTANCE, BOUND ACCOUNT, TIER, 4H QUOTA, WEEKLY.
  - Observed on this host: local rows for W3 (`192.168.1.3`) plus SSH node rows. Do not require those exact emails on another machine.
- [ ] `agm balance --csv`
  - Pass: stdout starts with `machine,ip,instance_id,instance_name,running,account_email,tier,quota_4h_pct,quota_weekly_pct,min_model_pct,threshold_pct`
  - Fail: tracing logs on stdout when `--csv` or `--json` is set.
- [ ] `agm accounts --csv`, `agm instances --csv`, `agm ssh nodes --csv` print CSV and exit 0.

## SSH and GitMap

- [ ] `agm ssh status` prints GitMap installed yes or no, RSA vault state, then the node table.
  - Decision: the word `status` must be a subcommand. The first build treated `agm ssh status` as `ssh root@status` and asked for a password. That is a fail. The fix is in `src-tauri/src/bin/agm/ssh_cmds.rs`.
- [ ] `agm ssh nodes` lists alias, host, user. Empty list is allowed on a machine with no nodes. It is not a fail by itself.
- [ ] `agm ssh import-gitmap` only if GitMap is installed and the user wants nodes imported. Pass: inserted or updated counts, and the stored password field is the RSA cipher, never the raw password.
- [ ] If `gitmap --version` fails, `agm ssh` must print the install hint, and `agm ssh install-gitmap` must be the command that opens the installer. Do not run the installer during a read-only test.

## Telegram (bot must already be configured)

- [ ] A switch alert must not show the literal text `&nbsp;` or `&bull;`. Indent is a Unicode figure space.
- [ ] The alert has inline buttons: Check Balance, Export CSV, Prompt Instance, Refresh Status.
- [ ] `/help` includes copyable examples for `/balance`, `/export accounts csv`, `/prompt <machine:instance> <text>`, `/ssh nodes`.
- [ ] `/balance` returns the credit lines, not only the switch card.
- [ ] `/export accounts csv` arrives as a document, not only as HTML text.
- [ ] If the bot token is missing, write `NOT RUN` and do not invent a token.

## Settings UI (when the GUI is open)

- [ ] Settings, SSH & GitMap (`/settings/ssh`): GitMap version or the install banner, node form (alias, host, port, username), badge that the password is RSA-2048 and not stored as plain text.
- [ ] Telegram settings: click-to-copy chips for `/balance`, `/export`, and `/prompt`.
- [ ] Saving a node with a password then reading the node file shows `ag_rsa_v1_` and not the password characters.

## Stop

- [ ] No password or bot token in the result block.
