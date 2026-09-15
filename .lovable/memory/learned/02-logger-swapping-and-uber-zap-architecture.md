# Memory: Pluggable Logger Architecture, Log Changer, and Uber Zap Migration

> Status: Active Proposal & Reference Architecture
> Ingested: 2026-09-03
> Category: `pkg/applogger` Core Architecture
> Related Ambiguity: `.lovable/ambiguous-questions/01-new-ambiguity/01-pluggable-logger-backend-and-uber-zap-migration.md`

---

## 1. Architectural Overview: The Log Changer Pattern

To address the limitations of hardcoded sinks, this architecture decouples the logger into three independent layers:
1. **The Log Changer (Unified Manager):** Thread-safe controller that allows changing the active backend, formatter, or writer at runtime without restarting the application.
2. **Pluggable Formatter & Writer Pipeline:** For lightweight, zero-dependency environments (Console text, JSON, Files, Multi-writers).
3. **Pluggable Enterprise Backends (Uber Zap):** High-throughput structured logging delegating directly to `go.uber.org/zap`, mapping `context.Context` to native `zap.Field` attributes.

```
                     ┌──────────────────────────────────────┐
                     │            applogger.Logger          │
                     │  (InfoContext, ErrorContext, etc.)   │
                     └──────────────────┬───────────────────┘
                                        │ Delegates to
                                        ▼
                     ┌──────────────────────────────────────┐
                     │         LogBackend (Interface)       │
                     └──────────┬────────────────┬──────────┘
                                │                │
             ┌──────────────────┘                └──────────────────┐
             ▼                                                      ▼
┌───────────────────────────┐                         ┌───────────────────────────┐
│      PipelineBackend      │                         │        ZapBackend         │
│  (Zero-dependency Engine) │                         │  (Uber Zap *zap.Logger)   │
├───────────────────────────┤                         ├───────────────────────────┤
│ Formatter: Console / JSON │                         │ Core: Zap Core Encoders   │
│ Writer: Stdout / File     │                         │ Sinks: Zap WriteSyncer    │
└───────────────────────────┘                         └───────────────────────────┘
```

---

## 2. Core Contracts & Interfaces

### 2.1 Backend Contract
```go
package applogger

import (
	"context"
	"time"

	"coding-guidelines/common/pkg/appfault"
)

// LogLevel defines standardized severity tiers.
type LogLevel int

const (
	LevelDebug LogLevel = iota
	LevelInfo
	LevelWarn
	LevelError
	LevelFatal
)

// LogRecord carries normalized event data across backends.
type LogRecord struct {
	Timestamp time.Time
	Level     LogLevel
	Message   string
	Context   context.Context
	Fields    map[string]any
	AppFault  *appfault.AppError
}

// LogBackend is the pluggable engine interface.
type LogBackend interface {
	Log(record LogRecord) error
	Sync() error
	Close() error
}
```

### 2.2 Formatter & Writer Contracts (Pipeline Mode)
```go
import "io"

// Formatter serializes a LogRecord into bytes.
type Formatter interface {
	Format(record LogRecord) ([]byte, error)
}

// Writer is an alias to standard io.Writer.
type Writer = io.Writer
```

---

## 3. Concrete Implementation: Formatter and Writer Pipeline

