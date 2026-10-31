# Deferred End-to-End Master Test Plan & Test Case Registry

- **Task**: Deferred End-to-End Master Test Execution
- **Spec Reference**: `02-spec/21-app/175-test-case-implementation-and-task-management/spec.md`
- **Execution Policy**: DEFERRED. Do not execute until user explicitly prompts for end-to-end testing.
- **Scope**: Complete E2E integration test catalog for Reverse Proxy, Auto-Switcher, Split-SQLite, Multi-Instance Isolation, and Prompt Resumption.

---

## 1. Test Suite Architecture

This catalog specifies all end-to-end integration test scenarios to be run as the final phase when commanded.

| Test ID | Subsystem | Target Scenario | Verification Mechanism |
| :--- | :--- | :--- | :--- |
| `E2E-175-01` | Reverse Proxy | Raw SSE Stream Multiplexing & Preamble Flushing | Stream multi-chunk payloads, verify double newline `\n\n` preservation and absence of client stalls. |
| `E2E-175-02` | Reverse Proxy | Upstream Backoff & Circuit Breaker Recovery | Simulate upstream HTTP 503 errors, assert exponential backoff jitter and automatic endpoint failover. |
| `E2E-175-03` | Reverse Proxy | Request Gzip Decompression Sniffer | Send gzip/deflate compressed payload without Content-Encoding header; verify transparent decompression. |
| `E2E-175-04` | Reverse Proxy | Claude Turn Alternation & Role Normalization | Send consecutive user messages; verify transparent concatenation and minimal dot injection for empty turns. |
| `E2E-175-05` | Auto-Switcher | 5-Hour Rolling Quota Boundary Protection | Probe candidate account 120s before reset window; verify rotation is withheld and backoff is applied. |
| `E2E-175-06` | Auto-Switcher | Consecutive Switch Velocity Throttling | Trigger 4 rapid switches within 30s; verify 4th switch is rejected with escalating cooloff penalty. |
| `E2E-175-07` | Auto-Switcher | Candidate Score Normalization & Idle Decay | Evaluate scoreboard with heterogeneous scores; verify [0..10] percentile bands and starvation boost. |
| `E2E-175-08` | Split-SQLite | WAL Non-Blocking Checkpoints | Insert 600 records across split databases; trigger `PRAGMA wal_checkpoint(PASSIVE)` and verify zero lockouts. |
| `E2E-175-09` | Multi-Instance | Environment Variable Token Scrubbing | Launch child process in mock sandbox; verify API keys (`OPENAI_API_KEY`, `GITHUB_TOKEN`) are completely stripped. |
| `E2E-175-10` | Multi-Instance | Collision-Free CDP Port Allocation | Spawn 3 mock instances concurrently; verify distinct ports assigned in range [9222..9299] without collision. |

---

## 2. Test Execution Playbook (For Future AI Execution)

When the user gives the prompt to execute End-to-End tests:
1. Run local sandboxed integration harness:
   ```pwsh
   python 03-ai-scripts/06-cicd-local-runner.py --skip-build --test-filter proxy::tests
   python 03-ai-scripts/06-cicd-local-runner.py --skip-build --test-filter auto_switcher::tests
   python 03-ai-scripts/06-cicd-local-runner.py --skip-build --test-filter db_orm::tests
   python 03-ai-scripts/06-cicd-local-runner.py --skip-build --test-filter instance::tests
   ```
2. Verify transient sandboxes are created and cleaned up with zero orphan processes.
3. Validate audit log entries in `task-history.db`.
