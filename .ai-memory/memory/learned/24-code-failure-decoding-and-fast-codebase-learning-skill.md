# 24 - Instruction Skill: Decode How Code Fails, Learn From It, Write Correct Syntax, Learn This Codebase Fast

- Written: 2026-10-06T01:55:00Z
- Status: MANDATORY. Every AI must read and follow this before editing any code in this repository.
- Requested by the user (verbatim): "Can this also include your learning so far in decoding how a code can fail, how AI can learn from it, how AI should not make mistakes in the syntax writing, and code-based understanding, where you want to emphasize upon AI to learn and write it as a, like, instruction skill so that AI must follow this and learn the code base very quickly."
- Pointer skill: `.agents/skills/agm-code-failure-decoding/skill.md`
- Related: `.ai-memory/cicd-index.md` (Recurring Failure Classes), `.ai-memory/strictly-avoid.md`, `.ai-memory/memory/learned/23-switch-audit-default-key-and-wrapper-pid-recovery.md`, task register in `.ai-memory/plans/readme.md`, read order in `.ai-memory/what-to-read.md`

Every rule below comes from a real failure in this repository. The commit or write-up is cited so you can read the evidence yourself.

---

## Part 1 - Learn this codebase fast (do this first, every task)

### 1.1 Ten-minute orientation protocol

Run these in order. Do not edit anything until all of them are done.

1. `git log -n 30 --oneline` and `git log -n 10 --stat`. Know what changed recently and who touched the file you are about to edit. If a file was changed in the last 30 commits, read that diff (`git show <sha> -- <file>`).
2. Read `.ai-memory/what-to-read.md` (changelog at the top is the latest state).
3. Read `.ai-memory/cicd-index.md` section "Recurring Failure Classes".
4. Read `.ai-memory/strictly-avoid.md` headings (Grep for `^## `), then read the sections that match your task.
5. Find the code with Grep by symbol name, not by guessing paths. Then read the callers (`Grep "fn_name\("`) and the types it uses (`Grep "struct TypeName"`).
6. Find the existing test next to the function (`Grep "mod tests"` in that file, then the test names). Run it before you edit so you know the baseline.
7. History of a single function: `git log "-L<start>,<end>:<path>" --format="%h %ad %s" --date=short -s`. In PowerShell the `-L` argument must be quoted or the parser fails.

### 1.2 Map of where things live

| Area | File | Key symbols |
|---|---|---|
| Account switch (default instance) | `src-tauri/src/modules/account.rs` | `switch_account`, `set_current_account_id_with_target`, audit `SwitchFacts` |
| Instances, PIDs, launch, close | `src-tauri/src/modules/instance.rs` | `is_instance_running`, `find_pids_for_data_dir`, `record_instance_pid`, `close_instance`, `should_spare_pid`, `other_instance_protection`, `copy_instance_with_options`, `switch_account_to_instance` |
| OS process discovery | `src-tauri/src/modules/process.rs` | `get_antigravity_pids`, `get_antigravity_executable_path` (returns `PathBuf`) |
| Running prompts, resume file, prompt tree | `src-tauri/src/modules/repo_db.rs` | `switch_prompt_snapshot`, `backup_running_prompts`, `dispatch_running_prompts`, `compute_project_conversation_tree` |
| Auto switcher, quota cadence, scoring | `src-tauri/src/modules/auto_switcher.rs` | `calculate_next_interval_seconds`, `MIN_GOOGLE_QUOTA_CHECK_SECONDS` |
| Audit / task history | `src-tauri/src/modules/task_history_db.rs` | `AuditTask` (implements `Drop`), `SwitchFacts` |
| Close and relaunch on switch | `src-tauri/src/modules/integration.rs` | `on_account_switch` |
| Config fields and defaults | `src-tauri/src/models/config.rs` | `#[serde(default = "...")]` functions |
| CLI | `src-tauri/src/bin/agm.rs` (about 13.5k lines) | `cmd_*` functions |
| UI pages | `src/pages/*.tsx` (`Accounts`, `Instances`, `Audit`) | |

### 1.3 Reading very large files

`instance.rs`, `repo_db.rs`, `auto_switcher.rs`, and `agm.rs` are thousands of lines. Never read them top to bottom. Grep for the symbol, then Read with `offset` and `limit` around the line. Read at least 30 lines above and below the edit point so you see the enclosing block, the `mod tests` boundary, and the closing braces.

### 1.4 Know the domain keys before touching data

Different strings look alike and break silently when mixed:

