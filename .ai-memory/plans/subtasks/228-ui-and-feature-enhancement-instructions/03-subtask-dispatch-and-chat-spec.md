---
plan: 228-ui-and-feature-enhancement-instructions
subtask: "03"
title: Dispatch Configuration, Strict E2E Verification Prompts, Future Chat Section Architecture & 50+ Improvement Ideas
domain: frontend-react-typescript-and-architecture
assigned_worker: Spec-Author-03
depends_on:
  - 02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md
citations:
  architecture_spec: ../../../../02-spec/21-app/228-ui-and-feature-enhancement-instructions/01-architecture-spec.md
  dispatch_spec: ../../../../02-spec/21-app/228-ui-and-feature-enhancement-instructions/03-dispatch-and-verification-spec.md
  chat_spec: ../../../../02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md
  improvements_spec: ../../../../02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md
target_files:
  - 02-spec/21-app/228-ui-and-feature-enhancement-instructions/03-dispatch-and-verification-spec.md
  - 02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md
  - 02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md
status: completed
---

# Subtask 03: Dispatch Configuration, Strict E2E Verification Prompts, Future Chat Section Architecture & 50+ Improvement Ideas

## 1. Context & Motivation

As part of the master execution plan `228-ui-and-feature-enhancement-instructions`, this subtask authors three foundational architectural specifications and defines the execution and verification contracts for:
1. **Dispatch Configuration & Verification Engine**: Simplifying mode buttons down to clean "Send", "Enqueue", and "Rerun" labels with rich tooltips, expanding popover width from `w-80` to `w-[26rem]` to eradicate message truncation, enforcing canonical semicolon syntax `@[file;<path>]` across all attachments, injecting strict pre-deployment verification questions, coordinating multi-account switching dispatch, and integrating release page / notes audit links into the rerun section.
2. **Future Chat Section Architecture**: Formulating the comprehensive architectural specification for integrating a first-class "Chat" navigation tab into Antigravity Manager with a Dual-Channel Engine:
   - **Channel 1 (IDE Chat Bridge)**: Bi-directional workspace copilot mirroring active Antigravity IDE sessions via CDP WebSocket and live `transcript_full.jsonl` watching.
   - **Channel 2 (Direct API Chat Client)**: Standalone headless AI chat client connecting directly to AGM's local Axum proxy gateway with Server-Sent Events (SSE).
3. **50+ System & UI Improvement Catalog**: Structuring an exhaustive, grounded catalog of 50 concrete enhancements spanning Instances, Prompts, SQLite Speed Database, Proxy Gateway, and Settings/Theming.

---

## 2. Target Specifications & Disjoint File Ownership

| Specification Document | Scope & Functional Areas | Associated Target Code Files |
|---|---|---|
| `02-spec/21-app/228-ui-and-feature-enhancement-instructions/03-dispatch-and-verification-spec.md` | Mode buttons simplification, `w-[26rem]` popover expansion, `@[file;<path>]` syntax, strict E2E prompts, multi-account dispatch, release notes rerun integration | `src/components/instances/PromptTreeView/DetailHeader.tsx`<br/>`src/components/instances/PromptTreeView/verificationTemplates.ts`<br/>`src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` |
| `02-spec/21-app/228-ui-and-feature-enhancement-instructions/04-chat-section-architecture-spec.md` | Future Chat tab architecture, dual-channel design (CDP IDE bridge + Axum SSE gateway), IPC registry, 4-tier origin badges, interactive omission callouts, caller self-preservation | `src/components/navbar/NavMenu.tsx`<br/>`src/pages/Chat.tsx`<br/>`src-tauri/src/commands/cmd_chat.rs`<br/>`src-tauri/src/modules/transcript_watcher.rs` |
| `02-spec/21-app/228-ui-and-feature-enhancement-instructions/05-fifty-plus-improvement-ideas-spec.md` | 50+ grounded improvements across 5 categories (Instances, Prompts, SQLite DB, Proxy Gateway, Settings/UI) | Repository-wide modules across frontend, backend, database schemas, and proxy pipelines |

---

## 3. Work Breakdown & Subtask Execution Checklist

### Phase 1: Dispatch & Verification Specification (`03-dispatch-and-verification-spec.md`)
- [x] **Item 1.1**: Document simplification of mode switcher buttons: `"Send (Preempt)"` $\to$ `"Send"`, `"Enqueue (FIFO)"` $\to$ `"Enqueue"`, and `"Rerun Turn"` $\to$ `"Rerun"`. Preserve keyboard shortcuts (`N`, `Q`, `R`) and rich operational descriptions in `title` attributes.
- [x] **Item 1.2**: Specify container width expansion of dispatch popover from `w-80` (320px) to `w-[26rem]` (416px, min `w-96` / 384px) to ensure unclipped message selector dropdowns and template titles.
- [x] **Item 1.3**: Codify canonical semicolon attachment syntax `@[file;<path>]` with trailing attachment block placement (`### Attachments:`) and dual-format backward compatibility for legacy `@[<path>]`.
- [x] **Item 1.4**: Define two mandatory built-in verification gate templates:
  - *"Have you tested these below instructions with end-to-end testing?"* (`verify-e2e-tested`)
  - *"Have you tested the below instructions with end-to-end testing and released it?"* (`verify-e2e-released`)
  along with their integration into `RERUN_RANDOM_MESSAGE_SET`.
