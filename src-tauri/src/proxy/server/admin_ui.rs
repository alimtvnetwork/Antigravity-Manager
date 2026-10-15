//! UI remote-control REST admin handlers.
//!
//! Part of `auditing-and-cli-ui-integration`: lets the `agm ui` CLI (and the
//! Telegram `/ui` commands) drive the running AGM UI — navigate to any
//! route, list the element tree of a view. Every action is recorded in the
//! UI interaction audit trail (`crate::modules::audit`).
//!
//! The navigate handler validates the route and emits a Tauri event
//! (`agm://ui-navigate`); the React frontend listens for it (see
//! `src/hooks/useUiRemoteControl.ts`) and performs the actual router
//! navigation. This keeps one code path: CLI/GUI/Telegram all converge on
//! the same frontend navigation.

use super::app_state::AppState;
use super::dto::ErrorResponse;
use axum::{
    extract::{Query, State},
    http::StatusCode,
    response::{IntoResponse, Json},
    routing::{get, post},
    Router,
};
use serde::Deserialize;
use std::collections::HashMap;
use tauri::Emitter;

// ── Route registry ──────────────────────────────────────────────────────────
// Mirrors `src-tauri/src/bin/agm/ui_cmds.rs` ROUTES (grounded in
// `src/App.tsx` createBrowserRouter + `src/pages/Settings.tsx` tabs).
// Keep the two lists in sync when routes are added.
pub(crate) const UI_ROUTES: &[&str] = &[
    "dashboard",
    "accounts",
    "instances",
    "api-proxy",
    "monitor",
    "token-stats",
    "user-token",
    "apikey-fun",
    "security",
    "email",
    "supabase",
    "audit",
    "settings",
    "settings/general",
    "settings/supabase",
    "settings/telegram",
    "settings/proxy",
    "settings/auto-switcher",
    "settings/email",
    "settings/advanced",
    "settings/about",
];

pub(crate) fn ui_route_table() -> Router<AppState> {
    Router::new()
        .route("/ui/navigate", post(navigate_ui))
        .route("/ui/element-tree", get(element_tree))
        .route("/ui/scroll", post(scroll_ui))
        .route("/ui/click", post(click_ui))
        .route("/ui/record/start", post(start_record_ui))
        .route("/ui/record/stop", post(stop_record_ui))
        .route("/ui/record/status", get(status_record_ui))
        .route("/ui/record/list", get(list_record_ui))
        .route("/ui/record/clear", post(clear_record_ui))
        .route("/ui/record/export", post(export_record_ui))
}

#[derive(Deserialize)]
pub(crate) struct NavigatePayload {
    route: String,
    actor: Option<String>,
}

fn parse_actor(raw: Option<String>) -> crate::modules::audit::InteractionActor {
    use crate::modules::audit::InteractionActor as A;
    match raw.as_deref().unwrap_or("cli") {
        "telegram" => A::Telegram,
        "user" => A::User,
        "system" => A::System,
        _ => A::Cli,
    }
}

/// POST /ui/navigate — validate the route, emit the Tauri navigation event,
/// and record the audit entry. Never fails silently: unknown routes return
/// 400 with the valid route list.
pub(crate) async fn navigate_ui(
    State(state): State<AppState>,
    Json(payload): Json<NavigatePayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let route = payload.route.trim().trim_start_matches('/').to_lowercase();
    if !UI_ROUTES.contains(&route.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "unknown route '/{route}'. valid routes: {}",
                    UI_ROUTES.join(", ")
                ),
            }),
        ));
    }
    let actor = parse_actor(payload.actor);

    // Notify the running frontend via Tauri event (best-effort: the audit
    // entry below is the durable record).
    if let crate::modules::integration::SystemManager::Desktop(handle) = &state.integration {
        // Justification: best-effort UI notification; the CLI already
        // validated the route and the audit entry is recorded regardless.
        crate::error::record_ignored(
            handle.emit(
                "agm://ui-navigate",
                serde_json::json!({"route": route, "actor": actor.as_str()}),
            ),
            "emit agm://ui-navigate",
        );
    }

    // Record the navigation in the audit trail (logged no-op when the
    // recorder is stopped — never an error to the caller).
    // Justification: audit is observability, not the action itself; a
    // failed audit write must not fail the navigation.
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor,
            method_name: "ui navigate".to_string(),
            route: route.clone(),
            component: None,
            element_id: None,
            trigger: crate::modules::audit::InteractionTrigger::Navigation,
            params: Some(serde_json::json!({"route": route})),
            result: "ok".to_string(),
            message: None,
        }),
        "audit ui navigate",
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "route": format!("/{route}"),
        "actor": actor.as_str(),
    })))
}