### 3.1 Formatters
```go
import (
	"encoding/json"
	"fmt"
)

// JSONFormatter outputs newline-delimited JSON.
type JSONFormatter struct{}

func (f *JSONFormatter) Format(r LogRecord) ([]byte, error) {
	payload := map[string]any{
		"timestamp": r.Timestamp.Format(time.RFC3339),
		"level":     r.Level.String(),
		"message":   r.Message,
	}

	if len(r.Fields) > 0 {
		payload["fields"] = r.Fields
	}

	// Extract standard tracing from context.Context
	if r.Context != nil {
		if traceId, isOk := r.Context.Value("traceId").(string); isOk {
			payload["traceId"] = traceId
		}
		if userId, isOk := r.Context.Value("userId").(string); isOk {
			payload["userId"] = userId
		}
	}

	if r.AppFault != nil {
		payload["errorCode"] = r.AppFault.Type().Code()
		payload["errorType"] = r.AppFault.Type().Name()
	}

	bytes, err := json.Marshal(payload)
	if err != nil {
		return nil, err
	}

	return append(bytes, '\n'), nil
}

// ConsoleFormatter outputs human-readable terminal text.
type ConsoleFormatter struct{}

func (f *ConsoleFormatter) Format(r LogRecord) ([]byte, error) {
	ctxInfo := ""
	if r.Context != nil {
		if traceId, isOk := r.Context.Value("traceId").(string); isOk {
			ctxInfo = fmt.Sprintf(" [trace=%s]", traceId)
		}
	}

	line := fmt.Sprintf("[%s] %-5s %s%s\n",
		r.Timestamp.Format("15:04:05"),
		r.Level.String(),
		r.Message,
		ctxInfo,
	)

	return []byte(line), nil
}
```

### 3.2 PipelineBackend Implementation
```go
import "sync"

// PipelineBackend combines a Formatter and an io.Writer.
type PipelineBackend struct {
	mu        sync.RWMutex
	formatter Formatter
	writer    Writer
}

func NewPipelineBackend(f Formatter, w Writer) *PipelineBackend {
	return &PipelineBackend{
		formatter: f,
		writer:    w,
	}
}

func (p *PipelineBackend) SetFormatter(f Formatter) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.formatter = f
}

func (p *PipelineBackend) SetWriter(w Writer) {
	p.mu.Lock()
	defer p.mu.Unlock()
	p.writer = w
}

func (p *PipelineBackend) Log(r LogRecord) error {
	p.mu.RLock()
	formatter := p.formatter
	writer := p.writer
	p.mu.RUnlock()

	bytes, err := formatter.Format(r)
	if err != nil {
		return err
	}

	_, err = writer.Write(bytes)
	return err
}

func (p *PipelineBackend) Sync() error {
	if syncer, isOk := p.writer.(interface{ Sync() error }); isOk {
		return syncer.Sync()
	}
	return nil
}

func (p *PipelineBackend) Close() error {
	if closer, isOk := p.writer.(io.Closer); isOk {
		return closer.Close()
	}
	return nil
}
```

---

## 4. Concrete Implementation: Uber Zap Backend

This backend delegates directly to `go.uber.org/zap`, mapping `context.Context` and `*appfault.AppError` into native `zap.Field` slices for zero-allocation performance.

