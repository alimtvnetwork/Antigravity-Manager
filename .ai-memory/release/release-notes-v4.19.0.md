## Quick Install v4.19.0

### Windows (PowerShell 5.1+)
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
# Or pinned version:
irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.19.0/install.ps1 | iex
```

### Linux / macOS (Bash)
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
# Or pinned version:
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.19.0/install.sh | bash
```

---

## What's Changed in v4.19.0

- **Split Database for Passwords**: Created dedicated split SQLite vault database (`email_vault_db.rs`, `security_vault.db`) with SSH RSA key derivation to protect email passwords.
- **Plain Email Configuration Window**: Added UI controls for email passwords, ports, and SMTP/IMAP settings to connect and read mailboxes.
- **Two-Way Email Import/Export**: Implemented JSON, CSV, and Excel two-way import/export for email configurations.
- **Mailbox Failover & Sensor Swapping**: Built background sensor pooling with bidirectional remote mailbox control and auto-swapper integration.
- **CI/CD Quality Gates & Rustfmt**: Resolved all TypeScript prop mismatches, Rust delimiter and borrow checker issues, and achieved 100% green CI passing.
