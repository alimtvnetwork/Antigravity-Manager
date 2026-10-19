# Root Cause Analysis and End-to-End Testing Specification: Architecture & Systems Reasoning

## 1. Metadata & Identification
- **Title**: Root Cause Analysis and End-to-End Testing: Prompt Tree Discrepancies, Timestamp Alignment, Image Previews, and Cross-Machine Relay Parity
- **Slug**: root-cause-analysis-and-end-to-end-testing
- **Author**: @aukgit
- **Created**: 2026-10-19
- **Status**: Active Specification
- **Target Subsystems**:
  - `src/components/instances/PromptTreeView/` (Selection stability, image preview card rendering, turn count alignment)
  - `src-tauri/src/modules/repo_db/` (Transcript parsing, turn extraction, timestamp computation)
  - `src-tauri/src/modules/telegram_inbound/` (Cross-machine prompt relay, date/time acknowledgment, proof screenshot delivery)
  - `src-tauri/src/modules/ssh_manager/` & `src-tauri/src/modules/supabase_sync/` (GitMap Port 22 SSH mesh fallback when Supabase is paused)

---

## 2. User Request (Verbatim)

```markdown
# Root Cause Analysis and End-to-End Testing: high priority instruction, non-negotiable task

I've added the screenshot. You can see I have a running prompt, and the running prompt is very different from what you can show here, show there. I've added two big screenshots so that you can understand in the prompt tree. Your calculation and everything mostly wrong. You need to find the root cause. Find with date, time when you send it, the acknowledgment prompt with image. Image I cannot see yet, so that is a scam. You are performing for the long period of time. I hope you understand and get out of this scam business. If something is not working, tell me that it is not working. It's a pure wastage. You waste my time, not your, but everyone's time. That's not really cool. You need to test it here with end-to-end testing as much as possible. Try to see by using the machine how you can fix it. Also follow through the telegram steps to send prompt from one machine to another. Make sure that it works. Currently, we still cannot connect with two machines where GitMap can.
```

---

## 3. High-Level Architecture & Root Cause Taxonomy

```
+--------------------------------------------------------------------------------------------------+
|                                    Antigravity Manager Desktop GUI                               |
|                                                                                                  |
|   +--------------------------+   +-----------------------------------------------------------+   |
|   |   Sidebar Tree Panel     |   |                   Detail Pane & Viewer                    |   |
|   |                          |   |                                                           |   |
|   |   C244 (Instance Mgmt)   |   |   Header: C244 RUNNING 12 turns · 16h 18m                 |   |
|   |   +-- T1: Instance Mgmt  |   |   PID: 32 (45s elapsed on current turn)                   |   |
|   |   +-- ...                |   |                                                           |   |
|   |   +-- T12: Formula UI    |   |   Attached Images (4) [Clean Deduplicated Grid Cards]     |   |
|   |                          |   |   +--------+  +--------+  +--------+  +--------+          |   |
|   |   [Selection Stable]     |   |   | Img 1  |  | Img 2  |  | Img 3  |  | Img 4  |          |   |
|   |   [No Flapping on 30s]   |   |   +--------+  +--------+  +--------+  +--------+          |   |
|   +--------------------------+   +-----------------------------------------------------------+   |
+--------------------------------------------------------------------------------------------------+
                                        ^
                                        | (Tauri IPC / State Queries)
                                        v
+--------------------------------------------------------------------------------------------------+
|                                   src-tauri Rust Backend                                         |
|                                                                                                  |
|   +--------------------------+   +-----------------------------------------------------------+   |
|   |   repo_db / transcript   |   |                 Telegram Inbound & Remote Relay           |   |
|   |                          |   |                                                           |   |
|   |   1. Turns = USER_INPUT  |   |   1. Date & Time timestamp in Telegram ACK card           |   |
|   |      (Exclude internal)  |   |   2. Remote screenshot via Session 1 API / CDP            |   |
|   |   2. step_count != turns |   |   3. Tier 1: GitMap SSH Port 22 Mesh (192.168.1.3 <-> .20)|   |
|   |   3. URI Normalization   |   |   4. Tier 2: Supabase Command Queue (when active)         |   |
|   +--------------------------+   +-----------------------------------------------------------+   |
+--------------------------------------------------------------------------------------------------+
```

