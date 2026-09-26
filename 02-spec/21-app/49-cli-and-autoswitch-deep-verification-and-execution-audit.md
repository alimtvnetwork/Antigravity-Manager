# 49: CLI & Auto-Switcher Deep Verification & Execution Audit

**Status:** Draft  
**Author:** Antigravity Orchestrator  
**Created Date:** 2026-09-26  

## User Request (Verbatim)
```text
Complete please

is it really done?? is it done properly?? WTF, I don't think so verify please

agm ff
agm status
agm switch-if-low-credit (swlc)
agm which-prompts-running (wpr) [--json] # should yield project which having prompts running, <seq> , id, conv id, conv name, prompts count (in queue) , having json will write in json mode properly

agm rerun prompts N [-prefix template-category name[ # N = 1  check into the gitmap for this similar examples please

<if we are in the repo folder>
agm prompts ls N --json --words 100 # will show running, N = 10 running prompts, shows prompts with sequence, as ASC stack list, and mention if displayed as table without json
agm prompts-export (pe) N -[f "file path or file name or nothing"] # default file agm-repo-slug-prompts.json flle, images needs to be given as base64 encoding
agm prompts-import (pi) -[f "file path or file name or nothing"] # default file agm-repo-slug-prompts.json flle but if the folder has more than one json it will prompt if other jsons are prompt and it wants to import or not, import means rerun 
agm status/credits # shows current account credits now and weekly both, --json will show in json mode with vm alias, ip, account using, 
agm email status # send email regarding this status
agm email help # send the email for help instruction to all 

agm email ls # show email info , --json will show in json mode, sender info , reciever(broatcast) info as well 
agm email add/rm/mv/export/ls # add these features with help please

## all will do the same as scripts-fixer, agy clear-cache --keep 10
agm clear-cache --keep/k 10
agm cache-clear --keep/k 10
agm clear cache --keep/k 10
agm clear-cache # default K = 10

clear??

agm instances ls # show everything for instances in table format
agm instances <seq, id, name alias> switch ff # switch the fast foward mode
agm instances <seq, id, name alias> ff # switch the fast foward mode
agm instances-all ff # switch the fast foward mode
agm instances rm <seq, id, name alias> 
agm instances create "new-name"  --data-only(do) # default copy ide, data only meaning only clone using data, need to test all these
agm instances rm-all # remove all except the default one

# current project repo
agm prompt <text> [--prefix/suffix template-group-name] # learn from gitmap please, do git pull
agm rerun # learn from gitmap please

agm recreate-project [project path or if you were in the same gitrepo then it will understand it] # remove the cache and everythign and also remove the project from agy and add it and create new conv to start with read all files and memory to understand the project, pelase # clear???

agm recreate <project alias or path, seq, id>,  <project alias or path, seq, id>, <project alias or path, seq, id> # do it for selected projects

[v4.75.0 | VM3 | 192.168.1.12] [Antigravity] -> [Antigravity | v4.75.0 | VM3 | 192.168.1.12] [Antigravity] Write the subject
```

## Architectural Verification Matrix

### 1. CLI Commands Verification Table
| Command | Primary Function | Flags & Options | Standard Outputs |
| :--- | :--- | :--- | :--- |
| `agm ff` | Fast-forwards rotation to freshest profile | `--json` | Transition payload with previous/next/credit/prompts |
| `agm status` / `credits` / `status/credits` | Reports credits and node state | `--json` | Immediate/weekly quota, threshold target, predicted next account, running prompts, prompts resent, attached images |
| `agm switch-if-low-credit` (`swlc`, `sfc`) | Evaluates quota against threshold and rotates | `-t <pct>`, `-f [path]`, `--json` | Exports to `agm-<node_alias>-switch.json`, structured switch decision with 5 telemetric answers |
| `agm is-low-credit-for-switch` (`ilc`) | Evaluates if quota is below threshold | `-t <pct>`, `-f [path]`, `--json` | Plain: `true`/`false`. JSON: full node identity and next candidate |
| `agm which-prompts-running` (`wpr`) | Lists projects with active prompts | `--json` | `seq`, `project`, `id`, `conv_id`, `conv_name`, `prompts_count` |
| `agm rerun prompts N` | Reruns last N prompts in ASC order | `-prefix <cat>`, `-suffix <cat>` | `git pull` pre-run, seeds `.antigravity_resume_task.json` |
| `agm prompts ls N` | Lists last N prompts in ascending order | `--words <W>`, `--json` | ASC stack list with word preview and table format |
| `agm prompts-export` (`pe`) | Exports prompts with Base64 images | `-f [path]` | Default `agm-<repo-slug>-prompts.json` |
| `agm prompts-import` (`pi`) | Imports prompts and schedules reruns | `-f [path]` | Scans directory, prompts if multiple JSON files found |
| `agm email status` | Dispatches node & credits status email | `--json` | HTML card + `[JSON]` block sent to recipients |
| `agm email help` | Dispatches instructions email | `--help` | Sends help guide card to all configured recipients |
| `agm email ls` | Lists mailboxes & notification recipients | `--json` | Table with SEQ, ID, EMAIL, SMTP, IMAP, DEFAULT, ACTIVE |
| `agm email add` | Adds mailbox or recipient | `--recipient`, `--default` | Sends JSON self-test verification email to the added address |
| `agm clear-cache` | Prunes old conversations & temp files | `--keep/k <N>`, `-k <N>` (default 10) | Preserves N recent conversations, frees disk space |
| `agm instances ls` | Lists all isolated instances | `--json` | Table with SEQ, ID, NAME, STATUS, BOUND EMAIL, PORT |
| `agm instances <target> ff` | Fast-forwards rotation on specific instance | N/A | Rotates profile for target instance |
| `agm instances-all ff` | Fast-forwards all registered instances | N/A | Rotates each instance sequentially |
| `agm instances create <name>` | Creates or clones an instance | `--data-only` (`--do`, `do`) | Clones IDE data or full executable |
| `agm instances rm-all` | Removes all instances except default | N/A | Preserves default instance, purges isolated instances |
| `agm prompt <text>` | Injects immediate prompt task | `--prefix`, `--suffix` | Pulls git, registers in DB, writes resume task |
| `agm recreate-project` / `recreate` | Deep cleans project and seeds bootstrap prompt | `<targets...>` | Prunes conversations/brain, registers bootstrap prompt |

### 2. Required Invariants
1. `credit_before_switch`: Captures quota percentage before profile switch.
2. `threshold_activated`: Captures activated rule percentage (e.g. 15% or 98%).
3. `previous_account`: Never displays `"default"`; displays genuine previous email.
4. `predicted_next_account`: Predicts best candidate before switch.
5. `selected_account`: Confirms selected account after switch.
6. `prompts_running` & `prompts_resent`: Confirms prompt counts and re-injection via `.antigravity_resume_task.json`.
8. Email Subject Standard: `[Antigravity | v{VERSION} | {VM} | {IP}] [JSON] Account Switched: {previous} -> {selected}` (Non-JSON emails: `[Antigravity | v{VERSION} | {VM} | {IP}] {Subject}`)
