# Subtask 005: Weekly Quota Capture & Display in Instance Mode

## Owner: Worker 02 (Frontend Specialist)
## Target Files: `src/pages/Instances.tsx`, `src/components/instances/InstanceTable.tsx`

### Requirements
1. **Weekly Quota Capture in Instance Cards**:
   - In `src/pages/Instances.tsx`, resolve `boundAccount?.quota?.weekly` alongside `geminiModel`.
   - Render the weekly quota progress bar directly below the 4-hour model quota bar inside each instance card.
2. **Weekly Quota in Instance Table**:
   - In `src/components/instances/InstanceTable.tsx`, render the weekly quota bar inside the Quota column.
3. **Consistent Styling**:
   - Reuse the updated `QuotaProgressBar` component with the new resilient height, equal width, and orange-to-red color progression.
