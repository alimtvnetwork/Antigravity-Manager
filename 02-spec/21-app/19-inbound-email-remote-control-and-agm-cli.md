# Inbound Email Remote Control, Plaintext Receipts, and AGM Native Terminal CLI

## 1. Executive Summary

This specification defines the universal pipe-delimited inbound email command grammar, 2-phase plaintext notification receipts (ACK and Result), 10-second sliding debounce rate-limiting stack, sender authorization access control lists (ACL), and the native `agm` terminal command-line interface (CLI) for Antigravity-Manager.

## 2. Inbound Email Remote Control Grammar

### 2.1 Pipe-Delimited Subject Grammar

Inbound email subjects follow a unified, flexible pipe-delimited grammar supporting 2 to 4 segments with optional whitespace:

```text
sub: [worker-name|ip] | [ins-{instance}] | (prompt|gitmap|cmd|update|ls|help|gitmap macro|gitmap update|agm update|agm status|agm instances|agm ls|agm ff/smart-switch|agy prompts ls|gitmap prompts ls) [ | proj-{project name} ]
```

#### Parsing Rules:
1. **Target Worker / IP (`segment 1`)**:
   - Matches worker node name case-insensitively against local machine name or `COMPUTERNAME` / `HOSTNAME`.
   - Matches full IP address (e.g., `192.168.1.50`), localhost (`localhost`, `127.0.0.1`, `local`), or wildcard (`*`, `all`, `any`).
   - Supports **partial IP octet matching**: e.g., `12` matches `192.168.1.12` or any local interface IP ending in `.12`.
2. **Instance Identifier (`segment 2`, Optional)**:
   - If a segment begins with `ins-` (case-insensitive, e.g., `ins-default`, `ins-0`, `ins-gemini`), it designates the target instance profile.
   - If omitted, execution defaults to the default/active instance.
3. **Command (`segment 2` or `3`)**:
   - `prompt`: Injects or executes a prompt instruction.
   - `gitmap`: Dispatches GitMap autonomous execution. Redundant prefixes (e.g. `gitmap gitmap <cmd>`) are intelligently collapsed to `gitmap <cmd>`.
   - `cmd`: Runs a PowerShell (Windows) or POSIX shell (Linux/macOS) command.
   - `update`: Triggers AGM binary update check and runner.
   - `ls` / `agm ls` / `agm instances`: Lists running instances and active project workspaces.
   - `help`: Returns a plaintext cheat sheet of all available commands.
   - `gitmap macro`: Executes GitMap predefined macros.
   - `gitmap update`: Triggers GitMap CLI update.
   - `agm update`: Triggers AGM updater.
   - `agm status`: Returns node telemetry, active profile, and system status.
   - `agm ff` / `agm smart-switch` / `agm ff/smart-switch`: Initiates account/workspace rotation.
   - `agy prompts ls` / `gitmap prompts ls`: Returns inventory of saved and active prompts.
4. **Project Target (`segment 4`, Optional)**:
   - Prefixed with `proj-` or `project:` (e.g., `proj-ecommerce`, `proj-frontend`).
   - If omitted, targets the primary or first active workspace.

### 2.2 Body Instruction Grammar

For prompt commands (`prompt`), the email body adheres to:

```text
prompt-name: (use from gitmap prompts ls)
prompt instruction:
<detailed multi-line instruction>
```

For shell commands (`cmd`) or `gitmap` actions:
- The body contains the raw shell script or command arguments to be executed directly.

---

## 3. Two-Phase Plaintext Notification Receipts

To eliminate spam filter flags, reduce bandwidth, and ensure high readability across mobile email clients, heavy HTML email templates are replaced with lightweight, clean plaintext notifications.

### 3.1 Phase 1: Immediate Acknowledgment (ACK)
Immediately upon parsing and validating an incoming command, the watcher sends an acknowledgment receipt to the sender:

```text
================================================================================
[AGM ACK] Command Acknowledged
================================================================================
Command:    <command string>
Target:     <node-name> (<local-ip>)
Instance:   <instance-name>
Status:     RUNNING
Received:   <ISO 8601 timestamp>

Execution is currently in progress. A completion result will follow.
================================================================================
```

### 3.2 Phase 2: Completion Result
Once the command finishes execution, a second receipt is dispatched with execution status, return code, and output:

```text
================================================================================
[AGM Result] Execution Finished
================================================================================
Command:    <command string>
Exit Code:  <0 / non-zero>
Duration:   <elapsed time in seconds>
Status:     <SUCCESS / FAILED>
Completed:  <ISO 8601 timestamp>

Output:
--------------------------------------------------------------------------------
<clean stdout/stderr output>
--------------------------------------------------------------------------------
================================================================================
```

---

## 4. Rate-Limiting Debounce Stack & Security ACL

### 4.1 Debounce Rate Limiter (10-Second Sliding Window)
1. Inbound messages are hashed by `(sender_email, normalized_command, target)`.
2. When multiple identical commands arrive within a 10-second window:
   - The first request triggers an immediate ACK and execution.
   - Intermediate requests within 10s are queued in the sliding debounce stack.
   - At most two emails are sent: the initial response, and at the 10-second window boundary, a consolidated completion summary.
   - Further identical spam requests are throttled.

### 4.2 Sender Authorization (Security ACL)
1. All inbound messages check `msg.from` against active records in `notify_recipients` (`email_vault.db`).
2. Senders not listed or having `is_active = 0` are rejected immediately.
3. Rejected attempts are recorded in `email_inbound_audit_log` with status `rejected_unauthorized_sender` and no email reply is sent to prevent mailbox harvesting.

---

## 5. Native AGM Terminal CLI Engine (`agm`)

The `agm` CLI is a native executable compiled via Cargo and added to the user's system PATH and PowerShell `$PROFILE` (mirroring the GitMap CLI architecture).

### 5.1 Command Reference

| Command | Description |
| :--- | :--- |
| `agm status` | Displays node status, active account, proxy port, running instances, and uptime. |
| `agm instances` / `agm ls` | Lists registered profiles, data directories, bound accounts, and running PIDs. |
| `agm ff` / `agm smart-switch` | Triggers manual workspace rotation / fast-forward. |
| `agm update` | Checks for newer releases on GitHub, downloads, and updates the `agm` executable. |
| `agm install` | Registers `agm` in `%LOCALAPPDATA%\agm-cli\` and adds it to PATH and PowerShell `$PROFILE`. |
| `agm ssh <[user@]host> [-p port] [--password <pwd>] [--update]` | Secure SSH login with password fallback and remote VM auto-update. |
| `agm --version` / `agm -v` | Displays version and build info. |
| `agm help` / `agm --help` | Displays help message and syntax examples. |

### 5.2 SSH Login & Remote VM Auto-Update
- **Interactive Password Handling**: If `--password` is not specified on the CLI and SSH key authentication fails or requires credentials, `agm` prompts securely via terminal standard input with echo disabled.
- **Remote VM Auto-Update (`--update`)**: When passed, `agm` executes a remote script over SSH to download the latest AGM release, install dependencies, and restart services on the VM machine.
