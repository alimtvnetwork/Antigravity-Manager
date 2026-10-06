# Honest Codebase Review — Antigravity-Manager

- **Reviewed version:** v4.160.0 (`a5698e0e`), `main` branch
- **Review date:** 2026-10-06
- **Reviewer:** AI-assisted audit (Muse Code) — measurements taken from the working tree, not from memory
- **Scope:** full tree: `src-tauri/` (Rust backend + CLI), `src/` (React frontend), `docs/`, `.github/workflows`, `.agents/skills`, scripts

## 1. Verdict up front

This is a **genuinely well-architected project with a file-size discipline problem**. The core idea — four chat protocols normalized into one canonical Gemini IR, with all business logic in a decoupled pipeline — is the right design, it is written down (`AGENTS.md`, `agm-core-architecture` skill), and the code mostly honors it. Inline test volume is strong (1,036 test attributes), the tree carries almost no TODO/FIXME litter (4 hits), and release/CI hygiene is above average for a solo/small-team project.

The honest counterweight: the same repo that documents a **500 KB per-file guard (Rule R19) ships a 594 KB / 14,251-line CLI file** (`src-tauri/src/bin/agm.rs`), plus three more 6–7k-line backend files. There are **~1,600 `.unwrap()`/`.expect()` calls** in server-adjacent code, **4 frontend test files for 148 TS/TSX files**, and **228 skill directories** whose maintenance cost is real. None of this is fatal. All of it is fixable without redesign.

**Scorecard (opinion, 1–10):** architecture 8, backend tests 7, frontend tests 3, file modularity 4, docs/ops 8, overall 7.

## 2. By the numbers (measured, 2026-10-06)

| Metric | Value | Source |
|---|---|---|
| Rust files / LOC | 194 files / ~160,000 lines | `src-tauri/src/**/*.rs` |
| TS/TSX files / LOC | 148 files / ~53,000 lines | `src/**/*.{ts,tsx}` |
| Proxy subsystem | 108 Rust files | `src-tauri/src/proxy/` |
| Largest file | `src-tauri/src/bin/agm.rs` — 594 KB, **14,251 lines** | measured |
| Next largest | `proxy/handlers/openai.rs` 7,029 · `modules/instance.rs` 6,973 · `modules/repo_db.rs` 5,911 · `pages/ApiProxy.tsx` 2,850 | measured |
| Rust test attributes | 1,036 (`#[test]` + `#[tokio::test]`) | measured |
| `#[ignore]` (quarantined) | 1 | measured |
| Frontend test files | 4 (`*.test.*` / `*.spec.*`) | measured |
| TODO/FIXME/HACK markers | 4 | measured |
| `.unwrap()` / `.expect()` | ~1,600 | measured |
| `panic!/unimplemented!/todo!/unreachable!`/clippy-allows | 33 | measured |
| CI workflows | `ci.yml`, `release.yml`, `deploy-pages.yml`, `purge-actions-artifacts.yml` | `.github/workflows/` |
| Skill directories | 228 total, 42 `agm-*` domain skills | `.agents/skills/` |
| `tests/` integration dir | fixtures only (4 JSON files), no `.rs` integration tests | `tests/` |

## 3. What is genuinely good (keep these baselines)

1. **Pipeline-first architecture, documented and followed.** Adapters normalize; the pipeline owns thinking blocks, sanitization, and prefix alignment. This is the single best decision in the repo — it localizes protocol churn and makes the 13 thinking/signature invariants (`I1`–`I4`) enforceable in one place.
2. **Strong backend unit-test volume.** 1,036 test attributes across 194 files (~5/file) with near-zero TODO litter signals tests are written alongside code, not bolted on.
3. **Zero-CI-quarantine discipline.** Heavy/OS-destructive tests are kept out of CI by convention (`#[ignore]`, `RUN_TEMP_E2E=1`), and CI compiles test targets without executing them. Pragmatic and correct for a repo that manages live IDE processes.
4. **Release engineering.** Atomic 14-location version bump script, dual-channel gates (`main` = stable, `beta` = preview), branch/tag alignment checks in `release.yml`, README+changelog sync rules. Small teams usually get this wrong; this repo gets it right.
5. **Split-SQLite and envelope conventions.** WAL-mode per-concern databases and the universal `{attributes, data}` JSON envelope with legacy fallback show real operational experience (concurrency, backward compat, portability via `${workDir}` variables).
6. **Headless/CLI parity as a rule, not an aspiration.** Routing GUI and CLI through identical service routes (`copy_instance_with_options`) is enforced by guideline and largely true in practice.
7. **Security posture is explicit.** IP firewall + CIDR + curfew windows + user tokens, AES-256-GCM email vault with machine-GUID salt, Windows ACL hardening for SSH keys. Even if any single control has gaps, the threat model is written down, which is more than most gateways offer.

## 4. What should be improved (ranked by payoff)

### P0 — Split the god-files (modularity 4/10)
- `agm.rs` (14,251 lines, 594 KB) **violates the repo's own 500 KB guard**. `openai.rs` (7,029), `instance.rs` (6,973), `repo_db.rs` (5,911) are in the same class. These files are unreviewable in a single PR, slow to compile-check mentally, and hostile to new contributors and to AI context windows alike.
- **Fix:** extract by command group / handler concern into modules behind the existing `mod` tree. No behavior change; pure moves. Target: no file over ~1,500 lines within two releases. Enforce with a CI file-size check (the guard exists on paper — automate it).

