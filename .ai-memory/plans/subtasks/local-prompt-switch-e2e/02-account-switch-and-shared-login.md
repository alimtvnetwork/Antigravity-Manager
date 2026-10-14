# 02 Account switch and shared login

Status: `COMPLETED`

Spec: `02-spec/21-app/local-cli-e2e/01-retest-cases.md` cases S1, S2.

Switch only the idle copy (`#2`). Switch it back to the account it had at the start, then `agm instances stop #2`. Default's PID and bound account stay put. `~/.gemini/google_accounts.json` field `active` stays the Default account. A copy launch must not write the shared keyring.
