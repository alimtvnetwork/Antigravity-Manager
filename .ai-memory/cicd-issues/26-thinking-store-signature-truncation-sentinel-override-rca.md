# RCA-26: Thinking Store Real Signature Override with Sentinel Placeholder in Pure Text Turns

## 1. Symptom

In GitHub Actions CI pipeline run [#35524439769](https://github.com/alimtvnetwork/Antigravity-Manager/actions/runs/35524439769), step `Run Rust tests` failed across Ubuntu (`ubuntu-latest`), Windows (`windows-2025`), and macOS (`macos-latest`):

```text
test proxy::thinking_store::tests::stores_full_thought_without_truncation ... FAILED

---- proxy::thinking_store::tests::stores_full_thought_without_truncation stdout ----
thread 'proxy::thinking_store::tests::stores_full_thought_without_truncation' panicked at src/proxy/thinking_store.rs:2217:9:
assertion `left == right` failed
  left: 32
 right: 60
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

failures:
    proxy::thinking_store::tests::stores_full_thought_without_truncation

test result: FAILED. 747 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 7.43s
```

## 2. Root Cause

During the upstream v3 proxy and thinking store synchronization, `restore_gemini_contents` in `src-tauri/src/proxy/thinking_store.rs` introduced a branching rule conditioned on `has_function_call`. For pure text model turns (`!has_function_call`), it unconditionally forced `thought_part["thoughtSignature"] = json!(SENTINEL_SIGNATURE)`. Because `SENTINEL_SIGNATURE` is `"skip_thought_signature_validator"` (length 32), any stored `ThinkingRecord` possessing a legitimate, full-length cryptographic thought signature (length 60) had its real signature clobbered by the 32-character sentinel placeholder, causing `stores_full_thought_without_truncation` to fail its assertion `assertion left == right failed: left: 32, right: 60`.

## 3. Resolution

Updated `restore_gemini_contents` in `src-tauri/src/proxy/thinking_store.rs` so that whenever `rec.signature` contains a valid cryptographic signature (`is_real_signature(sig)`), `thought_part["thoughtSignature"]` unconditionally receives the real signature `json!(sig)`. The fallback `SENTINEL_SIGNATURE` is strictly reserved for turns that lack a valid signature in the record. Furthermore, nested `if` statements were used to strictly satisfy the rule banning mixed polarity conditions (`if isA && !isB`).

## 4. Prevention & Learnings

- **Real Signatures Take Strict Precedence:** When restoring model thinking blocks from storage, real cryptographic signatures (`is_real_signature(sig)`) must never be overwritten or downgraded to synthetic sentinel strings (`skip_thought_signature_validator`).
- **Separation of Concerns between Tool Calls & Text Thinking:** A model turn may contain text-only thoughts with valid upstream signatures even in the absence of tool calls (`functionCall`). Signatures must not be coupled to the presence of tool calls.
- **Strict Adherence to Cross-Platform Guidelines:** Avoid mixed polarity checks and explicit boolean comparisons when handling option filters and control flow in Rust proxy handlers.
