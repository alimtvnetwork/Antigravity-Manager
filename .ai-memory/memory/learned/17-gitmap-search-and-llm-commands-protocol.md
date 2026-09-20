# Learned Protocol: GitMap Fast Search & LLM Commands Mandate

## 1. Context & Ban on Broad OS-Level Scans

AI agents must **NEVER** run broad, recursive filesystem scans via shell commands (e.g. `Get-ChildItem -Path "C:\Program Files*" -Recurse`, `find / -name ...`, or scanning entire system root directories).
These operations spawn long-running background tasks, lock execution threads, flood OS disk I/O, waste IDE resources, and cause noticeable UI stalls.

## 2. Mandatory Search Protocols

When searching for files, tokens, or repo topology, agents MUST use one of two ultra-fast pathways:

### Pathway A: GitMap Search & Discovery CLI

GitMap provides instantaneous indexed searching with extension filtering:
- `gitmap find-files (ff) <name>`: Find files matching exact filename.
  ```bash
  gitmap ff "Cargo.toml"
  ```
- `gitmap find-files-any (ffa) <substr> [-ext <ext>]`: Find files containing substring with optional extension filter.
  ```bash
  gitmap ffa "instance" -ext "rs, ts"
  ```
- `gitmap find-files-startswith (ffs) <prefix> [-ext <ext>]`: Find files with prefix.
  ```bash
  gitmap ffs "01-" -ext "md"
  ```
- `gitmap find-files-endswith (ffe) <suffix> [-ext <ext>]`: Find files with suffix.
  ```bash
  gitmap ffe "_test.go" -ext "go"
  ```
- `gitmap find (f) "<wildcard*>" [-ext <ext>]`: Universal glob search.
  ```bash
  gitmap f "*runner*" -ext "py"
  ```
- `gitmap list-files (lf) [pattern]`: List indexed repository files.

### Pathway B: Repository Python Discovery Suite (`03-ai-scripts/`)

- Fast File Scanner: `python 03-ai-scripts/11-fast-file-scanner.py --search "<pattern>" --limit 20`
- Fast Cached Grep: `python 03-ai-scripts/12-fast-cached-grep.py --pattern "<pattern>" --limit 20`
- Fast File Reader: `python 03-ai-scripts/17-fast-file-reader.py --read-file <path>`
- Fast Folder Listing: `python 03-ai-scripts/17-fast-file-reader.py --list-folder <path>`
- Codebase Topology: `python 03-ai-scripts/18-codebase-topology-discoverer.py`

## 3. GitMap LLM & AI Commands

GitMap features dedicated subcommands engineered specifically for AI agent integration:

1. **`gitmap llm-docs (ld)`**:
   Generates a consolidated LLM reference document (`LLM.md` or `LLM.json`) describing all 60+ gitmap commands, global flags, architecture, database schemas, and workflows.
   - `gitmap ld --stdout`: Prints Markdown LLM reference directly to stdout.
   - `gitmap ld --format json`: Outputs structured JSON for AI tooling.
   - `gitmap ld --sections commands,flags,patterns`: Targets specific sections.

2. **`gitmap pipeline-ai (pl-ai)`**:
   Autonomous CI/CD pipeline monitoring with dynamic waiting:
   - `gitmap pl-ai status`: Auto-delays before querying status and prints recommended next action.
   - `gitmap pl-ai status -t <sec>`: Delays dynamically based on workflow ETA, completely preventing tight polling loops.
   - `gitmap pl-ai status --json`: Returns machine-readable diagnostic payloads with `nextAiCommand`.

3. **`gitmap antigravity (agy)`**:
   Antigravity workspace and prompt synchronization:
   - `gitmap agy ls`: Lists registered Antigravity projects.
   - `gitmap agy status`: Reports IDE and profile runtime status.
   - `gitmap agy sync`: Synchronizes workspace configurations.
