# Subtask 01: Protocol Adapter Divergence Auditor (`Imp-670`)
Status: IN_PROGRESS
Parent Plan: 209-implementing-next-five-tasks.md

## Scope & Objective
Per AGENTS.md Pipeline First & Protocol Adapter Decoupling:
"Keep the pipeline strictly decoupled from specific protocols. The four protocols function purely as Gemini adapters. Adapters are restricted to parameter normalization, payload transformation, protocol divergence adaptation, and edge cases unsolvable within the pipeline stage. The pipeline stage uniformly handles the converted Gemini payloads, including thinking block backfilling, thinking budget filtering and backfilling, unified context structural alignment, prefix stability, and the sanitization of risky prompts and request headers."

Implement `src-tauri/src/proxy/handlers/common/protocol_adapter_divergence_auditor.rs` and unit tests in `src-tauri/src/proxy/handlers/common/tests_protocol_adapter_divergence_auditor.rs`.

## Requirements
- Define protocol adapter classifications (OpenAI Responses, OpenAI Chat Completions, Anthropic Claude, Google Gemini).
- Audit code structure to ensure adapters only perform parameter normalization and payload transformation.
- Intercept forbidden pipeline logic (thinking budget clamping, prefix stabilization, context structural realignment) residing inside adapter files.
- Provide structured report tracking detected protocol divergence, pipeline leaks, and compliance status.
