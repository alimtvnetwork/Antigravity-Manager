# Subtask: Email TLS Backend & Credential Redaction

**Target Files:**
- `src-tauri/Cargo.toml`
- `src-tauri/src/modules/email_sender.rs`
- `src-tauri/src/modules/email_inbound.rs`
- `src-tauri/src/commands/email.rs`

**Action:**
1. Add `native-tls = "0.2"` to `src-tauri/Cargo.toml`.
2. In `src-tauri/src/modules/email_sender.rs`:
   - Implement `native_tls::TlsConnector` wrapper supporting both implicit TLS (port 465) and STARTTLS (port 587/25).
   - For port 465, immediately perform TLS handshake after connect without waiting for plain banner.
   - For port 587, connect plain, read greeting, issue `STARTTLS`, perform TLS handshake, re-issue `EHLO`, then authenticate.
   - Redact all password contents in `send_smtp_cmd` error messages to `"<REDACTED>"`.
   - Return rich sanitized error diagnostics: `{ host, port, encryption_type, timeout_secs }`.
3. In `src-tauri/src/modules/email_inbound.rs`:
   - For port 993, perform immediate TLS handshake before reading IMAP greeting.
   - For port 143, issue `STARTTLS` before authentication.
   - Redact plaintext password in `send_imap_cmd` so socket errors never leak credentials.
4. In `src-tauri/src/commands/email.rs`:
   - Propagate sanitized connection diagnostics into error envelopes.

**Constraints:**
- Zero plaintext password leaks.
- Strict timeout handling (10s connect, 10s read).
- Never evaluate booleans explicitly against `true`.
