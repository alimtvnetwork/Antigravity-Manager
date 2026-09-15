---
name: no-time-suggestions-in-readme-txt
description: Hard prohibition — never suggest, recommend, mention, or auto-add timestamps, time-update workflows, or git-commit-time tips inside readme.txt or in chat about readme.txt. Sequence 02 in strictly-avoid.
type: constraint
---

# Avoid 02 — No time-related suggestions for `readme.txt`

**Sequence:** 02 (strictly-avoid)
**Status:** 🔴 Hard prohibition — perpetual. Once read, keep in working memory forever; never re-propose.

## The rule

1. **Never** suggest adding, updating, refreshing, or automating a timestamp inside `readme.txt`.
2. **Never** mention "git update time", commit time, last-modified time, or any time-flavoured metadata anywhere in `readme.txt` or in chat copy that talks about `readme.txt`.
3. **Never** propose scripts, hooks, CI jobs, or cron-like flows whose purpose is to keep `readme.txt` time fresh.
4. **Only** write/update `readme.txt` when the user explicitly asks for it in that turn. Use the format the user dictates; do not editorialise.
5. Do not offer "would you like me to also update the time?" follow-ups. Silence on this topic is mandatory.

## Why

The user has explicitly and repeatedly forbidden time-related suggestions
around `readme.txt`. Repeating the suggestion is a trust violation and
wastes their attention. This file exists so the prohibition survives
across sessions.

## How to apply

- On every task that touches `readme.txt`, re-read this file silently and obey.
- If a workflow seems to need a time refresh, **do nothing and say nothing** unless the user has asked in that exact turn.
- Cross-reference: `.lovable/strictly-avoid.md` → "readme.txt — time suggestions" and `02-spec/01-spec-authoring-guide/11-exceptions.md` → Strictly Prohibited section.

## Memory directive

Once an AI agent reads this file, it MUST treat the rule as a permanent
entry in its strictly-avoid working set for the remainder of every
session, never re-surfacing the prohibited suggestions.
