## Quick Install v4.17.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.17.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.17.0/install.sh | bash
```

---

## What's Changed in v4.17.0

- **Test Suite Resilience & SQLite Persistence**: Added dedicated dual-tier cache clearing (`clear_tool_signatures`) in `proxy_db` and nanosecond precision in `TestDataDir`.
- **Canonical Gemini 3.7 Flash Model Resolution**: Calibrated `resolve_real_model` to preserve canonical identifier `gemini-3.7-flash` across dynamic variant mappings.
- **Payload Audit & Retention Threshold Harmonization**: Balanced payload audit threshold with retention disk budgets to eliminate truncation while retaining 100% test reliability.
- **Code Formatting & Clean Builds**: Formatted multi-line tuple return types for complete `cargo fmt` compliance and zero warnings.
