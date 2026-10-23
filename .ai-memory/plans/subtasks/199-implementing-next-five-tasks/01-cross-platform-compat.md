# Subtask 199-01: Cross-Platform Compatibility Guard

Status: IN PROGRESS (TAKEN)
Owner: Agent-199-Lead
Spec: `02-spec/21-app/199-implementing-next-five-tasks/spec.md` (Task 1)

## Target
- Create `src-tauri/src/modules/release_gate/cross_platform_compat_guard.rs`
- Create `src-tauri/src/modules/release_gate/tests_cross_platform_compat.rs`
- Do NOT edit `src-tauri/src/modules/release_gate/mod.rs` (lead wires registration).

## Work
Implement a scanner that takes source snippets (file label + content) and reports platform-locked constructs lacking a `cfg` fallback or cross-platform abstraction:
- hardcoded drive-letter paths (e.g. `C:\`, `D:\`) in string literals,
- OS-specific shell invocations (`cmd.exe`, `powershell`, `/bin/sh`) without a platform branch,
- unconditional platform-specific API names,
- literal path-separator assumptions (`"\\"` joins, `"/"` joins on Windows-targeted code) where `PathBuf` should be used.

Each finding: rule id, severity, line number, explanation, remediation hint. Provide `evaluate_files(&[SourceFile]) -> CompatReport` with `is_clean()` and per-file findings. Follow the house guard style of the existing modules in the same folder (verdict structs, pure functions, no I/O).

## Tests (in the tests file, house style)
- Unguarded Windows-only path is flagged.
- `#[cfg(target_os = "windows")]`-gated code passes.
- `PathBuf`/`Path::join` construction passes.
- Bare `cmd.exe` spawn without a fallback ladder branch is flagged (note: the breakaway ladder itself is another guard's job; here only the missing platform branch matters).

## Invariants
Zero builds/tests during implementation. Relative paths only. GitMap-only search. No silent errors.
