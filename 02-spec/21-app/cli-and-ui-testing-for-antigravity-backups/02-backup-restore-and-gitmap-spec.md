# Backup & Restore Protocol and GitMap Delegation Specification

## 1. Metadata & Identification
- **Title**: Backup & Restore Protocol, Positional Argument Normalization, Real-Time Step Tracing, and GitMap Delegation Parity
- **Slug**: cli-and-ui-testing-for-antigravity-backups
- **Spec File**: `02-spec/21-app/cli-and-ui-testing-for-antigravity-backups/02-backup-restore-and-gitmap-spec.md`
- **Author**: @aukgit
- **Created**: 2026-10-20
- **Status**: Draft / Active Specification
- **Target Subsystems**:
  - `src-tauri/src/bin/agm/running_backup_cmds.rs` (Backup/Restore command execution, argument parsing, step tracing)
  - `src-tauri/src/bin/agm/instance_fastforward.rs` (5-stage fast-forward atomic switch lifecycle)
  - `src-tauri/src/bin/agm/misc_cmds.rs` (GitMap `agy` parity delegation)
  - `src-tauri/src/bin/agm/main.rs` (CLI command routing table)
  - `src-tauri/src/modules/backup_prompts_db/` (Split SQLite prompt backup store)
  - `src-tauri/src/modules/repo_db/` (Active prompts FIFO queue & workspace resumption)

---

## 2. Background & User Intent

Users and automated agents interact with Antigravity Manager (`agm`) and GitMap (`gitmap`) interchangeably to inspect, fast-forward, snapshot, and restore running prompts across multi-instance IDE profiles. Prior implementations introduced command syntax friction:
1. Users frequently execute conversational or descriptive commands such as `agm backup running #1`, `agm backup running default`, `agm restore running worker-1`, or `gitmap ag restore rp #1`.
2. The legacy parser in `running_backup_cmds.rs` expected rigid flags (`-i <id>`, `--instance=<id>`), causing positional tokens like `running`, `rp`, `#1`, `default`, or `a-6650` to either be silently dropped or mistaken for unrecognized subcommands.
3. Backup and restore operations lacked granular visibility into individual prompt items, Base64 image payloads, and screenshot attachments as they were transferred into split SQLite databases.
4. GitMap wrappers required uniform delegation parity for fast-forward switching (`gitmap agy ff`), running prompt enumeration (`gitmap agy running`), running prompt backups (`gitmap agy backup rp`), and post-switch prompt re-injection (`gitmap ag restore rp [instance]`).

---

## 3. Core Architectural Principles & Invariants

1. **Strictly Relative Paths Invariant**: All filesystem references across specifications, plans, and runtime JSON envelopes must use repository-relative paths (e.g., `02-spec/...`, `.ai-memory/...`, `scratch/...`).
2. **Caller IDE Self-Preservation**: Fast-forward rotation and instance termination must verify caller identity (`detect_caller_instance_id()`) to safeguard the developer's working IDE session (`default-copy-5597`, PID 11588).
3. **Strict FIFO Queue Invariant**: All prompt backup retrieval and workspace re-injection queries must maintain strict First-In-First-Out ordering via `ORDER BY created_at ASC, id ASC`. LIFO (`ORDER BY updated_at DESC`) is strictly forbidden.
4. **No Silent Errors Rule**: All fallible database operations, process spawns, and filesystem reads must propagate errors through `AppError` or record justified non-fatal operations via `record_ignored`.
5. **Two-Tier JSON Envelope Parity**: CLI commands executed with `--json` must output standardized machine-readable envelopes with metadata, execution timestamps, and structured status reports.

---

## 4. Technical Specification

### 4.1 Positional Instance Argument Resolution & Noise Token Stripping

The command entry points for `agm backup` and `agm restore` must process raw argument vectors through a unified argument normalizer: `parse_instance_and_noise_args(args: &[String])`.

#### 4.1.1 Noise Token Stripping Table
When parsing arguments for `agm backup` or `agm restore`, the following tokens are classified as non-instance noise keywords and stripped from positional arguments:

| Noise Token | Context / Intended Meaning | Processing Rule |
|---|---|---|
| `running` | Indicates active running prompt target | Stripped if not parsed as option value |
| `running-prompts` | Descriptive synonym for running prompts | Stripped |
| `rp` | Common shorthand abbreviation | Stripped |
| `prompts` | Generic prompt domain indicator | Stripped |
| `active` | Synonym for running prompts | Stripped |