### P1 — Reduce panic surface (~1,600 unwraps)
- `.unwrap()`/`.expect()` count is high for a long-running gateway daemon. Any one of them in a request path is a process abort, not a 500.
- **Fix:** forbid new unwraps in `src-tauri/src/proxy/` and `src-tauri/src/modules/` via `clippy::unwrap_used` (allow in `#[cfg(test)]`), then burn down existing ones starting with request handlers and token/quota paths. The codebase already leans on structured errors elsewhere — extend the pattern.

### P2 — Frontend test coverage (3/10)
- 4 test files for 148 components/pages. `ApiProxy.tsx` (2,850 lines) and `Settings.tsx` (~2,800+ lines implied by 191 KB) are business-logic-heavy pages with no visible safety net.
- **Fix:** extract pure logic (resolvers, mappers, state helpers — `resolve-focus-target.ts` is a good recent example) and unit-test those first; add component tests only for the highest-traffic pages (Accounts, Instances, ApiProxy). Aim for logic-coverage, not snapshot theater.

### P3 — Skill sprawl (228 directories)
- 42 `agm-*` domain skills are justified and genuinely useful. The remaining ~186 (many overlapping: `plan-steps` × N variants, `release*` × 8, `letterly*`, `gitmap*`) create discovery cost and stale-doc risk.
- **Fix:** archive or merge near-duplicates, add one `SKILL-INDEX.md` with "use this, not that" pointers, and lint for skills unreferenced for >2 releases.

### P4 — Small hygiene items
- `docs/` contains both `release-guide.md` and `release_guide.md` — pick one, redirect the other.
- Only 1 `#[ignore]` found in `src-tauri/src` while the quarantine convention implies more heavy tests exist — verify the e2e suite actually runs locally (`RUN_TEMP_E2E=1`) and isn't silently bit-rotting.
- `tests/` holds fixtures but no Rust integration tests; either grow it (gateway route-level tests would pay off fast) or document that inline tests are the deliberate strategy.
- `panic!`-family count (33) is low but each deserves a comment justifying why abort is correct there.

## 5. Baselines — proposed quality bars going forward

| Area | Baseline (new code must meet) | Stretch |
|---|---|---|
| File size | No new file > 1,000 lines; no touched file grows past 1,500 | CI-enforced 500 KB guard for all files |
| Panics | Zero new `.unwrap()`/`.expect()` outside `#[cfg(test)]` in `proxy/` and `modules/` | Burn down to <200 repo-wide |
| Backend tests | Every new handler/command ships with unit tests; touched-file coverage must not drop | Route-level integration tests in `tests/` |
| Frontend tests | New pure-logic modules ship with tests | Top-5 pages covered |
| Docs | One canonical doc per topic; duplicates removed or stub-linked | Quarterly dead-link + stale-skill pass |
| Commits | Keep current discipline: scoped, revertable, `gitmap cpf/cpb/cpr` semantics | Conventional-commit lint in CI |

## 6. How AI can improve this codebase (concrete, ordered)

1. **Mechanical god-file splitting (highest ROI).** An AI agent can split `agm.rs`/`openai.rs`/`instance.rs`/`repo_db.rs` into modules with zero behavior change: move code, fix imports, run `cargo clippy --all-targets --all-features` + `cargo test` per split. This is laborious for humans and ideal for agents — large context, verifiable output, small blast radius per commit.
2. **Unwrap burn-down with tests.** For each unwrap in request paths, generate the error-propagating replacement plus a regression test that triggers the failure branch. Agent does the edit; human reviews the error taxonomy.
3. **Invariant-to-test compiler.** The 13 architectural invariants (`I1`–`I4`, zero-prompt-loss, host-IDE immunity, etc.) are prose today. AI can convert each into property/unit tests (e.g., "thought parts always at `parts[0]`" → proptest over mapper outputs), closing the prose-vs-code gap.
4. **Frontend logic extraction.** Identify pure functions embedded in 2,800-line pages, extract to tested modules, rewire imports. Same shape as (1), smaller files.
5. **Skill/doc deduplication.** Embed all 228 skills, cluster near-duplicates, propose merges with diffs. A human approves; the agent executes.
6. **Release-note and changelog drafting** from `last-tag..HEAD` with the strict `@aukgit`-only attribution rule applied automatically — already partially scripted, easy to finish.
7. **Pre-flight as an agent gate.** `cargo fmt --check` + `cargo clippy --all-targets --all-features` + `npm run build` before every tag is agent-runnable today; wire it as a blocking local hook so humans can't forget.

What AI should **not** do here: redesign the pipeline/adapter split (it's correct), invent new protocols or storage engines, or "fix" the quarantine convention by running destructive tests in CI.

## 7. Methodology and limits (read before quoting this review)

- File/line/test/unwrap counts were measured directly from the tree on 2026-10-06 (commands: `Get-ChildItem` + `Select-String` + `Measure-Object`). Re-run them; they take seconds.
- Architectural claims were cross-checked against `AGENTS.md`, `.agents/skills/agm-core-architecture/SKILL.md`, `src-tauri/src/proxy/server.rs` routes, and the prior full-codebase workflow synthesis (9 agents, 230 tool calls).
- **Not run:** `cargo fmt --check`, `cargo clippy`, `npm run build`, `cargo test` — `cargo` is not on PATH in this environment, and the TypeScript toolchain was not exercised. Treat the "tests are strong" claim as volume-based, not execution-verified.
- **Not inspected line-by-line:** Supabase/SSH/email/telegram internals, upstream endpoint URLs, OAuth secrets (intentionally excluded).
- Opinion scores are the reviewer's judgment, not measurements. Disagree with specifics, not with vibes — the numbers above are the common ground.
