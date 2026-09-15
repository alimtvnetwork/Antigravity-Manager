# Current Plan

**Version:** 5.19.0 (Phase 12.6 — Spec slot 24 reserved with `25-threat-model.md` stub; Backup System initiative complete)
**Updated:** 2026-05-06

---

## Active Initiative — Backup Nodes, Cascading Roles, DB Convention Overhaul

**Mode:** SPEC ONLY. No implementation code. Per user verbatim §Important.5, "No implementation right now. Job is to write the spec only."

**Scope folders:**
- `02-spec/19-main-worker-service/` — primary
- `02-spec/04-database-conventions/` — DateTime INTEGER, enum tables Id/Code/Label
- `02-spec/05-split-db-architecture/` — DateTime INTEGER propagation
- `02-spec/19-main-worker-service/diagrams/` — ER diagram updates

**Stack-agnostic.** Default examples in Laravel/PHP per existing spec.

---

## Locked Decisions (from user verbatim)

| # | Decision |
|---|----------|
| D1 | `EnumPage` is renamed to **`AccessItem`**. Columns include `Label` and `PageUrlSuffix` (matcher). |
| D2 | DateTime columns are **INTEGER (epoch seconds)**, never TEXT. Applies to ALL specs. |
| D3 | Enum-like ref tables use exactly **`Id`, `Code`, `Label`** (drop `{TableName}Code` prefix style for these). |
| D4 | `Company` table uses **`Slug` and `Name`** only. Drop `CompanyName`. |
| D5 | **Main node holds NO Users.** Main holds Company + Company→Worker mapping only. Users live on Worker. |
| D6 | Worker nodes have a **`Sequence` INTEGER** column. |
| D7 | Worker UI label is **"Region"**. Code stays "Worker". |
| D8 | Worker nodes have **`IsBackup` BOOLEAN** and **`BackupOfWorkerNodeId` FK NULL**. |
| D9 | Backup nodes **never serve traffic**. Receive + store only. |
| D10 | Synced rows carry a **`SyncOp` flag** (`Insert` / `Update` / `Delete`) for diff application. |
| D11 | Cascading roles = **union of AccessItems across all assigned roles**. User can hold multiple roles simultaneously. |
| D12 | Add **cache-bin** tables/notes for role management in the ER diagram. |
| D13 | Encryption: shared internal RSA key pair between worker and its backup; main can issue **rotation instruction**. Zip password follows known pattern. |
| D14 | Date-by-date full snapshot storage on backup node. Restore by date is main-controlled. |
| D15 | **Cascading semantics is simple union (locked 2026-07-19).** No role hierarchy. Every effective grant traces to exactly one `RoleAccessItem` row. Supersedes OQ-A1. See `19-main-worker-service/18-cascading-roles-and-cache-bin.md` v1.1.0 §1 + §7. |
| D16 | **Cache-bin storage is per-process SQLite `:memory:` (locked 2026-07-19).** Redis and plain in-process map remain configurable alternatives against the same contract (four functions in §4 + invalidation endpoint in §5); they are not defaults. Supersedes OQ-A2. See `19-main-worker-service/18-cascading-roles-and-cache-bin.md` v1.1.0 §4 + §7. |

---

## Open Questions

All prior open questions (OQ-A1..A4) have been resolved and promoted to locked decisions D15, D16 (2026-07-19) and D-entries from Phases 8/11. See the Locked Decisions table above. No open questions remain.

---

## Phased Plan (12 phases, one per `next`)

### Phase 1 — Rename EnumPage → AccessItem

- Rename across `08-role-based-dashboards.md`, `04-main-db-schema.md`, `15-rbac-and-status-seed.md`, ERD.
- Define `AccessItem` columns: `AccessItemId`, `Code`, `Label`, `PageUrlSuffix`, `Description`.
- Define matcher logic (suffix match against route).
- Update all references; add migration note in `98-changelog.md`.

### Phase 2 — Global DB Convention Updates

- `02-spec/04-database-conventions/03-naming-conventions.md` — DateTime = INTEGER (epoch seconds, UTC).
- `02-spec/04-database-conventions/03-schema-design.md` — enum tables shape `Id/Code/Label`.
- `02-spec/05-split-db-architecture/` — propagate INTEGER DateTime convention.
- `02-spec/19-main-worker-service/04-main-db-schema.md` — flip all `*At TEXT` → `*At INTEGER`.
- Update `Company` to `(CompanyId, Slug, Name, ...)`.

### Phase 3 — Move Users off Main

