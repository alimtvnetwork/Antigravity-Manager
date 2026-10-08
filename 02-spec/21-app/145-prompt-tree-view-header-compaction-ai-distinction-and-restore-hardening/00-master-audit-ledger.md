# Master Audit Ledger: 145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening

**Task Slug:** `145-prompt-tree-view-header-compaction-ai-distinction-and-restore-hardening`  
**Target Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)  
**Execution Protocol:** `execute-parent-task-with-n-steps-v6` (A = 2, H = 2)  
**Lead Architect:** Antigravity Pairing Agent  

---

## 1. User Request (Verbatim)

```text
# High Priority Instruction

You can see that this is a serious bug where I have my prompts running, but I don't have the results of anything that is running there. I could not see the prompts, and I could not open the prompts in the main window or whatever I have sent. And there are prompts which I have not executed. Can this distinguish what is prompt, what is not prompt? Because some of those are prompt, some of those are not. It's by the AI instruction. So we need to distinguish those, and those could be a sub point or things like that that you need to work on in the UI. I do see that the prompts bit showed up, but still the header display for the prompts tree view, it's very much bad. It does not have the UI/UX. Seems like broken stuff. The menus are not compact, buttons are not compact. I do see that you have most of the text displayed. That's all right. Appreciate it. You should have export button as well, copy button as well. Copy with image. If I do that, can go there. I don't have the image. Images are also there. That's nice. That's good. The copy button is working, but still, it looks broken, especially the header section. That needs to be compacted, the design issue and things like that, you need to work on it. And also there are things which are not prompt, but showing up. You need to work on it. Also, the preview you need to improve. In between, it shows truncated bytes, which we need to fix in terms of the display, I think. Can you help with that? Improve the preview. And also, if a prompt is running, we need to have some indicator that it is a running prompt or in queue prompt. We need to have that. It is not there yet. So I think you need to look back and look deep, the prompt tree view, how the prompts are collecting, taking the prompts backup, restoring those. I think these are still very limited. You need to work on it very hard. Try to find the root cause. Try to write the spec by yourself, in details. Like the testing, UI/UX specs, make it more professional.
```

---

## 2. Audit Summary & Root Cause Matrix

| ID | Component | Defect Observed | Root Cause | Target Remediation |
|---|---|---|---|---|
| **D1** | `PromptTreeViewModal.tsx` | Header display is broken, cluttered, uncompact; menus & buttons have loose margins and mismatch styles | Header and sub-headers use fragmented `rounded-[5px]`, loose paddings, redundant copy buttons, and disjointed toolbar rows violating `AGENTS.md` capsule rules | Re-architect the entire header and preview toolbar into unified, ultra-compact segmented dark-glass pill capsules (`rounded-full`, shared border, subtle divider lines, glass backdrop, tight typography) |
| **D2** | `PromptTreeViewModal.tsx` & `repo_db.rs` | Non-prompts (AI subagent instructions, tool call steps, system events) show up alongside user prompts | Conversation collector blindly treats all turns in `transcript.jsonl` / `conversation_summaries.db` as prompt root nodes without deep origin classification | Implement strict 3-tier classification: `User Prompt` (Primary root), `AI Subagent Instruction` (Secondary child/sub-point), and `System/Tool Step` (suppressed from root or tagged); add visual badges and nested sub-point UI |
| **D3** | `PromptTreeViewModal.tsx` & `repo_db.rs` | Running and in-queue prompts lack prominent live indicators | Project headers and conversation nodes only show text without pulsing visual indicators; queue state is indistinct from idle | Add animated emerald pulse badges for running prompts (`● RUNNING`), amber clock badges for in-queue (`⏳ QUEUED`), live step execution summary, and running counts on project cards (`N running`) |
| **D4** | `PromptTreeViewModal.tsx` & `instance.rs` | Cannot see results of running prompts or focus prompt in main IDE window | Running prompt preview does not stream live transcript turns or tool outputs; "Focus IDE" does not guarantee window foreground activation or workspace path matching | Connect real-time transcript step polling to preview pane for active prompts, display latest tool actions, and harden `focusOrLaunchInstance` with exact folder target |
| **D5** | `PromptTreeViewModal.tsx` | Preview shows raw ugly byte markers like `<truncated 5301 bytes>` in the middle of text | Transcript logging inserts raw `<truncated ... bytes>` markers which are not rendered as rich UI components | Parse truncation markers into a sleek, dark-glass callout banner: `⚡ [Truncated: X.X KB preserved by log - Click to view full text]` with interactive smooth expansion |
| **D6** | `PromptTreeViewModal.tsx` | Export and Copy with Image capabilities incomplete or clunky | Only basic copy exists; image copying does not handle rich clipboard data or embedded base64/remote assets | Implement dedicated compact Export button (.md and .json) and "Copy with Image" (HTML clipboard + plain text + automatic image download fallback) |
| **D7** | `repo_db.rs` & `server.rs` | Prompt collection, backup, and restore are limited and lack FIFO queue guarantees | Backup only serializes flat active prompts; restore does not enforce FIFO insertion into `.antigravity_resume_task.json` across multi-instances | Harden prompt collection with dual-source transcript parsing, enforce strict FIFO order in backup/restore, and write comprehensive E2E test |

---

## 3. Invariant Checklist
- [x] **UI Capsule Rule:** Wrap adjacent toolbar actions into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling) per `AGENTS.md`.
- [x] **Prompt Origin Distinction:** Strictly distinguish root User Prompts from AI Subagent Instructions and System/Tool events.
- [x] **Running & Queue Indicators:** Prominently display glowing emerald pulse badges for running prompts and amber badges for queued prompts.
- [x] **Truncation Hygiene:** Never render raw `<truncated N bytes>` tags as ugly text; render sleek interactive callout banners.
- [x] **Copy & Export Parity:** Support single-prompt export (.md, .json) and rich copy with embedded image handling.
- [x] **FIFO Restore Preservation:** Prompts restored from backup must preserve exact sequence and FIFO injection.