#### 4.1.2 Positional Instance Resolution Ladder
Any remaining non-flag, non-noise token is evaluated against the 5-tier instance resolution ladder:

```
[Raw Argument Token]
       │
       ▼
1. Sequence Number Specifier ("#1", "#2", "1", "2")
       │ ──► Parse numeric index, query registry seq_num
       ▼
2. Exact Instance ID Match ("default", "inst-6650", "worker-1")
       │ ──► Query registry where id == specifier (case-insensitive)
       ▼
3. Prefix Stripped Match ("ins-1", "instance-1", "inst-1")
       │ ──► Strip "ins-", "instance-", "inst-", resolve numeric/id match
       ▼
4. Suffix / Suffix-Hyphen Match ("a-6650", "6650", "-6650")
       │ ──► Query registry where id ends with "-6650" or "6650"
       ▼
5. Fallback Resolution
       ├── Backup: Fall back to active instance ID (or "default")
       └── Restore: Fall back to None (scoping restore to ALL instances)
```

#### 4.1.3 Command Grammar Matrix

| User Command | Resolved Target | Operation Scope |
|---|---|---|
| `agm backup` | Active Instance | Snapshot active running prompts for current instance |
| `agm backup running` | Active Instance | Strip `running`, snapshot current instance |
| `agm backup running #1` | Instance `#1` | Strip `running`, resolve `#1` to concrete instance ID |
| `agm backup running worker-1` | `worker-1` | Strip `running`, resolve `worker-1` to concrete instance ID |
| `agm backup running a-6650` | `inst-a-6650` | Strip `running`, match suffix `a-6650` |
| `agm restore` | `None` (All) | Restore unrestored prompts across all instances |
| `agm restore running` | `None` (All) | Strip `running`, restore across all instances |
| `agm restore running #1` | Instance `#1` | Strip `running`, restore exclusively to instance `#1` |
| `agm restore #2` | Instance `#2` | Resolve `#2`, restore exclusively to instance `#2` |
| `agm restore running default` | `default` | Strip `running`, restore exclusively to default instance |

---

### 4.2 Real-Time Multimodal Step Tracing

Both `agm backup running [instance]` and `agm restore running [instance]` must emit granular, real-time trace messages detailing each step of the pipeline. Tracing must clearly differentiate text prompts, embedded Base64 image payloads, and referenced filesystem screenshot attachments.

#### 4.2.1 Real-Time Backup Trace Flow

```
[Step 1/3: Workspace Scan & Discovery]
[*] Target Instance: 'default' (resolved from '#1')
[*] Querying active SQLite prompts and workspace transcripts...
[+] Discovered 2 active prompt turn(s) across 1 project(s).

[Step 2/3: Multimodal Snapshotting & Image Tracing]
  • Prompt #1 [p-091e2cb7] (Project: 'Antigravity-Manager')
    ├─ Text Snippet: "Please confirm that you can sequence the proper tree view..."
    ├─ Attached Images: 2 detected
    │   ├─ Image [img-1]: 142.8 KB (MIME: image/png, embedded Base64 payload)
    │   └─ Sub-Image [img-2]: 68.4 KB (MIME: image/jpeg, disk ref: scratch/screen-01.png)
    └─ Snapshot Status: Secured into memory buffer.
  • Prompt #2 [p-55c24597] (Project: 'Antigravity-Manager')
    ├─ Text Snippet: "agm running --json # list prompts , instances..."
    ├─ Attached Images: 0 detected
    └─ Snapshot Status: Secured into memory buffer.

[Step 3/3: Split SQLite Persistence & Batch Commit]
[*] Writing batch records to split SQLite: 'backup_prompts.db'
[*] Batch ID: 'batch-20261020-023000-8159'
[SUCCESS] Secured 2 prompt(s) (2 image(s)) in split SQLite backup.
```

#### 4.2.2 Real-Time Restore Trace Flow

