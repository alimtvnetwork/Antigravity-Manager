# Architecture Specification: UI and Functionality Enhancement Instructions

**Slug:** `ui-and-functionality-enhancement-instructions`  
**Status:** `ACTIVE`  
**Version:** `1.0.0`  
**Author:** `@aukgit`

---

## 1. Executive Architectural Overview

This specification establishes the architectural blueprints for resolving 7 critical interconnected subsystems within Antigravity-Manager:
1. **Instance-Specific Running Prompt Isolation (`repo_db` & `liveness.rs`)**:
   - Eliminates false-positive cross-instance prompt bleeding where a task running exclusively in Instance #2 causes Instance #1 (Default) to also render a running badge.
   - De-couples `%USERPROFILE%/.gemini/antigravity-cli` from the default GUI IDE instance directory resolution.
   - Populates `instance_seq_num` across conversation node builders, preventing `null` sequence fallback in frontend filtering.
   - Restricts untagged/empty instance specifiers in `observe.rs` from silently defaulting to the active GUI instance.
2. **Prompt Tree View Ergonomics & Reduced Button Rounding (`PromptTreeView`)**:
   - Replaces excessive `rounded-full` pill button styling across the entire prompt section with crisp, professional `rounded-md` / `rounded-[5px]`.
   - Compacts view mode toggles (`Input`, `Output`, `Expand Full`) into a single segmented horizontal capsule anchored on the right-hand side of the header.
   - Introduces a context menu for left-hand sidebar tree items (Project, Conversation, Turn), supporting copy actions, rerun, inspect, and export.
   - Elevates enqueued prompts into a permanent FIFO Queue panel directly beneath the running prompt view card.
   - Distinctively highlights selected conversations with high-contrast active styling.
3. **Rich Multimodal Copy & Image Management**:
   - Distinguishes "Copy with Images" (synthesizing `turn.media_paths` into markdown image tags) from "Copy without images (Text only)".
   - Restores the interactive Detail button, displaying complete instance telemetry (Instance ID, Name, Sequence #, Process PID, Path, Liveness).
   - Enables markdown image previews and native PNG image copying to the clipboard.
4. **Standalone Portable JSON Export/Import**:
   - Implements dual-mode export: Standalone Portable JSON (with `${workDir}` path normalization and base64 `embedded_images` map) vs Lean JSON (compact relative references).
   - Fixes modal overlay z-index collisions and removes stacked `backdrop-blur-md` compositing failures.
5. **SSH Cluster & Node W2 Parity**:
   - Reclassifies node `w2` in `gitmap-ssh-nodes.json` as Windows (`"os": "windows"`).
   - Prevents `FleetMachinesTable.tsx` from collapsing into `<FleetDisabledState />` when remote mesh nodes are offline.
   - Integrates GitMap SSH execution fallback when native OpenSSH key authentication fails.
6. **App Updater Pinned-Version & v4.190 Trap Remediation**:
   - Fixes `install.ps1` binary name mismatch: copies `agm-ui.exe` as primary GUI binary and hardlinks/overwrites legacy `agm-alim.exe`, pointing all shortcuts to `agm-ui.exe`.
   - Adds regex-based release tag extraction in `args.rs` and `install.ps1` to support GitHub release URLs (e.g. `https://github.com/.../releases/tag/v4.226.3`).
7. **Coding Guidelines (CG) Enforcement**:
   - Eliminates complex/nested `if-else` ladders via early-return guard clauses.
   - Enforces positive boolean prefixes (`is`/`has`) and removes negative conditionals.
   - Replaces magic string literals with typed enums.
