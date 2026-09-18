# Pluggable & Context-Aware Logger Architecture Proposals

> Status: Pending Review
> Author: Antigravity AI
> Date: 2026-09-03
> Target Component: `04-code/golang/pkg/applogger` and `04-code/golang/pkg/logger`
> Reference Spec: `02-spec/03-error-manage/02-error-architecture/07-logging-and-diagnostics/04-pluggable-applogger-architecture.md`

---

## 1. Executive Summary & Problem Statement

The current `applogger` architecture couples formatters and writers directly inside sink implementations (e.g., `ConsoleSink` handles both text formatting and stdout output; `FileSink` hardcodes JSON formatting). Furthermore, it relies on a custom `appfault.ContextMap` instead of Go's standard `context.Context`.

This document presents 5 decoupled architectural patterns where:
1. **Formatters** are independently swappable (JSON, Pretty Console, Logfmt, custom).
2. **Writers** are independently swappable (`os.Stdout`, `*os.File`, `bytes.Buffer`, remote endpoints, `io.MultiWriter`).
3. **Context** is first-class via `context.Context` (automatic extraction of `traceId`, `userId`, `requestId`).

---

## 2. Core Architectural Decoupling

```
Application Code: log.InfoContext(ctx, "event", fields)
                           │
                           ▼
             ┌───────────────────────────┐
             │    Context Extractor      │  <-- Extracts traceId, spanId, userId from context.Context
             └─────────────┬─────────────┘
                           ▼
             ┌───────────────────────────┐
             │    Formatter Interface    │  <-- Swappable: JSON, Console, Logfmt
             │   Format(entry) -> []byte │
             └─────────────┬─────────────┘
                           ▼
             ┌───────────────────────────┐
             │     Writer Interface      │  <-- Swappable: Stdout, File, Remote, MultiWriter
             │   Write([]byte) -> (n, err│
             └───────────────────────────┘
```

---

## 3. Pattern 1: Decoupled Pipeline (Strategy Pattern)

**Best for:** Pure Go architecture, zero third-party dependencies, full custom control.

### Contracts
```go
package applogger

import (
	"context"
	"io"
	"time"
)

// Formatter formats a structured record into serialized bytes.
type Formatter interface {
	Format(record *LogRecord) ([]byte, error)
}

// Writer represents any standard Go writer.
type Writer io.Writer

// LogRecord carries normalized event data.
type LogRecord struct {
	Timestamp  time.Time      `json:"timestamp"`
	Level      string         `json:"level"`
	Message    string         `json:"message"`
	Fields     map[string]any `json:"fields,omitempty"`
	ContextMap map[string]any `json:"context,omitempty"`
	Caller     string         `json:"caller,omitempty"`
	Stack      string         `json:"stack,omitempty"`
}
```

### Concrete Formatters
```go
// JSONFormatter renders newline-delimited JSON.
type JSONFormatter struct{}

func (f *JSONFormatter) Format(record *LogRecord) ([]byte, error) {
	data, err := json.Marshal(record)
	if err != nil {
		return nil, err
	}
	return append(data, '\n'), nil
}

// ConsoleFormatter renders human-readable colored terminal output.
type ConsoleFormatter struct{}

func (f *ConsoleFormatter) Format(record *LogRecord) ([]byte, error) {
	output := fmt.Sprintf("[%s] %-5s %s",
		record.Timestamp.Format("15:04:05.000"),
		record.Level,
		record.Message,
	)
	if len(record.ContextMap) > 0 {
		output += fmt.Sprintf(" ctx=%v", record.ContextMap)
	}
	if len(record.Fields) > 0 {
		output += fmt.Sprintf(" fields=%v", record.Fields)
	}
	return []byte(output + "\n"), nil
}
```

### Engine with Runtime Swapping
```go
type Engine struct {
	mu         sync.RWMutex
	formatter  Formatter
	writer     Writer
	extractors []func(ctx context.Context) map[string]any
}

func (e *Engine) SetFormatter(f Formatter) {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.formatter = f
}

func (e *Engine) SetWriter(w Writer) {
	e.mu.Lock()
	defer e.mu.Unlock()
	e.writer = w
}

func (e *Engine) LogContext(ctx context.Context, level, msg string, fields map[string]any) {
	e.mu.RLock()
	formatter := e.formatter
	writer := e.writer
	extractors := e.extractors
	e.mu.RUnlock()

	ctxMap := make(map[string]any)
	for _, fn := range extractors {
		for k, v := range fn(ctx) {
			ctxMap[k] = v
		}
	}

	record := &LogRecord{
		Timestamp:  time.Now().UTC(),
		Level:      level,
		Message:    msg,
		Fields:     fields,
		ContextMap: ctxMap,
	}

	bytes, err := formatter.Format(record)
	if err != nil {
		return
	}
	_, _ = writer.Write(bytes)
}
```

---

## 4. Pattern 2: Go Standard `log/slog` Handler & Middleware (Go 1.21+)

**Best for:** Standard Go projects, modern zero-allocation performance, official Go ecosystem alignment.

