# Plan 116: macOS Installation, Gatekeeper Quarantine, and IDE Switching Resilience

## Status: COMPLETED
## Parent Task: 116-macos-installation-ide-detection-and-switching-fix

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
```

---

## Subtask Breakdown

- [x] **Task-01**: `subtasks/116-macos-installation-ide-detection-and-switching-fix/001-installer-quarantine-codesign-and-error-traps.md`
  - Overhaul `install.sh` for macOS: strip quarantine before/after mount, dynamic `.app` discovery, user `$HOME/Applications/` fallback, ad-hoc codesigning (`codesign --sign -`), CLI symlinking (`~/.local/bin/agm`), and POSIX error stack trace trap.
- [x] **Task-02**: `subtasks/116-macos-installation-ide-detection-and-switching-fix/002-backend-macos-process-ide-detection-and-arg-fix.md`
  - Fix macOS process launching and IDE detection in `src-tauri/src/modules/process.rs`: correct argument order with `--args` before application flags, eliminate unrecognized `--new-window` flag error to `open`, add Spotlight `mdfind` lookup, and expand search to `Contents/MacOS/` binaries.
- [x] **Task-03**: `subtasks/116-macos-installation-ide-detection-and-switching-fix/003-backend-macos-instance-launch-and-app-storage-parity.md`
  - Fix `launch_instance_inner_with_extra_workspaces` and `clone_instance_executable` in `src-tauri/src/modules/instance.rs`: branch between `.app` bundles (`open -n -a`) and launcher shell scripts (`Command::new`), and implement missing `.gemini` home and `app_storage.json` setup.
- [x] **Task-04**: `subtasks/116-macos-installation-ide-detection-and-switching-fix/004-first-time-ide-gathering-and-stack-trace-telemetry.md`
  - Implement first-time IDE information gathering and automatic configuration persistence at startup with rich stack traces and structured diagnostic logging.
- [x] **Task-05**: `subtasks/116-macos-installation-ide-detection-and-switching-fix/005-verification-minor-release-and-changelog.md`
  - Execute pre-flight checks (`cargo fmt`, `cargo clippy`, `npm run build`), targeted unit tests, minor version bump, changelog update attributing `@aukgit` `(Thanks to @aukgit)`, README synchronization, and atomic GitMap commit.
