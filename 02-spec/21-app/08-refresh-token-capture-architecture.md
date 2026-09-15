# Refresh Token Capture Architecture & Protocols

> Authoritative guide detailing how Antigravity-Manager discovers, parses, and extracts OAuth refresh tokens across operating systems, profiles, and runtimes.

---

## 1. Overview of Token Capture Pathways

Antigravity-Manager captures refresh tokens through four distinct pathways, evaluated in a deterministic fallback chain:

```mermaid
flowchart TD
    Start["Initiate Token Capture"] --> CheckKeyring{"Query OS System Keyring<br/>(service: 'gemini', user: 'antigravity')"}

    CheckKeyring -- "Found Token" --> ParseKeyring["Parse Keyring JSON Payload<br/>(Extract refresh_token)"]
    ParseKeyring --> Success["Token Successfully Captured"]

    CheckKeyring -- "Keyring Empty / Unsupported" --> ScanDB{"Scan Profile Candidate DBs<br/>(state.vscdb in APPDATA / .config)"}

    ScanDB -- "DB Found" --> QueryItemTable["Query ItemTable in SQLite<br/>(rusqlite SELECT value)"]
    QueryItemTable --> CheckVersion{"Format Detection"}

    CheckVersion -- ">= 1.16.5 (Unified State)" --> DecodeUnified["Decode antigravityUnifiedStateSync.oauthToken<br/>Sentinel: 'oauthTokenInfoSentinelKey'<br/>Protobuf Field 3: refresh_token"]
    CheckVersion -- "< 1.16.5 (Legacy Jetski)" --> DecodeLegacy["Decode jetskiStateSync.agentManagerInitState<br/>Protobuf Field 6 -> Field 3: refresh_token"]

    DecodeUnified --> Success
    DecodeLegacy --> Success

    ScanDB -- "No DB Available" --> CheckInteractive{"Interactive OAuth Requested?"}
    CheckInteractive -- "Yes" --> WebOAuth["Spawn Axum Localhost Callback Server<br/>Launch Google Consent URL<br/>Exchange auth code with Google API"]
    WebOAuth --> Success

    CheckInteractive -- "No" --> CheckV1["Inspect ~/.antigravity-agent/<br/>antigravity_accounts.json"]
    CheckV1 --> Success
```

---

## 2. Pathway 1: OS System Keyring (Modern >= 2.0.0 & agy CLI)

The modern Antigravity IDE and `agy` CLI persist credentials directly to the host OS credential store. Antigravity-Manager queries this directly via `read_from_system_keyring()` in `src-tauri/src/modules/integration.rs`:

| Platform | Underlying API / Command | Service / Target Name | User / Account |
| :--- | :--- | :--- | :--- |
| **Windows** | Win32 `advapi32.dll::CredReadW` | `gemini:antigravity` | `antigravity` |
| **macOS** | `/usr/bin/security find-generic-password` | `gemini` | `antigravity` |
| **Linux** | `secret-tool lookup` | `gemini` | `antigravity` |

### Keyring Payload Format

The decrypted credential blob is formatted as a JSON object:
```json
{
  "token": {
    "access_token": "ya29.a0...",
    "refresh_token": "1//04...",
    "token_type": "Bearer",
    "expiry": "2026-09-15T12:00:00Z"
  }
}
```
If the payload contains the `go-keyring-base64:` prefix (macOS format), the manager strips the prefix and decodes the Base64 bytes prior to JSON parsing.

---

## 3. Pathway 2: Profile SQLite Database (`state.vscdb`)

When the OS Keyring is empty or during profile migrations, the manager inspects local SQLite storage via `extract_oauth_state_from_file()` in `src-tauri/src/modules/migration.rs`.

### Candidate Database Locations

The candidate path resolver in `src-tauri/src/modules/db.rs` checks:
- **Windows:** `%APPDATA%\Antigravity\User\globalStorage\state.vscdb` and `%APPDATA%\Antigravity IDE\User\globalStorage\state.vscdb`
- **macOS:** `~/Library/Application Support/Antigravity/User/globalStorage/state.vscdb`
- **Linux:** `~/.config/Antigravity/User/globalStorage/state.vscdb`

### Protobuf Extraction Sequence

