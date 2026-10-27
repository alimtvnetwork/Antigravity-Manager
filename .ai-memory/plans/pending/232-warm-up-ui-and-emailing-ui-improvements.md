# Master Plan: Warm Up UI and Emailing UI Improvements (Task 232)

**Slug:** `warm-up-ui-and-emailing-ui-improvements`  
**Sequence:** 232  
**Status:** `PENDING`  
**Related Specs:**  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/01-architecture-spec.md`  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/02-warmup-ui-improvements-spec.md` (Improvements 01–50)  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/03-emailing-ui-improvements-spec.md` (Improvements 51–100)  
- `02-spec/21-app/232-warm-up-ui-and-emailing-ui-improvements/04-verification-and-subtask-spec.md`  
**Related Skill:**  
- `.agents/skills/warm-up-ui-and-emailing-ui-improvements/skill.md`  

---

## User Request (Verbatim)

```text
Can you please work on the warm up UI and emailing UI? Think of more different ways. Let's do it like this. First, let's plan 100 different ways that we can improve the UI with the specifications, writing every detail in the spec folder so that any AI can follow through and implement it. So every time I do that, you'll find out 100 different improvements in the UI and how the CLI can be better. So every different way. You can also get inspiration from the `Antigravity Manager or Antigravity Tools` tool and also Gitmap. Can you please do that for me? And also the design systems read from coding guidelines.

## task: Warm Up UI and Emailing UI Improvements
## slug: warm-up-ui-and-emailing-ui-improvements

# Actionable Items Must Follow Non-Negotiable

1. Git pull BEFORE starting any work — first try `gitmap pull <repo path>` (or run `gitmap pull` in repository folder); if that fails, fall back to regular `git pull` and resolve any merge conflicts so the repository is completely up to date and clean before starting.
2. Write spec under 02-spec/21-app/XX-warm-up-ui-and-emailing-ui-improvements/ and enqueue plan task in .ai-memory/plans/pending/XX-warm-up-ui-and-emailing-ui-improvements.md (subtasks in .ai-memory/plans/subtasks/XX-warm-up-ui-and-emailing-ui-improvements/) first, (xx should be figured out by the folder structure and running AI tools, xx is sequence 01, 02)
3. Search codebase exclusively via GitMap (gitmap aum search, gitmap find, gitmap cat, gitmap ps, gitmap py, gitmap llm train); TOTAL BAN on rg, ripgrep, grep, git grep, Select-String
4. Strictly use relative Git paths (02-spec/..., .ai-memory/..., cmd/...); only add the relative paths, never add the absolute path during your work, and ensure this is respected on the release page and in release notes as well
5. Plan 100 different ways to improve the UI with detailed specifications in the spec folder.
6. Use inspiration from `Antigravity Manager or Antigravity Tools` and Gitmap for UI improvements.
7. Ensure design systems align with coding guidelines.

## Must follow and spawn agent using

@[.agents/skills/execute-parent-task-with-n-steps-v6]

## Additional Instructions