### Code
```go
package applogger

import (
	"context"
	"io"
	"log/slog"
	"os"
)

// ContextHandler automatically inspects context.Context and appends trace attributes.
type ContextHandler struct {
	next slog.Handler
	keys []string
}

func NewContextHandler(next slog.Handler, keys ...string) *ContextHandler {
	return &ContextHandler{next: next, keys: keys}
}

func (h *ContextHandler) Enabled(ctx context.Context, lvl slog.Level) bool {
	return h.next.Enabled(ctx, lvl)
}

func (h *ContextHandler) Handle(ctx context.Context, record slog.Record) error {
	for _, key := range h.keys {
		if val := ctx.Value(key); val != nil {
			record.AddAttrs(slog.Any(key, val))
		}
	}
	return h.next.Handle(ctx, record)
}

func (h *ContextHandler) WithAttrs(attrs []slog.Attr) slog.Handler {
	return &ContextHandler{next: h.next.WithAttrs(attrs), keys: h.keys}
}

func (h *ContextHandler) WithGroup(name string) slog.Handler {
	return &ContextHandler{next: h.next.WithGroup(name), keys: h.keys}
}

// Factory supporting swappable Formatter and Writer
type Options struct {
	Format string    // "json" | "text"
	Writer io.Writer // os.Stdout, os.File, etc.
}

func NewSlogLogger(opts Options) *slog.Logger {
	var base slog.Handler
	handlerOpts := &slog.HandlerOptions{Level: slog.LevelDebug}

	if opts.Format == "json" {
		base = slog.NewJSONHandler(opts.Writer, handlerOpts)
	} else {
		base = slog.NewTextHandler(opts.Writer, handlerOpts)
	}

	wrapped := NewContextHandler(base, "traceId", "userId", "requestId", "spanId")
	return slog.New(wrapped)
}
```

---

## 5. Pattern 3: Interceptor Pipeline (Middleware Chain)

**Best for:** Security-focused systems, data redaction (PII masking), and audit streams.

```go
type LogEvent struct {
	Time    time.Time
	Level   string
	Message string
	Ctx     context.Context
	Meta    map[string]any
}

type Interceptor func(event *LogEvent) bool

// Example: PII Masking Interceptor
func MaskingInterceptor(event *LogEvent) bool {
	for k, v := range event.Meta {
		lower := strings.ToLower(k)
		if strings.Contains(lower, "password") || strings.Contains(lower, "token") {
			event.Meta[k] = "******"
		}
	}
	return true
}

// Example: Trace Enrichment Interceptor
func TraceInterceptor(event *LogEvent) bool {
	if event.Ctx != nil {
		if tid, isOk := event.Ctx.Value("traceId").(string); isOk {
			event.Meta["traceId"] = tid
		}
	}
	return true
}
```

---

## 6. Pattern 4: Atomic Hot-Swappable Logger (Zero Downtime Reconfiguration)

**Best for:** Dynamically altering destinations or verbosity without service restarts.

```go
type SwappableLogger struct {
	backend atomic.Pointer[Logger]
}

func (s *SwappableLogger) Swap(next Logger) {
	s.backend.Store(&next)
}

func (s *SwappableLogger) Info(ctx context.Context, msg string, fields ...any) {
	current := s.backend.Load()
	if current != nil {
		(*current).Info(ctx, msg, fields...)
	}
}
```

---

## 7. Pattern 5: Request-Scoped Context Logger (`FromContext`)

**Best for:** Web APIs (HTTP handlers, gRPC, background queue workers).

```go
type contextKey struct{}

// Inject logger into request context
func WithLogger(ctx context.Context, l *Logger) context.Context {
	return context.WithValue(ctx, contextKey{}, l)
}

// Retrieve scoped logger anywhere downstream in the business layer
func FromContext(ctx context.Context) *Logger {
	if l, isOk := ctx.Value(contextKey{}).(*Logger); isOk {
		return l
	}
	return DefaultLogger
}

// Usage in HTTP Middleware:
func Middleware(next http.Handler) http.Handler {
	return http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		reqLogger := DefaultLogger.With(
			"requestId", r.Header.Get("X-Request-ID"),
			"method", r.Method,
			"path", r.URL.Path,
		)
		ctx := WithLogger(r.Context(), reqLogger)
		next.ServeHTTP(w, r.WithContext(ctx))
	})
}
```

---

## 8. Comparison Matrix

| Feature | Pattern 1 (Pipeline Formatter/Writer) | Pattern 2 (`log/slog` Native) | Pattern 3 (Interceptor Chain) | Pattern 5 (Request-Scoped Context) |
| :--- | :--- | :--- | :--- | :--- |
| **Pluggable Formatter** | Direct interface (`Formatter`) | Native handler options (`JSONHandler` / `TextHandler`) | Serializer function | Inherited from base |
| **Pluggable Writer** | Standard `io.Writer` | Standard `io.Writer` | Standard `io.Writer` | Standard `io.Writer` |
| **Context Extraction** | Hook functions | Custom `slog.Handler` wrapper | Context interceptor | Direct `FromContext(ctx)` |
| **Ecosystem Standard** | Custom Clean Architecture | Official Go 1.21+ Standard | Custom Pipeline | Idiomatic Go Web Pattern |
| **Third-Party Deps** | Zero | Zero | Zero | Zero |

---

## 9. Recommended Direction

Combine **Pattern 2 (`log/slog` standard)** with **Pattern 5 (`FromContext` request-scoped extraction)**:
- Gives complete native Go 1.21+ `slog` compatibility with zero extra dependencies.
- Allows one-line switching of formatters (`slog.NewJSONHandler` vs `slog.NewTextHandler`).
- Allows plugging any `io.Writer` (`os.Stdout`, `*os.File`, or `io.MultiWriter(os.Stdout, file)`).
- Services can call `logger.FromContext(ctx).Info("message")` to seamlessly inherit request trace identifiers.
