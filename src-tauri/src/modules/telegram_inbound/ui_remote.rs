//! Telegram commands that open the Antigravity Manager window, photograph it,
//! and remember where the next plain-text prompt should go.
//!
//! This module also hosts the shared UI-action service surface
//! (`ui_service_goto` / `ui_service_ls` / `ui_service_scroll` plus the
//! canonical route registry): the `agm ui goto|ls|scroll` CLI subcommands
//! (Task-03/04, `src-tauri/src/bin/agm/ui_cmds.rs`) call the SAME functions,
//! so GUI, CLI, and Telegram drive the UI through identical routes.

use crate::error::AppError;
use crate::modules::audit::{
    record_ui_interaction, InteractionActor, InteractionTrigger, RecordInteractionInput,
};
use chrono::{Local, Utc};
use serde_json::{json, Value};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::injection::execute_prompt_injection;

const TARGET_TTL_SECS: u64 = 600;

struct PromptAim {
    instance: String,
    conversation: String,
    set_at: u64,
}

static PROMPT_AIM: Mutex<Option<PromptAim>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
enum UiVerb {
    Open,
    Shot,
    ShowAim,
    ClearAim,
    SetAim {
        instance: String,
        conversation: String,
    },
    AimedPrompt {
        text: String,
    },
    Goto {
        route: String,
    },
    Ls {
        filter: Option<String>,
    },
    Scroll {
        element_id: String,
    },
    UiUsage,
}

// ---------------------------------------------------------------------------
// Shared UI-action service surface (GUI / CLI / Telegram parity)
//
// The `agm ui goto|ls|scroll` CLI subcommands call the SAME `ui_service_*`
// functions as these Telegram handlers, so every entry point drives the UI
// through identical routes and the audit recorder sees the caller's actor.
// Navigation/scroll intents are appended to a pending-intent JSONL file that
// the GUI frontend's UI-command poller consumes and acknowledges:
//
//   {"v":1,"ts":<millis>,"actor":"telegram|cli|user","action":"goto|scroll",
//    "route":"/instances"|null,"element_id":"settings-supabase"|null}
// ---------------------------------------------------------------------------

/// One canonical UI route, mirroring the router table in `src/App.tsx`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct UiRouteInfo {
    pub route: &'static str,
    pub label: &'static str,
    pub blurb: &'static str,
}

/// Canonical route registry — single source of truth shared by GUI, CLI, and bot.
pub fn known_ui_routes() -> Vec<UiRouteInfo> {
    vec![
        UiRouteInfo {
            route: "dashboard",
            label: "Dashboard",
            blurb: "Overview cards and quick actions",
        },
        UiRouteInfo {
            route: "accounts",
            label: "Accounts",
            blurb: "Bound accounts, tiers and quota headroom (default landing page)",
        },
        UiRouteInfo {
            route: "instances",
            label: "Instances",
            blurb: "Instance profiles and live process states",
        },
        UiRouteInfo {
            route: "api-proxy",
            label: "API Proxy",
            blurb: "Local API gateway status (port 8045)",
        },
        UiRouteInfo {
            route: "monitor",
            label: "Monitor",
            blurb: "Live monitoring views",
        },
        UiRouteInfo {
            route: "token-stats",
            label: "Token Stats",
            blurb: "Token usage statistics",
        },
        UiRouteInfo {
            route: "user-token",
            label: "User Token",
            blurb: "User token management",
        },
        UiRouteInfo {
            route: "apikey-fun",
            label: "API Key Fun",
            blurb: "API key utilities",
        },
        UiRouteInfo {
            route: "security",
            label: "Security",
            blurb: "Security settings",
        },
        UiRouteInfo {
            route: "email",
            label: "Email",
            blurb: "Email vault and watcher settings",
        },
        UiRouteInfo {
            route: "supabase",
            label: "Supabase",
            blurb: "Supabase sync endpoints and node config",
        },
        UiRouteInfo {
            route: "audit",
            label: "Audit",
            blurb: "UI interaction audit recorder (start/stop, entries, export)",
        },
        UiRouteInfo {
            route: "settings",
            label: "Settings",
            blurb: "Application settings and subpages (e.g. settings/supabase)",
        },
        UiRouteInfo {
            route: "toolchain",
            label: "Toolchain",
            blurb: "Toolchain management",
        },
    ]
}

/// Canonicalize a user-supplied route (`instances`, `/Instances`,
/// `settings/supabase`) to `/instances`-style form. Returns `None` for
/// unknown top-level routes.
pub fn resolve_ui_route(input: &str) -> Option<String> {
    let trimmed = input.trim().trim_matches('/');
    if trimmed.is_empty() {
        return None;
    }
    let lower = trimmed.to_ascii_lowercase();
    let head = lower.split('/').next().unwrap_or("");
    if known_ui_routes().iter().any(|r| r.route == head) {
        Some(format!("/{lower}"))
    } else {
        None
    }
}

/// Pending-intent file consumed by the GUI frontend's UI-command poller.
/// Overridable via `AGM_UI_INTENTS_FILE` (CLI debugging, integration tests).
pub fn ui_intent_file_path() -> Result<PathBuf, AppError> {
    if let Ok(raw) = std::env::var("AGM_UI_INTENTS_FILE") {
        let raw = raw.trim();
        if !raw.is_empty() {
            return Ok(PathBuf::from(raw));
        }
    }
    let dir = crate::modules::account::get_data_dir()
        .map_err(|e| AppError::Config(format!("resolve app data dir for ui intents: {e}")))?;
    Ok(dir.join("ui_nav_intents.jsonl"))
}

