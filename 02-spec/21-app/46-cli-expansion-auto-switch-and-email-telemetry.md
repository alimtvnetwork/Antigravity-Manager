# Specification: CLI Expansion, Auto-Switch If Low Credit & Email Multi-VM Telemetry State Machine

> **Module ID:** `02-spec/21-app/46-cli-expansion-auto-switch-and-email-telemetry.md`  
> **Status:** Normative / Active  
> **Version:** 4.76.0  
> **Target:** `src-tauri/src/bin/agm.rs`, `src-tauri/src/modules/auto_switcher.rs`, `src-tauri/src/modules/instance.rs`, `src-tauri/src/modules/notification_hub.rs`, `src-tauri/src/modules/email_sender.rs`, `src-tauri/src/modules/email_inbound.rs`, `src-tauri/src/modules/repo_db.rs`  

---

## 1. User Request (Verbatim)

```text
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





Okay. So now I wanted to add a little bit more information to the system. It would be firstly CLI options. So I want to have AGM new commands and for example, fast forward AGM status, then AGM switch if no credit. That would be SFC, switch if no credits. So this one is still not working automatically. You need to work on it. And also I'm very concerned about the instance part. I really don't think that you have done it well, and you can test it here by running the stuff locally, by clicking or creating more instance here and then try to switch. And also you could boil down the, let's say, percentage of use to 98 to see how soon it can switch. You can do some operation, and you'll see the credits are gone, so it should automatically switch. It does not switch. It does not click on the smart fast forward. So that is one of the issues. And also if the email is added, make sure to self-email what is added. You should send yourself a JSON email, like the last switch for the VM IP and things like that. And when you send the email, there is no need to use the third brackets, okay? So just mention the version number, that's good. VM IP. You can mention the third bracket of the system at the beginning or no mention about the third bracket, I think. So your title looks like this currently, okay? The version and things like that, which should be written. If you are using third bracket, okay, fine. Absolutely fine. What you could do is you could just use this format in the email subject first and then write the subject. Okay, so this is a format change I think you need to do. So same way, it should have a JSON reader subject, so have a, let's say, flag of third bracket JSON segment. So in the email, when fast forward is happening, if email is connected, when a switch happens, it will also mark like this is the VM, this is the switch. In the JSON mode, it would write it. So when the next switch is happening fast forward, that will also read the email and check what is the last VM, if the last any one of the VM is using the same machine. Okay, it needs to have the IP, VM name, the email that it was in before, email that it is moved to, instance mode, how it was switched, what is the condition, the version of the AGM package. So these type of things, IP, okay, so that next time, next machine when it tries to rotate so that they are not in the same account. That is the most important part. And same thing they could do if the Supabase database is connected, and we will deal with the Supabase database later on. Can you please help me with this stuff, please?
```

---

## 2. Architectural Blueprint & Requirements

### 2.1 Auto-Switch If Low/No Credit Watcher Hardening
1. **Fallback Account ID Invariant:** When evaluating instances in `auto_switcher.rs`, if an instance has `bound_account_id: None` (such as the default instance or a newly created instance before explicit binding), the engine must fall back to the currently active account ID from `account::get_current_account_id()`. Skipping un-bound active instances previously resulted in zero auto-rotation.
2. **Pre-Evaluation Quota Refresh:** During check cycles or manual `swlc`/`sfc` invocations, the bound account's quota must be refreshed from the Google API via `fetch_quota_with_retry` to reflect real-time credit depletion instead of evaluating stale disk cache.
3. **Threshold Parameter Flexibility:** Support configurable low-quota thresholds up to 99% (e.g. 98% for immediate testing). When remaining quota drops below the threshold, trigger fast-forward rotation automatically without human interaction.

### 2.2 Email Subject & Multi-VM Telemetry JSON State Machine
1. **Subject Formatting:** Standardize email subjects emitted by AGM:
   ```text
   [Antigravity | v{VERSION} | {VM_NAME} | {LOCAL_IP}] [Antigravity] {Subject}
   ```
2. **JSON Switch Telemetry:** For switch events (auto-switch, fast-forward, manual instance switch):
   - Subject line appends `[JSON]`:
     ```text
     [Antigravity | v{VERSION} | {VM_NAME} | {LOCAL_IP}] [Antigravity] [JSON] Account Switched: {instance_name} -> {target_email}
     ```
   - Email body embeds a machine-readable JSON card:
     ```json
     {
       "agm_version": "v4.76.0",
       "vm_name": "VM3",
       "local_ip": "192.168.1.12",
       "old_email": "prev@domain.com",
       "new_email": "curr@domain.com",
       "instance_id": "default",
       "instance_name": "Default",
       "switch_mode": "auto",
       "reason": "Quota depleted ...",
       "timestamp": 1727350000
     }
     ```