- `04-main-db-schema.md` — remove `User`, `UserRole`, TOTP columns from Main.
- Document Users now live on Worker split-DB.
- Update `06-auth-and-2fa.md` — auth lookup flow becomes: Main resolves Company→Worker, Worker authenticates User.
- Update `12-split-db-tier-reconciliation.md` — move User/UserRole to Worker App tier.

### Phase 4 — Worker Node Field Additions

- Add `Sequence INTEGER NOT NULL` to `WorkerNode`.
- Add `IsBackup INTEGER NOT NULL DEFAULT 0` (boolean).
- Add `BackupOfWorkerNodeId INTEGER NULL` FK self-ref.
- Add UI mapping note: "Worker" → "Region" (frontend label).
- Update `05-worker-routing.md` — backup nodes excluded from selection pool.

### Phase 5 — Role / AccessItem N-M + Cache Bin + Cascading

- Document `UserRole` (N-M), `RoleAccessItem` (N-M).
- Define cache-bin tables: `RoleAccessCache` (per-role compiled access set, TTL).
- Cascading rules section: union semantics, multi-role behavior, examples (Admin alone, Editor alone, Admin+Editor).
- Cache invalidation protocol from Main.

### Phase 6 — Backup Node Concept Spec

- New file: `17-backup-nodes.md`.
- Sections: relationship model, registration flow, propagation Main→Worker→Worker-DB, "no serving traffic" invariant.
- ER additions to `WorkerNode` table (already in Phase 4).
- Worker-side mirror table `KnownBackupNode` (in Worker DB).

### Phase 7 — Incremental Diff Generator

- New file: `18-incremental-backup-sync.md`.
- Sections: cron schedule, last-sync watermark table, per-table walk, change-data-capture using `SyncOp` flag column on each app row.
- Watermark table schema in Worker DB.

### Phase 8 — Encryption + Zip Pipeline

- New file: `19-backup-encryption-and-keys.md`.
- RSA key pair shared between worker and its backups.
- Zip password derivation pattern.
- Key rotation instruction issued from Main: endpoint contract + state machine (Pending / Active / Retired).

### Phase 9 — Backup Endpoints Contract

- New file: `20-backup-endpoints.md`.
- Endpoints:
  1. `POST /API/V1/Backup/IncrementalDiff` (on backup node)
  2. `POST /API/V1/Backup/RotateKeys` (on backup node, Main-triggered)
  3. `POST /API/V1/Backup/RestoreByDate` (on backup node, Main-triggered)
  4. `GET  /API/V1/Backup/Snapshots` (list available dates)
  5. `GET  /API/V1/Backup/Health` (status)
- Auth: S2S OAuth (per `06-auth-and-2fa.md`).
- Add error codes in `14-error-codes.md`.

### Phase 10 — Backup Apply Logic

- Append to `20-backup-endpoints.md` or new `21-backup-apply-logic.md`.
- Decrypt → unzip → open SQLite diff → iterate rows → apply by `SyncOp` flag → idempotency via `(SourceTable, SourceRowId, SyncOpSeq)`.

### Phase 11 — Snapshot Storage + Restore Flow

- New file: `22-snapshot-storage-and-restore.md`.
- Date-named full DB zips on backup node filesystem.
- Retention policy (default 30 days).
- Restore flow: Main → Backup `RestoreByDate` → Backup decompresses → Worker pulls or Backup pushes to Worker.

### Phase 12 — ER Diagram + Acceptance Criteria + Sync

- Update `diagrams/erd-main-db.mmd`: AccessItem rename, Users removed, WorkerNode new fields, cache-bin notation, INTEGER DateTime.
- New diagram: `diagrams/erd-worker-db.mmd` (Users + KnownBackupNode + watermark).
- New diagram: `diagrams/seq-backup-incremental.mmd`.
- New diagram: `diagrams/seq-backup-restore.mmd`.
- Update `97-acceptance-criteria.md` with one AC per phase.
- Bump `package.json` 5.12.0 → 5.13.0.
- Run `npm run sync`.

---

## Status

