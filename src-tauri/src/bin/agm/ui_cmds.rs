//! ui_cmds — CLI bridge to the AGM UI: open / goto / ls.
//!
//! Task-03 of `auditing-and-cli-ui-integration`. Task-04 owns the
//! scroll/click/help sections (appended below the marker at the end of this
//! file).
//!
//! Conventions:
//! - `agm ui open` delegates to
//!   `antigravity_tools_lib::modules::delegate_updater::open_ui` — the same
//!   service route the existing `open-ui` CLI arm uses. Process spawning is
//!   never reimplemented here (AGENTS.md process-spawn invariant).
//! - `agm ui goto <route>` validates against the route registry extracted
//!   from `src/App.tsx` (`createBrowserRouter`) plus the settings subpage
//!   tabs in `src/pages/Settings.tsx`. Navigation itself is requested through
//!   the local management API (`POST /api/ui/navigate`) hosted by the running
//!   UI process on 127.0.0.1:8045.
//! - `agm ui ls [selector]` lists the route registry plus the static per-route
//!   element manifest defined in this module.
//! - CLI-driven actions are recorded with `actor=cli` through the Task-01
//!   audit storage (`audit::append_entry` → `<data-dir>/ui-audit/ui_audit.db`).
//!   The storage module is reused as-is and never duplicated here.
//! - "No Silent Errors": every failure prints the real message to stderr with
//!   a non-zero exit; nothing fails silently.
//!
//! LIMITATION (documented, not hidden): a headless CLI process cannot read the
//! running UI's live DOM — there is no element-tree introspection endpoint in
//! this build. `ls` therefore reports the static route/element manifest, and
//! `goto` / `ls --current` drive through the planned management-API surface.
//! If the running AGM build does not expose those endpoints, the commands
//! fail loudly with the real error (never a fake success).
//!
//! LEAD FOLLOW-UP (backend, outside this file): expose two routes on the
//! authenticated admin API (`src-tauri/src/proxy/server/router_admin_*.rs`,
//! mounted under `/api`) so these commands work against a running UI:
//!   - `POST /api/ui/navigate`  body `{ "route": "/settings/supabase" }`
//!     → emit the navigation to the frontend (e.g. a `ui://navigate` Tauri
//!     event consumed by the router) and return the new route.
//!   - `GET /api/ui/element-tree` → return the live element tree
//!     (route, components, element ids, focusable elements) as JSON.
//! Until those exist, `goto`/`ls --current` report the missing endpoint
//! explicitly.

use antigravity_tools_lib::modules::audit;
use antigravity_tools_lib::modules::cli::{
    forward_to_local_rest, is_daemon_running, CliContext, CliEnvelope,
};
use once_cell::sync::Lazy;

/// One UI route: canonical path, React page component, and a static element
/// manifest. Component names are grounded in `src/App.tsx` imports and the
/// page subdirectories; `known_ids` holds element `id=` attributes actually
/// found in the page source; `focusable` lists focusable element *kinds*
/// (never invented ids).
struct UiRoute {
    path: &'static str,
    page: &'static str,
    description: &'static str,
    components: &'static [&'static str],
    known_ids: &'static [&'static str],
    focusable: &'static [&'static str],
}