#[derive(Deserialize)]
pub(crate) struct ElementActionPayload {
    element_id: String,
    route: Option<String>,
    actor: Option<String>,
}

/// Shared helper: validate an element action, emit the Tauri event, audit it.
async fn handle_element_action(
    state: &AppState,
    payload: ElementActionPayload,
    event_name: &'static str,
    method_name: &'static str,
    trigger: crate::modules::audit::InteractionTrigger,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let element_id = payload
        .element_id
        .trim()
        .trim_start_matches('#')
        .to_string();
    if element_id.is_empty() || element_id.len() > 128 {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: "element_id must be 1..=128 chars".to_string(),
            }),
        ));
    }
    let route = payload
        .route
        .unwrap_or_else(|| "dashboard".to_string())
        .trim()
        .trim_start_matches('/')
        .to_lowercase();
    if !UI_ROUTES.contains(&route.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "unknown route '/{route}'. valid routes: {}",
                    UI_ROUTES.join(", ")
                ),
            }),
        ));
    }
    let actor = parse_actor(payload.actor);

    if let crate::modules::integration::SystemManager::Desktop(handle) = &state.integration {
        // Justification: best-effort UI notification; the audit entry below
        // is the durable record.
        crate::error::record_ignored(
            handle.emit(
                event_name,
                serde_json::json!({
                    "element_id": element_id,
                    "route": route,
                    "actor": actor.as_str(),
                }),
            ),
            "emit ui element action",
        );
    }

    // Justification: audit is observability, not the action itself.
    crate::error::record_ignored(
        crate::modules::audit::record_interaction(crate::modules::audit::RecordInteractionInput {
            actor,
            method_name: method_name.to_string(),
            route: route.clone(),
            component: None,
            element_id: Some(element_id.clone()),
            trigger,
            params: Some(serde_json::json!({"element_id": element_id, "route": route})),
            result: "ok".to_string(),
            message: None,
        }),
        "audit ui element action",
    );

    Ok(Json(serde_json::json!({
        "success": true,
        "element_id": element_id,
        "route": format!("/{route}"),
        "actor": actor.as_str(),
    })))
}

/// POST /ui/scroll — scroll the view until the element is visible, then focus it.
pub(crate) async fn scroll_ui(
    State(state): State<AppState>,
    Json(payload): Json<ElementActionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    handle_element_action(
        &state,
        payload,
        "agm://ui-scroll",
        "ui scroll",
        crate::modules::audit::InteractionTrigger::Scroll,
    )
    .await
}

/// POST /ui/click — activate an element through the same path as a user click.
pub(crate) async fn click_ui(
    State(state): State<AppState>,
    Json(payload): Json<ElementActionPayload>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    handle_element_action(
        &state,
        payload,
        "agm://ui-click",
        "ui click",
        crate::modules::audit::InteractionTrigger::Click,
    )
    .await
}

#[derive(Deserialize)]
pub(crate) struct ElementTreeQuery {
    route: Option<String>,
}

/// GET /ui/element-tree?route=<route> — return the element manifest for a
/// view: route, components, element ids, focusable element kinds.
/// This is the static manifest (grounded in the real page sources); a live
/// DOM snapshot is a future enhancement.
pub(crate) async fn element_tree(
    Query(query): Query<ElementTreeQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let route = query
        .route
        .unwrap_or_else(|| "dashboard".to_string())
        .trim()
        .trim_start_matches('/')
        .to_lowercase();
    if !UI_ROUTES.contains(&route.as_str()) {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(ErrorResponse {
                error: format!(
                    "unknown route '/{route}'. valid routes: {}",
                    UI_ROUTES.join(", ")
                ),
            }),
        ));
    }
    let manifest = element_manifest(&route);
    Ok(Json(serde_json::json!({
        "success": true,
        "route": format!("/{route}"),
        "components": manifest.components,
        "element_ids": manifest.element_ids,
        "focusable_kinds": manifest.focusable_kinds,
        "note": "static manifest grounded in page sources; live DOM snapshot not yet implemented",
    })))
}

