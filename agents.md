# Project Maintenance Guidelines

- **Architecture**: This project is a gateway that aggregates four AI protocols — OpenAI Responses, OpenAI Chat Completions, Anthropic Claude, and Google Gemini — and outputs Antigravity-style Gemini protocol format.
- **Pipeline First**: Keep the pipeline strictly decoupled from specific protocols. The four protocols function purely as Gemini adapters. Adapters are restricted to parameter normalization, payload transformation, protocol divergence adaptation, and edge cases unsolvable within the pipeline stage. The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers.
- **Backend Fix Strategy**: Prioritize protocol-agnostic, generic fixes within the pipeline rather than localized adapter modifications. Treat adapter-level patches as a last resort only when a generic pipeline solution is infeasible or degrades compatibility.
- **UI Design & Headless Compatibility**:
  - **Minimalist & Contextual UI**: Prioritize user-friendly, non-intrusive interactions. Reuse existing design conventions (e.g., pill toggle buttons, badge switches, or contextual setting panels) placed strictly within their most relevant sections rather than scattering unrelated controls. When rendering adjacent toolbar actions or desktop window controls (minimize, maximize, close), always wrap them into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, and dark-glass styling) rather than loose, disjointed circular buttons. Any inline `<select>` dropdowns mounted within segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to eliminate nested white pill-in-pill visual defects.
  - **4-Tier Prompt Origin Classification & Hierarchy**: Prompt tree views and inspectors must discriminate among 4 distinct origin tiers: `USER_PROMPT` (Sky badge, User icon), `SUBAGENT_INSTRUCTION` (Purple badge, Bot icon), `SYSTEM_MESSAGE` (Zinc badge, Terminal icon), and `TOOL_OUTPUT` (Amber badge, Wrench icon). Subagents must nest hierarchically beneath their parent user prompt with branch glyphs (`↳`, `└──`).
  - **Interactive Truncation Callouts**: Never render raw `<truncated N bytes>` or `<truncated N lines>` markers as plain text; parse and render them as interactive dark-glass omission banners (`⚡ [Omitted N bytes/lines of context - Click to inspect/expand]`).
  - **Headless & CLI Parity**: Ensure GUI configurations and lifecycle operations maintain functional parity across headless servers, CLI environments, and cross-platform environments. Any profile/instance duplication or settings synchronization must pass through identical underlying service routes (`copy_instance_with_options`), preserving project continuity (`workspaceStorage`) and allowing deep-merge settings synchronization from both GUI and CLI.
  - **Cross-Platform Compatibility**: Evaluate every code addition and dependency change for seamless cross-platform support.
- **Backend Fix Strategy & Prompt Engine**:
  - **FIFO Restore & Queue Invariant**: All prompt restore, dispatch, and queue processing queries across SQLite databases (`active_prompts`, `prompt_backups`, `repo_prompts.db`) MUST enforce strict First-In-First-Out ordering using `ORDER BY created_at ASC, id ASC`. Inverted LIFO queries (`ORDER BY updated_at DESC`) are strictly forbidden in dispatch logic.
  - **Deep Transcript Inspection & Full Log Precedence**: When inspecting conversation transcripts, always prioritize un-truncated `transcript_full.jsonl` over `transcript.jsonl`. Reverse-scan across at least 10 lines, bypassing trailing `TOKEN_USAGE`, `HEARTBEAT`, and `TELEMETRY` records, to extract genuine tool executions (`tool_calls`) and assistant thoughts (`thinking`).

- **Code Quality**: Prioritize root-cause, future-proof fixes rather than hardcoded logic, dead code, or speculative changes. Prioritize generalized solutions that cover entire classes of problems rather than one-off patches.
  - *Model Routing Example*: Prioritize wildcard patterns (e.g., `gemini-*-flash-*`) to anticipate future model releases rather than exact string matches.
  - *Prompt Sanitization Example*: For agent-client prompt sanitization, prioritize regex-based pattern matching over static keyword replacement, ensuring full coverage without stripping pipeline system prompts or user queries.
