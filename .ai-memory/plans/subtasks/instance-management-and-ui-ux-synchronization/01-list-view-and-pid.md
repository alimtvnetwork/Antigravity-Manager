# Subtask 01: List View Single-Line Path Tails & Explicit PID Badge

## Scope
- Files: `src/components/instances/InstanceRow.tsx`, `src/components/instances/InstanceCard.tsx`

## Implementation Steps
1. In `InstanceRow.tsx`:
   - Inspect status pill rendering. When `instance.is_running` is true and `instance.pid` is present, display `PID: ${instance.pid}` explicitly with green pulse dot.
   - Format executable path ending and data path ending inside a single flex container on the same line:
     `<span className="truncate">{exeTail}</span><span className="text-slate-400">·</span><span className="truncate">{dataTail}</span>`
2. In `InstanceCard.tsx`:
   - Confirm consistent `PID: ${instance.pid}` badge formatting.

## Verification
- Inspect JSX structure; verify single-line layout and PID label.