/// Route registry: top-level routes from `src/App.tsx` `createBrowserRouter`
/// plus the settings subpage tabs from `src/pages/Settings.tsx`.
static ROUTE_REGISTRY: Lazy<Vec<UiRoute>> = Lazy::new(|| {
    vec![
        UiRoute {
            path: "dashboard",
            page: "Dashboard",
            description: "Overview dashboard",
            components: &["Dashboard"],
            known_ids: &[],
            focusable: &["buttons", "links", "charts"],
        },
        UiRoute {
            path: "accounts",
            page: "Accounts",
            description: "Account management (default landing page)",
            components: &["AccountsContent", "AccountsDialogs"],
            known_ids: &[],
            focusable: &["buttons", "links", "inputs", "dialogs"],
        },
        UiRoute {
            path: "instances",
            page: "Instances",
            description: "Instance (profile) management",
            components: &[
                "InstancesHeader",
                "InstanceCard",
                "InstanceCardActions",
                "InstanceCardProjects",
                "CreateEditDialogs",
                "CopyInstanceDialog",
                "SwitchAccountDialog",
                "InstancesStates",
            ],
            known_ids: &["launch-immediately-check"],
            focusable: &["buttons", "links", "inputs", "checkboxes", "dialogs"],
        },
        UiRoute {
            path: "api-proxy",
            page: "ApiProxy",
            description: "Reverse-proxy / API gateway configuration",
            components: &[
                "ModelMappingCard",
                "AccountSchedulingSection",
                "AdvancedSettingsSection",
                "CliSetupTab",
                "CloudflaredSection",
                "CollapsibleCard",
                "ExternalProvidersSection",
                "McpSystemSection",
            ],
            known_ids: &["custom-key"],
            focusable: &["buttons", "inputs", "selects", "collapsible cards"],
        },
        UiRoute {
            path: "monitor",
            page: "Monitor",
            description: "Runtime monitor",
            components: &["Monitor"],
            known_ids: &[],
            focusable: &["buttons", "links"],
        },
        UiRoute {
            path: "token-stats",
            page: "TokenStats",
            description: "Token usage statistics",
            components: &["StatsCharts", "Tooltips"],
            known_ids: &[],
            focusable: &["buttons", "links", "charts"],
        },
        UiRoute {
            path: "user-token",
            page: "UserToken",
            description: "User token management",
            components: &["TokenDialogs"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "dialogs"],
        },
        UiRoute {
            path: "apikey-fun",
            page: "ApiKeyFun",
            description: "API key fun utilities",
            components: &["ApiKeyList", "SyncSection"],
            known_ids: &[],
            focusable: &["buttons", "inputs"],
        },
        UiRoute {
            path: "security",
            page: "Security",
            description: "Security settings and audit logs",
            components: &["Security"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "toggles"],
        },
        UiRoute {
            path: "email",
            page: "Email",
            description: "Email integration settings",
            components: &["Email"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "selects"],
        },
        UiRoute {
            path: "supabase",
            page: "Supabase",
            description: "Supabase sync dashboard",
            components: &["Supabase"],
            known_ids: &[],
            focusable: &["buttons", "inputs"],
        },
        UiRoute {
            path: "audit",
            page: "Audit",
            description: "UI interaction audit trail",
            components: &["AuditTable", "AuditDetailModal"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "table rows"],
        },
        UiRoute {
            path: "settings",
            page: "Settings",
            description: "Settings (all tabs)",
            components: &[
                "GeneralTab",
                "AccountTab",
                "ProxyTab",
                "EmailNotificationSettings",
                "ThemePicker",
                "SupabaseSyncSettings",
                "AdvancedTab",
                "DebugTab",
                "AboutTab",
            ],
            known_ids: &[],
            focusable: &["buttons", "inputs", "selects", "toggles", "checkboxes"],
        },
        UiRoute {
            path: "settings/general",
            page: "Settings",
            description: "Settings → General tab",
            components: &["GeneralTab"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "selects", "toggles"],
        },
        UiRoute {
            path: "settings/account",
            page: "Settings",
            description: "Settings → Account tab",
            components: &["AccountTab"],
            known_ids: &[],
            focusable: &["buttons", "inputs"],
        },
        UiRoute {
            path: "settings/proxy",
            page: "Settings",
            description: "Settings → Proxy tab",
            components: &["ProxyTab"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "selects"],
        },
        UiRoute {
            path: "settings/email",
            page: "Settings",
            description: "Settings → Email notifications tab",
            components: &["EmailNotificationSettings"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "toggles", "checkboxes"],
        },
        UiRoute {
            path: "settings/themes",
            page: "Settings",
            description: "Settings → Themes tab",
            components: &["ThemePicker"],
            known_ids: &[],
            focusable: &["buttons", "theme swatches"],
        },
        UiRoute {
            path: "settings/supabase",
            page: "Settings",
            description: "Settings → Supabase sync tab",
            components: &["SupabaseSyncSettings"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "toggles"],
        },
        UiRoute {
            path: "settings/advanced",
            page: "Settings",
            description: "Settings → Advanced tab",
            components: &["AdvancedTab", "AdvancedMaintenanceSection"],
            known_ids: &[],
            focusable: &["buttons", "inputs", "checkboxes"],
        },
        UiRoute {
            path: "settings/debug",
            page: "Settings",
            description: "Settings → Debug tab",
            components: &["DebugTab"],
            known_ids: &[],
            focusable: &["buttons", "inputs"],
        },
        UiRoute {
            path: "settings/about",
            page: "Settings",
            description: "Settings → About tab",
            components: &["AboutTab"],
            known_ids: &[],
            focusable: &["buttons", "links"],
        },
        UiRoute {
            path: "toolchain",
            page: "Toolchain",
            description: "Toolchain setup",
            components: &["ToggleRow"],
            known_ids: &[],
            focusable: &["buttons", "toggles"],
        },
    ]
});

/// Entry point wired from main.rs (mirrors `dispatch_fleet_domain_from_args`).
pub(crate) fn dispatch_ui_domain_from_args(args: &[String]) {
    if args.is_empty() {
        cmd_ui_open(&[]);
        return;
    }
    let sub = args[0]
        .trim_start_matches('/')
        .trim_start_matches('-')
        .to_lowercase();
    if sub == "help" || sub == "h" || args[0].starts_with('-') {
        cmd_ui_help_full(&args[1..]);
        return;
    }
    dispatch_ui_domain(&sub, &args[1..]);
}

pub(crate) fn dispatch_ui_domain(subcommand: &str, args: &[String]) {
    match subcommand {
        "open" => cmd_ui_open(args),
        "goto" => cmd_ui_goto(args),
        "ls" => cmd_ui_ls(args),
        "scroll" => cmd_ui_scroll(args),
        "click" => cmd_ui_click(args),
        "record" | "rec" | "audit" => cmd_ui_record(args),
        "help" | "h" => cmd_ui_help_full(args),
        _ => crate::common::handle_unknown_domain_command("ui", subcommand, args),
    }
}

/// Record one CLI-driven UI action in the Task-01 audit backend with
/// `actor=cli`. Reuses `audit::append_entry` (same SQLite DB the Tauri
/// commands write to) — never duplicated here.
///
/// Audit is auxiliary to the requested UI action: a failed write is surfaced
/// (never silent) but must not block the action itself.
// Justification: the audit record is bookkeeping; blocking `open`/`goto` on
// it would make a UI action fail for an observability reason.
fn record_cli_ui_action(
    method_name: &str,
    route: &str,
    trigger: &str,
    params: serde_json::Value,
    result: &str,
    message: &str,
) {
    let ui_target = audit::format_ui_target(route, Some("cli"), Some(method_name));
    let params_str = params.to_string();
    let ts = chrono::Utc::now().timestamp_millis();
    if let Err(err) = audit::append_entry(
        ts,
        "cli",
        method_name,
        &ui_target,
        trigger,
        &params_str,
        result,
        message,
    ) {
        eprintln!("[WARN] ui audit record failed (action still proceeds): {err}");
    }
}