- Instance id: `"default"` (also seen as `"__default__"`) or the instance uuid / `default-copy-8159`. Per-instance data (prompts, saved PIDs, leases, backups) is keyed by this.
- IDE flavor: `target_ide` such as `"ide"`, `"agy"`, or `"instance:<id>"`. It is not an instance id. Issue 62: `switch_prompt_snapshot(target_ide)` returned nothing because prompts are keyed by instance id.
- Conversation identity: `session_id` and `conversation_id` in `.antigravity_resume_task.json`. `agy -p` starts a new prompt and does not resume a conversation.

---

## Part 2 - How code fails in this repository (failure taxonomy with evidence)

Classify every failure into one of these classes before fixing it.

### A. Structural syntax broken by an edit tool

- `01b48a06`: a patch applied by `gitmap pe` removed the `#[test]` and `fn test_instance_binding_stale_or_exhausted() {` header inside `mod tests`, leaving a body with no opening brace. Result: `unexpected closing delimiter`. Fix restored the attribute and the fn line.
- 2026-10-02 session: an edit produced a duplicate `#[test]` on one function and dropped `#[test]` from `test_find_pids_target_path_normalization` (the test silently stopped running).
- RCA 44 (`8ce647c7`): an extra `</div>` inside a `<td>` in `src/components/accounts/AccountRow.tsx` broke the JSX tree (`TS17002 Expected corresponding JSX closing tag`).

Cause: replacing a range without re-reading the full enclosing block.

### B. A variable deleted or moved out of scope during an edit

- `4c28ecf8`: `let has_target = !clean_target.is_empty();` was deleted in `repo_db.rs` while `has_target` was still used on the next line.
- `7555462a` (v4.147.0): `now` used outside its scope.

Cause: editing one line of logic without grepping the other uses of the names it defines.

### C. Type mismatch and move errors

- RCA 43 (`1f0cf651`): `get_antigravity_executable_path` returns `PathBuf`, but callers assigned it to `String` fields (`E0308`). `AuditTask` implements `Drop`, so `Ok(task.id)` cannot move the field (`E0509`); it needs `task.id.clone()`.

Cause: assuming a return type instead of reading the signature.

### D. Borrow and lifetime errors with rusqlite

- `5bb32c51`: a `Statement` from `s_conn.prepare(sql)` was still borrowed when the connection was used again. Fix: hold it in `stmt_opt`, iterate, then `drop(stmt_opt)` before the next use of the connection.

### E. Adding a struct field without updating every initializer

- 2026-10-03: `SwitchFacts` gained `steps`, and `account.rs` failed to compile until `steps: None` was added.

### F. Dependency and feature drift

- `319d2f36` (v4.145.0): rusqlite backup API needed the `backup` feature in `Cargo.toml`.
- `f549e559` (v4.146.0): rusqlite `StepResult` is `#[non_exhaustive]`; a `match` needs a `_ =>` arm.

### G. Visibility and async signature errors

- `7555462a`: `get_antigravity_pids` was private but called from another module.
- `d77ebe3b` (v4.148.0): an integration test invoked an async function the wrong way and failed to compile ("integration test async invocation compile error"). Read the callee signature for `async` before calling it from a test.

Four releases (v4.145.0 to v4.148.0) were burned on classes F and G, one compile error per release, because each fix was pushed without running clippy locally.

### H. Runtime crashes that compile fine

- `2bbcba13` (v4.130.0): `tokio::runtime::Runtime::new()` and raw `tokio::spawn` inside Tauri async commands caused a startup white screen. Use `tauri::async_runtime::spawn` and `tauri::async_runtime::block_on`.

### I. Formatting drift

- 12 of the last 30 commits (v4.151.0 to v4.158.0) only fixed `cargo fmt` or compile errors: `ccf336b1`, `6228e437`, `5bb32c51`, `01b48a06`, `4c28ecf8`, `d56a384e`, `5de41444`, `a19644a7`, `76493525`, `65800177`, `e0ce4f7c`, `8ce647c7`. Rustfmt wraps at 100 columns, sorts imports, and splits long `if` conditions and method chains.

### J. Logic errors that pass every compiler and test

- Wrong key: issue 62 (IDE flavor used as instance id).
- Stale identity: macOS saves the `open` wrapper PID, which exits; trusting it alone reported a live instance as stopped (issue 62).
- Over-broad action: `close_antigravity(None)` during a single-instance switch killed other instances (fixed in v4.125.0 with `should_spare_pid`).
- Silent no-op replacing a verified path: a new fast path with no fallback. AGENTS.md: "Keep an explicit fallback to the verified path whenever you replace it."
- Preview-only conversations dropped because the parser required a `USER_INPUT` line (issue 60, 61).

### K. Git and release mistakes

