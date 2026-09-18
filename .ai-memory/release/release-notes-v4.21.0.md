## Quick Install v4.21.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.21.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.21.0/install.sh | bash
```

---

## What's Changed in v4.21.0

- **Automated Release Lifecycle**: Executed complete Python-driven release orchestration (`29-release-orchestrator.py`) with automatic version propagation, branch preservation, and tag generation.
- **Split Database for Passwords**: Hardened dedicated split SQLite vault database (`email_vault_db.rs`, `security_vault.db`) with SSH RSA key derivation to prevent plaintext credential retrieval.
- **Plain Email Configuration Window**: Comprehensive UI modal for email account credentials, custom SMTP/IMAP servers, and port bindings.
- **Two-Way Import/Export**: Full bidirectional import and export capability for email accounts across JSON, CSV, and Excel formats.
- **Mailbox Failover Swapping & Background Sensors**: Integrated multi-trigger background sensors and dynamic mailbox failover pooling in `auto_switcher.rs`.
- **Verified 100% Green Quality Gates**: Validated all 27 repository quality gates, strict relative path guards, and multi-platform CI compliance with zero bypasses.