/// `agm ui open` — open the AGM UI (launch if closed, bring to foreground if
/// running). Delegates to the shared `delegate_updater::open_ui` service
/// route; spawning logic is never reimplemented here.
pub(crate) fn cmd_ui_open(args: &[String]) {
    record_cli_ui_action(
        "ui open",
        "/",
        "programmatic",
        serde_json::json!({ "delegated": "delegate_updater::open_ui" }),
        "ok",
        "CLI requested UI open; launch handled by delegate_updater::open_ui",
    );
    antigravity_tools_lib::modules::delegate_updater::open_ui(args);
}

/// Normalize a user-supplied route: trim, drop the leading `/`, lowercase.
/// The app index redirects `/` to `/accounts`, so a bare `/` maps to
/// `accounts`.
fn normalize_route(raw: &str) -> String {
    let trimmed = raw.trim().trim_start_matches('/');
    if trimmed.is_empty() {
        return "accounts".to_string();
    }
    trimmed.to_lowercase()
}

fn find_route(route: &str) -> Option<&'static UiRoute> {
    ROUTE_REGISTRY.iter().find(|r| r.path == route)
}

/// Print the full valid route list (used on unknown routes — never silent).
fn print_valid_routes() {
    eprintln!("Valid routes:");
    for r in ROUTE_REGISTRY.iter() {
        eprintln!("  /{:<20} {}", r.path, r.description);
    }
}

/// True when the AGM UI backend is reachable: the Tauri app hosts the local
/// management API on 127.0.0.1:8045, or the `agm-alim` UI process is alive.
fn ui_backend_running() -> bool {
    if is_daemon_running() {
        return true;
    }
    let mut system = sysinfo::System::new();
    system.refresh_processes(sysinfo::ProcessesToUpdate::All);
    system.processes().values().any(|proc| {
        let name = proc.name().to_string_lossy().to_lowercase();
        let exe = proc
            .exe()
            .map(|p| p.to_string_lossy().to_lowercase())
            .unwrap_or_default();
        name.contains("agm-alim") || exe.contains("agm-alim")
    })
}

/// `agm ui goto <route>` — navigate the running UI to a page / settings page /
/// settings subpage. Unknown routes print the valid route list; when the UI
/// is not running the error tells the user to run `agm ui open` first.
pub(crate) fn cmd_ui_goto(args: &[String]) {
    let ctx = CliContext::parse(args);
    let raw = match ctx.positional_args.first() {
        Some(r) => r.clone(),
        None => {
            eprintln!("[ERROR] Missing route. Usage: agm ui goto <route>");
            print_valid_routes();
            std::process::exit(1);
        }
    };
    let route = normalize_route(&raw);
    let Some(entry) = find_route(&route) else {
        eprintln!("[ERROR] Unknown route '/{route}'.");
        print_valid_routes();
        std::process::exit(1);
    };
    if !ui_backend_running() {
        eprintln!("[ERROR] AGM UI is not running.");
        eprintln!("Run `agm ui open` first to launch it, then retry `agm ui goto /{route}`.");
        std::process::exit(1);
    }

    let params = serde_json::json!({ "route": format!("/{route}"), "page": entry.page });
    let body = serde_json::json!({ "route": format!("/{route}") });
    match forward_to_local_rest::<serde_json::Value>(
        reqwest::Method::POST,
        "/ui/navigate",
        Some(body),
    ) {
        Ok(resp) => {
            record_cli_ui_action(
                "ui goto",
                &format!("/{route}"),
                "navigation",
                params,
                "ok",
                "",
            );
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui goto",
                    None,
                    serde_json::json!({ "route": format!("/{route}"), "response": resp }),
                )
                .print_and_exit();
            }
            println!("[OK] AGM UI navigated to /{route} ({}).", entry.page);
        }
        Err(err) => {
            record_cli_ui_action(
                "ui goto",
                &format!("/{route}"),
                "navigation",
                params,
                "error",
                &err,
            );
            eprintln!("[ERROR] UI navigation failed: {err}");
            eprintln!("[HINT] The running AGM build did not accept POST /api/ui/navigate.");
            eprintln!(
                "       Lead follow-up: expose the UI-control endpoint on the management API"
            );
            eprintln!(
                "       (see the LEAD FOLLOW-UP block at the top of ui_cmds.rs), then retry."
            );
            eprintln!(
                "       The route '/{route}' is valid — open the UI and navigate by hand: agm ui open"
            );
            std::process::exit(1);
        }
    }
}

/// Print one route's element manifest (static registry — see module docs).
fn print_route_manifest(route: &UiRoute, json_output: bool) {
    if json_output {
        CliEnvelope::ok(
            "ui ls",
            None,
            serde_json::json!({
                "route": format!("/{}", route.path),
                "page": route.page,
                "description": route.description,
                "components": route.components,
                "known_element_ids": route.known_ids,
                "focusable_element_kinds": route.focusable,
                "note": "static manifest: no live DOM introspection from the headless CLI (see agm ui --help)",
            }),
        )
        .print_and_exit();
    }
    println!("Route: /{}", route.path);
    println!("Page: {}", route.page);
    println!("Description: {}", route.description);
    println!("Components:");
    for c in route.components {
        println!("  - {c}");
    }
    if route.known_ids.is_empty() {
        println!("Known element ids: (none catalogued)");
    } else {
        println!("Known element ids:");
        for id in route.known_ids {
            println!("  - #{id}");
        }
    }
    println!("Focusable element kinds: {}", route.focusable.join(", "));
}

