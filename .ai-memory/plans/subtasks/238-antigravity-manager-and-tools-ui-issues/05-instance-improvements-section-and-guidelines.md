# Subtask 05: Instance Improvements Section and Coding Guidelines Integration

- **Parent Plan**: `.ai-memory/plans/pending/238-antigravity-manager-and-tools-ui-issues.md`
- **Slug**: `antigravity-manager-and-tools-ui-issues - Task 05`
- **Status**: PENDING

## Scope
1. Move `ImprovementsFlywheel` from global page `src/pages/Instances.tsx` to inside `src/pages/instances/InstanceCard.tsx`.
2. Provide user options for task numbers to enqueue:
   - Dropdown options: `5, 10, 20, 50` (Default: `20`).
3. Wire automatic enqueue of 20 improvement requests directly to the IDE queue using prompts from `01-prompts/30-improvements/` and `01-prompts/17-coding-guidelines/`.
4. Update `src-tauri/src/modules/improvements/` to accept instance-scoped execution and target batch sizes.
