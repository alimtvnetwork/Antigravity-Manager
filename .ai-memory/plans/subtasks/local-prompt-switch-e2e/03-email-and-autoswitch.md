# 03 Email vault and auto-switch

Status: `COMPLETED`

Spec: `02-spec/21-app/local-cli-e2e/01-retest-cases.md` cases E1, B1.

`agm email ls` shows one account and one recipient and does not print a password. `agm auto-switch threshold 98` then `agm auto-switch threshold 15`. End on `15.0%`.