/// `agm ui ls [selector]` — list the UI element tree of the current view.
///
/// LIMITATION: the headless CLI cannot read the live DOM, so `ls` reports the
/// static route/element manifest (route, components, catalogued element ids,
/// focusable element kinds). `agm ui ls --current` attempts the planned
/// `GET /api/ui/element-tree` endpoint and fails loudly when it is missing.
pub(crate) fn cmd_ui_ls(args: &[String]) {
    let ctx = CliContext::parse(args);
    let selector = ctx.positional_args.first().map(|s| normalize_route(s));

    if selector.as_deref() == Some("current") {
        if !ui_backend_running() {
            eprintln!("[ERROR] AGM UI is not running.");
            eprintln!("Run `agm ui open` first to launch it, then retry `agm ui ls --current`.");
            std::process::exit(1);
        }
        match forward_to_local_rest::<serde_json::Value>(
            reqwest::Method::GET,
            "/ui/element-tree",
            None,
        ) {
            Ok(tree) => {
                if ctx.json_output {
                    CliEnvelope::ok("ui ls", None, tree).print_and_exit();
                }
                println!(
                    "{}",
                    serde_json::to_string_pretty(&tree)
                        .unwrap_or_else(|e| format!("[ERROR] serialize element tree: {e}"))
                );
            }
            Err(err) => {
                eprintln!("[ERROR] Live element tree unavailable: {err}");
                eprintln!("[HINT] The running AGM build did not accept GET /api/ui/element-tree.");
                eprintln!(
                    "       Lead follow-up: expose the UI-control endpoint on the management API"
                );
                eprintln!("       (see the LEAD FOLLOW-UP block at the top of ui_cmds.rs).");
                eprintln!(
                    "       Use `agm ui ls [<route>]` for the static route/element manifest."
                );
                std::process::exit(1);
            }
        }
        return;
    }

    match selector {
        None => {
            if ctx.json_output {
                let data: Vec<serde_json::Value> = ROUTE_REGISTRY
                    .iter()
                    .map(|r| {
                        serde_json::json!({
                            "route": format!("/{}", r.path),
                            "page": r.page,
                            "description": r.description,
                            "components": r.components,
                            "known_element_ids": r.known_ids,
                            "focusable_element_kinds": r.focusable,
                        })
                    })
                    .collect();
                CliEnvelope::ok("ui ls", None, data).print_and_exit();
            }
            println!("AGM UI routes ({}):", ROUTE_REGISTRY.len());
            for r in ROUTE_REGISTRY.iter() {
                println!("  /{:<20} {} [{}]", r.path, r.description, r.page);
            }
            println!();
            println!("Run `agm ui ls <route>` for a route's element manifest.");
            println!("Note: static manifest — no live DOM introspection from the headless CLI (see `agm ui --help`).");
        }
        Some(route) => match find_route(&route) {
            Some(entry) => print_route_manifest(entry, ctx.json_output),
            None => {
                eprintln!("[ERROR] Unknown route '/{route}'.");
                print_valid_routes();
                std::process::exit(1);
            }
        },
    }
}

/// `agm ui --help` / `agm ui help` — usage for the open/goto/ls sections.
pub(crate) fn cmd_ui_help() {
    println!("agm ui — bridge between the CLI and the AGM UI (Task-03: open/goto/ls)");
    println!();
    println!("Usage:");
    println!("  agm ui                      Open the AGM UI (same as `agm ui open`)");
    println!("  agm ui open                 Open the AGM UI (launch if closed, focus if running)");
    println!("  agm ui goto <route>         Navigate the running UI to a page");
    println!("  agm ui ls [route|--current] List the UI element tree (static manifest)");
    println!("  agm ui help                 Show this help");
    println!();
    println!("Route examples:");
    println!("  settings          accounts          instances          audit");
    println!("  settings/supabase settings/themes  api-proxy          toolchain");
    println!();
    println!("Options:");
    println!("  --json    Machine-readable JSON output (goto / ls)");
    println!();
    println!("Audit: every `open`/`goto` action is recorded with actor=cli in the");
    println!("Task-01 UI audit backend (<data-dir>/ui-audit/ui_audit.db).");
    println!();
    println!("Limitations (headless CLI):");
    println!("  - `ls` reports a static route/element manifest; it cannot read the");
    println!("    running UI's live DOM. `ls --current` and `goto` drive through the");
    println!("    planned management-API endpoints POST /api/ui/navigate and");
    println!("    GET /api/ui/element-tree — if the running build lacks them, the");
    println!("    command fails loudly with the real error instead of a fake success.");
    print_valid_routes();
}

// Task-04: scroll/click/help sections go below

// ──────────────────────────────────────────────────────────────────────────
// Task-04 — scroll / click / help
//
// HONEST-CAPABILITY MODEL: a headless CLI process cannot read or drive the
// running UI's live DOM by itself. `scroll` / `click` therefore follow three
// steps, each explicit about what did and did not happen:
//   1. VALIDATE the element id against the static element manifest
//      (`ROUTE_REGISTRY.known_ids` — only ids actually found in page
//      sources; never invented ids). Unknown ids fail loudly with the
//      catalogued-id list.
//   2. RECORD the intent in the Task-01 audit backend (`actor=cli`,
//      `trigger=scroll` / `trigger=click`) via `record_cli_ui_action`.
//   3. ATTEMPT the live bridge: `POST /api/ui/scroll` / `POST /api/ui/click`
//      on the local management API (127.0.0.1:8045). When the running AGM
//      build exposes the endpoint, the backend emits `agm://ui-scroll` /
//      `agm://ui-click` (mirroring `agm://ui-navigate` in
//      `src-tauri/src/proxy/server/admin_ui.rs`) and the frontend hook
//      `src/hooks/useUiRemoteControl.ts` performs the real scroll/click —
//      the same code path as a GUI interaction. When the endpoint is
//      missing, the command says so explicitly (`[NOTE]`, `performed:
//      false`) instead of reporting a fake success.
//
// LEAD FOLLOW-UP (backend, outside this file): expose on the authenticated
// admin API (`src-tauri/src/proxy/server/router_admin_*.rs`, mounted under
// `/api`):
//   - `POST /api/ui/scroll`  body `{ "route": "/instances",
//     "element_id": "launch-immediately-check", "actor": "cli" }`
//     → emit `agm://ui-scroll` with `{ element_id, route, actor }`.
//   - `POST /api/ui/click`   same body shape
//     → emit `agm://ui-click` with `{ element_id, route, actor }`.
// The frontend listeners for both events were appended to
// `src/hooks/useUiRemoteControl.ts` by Task-04; only the backend endpoints
// are still missing.
//
// DISPATCH WIRING (lead — above the Task-04 marker, do not edit from here):
// add `"scroll" => cmd_ui_scroll(args),` and `"click" => cmd_ui_click(args),`
// to the `dispatch_ui_domain` match, and route the `help` / `--help` arms to
// `cmd_ui_help_full(args)` to make the comprehensive help below live. Until
// then `scroll` / `click` are implemented but unreachable from the CLI.

