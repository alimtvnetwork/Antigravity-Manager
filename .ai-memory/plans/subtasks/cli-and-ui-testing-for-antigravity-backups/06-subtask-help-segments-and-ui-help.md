# Subtask 06: Help Segments, CLI Help Systems, and UI Contextual Help

**Module:** `cli-and-ui-testing-for-antigravity-backups`
**Version:** 4.239.0
**Updated:** 2026-10-20
**Status:** Ready for Execution

---

## 1. Context & Objectives

This subtask integrates comprehensive command-line interface (CLI) help documentation across all prompt backup, restoration, and running inspection commands, while elevating the user interface with contextual tooltips, segmented pill capsules, and dark-glass truncation callouts.

### Core Targets:
- CLI Prompts Help: `src-tauri/src/modules/cli/prompts_help.rs`
- CLI Suite Help: `src-tauri/src/modules/cli/help.rs`
- CLI Argument Routing: `src-tauri/src/modules/cli/args.rs` & `src-tauri/src/modules/cli/prompts.rs`
- UI Callouts & Tooltips: `src/components/instances/PromptTreeView/TruncatedContextCallout.tsx` & `ConversationNode.tsx`

---

## 2. Technical Invariants & Guardrails

1. **CLI Flag & Envelope Standard**:
   - Every CLI subcommand must support `--help` (`-h`), `--json` (`-j`), and `--instance` (`-i`).
   - Terminal help output must strictly format into structured 80-character bordered banners.
   - Machine-readable `--json` errors must output compliant `CliEnvelope` structures.
2. **UI Dark-Glass & Capsule Styling**:
   - Truncated prompt previews must never output raw `<truncated N bytes>` markers; they must render interactive dark-glass banners (`⚡ [Omitted N words of prompt context - Click to inspect/expand]`).
   - Adjacent toolbar controls must wrap inside contiguous segmented pill capsules (`rounded-full`, shared border, dark-glass background).
   - Any inline `<select>` controls inside segmented capsules MUST use transparent borderless styling (`bg-transparent border-0 focus:ring-0 focus:outline-none`) to prevent nested white pill-in-pill visual defects.
3. **Strict Relative Git Paths**:
   - Zero absolute paths across all documentation and error strings.

---

## 3. Step-by-Step Implementation Plan

### Step 1: Update CLI Prompts Help (`src-tauri/src/modules/cli/prompts_help.rs`)
Update `print_prompts_cli_help()` to document running prompts, backup, and restore:
```rust
pub(crate) fn print_prompts_cli_help() {
    println!("================================================================================");
    println!("             Antigravity-Manager: Prompt Dispatch & Management CLI              ");
    println!("================================================================================");
    println!("Usage:");
    println!("  agm prompts <subcommand> [options]");
    println!("  agm running [options]");
    println!("  agm tree [options]");
    println!();
    println!("Aliases: prompts, prompt, tree, running");
    println!();
    println!("Subcommands:");
    println!("  ls, list [--all] [-i <inst>] [--json]      List active & queued prompts");
    println!("  running [-i <inst>] [--json]               Inspect running prompts with dual sequence IDs");
    println!("  tree [--running] [-i <inst>] [--json]      Hierarchical prompt origin & project tree");
    println!("  send, run <prompt> [-i <inst>] [-r <repo>] Dispatch prompt via Smart Process Cache");
    println!("  enqueue, queue <prompt> [-i <inst>]        Enqueue prompt into FIFO execution queue");
    println!("  backup running [inst] [--json]             Snapshot running prompts & multimodal images");
    println!("  restore running [inst] [--json]            Restore prompts into FIFO queue & seed task");
    println!();
    println!("Options:");
    println!("  --instance, -i <id>       Target instance profile (default: active/default)");
    println!("  --repo, -r <path>         Target workspace directory (default: current directory)");
    println!("  --all, -a                 Inspect prompts across all registered instances");
    println!("  --limit, -l <N>           Limit restored prompts or preview words");
    println!("  --json, -j                Output machine-readable JSON envelope");
    println!("================================================================================");
}
```

### Step 2: Update CLI Suite Master Help (`src-tauri/src/modules/cli/help.rs`)
Update `print_all_cli_help()` to highlight `agm running`:
```rust
    println!("  prompts, prompt, running     Inspect, queue, backup & restore multimodal prompts");
```

### Step 3: Argument Dispatching (`src-tauri/src/modules/cli/args.rs` & `prompts.rs`)
In `args.rs`, register direct aliases:
- `"running" | "running-prompts" | "rp"` $\rightarrow$ `handle_prompts_subcommand(&["running".to_string()], cmd.as_str())`.
In `prompts.rs`, handle `running`, `backup running`, and `restore running`:
- Route `backup running` to `backup_active_running_prompts_for_instance()`.
- Route `restore running` to `restore_active_running_prompts_for_instance()`.
- Output dual sequence format: `[AGM:P001 | GM:#1]`, instance ID, word-bounded preview, and image indicator (`1 image` / `N images`).

### Step 4: UI Tooltips & Dark-Glass Callouts (`src/components/instances/PromptTreeView/`)
- In `TruncatedContextCallout.tsx`, ensure support for both byte-level truncation and word-limit truncation.
- Render callout banner:
```tsx
<div 
    onClick={onExpand}
    className="flex items-center gap-2 px-3 py-1.5 my-1.5 rounded-lg bg-zinc-900/80 dark:bg-zinc-950/80 border border-zinc-700/50 hover:border-cyan-500/50 cursor-pointer text-xs text-zinc-300 transition-all shadow-sm group"
>
    <span className="text-amber-400 group-hover:scale-110 transition-transform">⚡</span>
    <span>
        Omitted <strong className="text-white font-mono">{omittedWords}</strong> words of prompt context — <span className="underline decoration-dotted text-cyan-400">Click to expand</span>
    </span>
</div>
```

### Step 5: Contiguous Segmented Pill Styling
- Wrap toolbar controls in `src/components/instances/PromptTreeView/TreePanel.tsx` in a single contiguous container:
  `flex items-center rounded-full border border-zinc-700/60 bg-zinc-900/60 p-0.5 backdrop-blur-md`.
- Ensure all inner `<select>` elements use `bg-transparent border-0 focus:ring-0 focus:outline-none`.

---

## 4. Verification & Testing Protocol

1. **CLI Help Flag Verification**:
   - Execute:
     ```bash
     agm prompts --help
     agm running --help
     agm backup --help
     ```
   - Assert all output banners match the standard 80-character schema.
2. **JSON Envelope Parity**:
   - Execute `agm running --json` on an empty queue.
   - Assert returns `status: "ok"`, `version: "2.0"`, `data: { running_prompts: [] }`.
3. **UI Visual Integrity Check**:
   - Verify zero nested white pill-in-pill artifacts in Prompt Tree View.
