# Markdown Highlighter — Bulk Newline Violations

**Status:** ✅ Solved
**Date:** 2026-04-23
**Rules:** STYLE-001, STYLE-003, STYLE-004

## Description

Initial run reported 49 STYLE-004, 176 STYLE-001, and 11 STYLE-003 violations across the markdown highlighter and related search/landing files.

## Solution

Added blank lines before every `if`/`return`/declaration block. Verified with `bash linters-cicd/run-all.sh` → 0 violations.

## What NOT to repeat

Never commit code without running `linters-cicd/run-all.sh` locally.