```
[Step 1/3: Unrestored Batch Discovery]
[*] Target Scope: Instance 'default' (or All Instances if unconstrained)
[*] Querying unrestored batches in split SQLite: 'backup_prompts.db'
[+] Discovered 1 pending batch(es) with 2 unrestored prompt turn(s).

[Step 2/3: FIFO Unpack & Multimodal Reconstruction]
  • Restoring Prompt #1 [p-091e2cb7] (Created: 2026-10-20 02:24:41 UTC)
    ├─ FIFO Sequence: Turn #1 of 2
    ├─ Restoring 2 attached image payload(s)...
    │   ├─ Image [img-1]: Reconstituted Base64 payload (142.8 KB)
    │   └─ Sub-Image [img-2]: Re-anchored workspace reference: 'scratch/screen-01.png'
    └─ Seeding '.antigravity_resume_task.default.json' workspace resumption state.
  • Restoring Prompt #2 [p-55c24597] (Created: 2026-10-20 02:24:55 UTC)
    ├─ FIFO Sequence: Turn #2 of 2
    ├─ Attached Images: None
    └─ Seeding '.antigravity_resume_task.default.json' workspace resumption state.

[Step 3/3: Active Execution Queue Re-Injection]
[*] Re-inserting 2 prompt(s) into active_prompts FIFO execution table...
[*] Marking batch 'batch-20261020-023000-8159' as restored (1-day retention initiated).
[SUCCESS] Restored and re-enqueued 2 prompt(s) for instance 'default'.
```

---

### 4.3 GitMap Delegation Parity & CLI Routing Matrix

GitMap delegates Antigravity-specific commands directly to `agm` via child process dispatch or internal library calls. To maintain absolute parity across both terminal environments, GitMap commands must map 1:1 with AGM commands:

```
┌──────────────────────────────────────┐       ┌──────────────────────────────────────┐
│            GitMap CLI                │       │               AGM CLI                │
├──────────────────────────────────────┤       ├──────────────────────────────────────┤
│ gitmap agy running [--json]          │ ────► │ agm running [--json]                 │
│ gitmap agy ff [instance]             │ ────► │ agm ff [instance]                    │
│ gitmap agy backup rp [instance]      │ ────► │ agm backup running [instance]        │
│ gitmap ag restore rp [instance]      │ ────► │ agm restore running [instance]       │
│ gitmap agy backup running-prompts    │ ────► │ agm backup running                   │
│ gitmap ag restore running-prompts    │ ────► │ agm restore running                  │
└──────────────────────────────────────┘       └──────────────────────────────────────┘
```

#### 4.3.1 Command Equivalence & Behavior Rules

1. **`gitmap agy running [--json]`**:
   - Executes `agm running [--json]`.
   - Enumerates instances, sequence numbers (`#1`, `#2`), project sequences (`[AGM:P001 | GM:#1]`), conversation IDs (`[AGM:C001 | GM:<cid>]`), running prompt previews (100–200 words), and attached image counts (`img_count: N`).
   - If `--json` is supplied, returns a JSON array of instances with active prompts.

2. **`gitmap agy ff [instance]`**:
   - Executes `agm ff [instance]`.
   - Invokes the 5-stage fast-forward atomic switch lifecycle on the target instance (defaulting to the active instance if omitted).
   - Preserves the caller developer instance session.

3. **`gitmap agy backup rp [instance]` / `gitmap agy backup running-prompts`**:
   - Executes `agm backup running [instance]`.
   - Strips `rp` / `running-prompts` noise tokens and maps `[instance]` through `parse_instance_and_noise_args`.
   - Outputs step-by-step progress logging with image and sub-image counts.

4. **`gitmap ag restore rp [instance]` / `gitmap ag restore running-prompts [instance]`**:
   - Executes `agm restore running [instance]`.
   - If `[instance]` is provided, restricts restore to that specific instance; otherwise restores all unrestored batches.
   - Re-injects prompts into the active queue and updates workspace task resumption markers.

---

### 4.4 5-Stage Fast-Forward Atomic Switch Lifecycle

