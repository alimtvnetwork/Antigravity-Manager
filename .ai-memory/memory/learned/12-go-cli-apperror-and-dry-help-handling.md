# Go CLI AppError Enforcement & DRY Help Checking Architecture

> **Path:** `.ai-memory/memory/learned/12-go-cli-apperror-and-dry-help-handling.md`
> **Topic:** Universal elimination of standard Go `error` in favor of `*appfault.AppError`, and centralized DRY help/argument checking
> **Ingested:** 2026-09-15
> **Status:** Active (MANDATORY across all Go CLI tooling and connected repositories)

---

## 1. Executive Summary & Problem

During code audits of CLI subcommand implementations (e.g., `cmd/join.go`, `cmd/scan.go`), two critical anti-patterns were identified:

1. **Usage of standard Go `error`:** Functions returning error values were using the generic Go `error` interface instead of `*appfault.AppError`. This violates Rule 6 of `AGENTS.md` and fractures structured error handling, metadata propagation, and cross-language error envelope serialization.
2. **Repetitive Help Checking Boilerplate:** Handlers repeated copy-pasted boilerplate:
   ```go
   // ❌ Anti-pattern: duplicated across dozens of command files
   if len(args) == 0 || hasHelpFlag(args) {
       helptext.Print("join")
       return nil
   }
   ```
   This created code duplication (WET), varied help checking behavior across commands, and increased maintenance overhead.

---

## 2. Canonical Architectural Decisions

### Rule 1: Universal Go Error Return Type: `*appfault.AppError` (Standard)

- **Total Ban on Standard `error`:** In all Go packages (`cmd`, `pkg`, `internal`), functions that return failure metadata MUST use `*appfault.AppError` as their return type (e.g. `func runCommand(args []string) *appfault.AppError`).
- **Package Standard:** Always import `coding-guidelines/common/pkg/appfault` (or `pkg/appfault`).
- **Wrapping & Construction:** Wrap low-level errors via `appfault.Wrap(err, "domain.action", "E1002", "contextual message")` or construct new errors via `appfault.NewValidation(...)` or `appfault.NewSimple(...)`.

### Rule 2: Centralized DRY Help & Positional Argument Checking

- **Centralized Helper Function:** A single centralized helper (`cmd.CheckHelpOrEmpty` or `cmd.HandleHelpOrEmpty`) inspects arguments for help flags (`--help`, `-h`, `help`) and validates minimum required positional arguments.
- **Uniform Semantics:**
  - If a help flag is detected: prints embedded help text and returns `(true, nil)`.
  - If argument count is less than `minArgs` and no help flag: prints help text and returns `(true, appfault.NewValidation(...))`.
  - If valid: returns `(false, nil)`, signaling the command handler to proceed.
- **Zero Boilerplate:** Command handlers reduce their check to exactly 3 lines:
  ```go
  handled, appErr := CheckHelpOrEmpty("command_name", args, minArgs)
  if handled {
      return appErr
  }
  ```

---

## 3. Implementation Blueprint

### 3.1 Centralized Helper (`cmd/helpcheck.go`)

```go
package cmd

import (
    "coding-guidelines/common/pkg/appfault"
    "toolname/helptext"
)

// HasHelpFlag checks whether args contains -h, --help, or help.
func HasHelpFlag(args []string) bool {
    for _, a := range args {
        if a == "--help" || a == "-h" || a == "help" {
            return true
        }
    }
    return false
}

// CheckHelpOrEmpty provides DRY help interception and minimum argument validation.
func CheckHelpOrEmpty(command string, args []string, minArgs int) (bool, *appfault.AppError) {
    if HasHelpFlag(args) {
        helptext.Print(command)
        return true, nil
    }

    if len(args) < minArgs {
        helptext.Print(command)
        return true, appfault.NewValidation(
            "cli."+command,
            "E1001",
            "missing required arguments for command: "+command,
        )
    }

    return false, nil
}
```

### 3.2 Canonical Command Handler (`cmd/join.go`)

```go
package cmd

import (
    "coding-guidelines/common/pkg/appfault"
)

// RunJoin executes the join command.
func RunJoin(args []string) *appfault.AppError {
    handled, appErr := CheckHelpOrEmpty("join", args, 1)
    if handled {
        return appErr
    }

    // Domain execution logic returning *appfault.AppError
    return executeJoin(args[0])
}
```

---

## 4. Verification & Linting Gates

- **CI Guard:** `03-ai-scripts/28-go-preflight-ci.py` and `golangci-lint` verify that no exported or internal functions in `cmd/` return bare `error`.
- **DRY Help Auditor:** `03-ai-scripts/09-cli-help-auditor.py` audits CLI command definitions for centralized help coverage.
- **Spec Parity:** All specs under `02-spec/13-generic-cli/` (`03-subcommand-architecture.md`, `07-error-handling.md`, `09-help-system.md`) are synchronized to this pattern.
