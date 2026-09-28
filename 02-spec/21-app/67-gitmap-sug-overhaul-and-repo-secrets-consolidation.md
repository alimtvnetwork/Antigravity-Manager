# Spec 67: GitMap SUG (Shutdown-Until-Green) Overhaul, Real-Time Web UI, Multi-Target PE & Repo Secrets Consolidation

## 1. User Request (Verbatim)

```text
PS D:\work\antigravity-manager> gitmap sug help

  ╔════════════════════════════════════════════════════════╗
  ║   Shutdown Until Green (gitmap shutdown-until / sug)   ║
  ╚════════════════════════════════════════════════════════╝

  Usage:
    gitmap shutdown-until <command> [flags]
    gitmap sug <command> [flags]
    gitmap agy shutdown-until <command> [flags]
    gitmap agy sug <command> [flags]

  Watch List Commands:
    ls (list)                 List registered project targets in shutdown watch list
    add-projects <targets...>  Register one or more projects into watch list
    rm <targets...>           Remove designated projects from watch list
    agy-running-projects      Auto-populate watch list with all currently running AGY projects
    help                      Show this two-column interactive help menu

  Execution & Monitoring:
    run [-t <duration>]       Start continuous watch loop until all monitored pipelines turn green
    run --dry-run (-n)        Dry-run watch loop; simulates final OS shutdown when all pipelines pass
    run --once (-1)           Evaluate pipeline status once and exit without looping

  Flags:
    -t, --time <duration>     Polling check interval (default: 5m, minimum 2m; 10s in dry-run)
    -n, --dry-run             Simulate OS shutdown execution without powering off system
    -1, --once                Execute a single status evaluation cycle and exit immediately
    -h, --help                Display this comprehensive help menu

    Tip: Run 'gitmap sug agy-running-projects' to auto-register all active workspaces.
    Tip: Run 'gitmap sug run --dry-run' to test the watch loop safely without OS shutdown.
    Tip: Use 'gitmap sug ls' to inspect currently registered watch targets.

PS D:\work\antigravity-manager> gitmap agy sug help

  ╔════════════════════════════════════════════════════════╗
  ║   Shutdown Until Green (gitmap shutdown-until / sug)   ║
  ╚════════════════════════════════════════════════════════╝

  Usage:
    gitmap shutdown-until <command> [flags]
    gitmap sug <command> [flags]
    gitmap agy shutdown-until <command> [flags]
    gitmap agy sug <command> [flags]

  Watch List Commands:
    ls (list)                 List registered project targets in shutdown watch list
    add-projects <targets...>  Register one or more projects into watch list
    rm <targets...>           Remove designated projects from watch list
    agy-running-projects      Auto-populate watch list with all currently running AGY projects
    help                      Show this two-column interactive help menu

  Execution & Monitoring:
    run [-t <duration>]       Start continuous watch loop until all monitored pipelines turn green
    run --dry-run (-n)        Dry-run watch loop; simulates final OS shutdown when all pipelines pass
    run --once (-1)           Evaluate pipeline status once and exit without looping

  Flags:
    -t, --time <duration>     Polling check interval (default: 5m, minimum 2m; 10s in dry-run)
    -n, --dry-run             Simulate OS shutdown execution without powering off system
    -1, --once                Execute a single status evaluation cycle and exit immediately
    -h, --help                Display this comprehensive help menu

    Tip: Run 'gitmap sug agy-running-projects' to auto-register all active workspaces.
    Tip: Run 'gitmap sug run --dry-run' to test the watch loop safely without OS shutdown.
    Tip: Use 'gitmap sug ls' to inspect currently registered watch targets.

PS D:\work\antigravity-manager> gitmap agy sug agy-running projects
Error: [E1000:VALIDATION] validation: unknown sug subcommand: agy-running (at=cmdagy/agy_sug.go:72)
Usage:
  agy shutdown-until-green [command] [flags]

Aliases:
  shutdown-until-green, sug, shutdown-until

Flags:
  -n, --dry-run       Simulate shutdown without power off
  -h, --help          help for shutdown-until-green
  -t, --time string   Polling interval (minimum 2m, default 5m) (default "5m")

gitmap: [E1000:VALIDATION] validation: unknown sug subcommand: agy-running
  origin: cmdagy/agy_sug.go:72
PS D:\work\antigravity-manager> gitmap sug add-projects help
  ✔ Added 1 project(s) to shutdown watch list (total: 1)
PS D:\work\antigravity-manager>

PS D:\work\antigravity-manager> gitmap sug ls

  ╔════ SHUTDOWN-UNTIL-GREEN WATCH LIST (1 projects) ════╗
  [1] help

PS D:\work\antigravity-manager>


https://prnt.sc/I2tSQfeut9GQ


I think we need to improve this. It is the shut down until green. Okay? So this is where we can actually provide a project path. Now, if the project path is not there, you can just give a shutdown until and give a project path. There should just command for a project path, and we give the project path that should work. Now, if you check the API running projects, that actually gave no error at all. I think you need to fix that because you have given me as a suggestion, you didn't fix it. And also, what is it with the add projects? It could be add as well /add, or AP. Okay. Then what do I provide? The help is not there. And when I put help, it added the help as a project. So that is a total waste, total wrong thing to do. So first thing, it would check the project, if the project exists. Are we providing a URL, aliasing, or a folder path? Then a project needs to exist in the Git Map. Okay? If it does not, then it would give a nice error message, and what are the solutions that they could find the projects, how they could know what are the commands they could run to see the projects. Things like that. So it needs to be very concrete when you are doing it. Okay? It shouldn't be like, this, what you have done. Okay? So when we add the project, okay, so it would try to show the watch list that it's doing the watch. I do not work. How could I know that it is running at the moment? I have no idea. And add some examples at the end. I don't see any example in your help, like how one can use it. But yes, I believe the LS can show the watch list, what is running and let me try the LS. Yeah. In the help or, sorry, LS, it shows the project list, which is help, which in the same is very wrong, but it does not do anything. Where is my watch running? How could I see that my watch is running? Okay. So when I do watch, the idea there should be a watch command that I could look into and how the watch command is running. You need an explanation. For example, watch command can run on existing projects, and once the shutdown begins, it should actually clear the projects as well. There should be a settings inside it, and that settings can be changed. That settings can be changed from the whole root settings as well. And you can also have a UI that I can open in the browser and add projects. Think of this, okay? All the projects and things can be fed, where I can drag drop from the UI and make sure that these are in watch list and also see the browser that what is going on in the progress. It should be automated. It should be very powerful. Okay? So let's say I'm watching five to 10 projects. So usually, these projects should be pinged in every, let's say, five minutes. Right? Now, every five minutes, it would mention like, we are going to ping the Git Map PE on those projects. Now, Git Map PE also needs a path-based solution. That means in the Git Map PE, we can provide a path, we can provide the project name aliasing, and it'll work. Okay? Folder path and project name aliasing, these two should work. Also, the repo URL can work because it would get from the repo URL, right? So, I hope you understand. You need to have these options. So apparently, the shut down until green, it does not have the proper stuff. And it does not work, and you are saying it is working. That's quite funny. Other than targets, I think we have to be accurate on what the target means. We need to have a sub-point that actually explains what the target is. Also, we need to have a configuration that the ease, let's say shutdown until green is running. So it can enable, disable based on, let's say the project is added, but we have not run it. We can only do it by watch. We can do watch space UI that would again open the UI, on the watch mode, that which projects are running in the browser. We could just see the progress in every five minutes that it say that. Things like that. I don't see that it's completely done. I don't think that you can shut down the machine yet. Quite different things. Okay? Example, you could just attach the shutdown for this Git Map PE and see that the shutdown works. Okay, now we can do it because I'm here active. So if nothing is working, I could restart it back, so don't worry about it. You don't need to do the simulation. Does this make any sense? Please help me with these commands. Okay. Also, I think we can have a filter in the help, like a group. We can give a little bit of group names. The group names can have a slot in the help sections. Okay? And these help groups can also be provided as a suggestion. If I do help space, I could see all types of suggestions that I could type in and see the help. And since there is a search, an AUM search, two types of search, right? So I wanted to see what is the difference between these. What is the difference between how they work, how the search works, text file search works, things like that. So it needs to be mentioned a little bit clearly, which is not there yet. And also, what kind of commands we have with AGM, that's also not clear. Also, one more point, if you go into the repo secrets, if you go to the repo secrets, you do a git pull first. Yeah. So if you go to the repo secret section. Secrets section. There's this git map folder, right? You need to rename the git map to together name. It's not git hyphen map, it's git map together. So repo secrets, update that, and make a commit, make a push. Okay, so what I'm trying to showcase here is two points. These are very important. All the VM information nicely written inside this repository, like a JSON that can be imported directly using the git map. That's the first thing, rather than the VM path. The second thing is very crucial. It's in the SIO templates. Okay? The SIO templates company name, all the variables, we try to use it as Pascal case. That's the first thing. And variables should be used like dollar symbol and then curly brace. So update your code according to this. The next thing is that the Rise Up Asia needs to be together. It's R-I-S-E-U-P, then A-S-I-A space L-L-C together and different. So also Alim URL, it should be Alim hyphen portfolio, or Alim portfolio. So all variables should be Pascal case. You didn't correct it, so correct that. The URL is wrong. It should be Alim Karim.com. A-L-I-M-K-A-R-I-M.com, not Alim Ontari. So it's completely wrong. Alim need to correct, but also there can be aliasing, like AKA MD Alim Ontari, MD dot space A-L-I-M space U-L space K-A-R-I-M. So you need to have these variations. Okay? So region section, we can have variables that would have multiple segment, and we can pick any from this. Okay? Let's say a variable can be an array. Let's say region, it can be an array, it can be combination of three. And we can pick randomly, like an array bracket item if we are using the array segment. Okay? So we can randomly pick each one we are trying to pick. So that would be there in terms of the writing. So you need to change a little bit into your code how the template and variable in could work. Okay? Okay. So these are my ideas. We need to work on correct. Yeah. SIO template CLI, that looks good actually for now. Another thing that you need to correct is the commit-pull config. And I want all these config files, zero under two, as a sequence in the Git map order inside the repo secrets. Okay, correct that. So here the problem is that there are so many flags, okay, Boolean flags. These are not marked with is or has, okay? For example, the tree should be final thing should be is apply but not sync. The CD should be is apply CD. Okay, CD should be, that's the case. Okay, so everything needs to be passed the case, so. Sorry. Also, there are template file paths, for example, in the imports. The file path is repeating, which can be reduced by the work directory, which you have not. So make sure that you can also test this scenario. So I want to have this PowerShell or one-liner script inside this folder, actually, that I could run and execute and see that the test Git map can be created, removed, and test Git map. Remember that. Test Git map, not this. Very careful with it. Test Git map repository where it is inside the-- I mean, outside of the-- I mean, inside the D directory, then we have the test Git map, not inside work directory. Remember, this is a very important instruction. It cannot mess up. This is very crucial. Do you understand? Is it clear? Do you understand the task?
```