```go
package applogger

import (
	"go.uber.org/zap"
	"go.uber.org/zap/zapcore"
)

// ZapBackend adapts an official Uber Zap Logger to LogBackend.
type ZapBackend struct {
	logger *zap.Logger
}

// NewZapBackend creates a Zap adapter wrapping *zap.Logger.
func NewZapBackend(zapLog *zap.Logger) *ZapBackend {
	return &ZapBackend{logger: zapLog}
}

// NewProductionZapBackend creates an optimized JSON Zap backend.
func NewProductionZapBackend() (*ZapBackend, error) {
	cfg := zap.NewProductionConfig()
	cfg.EncoderConfig.EncodeTime = zapcore.ISO8601TimeEncoder
	zapLog, err := cfg.Build()
	if err != nil {
		return nil, err
	}

	return NewZapBackend(zapLog), nil
}

// Log handles dispatching to the native Zap Core.
func (zb *ZapBackend) Log(r LogRecord) error {
	fields := make([]zap.Field, 0, len(r.Fields)+4)

	// Map application custom fields
	for k, v := range r.Fields {
		fields = append(fields, zap.Any(k, v))
	}

	// Map context.Context attributes to native Zap fields
	if r.Context != nil {
		if traceId, isOk := r.Context.Value("traceId").(string); isOk {
			fields = append(fields, zap.String("traceId", traceId))
		}
		if userId, isOk := r.Context.Value("userId").(string); isOk {
			fields = append(fields, zap.String("userId", userId))
		}
		if spanId, isOk := r.Context.Value("spanId").(string); isOk {
			fields = append(fields, zap.String("spanId", spanId))
		}
	}

	// Map structured AppError
	if r.AppFault != nil {
		fields = append(fields,
			zap.String("errorCode", r.AppFault.Type().Code()),
			zap.String("errorType", r.AppFault.Type().Name()),
			zap.String("stackTrace", r.AppFault.StackTrace().String()),
		)
	}

	// Route to Zap according to severity level
	switch r.Level {
	case LevelDebug:
		zb.logger.Debug(r.Message, fields...)
	case LevelWarn:
		zb.logger.Warn(r.Message, fields...)
	case LevelError:
		zb.logger.Error(r.Message, fields...)
	case LevelFatal:
		zb.logger.Fatal(r.Message, fields...)
	default:
		zb.logger.Info(r.Message, fields...)
	}

	return nil
}

func (zb *ZapBackend) Sync() error {
	return zb.logger.Sync()
}

func (zb *ZapBackend) Close() error {
	return zb.Sync()
}
```

---

## 5. The "Log Changer": Dynamic Hot-Swappable Logger Manager

The `LoggerManager` provides the consumer API and allows changing the backend, formatter, or writer safely across concurrent goroutines:

```go
package applogger

import (
	"context"
	"io"
	"sync"
	"time"

	"coding-guidelines/common/pkg/appfault"
)

// LoggerManager manages active backends and handles hot-swapping.
type LoggerManager struct {
	mu      sync.RWMutex
	backend LogBackend
}

// NewLoggerManager constructs a manager with an initial backend.
func NewLoggerManager(initial LogBackend) *LoggerManager {
	return &LoggerManager{backend: initial}
}

// SetBackend hot-swaps the underlying engine (e.g. from Console to Uber Zap).
func (m *LoggerManager) SetBackend(next LogBackend) {
	m.mu.Lock()
	defer m.mu.Unlock()
	m.backend = next
}

// SetFormatter alters the formatter if the active backend is a PipelineBackend.
func (m *LoggerManager) SetFormatter(f Formatter) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if pipeline, isOk := m.backend.(*PipelineBackend); isOk {
		pipeline.SetFormatter(f)
	}
}

// SetWriter alters the writer if the active backend is a PipelineBackend.
func (m *LoggerManager) SetWriter(w io.Writer) {
	m.mu.RLock()
	defer m.mu.RUnlock()
	if pipeline, isOk := m.backend.(*PipelineBackend); isOk {
		pipeline.SetWriter(w)
	}
}

// InfoContext logs an informational message with context.Context.
func (m *LoggerManager) InfoContext(ctx context.Context, msg string, fields ...map[string]any) {
	m.dispatch(ctx, LevelInfo, msg, fields...)
}

// ErrorContext logs an error message with context.Context.
func (m *LoggerManager) ErrorContext(ctx context.Context, msg string, fields ...map[string]any) {
	m.dispatch(ctx, LevelError, msg, fields...)
}

// LogError logs a structured AppError with context.Context.
func (m *LoggerManager) LogError(ctx context.Context, err *appfault.AppError) {
	if err == nil {
		return
	}

	m.mu.RLock()
	backend := m.backend
	m.mu.RUnlock()

	_ = backend.Log(LogRecord{
		Timestamp: time.Now().UTC(),
		Level:     LevelError,
		Message:   err.Message(),
		Context:   ctx,
		AppFault:  err,
	})
}

func (m *LoggerManager) dispatch(ctx context.Context, lvl LogLevel, msg string, fields ...map[string]any) {
	m.mu.RLock()
	backend := m.backend
	m.mu.RUnlock()

	merged := make(map[string]any)
	for _, f := range fields {
		for k, v := range f {
			merged[k] = v
		}
	}

	_ = backend.Log(LogRecord{
		Timestamp: time.Now().UTC(),
		Level:     lvl,
		Message:   msg,
		Context:   ctx,
		Fields:    merged,
	})
}
```