- /plan first before doing work for improved work or less friction, faster execution for AI and coding without errors and following Coding Guidelines.
- /learn from @[.agents/skills/gitmap] skill to leverage GitMap high-speed search, toolchain discovery, and caching.
- Track in-flight file touches and prevent write collisions using GitMap RC Heatmap (`gitmap rc heatmap status` / `gitmap rc heatmap tree` / `gitmap rc heatmap live` / `gitmap rc heatmap sync`).
- Only add relative paths, never add absolute paths during work; this must be respected on release page and in release notes as well.
```

---

## Executive Overview

This master plan structures the implementation of **100 concrete improvements** spanning the Warm Up UI and Emailing UI across Antigravity Manager (`agm-ui`) and CLI (`agm`). Drawing deep inspiration from GitMap high-speed terminal tools and Antigravity Manager dark-glass design systems, these improvements transform passive background cron tasks into reactive, developer-first command centers with live telemetry, countdown clocks, account matrix controls, visual template designers, diagnostic ladders, and headless CLI parity.

---

## 10-Cluster Breakdown of 100 Improvements

### Warm Up UI & CLI Improvements (1–50)
- **Cluster 1: Smart Warmup Dashboard & Telemetry Visualizers (Improvements 01–10)**
  - Live token expenditure HUD, SVG heartbeat activity waveform, model latency sparklines (Gemini vs. Claude p50/p95), 4-tier origin warmup traffic badge (`WARMUP_PROBE`), execution drawer, weekly reset convergence radar, zero-usage verification callout, dark-glass header status capsule, in-flight pulse waves, and error diagnostic banner.
- **Cluster 2: Real-time Quota Reset Timers & Wave Animations (Improvements 11–20)**
  - Real-time 7-day countdown clocks (`T-04h:22m:15s`), universal synchronized ticker hook (`useWarmupResetTimer`), cold-start uninitialized account badge, reset window imminence glow, dual-layer rolling vs. weekly reset progress bars, time-zone normalized clocks, humanized relative reset labels, audio/haptic notifications, animated replenishment wave canvas, and stale quota warnings.
- **Cluster 3: Batch Warmup Orchestration & Account Matrix Controls (Improvements 21–30)**
  - Interactive account × model warmup matrix grid, tier-based warmup profiles, per-account opt-out toggles, dynamic priority model cascade, custom probe prompt editor, bulk model assignment popover, dry-run simulation mode, quota exhaustion lockout override, **surgical fix for missing `isWarmupConfirmOpen` modal in `AccountsDialogs.tsx`**, and multi-instance account warmup fleet sync.
- **Cluster 4: Model Scope Tuning, Sliders & Jitter Schedules (Improvements 31–40)**
  - Gaussian schedule jitter slider (±0–180s), staggered account batch size & delay controls, quiet hours window scheduler, adaptive exponential backoff on 429/503, caller IDE active preservation, background scan interval slider (1–15m), network connectivity gate, 3-strike circuit breaker on 5xx, model exclusion wildcard filters (`*-thinking-*`), and quota reserve guard threshold slider (default 5%).
- **Cluster 5: Warmup CLI Ergonomics & Terminal Visualizers via GitMap (Improvements 41–50)**
  - `agm warmup status` headless telemetry HUD, `agm warmup list` high-speed `termtable` account matrix, `agm warmup run` targeted terminal dispatcher, `agm warmup test` latency and token probe utility, `agm warmup config` headless manager, `agm warmup cooldown` pruning tool, `agm warmup history` SQLite audit stream, `agm warmup matrix` ANSI color-coded grid, `agm warmup watch` real-time TUI, and standardized `--json` envelope parity.

### Emailing UI & CLI Improvements (51–100)
- **Cluster 6: Unified Email Command Center & Health Diagnostics (Improvements 51–60)**
  - Live email activity & event stream HUD, mailbox connection health matrix, adaptive watcher cadence curve visualizer (60s idle -> 5s active), machine node & multi-instance target matrix, two-phase delivery receipt inspector (Phase 1 ACK vs. Phase 2 Result), email quota & rate limit gauge bar, inbound mailbox unread inspector & purge tool, automated failover circuit breaker card, dark-glass email status segmented capsule bar, and global notification snooze scheduler.
- **Cluster 7: Visual Email Template Designer & Dual-Theme Previews (Improvements 61–70)**
  - Visual email template designer with variable token palette, dual-theme side-by-side simulator (Outlook Light vs. Gmail Dark), pluggable brand theme token & CSS palette styler, template Jinja/Mustache syntax highlighter, HTML email size guard (<102KB Gmail clipping), interactive omission banners for massive CLI dumps, one-click test dispatch to developer mailbox, mobile (375px) vs. tablet vs. desktop responsive simulator, export template as HTML/MJML bundle, and event-to-template custom mapping rules engine.
- **Cluster 8: Inbound Remote Command Playground & Security Shields (Improvements 71–80)**
  - Inbound email command simulator & dry-run parser playground, SPF/DKIM/DMARC header verification security shield, HMAC-SHA256 command token & secret PIN authorization, 4-tier origin classification badges in receipts, granular per-command ACL checkboxes, interactive inbound audit log & incident replay drawer, command syntax auto-suggest & RFC subject line generator, multi-node wildcard broadcast simulator (`* | help`), command execution timeout & kill-switch controller, and caller instance self-preservation guard.
- **Cluster 9: 5-Stage Diagnostic Step Ladder & Self-Healing (Improvements 81–90)**
  - 5-stage diagnostic step ladder visualizer (DNS -> TCP -> TLS -> Auth -> Ping), automated DNS MX & port auto-discovery wizard, TLS certificate expiry & cipher suite inspector, interactive port connectivity & firewall socket probe, guided app password assistant, automated reconnect & 3-tier exponential backoff ladder, corrupted mailbox self-healing shield, granular SASL auth error classifier, dynamic network interface / VPN switch detector, and one-click diagnostic report exporter.
- **Cluster 10: Email CLI Management & Headless Fleet Parity via GitMap (Improvements 91–100)**
  - `agm email doctor` 5-stage ladder terminal diagnostic, `agm email watch --tail` interactive Lipgloss terminal HUD, `agm email test-inbound` headless command parser simulator, `agm email audit list` SQLite audit log filtering CLI, `agm email templates ls` & ANSI terminal previewer, `agm email send-custom` direct terminal notification dispatcher, `agm email vault export / import` headless encrypted migration, `agm email config get / set` key-value manager, GitMap macro automation engine inbound integration, and zero-storage action purge & cache cleaner trigger.

---

## Subtask Execution Roadmap

1. **Subtask 01 (`01-subtask-warmup-dashboard-and-visualizers.md`)**:
   - Scope: Improvements 01–20 (Smart Warmup Dashboard, Countdown Clocks, Waveforms, Tokens).
   - Target Components: `src/components/settings/SmartWarmup.tsx`, `src/components/warmup/`, `src/hooks/useWarmupResetTimer.ts`.
2. **Subtask 02 (`02-subtask-warmup-matrix-and-cli-ergonomics.md`)**:
   - Scope: Improvements 21–50 (Account Matrix, Jitter, Sliders, CLI commands, `AccountsDialogs.tsx` modal restoration).
   - Target Components: `src/pages/accounts/AccountsDialogs.tsx`, `src-tauri/src/modules/scheduler.rs`, `src-tauri/src/bin/agm/warmup_cmds.rs`.
3. **Subtask 03 (`03-subtask-email-command-center-and-diagnostics.md`)**:
   - Scope: Improvements 51–60, 81–90 (Unified Command Center, 5-Stage Diagnostic Ladder, Self-Healing).
   - Target Components: `src/components/settings/EmailNotificationSettings.tsx`, `src/components/email/DiagnosticLadder/`, `src-tauri/src/commands/email/conn_test.rs`.
4. **Subtask 04 (`04-subtask-email-template-builder-and-remote-playground.md`)**:
   - Scope: Improvements 61–80, 91–100 (Template Designer, Inbound Playground, Security Shields, Email CLI).
   - Target Components: `src/components/email/TemplateDesigner/`, `src/components/email/CommandPlayground/`, `src-tauri/src/bin/agm/email_cmds.rs`.

---

## Acceptance Criteria Checklist

- [x] 100 concrete improvements fully specified across Warm Up UI and Emailing UI.
- [x] All 10 clusters contain exactly 10 detailed improvements with complete contracts.
- [x] Root cause analysis and technical fix specified for missing `isWarmupConfirmOpen` modal in `AccountsDialogs.tsx`.
- [x] Zero absolute filesystem paths; strictly relative Git paths across all documents.
- [x] Zero builds and zero test runs during specification planning.
- [x] Full GitMap CLI terminal parity documented with `termtable` and `--json` contracts.
- [x] Design system alignment with 4-plane depth hierarchy, segmented pill capsules, and transparent dropdowns.