/// Append one UI intent as a JSON line to `path`. Extracted so unit tests
/// can target a temp file without touching the real data dir or env vars.
pub fn queue_ui_intent_to(
    actor: InteractionActor,
    action: &str,
    route: Option<&str>,
    element_id: Option<&str>,
    path: &PathBuf,
) -> Result<(), AppError> {
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).map_err(|e| {
                AppError::Unknown(format!("create ui intent dir {}: {e}", parent.display()))
            })?;
        }
    }
    let intent = json!({
        "v": 1,
        "ts": SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0),
        "actor": actor.as_str(),
        "action": action,
        "route": route,
        "element_id": element_id,
    });
    let mut file = OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|e| AppError::Unknown(format!("open ui intent file {}: {e}", path.display())))?;
    writeln!(file, "{intent}")
        .map_err(|e| AppError::Unknown(format!("append ui intent to {}: {e}", path.display())))?;
    Ok(())
}

/// Best-effort audit entry for a bot/CLI-driven UI action. Failures are routed
/// into the error module (never silent); the caller's primary result is
/// unaffected because the UI intent is already queued.
fn audit_ui_action(
    actor: InteractionActor,
    method_name: &str,
    route: &str,
    element_id: Option<&str>,
    trigger: InteractionTrigger,
    params: Value,
    result: &str,
    message: Option<String>,
) {
    let input = RecordInteractionInput {
        actor,
        method_name: method_name.to_string(),
        route: route.to_string(),
        component: None,
        element_id: element_id.map(str::to_string),
        trigger,
        params: Some(params),
        result: result.to_string(),
        message,
    };
    // Justification: audit recording is observability, not the action itself —
    // the UI intent is already queued, and a recorder failure must not flip the
    // user-visible outcome; the error module keeps the failure visible.
    crate::error::record_ignored(
        record_ui_interaction(input).map(|_| ()),
        "record ui audit entry",
    );
}

fn unknown_route_reply(requested: &str) -> String {
    let mut out = format!(
        "⚠️ <b>Unknown UI route:</b> <code>{}</code>\n\n🧭 <b>Valid routes:</b>\n",
        super::clean_for_telegram_html(requested.trim(), 48)
    );
    for r in known_ui_routes() {
        out.push_str(&format!(
            "• <code>/ui goto {}</code> — {}\n",
            r.route, r.label
        ));
    }
    out
}

/// Shared service: queue a UI navigation intent. Same function the
/// `agm ui goto <route>` CLI calls. Unknown routes list the valid routes
/// (never fail silently); only IO/config failures are `Err`.
pub fn ui_service_goto(actor: InteractionActor, route_raw: &str) -> Result<String, AppError> {
    let path = ui_intent_file_path()?;
    ui_service_goto_to(actor, route_raw, &path)
}

/// POST to the local admin API for immediate UI navigation.
///
/// The JSONL intent queue (written by `ui_service_goto`) is the durable
/// record; this POST triggers the live Tauri-event navigation so the UI
/// responds immediately instead of waiting for a poll. Best-effort: any
/// failure is tracked and the queued intent remains.
pub(crate) async fn post_ui_navigate_event(route: &str) {
    let token: String = crate::modules::config::load_app_config()
        .ok()
        .and_then(|c| {
            c.proxy
                .admin_password
                .filter(|p| !p.is_empty())
                .or(Some(c.proxy.api_key))
        })
        .unwrap_or_default();
    // Justification: best-effort live navigation; the JSONL intent queue
    // is the durable fallback and the audit entry is already recorded.
    crate::error::record_ignored(
        async {
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(5))
                .build()
                .map_err(|e| e.to_string())?;
            let mut req = client
                .post("http://127.0.0.1:8045/api/ui/navigate")
                .json(&serde_json::json!({"route": route, "actor": "telegram"}));
            if !token.is_empty() {
                req = req
                    .header("Authorization", format!("Bearer {token}"))
                    .header("x-api-key", &token);
            }
            let resp = req.send().await.map_err(|e| e.to_string())?;
            if !resp.status().is_success() {
                return Err(format!("navigate endpoint returned {}", resp.status()));
            }
            Ok::<(), String>(())
        }
        .await,
        "post /api/ui/navigate from telegram /ui goto",
    );
}

/// `ui_service_goto` against an explicit intent file (unit-test seam).
pub fn ui_service_goto_to(
    actor: InteractionActor,
    route_raw: &str,
    intent_path: &PathBuf,
) -> Result<String, AppError> {
    let is_canonical_intent = ui_intent_file_path()
        .map(|canonical| &canonical == intent_path)
        .unwrap_or(false);

    match resolve_ui_route(route_raw) {
        Some(route) => {
            queue_ui_intent_to(actor, "goto", Some(&route), None, intent_path)?;
            if is_canonical_intent {
                audit_ui_action(
                    actor,
                    "ui_goto",
                    &route,
                    None,
                    InteractionTrigger::Navigation,
                    json!({ "route": route }),
                    "ok",
                    None,
                );
            }
            Ok(format!(
                "🧭 <b>UI navigation sent</b>\n• <b>Route:</b> <code>{}</code>\nThe manager window navigates there now (intent also queued).",
                escape_html(&route)
            ))
        }
        None => {
            if is_canonical_intent {
                let requested = route_raw.trim();
                let ui_target = if requested.is_empty() {
                    "/ui".to_string()
                } else {
                    format!("/ui?invalid={requested}")
                };
                audit_ui_action(
                    actor,
                    "ui_goto",
                    &ui_target,
                    None,
                    InteractionTrigger::Navigation,
                    json!({ "requested": requested }),
                    "error",
                    Some(format!("unknown ui route: {}", requested)),
                );
            }
            Ok(unknown_route_reply(route_raw))
        }
    }
}