3. **Cross-VM Account Collision Prevention:**
   When fast-forward or auto-switch runs, if the default IMAP mailbox is configured and reachable:
   - Poll recent messages with subject `[JSON]` and `Account Switched`.
   - Parse switch events originating from other VMs (`vm_name != local_name` or `local_ip != my_ip`).
   - Add recent accounts (within the last 60 minutes) to `excluded_accounts` so sibling VMs never rotate into the same account simultaneously.
4. **Self-Email on Email Account Addition:**
   When an email account or recipient is added or updated via CLI or UI, dispatch a self-notification to configured recipients containing the added email metadata formatted with the standardized subject and JSON body.

### 2.3 AGM CLI Expanded Subcommands Specification

| Command | Aliases | Parameters | Description |
|---|---|---|---|
| `agm ff` | `smart-switch`, `fast-forward` | None | Trigger fast-forward rotation to freshest candidate profile |
| `agm status` | `credits` | `[--json]` | Show node status, active account, and credits (now and weekly) |
| `agm switch-if-low-credit` | `swlc`, `sfc`, `switch-if-no-credit` | `[threshold]` `[--force]` | Check active account quota, refresh, and switch if credits depleted or below threshold |
| `agm which-prompts-running` | `wpr` | `[--json]` | List projects with running prompts (`#`, ID, Conv ID, Conv Name, Prompts Count) |
| `agm prompts ls` | None | `[N]` `[--json]` `[--words <W>]` | List running prompts (ASC stack order) with word count cutoff |
| `agm prompts-export` | `pe` | `[N]` `[-f <path>]` | Export prompts to JSON (default `agm-<repo-slug>-prompts.json`) with Base64 image encoding |
| `agm prompts-import` | `pi` | `[-f <path>]` | Import prompts from JSON file; interactive selection if multiple JSONs exist |
| `agm prompt` | None | `<text>` `[--prefix <cat>]` `[--suffix <cat>]` | Pull git, wrap prompt with template, and queue in `active_prompts` |
| `agm rerun` | None | `[prompts N]` `[-prefix <cat>]` | Rerun last prompt or last N prompts with optional category prefix |
| `agm clear-cache` | `cache-clear`, `clear cache` | `[--keep/-k <N>]` | Prune conversations keeping N (default 10) and purge application caches via `agy_cleaner` |
| `agm instances ls` | `instance ls`, `ls` | `[--json]` | Tabular or JSON display of all configured sandbox profiles, PIDs, and node identity |
| `agm instances <target> ff` | `agm instances <target> switch ff` | `<target>` | Fast-forward rotate the specified instance profile |
| `agm instances-all ff` | None | None | Trigger fast-forward rotation across all instances |
| `agm instances rm` | None | `<target>` | Remove instance profile (rejects default instance) |
| `agm instances create` | None | `<name>` `[--data-only / --do]` | Create new instance (standard clone or data-only profile) |
| `agm instances rm-all` | None | None | Remove all instance profiles except default |
| `agm email status` | None | None | Display email vault status and dispatch status email |
| `agm email help` | None | None | Display instructions and dispatch help email to all recipients |
| `agm email ls` | None | `[--json]` | List mailbox accounts and notifier recipients in table or JSON mode |
| `agm email add` | None | `<email>` `[options]` | Add mailbox account or recipient; sends self-email confirmation |
| `agm email rm` | None | `<id\|email>` | Delete mailbox account or recipient |
| `agm email mv` | None | `<id\|email>` `--default` | Mark email account as default |
| `agm email export` | None | `[path]` | Export email accounts and recipients configuration to JSON |
| `agm recreate-project` | None | `[path]` | Clear project cache/conversations, re-add to `agy`, and spawn fresh session |
| `agm recreate` | None | `<targets...>` | Recreate project for comma-separated or space-separated list of projects |

---

## 3. Conformance & Verification Invariants

- **AC-CLI-001 (CLI Return Codes):** All CLI subcommands exit with code 0 on success and code 1 on handled errors with descriptive stderr messages.
- **AC-CLI-002 (JSON Format Purity):** When `--json` is supplied, stdout MUST be 100% valid parseable JSON with zero interleaved human log messages.
- **AC-CLI-003 (Default Instance Immutability):** Neither `agm instances rm default` nor `agm instances rm-all` may delete or alter the `default` instance.
- **AC-CLI-004 (Cross-VM Collision Guard):** Rotation candidates must never match accounts actively in use by running instances locally or reported by sibling VMs in recent `[JSON]` switch emails.
- **AC-CLI-005 (Safe Cache Staging):** Cache clearing via `agm clear-cache` must stage pruned conversations in `antigravity-cleaner-backup` for reversible undo.