---

## 2. Visual Reference Asset

![GitMap SUG Investigation & Repo Secrets](assets/screenshots/gitmap-sug-investigation.png)

---

## 3. Architecture & Functional Requirements

### 3.1 GitMap SUG (Shutdown-Until-Green) Command Routing & Tokenization
- **Direct Project Path Support**: If `gitmap sug <path>` is executed without explicit subcommand, automatically detect if argument is a path or project alias and add or run it.
- **Space-Tolerant Subcommands**: Accept both hyphenated and space-separated subcommands:
  - `agy-running-projects`, `agy-running projects`, `running-projects`, `running projects`
- **Help Interception on Subcommands**:
  - If any subcommand receives `help`, `-h`, or `--help` (e.g. `gitmap sug add-projects help` or `gitmap sug add help`), render help for that subcommand instead of adding "help" to the watch list.
- **Aliases**:
  - `add-projects`: `add`, `/add`, `ap`, `add-projects`
  - `rm`: `remove`, `del`, `/rm`, `rp`
  - `watch`: `run`, `watch`, `w`
  - `ui`: `watch ui`, `sug ui`, `web`
- **Target Existence Validation**:
  - Every added project must be validated: Is it a valid local folder path, recognized GitMap alias, or repository URL in the catalog?
  - If target is invalid or not found, fail with actionable suggestions and commands to locate projects (`gitmap list`, `gitmap lf`, `gitmap sc`).
  - Define clearly in help what `<target>` means (folder path, repo alias, or repo URL).