/// Shared service: queue a scroll-to-element intent. Same function the
/// `agm ui scroll <element-id>` CLI calls.
pub fn ui_service_scroll(
    actor: InteractionActor,
    element_id_raw: &str,
) -> Result<String, AppError> {
    let path = ui_intent_file_path()?;
    ui_service_scroll_to(actor, element_id_raw, &path)
}

/// `ui_service_scroll` against an explicit intent file (unit-test seam).
pub fn ui_service_scroll_to(
    actor: InteractionActor,
    element_id_raw: &str,
    intent_path: &PathBuf,
) -> Result<String, AppError> {
    let is_canonical_intent = ui_intent_file_path()
        .map(|canonical| &canonical == intent_path)
        .unwrap_or(false);

    let element_id = element_id_raw.trim();
    if element_id.is_empty() || element_id.len() > 128 || element_id.chars().any(|c| c.is_control())
    {
        if is_canonical_intent {
            audit_ui_action(
                actor,
                "ui_scroll",
                "/ui?invalid_element=true",
                None,
                InteractionTrigger::Scroll,
                json!({ "requested": element_id_raw }),
                "error",
                Some("invalid element id".to_string()),
            );
        }
        return Ok("⚠️ <b>Invalid element id.</b> Send <code>/ui scroll &lt;element-id&gt;</code> with a plain element id (max 128 chars).".to_string());
    }
    queue_ui_intent_to(actor, "scroll", None, Some(element_id), intent_path)?;
    if is_canonical_intent {
        audit_ui_action(
            actor,
            "ui_scroll",
            "/ui",
            Some(element_id),
            InteractionTrigger::Scroll,
            json!({ "element_id": element_id }),
            "ok",
            None,
        );
    }
    Ok(format!(
        "🖱️ <b>UI scroll queued</b>\n• <b>Element:</b> <code>{}</code>\nThe manager window scrolls it into view and focuses it on its next poll.",
        escape_html(element_id)
    ))
}

/// Shared service: render the UI route listing ("ls on the UI") as Telegram
/// HTML. Pure — no side effects, no audit (read-only query).
pub fn ui_service_ls(filter_raw: Option<&str>) -> String {
    let filter = filter_raw
        .map(|f| f.trim().to_ascii_lowercase())
        .filter(|f| !f.is_empty());
    let routes: Vec<UiRouteInfo> = known_ui_routes()
        .into_iter()
        .filter(|r| match &filter {
            Some(f) => r.route.contains(f) || r.label.to_ascii_lowercase().contains(f),
            None => true,
        })
        .collect();
    let mut out = String::from("🧭 <b>UI routes</b>\n");
    if routes.is_empty() {
        out.push_str(&format!(
            "No routes match <code>{}</code>. Send <code>/ui ls</code> for the full list.",
            super::clean_for_telegram_html(filter.as_deref().unwrap_or(""), 32)
        ));
        return out;
    }
    for r in &routes {
        out.push_str(&format!(
            "• <code>{}</code> — <b>{}</b>\n  <i>{}</i>\n",
            r.route, r.label, r.blurb
        ));
    }
    out.push_str(
        "\n💡 <code>/ui goto &lt;route&gt;</code> navigates · <code>/ui scroll &lt;id&gt;</code> focuses an element.",
    );
    out
}

/// Usage text for `/ui` subcommands.
pub fn ui_usage_text() -> String {
    "🖥️ <b>UI remote control</b>\n\
    • <code>/ui</code> — Open the manager window here\n\
    • <code>/ui goto &lt;route&gt;</code> — Navigate the UI (e.g. <code>/ui goto instances</code>)\n\
    • <code>/ui ls</code> — List UI routes (<code>/ui ls &lt;filter&gt;</code> narrows)\n\
    • <code>/ui scroll &lt;element-id&gt;</code> — Scroll an element into view and focus it\n\n\
    Unknown routes list the valid ones — nothing fails silently."
        .to_string()
}

pub struct LocalUiAction {
    pub reply: String,
    pub markup: Option<Value>,
    pub png: Option<Vec<u8>>,
    pub caption: String,
}

