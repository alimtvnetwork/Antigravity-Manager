---
name: rerun-and-attachment-format-correction
description: "Autonomously enforce strict FIFO prompt queueing on rerun operations and format bracketed prompt attachments with canonical file semicolon syntax (e.g. @[file;<path>]) across Antigravity Manager."
---

# Rerun and Attachment Format Correction

Autonomously ensure all prompt rerun operations across UI and Telegram inbound handlers route strictly through the FIFO queue (`repo_db::enqueue_prompt_for_instance` / `enqueuePrompt` with `status: 'queued'`) and standardize all bracketed attachment strings to include the `file;` prefix (`@[file;<file_path>]`).

## Instructions

1. **Rerun Enqueue Invariant**: Never send or inject reruns directly (`sendPromptNow`, `execute_prompt_injection`, or immediate spawn). All reruns must enqueue into the FIFO database queue.
2. **Canonical Attachment Syntax**: Ensure square/third brackets format attachments as `@[file;<file_path>]`. All extractors and parsers must seamlessly support `@[file;<path>]` while preserving backward compatibility for legacy `@[<path>]`.
3. **No Build / Targeted Checks**: Targeted checks and unit tests only on modified files.
4. **Strictly Relative Git Paths**: Enforce relative repository paths across all documentation and commit messages.
5. **GitMap Primacy**: Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`).
