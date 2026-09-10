# Master Remediation Ledger: Application Specification Audit Fixes

> **Plan Reference:** `.lovable/plans/pending/12-spec-remediation.md`
> **Audit Reference:** `02-spec/25-app-spec-audit/01-audit-2026-09-10-v1.md`
> **Status:** COMPLETED
> **Updated:** 2026-09-10T19:22:00+08:00

---

## Master Checklist: 1:1 Finding Remediation

- [x] Finding [F-001]: File `02-spec/21-app/03-proxy-engine-and-protocols.md`, Issue: `Pseudocode session ID formula breaks Google upstream protocol (score impact -15)`, Remedy: `Replace pseudocode with exact 64-bit FNV-1a integer hashing algorithm from src-tauri/src/proxy/common/session.rs`
- [x] Finding [F-002]: File `02-spec/21-app/04-modules-storage-and-persistence.md`, Issue: `request_logs schema has inverted column names and lacks 6 columns (score impact -12)`, Remedy: `Replace schema with exact 18-column DDL, types, default values, and index definitions from src-tauri/src/modules/proxy_db.rs`
- [x] Finding [F-003]: File `02-spec/21-app/04-modules-storage-and-persistence.md`, Issue: `ip_access_logs misnamed as ip_access and ip_whitelist table missing (score impact -12)`, Remedy: `Correct table name to ip_access_logs, document ip_whitelist table, and document all 4 production indexes from src-tauri/src/modules/security_db.rs`
- [x] Finding [F-004]: File `02-spec/21-app/06-api-contracts-and-ipc-registry.md`, Issue: `50+ Tauri IPC commands missing from command registry table (score impact -15)`, Remedy: `Expand registry to document all 105 commands in src-tauri/src/lib.rs with arguments, return types, and Serde renaming`
- [x] Finding [F-005]: File `02-spec/21-app/05-frontend-ui-and-state-management.md`, Issue: `Documents non-existent useProxyStore.ts Zustand store (score impact -10)`, Remedy: `Remove hallucinated useProxyStore, document the 5 actual stores in src/stores/, and document local page state in src/pages/ApiProxy.tsx`
- [x] Finding [F-006A]: File `02-spec/21-app/02-security-and-risks.md`, Issue: `Lacks verification and acceptance criteria section (score impact -2)`, Remedy: `Add ## Verification & Acceptance Criteria section with structured Given/When/Then criteria`
- [x] Finding [F-006B]: File `02-spec/21-app/03-proxy-engine-and-protocols.md`, Issue: `Lacks verification and acceptance criteria section (score impact -2)`, Remedy: `Add ## Verification & Acceptance Criteria section with structured Given/When/Then criteria`
- [x] Finding [F-006C]: File `02-spec/21-app/04-modules-storage-and-persistence.md`, Issue: `Lacks verification and acceptance criteria section (score impact -2)`, Remedy: `Add ## Verification & Acceptance Criteria section with structured Given/When/Then criteria`
- [x] Finding [F-006D]: File `02-spec/21-app/05-frontend-ui-and-state-management.md`, Issue: `Lacks verification and acceptance criteria section (score impact -2)`, Remedy: `Add ## Verification & Acceptance Criteria section with structured Given/When/Then criteria`
- [x] Finding [F-006E]: File `02-spec/21-app/06-api-contracts-and-ipc-registry.md`, Issue: `Lacks verification and acceptance criteria section (score impact -2)`, Remedy: `Add ## Verification & Acceptance Criteria section with structured Given/When/Then criteria`
- [x] Finding [F-007]: File `02-spec/21-app/`, Issue: `Zero unit test, integration test, or fixture specifications (score impact -10)`, Remedy: `Author test specifications across buildable units and scaffold fixture files under 02-spec/21-app/fixtures/`
- [x] Finding [F-008]: File `02-spec/21-app/01-index.md`, Issue: `Zero binding links to repo coding guidelines and error management (score impact -10)`, Remedy: `Add normative coding guideline binding table linking to 02-spec/02-coding-guidelines/, 03-error-manage/, and 04-database-conventions/`
- [x] Finding [F-009A]: File `02-spec/23-app-db/01-index.md`, Issue: `Empty placeholder document inventory (score impact -4)`, Remedy: `Author full SQLite database specifications for proxy_logs.db, security.db, and user_tokens.db`
- [x] Finding [F-009B]: File `02-spec/24-app-ui-design-system/01-index.md`, Issue: `Empty placeholder document inventory (score impact -4)`, Remedy: `Author design system component specifications, Tailwind tokens, and layout guidelines`
- [x] Finding [F-010]: File `02-spec/21-app/03-proxy-engine-and-protocols.md`, Issue: `Circuit breaker backoff steps described qualitatively without default vector (score impact -5)`, Remedy: `Specify explicit default backoff vector [60, 300, 1800, 7200], grace retry rules (<= 2000ms), and error delay buffers`
- [x] Finding [GAP-EXTRA-01]: File `02-spec/21-app/03-proxy-engine-and-protocols.md`, Issue: `Phantom OpenAI endpoint /v1/embeddings documented in spec`, Remedy: `Delete /v1/embeddings and add real mounted endpoints (/v1/completions, /v1/responses, /responses/compact, /v1/images/edits)`
- [x] Finding [GAP-EXTRA-02]: File `02-spec/21-app/02-security-and-risks.md`, Issue: `Spec claimed 0 SQL injection vulnerabilities, but security_db.rs:215 contains direct format! string interpolation`, Remedy: `Update threat model with Finding SEC-004 documenting raw string interpolation in security_db.rs`
- [x] Finding [GAP-EXTRA-03]: File `02-spec/21-app/05-frontend-ui-and-state-management.md`, Issue: `Navigation shell incorrectly specifies Sidebar.tsx instead of top Navbar.tsx, and omits dual-mode IPC/REST runtime`, Remedy: `Replace Sidebar.tsx with Navbar.tsx hierarchy and document dual-mode IPC/REST gateway (src/utils/request.ts)`
