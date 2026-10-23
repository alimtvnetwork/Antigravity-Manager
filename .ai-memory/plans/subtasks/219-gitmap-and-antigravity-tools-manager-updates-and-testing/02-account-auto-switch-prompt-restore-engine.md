# Subtask 219-02: Account Auto-Switching & Prompt Restore Resiliency
Task ID: `Imp-721`
Sequence: 219
Step: 307
Status: IN_PROGRESS

## Scope
- Focus on prompt restore reliability during auto-switch and quota exhaustion.
- Transition all uncompleted prompts (`running`, `dispatched`, `in_flight`, `pending`) to `backed_up` without arbitrary time windows.
- Re-attribute unrestored prompts to the target instance during switch.
- Return honest `RestoreOutcome` and enforce unscoped fallback when scoped restore yields 0 records.
