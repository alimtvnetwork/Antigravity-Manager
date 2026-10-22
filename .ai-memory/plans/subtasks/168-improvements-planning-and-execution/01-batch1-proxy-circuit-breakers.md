# Subtask 01: Batch 1 (Imp 331–336: Proxy Circuit Breakers & Bounding)

- **Parent Plan**: `.ai-memory/plans/pending/168-improvements-planning-and-execution.md`
- **Spec**: `02-spec/21-app/168-improvements-planning-and-execution/spec.md`
- **Status**: [COMPLETED]

## Execution Summary
- Created `circuit_breaker_counter.rs` managing rolling failure frequency and circuit states.
- Created `retry_decelerator.rs` scaling retry backoffs with jitter and failure pressure penalties.
- Created `role_normalization_validator.rs` validating role transitions across canonical IR.
- Created `frame_length_clamper.rs` splitting oversized SSE event payloads on UTF-8 boundaries.
- Created `tool_call_assembler.rs` aggregating streaming partial function arguments.
- Created `route_latency_calculator.rs` calculating EMA latencies and route weighting factors.
- Re-exported all items in `upstream/mod.rs`, `handlers/common.rs`, and `handlers/openai/mod.rs`.

## Targeted Improvements
- **Imp-331: Upstream Circuit Breaker Trip Counter** (`src-tauri/src/proxy/upstream/circuit_breaker_counter.rs`)
- **Imp-332: Adaptive Retry Delay Decelerator** (`src-tauri/src/proxy/upstream/retry_decelerator.rs`)
- **Imp-333: Protocol Role Normalization Validator** (`src-tauri/src/proxy/handlers/common/role_normalization_validator.rs`)
- **Imp-334: Stream Frame Length Bounding Clamper** (`src-tauri/src/proxy/handlers/common/frame_length_clamper.rs`)
- **Imp-335: OpenAI Tool Call Delta Assembler** (`src-tauri/src/proxy/handlers/openai/tool_call_assembler.rs`)
- **Imp-336: Proxy Route Latency Weight Calculator** (`src-tauri/src/proxy/upstream/route_latency_calculator.rs`)
