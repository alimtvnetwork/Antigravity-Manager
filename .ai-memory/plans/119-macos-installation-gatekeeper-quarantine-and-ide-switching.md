# Plan 119: macOS Installation, Gatekeeper Quarantine, and IDE Switching Resilience

## Status: IN_PROGRESS
## Parent Task: 119-macos-installation-gatekeeper-quarantine-and-ide-switching

---

## User Request (Verbatim)

```text
# High Priority Instruction

When we try to install in Mac, the problem is it does go to the trash. It cannot be installed. So many issues. Can you please check the Mac code and compare with Windows why it is not working, why it is not getting installed? Find the root cause, try to fix it. Write the root cause analysis, okay? Test it as much as you can. Okay, and make sure that we can install, run, and also switch the IDE in macOS. So take the precautions like finding the IDE information for the first time. Try to see where it is there, and every time that's in the install or anywhere else, try to make sure that you have the stack trace. Okay? Be respectful of the stack trace so that we know where things are wrong, and we can fix them. Does this make sense? Can you please work on this?

at the end make sure you release with a minor bump and change log and don't include absolute paths please

# Actionable Items Must Follow Non-Negotiable

1. Write spec under 02-spec/21-app/<slug>/ and enqueue plan task in .ai-memory/plans/<slug>.md (subtasks in .ai-memory/plans/subtasks/<slug>/) first
2. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
3. Check the Mac code and compare with Windows to identify installation issues.
4. Find the root cause of the installation problem on Mac.
5. Write the root cause analysis.
6. Test the installation process thoroughly on macOS.
7. Ensure the application can install, run, and switch the IDE in macOS.
8. Gather IDE information during the first-time installation.
9. Ensure stack traces are captured and analyzed for troubleshooting.
10. Release with a minor version bump and update the change log.
11. Avoid using absolute paths in the release.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

/plan First before doing the work to reduce the credits.
```

---

## Subtask Breakdown

- [ ] **Task-01**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/001-install-sh-quarantine-codesign-and-error-stack-trace.md`
  - In `install.sh`: pre-mount and post-mount quarantine clearing (`xattr -cr`, `xattr -r -d com.apple.quarantine`), dynamic `.app` bundle discovery inside mounted DMG volumes, user-writable `$HOME/Applications/` permission fallback, ad-hoc codesigning (`codesign --force --deep --sign -`), CLI symlinking (`~/.local/bin/agm`), and POSIX `ERR` trap capturing line numbers, commands, and stack traces.
- [ ] **Task-02**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/002-dmg-packaging-and-gatekeeper-repair-utility.md`
  - Overhaul `scripts/Fix_Damaged.command` with bilingual messages, dynamic app discovery, user-level attribute clearing, and ad-hoc codesigning. Package the repair utility directly inside DMGs via `scripts/package_dmg.sh`.
- [ ] **Task-03**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/003-backend-macos-process-ide-detection-and-stack-trace.md`
  - In `src-tauri/src/modules/process.rs`: ensure `/usr/bin/open` strictly orders custom application arguments after `--args` via `format_macos_open_args`, expand discovery to include `Contents/MacOS/` binaries, and integrate Spotlight `mdfind` fallback.
- [ ] **Task-04**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/004-backend-macos-instance-launching-and-switching-parity.md`
  - In `src-tauri/src/modules/instance.rs`: branch launcher execution between `.app` bundles (`open -n -a`) and shell scripts (`Command::new`), and achieve 100% parity with Windows/Linux by initializing `.gemini` home folders, writing `ide-install-wizard-shown: true`, and syncing `app_storage.json`.
- [ ] **Task-05**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/005-first-time-ide-gathering-and-telemetry-persistence.md`
  - In `src-tauri/src/lib.rs` and `process.rs`: execute `discover_and_persist_initial_ide_info()` on startup, persist discovered executable to `gui_config.json`, and capture `std::backtrace::Backtrace` on unexpected failures.
- [ ] **Task-06**: `subtasks/119-macos-installation-gatekeeper-quarantine-and-ide-switching/006-testing-minor-bump-release-and-changelog.md`
  - Execute pre-flight checks (`cargo fmt`, `npm run build`), unit tests, minor version bump, changelog update strictly attributing `@aukgit` `(Thanks to @aukgit)` without absolute paths, and atomic GitMap commit.
