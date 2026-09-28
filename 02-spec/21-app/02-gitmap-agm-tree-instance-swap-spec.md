# GitMap CLI Parity, Bracketed Dual-Sequence Project/Conversation Tree View, & Multi-Instance/Multi-Project Account Swapping Specification

## 1. Original User Request (Verbatim)

> Okay. So for AGM command and also the Telegram command, can you please add more commands from Git Map as well? For example, Git Map AGY commands inside the help of the Telegram section or Telegram help. Okay. So AGY running projects or backup these running storage projects. So this type of example I need backup, restore, things like that. And also prompting example from the Git Map need to be there so you can pull out the latest Git Map changes so that you understand what it did, how it did it, is it good enough or not? And based on that, you can add some of those commands, okay, for the verification. Yeah. So these are on top of my head now. Think. Okay. The reason I'm thinking here that from AGM, we need to have more commands like update, AGM update, also the Git Map update command, and also the SSH commands. Some of the SSH command we need from the Git Map as well. For example, Git Map SSH nodes, run something using Git Map on a specific machine. So things like that we need this in the Telegram example. Also, how can I view all the projects as a tree view? Each project has a conversation, like a tree view. In a project inside the bracket, maybe that conversation has a running prompt of 200 words, which is inside a bracket. It has a specific sequence number for the project, it has a specific sequence number for the conversation, so that you can easily prompt it as well using the GitMap or using the AGM from the command line, using a sequence number to the specific machine and specific instance and specific conversation. That requires us to make our sequence even better because we don't know if we run it on the AGM or GitMap. In two places we are generating the sequence number. Does it match? How do you think they will be matching? So I think both should have their own sequence number if that is the case. Also, let me know one thing. Did we complete the multi-instance creation and multiple project assigning to a specific instance and multiple account swapping? Did we do everything properly? Give me the list of things you have done, what not done, so that I can verify.

---

## 2. Architecture & Design Specifications

### 2.1 Dual AGM + GitMap Sequence Architecture (`[AGM:P001 | GM:#1]` & `[AGM:C001 | GM:<short_cid>]`)
Because AGM and GitMap maintain independent SQLite databases (`~/.antigravity_tools/repodb/repo.db` vs. GitMap's `repodb`), relying on a single implicit auto-increment integer across two tools can drift if one tool discovers a project first.
- **AGM Sequence IDs (`P001`, `C001`)**: Stored deterministically in `agm_project_sequences` and `agm_conversation_sequences` in `~/.antigravity_tools/repodb/repo.db`.
- **GitMap Sequence & Target IDs (`GM:#1` / `GM:<project_name>`, `GM:<short_cid>`)**: Computed deterministically alongside the AGM sequence ID so every tree node displays **both** identifiers in a single bracketed header:
  - **Project Bracket**: `[AGM:P001 | GM:#1] [ProjID: <project_id>] <repo_path> [Instance: #<inst_seq> <instance_name> (<bound_email>)]`
  - **Conversation Bracket**: `├─ [AGM:C001 | GM:<8char_conv_id>] "<Conversation Title>" (<status> · <steps> steps)`
  - **200-Word Prompt Bracket**: `│  └─ [Prompt ≤200w]: "<truncated 200-word running/last prompt>"`
- **Unified Resolution**: `resolve_prompt_target(target)` accepts `C001`, `P001`, `AGM:C001`, `AGM:P001`, `GM:#1`, raw `<conv_uuid_prefix>`, or `<project_id>` / folder name, allowing the user to copy either the AGM code or the GitMap code into `agm prompt` or `gitmap agy prompt-project`.

### 2.2 Instance- & Machine-Scoped Prompt Injection
- **Local Instance Targeting (`--instance <inst>`)**:
  - `agm prompt <C001|P001|GM:#1> "<text>" --instance <id|#seq|name>` resolves the target instance from `instances.json` (or `default`), binds the queued prompt to that instance's `data_dir` (`--user-data-dir=<data_dir>`), and opens the conversation inside that specific instance's workspace.
- **Remote Machine Targeting (`--node <node>`)**:
  - `agm prompt <C001|P001> "<text>" --instance <inst> --node <node>` delegates execution over SSH via `gitmap ssh exec "agm prompt <target> '<text>' --instance <inst>"` (or `gitmap agy ssh ...`), enabling single-command remote machine + instance + conversation targeting from both CLI and Telegram (`/prompt C001 --instance #2 --node worker-1 <text>`).

### 2.3 Multi-Instance Creation, Multi-Project Workspace Binding, & Multi-Account Swapping Isolation
1. **Per-Instance Prompt Backup & Re-Injection (`repo_db.rs`)**:
   - `backup_running_prompts(instance_id)` only transitions `active_prompts` rows matching `instance_id` (or all rows when `instance_id == "all"`), preventing an account swap on Instance B from interrupting running prompts on Instance A.
   - `resend_running_commands_for_instance(instance_id, limit)` only re-injects backed-up prompts belonging to the swapped instance, passing `--user-data-dir=<instance.data_dir>` when spawning `agy chat` for non-default instances.
2. **Multi-Project Workspace Restoration on Instance Relaunch (`instance.rs`)**:
   - `get_instance_workspace_folders(instance_id, data_dir)` collects all project paths bound to the instance from `running_projects` (`repo_db`) and `User/workspaceStorage/*/workspace.json`.
   - `launch_instance(instance_id)` passes the bound project workspace paths to the spawned Antigravity executable instead of forcing a blank `--new-window`, ensuring all open projects assigned to that instance reopen automatically after an account swap.
3. **Global Keyring Isolation for Secondary Instances (`instance.rs`)**:
   - When switching or launching a custom instance (`!is_default_inst`, which uses `--user-data-dir` and `--password-store=basic`), `switch_account_to_instance` and `launch_instance` do not overwrite the global OS Keyring or global `current_account_id` if the `default` instance is bound to another account.