/// Copy-paste affordances for the essentials: every button carries the command
/// template as native `copy_text`, so the user taps, copies, edits, and sends.
pub fn help_copy_markup() -> Value {
    json!({
        "inline_keyboard": [
            [
                { "text": "💰 Account Balances", "callback_data": "cmd:/balance" },
                { "text": "🚀 Prompt Wizard (/send)", "callback_data": "cmd:/send" }
            ],
            [
                { "text": "📥 Accounts CSV", "callback_data": "cmd:/export accounts csv" },
                { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" }
            ],
            [
                { "text": "🖥️ SSH Nodes", "callback_data": "cmd:/nodes" },
                { "text": "🔄 Live Status", "callback_data": "cmd:/status" }
            ],
            [
                { "text": "📋 /prompt", "copy_text": { "text": "/prompt default Write unit tests" } },
                { "text": "📋 /target", "copy_text": { "text": "/target W3 default" } },
                { "text": "📋 /balance", "copy_text": { "text": "/balance" } }
            ],
            [
                { "text": "📋 /screenshot", "copy_text": { "text": "/screenshot " } },
                { "text": "📋 /shot", "copy_text": { "text": "/shot" } },
                { "text": "📋 /ui", "copy_text": { "text": "/ui" } }
            ]
        ]
    })
}

pub fn window_command_section() -> &'static str {
    "\n\
    🎨 <b>Window, screenshot, and where the next prompt goes</b>\n\
    🔵 <b>This machine.</b> <code>/ui</code> opens Antigravity Manager when it is closed. <code>/shot</code> and <code>/screenshot</code> send the desktop mosaic, AGM Instances, AGM Accounts, and each running Antigravity IDE window. Each machine that runs the manager answers for its own screen.\n\
    <pre>/ui\n/shot\n/target default C001</pre>\n\
    🟢 Send <code>/target &lt;instance&gt; &lt;C001&gt;</code> first — your next plain message becomes the prompt for that conversation. <code>/target clear</code> cancels. For a guided picker with delivery proof, use <code>/send</code>.\n\
    🟡 <code>/open &lt;machine&gt;</code> opens AGM on a fleet machine; <code>/screenshot &lt;machine&gt;</code> photographs it. Bare <code>/open</code> / <code>/screenshot</code> stay local.\n\
    🟣 Voice notes stay audio. Telegram does not turn them into text. Type the prompt after <code>/target</code> or <code>/send</code>.\n\
    🟠 <b>Drive the UI remotely.</b> <code>/ui goto &lt;route&gt;</code> navigates the manager window (e.g. <code>/ui goto instances</code>); <code>/ui ls</code> lists every route; <code>/ui scroll &lt;element-id&gt;</code> scrolls an element into view and focuses it. Unknown routes list the valid ones — nothing fails silently.\n"
}

/// Handle `/ui`, `/shot`, `/target`, and a follow-up prompt. Other text returns None.
pub async fn prepare_local_ui_action(text: &str) -> Option<LocalUiAction> {
    let verb = classify_ui_verb(text)?;
    Some(run_ui_verb(verb).await)
}

fn classify_ui_verb(text: &str) -> Option<UiVerb> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with('/') {
        return aimed_prompt_verb(trimmed);
    }
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((cmd, rest)) => (cmd, rest.trim()),
        None => (trimmed, ""),
    };
    let cmd = cmd_raw
        .trim_start_matches('/')
        .split('@')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match cmd.as_str() {
        "ui" => Some(parse_ui_subcommand(rest)),
        "open" | "open-ui" | "launch-ui" => open_target_verb(rest),
        "shot" => Some(UiVerb::Shot),
        "screenshot" | "screen" => shot_target_verb(rest),
        "where" | "aim" => Some(parse_target_rest(rest)),
        "target" => Some(parse_target_rest(rest)),
        "agm" => match rest.split_once(char::is_whitespace) {
            Some((head, tail)) => match head.to_ascii_lowercase().as_str() {
                "ui" => Some(parse_ui_subcommand(tail.trim())),
                "open" | "open-ui" => Some(UiVerb::Open),
                "shot" | "screenshot" | "screen" => Some(UiVerb::Shot),
                "target" | "where" | "aim" => Some(parse_target_rest(tail.trim())),
                _ => None,
            },
            None => match rest.to_ascii_lowercase().as_str() {
                "ui" => Some(UiVerb::Open),
                "open" | "open-ui" => Some(UiVerb::Open),
                "shot" | "screenshot" | "screen" => Some(UiVerb::Shot),
                "target" | "where" | "aim" => Some(UiVerb::ShowAim),
                _ => None,
            },
        },
        _ => None,
    }
}

/// Parse the `/ui` subcommand line. Bare `/ui` keeps the historical behavior
/// (open the manager window); anything unrecognized prints usage — never
/// fails silently.
fn parse_ui_subcommand(rest: &str) -> UiVerb {
    let rest = rest.trim();
    if rest.is_empty() {
        return UiVerb::Open;
    }
    let (sub, args) = match rest.split_once(char::is_whitespace) {
        Some((s, a)) => (s.to_ascii_lowercase(), a.trim()),
        None => (rest.to_ascii_lowercase(), ""),
    };
    match sub.as_str() {
        "open" => UiVerb::Open,
        "goto" | "go" | "nav" | "navigate" => {
            if args.is_empty() {
                UiVerb::UiUsage
            } else {
                UiVerb::Goto {
                    route: args.to_string(),
                }
            }
        }
        "ls" | "list" | "routes" => UiVerb::Ls {
            filter: if args.is_empty() {
                None
            } else {
                Some(args.to_string())
            },
        },
        "scroll" | "focus" => {
            if args.is_empty() {
                UiVerb::UiUsage
            } else {
                UiVerb::Scroll {
                    element_id: args.to_string(),
                }
            }
        }
        "help" | "-h" | "--help" => UiVerb::UiUsage,
        _ => UiVerb::UiUsage,
    }
}