---

## 6. End-to-End Walkthrough: How to Swap at Runtime

Here is an end-to-end example showing how to swap formatters, writers, and engines dynamically:

```go
package main

import (
	"context"
	"os"

	"coding-guidelines/common/pkg/appfault"
	"coding-guidelines/common/pkg/applogger"
	"go.uber.org/zap"
)

func main() {
	// Setup request context with distributed tracing
	ctx := context.WithValue(context.Background(), "traceId", "txn-778899")
	ctx = context.WithValue(ctx, "userId", "user-101")

	// -------------------------------------------------------------
	// STEP 1: Start with Default Terminal Console Logger
	// -------------------------------------------------------------
	pipeline := applogger.NewPipelineBackend(&applogger.ConsoleFormatter{}, os.Stdout)
	log := applogger.NewLoggerManager(pipeline)

	log.InfoContext(ctx, "Phase 1: Booting application on terminal")

	// -------------------------------------------------------------
	// STEP 2: SWAP FORMATTER on the fly (Console -> JSON)
	// -------------------------------------------------------------
	log.SetFormatter(&applogger.JSONFormatter{})
	log.InfoContext(ctx, "Phase 2: Formatter swapped to JSON")

	// -------------------------------------------------------------
	// STEP 3: SWAP WRITER on the fly (Stdout -> File)
	// -------------------------------------------------------------
	file, _ := os.OpenFile("app.log", os.O_CREATE|os.O_WRONLY|os.O_APPEND, 0644)
	defer file.Close()

	log.SetWriter(file)
	log.InfoContext(ctx, "Phase 3: Writer swapped to file app.log")

	// -------------------------------------------------------------
	// STEP 4: SWAP ENTIRE ENGINE TO UBER ZAP
	// -------------------------------------------------------------
	// Instantiate Uber Zap production logger
	zapProductionLogger, _ := zap.NewProduction()
	zapBackend := applogger.NewZapBackend(zapProductionLogger)

	// Hot-swap the backend inside the LoggerManager
	log.SetBackend(zapBackend)

	// Now writing at maximum throughput via Uber Zap
	log.InfoContext(ctx, "Phase 4: Engine swapped to official Uber Zap")

	// -------------------------------------------------------------
	// STEP 5: Structured AppError with Uber Zap and Context
	// -------------------------------------------------------------
	fault := appfault.New(appfault.ErrDatabase, "database connection timed out").
		WithContext("host", "pg-cluster-01.internal").
		WithOp("db.connect")

	log.LogError(ctx, fault)
}
```

---

## 7. Migration Guide: Transitioning from Current `pkg/applogger`

1. **Step 1 (Deprecate Old Sinks):** Keep existing `ConsoleSink`, `FileSink`, `SQLiteSink` as backward-compatible wrappers implementing `LogBackend`.
2. **Step 2 (Introduce `LogBackend` Interface):** Add `LogBackend`, `Formatter`, and `Writer` to `interfaces.go`.
3. **Step 3 (Adopt `LoggerManager`):** Update `applogger.New()` to return `*LoggerManager`, exposing `.SetBackend()`, `.SetFormatter()`, and `.SetWriter()`.
4. **Step 4 (Native `context.Context` Methods):** Introduce `InfoContext(ctx, msg)`, `ErrorContext(ctx, msg)`, and `LogError(ctx, err)` to accept standard Go contexts.
5. **Step 5 (Official Uber Zap Adapter):** Replace the rudimentary string-based `ZapAdapter` with `ZapBackend` accepting `*zap.Logger` and mapping typed fields.
