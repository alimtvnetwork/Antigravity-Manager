# Learned Memory: Antigravity-Manager Dedicated Domain Skills Suite

> **Path:** `.ai-memory/memory/learned/19-agm-dedicated-domain-skills-suite.md`
> **Topic:** Creation of 6 specialized domain skills for Antigravity-Manager covering reverse proxy, thinking store, multi-instance sandboxing, split SQLite databases, React UI, and email remote control.
> **Date:** 2026-09-22
> **Status:** Active

---

## 1. Context & Rationale

Prior to this milestone, the `.agents/skills/` directory contained generic Prompt Architect meta-skills (such as movie CLI and installer skills) inherited from template repositories, without specialized domain skills tailored to the internal architecture of **Antigravity-Manager**.

To enable autonomous AI coding agents to diagnose, modify, and extend AGM subsystems with zero hallucination and high precision, a dedicated suite of 6 domain skills was created.

---

## 2. Established Skills Suite

| Skill Slug | Name | Core Focus & Source Files |
|---|---|---|
| `agm-proxy-engine` | **AGM Reverse Proxy Engine & Multi-Protocol Translation** | `src-tauri/src/proxy/server.rs`, `handlers/openai.rs`, `handlers/claude.rs`, `handlers/gemini.rs`, `token_manager.rs`, `proxy_pool.rs`. Covers Axum routing, protocol translations, and streaming SSE adapters. |
| `agm-thinking-store` | **AGM Thinking Store & Thought Signature Preservation** | `src-tauri/src/proxy/thinking_store.rs`, `handlers/thinking.rs`. Covers server-side Gemini thought caching, content-based turn matching, `{tenant}:{client_session_id}` isolation, and signature re-injection. |
| `agm-multi-instance-sandboxing` | **AGM Multi-Instance Sandboxing & Process Management** | `src-tauri/src/modules/instance.rs`, `process.rs`, `commands/instance.rs`, `src/stores/useInstanceStore.ts`. Covers profile isolation (`--user-data-dir`), process health scoring, and instance rotation. |
| `agm-split-sqlite-architecture` | **AGM Split-Database SQLite Architecture** | `src-tauri/src/modules/db.rs`, `proxy_db.rs`, `email_vault_db.rs`, `security_db.rs`, `user_token_db.rs`, `token_stats.rs`, `repo_db.rs`. Covers SQLite concurrency isolation, WAL mode, migrations, and encrypted credential vaults. |
| `agm-frontend-react-tauri` | **AGM Frontend React & Tauri IPC Architecture** | `src/pages/`, `src/components/`, `src/stores/`, `src/utils/request.ts`. Covers React 18, Tailwind CSS design system, Zustand stores, Tauri IPC contracts (`invoke`/`listen`), and Error History Drawer. |
| `agm-email-remote-control` | **AGM Inbound Email Remote Execution & SMTP Notifications** | `src-tauri/src/modules/email_inbound.rs`, `email_sender.rs`, `email_vault_db.rs`. Covers IMAP polling, `InboundAction` parsing, allowlists, and outbound HTML receipt delivery. |

---

## 3. Strict Compliance

Each skill adheres to:
1. Valid YAML frontmatter (`name` and `description`).
2. Strictly lowercase file naming (`skill.md`).
3. Strictly relative git paths with total ban on absolute paths or `file:///` URIs.
4. Positive affirmative boolean conventions.