struct RouteManifest {
    components: Vec<&'static str>,
    element_ids: Vec<&'static str>,
    focusable_kinds: Vec<&'static str>,
}

/// Static element manifest per route. Component names mirror the real
/// frontend sources; element ids are only those actually present in the
/// page source (never invented).
fn element_manifest(route: &str) -> RouteManifest {
    let focusable_kinds = vec!["button", "input", "select", "a[href]", "[tabindex]"];
    let mut by_route: HashMap<&str, RouteManifest> = HashMap::new();
    by_route.insert(
        "settings",
        RouteManifest {
            components: vec!["SettingsPage", "SettingsTabs", "GeneralTab"],
            element_ids: vec![],
            focusable_kinds: focusable_kinds.clone(),
        },
    );
    by_route.insert(
        "settings/supabase",
        RouteManifest {
            components: vec!["SettingsPage", "SupabaseSync", "TelegramSection"],
            element_ids: vec![],
            focusable_kinds: focusable_kinds.clone(),
        },
    );
    by_route.insert(
        "instances",
        RouteManifest {
            components: vec!["InstanceTable", "InstanceRowCells", "useInstancePage"],
            element_ids: vec![],
            focusable_kinds: focusable_kinds.clone(),
        },
    );
    by_route.insert(
        "accounts",
        RouteManifest {
            components: vec!["AccountTable", "AccountCard", "AddAccountDialog"],
            element_ids: vec!["#launch-immediately-check"],
            focusable_kinds: focusable_kinds.clone(),
        },
    );
    by_route.get(route).map_or(
        RouteManifest {
            components: vec!["App", "Layout", "Navbar"],
            element_ids: vec![],
            focusable_kinds,
        },
        |m| RouteManifest {
            components: m.components.clone(),
            element_ids: m.element_ids.clone(),
            focusable_kinds: m.focusable_kinds.clone(),
        },
    )
}

#[derive(Deserialize)]
pub(crate) struct RecordListQuery {
    limit: Option<u32>,
}

/// POST /ui/record/start — enable UI interaction recording.
pub(crate) async fn start_record_ui() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>
{
    let was_recording = crate::modules::audit::start_recording().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "success": true,
        "recording": true,
        "was_recording": was_recording,
    })))
}

/// POST /ui/record/stop — disable UI interaction recording.
pub(crate) async fn stop_record_ui() -> impl IntoResponse {
    let was_recording = crate::modules::audit::stop_recording();
    Json(serde_json::json!({
        "success": true,
        "recording": false,
        "was_recording": was_recording,
    }))
}

/// GET /ui/record/status — return whether UI interaction recording is active.
pub(crate) async fn status_record_ui() -> impl IntoResponse {
    let recording = crate::modules::audit::is_recording();
    let count = crate::modules::audit::list_recorded(5000)
        .map(|v| v.len())
        .unwrap_or(0);
    Json(serde_json::json!({
        "success": true,
        "recording": recording,
        "entries_count": count,
    }))
}

/// GET /ui/record/list?limit=<N> — list recorded UI interactions.
pub(crate) async fn list_record_ui(
    Query(query): Query<RecordListQuery>,
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(50).clamp(1, 5000);
    let entries = crate::modules::audit::list_recorded(limit).map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "success": true,
        "recording": crate::modules::audit::is_recording(),
        "count": entries.len(),
        "entries": entries,
    })))
}

/// POST /ui/record/clear — delete all recorded UI interaction entries.
pub(crate) async fn clear_record_ui() -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)>
{
    let deleted = crate::modules::audit::clear_recording().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "success": true,
        "deleted": deleted,
    })))
}

/// POST /ui/record/export — export recorded UI interactions to a timestamped JSON file.
pub(crate) async fn export_record_ui(
) -> Result<impl IntoResponse, (StatusCode, Json<ErrorResponse>)> {
    let path = crate::modules::audit::export_recording().map_err(|err| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(ErrorResponse {
                error: err.to_string(),
            }),
        )
    })?;
    Ok(Json(serde_json::json!({
        "success": true,
        "path": path.to_string_lossy().to_string(),
    })))
}