The Fast-Forward (`agm ff` / `gitmap agy ff`) command provides zero-downtime, lossless account switching by executing an atomic 5-stage pipeline:

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                 5-STAGE FAST-FORWARD ATOMIC SWITCH LIFECYCLE                │
├─────────────────────────────────────────────────────────────────────────────┤
│                                                                             │
│  STAGE 1: Pre-Flight Quota Verification & In-Flight Discovery               │
│  ├── Query Google OAuth quota status for candidate accounts                 │
│  ├── Rank accounts by rolling 5-hour and 7-day weekly availability           │
│  ├── Check Supabase and Email collision locks                               │
│  └── Detect in-flight prompts, Base64 images, and active PIDs               │
│                                                                             │
│  STAGE 2: Multimodal Running Prompt Snapshotting                            │
│  ├── Capture running prompts from active_prompts and transcripts            │
│  ├── Snapshot attached Base64 image payloads and disk screenshot references  │
│  ├── Persist batch into split SQLite backup database (backup_prompts.db)    │
│  └── Write atomic workspace resume file (.antigravity_resume_task.<id>.json)│
│                                                                             │
│  STAGE 3: Conscious Account Rotation & Keyring Swap                         │
│  ├── Apply Caller IDE Self-Preservation (guard default-copy-5597 / PID)     │
│  ├── Terminate target instance IDE process cleanly (SIGTERM / Win32 close)  │
│  ├── Inject fresh OAuth credentials and refresh token into state.vscdb      │
│  └── Re-launch target instance with updated workspace session               │
│                                                                             │
│  STAGE 4: Post-Switch Re-Injection & FIFO Unpack                            │
│  ├── Wait for IDE instance window / prompt channel availability             │
│  ├── Unpack backed-up prompts from split SQLite in strict FIFO order         │
│  ├── Re-inject Base64 images and attachments into active_prompts table       │
│  └── Dispatch restored turns to active workspace prompts                    │
│                                                                             │
│  STAGE 5: Multi-Channel Telemetry & Heartbeat Verification                  │
│  ├── Verify prompt goal heartbeat (.antigravity_goal_prompt.log)            │
│  ├── Record audit trail event in task_history_db                            │
│  ├── Broadcast structured telemetry alert to Telegram & Email               │
│  └── Output structured completion confirmation (CLI / JSON)                 │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
```

#### Detailed Stage Specifications:

- **Stage 1 (Pre-Flight Discovery)**:
  - Candidates are filtered by removing the current active account and accounts with active collision locks from other fleet nodes.
  - Quota is verified via Google Cloud API endpoints; only profiles exceeding the high-watermark threshold (e.g. 15.0% minimum, prioritizing 100%) are considered.
  - Active prompts are inventoried along with their image payload flags.

- **Stage 2 (Multimodal Snapshotting)**:
  - Calls `repo_db::backup_running_prompts(&instance_id)` and `backup_prompts_db::backup_active_running_prompts(Some(&instance_id), None)`.
  - All prompt records include full text, session IDs, project paths, and complete image payloads.
  - Writes a localized `.antigravity_resume_task.<instance_id>.json` file in the workspace directory.

- **Stage 3 (Conscious Rotation & Keyring Swap)**:
  - Verifies target instance is not the caller session via `refuse_if_caller_instance`.
  - Terminates only targeted instance worker PIDs.
  - Injects new credentials into the instance's `state.vscdb` database (`ItemTable`).

- **Stage 4 (Post-Switch Re-Injection & FIFO Unpack)**:
  - Waits up to 10 seconds for the instance process to settle.
  - Reads backup records in strict FIFO order (`ORDER BY created_at ASC, id ASC`).
  - Calls `repo_db::resend_running_commands_for_instance` and `repo_db::dispatch_running_prompts`.

- **Stage 5 (Multi-Channel Telemetry & Verification)**:
  - Confirms prompt execution vitality via `inspect_prompt_goal_status`.
  - Emits telemetry payload containing `previous_account`, `selected_account`, `predicted_next_account`, `prompts_running`, `prompts_resent`, and `has_images`.

---

## 5. Verification & Acceptance Criteria Matrix

| Item | Requirement | Verification Method | Pass/Fail Criteria |
|---|---|---|---|
| **V1** | Positional Instance Resolution | `agm backup running #1`, `agm backup running worker-1`, `agm backup running a-6650` | Correctly resolves instance ID without requiring `-i` or `--instance` flags |
| **V2** | Noise Token Stripping | `agm backup running`, `agm backup rp`, `agm restore running` | Noise tokens stripped cleanly without failing argument validation |
| **V3** | Real-Time Step Tracing | CLI execution output inspection | Output displays Step 1/3, Step 2/3, Step 3/3 with prompt IDs and image counts |
| **V4** | Multimodal Image Tracing | Prompts with Base64 payloads and disk screenshot files | Traces byte size, MIME type, and count of images/sub-images per prompt |
| **V5** | GitMap Delegation Parity | `gitmap agy running`, `gitmap agy ff`, `gitmap agy backup rp`, `gitmap ag restore rp` | Executes identical underlying AGM routines and produces consistent outputs |
| **V6** | 5-Stage Fast-Forward Lifecycle | `agm ff <inst>` / `gitmap agy ff <inst>` | Executes Stages 1–5 sequentially, preserving prompts and caller IDE session |
| **V7** | Strict FIFO Invariant | Prompt restore from backup database | Prompts restored strictly ordered by `created_at ASC, id ASC` |