/// Machine-targeted `/open` / `/screenshot` belong to the command router's
/// `open_shot` handlers: when the argument names a NON-local fleet machine,
/// decline here so `processing.rs` routes it to the machine-targeted handler.
/// Bare commands, local aliases, and unknown tokens keep the existing local
/// behavior exactly as before.
fn open_target_verb(rest: &str) -> Option<UiVerb> {
    match super::open_shot::resolve_machine_target(rest) {
        Ok(super::open_shot::OpenShotTarget::Remote(_)) => None,
        _ => Some(UiVerb::Open),
    }
}

fn shot_target_verb(rest: &str) -> Option<UiVerb> {
    match super::open_shot::resolve_machine_target(rest) {
        Ok(super::open_shot::OpenShotTarget::Local) if rest.trim().is_empty() => Some(UiVerb::Shot),
        _ => None,
    }
}

fn parse_target_rest(rest: &str) -> UiVerb {
    let rest = rest.trim();
    if rest.is_empty() || rest.eq_ignore_ascii_case("show") || rest.eq_ignore_ascii_case("ls") {
        return UiVerb::ShowAim;
    }
    if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("reset") {
        return UiVerb::ClearAim;
    }
    let parts: Vec<&str> = rest.split_whitespace().collect();
    match parts.as_slice() {
        [single] => UiVerb::SetAim {
            instance: (*single).to_string(),
            conversation: "C001".to_string(),
        },
        [first, second] => UiVerb::SetAim {
            instance: (*first).to_string(),
            conversation: (*second).to_string(),
        },
        [machine, instance, conversation, ..] => UiVerb::SetAim {
            instance: format!("{}:{}", machine, instance),
            conversation: (*conversation).to_string(),
        },
        _ => UiVerb::ShowAim,
    }
}

fn aimed_prompt_verb(text: &str) -> Option<UiVerb> {
    let aim = PROMPT_AIM
        .lock()
        .ok()?
        .as_ref()
        .map(|aim| (aim.instance.clone(), aim.conversation.clone(), aim.set_at))?;
    if now_secs().saturating_sub(aim.2) > TARGET_TTL_SECS {
        if let Ok(mut guard) = PROMPT_AIM.lock() {
            *guard = None;
        }
        return None;
    }
    Some(UiVerb::AimedPrompt {
        text: text.to_string(),
    })
}

