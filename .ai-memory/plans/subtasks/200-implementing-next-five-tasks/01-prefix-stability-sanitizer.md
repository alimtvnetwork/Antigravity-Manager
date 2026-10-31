# Subtask 01: Pipeline Prefix Stability & Header Sanitizer
Status: COMPLETED
Sequence: 200
Owner: Worker-1
Target Module: `src-tauri/src/proxy/handlers/common/prefix_stability_sanitizer.rs`
Test Suite: `src-tauri/src/proxy/handlers/common/tests_prefix_stability_sanitizer.rs`

## Evidence
- Implemented `PrefixStabilityPolicy`, `PrefixSanitizationReport`, `sanitize_request_headers`, and `serialize_deterministic_json`.
- Enforces stripping of internal proxy headers (`x-agm-internal`, `x-antigravity-internal`, `x-proxy-forwarded`) and disallowed Google headers (`x-goog-api-client`, `x-goog-api-key`).
- Implemented recursive deterministic JSON key sorting for bit-identical prefix stability in prompt caching.
- 10 unit tests covering forbidden header stripping, standard header retention, case insensitivity, policy custom header overrides, and recursive JSON key sorting.
