# Completed Plan: prompt-tree-context-menu-and-telegram-fixes

## User Request (Verbatim)
```text
Fix the prompt tree right click

# High Priority Instruction

Can you please have a look at the prompt tree? When we are in the prompt tree, it has only one prompt which I cannot click and see. If I right-click on the prompt tree view, nothing happens. You should be able to show a custom menu. I mentioned import, export, rerun. These types of things should be there. Also on top of the project section as well. This is not done. Also, the prompt preview and the results section, input and output, I mentioned to move to the right side and change the naming so that you can optimize the placing. So far there are so many places from the header that waste some spaces. I think you could reduce those and move the header to the actual place. Move the menus all together in the top level in one place so that you can save some space. The UI looks clunky, and you did not fix the copy prompt button, which I requested you to put everything together on top. In the copy all button, top level, you did not fix it. It's been several times I'm requesting the same thing. Please make sure that you improve the UI professionally. Also, make sure that in the telegram, you verify the steps. Verify the steps means that when we select the first machine, we shouldn't be receiving the response when we're sending the prompt. Select a machine and then proceed further. We should not be receiving from multiple machines the response because we already selected the machine. That is something that you need to fix in your code. Another part here is that if I go inside the prompt selection, the problem is it gives the recent prompt, which is very wrong, and I cannot type it. Let's say I select a project, and then I should be able to also type it, and that option is not there. You should fix it. At the end, release with the minor bump. Make sure the Git conflict is resolved. There are no issues.
```

## Architectural Specifications & Delivery Artifacts
- **Architecture Spec:** [01-architecture-spec.md](../../02-spec/21-app/prompt-tree-context-menu-and-telegram-fixes/01-architecture-spec.md)
- **Component Spec:** [02-component-spec.md](../../02-spec/21-app/prompt-tree-context-menu-and-telegram-fixes/02-component-spec.md)

## Implementation Summary

### Frontend Prompt Tree, Context Menus & Layout Optimization (Worker 01)
1. **Right-Click Custom Context Menus & Portal Clamping:**
   - **`ProjectContextMenu.tsx`**: Added "Import into Project" and "Rerun Queued/All" actions with viewport boundary clamping (`createPortal` within `window.innerWidth - 240` and `window.innerHeight - 380`).
   - **`ProjectNode.tsx`**: Direct `onContextMenu` attachment to project header rows. Added context menu and keyboard navigation (`Enter`, `Space`, `tabIndex={0}`) to child subagents, with self-sufficient project export handlers (`exportProjectAsJson`, `exportProjectAsYaml`, `exportProjectAsSqlite`).
   - **`ConversationContextMenu.tsx`**: Added "Rerun Nudge" and "Import into Conversation" options with coordinate clamping.
   - **`ConversationNode.tsx`**: Added turn-level context menu triggers, keyboard accessibility, and active selection highlights (`bg-sky-500/10 border-l-2 border-sky-500 text-sky-400 font-medium`) for active conversation turns.
2. **Right-Side Tab Relocation & Vertical Space Recovery:**
   - **`DetailHeader.tsx`**: Replaced Row 2 with a unified `justify-between` toolbar:
     - Left: Unified action capsule (`[ Copy All | v ]`, `[ Send N | v ]`, `[ Rerun xN | v ]`).
     - Right: Primary navigation tabs (`[ Prompt Instruction | AI Results & Outputs ]`).
     - Added "Copy Full Prompt" into the top-level `[ Copy All | v ]` flyout.
   - **`DetailTabs.tsx`**: Removed the redundant 3rd row navigation tabs and borders to eliminate ~44px of wasted vertical height, streamlining into a compact sub-bar atop the prompt content.
   - **`PromptInstructionView.tsx`**: Removed duplicate bottom copy buttons across Preview and Raw modes, keeping only stats and Show Less/All toggle.
   - **`PromptTreeViewModal.tsx`**: Tightened vertical spacing, wired `previewTab`/`setPreviewTab` to `DetailHeader`, and connected `onImportProject` and `onRerunProject`.

### Backend Telegram Routing, Cluster Shielding & Prompt UX (Worker 02)
1. **Universal Current Machine Resolution & Collision Shielding:**
   - **`node_label.rs`**: Prioritized `local`, `localhost`, `local_ip`, `local_machine_label()`, and `detect_machine_name()` before scanning remote cluster nodes. Prevented cluster mesh entries from masking local machine execution.
   - **`send_flow.rs`**: Deduplicated `machine_entries()` using `has_current_machine` and `is_current_machine()` so only a single "This machine" button is presented. In `dispatch_send_prompt()`, replaced legacy `eq_ignore_ascii_case("local")` with `super::node_label::is_current_machine(machine)`.
   - **`injection.rs`**, **`open_shot.rs`**, and **`processing.rs`**: Standardized all machine targeting to `node_label::is_current_machine()`.
2. **Supabase Queue Payload Normalization (`target_node_id`):**
   - Populated `"target_node_id": target_node`, `"source": "telegram"`, and `"command_text"` across `injection.rs`, `open_shot.rs`, and `processing.rs`. Eliminated wildcard broadcast storms to all nodes in the fleet caused by omitted `target_node_id` defaulting to `'*'`.
3. **CLI Targeted Dispatch (`agm prompt <target> [flags] <prompt_text>`):**
   - In `cli/prompts.rs`, added targeted dispatch in `handle_prompts_subcommand()` when arguments do not match standard subcommands (`ls`, `tree`, `send`, etc.). Resolved targets against AGM sequence codes and live projects, enabling remote GitMap cluster SSH execution to succeed without falling back into Supabase.
4. **Token Parsing Target Preservation:**
   - In `injection.rs`, guarded insertion of `"default"` project at index 0 so it only executes when `filtered_tokens.len() < 2` and the first token is neither a known project nor a directory. Preserved custom project and conversation names.
5. **Prompt Selection & Direct Typing UX Improvements:**
   - In `send_flow.rs`, redesigned `destination_locked_reply()` with prominent dividers and clear text: `💬 Type your prompt below and send it as a plain chat message.`
   - In `on_conversation_picked()`, wrapped past prompts in Telegram Bot API 7.2+ `<blockquote expandable>` with caption `📜 Optional: Tap below to resend a past prompt instead:`, ensuring typing is front-and-center.
   - Renamed Step 3 button to `"✏️ Enter Project / Conv ID"`.
   - In `consume_send_prompt()`, added graceful fallback in `SendStage::TypeConversation` to route direct prompt text to the default project.

## Verification Evidence
- TypeScript Build: `npm run build` -> Exit Code 0 (0 errors)
- Cargo Rust Tests: `cargo test --lib modules::telegram_inbound::tests` -> 15 passed, 0 failed
- Cargo Fmt: `cargo fmt -- --check` -> Clean
