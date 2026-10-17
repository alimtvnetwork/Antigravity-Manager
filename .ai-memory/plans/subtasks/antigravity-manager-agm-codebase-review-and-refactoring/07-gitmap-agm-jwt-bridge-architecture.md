# Subtask 07: GitMap <-> AGM Secure JWT Bridge Architecture

**Application:** Antigravity Manager (AGM)  
**Task Slug:** `antigravity-manager-agm-codebase-review-and-refactoring`  
**Subtask ID:** `07-gitmap-agm-jwt-bridge-architecture`  
**Status:** Ready / Implemented  
**Spec Reference:** [02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md](../../../../02-spec/21-app/antigravity-manager-agm-codebase-review-and-refactoring/01-architecture-spec.md)

---

## 1. Problem Statement & Architecture

### 1.1 Context
GitMap and Antigravity Manager run on the same developer machine and exchange operational context, including multi-node cluster status and email notification configurations. To prevent unauthorized local process snooping, all communication between GitMap CLI and the AGM local reverse proxy (`127.0.0.1:18188`) must be cryptographically authenticated using mutual HMAC-SHA256 JSON Web Tokens (JWT).

### 1.2 Core Security Invariants
1. **Loopback Interface Only:** The bridge is bound exclusively to `127.0.0.1:18188`.
2. **Short-Lived Lifetime:** Tokens expire within 60 seconds of issuance (`exp = iat + 60`).
3. **Anti-Replay Nonce Cache:** A sliding in-memory nonce cache rejects replayed `jti` identifiers within a 120-second window.
4. **Scoped RBAC Permissions:** Endpoints require explicit scopes (`email:read`, `cluster:sync`).

---

## 2. Technical Implementation Plan

### 2.1 Cryptographic Token Claims
```json
{
  "iss": "gitmap-client",
  "sub": "local-cli",
  "aud": "antigravity-manager",
  "iat": 1729209940,
  "exp": 1729210000,
  "jti": "nonce-a8b2c4e6-7f8a-4c2e-9d3a-1e5b7c9f0a2d",
  "scopes": [
    "email:read",
    "cluster:sync"
  ]
}
```

### 2.2 JWT Verification Middleware Structure
```rust
pub struct BridgeClaims {
    pub iss: String,
    pub sub: String,
    pub aud: String,
    pub exp: i64,
    pub iat: i64,
    pub jti: String,
    pub scopes: Vec<String>,
}

pub struct NonceCache {
    seen_nonces: HashMap<String, i64>,
}

impl NonceCache {
    pub fn is_nonce_valid(&mut self, jti: &str, now: i64) -> bool {
        // Purge expired nonces (> 120s)
        self.seen_nonces.retain(|_, seen_time| now - *seen_time < 120);

        if self.seen_nonces.contains_key(jti) {
            return false; // Replay detected
        }
        self.seen_nonces.insert(jti.to_string(), now);
        true
    }
}
```

### 2.3 Endpoint Matrix
| Route | Method | Required Scope | Description |
|---|---|---|---|
| `/api/v1/bridge/token` | POST | None (Shared Secret) | Issues short-lived JWT token |
| `/api/v1/bridge/email` | GET | `email:read` | Returns active email provider configuration |
| `/api/v1/bridge/cluster/sync` | POST | `cluster:sync` | Pushes multi-node fleet state to AGM |

---

## 3. Invariants & Positive Booleans

- `is_token_valid`: Positive boolean verifying signature, audience, and expiration.
- `has_matching_scope`: Positive boolean verifying that required scope exists in token claims.
- `is_nonce_valid`: Positive boolean verifying that the nonce has not been previously observed.

---

## 4. Verification & Testing

```bash
cd src-tauri && cargo test -p antigravity-tools-lib proxy
cd src-tauri && cargo clippy --all-targets --all-features
```

### Verification Criteria
- [x] Unauthenticated request to `/api/v1/bridge/email` returns `401 Unauthorized`.
- [x] Request with expired token (>60s) returns `401 Unauthorized`.
- [x] Request missing `email:read` scope returns `403 Forbidden`.
- [x] Repeated request with duplicate `jti` returns `409 Conflict`.
