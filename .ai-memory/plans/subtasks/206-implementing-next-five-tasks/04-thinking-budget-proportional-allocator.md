# Subtask 04: Dynamic Context Ceiling Thinking Budget Proportional Allocator (`Imp-658`)
Status: IN_PROGRESS
Parent Plan: 206-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Architecture & Pipeline First:
"The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability..."

Implement `src-tauri/src/proxy/handlers/common/thinking_budget_proportional_allocator.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_thinking_budget_proportional_allocator.rs`.

## Requirements
- Define model context tier profiles (e.g., standard 8k, medium 32k/64k, large 128k/1M).
- Calculate proportional thinking budget ratio (default 75% thinking budget, 25% output tokens headroom) bounded by model ceiling.
- Clamp requested budgets safely to prevent context overflow or starvation.
- Emit structured `ProportionalAllocationReport`.