- **Formatting & CI Discipline**:
  - **Unit Testing**: Keep focused — run targeted tests for touched modules locally; CI compiles test targets without executing them. Skip tests for trivial edits (constants, prompts, or config tweaks). No need to run the full suite locally.
  - **Pre-flight Checks**: Run the essentials before submitting PRs or release tags:
    - `cd src-tauri && cargo fmt -- --check` (for Rust edits)
    - `cd src-tauri && cargo clippy --all-targets --all-features` (comprehensive Rust gate, already includes compilation — no separate `cargo check` needed)
    - `npm run build` (when `src/` or frontend configs changed)
    - Rely on CI for full-app compilation (`tauri build`) and full test execution. Local pre-flight covers fmt + clippy + frontend build only.
- **Release Channels & Discipline**:
  - **Release Channel Separation**:
    - **Stable Releases (正式版)**: Exclusively on `main`. Deploys official production packages, updates Docker/GitHub `latest` tags, and services automatic update channels.
    - **Preview Releases (预览版 / Beta)**: Exclusively on `beta`. Independently builds and publishes pre-releases (`makeLatest: false`, `prerelease: true`) without touching production update channels.
  - **Maintainer Staging Protocol**:
    - When introducing new changes (features, major refactors, non-trivial fixes), prompt and confirm with maintainers whether to implement and test on `beta` branch first.
    - Validate stability on `beta` (with optional independent preview builds) prior to merging into `main`.
  - **Standard Release Workflow**:
    1. **Atomic Version Sync**: Run `npm run bump <patch|minor|beta|version>` to synchronize all project manifests and generate changelog skeletons.
    2. **Documentation & Attribution**:
       - Audit Git history (`<last-tag>..HEAD`). Attribution must attribute strictly `@aukgit` (`(Thanks to @aukgit)`) in `changelog.md` and `changelog_en.md`. Do NOT include any other GitHub user handles (`@...`) in release changelogs or release notes, ensuring the GitHub release page contributors list only contains `aukgit` and none else.
       - **Synchronize README Changelog (同步首页更新日志)**: For stable releases, you **MUST** update the release summary in both `README.md` (under "## 📝 更新日志") and `README_EN.md` (under "## 📝 Changelog"). Never update only `CHANGELOG.md` while leaving `README.md` / `README_EN.md` with outdated release notes. Pre-release / beta versions remain exclusively in changelogs; stable releases require full synchronization across both README files.
    3. **Pre-flight before Tagging**: Run the Pre-flight Checks above on the exact commit to be tagged.
    4. **Commit, Tag & Push**: Push stable releases to `main` (`git tag vX.Y.Z && git push origin vX.Y.Z`), reserving `beta` exclusively for pre-releases (`git tag vX.Y.Z-beta.N && git push origin vX.Y.Z-beta.N`). The release gate strictly intercepts cross-branch misplacement. Tags must match `CHANGELOG.md` headings character-for-character (including `v` prefix and pre-release suffix).
  - *Full procedure*: See `docs/release_guide.md` for bump options, changelog templates, and rollback steps.
- **Thinking Cache Invalidation Control (发版清理建议)**:
  - File: `src/components/common/SuggestionDeleteThinkingModal.tsx`
  - Routine releases (no prompt): Keep `SUGGESTION_DELETE_THINKING_STORE = false`.
  - Architecture / schema refactors (prompt users to clean once):
    1. Set `SUGGESTION_DELETE_THINKING_STORE = true`.
    2. Set `SUGGESTION_TARGET_VERSION = '<version>'` (e.g. `'4.8.2'`).
    3. On upgrade, users with existing cache get a one-time prompt; action state persists in `gui_config.json`.
