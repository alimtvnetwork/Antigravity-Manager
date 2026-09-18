---
name: readme.txt generator — forbidden suggestions
description: Never propose configurable 3-word message, idempotency variants, or local-time format for the readme.txt generator. Hard-coded only.
type: constraint
---
When working on the `readme.txt` refresher / generator feature:

**Forbidden suggestions — never offer these as options, never recommend, never ask about:**
- Configurable 3-word message / `--message` flag / curated random list. The prefix is hard-coded `let's start now`. Period.
- Any "idempotency" / "skip if same day" / "write only if missing" variants. The script ALWAYS rewrites with a fresh timestamp on every invocation.
- Local-machine time, "Malaysia-style format only", or any timezone other than Asia/Kuala_Lumpur. Timestamp is ALWAYS computed in `Asia/Kuala_Lumpur` (UTC+08:00).

**Locked behavior (do not re-litigate):**
- Prefix: literal `let's start now`
- Timestamp: `Asia/Kuala_Lumpur`, format `dd-MMM-yyyy hh:mm:ss tt` (e.g. `27-Apr-2026 05:04:39 AM`)
- Behavior: unconditional overwrite on every run
- Target file: `readme.txt` at repo root

**Why:** User explicitly rejected these branches and instructed to never suggest them again. Re-asking wastes their time.

**How to apply:** Skip these questions entirely on any future readme.txt work. Implement directly with the locked behavior above.
