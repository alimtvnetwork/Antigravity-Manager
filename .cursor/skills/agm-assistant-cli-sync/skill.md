---
name: agm-assistant-cli-sync
description: Specialized skill for managing bidirectional assistant and CLI tool synchronizations (Hermes, OpenCode, OpenClaw, Droid, CLI paths, API key discovery) with the Antigravity-Manager proxy gateway.
---

# AGM Assistant & CLI Synchronization Engine

This skill governs the bidirectional configuration synchronization, model discovery, and status probing between Antigravity-Manager and external developer AI CLI tools and assistant ecosystems: **OpenCode**, **Hermes**, **OpenClaw**, **Droid**, and generic CLI executables.

---

## 1. Architectural Topology

AGM acts as the centralized AI proxy gateway on port 8045. The assistant sync modules locate external client configurations, inject AGM as an active custom provider (`antigravity-manager`), back up original configuration files, and expose real-time sync telemetry to the GUI and REST endpoints.

```
+----------------------------------------------------------------------------------------------------+
|                                    Antigravity-Manager Gateway                                     |
|                       Port 8045 (Axum Server, Canonical Intermediate Representation)                |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                     +----------------------------+----------------------------+
                     |                            |                            |
                     v                            v                            v
+-----------------------------+ +-----------------------------+ +-----------------------------+
|    OpenCode Synchronizer    | |     Hermes Synchronizer     | |    OpenClaw Synchronizer    |
|   (src/proxy/opencode_sync) | |   (src/proxy/hermes_sync)   | |   (src/proxy/openclaw_sync) |
| - .config/opencode/opencode | | - .hermes/config.yaml       | | - .openclaw/openclaw.json   |
| - Custom provider injection | | - YAML parser (yaml_rt)     | | - Target v1 / v2 versioning |
| - Model family discovery    | | - Auto backup (.bak)        | | - Atomic lock protection    |
+-----------------------------+ +-----------------------------+ +-----------------------------+
                     |                            |                            |
                     +----------------------------+----------------------------+
                                                  |
                     +----------------------------+----------------------------+
                     |                                                         |
                     v                                                         v
+---------------------------------------------+ +---------------------------------------------+
|              Droid Synchronizer             | |            Generic CLI Path Scanner         |
|           (src/proxy/droid_sync)            | |             (src/proxy/cli_sync)            |
| - Android / Droid agent environment config  | | - APPDATA, LOCALAPPDATA, ~/.cargo, ~/.bun   |
| - SOCKS / HTTP loopback proxy binding       | | - Grok, Claude, Gemini CLI executable probe |
+---------------------------------------------+ +---------------------------------------------+
```

---

## 2. Supported Assistant Ecosystems & Key Files

| Assistant Ecosystem | Core Sync File | Config Target Path | Configuration Format |
|---|---|---|---|
| **OpenCode** | `src-tauri/src/proxy/opencode_sync.rs` | `~/.config/opencode/opencode.json` (or `.jsonc`) | JSON / JSONC with `@ai-sdk/openai-compatible` |
| **Hermes** | `src-tauri/src/proxy/hermes_sync.rs` | `~/.hermes/config.yaml` | YAML (`yaml_rt` crate) |
| **OpenClaw** | `src-tauri/src/proxy/openclaw_sync.rs` | `~/.openclaw/openclaw.json` | JSON (v1 `<2026.8.1` and v2 `\ge 2026.8.1`) |
| **Droid** | `src-tauri/src/proxy/droid_sync.rs` | Droid environment & tool configurations | JSON / env flags |
| **CLI Tools** | `src-tauri/src/proxy/cli_sync.rs` | Windows/macOS/Linux executable lookup paths | Path probing, executable runner |

---

## 3. Core Synchronization Principles & Invariants

### 3.1 Non-Destructive Backup (`.antigravity-manager.bak`)
Before modifying any assistant configuration, the synchronizer creates an atomic copy of the active file using the suffix `.antigravity-manager.bak`.
- If an existing backup is present, it is preserved rather than overwritten by subsequent sync cycles.
- When un-syncing or restoring original settings, the backup is restored atomically.

### 3.2 Atomic Write with UUID Temp Files
To eliminate corrupted configurations during sudden process termination:
1. Write payload to temporary file: `<config_path>.tmp.<uuid>`.
2. Restrict filesystem permissions on Unix/Windows.
3. Atomically rename/replace over target file using standard OS atomic swap.

### 3.3 Concurrency Guard (Process Mutex)
Each assistant synchronizer maintains a static process-wide mutex guard (e.g. `OPENCODE_CONFIG_MUTEX`, `HERMES_CONFIG_MUTEX`, `OPENCLAW_CONFIG_MUTEX`).
- Poisoned locks are handled gracefully using `lock().unwrap_or_else(|poisoned| poisoned.into_inner())`.

### 3.4 Provider ID Invariant
All assistant configurations register AGM using the standard provider identifier:
- Provider ID: `antigravity-manager`
- Provider Display Name: `Antigravity Manager`
- Base URL: `http://127.0.0.1:8045/v1` (or user-defined proxy port)

---

## 4. Model Discovery & Mapping
The synchronizers continuously inspect active models available in AGM and project them into the assistant configs:
- Supported Families: `gemini-2.5-pro`, `gemini-2.5-flash`, `claude-3-7-sonnet`, `o3-mini`, `gpt-4o`.
- Maps model variant tiers and thinking flags to assistant-specific schema attributes.

---

## 5. Verification & Testing Checklist

When authoring or modifying assistant sync logic:
1. Ensure backup file existence is verified before writing changes.
2. Confirm atomic rename prevents partial writes.
3. Test against Windows (`%APPDATA%`, `%LOCALAPPDATA%`) and Unix (`~/.config`, `~/.*`) path hierarchies.
4. Run Rust formatting and clippy gates:
   ```bash
   cd src-tauri && cargo fmt -- --check
   cd src-tauri && cargo clippy --all-targets --all-features
   ```