/// Normalize an element id: trim whitespace and accept a leading `#`
/// (CSS-id style): `agm ui scroll #launch-immediately-check`.
fn normalize_element_id(raw: &str) -> String {
    raw.trim().trim_start_matches('#').to_string()
}

/// Every registry route that catalogues `element_id` in its static
/// `known_ids` manifest. Element ids are never invented: only ids actually
/// found in page sources are listed (see `ROUTE_REGISTRY`).
fn routes_for_element_id(element_id: &str) -> Vec<&'static UiRoute> {
    ROUTE_REGISTRY
        .iter()
        .filter(|r| r.known_ids.iter().any(|id| *id == element_id))
        .collect()
}

/// Print every catalogued element id with its route — used in error hints so
/// an unknown id never fails silently.
fn print_catalogued_elements() {
    eprintln!("Catalogued element ids (static manifest — `agm ui ls <route>` for details):");
    for r in ROUTE_REGISTRY.iter().filter(|r| !r.known_ids.is_empty()) {
        for id in r.known_ids {
            eprintln!("  #{id:<24} /{}", r.path);
        }
    }
}

/// Resolve the scroll/click target: `(element_id, route)`.
///
/// Accepted forms: `agm ui <scroll|click> <element-id>` or
/// `agm ui <scroll|click> <element-id> <route>`. The id is validated against
/// the static element manifest; an id catalogued on several routes requires
/// the `<route>` argument. Every failure prints the real reason and exits
/// non-zero.
fn resolve_element_target(ctx: &CliContext, action: &str) -> (String, &'static UiRoute) {
    let raw_id = match ctx.positional_args.first() {
        Some(id) => id.clone(),
        None => {
            eprintln!("[ERROR] Missing element id. Usage: agm ui {action} <element-id> [route]");
            print_catalogued_elements();
            std::process::exit(1);
        }
    };
    let element_id = normalize_element_id(&raw_id);
    if element_id.is_empty() {
        eprintln!("[ERROR] Empty element id. Usage: agm ui {action} <element-id> [route]");
        std::process::exit(1);
    }

    if let Some(route) = ctx.positional_args.get(1).map(|s| normalize_route(s)) {
        let Some(entry) = find_route(&route) else {
            eprintln!("[ERROR] Unknown route '/{route}'.");
            print_valid_routes();
            std::process::exit(1);
        };
        if !entry.known_ids.iter().any(|id| *id == element_id) {
            eprintln!("[ERROR] Element '#{element_id}' is not catalogued on '/{route}'.");
            let known_on = routes_for_element_id(&element_id);
            if known_on.is_empty() {
                eprintln!("[HINT] '#{element_id}' is not catalogued on any route.");
            } else {
                eprintln!("[HINT] '#{element_id}' is catalogued on:");
                for r in &known_on {
                    eprintln!("  /{}", r.path);
                }
            }
            print_catalogued_elements();
            std::process::exit(1);
        }
        return (element_id, entry);
    }

    match routes_for_element_id(&element_id).as_slice() {
        [] => {
            eprintln!("[ERROR] Unknown element id '#{element_id}'.");
            eprintln!("[HINT] Only element ids actually found in page sources are catalogued -");
            eprintln!("[HINT] never invent an id. `agm ui ls <route>` shows a route's manifest.");
            print_catalogued_elements();
            std::process::exit(1);
        }
        [entry] => (element_id, *entry),
        many => {
            eprintln!(
                "[ERROR] Ambiguous element id '#{element_id}' — catalogued on several routes:"
            );
            for r in many {
                eprintln!("  /{}", r.path);
            }
            eprintln!("Disambiguate with the route: agm ui {action} {element_id} <route>");
            std::process::exit(1);
        }
    }
}

/// Attempt the live DOM bridge for a scroll/click through the local
/// management API. Mirrors the `goto` pattern (`POST /api/ui/navigate`):
/// the backend validates, emits the `agm://ui-scroll` / `agm://ui-click`
/// Tauri event, and the frontend performs the real action through the same
/// code path as a GUI interaction.
///
/// Returns `Err` with the real backend error when the endpoint is missing —
/// the caller reports it honestly and never claims the action happened.
fn attempt_ui_element_bridge(
    endpoint: &str,
    element_id: &str,
    route_path: &str,
) -> Result<serde_json::Value, String> {
    let body = serde_json::json!({
        "route": format!("/{route_path}"),
        "element_id": element_id,
        "actor": "cli",
    });
    forward_to_local_rest::<serde_json::Value>(reqwest::Method::POST, endpoint, Some(body))
}

