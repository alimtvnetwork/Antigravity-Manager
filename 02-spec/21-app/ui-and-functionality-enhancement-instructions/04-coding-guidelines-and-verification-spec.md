# Specification: Coding Guidelines (CG) & Verification Protocol

## 1. Executive Summary
This specification defines the non-negotiable coding guideline rules and verification gates for this task.

## 2. Coding Guideline Passes (CG Execution)
1. **Boolean Logic & Complex Condition Flattening**:
   - Invert negative guard clauses (`if (!condition) return;`).
   - Eliminate nested multi-level `if/else` ladders; decompose compound conditionals.
2. **Naming Conventions & Positive Prefixes**:
   - Enforce `is` and `has` prefixes exclusively for all boolean flags.
   - Ban negative boolean identifiers (`isDisabled`, `notFound`, etc.).
3. **Constants & Enums Extraction**:
   - Replace magic strings and raw literals with strongly-typed enums and centralized constants.
4. **CLI Commands & Help Standardization**:
   - Ensure all subcommands have comprehensive, structured help descriptions and examples.
5. **Function Returns & Explicit Return Types (D14)**:
   - Every function signature must explicitly declare its return type; avoid ambiguous implicit returns.

## 3. Verification & Pre-flight Gates
- Rust formatting check: `cd src-tauri && cargo fmt -- --check`.
- Rust clippy lint gate: `cd src-tauri && cargo clippy --all-targets --all-features`.
- Frontend build check: `npm run build`.
- Release preparation: `npm run bump minor`, changelog synchronization (attributing exclusively `@aukgit`), and atomic push via GitMap.
