## Quick Install v4.16.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.16.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.16.0/install.sh | bash
```

---

## What's Changed in v4.16.0

- **Unified Multi-Protocol Streaming Pipeline Engine**: Synchronized and merged the canonical Inbound sanitization and Outbound extraction pipeline engine across OpenAI Chat (`/v1/chat/completions`), Anthropic Claude (`/v1/messages`), OpenAI Responses (`/v1/responses`), and Google Gemini Native protocols.
- **Thinking Store & Cryptographic Signature Normalization**: Integrated server-authoritative multi-tier Thinking Store with L1 memory (`DashMap`) and L2 SQLite (`thinking_store.db`) persistence. Employs `flate2` Gzip compression (`AGZ1` header) for thoughts exceeding 384 chars and provides automatic cryptographic signature healing to permanently eliminate Google upstream 400 signature validation errors.
- **SQLite Concurrency, Query Acceleration & Bounded Disk Quota**: Replaced per-query connection creation with lazy read-only connection pooling and `prepare_cached` statement caching, slashing long-context signature query latency by 98.5%. Introduced disk quota bounds (`proxy.log_retention.max_disk_mb`) with automated progressive page reclamation.
- **Ecosystem & Tooling Alignment**: Filtered Claude desktop client billing metadata (`x-anthropic-billing-header:`) for Gemini targets with large MCP toolsets, preventing Google WAF `RESOURCE_EXHAUSTED` 429 errors. Added DeepSeek Harness (DSH) and WorkBuddy tool call schema adaptations.
- **Codebase Cleanliness & Zero-Hallucination Compliance**: Maintained strictly English-only comments and documentation, full attribution to upstream `lbjlaq/Antigravity-Manager`, and preserved all AGM by Alim custom error management and multi-instance capabilities.