/// Shared tail of `scroll` / `click`: record the audit entry, then either
/// report the performed action or honestly report the missing bridge.
///
/// `performed=false` exits 0 by design: the command itself was well-formed
/// and fully processed (validated + audited); the environment lacks the
/// live-DOM bridge, which is reported here in full rather than hidden. A
/// failed *command* (bad id, unknown route, UI not running) exits non-zero
/// before this point.
#[allow(clippy::too_many_arguments)]
fn finish_element_action(
    command: &str,
    trigger: &str,
    element_id: &str,
    route_path: &'static str,
    outcome: Result<serde_json::Value, String>,
    json_output: bool,
    ok_summary: &str,
) {
    let route_label = format!("/{route_path}");
    let params = serde_json::json!({ "element_id": element_id, "route": route_label });
    match outcome {
        Ok(resp) => {
            record_cli_ui_action(command, &route_label, trigger, params, "ok", ok_summary);
            if json_output {
                CliEnvelope::ok(
                    command,
                    None,
                    serde_json::json!({
                        "element_id": element_id,
                        "route": route_label,
                        "performed": true,
                        "response": resp,
                    }),
                )
                .print_and_exit();
            }
            println!("[OK] {ok_summary}");
        }
        Err(err) => {
            let note = format!(
                "live {trigger} bridge unavailable: {err}. \
                 POST /api/ui/{trigger} is not exposed by the running AGM build \
                 (lead follow-up: add the endpoint in \
                 src-tauri/src/proxy/server/admin_ui.rs, emitting agm://ui-{trigger} \
                 for src/hooks/useUiRemoteControl.ts). \
                 Nothing was scrolled, focused, or clicked."
            );
            record_cli_ui_action(command, &route_label, trigger, params, "pending", &note);
            if json_output {
                CliEnvelope::ok(
                    command,
                    None,
                    serde_json::json!({
                        "element_id": element_id,
                        "route": route_label,
                        "performed": false,
                        "note": note,
                    }),
                )
                .print_and_exit();
            }
            // Honest capability statement — never a fake success.
            eprintln!(
                "[NOTE] #{element_id} is a valid element on {route_label} \
                 (validated against the static element manifest),"
            );
            eprintln!(
                "[NOTE] and the intent was recorded in the UI audit trail \
                 (actor=cli, trigger={trigger})."
            );
            eprintln!("[NOTE] The live {trigger} bridge is NOT available: {err}");
            eprintln!("[NOTE] Nothing was scrolled, focused, or clicked.");
            eprintln!("[NOTE] Lead follow-up: expose POST /api/ui/{trigger} on the management API");
            eprintln!("[NOTE] (see the Task-04 block at the end of ui_cmds.rs).");
        }
    }
}

/// `agm ui scroll <element-id> [route]` — scroll the view until the element
/// is visible, then focus it.
///
/// Flow: validate the id against the static manifest → require the running
/// UI → record the audit entry (`actor=cli`, `trigger=scroll`) → attempt the
/// live bridge (`POST /api/ui/scroll`). Unknown ids, unknown routes, and a
/// stopped UI fail loudly; a missing bridge endpoint is reported honestly
/// (see `finish_element_action`) instead of faked.
pub(crate) fn cmd_ui_scroll(args: &[String]) {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_ui_help_topic("scroll");
        return;
    }
    let ctx = CliContext::parse(args);
    let (element_id, route) = resolve_element_target(&ctx, "scroll");
    if !ui_backend_running() {
        eprintln!("[ERROR] AGM UI is not running.");
        eprintln!("Run `agm ui open` first to launch it, then retry `agm ui scroll {element_id}`.");
        std::process::exit(1);
    }
    let outcome = attempt_ui_element_bridge("/ui/scroll", &element_id, route.path);
    finish_element_action(
        "ui scroll",
        "scroll",
        &element_id,
        route.path,
        outcome,
        ctx.json_output,
        &format!("Scrolled to #{element_id} on /{}; focused.", route.path),
    );
}

/// `agm ui click <element-id> [route]` — activate an element through the
/// same code path as a user click: the backend emits `agm://ui-click` and
/// the frontend hook performs a real `.click()`, so the audit recorder logs
/// it with `actor=cli` and behavior matches the GUI exactly.
///
/// Same honest-capability flow as `scroll` (validate → require UI → audit →
/// attempt bridge); a missing bridge endpoint is reported, never faked.
pub(crate) fn cmd_ui_click(args: &[String]) {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_ui_help_topic("click");
        return;
    }
    let ctx = CliContext::parse(args);
    let (element_id, route) = resolve_element_target(&ctx, "click");
    if !ui_backend_running() {
        eprintln!("[ERROR] AGM UI is not running.");
        eprintln!("Run `agm ui open` first to launch it, then retry `agm ui click {element_id}`.");
        std::process::exit(1);
    }
    let outcome = attempt_ui_element_bridge("/ui/click", &element_id, route.path);
    finish_element_action(
        "ui click",
        "click",
        &element_id,
        route.path,
        outcome,
        ctx.json_output,
        &format!(
            "Clicked #{element_id} on /{} (same handler as a GUI click).",
            route.path
        ),
    );
}

