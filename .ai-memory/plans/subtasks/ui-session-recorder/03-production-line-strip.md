# 03 Production line strip

Status: `PENDING`

Spec: `02-spec/21-app/ui-session-recorder/01-guideline.md`

Parse developer-mode frames with `parseStackLine` in `src/lib/stack-frame-parser.ts`. Drop frames where `isInternal` is true. When `developerMode` is false, delete `file` and `line` before the step is saved, including frames that came from a stack string. Keep `function`. A Rust `AppError` from a failed command is recorded by the frontend. Include a Rust frame list only for that failed command, and strip `file` and `line` unless developer mode is on. Do not set `RUST_BACKTRACE=1` for a production recording.
