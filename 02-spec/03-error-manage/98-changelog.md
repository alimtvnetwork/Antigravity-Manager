# Error Management — Changelog

**Version:** 3.2.0
**Last Updated:** 2026-04-16

---

## v2.3.1 — 2026-09-01

### Mandatory Error Type First & Extended Collection Mutators

#### Changed — `pkg/appfault` Constructors
- Mandated `errtype.Variation` as the first argument in `Wrap(errType, cause, msg)` and `WrapType(errType, cause)`.
- Added `NewType(errType)` creating AppErrors with default type names.
- Guaranteed nil return when `cause == nil` or `errType == None`.

#### Added — `pkg/appfaults` Extended Mutators
- `AddType(errType)`: Adds error from enum type.
- `AddTypeMsg(errType, msg)`: Adds error from enum type + message.
- `AddTypeMsgf(errType, fmt, args...)`: Formatted message error.
- `AddError(errType, cause)`: Wraps cause error with explicit type.
- `AddErrorMsg(errType, cause, msg)`: Wraps cause error with explicit type and message.
- `AddWithContext(errType, msg, ctx)`: Adds error with custom context map.

---

## v2.3.0 — 2026-09-01

### Dedicated `errtype` Package & Integer-Backed Enums

#### Added — `pkg/errtype` Package
- `type Variation uint16` with `None = 0` (no error / success).
- Standard variations: `Generic (1)`, `Validation (2)`, `NotFound (3)`, `Precondition (4)`, `Execution (5)`, `Database (6)`, `Network (7)`, `Timeout (8)`, `IO (9)`, `Unauthorized (10)`, `Forbidden (11)`, `Internal (12)`, `Unknown (13)`.
- Extensible custom error types (`const CustomErr errtype.Variation = 1001`).

#### Added — Integer Enums with PascalCase Serialization
- `SeverityType` (`byte`) and `PriorityType` (`byte`) enums in `pkg/appfault`.
- PascalCase string outputs (`"Info"`, `"Warn"`, `"Error"`, `"Critical"`, `"Fatal"`) via `String()` and custom JSON/YAML marshaling.
- Documented in retrospective `02-spec/03-error-manage/01-error-resolution/03-retrospectives/07-golang-integer-enums-and-pascal-serialization.md`.

---

## v2.2.0 — 2026-04-02

### Domain Convenience Constructors + Error Merge

#### Added — Domain convenience constructors (in `02-apperror-struct.md`)

- `UrlError(errType, url)` / `WrapUrlError(cause, errType, url)` — auto-sets `WithUrl()`
- `SlugError(errType, slug)` / `WrapSlugError(cause, errType, slug)` — auto-sets `WithSlug()`
- `SiteError(errType, siteId)` / `WrapSiteError(cause, errType, siteId)` — auto-sets `WithSiteId()`
- `EndpointError(errType, method, ep, statusCode)` / `WrapEndpointError(...)` — auto-sets `WithEndpoint()` + `WithMethod()` + `WithStatusCode()`
- Convenience summary table (section 2.2.6)

#### Added — Error merge methods (in `02-apperror-struct.md`)

- `Merge(errors)` — combines multiple `AppError` into one, uses first error's code
- `MergeWithCode(code, message, errors)` — merges under a specific error code
- Batch validation and multi-step processing examples

---

## v2.1.0 — 2026-04-02

### WrapTypeMsg Constructor + Path Convenience Methods

#### Added — `WrapTypeMsg` constructor (in `02-apperror-struct.md`)

- `WrapTypeMsg(cause error, errType ErrorType, message string)` — wraps with enum code but custom message
- Enables 3-level progression: `Wrap()` → `WrapType()` → `WrapTypeMsg()`

#### Added — Path convenience constructors (in `02-apperror-struct.md`)

- `PathError(errType, path)` — creates path-related AppError with automatic `WithPath()` diagnostic
- `WrapPathError(cause, errType, path)` — wraps cause with path variant + automatic `WithPath()` diagnostic

#### Added — New path variants (in `05-apperrtype-enums.md`)

- `PathMissing` (E4016) — required path is missing
- `PathFailedToCreate` (E4017) — failed to create path
- `PathFailedToRead` (E4018) — failed to read path
- `PathFailedToWrite` (E4019) — failed to write to path
- `PathFailedToDelete` (E4020) — failed to delete path

#### Changed — Root `readme.md`

- Expanded CODE-RED-005/006 example from 2 levels to 3-level progression (✅ → ✅✅ → ✅✅✅)
- Added `PathError` / `WrapPathError` usage examples

