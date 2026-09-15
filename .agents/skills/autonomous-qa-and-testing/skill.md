---
name: autonomous-qa-and-testing
description: Autonomously run test suites, verify quality gates, and prevent regressions across polyglot stacks.
---

# Autonomous QA and Testing

Executes comprehensive testing, linting verification, and quality gate validation.

## Checks
1. **TypeScript / React:** `npm run test`, `npm run lint`
2. **Go:** `go test ./...`
3. **Python Linters:** `python 03-ai-scripts/06-cicd-local-runner.py`
4. **Git Hygiene:** Verify no un-ignored test dumps or binaries via `git status`.
