# Subtask 01: Transport Backoff Stepper with Jitter
Status: COMPLETED
Sequence: 198
Owner: Subagent-1
Target Module: `src-tauri/src/proxy/upstream/transport_backoff_stepper.rs`
Test Suite: `src-tauri/src/proxy/upstream/tests_transport_backoff_stepper.rs`
Objective: Implement adaptive exponential backoff with full jitter and decorrelated jitter on upstream network failures per AGENTS.md timeout and retry rules.

Evidence:
- Module `src-tauri/src/proxy/upstream/transport_backoff_stepper.rs` created with `JitterMode`, `NetworkFailureType`, `BackoffConfig`, `BackoffStepResult`, and `TransportBackoffStepper`.
- Test suite `src-tauri/src/proxy/upstream/tests_transport_backoff_stepper.rs` created covering unrecoverable error rejection, max attempts exhaustion, ceiling clamping, full jitter, equal jitter, decorrelated jitter, and state resets.
- Registered and re-exported in `src-tauri/src/proxy/upstream/mod.rs`.
