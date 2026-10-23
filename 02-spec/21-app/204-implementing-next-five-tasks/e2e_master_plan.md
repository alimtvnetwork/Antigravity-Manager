# Sequence 204 E2E Master Test Plan: Multi-Stage Launcher, Clippy Gate, PR Cost Disclosure, Headroom Evaluator, and FIFO Order Enforcer

## Objectives
Validate Batch 204 end-to-end integration across multi-stage process launching, cargo clippy warning auditing, PR unverified paths cost disclosure, thinking budget headroom enforcement, and automatic FIFO query rewrites.

## Test Scenarios
1. **Multi-Stage Launcher Engine**:
   - Verify resolution ladder: direct target binary -> base detected binary -> shell dispatch (`cmd.exe /c start`).
   - Validate that `.cmd` and `.bat` files are never directly invoked via `Command::new`.
2. **Preflight Clippy Gate**:
   - Parse clippy JSON and text outputs.
   - Intercept compiler warnings, suggestions, and errors before release tags.
3. **PR Unverified Paths Linter**:
   - Validate PR bodies for `## Unverified Paths` and `## Cost Disclosure` sections.
   - Reject empty or missing disclosure blocks.
4. **Thinking Budget Headroom Evaluator**:
   - Calculate output tokens headroom.
   - Clamp thinking budget below `max_output_tokens` ceiling to prevent HTTP 400 errors.
5. **FIFO Prompt Order Enforcer**:
   - Transform arbitrary/missing queries into canonical `ORDER BY created_at ASC, id ASC`.
   - Strip inverted LIFO descending sort clauses.
