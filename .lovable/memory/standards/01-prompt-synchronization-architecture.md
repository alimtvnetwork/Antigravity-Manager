---
name: Prompt Synchronization Architecture
description: Internal, self-contained prompt categorization in 01-prompts/ compiled to top-level 01-prompts/ via prompt-sync-config.json.
type: standard
---

# Prompt Synchronization Architecture

**Authoritative Source:** `01-prompts/`
**Compiled Output:** `01-prompts/*.md` (flat prompt files)
**Configuration:** `scripts/prompt-sync-config.json`
**Execution Script:** `scripts/update-prompts.ps1` / `scripts/update-prompts.sh`

## 1. Internal, Self-Contained Source of Truth

To eliminate external repository sync overhead, all system and AI execution prompts are authored, versioned, and edited exclusively inside this repository under:
`01-prompts/`

Categorized folders include:
- `00-folder-structure/`
- `01-prompt-library-setup/`
- `02-core-workflow/`
- `03-read-write/`
- `04-coding-standards/`
- `05-coding-guidelines/`
- `06-testing-and-qa/`
- `07-bug-fix/`
- `08-dry-code/`
- `09-commit-and-multi-agent-code-fix/`
- `10-ui-and-design/`
- `11-content-and-seo/`
- `12-old-plan-prompts/`
- `13-plan-audit/`
- `14-execute/`
- `15-cg-execute/`
- `16-prompt-engineering/`
- `17-ci-cd/`
- `18-release-management/`
- `19-insults/`
- `20-memory-consolidate/`
- `21-old-execute-prompts/`
- `22-ai-fix-script-prompts/`

## 2. Compilation and Flat Mirroring

Top-level AI tools and command shortcuts read from flat, sequentially numbered prompts in `01-prompts/` (e.g. `01-unified-ai-prompt-v4.md`, `02-next-steps.md`, `05-read-memory-enhanced.md`, etc.).

### Synchronization Command:
```powershell
.\scripts\update-prompts.ps1
```
or on Linux/macOS:
```bash
./scripts/update-prompts.sh
```

### Config Variable Substitution:
`scripts/prompt-sync-config.json` defines folder prefix variables (`SOURCE_PREFIX`, `TARGET_PREFIX`) that are compiled at execution time before copying files.

## 3. Strict Rules for AI Agents

1. **Modify the Source First:** Never edit `01-prompts/*.md` directly without updating the corresponding source file in `01-prompts/`.
2. **Compile After Edits:** Always execute `scripts/update-prompts.ps1` (or `scripts/update-prompts.sh`) to re-sync the flat prompts and run the linter validation.
3. **No External Clone:** Never attempt to git clone external prompt repositories to update local prompts; this repository is fully self-contained.
