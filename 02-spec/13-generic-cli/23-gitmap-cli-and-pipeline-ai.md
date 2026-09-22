# Specification: GitMap Autonomous CLI Engine, Pipeline-AI Dynamic Waiting, and Resilient Distribution

> **Version:** 1.0.0 · **Updated:** 2026-09-23 · **Status:** Active  
> **Lead Architect:** Md. Alim Ul Karim · **Sponsor:** RISE UP ASIA LLC  
> **Related Specs:**
> - [01-index.md](01-index.md) — Generic CLI Overview & Standards
> - [22-self-update-gold-standard.md](22-self-update-gold-standard.md) — Self-Update Gold Standard
> - [02-spec/14-update/26-installer-multi-version-fallback-and-release-blocks.md](../14-update/26-installer-multi-version-fallback-and-release-blocks.md) — Multi-Version Fallback Ladder Spec
> - [02-spec/02-coding-guidelines/06-cicd-integration/06-distribution.md](../02-coding-guidelines/06-cicd-integration/06-distribution.md) — Distribution & Asset Decoupling

---

## 1. Architectural Scope & Purpose

GitMap is an ultra-fast developer companion, polyglot automation engine (AUM), and autonomous CI/CD self-healing tool designed for AI coding agents and systems engineers.

This specification establishes the normative architecture, command topologies, Pipeline-AI waiting protocols, multi-version installer fallback mechanisms, and release distribution standards for GitMap and all companion command-line tools across the meta-repository.

### Core Architectural Pillars
1. **Zero-Latency Repository Scanning**: Multi-threaded memory-mapped streaming discovery with lazy regex matching and binary null-byte filtering.
2. **Autonomous CI/CD Self-Healing (Pipeline-AI)**: Non-blocking workflow state inspection, automated failure log isolation, and dynamic ETA sleep protocols to eliminate credit-wasting tight polling loops.
3. **Resilient Installer Fallback Ladder**: Standalone shell installers (`install.ps1`, `install.sh`) that dynamically resolve 5 to 10 release candidates, falling back sequentially upon download or network failure.
4. **Decoupled Distribution**: Binary-only release assets with strict exclusion of installer scripts, accompanied by isolated single-line code blocks in release notes for 1-click installation.
5. **Strict Coding Guidelines & Error Management**: Structured error handling standardizing on `*appfault.AppError`, affirmative booleans without explicit `true` comparisons, and strict 8–15 line function bounding.

---

## 2. Command Architecture & Subsystem Topology

GitMap organizes operational capabilities into discrete, high-performance command groups:

```text
gitmap
├── aum                     # High-performance polyglot automation
│   ├── search              # Multi-core streaming regex/literal search
│   ├── guard               # File size (500KB) and null-byte binary probe
│   ├── sequence            # Markdown sequence gap detector & title normalizer
│   ├── exclude             # Persistent SQLite search exclusion rules
│   └── benchmark           # Comparative Go vs Python runtime benchmarks
├── pipeline-ai (pl-ai)     # Autonomous CI/CD self-healing & telemetry
│   ├── status              # Status probe with dynamic ETA and next-command inference
│   ├── error-logs          # Automated failure log extractor for 4-part RCA
│   └── purge               # Zero-storage workflow artifact purge
├── find-files (ff, ffa)    # Microsecond filename and pattern resolver
├── replace (replace-regex) # Transactional codebase string & regex replacement
├── cpf / cpb / cpr / pcp   # Autonomous semantic branch, commit, and push routines
└── cluster / sc / ssh      # Multi-node cluster orchestration & remote delegation
```

---

## 3. Pipeline-AI Dynamic Waiting Protocol (Anti-Credit-Waste Mandate)

Tight polling loops (`while true { gh run view }`) burn API rate limits, consume excessive tokens, and exhaust cloud billing credits. GitMap mandates an adaptive dynamic sleep protocol.

### Dynamic Telemetry Protocol
1. **Query Pipeline Status**: Execute `gitmap pipeline-ai status --json` to inspect the live run state.
   ```json
   {
     "repo": "alimtvnetwork/Antigravity-Manager",
     "branch": "main",
     "run_id": 35749889288,
     "status": "in_progress",
     "is_running": true,
     "etaSeconds": 145,
     "nextAiCommand": "gitmap pipeline-ai status -t 145"
   }
   ```
2. **Adaptive Interval Backoff**:
   - If `etaSeconds > 120s`: sleep 25s–30s before subsequent poll.
   - If `60s < etaSeconds <= 120s`: sleep 15s–20s before subsequent poll.
   - If `etaSeconds <= 60s`: sleep 5s–10s before subsequent poll.
