# Master Audit Ledger: 144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement

**Task Slug:** `144-prompt-tree-view-ui-progress-bar-and-agm-cli-enhancement`  
**Target Application:** Antigravity-Manager (`agm`, GUI, Proxy, Backend)  
**Execution Protocol:** `execute-parent-task-with-n-steps-v6` (A = 2, H = 2)  
**Lead Architect:** Antigravity Pairing Agent  

---

## 1. Audit Summary & Root Cause Matrix

| ID | Component | Defect Observed | Root Cause | Target Remediation |
|---|---|---|---|---|
| **D1** | `PromptTreeViewModal.tsx` | Loose, uncompact header buttons; clashing multi-color buttons; disjointed Close button | Header toolbar built with disjointed `rounded-[4px]` and `rounded-full` divs without unified pill capsule wrapper | Consolidate actions into contiguous segmented dark-glass capsules (`rounded-full`, shared border, subtle divider lines, glass backdrop) per `AGENTS.md` |
| **D2** | `PromptTreeViewModal.tsx` | Cannot distinguish human prompts from AI subagent instructions | No discriminator field in `AgmConversationNode`; transcript inspector does not classify role/source | Add heuristic tagger & badges (`User Prompt` vs `AI Subagent Instruction`) on conversation nodes and preview header |
| **D3** | `PromptTreeViewModal.tsx` | In-flight / running indicator missing on collapsed project cards | Project card header only checks `is_running` for title color; no pulse badge or counter on project summary | Add emerald pulse badge and running count (`N running`) to parent project cards |
| **D4** | `PromptTreeViewModal.tsx` | Repeated identical prompts cause noisy duplicate entries | All conversation nodes rendered 1:1 without grouping | Implement prompt grouping by normalized prompt text hash; render repeat count badge (`x4`) and collapsible sub-runs |
| **D5** | `PromptTreeViewModal.tsx` | Ugly truncated byte markers in preview; no byte-level context | Words capped at 120 with raw ellipses; no byte size or clean expansion callout | Render sleek truncation indicator: `⚡ [Truncated: X.X KB preserved - Click to Expand Full Text]` |
| **D6** | `PromptTreeViewModal.tsx` | Missing single-prompt Export action | Only bulk Backup exists; no Export to Markdown/JSON file | Add dedicated `Export` button in the segmented action capsule with format selection (`.md` or `.json`) |
| **D7** | `QuotaProgressBar.tsx` | Cluttered with 11 milestone balls; checkmarks render even when quota is empty | Default checkpoints has 11 entries `[100, 90, ..., 0]`; checkmark SVG renders unconditionally | Cap balls at $\le 5$ (default `[100, 75, 50, 25, 0]`); replace checkmark with numerical percentage typography when quota $< 25\%$ |
| **D8** | `QuotaProgressBar.tsx` | Background gradient lacks critical-zone depth and contrast | Gradient starts from light `rose-500` | Anchor gradient with deep dark-red left edge (`#520808` to `rose-600`) and crisp white accents |
| **D9** | `InstanceCard.tsx` / `InstanceTable.tsx` | Excessive whitespace in cards and scrollbars | Loose vertical padding and default browser scrollbars | Thin scrollbars (`scrollbar-thin`), tighten vertical card gap, improve dark theme contrast |
| **D10** | `agm.rs` / `cli.rs` | Missing comprehensive CLI verb coverage for instance, prompt, and system maintenance | CLI commands grew organically without structured 7-domain hierarchy | Formalize and expand 46 AGM CLI commands mirroring modern `gitmap` CLI workflows |
| **D11** | `server.rs` / `http_api.rs` | Instances and prompt tree management not exposed via authenticated REST endpoints | Only Tauri IPC exists for instance lifecycle and prompt tree retrieval | Design and mount authenticated REST endpoints (`Authorization: Bearer <token>`) on port 8045 under `/api/*` |
| **D12** | E2E Testing | Need verification of instance creation, prompt execution, and account switching | Manual testing prone to regressions | Author structured E2E test protocol verifying instance continuity, prompt capture, and CLI parity |

---

## 2. Invariant Checklist
- [x] **UI Capsule Rule:** Wrap adjacent toolbar actions into contiguous segmented pill capsules (`rounded-full`, shared border, subtle divider lines, dark-glass styling).
- [x] **Ball Cap Rule:** Never render more than 5 milestone checkpoint balls on any quota progress bar.
- [x] **Critical Threshold Typography:** Replace checkmarks with explicit `{percentage}%` when quota drops below 25%.
- [x] **Strict Relative Paths:** All documentation, logs, and CLI commands must reference relative paths.
- [x] **CLI & GUI Parity:** Operations available in GUI must be supported in AGM CLI and secure REST endpoints.
- [x] **Atomic GitMap Release:** Single final atomic commit via `gitmap cpf`.
