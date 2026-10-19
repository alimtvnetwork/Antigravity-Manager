# Subtask 05: Auto-Switcher and Audit Log Analysis

## Objective
Analyze the auto-switching step progression, candidate scoring algorithms, audit log records in `task_index.db:hot_tasks_cache`, and prompt restoration statuses.

## Execution Steps
1. Document the auto-switcher decision pipeline:
   - Rolling 5h quota vs 7-day weekly quota calculations.
   - Multiplicative scoring formula: $(S_{active} \times M_{tier} \times Q_{weekly}) / 1000$.
   - Bound account exclusion on sibling instances.
2. Query and inspect the audit log in `task_index.db`:
   - Query `hot_tasks_cache` for `action = 3` (SwitchAccount).
   - Verify timestamp, instance ID, account email, and transition details.
3. Validate prompt status in `repo_prompts.db:active_prompts`:
   - Trace status progression: `running` -> `backed_up` -> `dispatched`.
4. Summarize step execution in the audit report.
