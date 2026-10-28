//! CLI handlers for `agm improvements` — plan 100, implement 10 at a time.

use antigravity_tools_lib::modules::improvements::{self, ImprovementKind};

pub(crate) fn dispatch_improvements_domain(args: &[String]) {
    if args
        .iter()
        .any(|a| a == "--help" || a == "-h" || a == "help")
        || args.first().map(|s| s.as_str()) == Some("help")
    {
        print_improvements_help();
        return;
    }

    let is_json = args.iter().any(|a| a == "--json" || a == "-j");
    let dry_run = args.iter().any(|a| a == "--dry-run" || a == "-n");
    let instance = parse_instance(args);
    let action = parse_action(args);

    let result = match action {
        "plan" => {
            let kind = parse_kind(args);
            improvements::plan_flywheel(kind, dry_run, &instance)
        }
        "next" | "next-10" | "next10" | "run" => improvements::next_batch(dry_run, &instance),
        "detail" => improvements::detail_flywheel(dry_run, &instance),
        "check" | "ck" => improvements::check_pending(),
        "status" | "st" | "ls" => improvements::load_state(),
        other => {
            eprintln!(
                "[ERROR] Unknown improvements subcommand '{other}'. Run 'agm improvements help'."
            );
            std::process::exit(1);
        }
    };

    match result {
        Ok(state) => {
            if is_json {
                match serde_json::to_string_pretty(&state) {
                    Ok(body) => println!("{body}"),
                    Err(e) => {
                        eprintln!("[ERROR] serialize improvements status: {e}");
                        std::process::exit(1);
                    }
                }
            } else {
                println!("{}", improvements::format_cli_text(&state));
            }
        }
        Err(e) => {
            eprintln!("[ERROR] {e}");
            std::process::exit(1);
        }
    }
}

fn parse_action(args: &[String]) -> &str {
    let mut i = 0;
    while i < args.len() {
        let arg = args[i].as_str();
        if arg == "--instance" || arg == "-i" {
            i += 2;
            continue;
        }
        if arg.starts_with('-') {
            i += 1;
            continue;
        }
        if matches!(arg, "ui" | "code" | "system" | "ideas" | "all" | "ux") {
            i += 1;
            continue;
        }
        return arg;
    }
    "status"
}

fn parse_kind(args: &[String]) -> ImprovementKind {
    for token in args {
        let lower = token.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "ui" | "code" | "system" | "ideas" | "all" | "ux"
        ) {
            return ImprovementKind::parse(&lower).unwrap_or(ImprovementKind::System);
        }
    }
    ImprovementKind::System
}

fn parse_instance(args: &[String]) -> String {
    let mut i = 0;
    while i < args.len() {
        let arg = &args[i];
        if (arg == "--instance" || arg == "-i") && i + 1 < args.len() {
            return args[i + 1].clone();
        }
        if let Some(rest) = arg.strip_prefix("--instance=") {
            return rest.to_string();
        }
        i += 1;
    }
    "default".to_string()
}

pub(crate) fn print_improvements_help() {
    println!("AGM 100 Improvements Flywheel:");
    println!(
        "  agm improvements plan [ui|code|system|ideas] [--dry-run] [--json] [--instance <id>]"
    );
    println!("  agm improvements next | next-10 | run [--dry-run] [--json] [--instance <id>]");
    println!("  agm improvements status [--json]");
    println!("  agm improvements check [--json]");
    println!("  agm improvements detail [--dry-run] [--json] [--instance <id>]");
    println!("  agm improvements help");
    println!();
    println!("Description:");
    println!("  Plan exactly 100 improvements as 10 batches of 10, then implement each batch");
    println!("  as a sequence of thought. Wraps 01-prompts/30-improvements/ (01–07).");
    println!("  Ledger: .ai-memory/improvements/flywheel.json");
    println!("  Same option exists on the Telegram page UI and as /improvements in Telegram.");
    println!();
    println!("Commands:");
    println!("  plan     Materialize 10 batch stubs and enqueue the discovery prompt (01–05)");
    println!("  next     Claim the next pending batch of 10 and enqueue 07-implement-next-10");
    println!("  status   Show wave, kind, and per-batch pending|dispatched|done");
    println!("  check    Refresh completed-batch counts from the ledger");
    println!("  detail   Enqueue 04-next-detail-out-100-improvements.md");
    println!("  help     Show this help");
    println!();
    println!("Kinds:");
    println!("  ui       01-prompts/30-improvements/01-next-100-ui-improvements.md");
    println!("  code     01-prompts/30-improvements/02-next-100-code-improvements.md");
    println!(
        "  system   01-prompts/30-improvements/03-next-100-whole-system-improvements.md (default)"
    );
    println!("  ideas    01-prompts/30-improvements/05-next-100-ideas-for-improvements.md");
    println!();
    println!("Options:");
    println!("  --dry-run, -n     Plan or preview next-10 without enqueueing a prompt");
    println!("  --json, -j        Machine-readable FlywheelState JSON");
    println!("  --instance, -i    Target instance (default: default). Never use this to stop v2.");
    println!("  --help, -h        Show help message");
    println!();
    println!("Telegram:");
    println!("  /improvements            Status card with Plan 100 / Next 10 / Status / Check");
    println!("  /improvements plan [ui]  Start a 100-item wave");
    println!("  /improvements next       Implement the next 10");
    println!("  /next10                  Alias for next-10");
    println!("  Callbacks: im:pl  im:n10  im:st  im:ck  (≤60 bytes)");
    println!();
    println!("Examples:");
    println!("  agm improvements help");
    println!("  agm improvements plan ui --dry-run --json");
    println!("  agm improvements status --json");
    println!("  agm improvements next --dry-run --json");
    println!("  agm improvements check --json");
    println!("  agm improvements plan system");
    println!("  agm improvements next");
}
