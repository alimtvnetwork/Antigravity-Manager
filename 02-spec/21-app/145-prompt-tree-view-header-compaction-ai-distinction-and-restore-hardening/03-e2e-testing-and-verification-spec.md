# 03-e2e-testing-and-verification-spec: E2E Verification & Test Suite Specification

- **Spec ID:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening/03-e2e-testing-and-verification-spec.md`
- **Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)
- **Status:** `APPROVED`
- **Lead Architect:** Antigravity Pairing Agent

---

## 1. Test Verification Scope

This specification establishes the verification protocol for testing:
1. **Tree Collection & 3-Tier Classification:**
   - Real transcript inspection extracting User Prompts, AI Subagent Instructions, and filtering Non-Prompts.
   - Dual-source ingestion checking `transcript_full.jsonl` vs `transcript.jsonl`.
2. **Results & Tool Execution Output Extraction:**
   - Extracting `latest_response`, `thinking`, `tool_calls`, and `latest_step_summary`.
3. **UI Capsule & Layout Verification:**
   - Verifying compact header rendered with zero overflow.
   - Verifying rich copy ("Copy with Image") and export (`.md`, `.json`).
   - Verifying truncation banner removal and sleek inline badge rendering.
4. **FIFO Backup & Restore:**
   - Round-trip backup export and FIFO injection into `.antigravity_resume_task.json`.
5. **Pre-flight Checks:**
   - Cargo fmt, Cargo clippy, Frontend TypeScript build (`npm run build`).
