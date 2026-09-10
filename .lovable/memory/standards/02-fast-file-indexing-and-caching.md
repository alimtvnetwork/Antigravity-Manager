---
name: Fast File Indexing & Caching Strategy
description: Explains how AI agents leverage 03-ai-scripts/11-fast-file-scanner.py and 09-fast-cached-grep.py to index and search repository files in tmp/ cache for rapid retrieval and zero-overhead discovery.
type: standard
---

# Fast File Indexing & Caching Strategy

**Scanner Tool:** `03-ai-scripts/11-fast-file-scanner.py`
**Grepper Tool:** `03-ai-scripts/12-fast-cached-grep.py`
**Cache Outputs:** `tmp/repo-file-cache.json`, `tmp/file-list-all.txt`, `tmp/file-list-<slug>.txt`, `tmp/grep-results.json`
**Tags:** `#file-scanner`, `#cached-grep`, `#caching`, `#performance`, `#ai-workflow`

## 1. The Multi-Step File Discovery Problem

In large repositories (>2,000 files), repeated ad-hoc file searches (e.g. recursive `os.walk`, deep grep searches, or multiple shell glob queries) consume significant time and token budget across multiple agent steps.

## 2. The Solution: Pre-Computed Fast File Caching

Before embarking on multi-file audits, refactoring passes, or cross-spec updates, AI agents can execute `08-fast-file-scanner.py` once to build a filtered, memory-efficient index in `tmp/`:

```bash
# Scan full repository:
python 03-ai-scripts/11-fast-file-scanner.py

# Scan specific language files:
python 03-ai-scripts/11-fast-file-scanner.py --lang go,ts,tsx

# Scan specifications only:
python 03-ai-scripts/11-fast-file-scanner.py --path spec/ --ext .md
```

## 3. Cached File Contract

The scanner automatically writes structured outputs into `tmp/`:

1. **`tmp/repo-file-cache.json`**: Structured JSON containing metadata, scan duration, extension statistics, and an array of relative file paths.
2. **`tmp/file-list-all.txt`**: Clean newline-delimited text list for immediate ingestion by subsequent Python/Bash scripts.
3. **`tmp/file-list-<filter>.txt`**: Scoped text file list for specific language/path combinations (e.g. `tmp/file-list-lang-go_ts_jsx.txt`).

## 4. Instant Cache Querying & Content Grepping

Agents can query the pre-computed index in sub-millisecond time:

```bash
# Instant file path lookup from pre-computed cache:
python 03-ai-scripts/11-fast-file-scanner.py --query-cache "slides"

# Parallel cached content grepping:
python 03-ai-scripts/12-fast-cached-grep.py --pattern "AppError"
```

## 5. Agent Consumption Pattern in Python Scripts

In subsequent turns or script executions, agents do NOT need to re-query the filesystem:

```python
import json

# Read pre-computed file cache instantly (<1ms):
with open("tmp/repo-file-cache.json", "r", encoding="utf-8") as f:
    file_list = json.load(f)["files"]

for filepath in file_list:
    # Process files directly
    pass
```

## 6. Exclusions & Clean Filtering

The scanner automatically filters out:
- Version control: `.git/`
- Package managers & builds: `node_modules/`, `dist/`, `build/`, `bin/`, `obj/`, `.next/`, `.cache/`, `.venv/`
- Internal agent dirs: `.gemini/`, `.agent/`
- Binary assets: images, zip archives, compiled executables, SQLite databases.
- Preserves necessary metadata folders: `.lovable/`, `.github/`.