async fn run_ui_verb(verb: UiVerb) -> LocalUiAction {
    match verb {
        UiVerb::Open => {
            let note = ensure_manager_window();
            LocalUiAction {
                reply: format!("🖥️ <b>Antigravity Manager</b>\n{note}"),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::Shot => {
            let reply = super::open_shot::execute_screenshot_command("").await;
            LocalUiAction {
                reply,
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::ShowAim => LocalUiAction {
            reply: aim_status_html(),
            markup: Some(help_copy_markup()),
            png: None,
            caption: String::new(),
        },
        UiVerb::ClearAim => {
            if let Ok(mut guard) = PROMPT_AIM.lock() {
                *guard = None;
            }
            LocalUiAction {
                reply: "Target cleared — your next message won't be sent as a prompt.".to_string(),
                markup: None,
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::SetAim {
            instance,
            conversation,
        } => {
            if let Ok(mut guard) = PROMPT_AIM.lock() {
                *guard = Some(PromptAim {
                    instance: instance.clone(),
                    conversation: conversation.clone(),
                    set_at: now_secs(),
                });
            }
            LocalUiAction {
                reply: format!(
                    "🎯 Your next message will go to {} · {}. This stays active for 10 minutes. /target clear cancels it.",
                    escape_html(&instance),
                    escape_html(&conversation)
                ),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::AimedPrompt { text } => {
            let (instance, conversation) = PROMPT_AIM
                .lock()
                .ok()
                .and_then(|g| {
                    g.as_ref()
                        .map(|aim| (aim.instance.clone(), aim.conversation.clone()))
                })
                .unwrap_or_else(|| ("default".to_string(), "C001".to_string()));
            let inject_cmd = if instance.contains(':') {
                let (m_part, i_part) = instance.split_once(':').unwrap_or(("local", "default"));
                format!("{conversation} --node {m_part} --instance {i_part} {text}")
            } else if !(conversation.to_uppercase().starts_with('C')
                || conversation.to_uppercase().starts_with('P')
                || conversation.to_uppercase().starts_with("GM:"))
                && crate::modules::instance::resolve_instance_id(&conversation).is_ok()
            {
                // `/target <machine> <instance>` where `instance` held machine and `conversation` held instance name
                format!("default --node {instance} --instance {conversation} {text}")
            } else {
                format!("{conversation} --instance {instance} {text}")
            };
            let injected = execute_prompt_injection(&inject_cmd).await;
            let (png, note) = match capture_manager_png() {
                Ok(bytes) => (
                    Some(bytes),
                    "A screenshot of the manager window is attached above (the conversation itself wasn't clicked).".to_string(),
                ),
                Err(err) => (
                    None,
                    format!("Screenshot unavailable: {}", escape_html(&err)),
                ),
            };
            let now_utc = Utc::now();
            let utc_str = now_utc.to_rfc3339();
            let local_str = Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string();
            let (_, detected_images) =
                crate::modules::repo_db::extract_image_payload_or_path(&text);
            let media_count = detected_images.len();
            let (target_node, target_instance) = if instance.contains(':') {
                let (m_part, i_part) = instance.split_once(':').unwrap_or(("local", "default"));
                (m_part.to_string(), i_part.to_string())
            } else if !(conversation.to_uppercase().starts_with('C')
                || conversation.to_uppercase().starts_with('P')
                || conversation.to_uppercase().starts_with("GM:"))
                && crate::modules::instance::resolve_instance_id(&conversation).is_ok()
            {
                (instance.clone(), conversation.clone())
            } else {
                ("local".to_string(), instance.clone())
            };

            LocalUiAction {
                reply: format!(
                    "✅ <b>Prompt Dispatched & Acknowledged</b>\n\
                    • <b>Dispatched At:</b> <code>{}</code> ({})\n\
                    • <b>Target Node:</b> <code>{}</code>\n\
                    • <b>Instance:</b> <code>{}</code>\n\
                    • <b>Conversation:</b> <code>{}</code>\n\
                    • <b>Attached Media:</b> {} image(s)\n\
                    {}\n\n{}",
                    utc_str,
                    local_str,
                    escape_html(&target_node),
                    escape_html(&target_instance),
                    escape_html(&conversation),
                    media_count,
                    note,
                    injected
                ),
                markup: None,
                caption: format!("AGM after prompt to {} on {}", conversation, instance),
                png,
            }
        }
        UiVerb::Goto { route } => {
            let note = ensure_manager_window();
            let body = match ui_service_goto(InteractionActor::Telegram, &route) {
                Ok(text) => {
                    // Trigger immediate navigation via the local admin API
                    // (the queued intent is the durable fallback).
                    if let Some(resolved) = resolve_ui_route(&route) {
                        post_ui_navigate_event(&resolved).await;
                    }
                    text
                }
                Err(err) => format!(
                    "❌ <b>UI navigation failed:</b> <code>{}</code>",
                    escape_html(&err.to_string())
                ),
            };
            LocalUiAction {
                reply: format!("{body}\n{note}"),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::Ls { filter } => LocalUiAction {
            reply: ui_service_ls(filter.as_deref()),
            markup: Some(help_copy_markup()),
            png: None,
            caption: String::new(),
        },
        UiVerb::Scroll { element_id } => {
            let note = ensure_manager_window();
            let body = match ui_service_scroll(InteractionActor::Telegram, &element_id) {
                Ok(text) => text,
                Err(err) => format!(
                    "❌ <b>UI scroll failed:</b> <code>{}</code>",
                    escape_html(&err.to_string())
                ),
            };
            LocalUiAction {
                reply: format!("{body}\n{note}"),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::UiUsage => LocalUiAction {
            reply: ui_usage_text(),
            markup: Some(help_copy_markup()),
            png: None,
            caption: String::new(),
        },
    }
}

fn aim_status_html() -> String {
    match PROMPT_AIM.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(aim) => format!(
                "🎯 Current target: {} · {}. Your next plain message will be sent there as a prompt.",
                escape_html(&aim.instance),
                escape_html(&aim.conversation)
            ),
            None => "No target set. Send /target default C001 first, then your prompt.".to_string(),
        },
        Err(_) => "Target lock failed. Send <code>/target default C001</code> again.".to_string(),
    }
}

fn ensure_manager_window() -> String {
    if manager_process_running() {
        return format!(
            "Already open on <code>{}</code>. Use <code>/shot</code> to photograph this screen.",
            clean_machine()
        );
    }
    crate::modules::delegate_updater::open_ui(&[]);
    format!(
        "Launch requested on <code>{}</code>. If the window is still starting, send <code>/shot</code> again in a few seconds.",
        clean_machine()
    )
}

pub fn capture_manager_png_cli() -> Result<usize, String> {
    capture_manager_png().map(|bytes| bytes.len())
}

/// Capture the manager window and upload it to the saved Telegram chat.
pub async fn send_window_shot_to_saved_chat() -> Result<usize, String> {
    let png = capture_manager_png()?;
    let len = png.len();
    let cfg = super::config::load_config().map_err(|e| e.to_string())?;
    let chat_id = cfg
        .allowed_chat_id
        .ok_or_else(|| "telegram chat id is not configured".to_string())?;
    if cfg.bot_token.trim().is_empty() {
        return Err("telegram bot token is not configured".to_string());
    }
    super::media::send_telegram_photo(
        &cfg.bot_token,
        chat_id,
        &png,
        "AGM window on this machine",
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(len)
}

fn capture_manager_png() -> Result<Vec<u8>, String> {
    let opened = if manager_process_running() {
        false
    } else {
        crate::modules::delegate_updater::open_ui(&[]);
        true
    };
    if opened {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    let mut last_err = String::from("manager window was not found");
    for _ in 0..4 {
        match capture_once() {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_err = err;
                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        }
    }
    Err(last_err)
}

fn capture_once() -> Result<Vec<u8>, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("window screenshot is implemented for Windows".to_string());
    }
    #[cfg(target_os = "windows")]
    {
        let dir = std::env::temp_dir().join("agm-ui-shot");
        std::fs::create_dir_all(&dir).map_err(|e| format!("create shot folder: {e}"))?;
        let stamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis())
            .unwrap_or(0);
        let script = dir.join(format!("capture-{stamp}.ps1"));
        let png = dir.join(format!("window-{stamp}.png"));
        {
            let mut file =
                std::fs::File::create(&script).map_err(|e| format!("write shot script: {e}"))?;
            file.write_all(CAPTURE_PS1.as_bytes())
                .map_err(|e| format!("write shot script bytes: {e}"))?;
        }
        let mut cmd = Command::new("powershell.exe");
        cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
            "-OutPath",
            &png.to_string_lossy(),
        ]);
        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            cmd.creation_flags(0x08000000);
        }
        let output = cmd
            .output()
            .map_err(|e| format!("start screenshot helper: {e}"))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            let stdout = String::from_utf8_lossy(&output.stdout);
            return Err(format!(
                "screenshot helper exited {}: {} {}",
                output.status,
                stderr.trim(),
                stdout.trim()
            ));
        }
        let bytes = std::fs::read(&png).map_err(|e| format!("read screenshot: {e}"))?;
        if bytes.len() < 32 {
            return Err("screenshot file was empty".to_string());
        }
        // Justification: the PNG bytes are already in memory; deleting the temp files is cleanup.
        crate::error::record_ignored(std::fs::remove_file(&png), "remove temp screenshot");
        crate::error::record_ignored(std::fs::remove_file(&script), "remove temp shot script");
        Ok(bytes)
    }
}

fn manager_process_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    sys.processes().values().any(|proc| {
        let name = proc.name().to_string_lossy().to_ascii_lowercase();
        name.contains("agm-alim")
            || name.contains("agm-ui")
            || name.contains("antigravity-tools")
            || name == "agm"
            || name == "agm.exe"
            || name.contains("antigravity_manager")
    })
}

fn clean_machine() -> String {
    escape_html(&crate::modules::email_watcher::detect_machine_name())
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

const CAPTURE_PS1: &str = r#"
param([Parameter(Mandatory = $true)][string]$OutPath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmWinShot {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
[void][AgmWinShot]::SetProcessDPIAware()
$procNames = @('agm-alim', 'agm-ui', 'antigravity-tools', 'agm', 'antigravity_manager')
$proc = Get-Process -Name $procNames -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
$h = [IntPtr]::Zero
if ($proc) {
  $h = $proc.MainWindowHandle
  [void][AgmWinShot]::ShowWindow($h, 9)
  [void][AgmWinShot]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 350
}
if ($h -eq [IntPtr]::Zero) {
  $h = [AgmWinShot]::GetForegroundWindow()
}
$rect = New-Object AgmWinShot+RECT
$captured = $false
if ($h -ne [IntPtr]::Zero -and [AgmWinShot]::GetWindowRect($h, [ref]$rect)) {
  $w = $rect.R - $rect.L
  $ht = $rect.B - $rect.T
  if ($w -ge 40 -and $ht -ge 40) {
    $bmp = New-Object System.Drawing.Bitmap $w, $ht
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
    $captured = $true
  }
}
if (-not $captured) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $w = $bounds.Width
  $ht = $bounds.Height
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
}
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Output "OK ${w}x${ht}"
"#;

#[cfg(test)]
mod tests {
    use super::classify_ui_verb;
    use super::parse_ui_subcommand;
    use super::queue_ui_intent_to;
    use super::resolve_ui_route;
    use super::ui_service_goto_to;
    use super::ui_service_ls;
    use super::ui_service_scroll_to;
    use super::UiVerb;
    use crate::modules::audit::InteractionActor;

    fn temp_intent_path(tag: &str) -> std::path::PathBuf {
        let mut p = std::env::temp_dir();
        p.push(format!(
            "agm-ui-intent-test-{}-{}.jsonl",
            std::process::id(),
            tag
        ));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn classifies_window_commands_and_aim() {
        assert_eq!(classify_ui_verb("/ui"), Some(UiVerb::Open));
        assert_eq!(classify_ui_verb("/open"), Some(UiVerb::Open));
        assert_eq!(classify_ui_verb("/screenshot"), Some(UiVerb::Shot));
        assert_eq!(classify_ui_verb("/shot@MyBot"), Some(UiVerb::Shot));
        assert_eq!(classify_ui_verb("/agm ui"), Some(UiVerb::Open));
        // Unknown machine tokens keep the existing local behavior untouched.
        assert_eq!(
            classify_ui_verb("/open definitely-not-a-machine-xyz"),
            Some(UiVerb::Open)
        );
        assert_eq!(
            classify_ui_verb("/screenshot definitely-not-a-machine-xyz"),
            Some(UiVerb::Shot)
        );
        assert_eq!(
            classify_ui_verb("/target default C001"),
            Some(UiVerb::SetAim {
                instance: "default".to_string(),
                conversation: "C001".to_string(),
            })
        );
        assert_eq!(classify_ui_verb("/target clear"), Some(UiVerb::ClearAim));
        assert_eq!(classify_ui_verb("/help"), None);
        assert_eq!(classify_ui_verb("/send"), None);
        assert_eq!(classify_ui_verb("just a sentence"), None);
    }

    #[test]
    fn classifies_ui_subcommands() {
        // Bare `/ui` keeps the historical open-window behavior.
        assert_eq!(classify_ui_verb("/ui"), Some(UiVerb::Open));
        assert_eq!(classify_ui_verb("/ui open"), Some(UiVerb::Open));
        assert_eq!(
            classify_ui_verb("/ui goto instances"),
            Some(UiVerb::Goto {
                route: "instances".to_string()
            })
        );
        assert_eq!(
            classify_ui_verb("/ui ls"),
            Some(UiVerb::Ls { filter: None })
        );
        assert_eq!(
            classify_ui_verb("/ui ls inst"),
            Some(UiVerb::Ls {
                filter: Some("inst".to_string())
            })
        );
        assert_eq!(
            classify_ui_verb("/ui scroll settings-supabase"),
            Some(UiVerb::Scroll {
                element_id: "settings-supabase".to_string()
            })
        );
        // Missing args and unknown subcommands print usage, never fail silently.
        assert_eq!(classify_ui_verb("/ui goto"), Some(UiVerb::UiUsage));
        assert_eq!(classify_ui_verb("/ui scroll"), Some(UiVerb::UiUsage));
        assert_eq!(classify_ui_verb("/ui frobnicate"), Some(UiVerb::UiUsage));
        assert_eq!(classify_ui_verb("/ui --help"), Some(UiVerb::UiUsage));
        // `/agm ui ...` aliases share the same subcommands.
        assert_eq!(classify_ui_verb("/agm ui"), Some(UiVerb::Open));
        assert_eq!(
            classify_ui_verb("/agm ui goto accounts"),
            Some(UiVerb::Goto {
                route: "accounts".to_string()
            })
        );
        assert_eq!(parse_ui_subcommand(""), UiVerb::Open);
    }

    #[test]
    fn resolves_ui_routes() {
        assert_eq!(
            resolve_ui_route("instances"),
            Some("/instances".to_string())
        );
        assert_eq!(
            resolve_ui_route("/Instances"),
            Some("/instances".to_string())
        );
        assert_eq!(
            resolve_ui_route("settings/supabase"),
            Some("/settings/supabase".to_string())
        );
        assert_eq!(resolve_ui_route("audit"), Some("/audit".to_string()));
        assert_eq!(resolve_ui_route("nope"), None);
        assert_eq!(resolve_ui_route(""), None);
        assert_eq!(resolve_ui_route("/"), None);
    }

    #[test]
    fn ui_ls_lists_routes_and_filters() {
        let all = ui_service_ls(None);
        assert!(all.contains("<code>instances</code>"));
        assert!(all.contains("<code>audit</code>"));
        assert!(all.contains("/ui goto"));
        let filtered = ui_service_ls(Some("token"));
        assert!(filtered.contains("<code>token-stats</code>"));
        assert!(!filtered.contains("<code>instances</code>"));
        let empty = ui_service_ls(Some("zzz-no-such-route"));
        assert!(empty.contains("No routes match"));
        assert!(empty.contains("/ui ls"));
    }

    #[test]
    fn goto_queues_intent_jsonl() {
        let path = temp_intent_path("goto");
        let reply = ui_service_goto_to(InteractionActor::Telegram, "instances", &path).unwrap();
        assert!(reply.contains("/instances"));
        let raw = std::fs::read_to_string(&path).unwrap();
        let intent: serde_json::Value = serde_json::from_str(raw.trim()).unwrap();
        assert_eq!(intent["v"], 1);
        assert_eq!(intent["actor"], "telegram");
        assert_eq!(intent["action"], "goto");
        assert_eq!(intent["route"], "/instances");
        assert!(intent["element_id"].is_null());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn goto_unknown_route_lists_valid_and_queues_nothing() {
        let path = temp_intent_path("goto-unknown");
        let reply = ui_service_goto_to(InteractionActor::Telegram, "nope", &path).unwrap();
        assert!(reply.contains("Unknown UI route"));
        assert!(reply.contains("/ui goto instances"));
        assert!(!path.exists());
    }

    #[test]
    fn scroll_queues_intent_and_validates_element_id() {
        let path = temp_intent_path("scroll");
        let reply =
            ui_service_scroll_to(InteractionActor::Telegram, "settings-supabase", &path).unwrap();
        assert!(reply.contains("settings-supabase"));
        let raw = std::fs::read_to_string(&path).unwrap();
        let intent: serde_json::Value = serde_json::from_str(raw.trim()).unwrap();
        assert_eq!(intent["actor"], "telegram");
        assert_eq!(intent["action"], "scroll");
        assert_eq!(intent["element_id"], "settings-supabase");
        assert!(intent["route"].is_null());

        // Invalid element ids are rejected with guidance, never queued.
        let bad_path = temp_intent_path("scroll-bad");
        let bad = ui_service_scroll_to(InteractionActor::Telegram, "   ", &bad_path).unwrap();
        assert!(bad.contains("Invalid element id"));
        assert!(!bad_path.exists());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn queue_ui_intent_appends_jsonl() {
        let path = temp_intent_path("queue");
        queue_ui_intent_to(InteractionActor::Cli, "goto", Some("/audit"), None, &path).unwrap();
        queue_ui_intent_to(InteractionActor::Cli, "scroll", None, Some("x"), &path).unwrap();
        let raw = std::fs::read_to_string(&path).unwrap();
        let lines: Vec<&str> = raw.lines().collect();
        assert_eq!(lines.len(), 2);
        assert!(lines[0].contains("\"actor\":\"cli\""));
        assert!(lines[1].contains("\"element_id\":\"x\""));
        let _ = std::fs::remove_file(&path);
    }
}