/// One-line purpose + concrete example + expected output for a single
/// `agm ui` subcommand. Used by `agm ui <scroll|click> --help` and by the
/// comprehensive help below — never a bare flag list.
fn print_ui_help_topic(topic: &str) {
    match topic {
        "open" => {
            println!("open — Open the AGM UI.");
            println!("  Purpose:  Launch the AGM UI when it is closed, or bring it to the");
            println!("            foreground when it is already running. Delegates to the same");
            println!("            service route as the existing `open-ui` command — process");
            println!("            spawning is never reimplemented in the UI bridge.");
            println!("  Example:  agm ui open");
            println!(
                "  Output:   (the UI window opens or gains focus; nothing is printed on success)"
            );
        }
        "goto" => {
            println!("goto — Navigate the running UI to a page.");
            println!("  Purpose:  Drive the running UI to any route: a top-level page, the");
            println!("            settings page, or a settings tab (settings/supabase, ...).");
            println!("  Example:  agm ui goto settings/supabase");
            println!("  Output:   [OK] AGM UI navigated to /settings/supabase (Settings).");
            println!("  Errors:   unknown route  -> prints the valid route list, exits 1.");
            println!("            UI not running -> `Run \\`agm ui open\\` first ...`, exits 1.");
        }
        "ls" => {
            println!("ls — List UI routes and the element manifest.");
            println!("  Purpose:  With no argument, list every route. With a route, print that");
            println!("            route's element manifest: page component, catalogued element");
            println!("            ids, and focusable element kinds. `--current` asks the running");
            println!("            UI for its live element tree instead of the static manifest.");
            println!("  Example:  agm ui ls instances");
            println!("  Output:");
            println!("            Route: /instances");
            println!("            Page: Instances");
            println!("            Description: Instance (profile) management");
            println!("            Components:");
            println!("              - InstancesHeader");
            println!("              - InstanceCard");
            println!("              ...");
            println!("            Known element ids:");
            println!("              - #launch-immediately-check");
            println!(
                "            Focusable element kinds: buttons, links, inputs, checkboxes, dialogs"
            );
            println!("  Example:  agm ui ls --current");
            println!("  Output:   (pretty-printed live element tree JSON from the running UI)");
        }
        "scroll" => {
            println!("scroll — Scroll the view until the element is visible, then focus it.");
            println!("  Purpose:  Bring a catalogued element into view and focus it. The id is");
            println!("            validated against the static element manifest first — unknown");
            println!(
                "            ids fail with the catalogued-id list instead of failing silently."
            );
            println!("  Example:  agm ui scroll launch-immediately-check instances");
            println!(
                "  Output:   [OK] Scrolled to #launch-immediately-check on /instances; focused."
            );
            println!("  Bridge:   the live scroll happens through POST /api/ui/scroll (backend).");
            println!("            When the running build lacks that endpoint, the command still");
            println!(
                "            validates + audits and says so explicitly — nothing is scrolled:"
            );
            println!(
                "  Output:   [NOTE] #launch-immediately-check is a valid element on /instances ..."
            );
            println!("            [NOTE] Nothing was scrolled, focused, or clicked.");
        }
        "click" => {
            println!("click — Activate an element, exactly like a user click.");
            println!("  Purpose:  Drive the element's real handler through the same code path as");
            println!("            a GUI click, so the audit recorder logs it with actor=cli and");
            println!("            behavior matches the GUI exactly.");
            println!("  Example:  agm ui click launch-immediately-check instances");
            println!("  Output:   [OK] Clicked #launch-immediately-check on /instances (same handler as a GUI click).");
            println!("  Bridge:   same honest-bridge behavior as `scroll` (POST /api/ui/click).");
        }
        "record" => {
            println!("record — Control and inspect the UI interaction audit recorder.");
            println!("  Purpose:  Start, stop, check status, list/tail, clear, or export recorded");
            println!("            UI, CLI, and system interactions stored in ui_audit.db.");
            println!("  Example:  agm ui record start");
            println!("  Output:   [OK] UI audit recording started.");
            println!("  Example:  agm ui record list --limit 20 --json");
            println!("  Output:   (JSON envelope containing recorded interaction entries)");
        }
        _ => {
            eprintln!(
                "[ERROR] Unknown help topic '{topic}'. Topics: open, goto, ls, scroll, click, record."
            );
            std::process::exit(1);
        }
    }
}

