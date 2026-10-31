# Subtask 04: Modern Dark-Glass UI Redesign and Telegram Media Enhancements

## 1. Overview
- **Subtask ID**: `04-modern-dark-glass-ui-and-telegram-enhancements`
- **Component**: `src/components/instances/PromptTreeView/` & `src-tauri/src/modules/telegram_inbound/`
- **Owner**: Full-stack Worker
- **Status**: COMPLETED

## 2. Objectives
1. Modernize Prompt Tree View:
   - Apply dark-glass depth (`bg-[#0c0817]/95 backdrop-blur-2xl border border-purple-900/40 shadow-2xl`).
   - Wrap adjacent controls in segmented pill capsules (`rounded-full`, shared border).
   - Eliminate flat styling, replace with modern gradients, subtle glows, and high-contrast typography.
2. Upgrade Telegram Remote Control:
   - Format prompt status cards with rich HTML and inline buttons (`[Inspect]`, `[View Images]`, `[Rerun]`).
   - Add prompt image sending via Telegram (`send_telegram_photo`) so attachments can be viewed and copied directly on mobile/desktop Telegram.

## 3. Verification Criteria
- [x] Prompt Tree View modal, header, toolbar, conversation items, and detail header elevated to obsidian dark glass (`PromptTreeViewModal.tsx`, `ModalHeader.tsx`, `ConversationNode.tsx`).
- [x] Telegram inbound `/inspect` command registered and inline buttons (`[🖼️ Send Images]`, `[↻ Rerun]`, `[🔍 Verify]`, `[📋 Copy]`) hooked up in `reports_b.rs`, `processing.rs`, `daemon.rs`, and `send_flow.rs`.
- [x] Direct Telegram image delivery via `send_telegram_photo` delivering prompt turn image attachments directly to Telegram chats.