- [x] **Phase 0** — Plan written.
- [x] **Phase 1** — `EnumPage` → `AccessItem` rename.
- [x] **Phase 2** — DB convention overhaul.
- [x] **Phase 3** — Users moved off Main; credential-blind proxy.
- [x] **Phase 4** — `WorkerNode` `Sequence` / `IsBackup` / `BackupOfWorkerNodeId`; "Region" UI label.
- [x] **Phase 5** — `18-cascading-roles-and-cache-bin.md` v1.0.0; OQ-A1/A2 default proposals adopted.
- [x] **Phase 6** — `19-backup-nodes.md` v1.0.0; `MAIN-800-*` lifecycle codes; status seed extended.
- [x] **Phase 7** — `20-incremental-backup-sync.md` v1.0.0 (two `SyncOp` shapes, `BackupSyncWatermark`, envelope = SQLite file, compaction safety rule); `WORKER-910-*` + `MAIN-810-01` codes; tunables §2.11 extended.
- [x] **Phase 8** — `21-backup-encryption-and-keys.md` v1.0.0 (Pair-RSA + Envelope-AES + HKDF zip password resolving OQ-A3, four-state rotation, `BackupKeyEpoch` table); `WORKER-920-01..05` + `MAIN-820-01..03` codes; tunables §2.12 added.
- [x] **Phase 9** — `22-backup-endpoints.md` v1.0.0 (5 S2S OAuth endpoints: IncrementalDiff/RotateKeys/RestoreByDate/Snapshots/Health; endpoint↔scope matrix; CODE-RED handler budgets); `MAIN-830-01..02` wire codes; tunables §2.13 added.
- [x] **Phase 10** — `23-backup-apply-logic.md` v1.0.0 (5-stage pipeline, V1–V7 validation, single-TX dispatch, DLQ no-silent-skip, V7 idempotency via UNIQUE lock); `BackupApplyIdempotency` + `BackupApplyDeadLetter` tables; `WORKER-930-01..04` opening overflow `WORKER-21200-21299` + `MAIN-840-01`; tunables §2.14 added.
- [x] **Phase 11** — `24-snapshot-storage-and-restore.md` v1.0.0 (3-moment lifecycle, 8-step Build using `sqlite3_backup_init`, separate HKDF salt for snapshot password, 8-step Restore re-sealing under current Active KeyEpoch); BE-6 RestoreInbox on primary; `BackupSnapshotCatalog` + `BackupSnapshotJob` + `BackupRestoreJob` tables; final `Backup` S2S audience wiring with mandatory `PairingId` JWT claim; `WORKER-940-01..04` + `MAIN-840-02`; tunables §2.15 added — **OQ-A4 resolved at 30 days rolling**.
- [x] **Phase 12** — Final consolidation: 3 new diagrams (`erd-backup-tier.mmd`, `seq-incremental-backup.mmd`, `seq-backup-restore.mmd`); diagrams index → v1.1.0; `97-acceptance-criteria.md` → v1.1.0 with 13 new Backup-tier criteria; cross-spec stubs noted (auth `Backup` audience, `PairingId` JWT claim, BE-1..BE-6 endpoint catalogue merge); linter `BACKUP-*` / `DB-SYNCOP-*` promoted; `AppBackupTrackedTable` seed referenced; **version bumped to `5.13.0` (final, phase suffix removed)**. Backup System spec arc COMPLETE.

> **File numbering:** Phase 5 took the `17-…` slot, so backup-nodes is `18-…`, incremental sync `19-…`, encryption `20-…`, endpoints `21-…`, apply logic `22-…`, snapshot/restore `23-…`. Diagrams folder unchanged.

---

## Cross-spec impact summary

| Spec | Impacted by Phases |
|------|-------------------|
| `04-database-conventions/` | 2 |
| `05-split-db-architecture/` | 2, 3 |
| `19-main-worker-service/04-main-db-schema.md` | 1, 2, 3, 4, 5 |
| `19-main-worker-service/06-auth-and-2fa.md` | 3 |
| `19-main-worker-service/08-role-based-dashboards.md` | 1, 5 |
| `19-main-worker-service/12-split-db-tier-reconciliation.md` | 3 |
| `19-main-worker-service/14-error-codes.md` | 9 |
| `19-main-worker-service/15-rbac-and-status-seed.md` | 1, 5 |
| `19-main-worker-service/diagrams/` | 12 |
| New files: `17-…` through `22-…` | 6–11 |

---

## Phase 13 — Codebase Audit, Query Wrappers & Enum Fixes

- ✅ **COMPLETED**. See `## Completed` section.

---

*Plan v2.1.0 — 2026-08-09. Type `next` to begin Phase 14.*

---

## Completed

- [x] **Phase 13** - Code Red Enum and Boolean Refactor
  - Enforce Enum Type suffix, eliminate string unions.
  - Implement strictly typed query wrappers for PHP, TS, Py.
