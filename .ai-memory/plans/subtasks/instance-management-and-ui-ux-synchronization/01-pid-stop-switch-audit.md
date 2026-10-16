# PID label, verified stop, switch dialog, audit fields

- Status badge text is `PID <n>`.
- Path cells keep the filename ending.
- `close_instance` kills the instance executable PID and errors if it survives.
- Switch aborts when close is not confirmed.
- Audit payload includes `recent_switches`.
- `agm instances current` protects the caller instance.