### 3.2 GitMap SUG Watch Loop, Auto-Clear & Configuration
- **Active Watch Visibility**:
  - `gitmap sug watch` (and `gitmap sug run`) displays active watch status, interval countdown, and last-checked state of each project.
  - Option to auto-clear projects once shutdown is triggered (`autoClearOnShutdown: true`).
  - Settings configurable via CLI (`gitmap sug config`) or global root settings.

### 3.3 Real-Time Browser Web UI (`gitmap sug ui` / `gitmap sug watch ui`)
- Serve a modern browser dashboard (e.g. `http://127.0.0.1:4892`).
- Real-time progress table: project name, path, CI/CD pipeline status (green/yellow/red), last check time, next check ETA.
- Interactive controls:
  - Project input field (path/alias/URL) + project dropdown picker for 1-click addition.
  - Drag-and-drop support for adding folders.
  - Toggle auto-clear on shutdown.
  - "Check Now" and "Dry Run Shutdown" triggers.

### 3.4 GitMap PE Multi-Target & Direct Path / Alias / URL Support
- Enable `gitmap pe` to accept direct folder path, alias, or repository URL.
- Resolve target path to locate repository root and query pipeline status.

### 3.5 Grouped Help Menu & Command Clarification
- Category / group filtering in `gitmap help <group>` (e.g. `gitmap help sug`, `gitmap help aum`, `gitmap help search`).
- Help auto-suggestions on `gitmap help `.
- Explicit documentation clarifying:
  - `gitmap search`: Multi-core concurrent filesystem text grep.
  - `gitmap aum search`: Abstract Syntax Tree / symbolic symbol search.
  - `gitmap agm`: Antigravity Manager command parity and integrations.

