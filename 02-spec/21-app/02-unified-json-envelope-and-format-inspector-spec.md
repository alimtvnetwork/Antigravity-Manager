# 02 — Canonical Specification: Unified JSON Envelope Schema & Smart Format Inspector

> **/goal** Establish the authoritative, system-wide standard for all structured JSON data across Antigravity-Manager, the AGM CLI (`agm`), and connected secret repositories (`repo-secrets`). Every JSON emitted or ingested must conform to the two-tier `{ "attributes": ..., "data": ... }` envelope, supported by the `agm which-format` CLI command for autonomous inspection, diff/mutation impact summaries, and single-line batch execution.

**Version:** 1.0.0 (Canonical)  
**Last Updated:** 2026-09-29  
**Status:** Binding & Enforced  
**Applies To:** Antigravity-Manager Core, AGM CLI (`agm`), `d:\work\repo-secrets`, Import/Export Pipelines, Backup Engines

---

## 1. Unified JSON Envelope Architecture

Every structured JSON file throughout the platform consists of two top-level keys:

```json
{
  "attributes": {
    "type": "agm/supabase-endpoints",
    "version": "1.0",
    "source": "agm-cli",
    "created_at": "2026-09-29T18:50:00Z",
    "node_id": "8236325f-0727-48c7-8015-2ab2e9f2f0f1",
    "node_alias": "Node-823632",
    "encoding": "utf-8",
    "checksum_sha256": "optional-hash-for-integrity",
    "description": "Supabase fleet endpoint configuration"
  },
  "data": {
    ...
  }
}
```

### 1.1 Top-Level Key Invariants
1. **`attributes` (Metadata Section)**:
   - **`type` (Required)**: Standardized URI identifier declaring exact schema (see Taxonomy in Section 2).
   - **`version` (Required)**: Semantic schema version (e.g. `"1.0"`).
   - **`source` (Optional)**: Originating system/tool (`"agm-cli"`, `"agm-gui"`, `"repo-secrets"`, `"backup-engine"`).
   - **`created_at` (Optional)**: ISO 8601 UTC timestamp.
   - **`node_id` / `node_alias` (Optional)**: Machine origin metadata.
   - **`encoding` (Optional)**: `"utf-8"`, or `"base64"` if payload is encoded.
2. **`data` (Payload Section)**:
   - Contains the payload content (array or object) specific to the declared `attributes.type`.

---

## 2. Standardized Type Taxonomy

| Declared `attributes.type` | Payload Description (`data`) | Primary Target Command | Mutation / Change Impact |
| :--- | :--- | :--- | :--- |
| `agm/supabase-endpoints` | List of Supabase endpoint configurations (`id`, `url`, `api_key`, `role`) | `agm supabase load-json <file> -y` | Registers/updates remote Supabase REST endpoints in local `supabase_config.json` |
| `agm/supabase-credentials` | Single or multiple Supabase service keys/tokens | `agm supabase load-json <file> -y` | Ingests fleet credentials into local node vault |
| `agm/accounts-export` | Google OAuth account tokens, refresh tokens, and quotas | `agm accounts import <file> -y` | Merges accounts into `accounts.json` and local OS Keyring |
| `agm/instances-export` | Multi-instance profiles and data directory mappings | `agm instances import <file> -y` | Registers sandbox IDE profiles in `instances.json` |
| `agm/config-backup` | System GUI and daemon configuration | `agm config restore <file> -y` | Overwrites or patches `gui_config.json` |
| `agm/prompt-backups` | Backed-up in-flight prompts and task states | `agm prompts restore <file> -y` | Restores conversational tasks into `backup-prompts.db` |
| `agm/proxy-bindings` | Model routing and quota protection rules | `agm proxy bindings import <file> -y` | Reconfigures local AI proxy bindings |

---

## 3. Backward Compatibility & Signature Auto-Detection

When reading files:
1. **Envelope Present**: If root JSON contains `"attributes"` and `"data"`, read `attributes.type`.
2. **Legacy Fallback Heuristics**: If root JSON is flat, inspect top-level fields:
   - Contains `endpoints` array with `url` and `api_key` $\rightarrow$ inferred `agm/supabase-endpoints`.
   - Contains `service`, `endpoint`, and `token` (or base64 encoded) $\rightarrow$ inferred `agm/supabase-credentials`.
   - Contains `accounts` array with `email` and `token` $\rightarrow$ inferred `agm/accounts-export`.
   - Contains `instances` array with `bound_email` $\rightarrow$ inferred `agm/instances-export`.
3. **Format Inspector Output**: When a legacy file is detected, the inspector notes:
   `[Legacy Format Detected -> Auto-compatible, migration recommended]`.

---

## 4. The CLI Contract: `agm which-format` / `agm format inspect`

### 4.1 Invocation Syntax
```bash
# 1. Inspect specific file
agm which-format ./my-config.json

# 2. Inspect multiple files
agm which-format file1.json file2.json file3.json

# 3. Auto-scan directory (scans all *.json in current or target folder)
agm which-format
agm which-format ./configs/
```

### 4.2 Terminal Output Structure
When executed, the command produces 4 distinct sections:

1. **Header & Scan Telemetry**: Path scanned, total files discovered.
2. **Matched Formats Table & Mutation Impact**:
   - File path.
   - Identified data type (`attributes.type`).
   - Summary of changes ("What will be modified on import").
   - Recommended individual import command.
3. **Unrecognized / Invalid Files**:
   - Lists files that do not match any known AGM schema with specific diagnostic reasons.
4. **Single-Line Bulk Execution Command**:
   - Generates the exact command to import all recognized files in one line, with `-y` to bypass prompts:
     `agm import --all file1.json file2.json -y` (or chained `agm ...`).
