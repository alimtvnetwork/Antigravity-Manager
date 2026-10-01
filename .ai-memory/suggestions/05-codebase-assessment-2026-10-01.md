# Suggestion 05: Codebase Assessment (2026-10-01)

Scope: an opinion from working on CI/CD, the multi-instance E2E script, and memory upkeep. It is not a full code audit; the source tree was only measured, not reviewed line by line.

## Size snapshot

| Metric | Value |
|--------|-------|
| Rust files under `src-tauri/src` | 191 |
| Rust lines under `src-tauri/src` | ~147,500 |
| `src-tauri/src/bin/agm.rs` | 13,563 lines (one file) |
| Next largest | `proxy/handlers/openai.rs` 7,583; `proxy/token_manager.rs` 5,884; `proxy/server.rs` 5,041; `proxy/thinking_store.rs` 4,807; `proxy/opencode_sync.rs` 4,372 |
| TypeScript files under `src/` | 130 |
| `changelog.md` | 6,495 lines |

## Strengths
- **Clear architecture rule.** AGENTS.md makes the pipeline protocol-agnostic and keeps the four protocol adapters thin. That is the right shape for a gateway and makes generic fixes possible.
- **Headless parity is taken seriously.** Most GUI features have an `agm` CLI command, and `scripts/test-instance-e2e.ps1` exercises real instance switch, fast-forward, prompt backup, and heartbeat end to end.
- **Strong failure memory.** 42 CI/CD RCAs, a recurring-failure-class table in `cicd-index.md`, and `strictly-avoid.md` give a new agent real history instead of guesses.
- **Release discipline exists on paper** (channel separation, attribution rules, changelog/README sync) and is now partly enforced by the pre-commit hook and the release rustfmt gate.

## Risks and what to do about them

1. **`agm.rs` is a 13.5k-line monolith.** Every CLI change touches the same file, so rustfmt drift (RCA 40) and merge conflicts concentrate there, and compile times suffer. *Do:* split by command family into `src-tauri/src/cli/<family>.rs` (instances, prompts, email, supabase, telegram, prune), keeping `agm.rs` as the dispatcher. Move one family per PR.
2. **Several proxy files exceed 4–7k lines** (`openai.rs`, `token_manager.rs`, `server.rs`). Same advice: extract cohesive submodules when next touched, not as a big-bang refactor.
3. **Documentation rules were not enforced, so they drifted.** rustfmt was "required" for months before a hook existed; README version badges show `4.95.0` while the app is `4.109.4`. *Do:* prefer a check script (pre-commit or CI) over a written rule whenever a rule can be checked mechanically, for example badge version equals `package.json` version.
4. **Version bumps mixed with feature commits.** `v4.109.2` and `v4.109.4` each tag a bump that sits right after a large `feat(sync)` commit, and the changelog has no entries for `v4.109.1`–`v4.109.4`. *Do:* run `npm run bump` only on a commit whose CI is green, and let the bump script fail if the newest changelog heading does not match the new version.
5. **Two changelog formats in one file.** `changelog.md` starts with `## [vX.Y.Z]` sections and then switches to a nested `*   **vX.Y.Z (date)**` history list. The release gate matches headings, so entries in the second format are invisible to it. *Do:* pick one format (the heading format) and migrate new entries only.
6. **Memory folder sprawl.** Overlapping folders: `issues/`, `pending-issues/`, `resolved-issues/`, `solved-issues/`; `ambiguous-questions/` vs `question-and-ambiguity/`; `suggestions/` vs `memory/suggestions/`; plan 90 sits at `plans/` root while others use `pending/` and `completed/`. An agent can easily write to the wrong one. *Do:* consolidate to one folder per concept and leave a one-line pointer readme in each retired folder.
7. **Tests compile but do not run in CI**, and Clippy never fails CI. See suggestion 04 items 5 and 6.
8. **Local tooling limits.** `cargo clippy --all-targets --all-features` runs out of memory on the 8 GB Windows host. CI is the only trustworthy Clippy, which makes the "tag only on green CI" rule essential.

## Overall
Feature-rich and well documented. The main risk is not missing features but that the rules live in prose while the code grows faster than the checks. Converting the highest-value rules into hooks and CI gates (started with rustfmt in plan 89) and slicing the largest files will cut most recurring failures.