### 3.6 Repo Secrets Standardization (`D:\work\repo-secrets`)
- **Directory Rename**: Rename `01-git-map` to `01-gitmap` (removing the hyphen).
- **Importable VM JSON**: Ensure `gitmap-ssh-nodes.json` and `vmpass.json` are fully structured for direct import via `gitmap ssh nodes import-json`.
- **SIO Templates Normalization (`seo-templates.json`, `seo-templates-cli.json`)**:
  - Convert all variables to PascalCase (e.g. `Company`, `CompanyUrl`, `MarekRole`, `AlimUrl`).
  - Template variable syntax: `${Variable}` (dollar with curly braces).
  - Enforce company name: `RISEUP ASIA LLC`.
  - Enforce URL: `alimkarim.com`, portfolio `alim-portfolio`, alias variations (`MD. Alim Ul Karim`, `AKA MD Alim Karim`).
  - Support array variables with random or indexed segment selection (e.g. `${Region[0]}` or random pick).
- **Commit-Pull Config & Boolean Prefix Standardization**:
  - Re-sequence config files in `01-gitmap` if applicable.
  - Enforce positive boolean prefixes: `isApplyTree` / `isTree`, `isApplyFinalSync` / `isFinalSync`, `isApplyCd` / `isCd`, `isRecreate`, `isPushImmediate`.
  - Use `workDir` variable to avoid repeating template paths in `imports`.
- **`D:\test-gitmap` PowerShell Automation**:
  - Create a one-liner / PowerShell script (`test-gitmap-recreate.ps1`) in `01-gitmap` to create, test, and remove `D:\test-gitmap` strictly located at `D:\test-gitmap` (NOT `D:\work\test-gitmap`).

---

## 4. Acceptance Criteria & Test Matrix

- **AC-67-001 (Screenshot Ingestion)**: Ingested screenshot from `https://prnt.sc/I2tSQfeut9GQ` into `assets/screenshots/gitmap-sug-investigation.png`.
- **AC-67-002 (SUG Subcommand & Help Interception)**: `gitmap sug add-projects help` displays help without adding "help" as project; `gitmap agy sug agy-running projects` succeeds.
- **AC-67-003 (SUG Target Validation & Aliases)**: `add-projects` supports `add`, `/add`, `ap`; validates local folder, alias, or URL, showing helpful recovery commands on missing targets.
- **AC-67-004 (SUG Watch & Auto-Clear Configuration)**: `gitmap sug watch` provides live monitoring; auto-clears targets on shutdown when configured.
- **AC-67-005 (SUG Web Dashboard)**: `gitmap sug ui` / `gitmap sug watch ui` launches interactive web UI with real-time status and project picker.
- **AC-67-006 (PE Path & Alias Support)**: `gitmap pe <path>` and `gitmap pe <alias>` resolve targets properly.
- **AC-67-007 (Help Menu Grouping & Search Documentation)**: `gitmap help <group>` filters commands; documents search vs AUM search.
- **AC-67-008 (Repo Secrets 01-gitmap Rename & Templates Normalization)**: `01-git-map` renamed to `01-gitmap`; SIO templates use PascalCase `${Variable}` with `RISEUP ASIA LLC`, `alimkarim.com`, and array support.
- **AC-67-009 (Commit-Pull Config & D:\test-gitmap Script)**: Boolean prefixes updated to positive `is*`; `workDir` path deduplication; script in `01-gitmap` tests `D:\test-gitmap` (outside `D:\work`).
