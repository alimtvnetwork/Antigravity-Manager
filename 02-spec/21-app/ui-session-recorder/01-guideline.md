# UI session recorder guideline

Slug: `ui-session-recorder`

This file is the implementation contract. Do not build the recorder from memory. Follow the rules below. The current code does not record a session. A later change implements it. This document does not add that code.

## What exists today

- `src/stores/error-store.ts` keeps the last 10 clicks in memory (`recordClickEvent`, `getRecentClicks`). `captureError` copies that ring onto `CapturedError.uiClickPath` when an error is captured.
- `src/lib/error-listener.ts` calls `recordClickEvent` from the document click listener.
- `src/lib/error-report-generator.ts` prints `uiClickPathArrow` into an error report.
- `src/lib/stack-frame-parser.ts` parses one V8 line (`at …`) into `{ function, file, line, column, isInternal }` via `parseStackLine`.
- `src-tauri/src/modules/process/launch.rs` sets `RUST_BACKTRACE=1` on child IDE processes. That is not this recorder. A production recording must not turn that variable on.

Nothing records the methods that run between clicks. Nothing is gated by a developer mode.

## Modes

- Developer mode is on for the current builds. A recording then stores function name, file, and line for every audited call.
- A production recording still stores the function name. It does not store a file path or a line number.
- Recording and stack capture run only when the user has started a recording, or when an error is captured while developer mode is on. A normal production click does not write a trace.
- One flag decides the line policy. `developerMode === true` keeps `file` and `line`. `developerMode === false` omits both fields even if a stack string contains them. Strip them before the record is saved. Do not rely on a minified stack happening to lack lines.

## What one recording is

A session is a JSON document an AI can read without the live UI.

- `sessionId`, `startedAt`, `stoppedAt`, `developerMode`, `appVersion`, `route` at start.
- Ordered `steps`. Each step is one of `click`, `invoke`, `result`, `error`.
- A click step has a stable target (`data-audit` value, else role plus accessible name), the route, and the time. It does not store typed secrets, prompt bodies, or tokens.
- An invoke step names the Tauri command or the frontend function. In developer mode it also has `file` and `line`. In production it has `function` only.
- A result step says ok or error and the elapsed time. An error step attaches the same fields `CapturedError` already has (`function`, and `file` plus `line` only in developer mode) plus the step ids that led to it.
- Cap the session at 500 steps or 2 MB. Drop the oldest step and record one `truncated` marker. Do not grow without a bound.

```json
{
  "sessionId": "string",
  "startedAt": "ISO-8601",
  "stoppedAt": "ISO-8601 or null while recording",
  "developerMode": true,
  "appVersion": "string",
  "route": "string",
  "steps": []
}
```

Step fields:

| kind | required | developer mode only |
| :--- | :--- | :--- |
| `click` | `id`, `at`, `target`, `route` | none |
| `invoke` | `id`, `at`, `function`, `afterClickId` | `file`, `line` |
| `result` | `id`, `at`, `invokeId`, `ok`, `elapsedMs` | none |
| `error` | `id`, `at`, `function`, `ledBy` (step ids) | `file`, `line` |
| `truncated` | `id`, `at`, `dropped` | none |

`function` is the command name (`switch_account_to_instance`) or the frontend function name. It is never empty on `invoke` and `error`.

## Sample: developer mode

```json
{
  "sessionId": "rec-dev-001",
  "startedAt": "2026-10-14T13:00:00Z",
  "stoppedAt": "2026-10-14T13:00:04Z",
  "developerMode": true,
  "appVersion": "4.198.0",
  "route": "/accounts",
  "steps": [
    {
      "kind": "click",
      "id": "s1",
      "at": "2026-10-14T13:00:01Z",
      "target": "accounts.switch",
      "route": "/accounts"
    },
    {
      "kind": "invoke",
      "id": "s2",
      "at": "2026-10-14T13:00:01Z",
      "function": "switch_account_to_instance",
      "afterClickId": "s1",
      "file": "src-tauri/src/modules/instance/switch.rs",
      "line": 349
    },
    {
      "kind": "error",
      "id": "s3",
      "at": "2026-10-14T13:00:04Z",
      "function": "verify_instance_relaunched",
      "file": "src-tauri/src/modules/instance/observe.rs",
      "line": 165,
      "ledBy": ["s1", "s2"]
    }
  ]
}
```

## Sample: production

The same click and the same failure. `function` stays. `file` and `line` are absent. A missing line is not a missing function. Search the repo for `verify_instance_relaunched`.

```json
{
  "sessionId": "rec-prod-001",
  "startedAt": "2026-10-14T13:00:00Z",
  "stoppedAt": "2026-10-14T13:00:04Z",
  "developerMode": false,
  "appVersion": "4.198.0",
  "route": "/accounts",
  "steps": [
    {
      "kind": "click",
      "id": "s1",
      "at": "2026-10-14T13:00:01Z",
      "target": "accounts.switch",
      "route": "/accounts"
    },
    {
      "kind": "invoke",
      "id": "s2",
      "at": "2026-10-14T13:00:01Z",
      "function": "switch_account_to_instance",
      "afterClickId": "s1"
    },
    {
      "kind": "error",
      "id": "s3",
      "at": "2026-10-14T13:00:04Z",
      "function": "verify_instance_relaunched",
      "ledBy": ["s1", "s2"]
    }
  ]
}
```

## How an AI uses a session

Walk `steps` in order. Find the last `error` step. Read `function`. In a developer-mode file, open `file` at `line`. In a production file, search the repo for that function name. The click steps before the error are the reproduction. Do not treat a missing line number in production as a missing function.

## Where a later implementation plugs in

- Keep using `recordClickEvent` for the error modal. Add a session sink beside it so a recording and an error share one click shape.
- Wrap `invoke` in one helper. Do not add a manual log line in every command. The wrapper records the command name, start, and result. Developer-mode frames come from `new Error().stack`, parsed by `parseStackLine`, then filtered. Drop frames where `isInternal` is true.
- Rust commands that fail while a recording is active return `AppError` as they do now. The frontend records the command name. A Rust frame list is included only for that failed command, with `file` and `line` removed unless developer mode is on.
- Export is a JSON file the user saves. The error report generator can append the session id. Do not write prompt text, passwords, or API keys into the file.

## Later implementation order

1. Add `developerMode` (default true until a production switch exists) and a Start / Stop recording control on the existing error or settings surface.
2. Extend the click listener and one `invoke` wrapper to append steps only while recording.
3. Strip `file` and `line` when `developerMode` is false, including frames parsed from a stack string.
4. Save the session JSON and add a fixture test. The production fixture has `function` and no `line`. The developer fixture has `function`, `file`, and `line`.
5. Do not set `RUST_BACKTRACE=1` for a production recording.

## Fixture the test must lock

A pure function `framesForMode(frames, developerMode)` returns the stored frames.

- Input frame: `{ "function": "verify_instance_relaunched", "file": "src-tauri/src/modules/instance/observe.rs", "line": 165 }`.
- `developerMode: true` keeps all three fields.
- `developerMode: false` keeps `function` and deletes `file` and `line`.
- An empty `function` is a test failure in both modes.
