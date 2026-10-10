# Architecture Specification: Proxy Modular Architecture & CI/CD Pipeline Resolution

> **Target:** `02-spec/21-app/gitmap-pe-execution-and-csv-issue-resolution/01-architecture-spec.md`  
> **Status:** APPROVED  
> **Author:** @aukgit  
> **Version:** 1.0.0  
> **Slug:** `gitmap-pe-execution-and-csv-issue-resolution`  
> **Scope:** Rust Reverse Proxy Layer, God-Module Refactoring Post-Mortem, Visibility Architecture, CI Pipeline RCA (#38055868344), Symbol Encapsulation, Speech-to-Text Phonetic Disambiguation  

---

## 1. Executive Summary & Problem Formulation

Antigravity-Manager (AGM) operates a local, high-performance reverse proxy gateway written in Rust (`src-tauri/src/proxy/`) that aggregates four upstream AI protocols (OpenAI Responses, OpenAI Chat Completions, Anthropic Claude, and Google Gemini) and outputs canonical Antigravity-style Gemini protocol streams.

In recent maintenance cycles (commits `22a87aae` and `50a0e58f`), a repository-wide refactoring effort was executed to decompose monolithic source files ("god modules") exceeding 500 lines into fine-grained submodules to strictly conform to repository file-size limits (`02-spec/02-coding-guidelines/02-canonical-size-tier.md`). 

While this mechanical modularization succeeded in bringing individual file lengths below the 500-line ceiling, it introduced severe Rust module hierarchy and visibility regressions. Specifically, in GitHub Actions CI Run `#38055868344` on commit `13a820d` (`main` branch), the Rust verification pipeline failed completely across macOS, Ubuntu, and Windows runners:
- **503 compilation errors** occurred during the library build (`cargo check` / `cargo clippy` on `agm-alim`).
- **606 compilation errors** and **1,108 warnings** were emitted during test compilation (`agm-alim (lib test)`).
- Multiple runners halted immediately at the crate root due to module path resolution failures (`error[E0583]: file not found for module appruntimeflags`).

This specification provides the end-to-end architectural analysis of the Rust proxy layer, conducts a root cause analysis (RCA) of the failures captured via GitMap's pipeline telemetry (`gitmap pe`), formalizes Rust's visibility boundaries (`pub(crate)` vs `pub` vs private) across modular sub-files, disambiguates the user request's phonetic "CSV" artifact, and defines immutable architectural invariants to prevent future decomposition regressions.

---

## 2. Phonetic Disambiguation: "CSV" vs "CI/CD" Pipeline Telemetry

### 2.1 Speech-to-Text (STT) Acoustic Artifact Analysis
The user prompt references `gitmap-pe-execution-and-csv-issue-resolution`. In developer workflows utilizing automated voice dictation and speech-to-text (STT) models (such as Letterly, Dragon, or Whisper), acoustic token confusion occurs frequently when technical acronyms are vocalized:

- **Intended Technical Phrase:** "CI/CD" (Continuous Integration / Continuous Delivery).
  - Phonetic transcription: `/ˌsiː.aɪ.siːˈdiː/` or collapsed colloquial `/siː.siː.diː/`.
- **Interpreted Token:** "CSV" (Comma-Separated Values).
  - Phonetic transcription: `/ˌsiː.esˈviː/`.

Because the terminal characters `/diː/` and `/viː/` have high acoustic similarity over standard microphone sampling rates, the STT engine erroneously transcribed "CI/CD" as "CSV".

### 2.2 GitMap Telemetry Correlation (`gitmap pe`)
The prefix `gitmap pe` directly validates this phonetic diagnosis:
- `gitmap pe` is the shorthand invocation for `gitmap pipeline-errors`.
- It connects to GitMap's SQLite pipeline telemetry database (`data/pipeline/alimtvnetwork-antigravity-manager/sql.db`).
- In run `#38055868344`, `gitmap pe` reported broken CI runs across 6 pipeline sections (`Check Rust Code` on macOS/Ubuntu/Windows and `Build Tauri App` on macOS/Ubuntu/Windows).
- No actual tabular `.csv` data files are involved in this task. The problem scope strictly concerns resolving the compilation failures across the CI/CD pipeline.

---

## 3. Antigravity-Manager Rust Proxy Layer Architecture

### 3.1 Subsystem Overview (`src-tauri/src/proxy/`)
The reverse proxy layer is the core gateway of Antigravity-Manager. It mediates all traffic between developer tools (Hermes Agent, OpenCode, OpenClaw, Droid, Gemini CLI, Cursor, Antigravity IDE) and upstream inference providers:

```mermaid
flowchart TD
    Client["Client IDE / CLI Tools\n(Hermes, OpenCode, Antigravity)"] -->|HTTP / WebSocket| AxumServer["Axum Server (src-tauri/src/proxy/server/)\nRouter Proxy & Admin"]
    
    AxumServer --> Security["Security Guard (src-tauri/src/proxy/security/)\nIP CIDR Whitelist & Curfew"]
    Security --> Auth["Token Manager (src-tauri/src/proxy/token_manager/)\nBearer Token Provisioning"]
    
    Auth --> Mappers["Protocol Mappers (src-tauri/src/proxy/mappers/)\nClaude, OpenAI, Gemini Adapters"]
    
    subgraph State Management
        SessionStore["HTTP Session Store\n(src-tauri/src/proxy/http_session_store/)\nTree-based Multi-Turn Storage"]
        SigCache["Signature Cache\n(src-tauri/src/proxy/signature_cache/)\nTool & Thinking Signatures"]
        SessionMgr["Session Manager\n(src-tauri/src/proxy/session_manager/)\nFingerprint Sanitization"]
        RateLimit["Rate Limit Tracker\n(src-tauri/src/proxy/rate_limit/)\nQuota Windows & Backoff"]
    end
    
    Mappers <--> State Management
    
    Mappers --> Pipeline["Pipeline Stage (src-tauri/src/proxy/pipeline/)\nThinking Invariant I4 & Content Normalization"]
    
    Pipeline --> Upstream["Upstream Client (src-tauri/src/proxy/upstream/)\nFallback Ladder: Daily -> Sandbox -> Prod"]
    
    Upstream --> Egress["Proxy Pool (src-tauri/src/proxy/proxy_pool/)\nSOCKS5 / HTTP Proxy Egress"]
    Egress --> LLM["Upstream LLM Endpoints\n(Gemini, Claude, OpenAI, ZAI)"]
```

### 3.2 Core Gateway vs. Adjacent Subsystems
Per `docs/module-tiers.md` and repository architectural invariants:
- **Core Tier:** `src-tauri/src/proxy/` is strictly Tier-1 Core Gateway infrastructure.
- **Dependency Flow Rule:** Proxy code may import common support utilities from `crate::modules` (such as `logger`), but must **NEVER** import Adjacent modules (`email_*`, `telegram_inbound`, `supabase_*`, `ssh_manager`, `cloudflared`, `notification_hub`, `backup_prompts_db`, `agy_cleaner`).
- Dependencies must strictly point **ADJACENT -> PROXY**, never the reverse.

---

## 4. Forensic Analysis of the God-Module Refactor

### 4.1 Refactoring Motivation & Methodology
Prior to commit `22a87aae`, multiple core proxy modules were monolithic single-file implementations exceeding 1,000 lines of code. This triggered continuous warnings under repository guideline `02-spec/02-coding-guidelines/02-canonical-size-tier.md` (which caps standard source files at <= 500 lines).

An automated script was executed across commits `22a87aae` and `50a0e58f` to segment large files:
1. Large files like `rate_limit.rs`, `signature_cache.rs`, `http_session_store.rs`, and `session_manager.rs` were replaced with directory modules containing split files.
2. `src-tauri/src/lib.rs` was split into `appruntimeflags.rs`, `run.rs`, and `setup_app.rs`.
3. Unit test modules (`#[cfg(test)] mod tests`) were extracted from file tails into standalone `tests.rs` files.
4. Auto-generated re-exports were placed in top-level module files (e.g. `pub(crate) use tests::tests;`).

### 4.2 Structural Fragmentation & Defect Injections
The automated partitioning script had zero semantic awareness of Rust's compile-time visibility and module lookup invariants:

1. **Crate Root Path Inversion:**
   - In Rust 2018+, submodules of a file `foo.rs` reside in `foo/bar.rs`.
   - However, `src-tauri/src/lib.rs` is the **crate root**! Submodules declared as `mod appruntimeflags;` inside `lib.rs` are looked up by `rustc` at `src/appruntimeflags.rs` or `src/appruntimeflags/mod.rs`, **NOT** `src/lib/appruntimeflags.rs`.
   - The script created `src-tauri/src/lib/appruntimeflags.rs`, triggering instant compiler fatal error `E0583` on all platforms.

2. **Submodule Visibility Erasure:**
   - In a single monolithic file, private methods (`fn helper(&self)`) and struct fields (`field: T`) are accessible everywhere within the file, including sibling functions and child test modules.
   - When split across files in a directory module, each file becomes a distinct Rust submodule.
   - Submodules cannot access private items of sibling submodules. Because the splitter left methods and fields without `pub(crate)` qualifiers, cross-file calls failed with `error[E0624]: method is private` and `error[E0616]: field is private`.

3. **Test Module Re-Export Pollution:**
   - The splitter extracted `tests.rs` containing `#[cfg(test)] pub(crate) mod tests { ... }`.
   - In the parent module, it injected `pub(crate) use tests::tests;`.
   - This exposed internal test modules into the crate-wide namespace, caused duplicate symbol clashes, and failed during non-test compilation passes.

---

## 5. Root Cause Analysis (RCA) of 606 Compilation Errors

### 5.1 RCA Part 1: Symptom & Concrete Evidence
In CI Run `#38055868344`, execution failed in `Check Rust Code` across all three OS runners:
```text
error: could not compile `agm-alim` (lib) due to 503 previous errors; 1103 warnings emitted
error: could not compile `agm-alim` (lib test) due to 606 previous errors; 1108 warnings emitted
Process completed with exit code 1.
```

### 5.2 RCA Part 2: Immediate Failure Mechanisms (5 Primary Failure Classes)

| Failure Class | Rust Diagnostic | Root Mechanism | Example Location |
| :--- | :--- | :--- | :--- |
| **Class A: Crate Root Module Lookup** | `E0583: file not found for module` | `lib.rs` submodules located in `src/lib/` instead of `src/` or lacking `#[path]` attributes | `src-tauri/src/lib.rs:23` looking for `mod appruntimeflags` |
| **Class B: Sibling Method Invisibility** | `E0624: method / associated function is private` | Cross-file calls between sibling submodules target unexported `fn` items | `src-tauri/src/proxy/rate_limit/ratelimittracker_impl_2.rs:311` calling `parse_rate_limit_reason` |
| **Class C: Sibling Struct Field Invisibility** | `E0616: field of struct is private` | Methods in split `impl` chunks access struct fields declared without `pub(crate)` | `src-tauri/src/proxy/rate_limit/ratelimittracker_impl.rs:288` accessing `self.limits` |
| **Class D: Namespace Clashes & Test Exposure** | `E0252 / E0255 / E0428` | Parent files re-exporting `pub(crate) use tests::tests;` or `pub use tests::*;` | `src-tauri/src/proxy/signature_cache.rs:11` re-exporting `tests::tests` |
| **Class E: Trait & Type Mismatches** | `E0277 / E0308 / E0599 / E0600` | StreamExt out of scope, dereference omissions on `&mut bool`, Axum 0.7 middleware bounds | `src-tauri/src/proxy/mappers/gemini/wrapper/v2_compression.rs:74` assigning `bool` to `&mut bool` |

### 5.3 RCA Part 3: Underlying Architectural Flaw
The underlying architectural flaw was treating code decomposition as purely textual/physical line splitting rather than semantic Rust module reconstruction. In Rust, file boundaries **are** module boundaries. A file split is inherently a module boundary introduction, requiring explicit visibility scoping (`pub(crate)`) and namespace containment.

### 5.4 RCA Part 4: Systemic Preventive Invariants
1. **Never perform blind mechanical splits without local compilation verification.**
2. **Every cross-file internal API within a multi-file module must be explicitly marked `pub(crate)`.**
3. **Test modules must remain strictly private to their defining module and never be re-exported with `pub` or `pub(crate)`.**
4. **Crate root (`src/lib.rs`) files must keep their submodules in `src/` or use explicit `#[path = "..."]` directives.**

---

## 6. Rust Visibility Architecture in Modular Decomposition

To ensure strict encapsulation without breaking modular sibling files, Antigravity-Manager defines the following 4-tier visibility model:

```mermaid
flowchart TD
    subgraph External Crates
        Ext["External Binaries / CLI / Tests"]
    end
    
    subgraph agm-alim Crate
        subgraph Proxy Module
            subgraph Submodule A
                Priv["private fn / fields\n(File internal only)"]
                PubSuper["pub(super)\n(Parent module only)"]
            end
            subgraph Submodule B
                PubCrate["pub(crate)\n(Entire crate visible,\nideal for split modules)"]
            end
        end
        PubAPI["pub\n(Crate public API)"]
    end
    
    Ext -->|Can access only| PubAPI
    Submodule B -->|Can access| PubCrate
    Submodule B -.->|BLOCKED| Priv
```

### 6.1 Visibility Specification Table

| Visibility Modifier | Accessible Scope | Appropriate Usage in AGM |
| :--- | :--- | :--- |
| `private` (`fn foo()`, `field: T`) | Only within the current `.rs` file and its inner submodules | Purely internal helper functions and local struct fields never accessed outside the file. |
| `pub(super)` | Current `.rs` file and its immediate parent module | Items shared strictly between a submodule and its direct parent. |
| `pub(crate)` | Anywhere within `src-tauri/` (`agm-alim` crate) | **The Standard for Modular Split Files.** All struct fields, methods, constructors, and helpers shared across split files in a directory module. |
| `pub` | Entire crate + external crates/binaries | Public gateway API items (`AxumServer`, `ProxyConfig`, `TokenManager`, `SignatureCache`). |

### 6.2 Test Module Encapsulation Rule
Test modules in split files must be written as:
```rust
#[cfg(test)]
mod tests {
    use super::*;
    // Unit tests...
}
```
In the parent module file:
- **FORBIDDEN:** `pub(crate) use tests::tests;` or `pub use tests::*;`
- **REQUIRED:** Either omit the re-export entirely (tests execute autonomously via `cargo test`), or maintain conditional compilation:
```rust
#[cfg(test)]
mod tests;
```

---

## 7. Architecture Invariants & Coding Guidelines

### 7.1 Affirmative Boolean Invariant
All boolean identifiers across Rust and TypeScript codebases must use positive polarity (`is_*`, `has_*`, `should_*`, `can_*`).
- **FORBIDDEN:** `is_not_expired`, `has_no_retry`, `disable_fallback = false`
- **REQUIRED:** `is_expired`, `has_retry`, `is_fallback_enabled = true`

### 7.2 Strict Relative Git Paths Invariant
All file paths documented in specifications, plans, and code comments must be strictly relative to the repository root (e.g., `src-tauri/src/proxy/rate_limit.rs`). Never use absolute local filesystem paths (`C:\...`, `/home/...`) or `file:///` URIs.

### 7.3 No Silent Errors (House Rule)
Every fallible operation must route its failure through `AppError` (`src-tauri/src/error.rs`) and `tracing::error!`. Never swallow errors with bare `let _ =`, `.ok()`, or `.unwrap_or_default()`. If an operation is genuinely best-effort, call `crate::error::record_ignored(...)` with a mandatory `// Justification:` comment.

---

## 8. Verification Gate & Quality Acceptance Criteria

To achieve green status and close the pipeline failure:
1. **Local Pre-Flight Gate:**
   - `cd src-tauri && cargo fmt -- --check` must exit 0.
   - `cd src-tauri && cargo clippy --all-targets --all-features` must exit 0 with 0 errors and 0 warnings.
   - `cd src-tauri && cargo test` must pass all test suites.
2. **CI Pipeline Gate:**
   - GitHub Actions CI Run across `Check Rust Code (macos-latest)`, `Check Rust Code (ubuntu-latest)`, and `Check Rust Code (windows-2022)` must conclude with `conclusion: success`.
   - `gitmap pe` must report `0 failed section(s)`.
