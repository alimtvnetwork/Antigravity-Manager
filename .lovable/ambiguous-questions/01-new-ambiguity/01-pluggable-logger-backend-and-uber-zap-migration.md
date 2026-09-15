# Unified Pluggable Logger Architecture and Uber Zap Migration

Slug: pluggable-logger-backend-and-uber-zap-migration
Status: open
Raised: 2026-09-03
Blocking: 04-code/golang/pkg/applogger refactoring

## Question

How should `04-code/golang/pkg/applogger` be refactored to support seamless hot-swapping between internal Formatter+Writer pipelines and external enterprise loggers like Uber Zap (`go.uber.org/zap`), while maintaining native Go `context.Context` tracing and `*appfault.AppError` integration?

## Problem & Missing Capabilities in Current Code

1. **Coupled Sinks:** In `pkg/applogger`, each sink (`ConsoleSink`, `FileSink`, `SQLiteSink`) couples formatting (JSON vs text string interpolation) directly with destination I/O. There is no isolated `Formatter` or `Writer` interface.
2. **Missing Uber Zap Interoperability:** The existing `ZapAdapter` only supports untyped `fmt.Sprint` logging without passing through structured fields, Zap typed cores, or Go `context.Context`.
3. **No Dynamic Swapper (Log Changer):** No unified manager exists to switch the active logging engine or redirect outputs at runtime without rebuilding the application.
4. **Proprietary Context:** Uses custom `appfault.ContextMap` rather than standard Go `context.Context`, making distributed tracing (`traceId`, `spanId`) difficult to propagate across HTTP handlers and gRPC boundaries.

## Options Considered

### Option A: Standard `log/slog` Handler Adapter (Idiomatic Go 1.21+)
- **Architecture:** Adopt Go's native `slog.Logger` as the foundation. Implement custom `slog.Handler` wrappers for context extraction, and use `zapslog` (or `zapcore`) as a swappable handler.
- **Pros:** Zero third-party dependencies for standard use; native Go standard library compatibility; ecosystem tools natively integrate.
- **Cons:** Uber Zap features (like zero-alloc field encoders) are adapted to `slog.Record` instead of native `zap.Field`.

### Option B: Unified Dual-Mode `LogBackend` Engine (Recommended)
- **Architecture:** Define a universal `LogBackend` interface. Provide two primary backend drivers:
  1. `PipelineBackend`: Decoupled `Formatter` (`JSON`, `Console`, `Logfmt`) + `io.Writer` (`Stdout`, `File`, `MultiWriter`).
  2. `ZapBackend`: Wraps `*zap.Logger` directly, mapping `context.Context` to native `zap.Field` slices.
  Provide a thread-safe `LoggerManager` ("Log Changer") that allows calling `logger.SetBackend(newBackend)`, `logger.SetFormatter(f)`, or `logger.SetWriter(w)` dynamically.
- **Pros:** Full access to native Uber Zap performance when needed; lightweight zero-dependency pipeline for CLI/embedded modes; zero downtime hot-swapping.
- **Cons:** Requires maintaining the lightweight pipeline alongside the Zap adapter.

### Option C: Uber Zap as Exclusive Engine
- **Architecture:** Make `go.uber.org/zap` the sole logging engine across all services, wrapping it in an `AppError` bridge.
- **Pros:** One single logging engine; highest possible throughput.
- **Cons:** Heavy external dependency for lightweight tools, CLI scripts, and embedded packages.

## Impact if Guessed Wrong

- Forcing Uber Zap everywhere adds heavy dependencies to lightweight CLIs.
- Keeping the current hardcoded sink structure prevents teams from redirecting logs or changing formats in enterprise cloud deployments.
- Omitting `context.Context` breaks OpenTelemetry tracing across microservices.

## Next Steps for User Review

Review the concrete implementation code in:
- `.lovable/memory/learned/02-logger-swapping-and-uber-zap-architecture.md`
- `.lovable/suggestions/03-pluggable-context-logger-architectures.md`
