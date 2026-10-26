---
name: telegram-onboarding-and-prompt-sending-issue
description: Autonomously diagnose, optimize, and resolve Telegram onboarding and prompt dispatch latency (achieving 1-2s response times), refactoring polling and notifier loops, eliminating backoff delays, and hardening Telegram prompt dispatch.
---

# Telegram Onboarding and Prompt Sending Optimization

## Mandates & Objectives
- **Sub-2-Second Telegram Response Latency**: Eliminate multi-second or multi-minute response latency across Telegram onboarding, commands (`/status`, `/start`, `/help`, `/instances`, `/accounts`), and prompt dispatch (`/send`, `/rerun`, text prompts).
- **Notifier & Broadcasting Loop Diagnostics**: Audit the background broadcasting pattern, polling interval (`long_polling` timeout vs sleep loops), broadcast notifier mutex contention, and queue dispatch pacing.
- **Immediate Task Dispatch**: Ensure inbound prompts from Telegram immediately notify the background queue drainer via `Notify` or instant trigger without waiting for 30s background ticks.
- **Onboarding Flow Resilience**: Resolve onboarding connection verification, webhook vs long-polling conflicts, token verification latency, and chat registration timeouts.
- **Strict Coding Guidelines & Verification**: Use monadic Results, typed errors, positive booleans, and relative paths.
