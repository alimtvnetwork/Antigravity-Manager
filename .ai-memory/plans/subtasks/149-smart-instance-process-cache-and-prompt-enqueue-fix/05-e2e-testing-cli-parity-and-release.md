# Subtask 05: Host-Shielded E2E Testing, CLI Parity & Release Ceremony

- **Parent Plan**: `.ai-memory/plans/149-smart-instance-process-cache-and-prompt-enqueue-fix.md`
- **Owner**: Lead Orchestrator
- **Status**: `[QUEUED]`

---

## 1. Objectives

1. **Host-Shielded E2E Verification Harness**:
   - Update `03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py` to test all 5 core capabilities:
     * `TC01`: Cache Hit (No Relaunch Verification) — preserves living PID and prevents duplicate process launches.
     * `TC02`: Process Death Check & Cold Launch Path — invalidates stale cache, re-scans OS process table, and triggers launch only when confirmed 100% dead.
     * `TC03`: Strict FIFO Queue Storage & Ordering — verifies `enqueue_prompt` persists to `active_prompts` with status `queued` in FIFO order (`ORDER BY created_at ASC, id ASC`) and writes `.antigravity_resume_task.json`.
     * `TC04`: False-Positive Running Prompt Elimination — verifies that stale prompts and idle conversations never report as `running`.
     * `TC05`: PromptTreeViewModal UI Compaction & Segmented Actions — verifies absence of bracket clutter `[...]` and clean badge presentation.
   - Maintain zero-disruption host PID shielding across all runs.

2. **Pre-flight Quality Gates**:
   - `cd src-tauri && cargo fmt -- --check`
   - `npm run build`
   - `python 03-ai-scripts/45-prompt-dispatch-process-cache-e2e.py`

3. **Minor Version Bump & Release Ceremony**:
   - Bump minor version to `4.168.0` (`npm run bump minor`).
   - Synchronize changelogs (`CHANGELOG.md` and `CHANGELOG_EN.md`) strictly attributing `@aukgit` (`(Thanks to @aukgit)`).
   - Synchronize `README.md` and `README_EN.md` under `## 📝 Changelog`.
   - Execute atomic GitMap commit and push: `gitmap cpf "chore(release): bump version to 4.168.0 and update changelog"`.
   - Create tag `v4.168.0` and push to remote.
   - Verify pipeline status with `gitmap pe`.
