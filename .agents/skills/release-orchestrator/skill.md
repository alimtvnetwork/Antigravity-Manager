---
name: release-orchestrator
description: >-
  Execute full automated release orchestration, semantic version bumping, branch management, and tag creation using Python scripts.
---

# Automated Release Orchestrator

Execute full automated release orchestration, semantic version bumping, branch management, and tag creation using Python scripts.

## Core Directives
1. Determine bump tier (MINOR default, reset PATCH to 0).
2. Use `03-ai-scripts/29-release-orchestrator.py` to coordinate version updates across packages and changelog.
3. Verify git clean status before release execution.