```mermaid
sequenceDiagram
    autonumber
    participant Mgr as Antigravity-Manager
    participant DB as state.vscdb (SQLite)
    participant Proto as Protobuf Decoder

    Mgr->>DB: SELECT value FROM ItemTable WHERE key = 'antigravityUnifiedStateSync.oauthToken'
    alt New Format Found (>= 1.16.5)
        DB-->>Mgr: Return Base64 String (Outer Layer)
        Mgr->>Proto: decode_unified_state_entry(outer_b64)
        Proto-->>Mgr: (sentinel_key, oauth_info_blob)
        Note over Mgr: Verify sentinel_key == 'oauthTokenInfoSentinelKey'
        Mgr->>Proto: find_field(oauth_info_blob, 3)
        Proto-->>Mgr: UTF-8 byte stream
        Note over Mgr: Extracted refresh_token!
        Mgr->>Proto: find_varint_field(oauth_info_blob, 6)
        Proto-->>Mgr: is_gcp_tos boolean flag
    else Fallback to Legacy Format (< 1.16.5)
        Mgr->>DB: SELECT value FROM ItemTable WHERE key = 'jetskiStateSync.agentManagerInitState'
        DB-->>Mgr: Return Base64 String
        Mgr->>Proto: Base64 Decode blob
        Mgr->>Proto: find_field(blob, 6) -> oauthTokenInfo
        Mgr->>Proto: find_field(oauthTokenInfo, 3) -> refresh_token
        Proto-->>Mgr: UTF-8 byte stream (refresh_token)
    end
```

### Protobuf Wire Encoding Details

1. **New Unified State Format (`antigravityUnifiedStateSync.oauthToken`):**
   - **Outer Wrapper:** Protobuf Field 1 contains nested protobuf.
   - **Middle Layer:** Field 2 contains inner state entry.
   - **Inner Entry:** Field 1 contains Base64-encoded string payload.
   - **OAuthInfo Structure:**
     - `Field 3 (Length-Delimited, Wire Type 2)`: Refresh Token (UTF-8 string).
     - `Field 6 (Varint, Wire Type 0)`: `is_gcp_tos` acceptance flag (`1` = true).
2. **Enterprise Project ID:**
   - Queried from key `antigravityUnifiedStateSync.enterprisePreferences`.
   - Field 3 in the decoded inner message holds the Google Cloud Project ID string.

---

## 4. Pathway 3: Interactive Browser OAuth Consent

When adding a fresh account without an existing installation, the manager runs an interactive OAuth2 flow defined in `src-tauri/src/modules/oauth.rs`:

```mermaid
sequenceDiagram
    autonumber
    participant User as Developer / User
    participant Mgr as Manager (Axum Server)
    participant Google as Google OAuth2 Endpoint

    Mgr->>Mgr: Bind ephemeral local HTTP server (e.g. 127.0.0.1:51121)
    Mgr->>User: Open system default browser with auth URL
    Note over User,Google: Scopes: cloud-platform, userinfo.email, userinfo.profile, cclog, experimentsandconfigs
    User->>Google: Grant OAuth Consent
    Google->>Mgr: HTTP GET /callback?code=4/0Ab...&state=...
    Mgr->>Google: POST https://oauth2.googleapis.com/token (code, client_id, client_secret)
    Google-->>Mgr: 200 OK (access_token, refresh_token, id_token, expires_in)
    Mgr->>Mgr: Persist Account & Token into local manager DB (accounts.json / SQLite)
```

### Client Credentials & Scopes

- **Standard Google Client ID:** `1071006060591-tmhssin2h21lcre235vtolojh4g403ep.apps.googleusercontent.com`
- **Redirect URI:** `http://127.0.0.1:<random_port>/callback`
- **Offline Access:** `access_type=offline` and `prompt=consent` are mandatory parameters to force Google to return a `refresh_token`.

---

## 5. Pathway 4: V1 Directory Migration (`~/.antigravity-agent/`)

For backward compatibility, the manager scans `~/.antigravity-agent/antigravity_accounts.json` via `import_from_v1()` in `src-tauri/src/modules/migration.rs`:
- Extracts legacy user profiles, backup file paths, and cached access tokens.
- Re-reads referenced `state.vscdb` backup snapshots to restore persistent refresh tokens.

---

## 6. Implementation Checklist for AI Agents

When implementing or debugging token capture in new tools or plugins:
1. [ ] Check OS Keyring first (`gemini:antigravity`) using native OS APIs to minimize file I/O locks.
2. [ ] If accessing `state.vscdb`, open SQLite with read-only flag or query while Antigravity is dormant to avoid database lock errors.
3. [ ] Parse protobuf with wire-type awareness (Wire type 2 for length-delimited strings, Wire type 0 for varints).
4. [ ] Always verify `refresh_token` format (starts with `1//` for standard Google OAuth offline tokens).
