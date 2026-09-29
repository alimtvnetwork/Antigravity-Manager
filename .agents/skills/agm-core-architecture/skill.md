---
name: agm-core-architecture
description: Master architectural guide, subsystem index, and invariant directory for Antigravity-Manager. Use this skill as the primary gateway to navigate the 22 specialized AGM skills, core system invariants, directory-to-skill mappings, and end-to-end data flows.
---

# AGM Core Architecture & Subsystem Skill Directory

Master reference and architectural index for **Antigravity-Manager (AGM)** — an enterprise multi-protocol AI gateway, multi-instance IDE profile sandbox, autonomous account quota switcher, and native terminal automation platform.

---

## 1. High-Level System Topology

```
+----------------------------------------------------------------------------------------------------+
|                                    Inbound Client Protocols                                        |
|      OpenAI Responses       OpenAI Chat Completions       Anthropic Claude       Google Gemini      |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                Axum Proxy Gateway (Port 8045)                                      |
|    - Dual-stack socket2 IPv4/IPv6 listener (SO_REUSEADDR)                                          |
|    - Security Firewall: IP Filter (security.db), Curfew Windows & User Tokens (user_tokens.db)     |
|    - Headless/Docker SPA fallback to dist/index.html                                               |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                            Protocol Normalization -> Canonical IR                                  |
|   OpenAI Adapter (/v1/chat, /v1/responses)  Claude Adapter (/v1/messages)  Gemini Adapter (v1beta) |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                            Unified Pipeline Stage (Strictly Decoupled)                             |
|   1. Inbound Thinking Pipeline: Tool ID normalization, thought positioning (I1), signature mig (I4)|
|   2. Thinking Store Hydration: Restore thought signatures & causal chains from L1/L2 cache         |
|   3. Prompt Sanitizer: Strip billing headers, neutralize vendor intros, preserve code blocks       |
|   4. Deterministic Prefix Alignment: systemInstruction -> tools -> genConfig -> safety -> contents |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                                Upstream Fallback Ladder (rquest)                                   |
|               Daily (P1) ------------> Sandbox (P2) ------------> Prod (P3)                        |
|               (Fallback triggered exclusively on network transport errors, 408, 404, or 5xx)       |
+-------------------------------------------------+--------------------------------------------------+
                                                  |
                                                  v
+----------------------------------------------------------------------------------------------------+
|                       Outbound SSE Streaming & Token Diffusion (CanonicalUsage)                    |
|          OpenAI SSE Stream              Claude SSE Stream              Gemini Stream               |
+----------------------------------------------------------------------------------------------------+
```

---

## 2. Directory-to-Skill Mapping Matrix

When modifying any component in the repository, refer to the corresponding skill:

| Subsystem / Directory | Core Responsibilities | Specialized Skill |
|---|---|---|
| `src-tauri/src/proxy/` | Axum server, 4 protocol adapters, upstream fallback, token management | [`agm-proxy-engine`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-proxy-engine/skill.md) |
| `src-tauri/src/proxy/thinking_store.rs` | L1 DashMap & L2 SQLite cache, causal anchors, Gzip compression | [`agm-thinking-store`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-thinking-store/skill.md) |
| `src-tauri/src/modules/instance.rs`, `process.rs` | Workspace sandboxing, conscious PID matching, OS keyring bypass | [`agm-multi-instance-sandboxing`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-multi-instance-sandboxing/skill.md) |
| `src-tauri/src/modules/account.rs`, `oauth.rs` | OAuth refresh ladder, 5h rolling vs 7d quota calculations | [`agm-account-quota-switcher`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-account-quota-switcher/skill.md) |
| `src-tauri/src/modules/auto_switcher.rs` | Candidate scoring formula, cooldown guards, single-circle daemon | [`agm-auto-switcher-routing`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-auto-switcher-routing/skill.md) |
| `src-tauri/src/bin/agm.rs` | 11,000-line native CLI, Dual Sequence tree `[AGM:P001 \| GM:#1]` | [`agm-cli-suite`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-cli-suite/skill.md) |
| `src-tauri/src/modules/json_envelope.rs` | Universal Two-Tier envelope `{ attributes, data }`, format inspector | [`agm-json-envelope-format-inspector`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-json-envelope-format-inspector/skill.md) |
| `src-tauri/src/modules/repo_db.rs`, `backup_prompts_db.rs` | Prompt state machine, Base64 image payload preservation | [`agm-prompt-backup-resumption`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-prompt-backup-resumption/skill.md) |
| `scripts/test-instance-e2e.ps1`, `tests/` | Quarantined local-only tests (`#[ignore]`, `RUN_TEMP_E2E=1`) | [`agm-e2e-testing-verification`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-e2e-testing-verification/skill.md) |
| `scripts/dev-tool-clear.ps1`, `agy_cleaner.rs` | Process hygiene, conversation pruning (`--keep N`), cache clearing | [`agm-process-cleaner-hygiene`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-process-cleaner-hygiene/skill.md) |
| `src-tauri/src/modules/notification_hub.rs` | Multi-channel broadcast, subject headers `[AGM vX.Y.Z \| node \| ip]` | [`agm-telemetry-notification-hub`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-telemetry-notification-hub/skill.md) |
| `src-tauri/src/modules/telegram_inbound.rs`, `email_inbound.rs` | Remote daemons, sliding 10s debounce, 3,800-char chunking | [`agm-inbound-remote-control`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-inbound-remote-control/skill.md) |
| `src/` (React 19, Zustand, Tailwind) | Dual-mode Tauri IPC / Web REST fallback, 300px MiniView widget | [`agm-frontend-react-tauri`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-frontend-react-tauri/skill.md) |
| `src-tauri/src/modules/delegate_updater.rs` | 3-stage self-delegating CLI updater, Win32 console spawning | [`agm-delegate-updater`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-delegate-updater/skill.md) |
| `install.ps1`, `install.sh`, `Casks/` | Cross-platform installers, multi-version ladder, aria2c acceleration | [`agm-installer-distribution`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-installer-distribution/skill.md) |
| `src-tauri/src/modules/supabase_schema.rs`, `workspace_lease_manager.rs` | PostgreSQL cluster sync, 90s TTL distributed workspace leases | [`agm-supabase-sync-cluster`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-supabase-sync-cluster/skill.md) |
| `src-tauri/src/modules/security_db.rs`, `user_token_db.rs` | Reverse proxy IP firewall, CIDR whitelist/blacklist, curfew windows | [`agm-security-network-guard`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-security-network-guard/skill.md) |
| `src-tauri/src/modules/email_sender.rs`, `email_vault_db.rs` | AES-256-GCM encrypted password vault, machine GUID salt | [`agm-email-template-syntax`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-email-template-syntax/skill.md) |
| `src-tauri/src/modules/ssh_manager.rs` | Native SSH key generation, mesh deployment, managed `~/.ssh/config` | [`agm-ssh-key-management-parity`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-ssh-key-management-parity/skill.md) |
| Fast-Forward Shortcut Engine | Dynamic keybindings, active window restore, crash watchdog | [`agm-fast-forward-shortcuts`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-fast-forward-shortcuts/skill.md) |
| Split SQLite Architecture (`repo_prompts.db`, `proxy_logs.db`, etc.) | WAL mode concurrency, 5000ms busy timeout, schema migrations | [`agm-split-sqlite-architecture`](file:///d:/work/Antigravity-Manager/.agents/skills/agm-split-sqlite-architecture/skill.md) |

---

## 3. Core Architectural Invariants

Every change made to this repository MUST strictly uphold the following invariants:

1. **Pipeline First & Protocol Decoupled**:
   - The 4 protocol adapters function purely as parameter normalizers and payload mappers.
   - Business logic (thinking injection, tool call normalization, sanitization, prefix stability) belongs exclusively to the pipeline stage.
2. **Process Discrimination & Host IDE Immunity**:
   - Maintenance commands (clearing, updating, stopping) targeting the default IDE must NEVER touch processes belonging to isolated sandboxes (`.antigravity_tools`, `/instances/`).
   - Instance operations must resolve conscious PIDs matching the exact `--user-data-dir` argument. The developer's primary IDE is 100% immune from unexpected termination.
3. **OS Keyring Bypass via SSH Emulation**:
   - Secondary instances bypass Windows Credential Manager / macOS Keychain by injecting `SSH_CONNECTION`, `SSH_CLIENT`, `SSH_TTY`, `WSL_DISTRO_NAME`, and `DOCKER_CONTAINER`.
   - Marker files (`*-keyring-unavailable`) force the Go language server to read isolated file-based tokens (`%USERPROFILE%\.gemini\jetski-standalone-oauth-token`) and local `state.vscdb`.
4. **Thought Block & Signature Invariants**:
   - **I1**: Parts with `thought: true` must reside strictly at `parts[0]` in model turns.
   - **I2**: Gemini `thought: true` parts must NEVER carry signatures. Signatures belong on the first non-thought anchor part (`text` or `functionCall`).
   - **I3**: Foreign signatures must be stripped across protocols (Claude signatures stripped for Gemini, Gemini signatures stripped for Claude).
   - **I4**: Trivial placeholder thoughts (`...`, `·`, `[undefined]`) must be dropped, and their cryptographic signatures migrated to the turn's anchor part.
5. **Zero Prompt Loss Invariant**:
   - Before an instance is terminated or an account is switched, running and queued prompts (including Base64 multimodal images) must be backed up to `backup_prompts.db` and `.antigravity_resume_task.json`.
   - Following restart, prompts are automatically verified and re-injected.
6. **Zero-CI Quarantine for Heavy Tests**:
   - Any test that spawns processes, modifies SQLite, or tests account switching must be decorated with `#[ignore = "local_only_e2e"]` and require `RUN_TEMP_E2E=1`.
   - Automated CI pipelines must compile test targets but NEVER execute OS-destructive or heavy tests.
7. **Strict Release Attribution & Contributor Isolation**:
   - All release notes in `changelog.md` and GitHub Releases must attribute strictly `@aukgit` (`(Thanks to @aukgit)`).
   - Automated workflows sanitize release notes to prevent auto-generated external contributor pollution.
8. **Universal Two-Tier JSON Envelope**:
   - File exports and sync payloads must follow the `{ "attributes": { "type", "version", ... }, "data": ... }` schema via `json_envelope.rs`.
   - Imports must support `unpack_envelope()` with backward-compatible legacy flat JSON fallback.
9. **SSH Fleet Key Parity & Windows ACL Hardening**:
   - Managed `~/.ssh/config` modifications must be bounded strictly within `# --- BEGIN AGM MANAGED SSH CONFIG ---` blocks, with daemon directives sanitized.
   - On Windows, administrator keys must install to `%ProgramData%\ssh\administrators_authorized_keys` with `icacls` ACL permissions restricted to `SYSTEM` and `BUILTIN\Administrators`.
10. **Real-Time Task Heartbeat Liveness**:
    - Running tasks must execute the 5-second prompt heartbeat runner (`scripts/prompt_heartbeat_runner.py`), updating `.antigravity_goal_prompt.log` and tracking PID files before and after account rotations.

---

## 4. Pre-Flight Verification Checklist

Before opening a PR, pushing commits, or creating a release tag, run:

```bash
# 1. Rust Formatting
cd src-tauri && cargo fmt -- --check

# 2. Rust Clippy Gate (includes compilation and feature checks)
cd src-tauri && cargo clippy --all-targets --all-features

# 3. Frontend Build Gate
npm run build

# 4. Optional: Run Quarantined Local E2E Suite (local development only)
$env:RUN_TEMP_E2E="1"; cd src-tauri; cargo test -- --ignored test_local_e2e_instance_switch_and_prompt_restore
```
