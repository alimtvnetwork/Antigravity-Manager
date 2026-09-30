# CI/CD RCA 40: Recurring Rustfmt Drift (11th Occurrence) and Releases Shipping While CI Was Red

## 1. Symptoms

`CI` on `main` failed on every completed run from `db5a3787` through `2fe82117` (2026-09-30). The failing step changed over time:

| Commits | Failing steps | Status |
|---------|---------------|--------|
| `db5a3787`, `7f7eb793` | `Run Clippy`, `TypeScript check`, `Build Tauri app (debug)` | Fixed by later commits |
| `bf491cfd` … `a72f9e9a` | `Check Rust formatting`, `TypeScript check`, `Build Tauri app (debug)` | TS/build fixed in `c4983761` (`Accounts.tsx` use-before-declare, `agy-clean-modal.tsx` Escape handler) |
| `c4983761` … `2fe82117` | `Check Rust formatting` only (all 3 OS) | Fixed in this RCA |

The remaining failure:

```text
Run cd src-tauri && cargo fmt -- --check
Diff in src-tauri/src/bin/agm.rs:8609:
-    let non_flag_args: Vec<String> = args.iter().filter(|a| !a.starts_with('-')).cloned().collect();
+    let non_flag_args: Vec<String> = args
+        .iter()
+        .filter(|a| !a.starts_with('-'))
+        .cloned()
+        .collect();
```

Meanwhile the `Release` workflow published `v4.109.1`, `v4.109.2`, `v4.109.3`, `v4.109.4` successfully from commits whose CI was red.

## 2. Root Cause

**Direct cause:** Release-bump commit `d606ae20` (`chore(release): bump version to 4.109.0`) also carried hand-edited Rust changes in `src-tauri/src/bin/agm.rs`, `instance.rs`, `email_sender.rs`, and others. One line exceeded rustfmt's width and was never formatted. `git blame` attributes the line to `d606ae20`.

**Why it keeps recurring (the real root cause):** The pre-flight rule "run `cargo fmt -- --check` before committing" existed only as documentation (`AGENTS.md`, RCAs 02/12/16/18/20/22/27/28/29/34). Nothing enforced it:

1. No git hook. `core.hooksPath` was unset and `.git/hooks` held only samples, so any commit (human, AI agent, `npm run bump --commit`, sync scripts) could land unformatted Rust.
2. Release workflow did not gate on formatting. `verify-release-target` checked only tag/branch alignment, so every tag pushed on a red commit still built and published binaries.
3. Every run was then followed by a new commit, which cancelled the in-flight CI (`cancelled` entries for `ae43ea9d`, `4b9b36fe`, `056ad955`, `8d6182e9`). Nobody saw a finished red run for the latest commit until later.

Documentation-only gates fail at scale because agents and bump or sync scripts do not consistently read or apply them.

## 3. Fix

1. `cd src-tauri && cargo fmt`: reformatted `agm.rs:8612`. `cargo fmt -- --check` exits 0.
2. Enforced gate, local: tracked `.githooks/pre-commit`. When any staged `src-tauri/*.rs` file exists, it runs `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check` and blocks the commit with the diff and fix command. If `cargo` is missing it prints a warning instead of blocking (CI still catches it).
3. Auto-install: `scripts/install-git-hooks.mjs` sets `git config core.hooksPath .githooks`. It is wired to `npm install` through the `prepare` script, and also runnable as `npm run hooks:install`. It never fails the install when `.git` or `git` is absent.
4. Enforced gate, release: `release.yml` → `verify-release-target` runs `cargo fmt -- --check` (rustfmt component only, no compile) before any build job starts. A tag on an unformatted commit is intercepted instead of publishing. This lands as a separate commit because pushing workflow files needs a token with the `workflow` scope (`gh auth refresh -h github.com -s workflow`); the default `gh`/GCM token here has only `repo`.

Verified locally: the hook blocks a deliberately unformatted staged line (exit 1) and passes the formatted tree (exit 0).

## 4. Prevention Rules (for every AI model reading this)

- **Never mix code edits into a release-bump commit.** `npm run bump` commits must contain only manifest, version, and changelog changes. Land code in its own commit first and let CI go green.
- **Never bypass the hook with `--no-verify`** to "just push". If the hook blocks, run `cd src-tauri && cargo fmt` and re-stage.
- **Do not tag a release until the CI run for that exact SHA has completed green.** A `cancelled` run is not a pass. Check with `gh run list -R alimtvnetwork/Antigravity-Manager --commit <sha>`.
- **`gh` defaults to `upstream` (`lbjlaq/Antigravity-Manager`)** in this clone. Always pass `-R alimtvnetwork/Antigravity-Manager` when checking CI, or you will read the wrong repo's green runs.
- Editing `.github/workflows/*` requires pushing with a `workflow`-scoped token; a `repo`-only token is rejected with "refusing to allow a Personal Access Token to create or update workflow". Keep workflow edits in their own commit so the rest of a fix can still ship.
- After a fresh clone, run `npm install` (or `npm run hooks:install`) so the pre-commit guard is active. Confirm with `git config core.hooksPath`, which should print `.githooks`.

## 5. Related RCAs

Rustfmt drift: 02, 12, 16, 18, 20, 22, 27, 28, 29, 34 (all documentation-only remediations; superseded by the enforced gates above). Releases with missing or invalid artifacts: 39.
