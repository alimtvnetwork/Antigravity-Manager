---
name: agm-json-envelope-format-inspector
description: Specialized skill for managing Antigravity-Manager two-tier JSON envelope schemas (v2.0), variable interpolation (${workDir}, ${repoDir}), relative path enforcement, legacy payload fallback, format inspection algorithms, and bulk import CLI generation.
---

# AGM Two-Tier JSON Envelope v2.0 & Variable Interpolation Engine

Comprehensive architectural guidance, schema definitions, format detection algorithms, and CLI integration standards for the JSON Envelope v2.0 and Format Inspector subsystem in Antigravity-Manager (`src-tauri/src/modules/json_envelope.rs` and `src-tauri/src/bin/agm.rs`).

---

## 1. Subsystem Architecture Overview

To eliminate path rigidity, host environment dependency, and data ambiguity across backups, exports, and multi-machine sync files, Antigravity-Manager enforces the **Two-Tier JSON Envelope Schema v2.0**:

```
+-----------------------------------------------------------------------------------------+
|                                    Inbound JSON File                                    |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                    Format Classifier & Variable Expansion Engine                        |
|                     src-tauri/src/modules/json_envelope.rs                              |
|  - Check for Tier 1 standard envelope: attributes.type & attributes.version ("2.0")     |
|  - 2-Pass Chained Variable Interpolation: ${workDir}, ${repoDir}, $variables.varName     |
|  - Relative Path Resolution: resolve_relative_json_path (vault/, instances/, .)         |
|  - Fallback to Tier 2 legacy signatures: detect top-level key patterns                  |
+--------------------------------------------+--------------------------------------------+
                      |                                            |
                      v                                            v
     [Tier 1: Standard Envelope v2.0]               [Tier 2: Legacy Flat JSON]
     {                                              {
       "attributes": {                                "active_instance_id": "...",
         "type": "agm/instances-export",               "instances": [ ... ]
         "version": "2.0",                          }
         "source": "instances.json",                --> auto-classified into
         "workDirectory": {                             agm/instances-export
           "path": "${workDir}",
           "isApplied": true,
           "isEnforced": false
         },
         "importCommand": "agm instances import instances.json"
       },
       "variables": {
         "workDir": "D:\\work",
         "repoDir": "${workDir}\\antigravity-manager"
       },
       "data": { ... }
     }
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                               FormatInspectionResult                                    |
|  - Status: matched / unrecognized                                                       |
|  - Detected Schema & Human-Friendly Description                                         |
|  - Item Count & Mutation Summary                                                        |
|  - Suggested AGM CLI Import Command (with relative path & -y non-interactive flag)      |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/json_envelope.rs` | Canonical envelope structs (`JsonEnvelope<T>`, `JsonAttributes`, `WorkDirectoryConfig`), 2-pass variable interpolation engine (`expand_variables_in_value`, `resolve_chained_variables`), relative path resolver (`resolve_relative_json_path`), and format classifier. |
| `src-tauri/src/bin/agm.rs` | CLI integration: `agm which-format <file>` (aliases: `format`, `inspect-format`, `scan-format`), reporting detected type, item count, and clean relative execution commands. |
| `src/pages/Accounts.tsx` | Accounts page modal supporting drag-and-drop or paste of JSON envelopes with real-time format detection and preview. |
| `tests/fixtures/json_envelope/` | Standardized test fixtures for all supported envelope schemas and unrecognized negative cases. |

---

## 3. Supported Format Taxonomy

| Envelope Type String | Target Domain | Core Keys / Data Content |
|---|---|---|
| `agm/accounts-export` | Accounts | Exported Google OAuth accounts, refresh tokens, quotas, and profile bindings. |
| `agm/instances-export` | Instances | Instance metadata, directory configurations, and isolated profile mappings. |
| `agm/supabase-credentials` | Supabase Vault | Centralized Supabase URL, service role key, and encryption secrets. |
| `agm/supabase-endpoints` | Supabase Multi-Node | Multi-region Supabase connection profiles and node routing endpoints. |
| `agm/config-backup` | System Config | Full application configuration (`gui_config.json`, proxy settings, switcher rules). |
| `agm/prompt-backups` | Prompt Telemetry | Backed up running prompts and task recovery snapshots from `backup-prompts.db`. |
| `agm/proxy-bindings` | Proxy Rules | Custom model redirect rules, thinking budgets, and upstream proxy pool assignments. |
| `agm/ssh-nodes-export` | Cluster Nodes | Mesh nodes list, connection parameters, public keys, and node health statuses. |

---

## 4. Variable Interpolation Engine (`src-tauri/src/modules/json_envelope.rs`)

### Syntax Variants Supported
1. `${varName}` (e.g. `${workDir}`)
2. `${variables.varName}` (e.g. `${variables.repoDir}`)
3. `$variables.varName` (e.g. `$variables.secretsDir`)

### 2-Pass Chained Resolution
Chained variables resolve in dependency order:
- `workDir`: `"D:\\work"`
- `repoDir`: `"${workDir}\\antigravity-manager"` $\to$ `"D:\\work\\antigravity-manager"`
- `dataDir`: `"${repoDir}\\instances"` $\to$ `"D:\\work\\antigravity-manager\\instances"`

### Value Expansion (`expand_variables_in_value`)
Recursively walks Serde JSON structures:
- String scalars: string interpolation via `resolve_string_variable`.
- Arrays: maps `expand_variables_in_value` over every element.
- Objects: recursively expands all values in key-value pairs.
- Numbers/Booleans/Null: returned untouched.

---

## 5. Relative Path Enforcement & Resolution

All generated import commands strictly use clean relative paths rather than machine-specific absolute paths (e.g. `agm instances import instances.json` instead of `agm instances import D:\work\instances.json`).

### Search Hierarchy (`resolve_relative_json_path`)
When an import command receives a relative filename:
1. Checks direct relative path in current working directory (`.`).
2. Checks common fallback directories:
   - `vault/`
   - `instances/`
   - `vault/instances/`
   - `02-antigravity-manager/vault/`
   - `01-gitmap/`
3. Returns resolved absolute path if found, or returns original path for error reporting.

---

## 6. Format Inspector CLI Usage (`agm which-format`)

```bash
# Inspect a candidate JSON file
agm which-format accounts.json

# Sample Output:
# [FORMAT] Detected: agm/accounts-export (v2.0)
# [ITEMS]  Found 4 accounts
# [SOURCE] Created by agm-cli at 2026-09-30 12:00:00 on worker-1
# [ACTION] To import, run:
#          agm accounts import accounts.json -y

# Directory recursion and batch discovery
agm which-format ./vault/ -y --run
```

---

## 7. Key Invariants

1. **Non-Destructive Inspection**: `agm which-format` MUST remain purely non-mutating unless `--run` (`-r`) is explicitly specified.
2. **Backward Compatibility**: `unpack_envelope()` must seamlessly accept both Schema v2.0 envelopes and legacy flat JSON payloads without throwing errors.
3. **Relative Path Generation**: Commands emitted by the inspector or export utilities must never hardcode absolute drive letters or home directories.
4. **Idempotent Imports**: Bulk imports must identify duplicates by primary keys (e.g. account email or instance ID) and update existing records safely.
