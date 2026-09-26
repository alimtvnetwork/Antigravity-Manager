# Comprehensive CLI Verification & Auto-Switcher Invariants Specification

**Specification Version:** 1.0.0
**Status:** Active
**Author:** AI Agent (Antigravity Manager Engine)
**Date:** 2026-09-26

---

## 1. User Request (Verbatim)

```text
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





Okay. So now I wanted to add a little bit more information to the system. It would be firstly CLI options. So I want to have AGM new commands and for example, fast forward AGM status, then AGM switch if no credit. That would be SFC, switch if no credits. So this one is still not working automatically. You need to work on it. And also I'm very concerned about the instance part. I really don't think that you have done it well, and you can test it here by running the stuff locally, by clicking or creating more instance here and then try to switch. And also you could boil down the, let's say, percentage of use to 98 to see how soon it can switch. You can do some operation, and you'll see the credits are gone, so it should automatically switch. It does not switch. It does not click on the smart fast forward. So that is one of the issues. And also if the email is added, make sure to self-email what is added. You should send yourself a JSON email, like the last switch for the VM IP and things like that. And when you send the email, there is no need to use the third brackets, okay? So just mention the version number, that's good. VM IP. You can mention the third bracket of the system at the beginning or no mention about the third bracket, I think. So your title looks like this currently, okay? The version and things like that, which should be written. If you are using third bracket, okay, fine. Absolutely fine. What you could do is you could just use this format in the email subject first and then write the subject. Okay, so this is a format change I think you need to do. So same way, it should have a JSON reader subject, so have a, let's say, flag of third bracket JSON segment. So in the email, when fast forward is happening, if email is connected, when a switch happens, it will also mark like this is the VM, this is the switch. In the JSON mode, it would write it. So when the next switch is happening fast forward, that will also read the email and check what is the last VM, if the last any one of the VM is using the same machine. Okay, it needs to have the IP, VM name, the email that it was in before, email that it is moved to, instance mode, how it was switched, what is the condition, the version of the AGM package. So these type of things, IP, okay, so that next time, next machine when it tries to rotate so that they are not in the same account. That is the most important part. And same thing they could do if the Supabase database is connected, and we will deal with the Supabase database later on. Can you please help me with this stuff, please?
```

---

## 2. Invariants & Audit Gates

### 2.1 Core Commands Inventory & Behavior Audit
1. `agm ff` / `agm smart-switch`: Triggers rotation using `auto_switcher::check_and_rotate_for_threshold(Some(100.0), true)`.
2. `agm status` / `agm credits`: Reports node identity, immediate quota, weekly quota, tier, running instances, and running prompts; supports `--json`.
3. `agm switch-if-low-credit` (`swlc` / `sfc`): Supports optional threshold argument, `--json`, and `-f [path]` writing to `agm-<node_alias>-switch.json`.
4. `agm is-low-credit-for-switch` (`ilc`): Non-mutating query; plain mode returns `true`/`false`; `--json` mode returns full machine telemetry; `-f` writes to file.
5. `agm which-prompts-running` (`wpr`): Discovers running projects from workspaceStorage and repo database, reports `<seq>`, ID, conv ID, conv name, and prompts count; supports `--json`.
6. `agm prompts ls [N] [--json] [--words W]`: Lists active prompts in ascending stack order with sequence numbers and preview words.
7. `agm prompts-export [N] [-f path]`: Exports active prompts with Base64-encoded images to `agm-<repo-slug>-prompts.json` or target file.
8. `agm prompts-import [-f path]`: Discovers and imports JSON prompt files, prompting if multiple files exist.
9. `agm email add <email> [pwd] [options]`: When an email sender account is added, immediately dispatches a self-test JSON verification email.
10. `agm clear-cache` / `agm cache-clear` / `agm clear cache`: Respects `--keep/k N` (defaults to 10), pruning old conversations and temp caches.
11. `agm instances ls`: Renders instances in clean ASCII table with sequence numbers and process status.
12. `agm instances <seq|id|alias> ff`: Switches targeted instance to next freshest account.
13. `agm instances-all ff`: Rotates all instances sequentially.
14. `agm instances create <name> [--data-only]`: Clones executable by default, or only copies data directory when `--data-only` (`do`) is passed.
15. `agm recreate-project [path]` & `agm recreate <targets...>`: Purges caches, removes conversations from agy, and seeds bootstrap conversation.

### 2.2 Auto-Switcher 98% Threshold & Reactive Loop
- The auto-switcher reactive loop must evaluate quota drops in ticks `<= 5s`.
- When threshold is set high (e.g. 98%), candidates must be found even when standby accounts are between 15% and 97% using intelligent fallback selection.
- In GUI, slider changes to 98% immediately invoke `smartRotateProfileAccount`.
- Process termination must safely terminate IDE instances (`kill_instance_processes`), inject credentials into SQLite/`storage.json`, and restart with arguments.
