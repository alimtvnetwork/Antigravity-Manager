# Subtask 06: Standalone JSON Import/Export & Overlay Isolation

Parent Task: `ui-and-functionality-enhancement-instructions`
Task-ID: `Task-08`
Owner: Worker 03

## Objectives
1. Provide dual export format: Standalone JSON (with base64 embedded images, fully portable) vs Lean JSON (with relative paths).
2. Fix import/export modal overlay: ensure backdrop clicks stop propagation and do not trap mouse events (`e.stopPropagation()`).
3. Set import/export modals to `z-[100050]`.
