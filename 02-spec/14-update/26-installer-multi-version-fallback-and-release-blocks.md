# Specification: Installer Multi-Version Fallback Ladder and Isolated Release Code Blocks

## 1. Architectural Scope & Purpose

This specification governs the design, behavior, and release requirements for standalone installer scripts (`install.ps1`, `install.sh`) and publication formats across the Antigravity-Manager repository and companion tools (e.g. GitMap).

### Core Objectives
1. **Decouple Installers from Binary Release Assets**: Standalone installer scripts reside at the repository root and are fetched directly from GitHub raw endpoints or git tags. They MUST NOT be packaged or uploaded as downloadable binary release assets.
2. **Multi-Version Fallback Ladder (5 to 10 Candidates)**: If the target release (latest or pinned) fails due to download timeouts, missing platform binaries, network instability, or corrupted archives, the installer must automatically retreat to the immediately preceding version in sequence and retry, supporting up to 10 sequential fallback attempts.
3. **Robust Pinned Version Resolution**: Installers must detect version constraints from CLI arguments, positional parameters, environment variables, scriptblock invocations, or raw tag URLs.
4. **Clean Aria2c Accelerator Delegation**: When aria2c is available, the installer delegates the transfer with an explicit notice, runs in clean/quiet mode without terminal progress bar spam, and prints concise start and completion summaries.
5. **Release Page Code Block Isolation**: Every published release body must format installation commands into separate, dedicated Markdown code blocks with independent copy buttons, eliminating mixed-content blocks that require manual editing.

---

## 2. Release Asset Decoupling Contract

### Principles
- Binary release assets (`release-files/`) MUST ONLY contain compiled platform artifacts (e.g. `.exe`, `.msi`, `.deb`, `.rpm`, `.AppImage`, `.dmg`, `.tar.gz`) and cryptographic signatures (`checksums.txt`, `.sig`).
- Standalone installer scripts (`install.ps1`, `install.sh`, `arch-install.sh`) MUST NOT be copied into `release-files/` or uploaded via GitHub release actions.
- Rationale: Storing installer scripts inside release assets causes circular dependency issues, confuses binary artifact listings, and breaks pinned installation if release assets differ from the git repository tree.

---

## 3. Multi-Version Fallback Ladder Architecture

### Candidate Queue Construction
1. **API Discovery**: Query GitHub API endpoints (`/releases?per_page=30` and `/tags`) to collect up to 10 valid semantic release versions.
2. **Historical Fallback List**: If the API is rate-limited, blocked, or offline, replenish the queue from an embedded array of known historical releases (e.g., `4.59.0`, `4.58.0`, `4.57.0`, `4.56.0`, etc.).
3. **Queue Sizing**: The candidate queue maintains a minimum of 5 and a maximum of 10 candidate releases ordered descending by version.
4. **Execution Ladder Loop**:
   - The installer attempts to download and verify candidate `i` (Attempt 1 of 10).
   - If candidate `i` succeeds, the installation completes and exits 0.
   - If candidate `i` fails (HTTP 404, checksum mismatch, empty package, execution error), the installer catches the failure, logs a warning, and retreats to candidate `i+1`.
   - Only when all 10 candidates fail does the installer exit with non-zero status.

```text
[Candidate Queue: v4.59.0, v4.58.0, v4.57.0, ..., v4.49.0] (Up to 10 releases)
       │
       ▼ Attempt 1: v4.59.0
   [Success?] ──Yes──► [Install & Verify] ──► [Exit 0]
       │ No (404 / hash fail)
       ▼ Attempt 2: v4.58.0 (Retreat)
   [Success?] ──Yes──► [Install & Verify] ──► [Exit 0]
       │ No
       ▼ Attempt 3: v4.57.0 (Retreat)
       ...
```

---

## 4. Pinned Version Resolution Standard

Installers must resolve the target version following this strict precedence order:

1. **Explicit CLI Flag**:
   - PowerShell: `-Version "4.56.0"`
   - Bash: `--version 4.56.0` or `-v 4.56.0` or `--version=4.56.0`
2. **Positional Argument**:
   - PowerShell: `.\install.ps1 "4.56.0"`
   - Bash: `./install.sh 4.56.0`
3. **Environment Variables**:
   - Primary: `AGM_VERSION` (or tool-specific `$env:GITMAP_VERSION`)
   - Secondary: `VERSION`
   - Universal fallback: `INSTALLER_VERSION`
4. **Raw Git Tag URL Detection**:
   - If invoked via `irm .../vX.Y.Z/install.ps1 | iex` or `curl .../vX.Y.Z/install.sh | bash`, inspect parent process command lines or `$MyInvocation` for git tag patterns:
     `(?i)(releases/download/|raw\.githubusercontent\.com/[^/]+/[^/]+/)v?([0-9]+\.[0-9]+(\.[0-9]+)?(-[a-zA-Z0-9.]+)?)/`
5. **Default**: Latest release discovered via GitHub API or historical fallback list.

---

## 5. Aria2c Delegation & Clean Output Standard

When delegating downloads to `aria2c`, the installer must adhere to clean logging conventions:

### Delegation Notice
```text
[*] Delegating download request to aria2c accelerator...
[*] Accelerating download with aria2c (16 connections, 80 splits, 1MB chunks)...
```

### Quiet Parameters
To prevent terminal flooding with multi-line interactive progress dumps:
- PowerShell:
  `--summary-interval=0 --console-log-level=error --show-console-readout=false`
- Bash:
  `--summary-interval=0 --console-log-level=error --show-console-readout=false`

### Completion & Fallback Summary
- On Success:
  `[OK] Download completed successfully via aria2c (85.24 MB).`
- On Failure:
  `[!] aria2c finished with code 1; delegating download request to secondary downloader (curl / Invoke-WebRequest)...`

---

## 6. GitHub Release Page & Markdown Code Block Standards

Every published GitHub / GitLab release body must present installation one-liners in separate, dedicated Markdown code blocks. Never combine direct and pinned commands into a single block with comments.

### Required Release Body Structure

```markdown
## Quick Install vX.Y.Z

### Windows (PowerShell 5.1+)

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/<owner>/<repo>/main/install.ps1 | iex
```

#### Pinned Version Install (vX.Y.Z)
```powershell
irm https://raw.githubusercontent.com/<owner>/<repo>/vX.Y.Z/install.ps1 | iex
```

---

### Linux / macOS (Bash)

#### Direct Latest Install (Auto-Updating)
```bash
curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/main/install.sh | bash
```

#### Pinned Version Install (vX.Y.Z)
```bash
curl -fsSL https://raw.githubusercontent.com/<owner>/<repo>/vX.Y.Z/install.sh | bash
```

---
```

This layout provides an independent copy button for each variant so developers can copy and install with a single click.
