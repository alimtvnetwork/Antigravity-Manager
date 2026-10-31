# Subtask 03: Steps 6 & 7 - Dynamic Route Balancer & Upstream TLS SNI Sanitizer Tests

- **Parent Plan**: `.ai-memory/plans/pending/179-test-case-implementation-and-task-management.md`
- **Spec Reference**: `02-spec/21-app/179-test-case-implementation-and-task-management/spec.md#step-6-reverse-proxy-dynamic-route-balancer-unit-test-suite-imp-549`
- **Target Files**:
  - `src-tauri/src/proxy/upstream/tests_dynamic_route_balancer.rs`
  - `src-tauri/src/proxy/upstream/tests_tls_sni_sanitizer.rs`
- **Status**: [COMPLETED]

## Description
1. Step 6 (Imp-549): Authored `src-tauri/src/proxy/upstream/tests_dynamic_route_balancer.rs` with 6 unit test scenarios verifying initial weight, weight reduction on failure, gradual recovery on success, minimum weight floor (5), and route isolation. Registered in `src-tauri/src/proxy/upstream/mod.rs`.
2. Step 7 (Imp-550): Authored `src-tauri/src/proxy/upstream/tests_tls_sni_sanitizer.rs` with 8 unit test scenarios validating scheme stripping, port extraction, IPv4/IPv6 literal normalization, IDN Punycode detection, and invalid hyphen checks. Registered in `src-tauri/src/proxy/upstream/mod.rs`.
