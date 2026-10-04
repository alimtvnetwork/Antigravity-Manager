# 116 — macOS Installation, Gatekeeper Quarantine, and IDE Switching Resilience

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

## 1. Acceptance Criteria

- [ ] **AC-01 (Installation Resilience)**: `install.sh` on macOS mounts the DMG, dynamically discovers `.app` bundles, falls back to `$HOME/Applications/` if `/Applications/` is read-only, clears quarantine via `xattr -cr`, applies local ad-hoc code-signing via `codesign --sign -`, and symlinks `agm` CLI to `$HOME/.local/bin/agm`.
- [ ] **AC-02 (Error Traps & Stack Traces in Shell)**: `install.sh` handles errors with an explicit `ERR` trap and prints detailed failure line numbers and commands.
- [ ] **AC-03 (macOS IDE Detection & Arguments Parity)**: `src-tauri/src/modules/process.rs` correctly passes `--args` before any flags when calling `open`, supports Spotlight `mdfind` searches, and detects executables within `.app/Contents/MacOS/`.
- [ ] **AC-04 (macOS Instance Launching & Script Execution)**: `src-tauri/src/modules/instance.rs` executes shell scripts directly via `Command::new` rather than passing them to `open -a`, and initializes `.gemini` home folders and `app_storage.json`.
- [ ] **AC-05 (First-Time IDE Discovery & Auto-Persistence)**: Probes and persists detected IDE paths on first run, logging structured diagnostics and stack traces.
- [ ] **AC-06 (Release Discipline)**: Minor version bump, changelog updating attributing `@aukgit` `(Thanks to @aukgit)`, synchronized README files, and zero absolute paths.

---

## 2. Traceability Matrix

| Requirement | Implementation Target | Verification Step |
| :--- | :--- | :--- |
| AC-01 (Installation) | `install.sh` (`install_macos`) | `install.sh --dry-run` and DMG inspection |
| AC-02 (Stack Traces) | `install.sh` (trap handler) | Error trigger and trap output test |
| AC-03 (IDE Detection) | `src-tauri/src/modules/process.rs` | Unit tests in `process.rs` |
| AC-04 (Instance Launch) | `src-tauri/src/modules/instance.rs` | Unit tests in `instance.rs` |
| AC-05 (First-Time Setup) | `src-tauri/src/modules/process.rs` / `lib.rs` | Startup verification and diagnostics log |
| AC-06 (Release) | `package.json`, `Cargo.toml`, `CHANGELOG.md` | Pre-flight checks and `git status` audit |
