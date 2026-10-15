# Decisions the AI used

Tick these before any case. Each line is a choice a later AI must repeat, not invent.

- [ ] Score only the binary whose version matches the commit under test. On 2026-10-14 the installed `agm` 4.197.0 still wiped `prompt_tree_cache`. The local debug binary 4.197.1 kept it. A green reading of source is not a pass.
- [ ] Prefer the copy instance `default-copy-5597` for switch, fast-forward, and prompt send. Leave the Default IDE process alone. On the 2026-10-15 run Default was `Antigravity.exe` PID 10276 and was not touched.
- [ ] `agm auto-switch test` is a dry decision. `agm ff -i <copy> --force` is a real rotation. Do not run a real rotation on Default unless the user asked for that instance.
- [ ] A threshold of 98% is temporary. End with `agm auto-switch status` showing 15.0. Leaving 98 makes the live daemon rotate on the next tick.
- [ ] CLI `agm prompts backup` / `restore` is the SQLite snapshot path. The Prompt Tree Backup and Restore buttons are a different path (native file dialog). Passing one does not pass the other.
- [ ] CLI `agm prompts send` exit 0 is not a window Send pass. Confirm `agy.exe` stays up and its command line has `--conversation` and does not have `--resume`.
- [ ] `agm instances open-ui` proves the manager process started. It does not prove a specific settings screen. A screen pass needs the window or a screenshot of that screen.
- [ ] Supabase `NXDOMAIN` means the free-tier project is paused. Do not create a new project. Do not call it a bad API key.
- [ ] Do not search the repo with `rg`, `grep`, or `Select-String`. Use `gitmap aum search`.
- [ ] Do not write absolute paths into tracked notes. Record instance ids, prompt ids, exit codes, and relative log names.