- **Branch & History Hygiene**:
  - Branch from remote bases (`origin/beta` for staged features, `origin/main` for direct hotfixes) rather than local branches to prevent untracked ancestor commits.
  - Inspect in-flight PRs (`gh pr list --base main`) before rewriting published tips, avoiding force-pushes across shared branches.
  - Retain local rollback refs (`backup/*`) before history rewrites, confirming zero content drift via `git diff --stat <backup> HEAD`.
- **PR Scope & Grouping**:
  - **Single Problem Scope**: A PR represents a cohesive collection of fixes or features dedicated to a single problem class. Keep unrelated concerns (such as governance, release tooling, or documentation) in isolated PRs.
  - **Self-Contained & Individually Revertable**: A PR may contain multiple commits, but each commit must represent an independent, self-contained functional unit that is individually revertable, avoiding messy or tangled changesets.
  - **Local Convergence & Final-State Commits**: Commit freely during local debugging on development branches; however, before opening or merging a PR, audit and consolidate scattered iterative attempts into clean, high-quality units. Each consolidated commit must describe only its successful final state and rationale, eliminating intermediate trial-and-error noise.
  - **Review & Template Alignment**: Route every PR through peer review and complete `.github/pull_request_template.md` (problem classification, behavior alterations, unverified paths, and rollback strategy).
- **Contributor Respect & Attribution**:
  - Preserve authorship by preferring the contributor's own PR for squash commits, or attaching explicit `Co-authored-by:` trailers on merge commits and proxy PRs.
  - Disclose costs before merging: highlight affected existing behaviors and unverified paths alongside improvements.
  - Own tooling and documentation gaps directly rather than attributing downstream frictions to contributors.
  - Accompany every rejection with a concrete file-by-file accept/drop breakdown and an actionable path forward.
- **Risky Path Replacement**:
  - Keep an explicit fallback to the verified path whenever you replace it (e.g. "0 matches found → full restart"). A silent no-op is worse than the failure it was meant to fix.
  - Order side effects to fail before the point of no return: write credentials before killing a process, validate before deleting.
  - Detect broadly, act narrowly: a matcher may recognize a whole class of problems, while its effect stays inside the intended data — not across line breaks, tags, or other clauses. Bound every wait with a timeout.
- **No Silent Errors** (house rule, 2026-10-10): every fallible operation must route its failure into the error module. Swallowing an error — discarding it where no one can ever see it — is forbidden.
  - **Rust** (`src-tauri/src/`): errors flow into `AppError` (`src-tauri/src/error.rs`) and are logged via `tracing` (file + console + Tauri log bridge). Never leave a bare `let _ = <fallible>`, `.ok()`, or `.unwrap_or_default()` on a `Result` without routing the `Err`:
    - FORBIDDEN: `let _ = std::fs::write(&path, data);`
    - REQUIRED: propagate with context — `std::fs::write(&path, data).map_err(|e| AppError::Io(format!("write config {path:?}: {e}")))?;`
    - For genuinely best-effort ops (cleanup, best-effort notifies): `crate::error::record_ignored(std::fs::remove_file(&tmp), "remove temp file")` **plus** a one-line `// Justification:` comment explaining why it is safe to continue. Both the tracking call and the comment are required — neither alone suffices.
  - **TypeScript** (`src/`): failures go through the error store (`src/stores/error-store.ts`). Use `captureError`/`captureException` for user-facing failures (opens the error modal), `trackWarning` for benign/best-effort failures (records into error history without a modal — never a bare `catch {}`):
    - FORBIDDEN: `await invoke('save_x').catch(() => {});` or `catch {}` / `catch (e) { console.warn(e); }`
    - REQUIRED: `catch (e) { useErrorStore.getState().trackWarning(e, { source: 'MyComp.save', triggerAction: 'save_x' }); /* fallback */ }`
  - Success paths must never change as part of error-visibility work: failures become visible, successes behave exactly as before. No fake success — a failed op must not report success to the user.

Maintained by @aukgit