---

## v2.0.0 — 2026-04-02

### `apperrtype` v2 Migration — Single Variation Enum

**Breaking change:** Migrated from per-domain `byte` enums to a single `uint16 Variation` enum with global registry. Inspired by [evatix-go/errorwrapper/errtype](https://gitlab.com/auk-go/errorwrapper/-/tree/develop/errtype).

#### Changed — `05-apperrtype-enums.md` (full rewrite)

- Replaced 14 per-domain `byte` enums (`PluginError`, `ConfigError`, etc.) with single `Variation uint16`
- Replaced `ErrorDetail{Code, Message}` with `VariantStructure{Name, Code, Message, Variant}`
- Replaced per-domain detail maps with single `variantRegistry map[Variation]VariantStructure`
- `ErrorType` interface gains `Name() string` method
- Added display methods on `Variation`: `String()`, `CodeTypeName()`, `CodeTypeNameWithReferences()`
- Added display methods on `VariantStructure`: `TypeNameCodeMessage()`, `CodeTypeNameWithMessage()`, `Error()`, `ErrorNoRefs()`, `Panic()`
- Added `IsValid()` and `Structure()` methods on `Variation`
- Expanded domains: E15xxx (Network), E16xxx (Process), E17xxx (Encoding), E18xxx (Permission)
- Added migration table documenting v1→v2 mapping

#### Added — `StringToVariantMap` (in `05-apperrtype-enums.md`)

- New `string_to_variant_map.go` — reverse-lookup from PascalCase name → `Variation`
- `VariationFromName(name) (Variation, bool)` — safe lookup
- `MustVariationFromName(name) Variation` — panics if not found

#### Added — `CodeToVariantMap` (in `05-apperrtype-enums.md`)

- New `code_to_variant_map.go` — reverse-lookup from string code (e.g. `"E2010"`) → `Variation`
- `VariationFromCode(code) (Variation, bool)` — safe lookup
- `MustVariationFromCode(code) Variation` — panics if not found

#### Changed — `02-apperror-struct.md`

- Updated `NewType` / `WrapType` constructor signatures to accept `apperrtype.ErrorType`
- Added section 2.3.1: Variation display methods with corrected signatures and examples
- Added section 2.3.2: `Structure()` lookup with `VariantStructure` display method table
- Added section 2.3.3: Direct error creation from `VariantStructure` (`Error()`, `ErrorNoRefs()`, `Panic()`)
- Fixed all example output formats to match actual `05-apperrtype-enums.md` implementations
- Replaced non-existent variants (`DatabaseTimeout`, `ConfigMissing`) with valid ones

#### Changed — `04-codes-and-policy.md`

- Replaced v1 `PluginError byte` + `ErrorDetail` + per-domain map examples with v2 `Variation` + `VariantStructure` + `variantRegistry`
- Updated rules section to reflect single-enum architecture
- Fixed spec cross-reference link to point to `05-apperrtype-enums.md`

#### Changed — Root `readme.md`

- Updated `apperrtype` package section from v1 pattern to v2
- Added `VariantStructure`, `variantRegistry`, `StringToVariantMap` documentation
- Added `VariationFromName()` reverse-lookup example
- Fixed spec link from `04-codes-and-policy.md` to `05-apperrtype-enums.md`

#### Files Modified

| File | Change |
|------|--------|
| `02-error-architecture/06-apperror-package/01-apperror-reference/05-apperrtype-enums.md` | Full rewrite to v2 |
| `02-error-architecture/06-apperror-package/01-apperror-reference/02-apperror-struct.md` | Display methods + signature fixes |
| `02-error-architecture/06-apperror-package/01-apperror-reference/04-codes-and-policy.md` | v1→v2 examples |
| `readme.md` (project root) | v1→v2 apperrtype section |

---

## v1.0.0 — 2026-03-31

### Initial Consolidation

#### Added

- Created `04-error-manage/` as the single canonical location for all error management specs
- Organized into 3 categories: Error Resolution, Error Architecture, Error Code Registry
- New `01-index.md` with core principles, common pitfalls, and cross-references

#### Consolidated From

#### Structure

- `01-error-resolution/` — Retrospectives, verification patterns, debugging guides, cheat sheet, cross-reference diagram
- `02-error-architecture/` — Error handling reference, delegation fix, notification colors, error modal, response envelope, apperror, logging
- `03-error-code-registry/` — Master registry, integration guide, collision resolution, utilization report, overlap validator, schemas, scripts, templates

---

*Keep this file updated when specs change.*