### 3.1 Defect Classification Matrix

| Defect ID | Subsystem | Symptom | Grounded Root Cause | Architectural Remediation |
|---|---|---|---|---|
| **BUG-01** | `PromptTreeView` | Title flapping & jumping between projects every 30s | `usePromptTreeState.ts` recreates callbacks every render without `useCallback`. The 30s sync interval triggers `loadTree()` which calls `performAutoSelection()` and defaults to the newest active project if ephemeral turn ref isn't found. | Stabilize callbacks with `useCallback`. Guard auto-selection with a stable selection key (`selectedConversationId`, `selectedTurnNumber`). Prevent background sync from overwriting active turn selection. |
| **BUG-02** | `PromptInstructionView` & `InstanceCardProjects` | Turn count says 16 turns, but only T1-T12 are rendered | `InstanceCardProjects.tsx:61` and `DetailHeader.tsx:471` use transcript line count (`c.step_count`) instead of actual prompt turns (`c.prompts.length`). | Always count turns as `c.prompts?.length || 1`. Label transcript event count separately if needed. |
| **BUG-03** | `PromptInstructionView` & `clipboard.ts` | Image count says 6 instead of 4, images invisible ("Image I cannot see yet") | (a) Unscoped aggregation across all turns + raw string matching against markdown regex duplicates count from 4 to 6. (b) `resolveImageSrc` has an off-by-one error on `file:///` (`slice(7)` instead of `slice(8)` on Windows `file:///C:/...`), and `<img onError>` has `style.display='none'`, vanishing broken images silently. | Scope image extraction to selected turn. Normalize file paths (strip `file:///`, decode percent-encoding). Fix slice offset. Provide visible image placeholder card with click-to-preview. |
| **BUG-04** | `PromptInstructionView` | Timestamp confusion (`16h 18m` vs `PID 32 45s`) | `16h 18m` is the cumulative lifespan of the entire conversation session (turn 1 to turn 16). `PID 32 45s` is elapsed runtime for the active process. | Add clear disambiguation in UI: "Session Span: 16h 18m (01:23 AM -> 05:41 PM)" vs "Active Turn Elapsed: 45s (PID 32)". |
| **BUG-05** | `telegram_inbound` | Telegram prompt ACK missing Date/Time and Proof Image | `injection.rs` formats ACK text without date/time. Proof shot via `CopyFromScreen` crashes in Session 0 on Windows OpenSSH. | Include formatted Local/UTC timestamps on all Telegram response envelopes. Bypass Session 0 via CDP screenshot or AGM local API proxy. |
| **BUG-06** | `supabase_sync` & `FleetMachinesTable` | AGM cannot connect across machines while GitMap can | GitMap uses direct peer-to-peer SSH (port 22). AGM relied on paused Supabase cloud endpoints (`pezjuuddecbyfmqxytrv.supabase.co`). | Promote GitMap SSH cluster nodes to primary tier-1 fleet discovery in `get_fleet_machines` and remote prompt relay. |

---

## 4. Invariants & Negative Guardrails
- **FIFO Dispatch Invariant**: All queries on prompt tables must maintain `ORDER BY created_at ASC, id ASC`.
- **No Silent Failures**: Never suppress image loading errors with `style.display = 'none'`. Always display an interactive fallback card.
- **Strict Relative Git Paths**: Zero absolute filesystem paths (`D:\...`, `C:\...`) in code, specs, and logs.
- **Process Immunity**: Never kill developer session PID 32 (`Antigravity.exe`).
- **No Credential Leaks**: Never print SSH private keys or bot tokens in chat or logs.