/// `agm ui record <start|stop|status|list|tail|clear|export> [--limit N] [--json]`
pub(crate) fn cmd_ui_record(args: &[String]) {
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_ui_help_topic("record");
        return;
    }
    let ctx = CliContext::parse(args);
    let action = ctx
        .positional_args
        .first()
        .map(|s| s.to_lowercase())
        .unwrap_or_else(|| "status".to_string());

    match action.as_str() {
        "start" | "on" => {
            let was = match audit::start_recording() {
                Ok(w) => w,
                Err(err) => {
                    eprintln!("[ERROR] Failed to start UI audit recording: {err}");
                    std::process::exit(1);
                }
            };
            if is_daemon_running() {
                let _ = forward_to_local_rest::<serde_json::Value>(
                    reqwest::Method::POST,
                    "/ui/record/start",
                    Some(serde_json::json!({})),
                );
            }
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record start",
                    None,
                    serde_json::json!({ "recording": true, "was_recording": was }),
                )
                .print_and_exit();
            }
            println!("[OK] UI audit recording started (was_recording={was}).");
        }
        "stop" | "off" => {
            let was = audit::stop_recording();
            if is_daemon_running() {
                let _ = forward_to_local_rest::<serde_json::Value>(
                    reqwest::Method::POST,
                    "/ui/record/stop",
                    Some(serde_json::json!({})),
                );
            }
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record stop",
                    None,
                    serde_json::json!({ "recording": false, "was_recording": was }),
                )
                .print_and_exit();
            }
            println!("[OK] UI audit recording stopped (was_recording={was}).");
        }
        "status" => {
            let recording = audit::is_recording();
            let count = audit::list_recorded(5000).map(|v| v.len()).unwrap_or(0);
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record status",
                    None,
                    serde_json::json!({ "recording": recording, "entries_count": count }),
                )
                .print_and_exit();
            }
            println!(
                "UI audit recording: {} ({} recorded entries)",
                if recording { "ACTIVE" } else { "STOPPED" },
                count
            );
        }
        "list" | "ls" | "tail" => {
            let mut flag_limit: Option<u32> = None;
            let mut idx = 0;
            while idx < args.len() {
                match args[idx].as_str() {
                    "--limit" | "-n" => {
                        if let Some(val) = args.get(idx + 1) {
                            flag_limit = val.parse::<u32>().ok();
                        }
                        idx += 1;
                    }
                    arg if arg.starts_with("--limit=") => {
                        flag_limit = arg
                            .strip_prefix("--limit=")
                            .and_then(|v| v.parse::<u32>().ok());
                    }
                    _ => {}
                }
                idx += 1;
            }
            let limit = flag_limit
                .or_else(|| {
                    ctx.positional_args
                        .get(1)
                        .and_then(|v| v.parse::<u32>().ok())
                })
                .unwrap_or(50)
                .clamp(1, 5000);
            let entries = match audit::list_recorded(limit) {
                Ok(e) => e,
                Err(err) => {
                    eprintln!("[ERROR] Failed to list UI audit entries: {err}");
                    std::process::exit(1);
                }
            };
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record list",
                    None,
                    serde_json::json!({
                        "recording": audit::is_recording(),
                        "count": entries.len(),
                        "entries": entries,
                    }),
                )
                .print_and_exit();
            }
            println!(
                "UI Audit Entries ({} shown, recording={}):",
                entries.len(),
                audit::is_recording()
            );
            for e in &entries {
                println!(
                    "  #{} [{}] actor={} trigger={} method={} target={} result={} params={} msg={}",
                    e.id,
                    e.timestamp,
                    e.actor,
                    e.trigger,
                    e.method_name,
                    e.ui_target,
                    e.result,
                    e.params,
                    e.message
                );
            }
        }
        "clear" | "reset" => {
            let deleted = match audit::clear_recording() {
                Ok(d) => d,
                Err(err) => {
                    eprintln!("[ERROR] Failed to clear UI audit entries: {err}");
                    std::process::exit(1);
                }
            };
            if is_daemon_running() {
                let _ = forward_to_local_rest::<serde_json::Value>(
                    reqwest::Method::POST,
                    "/ui/record/clear",
                    Some(serde_json::json!({})),
                );
            }
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record clear",
                    None,
                    serde_json::json!({ "deleted": deleted }),
                )
                .print_and_exit();
            }
            println!("[OK] Cleared {deleted} UI audit record(s).");
        }
        "export" => {
            let path = match audit::export_recording() {
                Ok(p) => p,
                Err(err) => {
                    eprintln!("[ERROR] Failed to export UI audit entries: {err}");
                    std::process::exit(1);
                }
            };
            let path_str = path.to_string_lossy().to_string();
            if ctx.json_output {
                CliEnvelope::ok(
                    "ui record export",
                    None,
                    serde_json::json!({ "path": path_str }),
                )
                .print_and_exit();
            }
            println!("[OK] Exported UI audit entries to {path_str}");
        }
        other => {
            eprintln!(
                "[ERROR] Unknown `agm ui record` action '{other}'. Expected: start, stop, status, list, tail, clear, export."
            );
            std::process::exit(1);
        }
    }
}

/// `agm ui help [topic]` / `agm ui --help` (Task-04) — comprehensive help
/// with examples for EVERY subcommand (open, goto, ls, scroll, click).
///
/// Each subcommand gets a one-line purpose, a concrete example, and the
/// expected output — no bare flag lists. With a topic argument
/// (`agm ui help scroll`) only that subcommand's help is printed.
///
/// DISPATCH NOTE (lead): `dispatch_ui_domain` above the Task-04 marker
/// currently routes `help` / `--help` to Task-03's `cmd_ui_help`
/// (open/goto/ls only). Point those arms at `cmd_ui_help_full` to make this
/// comprehensive help live.
pub(crate) fn cmd_ui_help_full(args: &[String]) {
    let ctx = CliContext::parse(args);
    if let Some(topic) = ctx.positional_args.first() {
        print_ui_help_topic(&topic.to_lowercase());
        return;
    }
    println!("agm ui — bridge between the CLI and the AGM UI");
    println!();
    println!("Usage:");
    println!("  agm ui                      Open the AGM UI (same as `agm ui open`)");
    println!("  agm ui open                 Open the AGM UI (launch if closed, focus if running)");
    println!("  agm ui goto <route>         Navigate the running UI to a page");
    println!("  agm ui ls [route|--current] List routes and the element manifest");
    println!(
        "  agm ui scroll <id> [route]  Scroll the view until the element is visible, then focus it"
    );
    println!("  agm ui click <id> [route]   Activate an element (same code path as a GUI click)");
    println!("  agm ui help [topic]         Show this help (topic: open, goto, ls, scroll, click)");
    println!();
    println!("Subcommands — what to do, with a concrete example and the expected output:");
    println!();
    for topic in ["open", "goto", "ls", "scroll", "click"] {
        print_ui_help_topic(topic);
        println!();
    }
    println!("Element ids:");
    println!("  Only element ids actually present in page sources are catalogued — never");
    println!("  invent an id. Show them with `agm ui ls <route>`; an unknown id also prints");
    println!("  the full catalogued list as an error hint.");
    println!();
    println!("Options:");
    println!("  --json      Machine-readable JSON output (goto, ls, scroll, click).");
    println!("  --help, -h  Show this help. Per-subcommand help: `agm ui scroll --help`.");
    println!();
    println!("Audit:");
    println!("  Every open/goto/scroll/click is recorded with actor=cli in the Task-01 UI");
    println!("  audit backend (<data-dir>/ui-audit/ui_audit.db). The trigger field holds");
    println!("  the action kind: programmatic, navigation, scroll, click.");
    println!();
    println!("Limitations (headless CLI):");
    println!("  A headless CLI cannot read the running UI's live DOM. `ls` without --current");
    println!("  reports the static route/element manifest; `ls --current`, `goto`, `scroll`,");
    println!("  and `click` drive through the management API on 127.0.0.1:8045");
    println!("  (POST /api/ui/navigate, GET /api/ui/element-tree, POST /api/ui/scroll,");
    println!("  POST /api/ui/click). When the running AGM build lacks an endpoint, the");
    println!("  command reports it explicitly instead of faking success.");
}
