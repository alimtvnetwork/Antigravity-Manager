# Prompt Index

**Version:** 3.8.0
**Updated:** 2026-04-27

---

## Purpose

Central index of all AI prompts available in this project. Reference this file to discover which prompts exist and when to use them.

---

## Available Prompts

| # | File | Trigger | Purpose |
|---|------|---------|---------|
| 01 | [01-read-prompt.md](prompts/01-read-prompt.md) | "read memory" | **Canonical v1.0.0** — Full AI onboarding (Phases 1–4 + CI/CD lessons). Self-contained protocol. |
| 02 | [02-write-prompt.md](prompts/02-write-prompt.md) | "write memory" / "end memory" *(legacy v3.3 protocol)* | Earlier end-of-session memory persistence prompt — kept for reference |
| 03 | [03-write-prompt.md](prompts/03-write-prompt.md) | "write memory" / "end memory" / "update memory" | **Canonical** end-of-session memory persistence protocol (v1.0.0, 2026-04-19) |
| 04 | [04-write-prompt.md](prompts/04-write-prompt.md) | "write memory" / "end memory" / "update memory" | **Canonical v2.0.0** — adds CI/CD issues tracking + cicd-index, supersedes 03 (2026-04-23) |
| 05 | [05-write-prompt.md](prompts/05-write-prompt.md) | "write memory" / "end memory" / "update memory" | **Canonical v3.0.0** — adds avoid/ memory, recent-spec verbatim rule, lowercase-hyphen mandate, restructure check, supersedes 04 (2026-04-27) |

---

## Adding New Prompts

1. Create the file in `01-prompts/` with numeric prefix: `NN-descriptive-name.md`
2. Add an entry to this index table
3. Bump the version of this file

---

*Prompt index — v3.8.0 — 2026-04-27*