- Git tracks `readme.md` and `readme_en.md` in lowercase. `git add README.md` staged nothing on a case-insensitive filesystem; fixed in `a3fa0ee1`.
- A cancelled macOS CI job is not green. Rerun with `gh run rerun <id> --failed`, then tag.
- `gh` defaults to upstream `lbjlaq/Antigravity-Manager` in this clone. Pass `-R alimtvnetwork/Antigravity-Manager`.

---

## Part 3 - How the AI must learn from a failure (the loop)

1. Copy the exact error text (file, line, error code). Never fix from memory of the error.
2. Classify it into a class from Part 2.
3. Write the root cause in one sentence. If you cannot, you have not found it yet; read more code.
4. Search for siblings of the same class across the repo (same symbol, same pattern, same struct initializer). Fix the class, not only the line CI reported.
5. Add or update a test that fails without the fix. For logic errors (class J) this is mandatory.
6. Run the full local gate (Part 5), not only the step that failed. CI hides later failures behind the first red step (cicd-index "Hidden failures behind first red step").
7. Record it: an RCA in `.ai-memory/issues/` or `.ai-memory/cicd-issues/`, a ban in `.ai-memory/strictly-avoid.md` (append only), and a line in `.ai-memory/what-to-read.md`.
8. If the same class appears a third time, propose an automated guard (hook, lint, test) in `.ai-memory/suggestions.md`. Prose rules alone failed 11 times for rustfmt.

---

## Part 4 - Syntax-writing rules (must follow on every edit)

### 4.1 Before the edit

- Read the whole enclosing function, plus 30 lines around the edit point.
- Know whether you are inside `mod tests`, an `impl` block, a closure, or a `match` arm.
- For a signature you call, Read the signature. Do not assume `String` vs `PathBuf`, `Option` vs `Result`, sync vs `async`.

### 4.2 While editing

- Use StrReplace with an anchor that includes a unique line such as the `fn` signature. Never replace a range that starts or ends mid-block.
- When editing inside `mod tests`, the replacement must contain the full `#[test]` line and the `fn name() {` line. Count `#[test]` attributes after the edit: exactly one per test function.
- When you delete or move a `let`, Grep the function for every use of that name.
- When you add a struct field, Grep `StructName {` across `src-tauri/src` and update every initializer, including tests. Prefer `#[serde(default)]` plus a default function for config structs.
- When you change a return type or visibility, Grep every caller.
- Hold rusqlite `Statement`s in an inner scope (or `drop` them) before reusing the connection.
- A `match` on an external `#[non_exhaustive]` enum needs a `_ =>` arm.
- Moving a field out of a `Drop` type needs `.clone()`.
- Inside Tauri commands, use `tauri::async_runtime`, never `tokio::runtime::Runtime::new()`.
- Keep lines under 100 columns. Break long `if` conditions and method chains one item per line, the way rustfmt does.
- JSX: every opened tag closes inside the same parent. After editing a table row, count `<td>` against `</td>` and `<div>` against `</div>` in the edited block.
- Do not write a code comment that explains your change to a reviewer. Comments only state constraints the code cannot show.

### 4.3 After the edit

- Read the edited region back once. Check braces, attributes, and that every referenced name is defined.
- Run the gate in Part 5.

---

## Part 5 - Local gate before every push (not after CI fails)

Rust edits (from `src-tauri`, with `$env:CARGO_TARGET_DIR="D:\work\antigravity-manager\src-tauri\target"; $env:CARGO_BUILD_JOBS="1"` on this host):

1. `cargo fmt` then `cargo fmt -- --check`
2. `cargo clippy --all-targets --all-features` with zero errors. Check that new warnings are not on your lines.
3. Targeted tests for the touched module: `cargo test --lib -- --test-threads=1 <test_name>`

Frontend edits: `npm run build`.

Release: tag only after `gh run view <id> -R alimtvnetwork/Antigravity-Manager --json conclusion,headSha` shows `success` for that exact SHA.

Staging: explicit paths only, using the exact case git tracks (`git ls-files | Select-String <name>`). Never `git add -A`, never `--no-verify`, never force-push.

---

## Part 6 - Behaviour rules learned from the user

- Root cause first, generalized fix second. One-off patches are rejected (AGENTS.md "Code Quality").
- Fixes go in the protocol-agnostic pipeline before adapters (AGENTS.md "Backend Fix Strategy").
- Detect broadly, act narrowly. A matcher may recognize a whole class, but its effect stays inside the intended data (another instance's PID is outside it).
- Order side effects to fail before the point of no return: write credentials before killing a process, validate before deleting. Bound every wait with a timeout.
- Never claim something was verified live if it was not. State what was not verified.
- Commit and push after every change without asking.
