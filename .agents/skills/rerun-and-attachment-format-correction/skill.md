---
name: rerun-and-attachment-format-correction
description: "Autonomously enforce strict FIFO prompt queueing on rerun operations and format prompt attachments with the canonical 3-tier reference pattern ([Title | Slug | Seq](file;<path>), @[<path>], and @[.agents/skills/execute-parent-task-with-n-steps-v6]) across Antigravity Manager."
---

# Rerun and Attachment Format Correction

Autonomously ensure all prompt rerun operations across UI and Telegram inbound handlers route strictly through the FIFO queue (`repo_db::enqueue_prompt_for_instance` / `enqueuePrompt` with `status: 'queued'`) and standardize all prompt attachment blocks to the canonical 3-tier format.

## Instructions

1. **Rerun Enqueue Invariant**: Never send or inject reruns directly (`sendPromptNow`, `execute_prompt_injection`, or immediate spawn). All reruns must enqueue into the FIFO database queue.
2. **Canonical Attachment Syntax**: 
   - Never write `@[file;<path>]` inside `@[]` brackets.
   - Attachments and task references must follow this exact 3-tier format with normalized forward slashes:
     ```markdown
     [Title | Slug of the task | Sequence 01/02/xx](file;C:/path/to/media.png)
     @[C:/path/to/media.png]

     @[.agents/skills/execute-parent-task-with-n-steps-v6]
     ```
   - Parsers must extract clean forward-slash paths from both `[...](file;...)` and `@[...]`, supporting backwards compatibility while producing strictly canonical output.
3. **No Build / Targeted Checks**: Targeted checks and unit tests only on modified files.
4. **Strictly Relative Git Paths**: Enforce relative repository paths across all documentation and commit messages.
5. **GitMap Primacy**: Search exclusively via GitMap (`gitmap aum search`, `gitmap find`, `gitmap cat`).
