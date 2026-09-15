# Task Retention, Line Streaming, Atomic File Writes & Extensible ApiManager

**Date:** 2026-09-09
**Status:** Learned & Consolidated
**Scope:** `04-code/golang/pkg/sqlitelogger`, `04-code/golang/pkg/errcmd`, `04-code/golang/pkg/fileutil`, `04-code/golang/pkg/lazyonce`, `02-spec/21-app/`

---

## 1. Overview of 4-Part Architectural Enhancement

Across recent turns, four core infrastructure enhancements and an extensible API management architecture were specified and verified:

1. **Task Retention & Query Filtering (`sqlitelogger`):** Automatic pruning of stale task DBs and rich multi-parameter log queries.
2. **Live Line Streaming & Process Context (`errcmd`):** Real-time stdout/stderr line interception with configurable cwd and environment.
3. **Atomic File Writes (`fileutil`):** Crash-resilient file persistence avoiding partial write corruption.
4. **Reset & Context Support (`lazyonce`):** Invalidation and cancellation awareness for singleton initialization.
5. **Extensible `ApiManager` Architecture:** Enterprise log shipping to remote REST/gRPC endpoints with dynamic rotation policies.

---

## 2. Component Specifications

### 2.1 Task Retention & Query Filtering (`sqlitelogger`)
- **Pruning API:** `PruneTasks(olderThan time.Duration, keepLast int) *appfault.AppError` removes task database files older than the specified duration while preserving the N most recent task databases.
- **Query Filter:** `TaskLogQuery` struct supporting `TaskId`, `MinLevel`, `StartTime`, `EndTime`, `SearchText`, `Limit`, and `Offset`.
- **Zero Raw SQL Injection:** All queries use parameterized statements with verified indices on `(TaskId, Timestamp)`.

### 2.2 Live Line Streaming & Context (`errcmd`)
- **Streaming Hooks:** `WithStdoutHandler(func(line string))` and `WithStderrHandler(func(line string))` allow streaming process output line-by-line in real time to terminal UI, websockets, or log collectors.
- **Execution Options:** `WithCwd(dir string)` and `WithEnv(env []string)` allow hermetic execution environments.
- **Context Cancellation:** Clean signal propagation (`SIGTERM`, followed by graceful `SIGKILL` on timeout).

### 2.3 Atomic File Writes (`fileutil`)
- **API:** `AtomicWriteFile(filePath string, data []byte, perm os.FileMode) *appfault.AppError`.
- **Protocol:**
  1. Write to a temporary file in the *same* filesystem directory (`<target>.tmp.<pid>.<nanos>`).
  2. Flush file buffers to physical storage via `file.Sync()`.
  3. Close file handle cleanly.
  4. Perform atomic replacement via `os.Rename(tmpPath, filePath)`.
  5. Clean up temporary file on failure using deferred cleanup.

### 2.4 LazyOnce Reset & Context Support (`lazyonce`)
- **Reset Capability:** `Reset()` allows clearing the cached instance and execution flag under mutex protection, enabling hot-reloading of configuration or testing resets.
- **Context Support:** `DoWithContext(ctx context.Context, fn func() (T, error)) (T, error)` checks context deadline before and during initialization, immediately returning context errors without caching failures.

---

## 3. Extensible `ApiManager` Specification

For remote logging and metrics aggregation:
- **Batching & Buffering:** In-memory queue with max batch size (e.g. 100 entries) and max flush interval (e.g. 5 seconds).
- **Pluggable Shippers:** Interface `LogShipper` defines `ShipLogs(ctx context.Context, batch []LogEntry) *appfault.AppError`.
- **Fault-Tolerant Delivery:** Exponential backoff retry with jitter, saving undeliverable entries to local spillover SQLite disk storage until network recovery.
- **Extensible Rotation & Hooking:** Pluggable lifecycle hooks (`OnBeforeRotate`, `OnAfterRotate`, `OnShipFailure`) allowing runtime reconfiguration of destination endpoints or auth tokens.