- [x] **Item 1.5**: Detail multi-account switching API dispatch flow (`cmd_accounts::switch_account`), ensuring pre-switch verification and atomic rollback before prompt injection.
- [x] **Item 1.6**: Detail release notes and changelog audit integration within the rerun panel, enabling one-click verification of release assets.

### Phase 2: Future Chat Section Architecture (`04-chat-section-architecture-spec.md`)
- [x] **Item 2.1**: Establish top-level navigation integration in `src/components/navbar/NavMenu.tsx`.
- [x] **Item 2.2**: Formulate Channel 1 (IDE Chat Bridge) using CDP WebSocket discovery and `transcript_full.jsonl` log monitoring via Rust `notify` crate, with 10-line reverse scanning to filter heartbeat/telemetry.
- [x] **Item 2.3**: Formulate Channel 2 (Direct API Chat Client) connecting to local Axum proxy with SSE streaming, decoupled from IDE lifecycle.
- [x] **Item 2.4**: Define Tauri IPC commands (`chat_send_message`, `chat_cancel_stream`, `chat_list_sessions`, `chat_get_transcript`) and frontend event streams (`chat:chunk`, `chat:thought`, `chat:tool_call`, `chat:done`).
- [x] **Item 2.5**: Define UI layout hierarchy, 4-tier prompt origin classification (`USER_PROMPT`, `SUBAGENT_INSTRUCTION`, `SYSTEM_MESSAGE`, `TOOL_OUTPUT`), dark-glass omission banners (`⚡ [Omitted N bytes/lines of context]`), and `ChatErrorBoundary`.
- [x] **Item 2.6**: Codify Caller IDE Self-Preservation invariant (`detect_caller_instance_id()`) preventing accidental disruption of active development sessions.

### Phase 3: 50+ Improvement Ideas Catalog (`05-fifty-plus-improvement-ideas-spec.md`)
- [x] **Item 3.1**: Author 10 grounded improvements for Instances Architecture & Lifecycle (Items 01–10).
- [x] **Item 3.2**: Author 10 grounded improvements for Prompts Architecture, FIFO Queue & Tree View (Items 11–20).
- [x] **Item 3.3**: Author 10 grounded improvements for Speed Database, SQLite Engine & Persistence (Items 21–30).
- [x] **Item 3.4**: Author 10 grounded improvements for Proxy Gateway, Pipeline Engine & Protocol Adapters (Items 31–40).
- [x] **Item 3.5**: Author 10 grounded improvements for Settings, Theming, Accessibility & UI/UX Experience (Items 41–50).

---

## 4. Verification & Testing Contracts

Execution workers implementing downstream code changes must verify against the following test contracts:

### Contract 1: Popover & Mode Buttons Verification
- **Target**: `src/components/instances/PromptTreeView/DetailHeader.tsx`
- **Verification Rule**: Button text must strictly match `"Send"`, `"Enqueue"`, and `"Rerun"`. Popover container must carry `w-[26rem]` or `w-96`. Dropdown options with > 40 characters must render without text truncation.

### Contract 2: Semicolon Attachment Formatting
- **Target**: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts` & `src/components/instances/PromptTreeView/__tests__/rerunActions.test.ts`
- **Verification Rule**: All enqueued prompts containing media paths must output `@[file;<file_path>]`. Zero occurrences of legacy `@[path]` without `file;` prefix in generated payloads.

### Contract 3: Strict Pre-Deployment Verification Prompts
- **Target**: `src/components/instances/PromptTreeView/verificationTemplates.ts`
- **Verification Rule**: Presets `verify-e2e-tested` and `verify-e2e-released` must exist in `VERIFICATION_OPTIONS` with exact required phrasing. Both questions must exist in `RERUN_RANDOM_MESSAGE_SET`.

### Contract 4: FIFO Monotonicity on Reruns
- **Target**: `src/components/instances/PromptTreeView/usePromptTreePromptActions.ts`
- **Verification Rule**: All rerun rounds (1..N) must invoke `enqueue_prompt`. Calling `send_prompt_now` during rerun is strictly prohibited (assert 0 calls in unit tests).

---

## 5. Invariant Checklist

- [x] **Strict Relative Paths**: All paths in specs and documentation are repository-relative. Zero absolute paths (`D:\...`, `C:\...`).
- [x] **Decoupled Pipeline Architecture**: Upstream protocol adapters restricted to normalization; pipeline handles thinking backfill and sanitization.
- [x] **Caller IDE Self-Preservation**: Hardened against disrupting the developer's working IDE session.
- [x] **No Silent Errors**: All fallible operations must propagate or route to `useErrorStore` / `AppError`.
- [x] **FIFO Invariant**: All prompt queue operations enforce `ORDER BY created_at ASC, id ASC`.
