# Specification 58: Telegram & AGM GitMap AGY Parity, VM Fleet Orchestration & Remote Prompt Routing

## Metadata
- **Version:** 1.0.0
- **Created:** 2026-09-27
- **Authority:** User Request (Verbatim) & AGENTS.md
- **Status:** Active

---

## 1. User Request (Verbatim)

```text
Okay, I think you messed up most of the installation right now. The reason is that your install script is installing from the original fork version. Are you stupid? Where did you make this mistake? How did you make this mistake? Are you stupid? Okay. I really do not appreciate this type of stupidity. And also in your Telegram agent, we should have a command to see the VM nodes. Okay? And that would basically going to use the VM nodes just like the Gitmap. So you should integrate some things like this. So Gitmap, it can also get the node information or node credential from the Gitmap. Okay, this type of commands. Okay. Once, let's say it got the injection, it updated the stuff, the node names and things like that, we should be able to run the node LS and that would actually basically going to tell us which nodes are running active, things like that. And also in those node, we can run prompts. So we can also check the running prompts as well. We can say nodes, alias name, space, the running prompts, and that would actually show up the running prompts that we have. We could also limit the output to the Telegram, because it seems like the response that we give, that actually gets truncated. So what is the solution towards that? You can think of it. Probably we have to chunk it and give multiple responses if the response is bigger. Think of this, in the Telegram actually. And also the next thing is that we should be able to see the running prompts, short version, compact version, just like the Gitmap has. And we should be able to inject prompt to the conversation. We can just see the running projects, running conversation, and we can see their ID and based on that, there will be a sample format, how we can communicate or send prompt. There will be a prompts list like the Gitmap. It will reuse the Gitmaps once and just show it. Okay? So all kinds of things we need in the Telegram, so try to include it. So in the future, I can say prompt and the project name, and the node and VM. So you could just send that request to that VM, to that project, okay, to that instant, and just like the email, okay, we need to have this system ready. So think about all these updates, all these new commands that AZY have in the Gitmap. Try to read all this help. Okay, try to get inside it. Try to have similar commands in the AGM. Okay? Do you understand me? Can you please help me with this?
```

---

## 2. Architectural Objectives

1. **Zero-Tolerance Upstream Fork Elimination**: Ensure zero occurrences of `lbjlaq/Antigravity-Manager` across all scripts, installer templates, API fallbacks, and documentation. All downloads and GitHub release checks must strictly target `alimtvnetwork/Antigravity-Manager`.
2. **GitMap AGY Subsystem Parity in AGM & Telegram**:
   - `/active`, `/running` (and `agm agy active`): Display running conversations and prompts in a compact GitMap-style table (ID, Title, Path, Steps, Elapsed, Status).
   - `/queues`, `/queue` (and `agm agy queues`): Display queued prompts across workspaces.
   - `/projects`, `/workspaces` (and `agm agy ls`): Display discovered projects with Seq, Conv Name, ID, Project, Path, and ready-to-copy injection syntax.
   - `/prompts`, `/prompt ls` (and `agm agy prompts`): Display available reusable prompt templates (e.g. `read-all`, `is-done`, `ci-cd-fix`).
3. **VM Fleet Nodes Topology & Credentials (`/nodes`, `/node ls`, `agm nodes`)**:
   - Aggregate GitMap cluster nodes (`gitmap cluster status`, `gitmap cluster nodes`), Supabase Root DB `nodes`, and local machine telemetry.
   - Resolve node credentials from GitMap node/cluster vault.
   - Render clean, informative node status cards with online/idle/offline badges, IP, uptime, and last seen timestamps.
4. **Node-Scoped Running Prompts (`/nodes <alias> prompts`)**:
   - Inspect active running prompts scoped to a specific VM node (or local machine).
5. **Cross-Machine Prompt Injection (`/prompt <node> <project> <text>`)**:
   - Local: Save into split SQLite `repo_prompts.db` and spawn `agy`.
   - Remote: Execute via `gitmap cluster exec <node> "agm prompt ..."` or enqueue in Supabase.
   - Provide concrete sample format in every help and project response.
6. **Telegram Intelligent Message Chunking & Pacing**:
   - Partition payloads exceeding 3800 characters along newline boundaries (`\n\n`, `\n`).
   - Deliver sequential chunks with 80ms transmission pacing.
   - Auto-retry with stripped HTML tags if Telegram returns `400 Bad Request`.

---

## 3. Command Matrix (Telegram & AGM CLI Parity)

| Feature | Telegram Bot Command | AGM CLI Command | GitMap Equivalent | Description |
|---|---|---|---|---|
| **Cluster Nodes** | `/nodes`, `/node ls`, `/nodes ls` | `agm telegram nodes`, `agm nodes` | `gitmap cluster status` / `nodes` | List all active VM cluster nodes, IP, roles & status |
| **Node Prompts** | `/nodes <alias> prompts` | `agm telegram prompts <alias>` | `gitmap agy ssh ...` | List running prompts scoped to a specific node |
| **Active Prompts** | `/active`, `/running` | `agm agy active`, `agm running` | `gitmap agy active` | Compact table of active conversations running prompts |
| **Prompt Queues** | `/queues`, `/queue` | `agm agy queues`, `agm queues` | `gitmap agy queues` | Inspect pending prompt queues across workspaces |
| **Projects List** | `/projects`, `/workspaces` | `agm agy ls`, `agm projects` | `gitmap agy ls` | List discovered projects, IDs, paths & sample prompt syntax |
| **Templates List** | `/prompts`, `/prompt ls` | `agm agy prompts`, `agm prompt ls` | `gitmap agy prompt ls` | List reusable prompt templates |
| **Inject Prompt** | `/prompt <node> <proj> <text>` | `agm telegram prompt ...` | `gitmap agy prompt -n/-t` | Dispatch prompt to local or remote VM project |
| **Help Menu** | `/help`, `/start` | `agm help`, `agm telegram help` | `gitmap agy --help` | Display comprehensive command reference & examples |

---

## 4. Verification Gates
- [x] Zero references to `lbjlaq` in `install.ps1`, `install.sh`, `deploy/arch/install.sh`.
- [x] `install.ps1 -DryRun` confirms download targets resolve to `alimtvnetwork`.
- [x] `agm telegram nodes` delivers formatted fleet report to Telegram.
- [x] `agm telegram projects` delivers projects catalog with sample prompt format.
- [x] `agm telegram prompts [node]` displays node-scoped running prompts.
- [x] Telegram responses >3800 chars are chunked and HTML-safe.
