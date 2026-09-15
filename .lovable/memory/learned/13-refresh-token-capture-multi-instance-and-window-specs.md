# Refresh Token Capture, Multi-Instance Antigravity, & Window Customization

> **Path:** `.lovable/memory/learned/13-refresh-token-capture-multi-instance-and-window-specs.md`
> **Topic:** Authoritative reverse-engineered specifications for token capture pathways, multi-instance profile isolation, and window identity overlay
> **Ingested:** 2026-09-15
> **Status:** Active

---

## 1. Overview & Context

This memory file records the verified technical architecture, reverse-engineered codebase pathways, and design solutions for three core Antigravity capabilities:
1. **Refresh Token Discovery & Capture:** How Antigravity-Manager extracts OAuth refresh tokens from profiles, operating systems, and interactive authorization.
2. **Concurrent Multi-Instance Execution:** How to run multiple isolated Antigravity instances with distinct profiles concurrently.
3. **Window Identity Display:** How to project the active account/username onto the top of the Antigravity window.

Primary documentation index: [01-instructions/01-index.md](../../01-instructions/01-index.md).

---

## 2. Token Capture Pathways Ground Truth

Verified directly from `src-tauri/src/modules/integration.rs`, `migration.rs`, `db.rs`, and `oauth.rs`:

1. **OS Keyring (`integration.rs:512`):**
   - Windows: `advapi32::CredReadW` with target `gemini:antigravity`.
   - macOS: `/usr/bin/security find-generic-password -s gemini -a antigravity -w`.
   - Linux: `secret-tool lookup service gemini username antigravity`.
   - Strips `go-keyring-base64:` if present and parses JSON `{"token": {"refresh_token": "..."}}`.
2. **Profile SQLite Storage (`migration.rs:419`):**
   - Database: `state.vscdb` (`ItemTable`).
   - Unified format (>= 1.16.5): key `antigravityUnifiedStateSync.oauthToken`, sentinel `oauthTokenInfoSentinelKey`, Protobuf Field 3 (`refresh_token`), Field 6 (`is_gcp_tos`).
   - Legacy format (< 1.16.5): key `jetskiStateSync.agentManagerInitState`, Protobuf Field 6 (`oauthTokenInfo`), Field 3 (`refresh_token`).
3. **Interactive OAuth Consent (`oauth.rs:327`):**
   - Ephemeral Axum local server on loopback port.
   - Google OAuth URL with `access_type=offline` and `prompt=consent`.
   - Token exchange at `https://oauth2.googleapis.com/token`.
4. **V1 Migration (`migration.rs:17`):**
   - Path: `~/.antigravity-agent/antigravity_accounts.json`.

---

## 3. Multi-Instance Antigravity Execution (Feasibility: YES)

- **Root Cause of Single-Instance Limitation:** Default launcher in `process.rs` kills existing processes (`close_antigravity`) and targets single global `state.vscdb` and OS keyring (`gemini:antigravity`).
- **Solution:**
  - Partition user data: `--user-data-dir "<profile_dir>"`.
  - Partition extensions: `--extensions-dir "<ext_dir>"`.
  - Pass `--password-store="basic"` to prevent keyring collision.
  - Direct token injection into each profile's `<profile_dir>/User/globalStorage/state.vscdb`.
  - Process supervisor tracks instances by `(account_id, pid)` rather than bulk killing.

---

## 4. Window Username Customization (Feasibility: YES)

- **Option 1 (Native settings.json - Recommended):**
  Write `"window.title": "${activeEditorShort}${separator}Antigravity [${user_email}]"` to `<profile_dir>/User/settings.json`. Native, zero-hook, cross-platform.
- **Option 2 (Win32 Dynamic Hook):**
  `EnumWindows` + `SetWindowTextW` in `src-tauri` after process startup to dynamically append username without restart.
- **Option 3 (Tauri Floating Overlay Badge):**
  Transparent, borderless Tauri child window pinned to top right of IDE window with avatar, email, and live quota HUD.

---

## 5. Directory Invariant

All corresponding AI instructions and diagrams are maintained under `01-instructions/`:
- `01-instructions/01-index.md`
- `01-instructions/02-refresh-token-capture-architecture.md`
- `01-instructions/03-multi-instance-profile-isolation.md`
- `01-instructions/04-window-username-overlay-guide.md`
