---
name: button-functionality-and-testing-instructions
description: >-
  Specialized skill for diagnosing, repairing, and verifying the send/enqueue buttons with image forwarding, OneNote/Antigravity binary image and base64/file-path clipboard copying, multi-image batch copy, prompt title header sizing/cleaning, complete database reset & regeneration, Telegram prompt sending verification, and GitMap fleet mesh SSH node account alignment in Antigravity-Manager.
---

# Button Functionality and Testing Instructions

This skill governs the end-to-end diagnosis, implementation, and multi-stage testing of:
1. **Send & Enqueue Buttons**: Ensuring prompt sending (both immediate send and queueing) operates reliably with or without images, across single and running instances.
2. **Clipboard Pure Image & Prompt-with-Image Copying**: Ensuring image copying writes actual binary PNG image data (supported by OneNote, Antigravity, Discord, and Slack) as well as fallback markdown referencing local file paths or base64 data URIs. Batch copy of multiple images at once.
3. **Prompt Tree Title & Header Polish**: Eliminating prefixes before the title (removing redundant turn badges from the title header line) and enlarging the conversation title typography.
4. **Tree & Cache Reset**: Completely wiping the prompt tree cache and SQLite database, triggering a clean transcript scan and fresh tree reconstruction.
5. **Fleet SSH & Lead Node GitMap Linkage**: Ensuring SSH mesh nodes match GitMap metadata correctly, resolving host/user accounts accurately.
6. **E2E Testing & Minor Release Ceremony**: Creating fresh test instances, executing real test workflows, validating CI/CD via `gitmap b -t`, and cutting a minor release.
