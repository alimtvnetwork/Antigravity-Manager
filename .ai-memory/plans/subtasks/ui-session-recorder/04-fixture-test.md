# 04 Fixture test

Status: `PENDING`

Spec: `02-spec/21-app/ui-session-recorder/01-guideline.md`

Add `framesForMode(frames, developerMode)`.

- Input: `{ "function": "verify_instance_relaunched", "file": "src-tauri/src/modules/instance/observe.rs", "line": 165 }`.
- `developerMode: true` keeps `function`, `file`, and `line`.
- `developerMode: false` keeps `function` and has no `file` and no `line`.
- An empty `function` fails in both modes.

The production sample and the developer sample in the spec are the two fixtures. A missing line in production is not a missing function.
