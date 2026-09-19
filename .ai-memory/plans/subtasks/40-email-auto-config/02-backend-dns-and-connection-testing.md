# Subtask 40.2: Backend DNS Resolution & Direct Connection Testing

## Scope
1. Fix DNS resolution in `src-tauri/src/modules/email_sender.rs` and `src-tauri/src/modules/email_inbound.rs` using `std::net::ToSocketAddrs` so domain hostnames work.
2. In `src-tauri/src/modules/email_sender.rs`:
   - Implement `send_via_account_credentials(account: &EmailAccount, password: &str, ...)`
   - Implement `render_self_test_email(email: &str, machine_name: &str, machine_ip: &str) -> (String, String)`
3. In `src-tauri/src/commands/email.rs`:
   - Expose `test_direct_email_connection(account: EmailAccountInput) -> AppResult<String>`
4. In `src-tauri/src/lib.rs`:
   - Register `commands::test_direct_email_connection`
5. In `src/services/emailService.ts`:
   - Export `testDirectEmailConnection(account: EmailAccountInput): Promise<string>`

## Target Files
- `src-tauri/src/modules/email_sender.rs`
- `src-tauri/src/modules/email_inbound.rs`
- `src-tauri/src/commands/email.rs`
- `src-tauri/src/lib.rs`
- `src/services/emailService.ts`
