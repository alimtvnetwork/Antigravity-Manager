---
name: agm-json-envelope-format-inspector
description: Specialized skill for managing Antigravity-Manager two-tier JSON envelope schemas, legacy payload fallback, format inspection algorithms, and bulk import CLI generation.
---

# AGM Two-Tier JSON Envelope & Format Inspector Architecture

This skill provides comprehensive architectural guidance, schema definitions, format detection algorithms, and CLI integration standards for the JSON Envelope and Format Inspector subsystem in Antigravity-Manager.

---

## 1. Subsystem Architecture Overview

To eliminate ambiguity across configuration backups, credentials exports, and multi-machine sync files, Antigravity-Manager implements a two-tier JSON envelope standard (Plan 84 / Spec 02):

```
+-----------------------------------------------------------------------------------------+
|                                    Inbound JSON File                                    |
+--------------------------------------------+--------------------------------------------+
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                           Format Classifier & Inspector                                 |
|                     src-tauri/src/modules/json_envelope.rs                              |
|  - Check for Tier 1 standard envelope: attributes.type                                  |
|  - Fallback to Tier 2 legacy signatures: detect top-level key patterns                  |
+--------------------------------------------+--------------------------------------------+
                      |                                            |
                      v                                            v
     [Tier 1: Standard Envelope]                    [Tier 2: Legacy Flat JSON]
     {                                              {
       "attributes": {                                "active_instance_id": "...",
         "type": "agm/accounts-export",               "instances": [ ... ]
         "version": "1.0",                          }
         "source": "agm-cli",
         "created_at": "..."                        --> auto-classified into
       },                                               agm/instances-export
       "data": { ... }
     }
                                             |
                                             v
+-----------------------------------------------------------------------------------------+
|                               FormatInspectionResult                                    |
|  - Status: matched / unrecognized                                                       |
|  - Detected Schema & Human-Friendly Description                                         |
|  - Item Count & Mutation Summary                                                        |
|  - Suggested AGM CLI Import Command (with -y non-interactive flag)                      |
+-----------------------------------------------------------------------------------------+
```

---

## 2. Key Files & Core Responsibilities

| File Path | Core Responsibilities |
|---|---|
| `src-tauri/src/modules/json_envelope.rs` | Canonical envelope structs (`JsonEnvelope<T>`, `EnvelopeAttributes`), legacy format heuristics, `classify_format`, `inspect_json_file`, `resolve_json_targets`, and CLI command generator. |
| `src-tauri/src/bin/agm.rs` | CLI integration: `agm which-format <file>` (aliases: `format`, `inspect-format`, `scan-format`), reporting detected type, item count, and copy-pasteable execution command. |
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

## 4. Format Inspector CLI Usage (`agm which-format`)

The `agm which-format` command inspects arbitrary JSON files without modifying system state:

```bash
# Inspect a candidate JSON file
agm which-format ./backup.json

# Sample Output:
# [FORMAT] Detected: agm/accounts-export (v1.0)
# [ITEMS]  Found 4 accounts
# [SOURCE] Created by agm-cli at 2026-09-29T19:40:00Z on node-1
# [ACTION] To import, run:
#          agm accounts import ./backup.json -y

# Directory recursion and batch discovery
agm which-format ./backups/
```

### Directory Traversal & Target Resolution (`resolve_json_targets`)
When given directories or globs:
- Recursively gathers all matching `*.json` files.
- Inspects each candidate, skipping non-JSON or unparseable files without crashing.
- `build_bulk_import_command` synthesizes a single composite one-line command (chaining with `&&` and `-y`) to import all detected schemas in one go.

---

## 5. Architectural Invariants for JSON Exporters & Importers

1. **Always Emit Envelopes**: All new export functions must wrap payloads inside `JsonEnvelope::new(payload, "agm/<domain>-export")`.
2. **Never Break Legacy Reading**: Importers must always pass incoming JSON through `json_envelope::unpack_envelope`, ensuring legacy flat files from older AGM releases continue to import seamlessly.
3. **No Interactive Prompts with `-y`**: When the user passes `-y` or `--yes`, CLI import handlers must execute non-interactively without prompting for stdin confirmation.