3. **Automated Error Extraction for 4-Part RCA**:
   - Never stream or parse multi-megabyte raw logs of passing steps.
   - On run completion with failure, invoke `gitmap pipeline error-logs --run-id <ID>`.
   - Isolates exact compiler errors, fatal linker messages (`CVT1100`, `LNK1123`), panic backtraces, and failing unit test assertions (`##[error]`, `FAIL:`).

---

## 4. Multi-Version Fallback Ladder (5 to 10 Release Candidates)

When installing or updating GitMap and companion tools via standalone scripts, network instability, CDN propagation delays, or platform artifact omissions must never cause fatal aborts.

### Candidate Selection Algorithm
1. **Discovery Phase**:
   - Query GitHub Releases API (`/releases?per_page=30`) and GitHub Tags API.
   - Collect up to 10 semantic release tags (e.g. `v4.60.0`, `v4.59.0`, `v4.57.0`, ..., `v4.49.0`).
   - If API queries fail or rate-limit, fallback to an embedded array of verified historical versions.
2. **Sequential Retreat Loop**:
   - Attempt candidate 1 (latest available version).
   - If archive download, checksum validation, or execution fails:
     - Log non-fatal warning: `[!] Version vX.Y.Z failed. Retreating to preceding version vX.Y.(Z-1)...`
     - Attempt candidate 2.
     - Continue up to 10 sequential attempts.
   - Exit with success (`0`) as soon as any candidate installs and passes self-verification (`--version` check).
3. **Pinned Version Override**:
   - If `-Version "4.56.0"` or `--version 4.56.0` is explicitly provided, bypass the ladder and install the exact requested version.
   - Pinned mode must never silently retreat or substitute arbitrary versions.

---

## 5. Release Asset Decoupling & Publication Standards

### Strict Asset Decoupling
- **Binary Release Assets Only**: Assets uploaded to GitHub Releases (`release-files/`) MUST ONLY contain compiled platform artifacts (`.exe`, `.AppImage`, `.dmg`, `.deb`, `.rpm`, `.zip`, `.tar.gz`, `updater.json`, `checksums.txt`).
- **Installer Script Exclusion**: Standalone installer scripts (`install.ps1`, `install.sh`) MUST NOT be copied into `release-files/` or published as release assets.
- **Root Raw Distribution**: Installers are maintained at the repository root and fetched dynamically via raw GitHub content URLs or git tag endpoints:
  - Latest: `https://raw.githubusercontent.com/<owner>/<repo>/main/install.ps1`
  - Pinned: `https://raw.githubusercontent.com/<owner>/<repo>/<tag>/install.ps1`

### Mandatory Binary Asset Gate
Every automated CI/CD release workflow MUST assert that compiled binary artifacts exist in `release-files/` before creating or updating a release:
```bash
if [ -z "$(find . -maxdepth 1 -type f \( -name '*.exe' -o -name '*.dmg' -o -name '*.AppImage' -o -name '*.deb' -o -name '*.rpm' -o -name '*.zip' \))" ]; then
  echo "::error::CRITICAL: No binary artifacts found! Refusing to publish empty release."
  exit 1
fi
```

### Release Notes Code Block Isolation
Every release description MUST present quick-install commands in separate, isolated Markdown code blocks with zero inline comment noise to provide developers with native 1-click copy buttons:

#### Direct Latest Install (Auto-Updating)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

#### Pinned Version Install
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/v4.60.0/install.ps1 | iex
```

---

## 6. Coding Guidelines & Cross-Language Conventions

### Go Error Handling Standard (`*appfault.AppError`)
In all Go packages and CLI modules, functions returning structured failure metadata MUST use `*appfault.AppError` (`pkg/appfault`), replacing legacy `*apperror.AppError`:
```go
func RunPipelineAI(ctx context.Context, cfg *Config) *appfault.AppError {
    if cfg == nil {
        return appfault.NewInvalidArgument("config must not be nil")
    }
    // Execution logic bounded to 8-15 lines
    return nil
}
```

### Boolean Evaluation Standards
- **Zero Explicit True Comparisons**: NEVER evaluate booleans against `true` (`if isRunning == true` is prohibited; use `if isRunning`).
- **No Mixed Polarity**: Never combine affirmative and negative checks in a single conditional statement.

### Windows MSVC Resource Manifest Linking
On Windows platforms, when embedding Common-Controls 6.0 manifests for native dialogs or tests, configure the build attributes without default application manifests:
```rust
let windows = tauri_build::WindowsAttributes::new_without_app_manifest();
tauri_build::try_build(tauri_build::Attributes::new().windows_attributes(windows))
    .expect("failed to run build script");
```
This guarantees zero duplicate resource collisions (`CVT1100`, `LNK1123`) when linking against compiled resource libraries.
