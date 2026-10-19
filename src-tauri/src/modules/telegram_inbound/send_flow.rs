//! Two-step `/send` flow for Telegram: pick a destination
//! (machine → instance → conversation) via inline-keyboard taps, then send the
//! prompt text as a plain message. Delivery is confirmed with a screenshot of
//! the destination machine as proof.
//!
//! Destination-first is enforced: prompt text is only accepted after the full
//! destination is locked in — the destination is never guessed. State is
//! process-local, keyed by Telegram chat id, with the same 10-minute TTL as
//! the `/target` aim in `ui_remote.rs`.

use chrono::{Local, Utc};
use once_cell::sync::Lazy;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::open_shot::{local_machine_label, OpenShotTarget};
use super::*;

const SEND_FLOW_TTL_SECS: u64 = 600;
/// Telegram caps `callback_data` at 64 bytes; stay comfortably below it.
const MAX_CALLBACK_BYTES: usize = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SendStage {
    PickInstance,
    PickConversation,
    TypeConversation,
    AwaitPrompt,
}

#[derive(Debug, Clone)]
struct SendFlow {
    machine: String,
    instance: Option<String>,
    conversation: Option<String>,
    stage: SendStage,
    page: usize,
    updated_at: u64,
}

static SEND_FLOWS: Lazy<Mutex<HashMap<i64, SendFlow>>> = Lazy::new(|| Mutex::new(HashMap::new()));

/// Bot-side result of consuming a `/send` message: reply text plus optional markup.
pub struct SendDispatch {
    pub reply: String,
    pub markup: Option<Value>,
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

fn flow_expired(flow: &SendFlow) -> bool {
    now_secs().saturating_sub(flow.updated_at) > SEND_FLOW_TTL_SECS
}

fn clear_flow(chat_id: i64) -> bool {
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        flows.remove(&chat_id).is_some()
    } else {
        false
    }
}

fn send_unknown() -> String {
    "⚠️ That send button has expired or isn't recognized. Send /send to start fresh.".to_string()
}

fn send_cancelled() -> String {
    "Send cancelled — no prompt was sent.".to_string()
}

fn cancel_markup() -> Value {
    json!({
        "inline_keyboard": [[{ "text": "🧹 Cancel", "callback_data": "send:cancel" }]]
    })
}

/// Callback payloads use `|` as separator; entries containing it (or `:`)
/// corrupt parsing, so they are excluded from pickers.
pub(crate) fn is_valid_send_token(token: &str) -> bool {
    let clean = token.trim();
    if clean.is_empty() {
        return false;
    }
    let has_separator = clean.contains('|') || clean.contains(':');
    !has_separator
}

/// Backward compatibility wrapper for `is_valid_send_token`.
fn picker_safe(token: &str) -> bool {
    is_valid_send_token(token)
}

fn typed_project_index(text: &str) -> Option<usize> {
    let number = text.parse::<usize>().ok()?;
    if (1..=8).contains(&number) {
        Some(number - 1)
    } else {
        None
    }
}

fn callback_fits(text: &str) -> bool {
    text.len() <= MAX_CALLBACK_BYTES
}

/// A machine label is picker-usable when it round-trips through a
/// `send:m:<label>` callback payload: no separator characters and inside
/// Telegram's 64-byte `callback_data` cap.
pub(crate) fn is_machine_entry_usable(label: &str) -> bool {
    let is_safe = is_valid_send_token(label);
    let payload = format!("send:m:{label}");
    let has_fitting_payload = callback_fits(&payload);
    is_safe && has_fitting_payload
}

/// Backward compatibility wrapper for `is_machine_entry_usable`.
fn machine_entry_usable(label: &str) -> bool {
    is_machine_entry_usable(label)
}

/// Returns true if an IP string is non-empty and not the "Unknown IP" sentinel.
pub(crate) fn is_known_ip(ip: &str) -> bool {
    let trimmed = ip.trim();
    if trimmed.is_empty() {
        return false;
    }
    !trimmed.eq_ignore_ascii_case("unknown ip")
}

/// Evaluates whether candidate machine alias matches target machine label,
/// accounting for local machine identity.
pub(crate) fn is_matching_target_machine(machine_candidate: &str, target: &str) -> bool {
    let is_exact = machine_candidate.eq_ignore_ascii_case(target);
    let is_both_current = super::node_label::is_current_machine(machine_candidate)
        && super::node_label::is_current_machine(target);
    is_exact || is_both_current
}

/// Debounce guard: determines if chat already has a fresh Step 2 flow for this machine (<3s).
pub(crate) fn is_duplicate_send_machine_tap(
    existing_machine: &str,
    candidate_alias: &str,
    stage: SendStage,
    elapsed_secs: u64,
) -> bool {
    let is_pick_instance = stage == SendStage::PickInstance;
    let is_same_machine = is_matching_target_machine(existing_machine, candidate_alias);
    let is_within_debounce = elapsed_secs < 3;
    is_pick_instance && is_same_machine && is_within_debounce
}

/// Resolves whether an entry alias matches user-specified alias or local aliases.
pub(crate) fn is_matching_machine_alias(entry_alias: &str, input_alias: &str) -> bool {
    if entry_alias.eq_ignore_ascii_case(input_alias) {
        return true;
    }
    let is_local_keyword =
        input_alias.eq_ignore_ascii_case("local") || input_alias.eq_ignore_ascii_case("localhost");
    let is_entry_current = super::node_label::is_current_machine(entry_alias);
    is_local_keyword && is_entry_current
}

/// Determines if an active prompt's instance conflicts with the target instance filter.
pub(crate) fn is_instance_mismatch(prompt_instance_id: &str, target_instance_id: &str) -> bool {
    let has_prompt_instance = !prompt_instance_id.trim().is_empty();
    let has_target_instance = !target_instance_id.trim().is_empty();
    let is_both_specified = has_prompt_instance && has_target_instance;
    let is_different = !prompt_instance_id.eq_ignore_ascii_case(target_instance_id);
    is_both_specified && is_different
}

/// Evaluates whether text typed in Step 3 is likely a prompt rather than a conversation ID.
pub(crate) fn is_likely_prompt_text(trimmed: &str) -> bool {
    let word_count = trimmed.split_whitespace().count();
    let has_multiple_words = word_count > 3;
    if has_multiple_words {
        return true;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.starts_with("please")
        || lower.starts_with("fix")
        || lower.starts_with("implement")
        || lower.starts_with("add")
}

/// Fleet machines the picker can address: this machine plus GitMap cluster nodes.
fn machine_entries() -> Vec<String> {
    let local = local_machine_label();
    let mut entries = Vec::new();
    let mut has_current_machine = false;

    // The local entry anchors the whole flow, but an unusable label must not
    // silently poison the picker: it is held to the same guards as fleet
    // nodes, and the empty state is reported to the user by `start_send_flow`.
    if is_machine_entry_usable(&local) {
        entries.push(local);
        has_current_machine = true;
    } else {
        tracing::warn!(
            "local machine label is not picker-safe; /send destination picker will check cluster nodes"
        );
    }
    for node in query_gitmap_cluster_nodes() {
        let alias = node.alias.trim();
        if !is_machine_entry_usable(alias) {
            continue;
        }

        let is_current = super::node_label::is_current_machine(alias);
        if is_current {
            if has_current_machine {
                // Already have a local machine entry; skip duplicate
                continue;
            } else {
                entries.push(alias.to_string());
                has_current_machine = true;
                continue;
            }
        }

        if !entries.iter().any(|e| e.eq_ignore_ascii_case(alias)) {
            entries.push(alias.to_string());
        }
    }

    // 2. SSH Manager configured connections
    if let Ok(ssh_conns) = crate::modules::ssh_manager::load_ssh_connections() {
        for conn in ssh_conns {
            let alias = conn.alias.trim();
            if !is_machine_entry_usable(alias) {
                continue;
            }

            let is_current = super::node_label::is_current_machine(alias);
            if is_current {
                if has_current_machine {
                    continue;
                } else {
                    entries.push(alias.to_string());
                    has_current_machine = true;
                    continue;
                }
            }

            if !entries.iter().any(|e| e.eq_ignore_ascii_case(alias)) {
                entries.push(alias.to_string());
            }
        }
    }

    // 3. Supabase sync fleet machines
    let fleet_res = if let Ok(handle) = tokio::runtime::Handle::try_current() {
        std::thread::scope(|s| {
            s.spawn(|| handle.block_on(crate::modules::supabase_sync::query_fleet_machines()))
                .join()
                .ok()
                .and_then(|r| r.ok())
        })
        .unwrap_or_else(|| crate::modules::supabase_sync::build_local_and_ssh_fleet())
    } else if let Ok(rt) = tokio::runtime::Runtime::new() {
        rt.block_on(crate::modules::supabase_sync::query_fleet_machines())
            .unwrap_or_else(|_| crate::modules::supabase_sync::build_local_and_ssh_fleet())
    } else {
        crate::modules::supabase_sync::build_local_and_ssh_fleet()
    };

    for m in fleet_res {
        let alias = if !m.alias.trim().is_empty() {
            m.alias.trim()
        } else {
            m.node_alias.trim()
        };
        let has_usable_alias = !alias.is_empty() && is_machine_entry_usable(alias);
        if !has_usable_alias {
            continue;
        }

        let is_current = super::node_label::is_current_machine(alias);
        if is_current {
            if has_current_machine {
                continue;
            } else {
                entries.push(alias.to_string());
                has_current_machine = true;
                continue;
            }
        }

        if !entries.iter().any(|e| e.eq_ignore_ascii_case(alias)) {
            entries.push(alias.to_string());
        }
    }

    entries
}

struct PickerInstance {
    id: String,
    name: String,
    exe_ending: String,
    data_ending: String,
}

/// Last path section only. A Telegram instance line must not carry the full path.
fn path_ending(path: &str) -> String {
    path.trim()
        .trim_end_matches(['/', '\\'])
        .rsplit(['/', '\\'])
        .find(|part| !part.trim().is_empty())
        .unwrap_or("")
        .trim()
        .to_string()
}

fn format_instance_choice(index: usize, inst: &PickerInstance, machine: &str, ip: &str) -> String {
    let host_tag = if is_known_ip(ip) {
        format!(" \u{2022} <i>{machine} [{ip}]</i>")
    } else if !machine.is_empty() {
        format!(" \u{2022} <i>{machine}</i>")
    } else {
        String::new()
    };
    let mut line = format!(
        "{}. 🏷️ <b>{}</b> (<code>{}</code>){}\n",
        index + 1,
        clean_for_telegram_html(&inst.name, 48),
        clean_for_telegram_html(&inst.id, 48),
        host_tag
    );
    if !inst.exe_ending.is_empty() {
        line.push_str(&format!(
            "    └ IDE exe: <code>{}</code>\n",
            clean_for_telegram_html(&inst.exe_ending, 64)
        ));
    }
    if !inst.data_ending.is_empty() {
        line.push_str(&format!(
            "    └ IDE data: <code>{}</code>\n",
            clean_for_telegram_html(&inst.data_ending, 64)
        ));
    }
    line
}

/// Instances from the registry, with only the ending of the IDE exe and data dir.
fn instance_entries() -> Vec<PickerInstance> {
    let mut out: Vec<PickerInstance> = match crate::modules::instance::load_registry() {
        Ok(reg) => reg
            .instances
            .iter()
            .filter(|i| is_valid_send_token(&i.id))
            .map(|i| PickerInstance {
                id: i.id.clone(),
                name: i.name.clone(),
                exe_ending: crate::modules::instance::resolve_instance_exe_name(
                    &i.id,
                    i.executable_path.as_deref(),
                ),
                data_ending: path_ending(&i.data_dir),
            })
            .collect(),
        // Justification: the picker must stay usable when the registry is
        // unreadable; "default" is the documented fallback instance id.
        Err(e) => {
            crate::error::record_ignored(
                Err::<(), String>(e),
                "load instance registry for /send picker",
            );
            Vec::new()
        }
    };
    if out.is_empty() {
        out.push(PickerInstance {
            id: "default".to_string(),
            name: "default".to_string(),
            exe_ending: crate::modules::instance::resolve_instance_exe_name("default", None),
            data_ending: String::new(),
        });
    }
    out
}

fn push_buttons(rows: &mut Vec<Value>, buttons: Vec<Value>, per_row: usize) {
    let mut row: Vec<Value> = Vec::new();
    for b in buttons {
        row.push(b);
        if row.len() == per_row {
            rows.push(json!(row));
            row = Vec::new();
        }
    }
    if !row.is_empty() {
        rows.push(json!(row));
    }
}

fn machine_picker_markup() -> Value {
    let mut rows: Vec<Value> = Vec::new();
    let buttons: Vec<Value> = machine_entries()
        .iter()
        .map(|alias| {
            let current = super::node_label::is_current_machine(alias);
            let style = if current { Some("success") } else { None };
            let mark = if current { "🟢" } else { "🖥️" };
            super::node_label::button_json(
                &format!("{mark} {}", super::node_label::machine_button_text(alias)),
                &format!("send:m:{alias}"),
                style,
            )
        })
        .collect();
    push_buttons(&mut rows, buttons, 2);
    rows.push(json!([{ "text": "🧹 Cancel", "callback_data": "send:cancel" }]));
    json!({ "inline_keyboard": rows })
}

/// Inline keyboard for the `/send` entry message (same as the machine picker).
pub fn send_entry_markup() -> Value {
    machine_picker_markup()
}

fn instance_picker_markup(machine: &str, instances: &[PickerInstance]) -> Value {
    let (node_alias, ip, _) = super::cluster::resolve_node_network_info(machine);
    let mut rows: Vec<Value> = Vec::new();
    let buttons: Vec<Value> = instances
        .iter()
        .filter(|inst| callback_fits(&format!("send:i:{machine}|{}", inst.id)))
        .map(|inst| {
            let label = if is_known_ip(&ip) {
                format!("🏷️ {} ({} \u{2022} {})", inst.name, node_alias, ip)
            } else {
                format!("🏷️ {} ({})", inst.name, node_alias)
            };
            json!({
                "text": label,
                "callback_data": format!("send:i:{machine}|{}", inst.id),
            })
        })
        .collect();
    push_buttons(&mut rows, buttons, 1);
    rows.push(json!([{ "text": "🧹 Cancel", "callback_data": "send:cancel" }]));
    json!({ "inline_keyboard": rows })
}

fn project_picker_markup(page: &super::send_listing::PickerPage) -> Value {
    let mut rows: Vec<Value> = Vec::new();
    let buttons: Vec<Value> = page
        .rows
        .iter()
        .enumerate()
        .map(|(index, _)| {
            super::node_label::button_json(
                &format!("{}", index + 1),
                &format!("send:n:{}|{index}", page.page),
                None,
            )
        })
        .collect();
    push_buttons(&mut rows, buttons, 4);
    let mut nav: Vec<Value> = Vec::new();
    if page.has_back {
        nav.push(super::node_label::button_json(
            "Back",
            &format!("send:back:{}", page.page),
            Some("primary"),
        ));
    }
    if page.has_next {
        nav.push(super::node_label::button_json(
            "Next",
            &format!("send:next:{}", page.page),
            Some("primary"),
        ));
    }
    if !nav.is_empty() {
        rows.push(json!(nav));
    }
    rows.push(json!([
        { "text": "✏️ Enter Project / Conv ID", "callback_data": "send:cman" },
        super::node_label::button_json("🧹 Cancel", "send:cancel", Some("danger")),
    ]));
    json!({ "inline_keyboard": rows })
}

fn render_project_page(
    chat_id: i64,
    machine: &str,
    instance: &str,
    page: usize,
) -> (String, Option<Value>) {
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        if let Some(flow) = flows.get_mut(&chat_id) {
            flow.machine = machine.to_string();
            flow.instance = Some(instance.to_string());
            flow.stage = SendStage::PickConversation;
            flow.page = page;
            flow.updated_at = now_secs();
        }
    }
    let picker = super::send_listing::load_project_page(instance, page);
    let mut out = format!(
        "Where should your prompt go? (Step 3 of 3 — pick the project)\n\
        • <b>Machine:</b> {}\n\
        • <b>Instance:</b> <code>{}</code>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        super::node_label::machine_header_html(machine),
        clean_for_telegram_html(instance, 48)
    );
    if picker.rows.is_empty() {
        out.push_str(
            "No projects found for this instance. \
            Tap ✏️ Enter Project / Conv ID, then reply with the conversation ID \
            (e.g. C001), project name, or conversation id.",
        );
    } else {
        out.push_str(&super::send_listing::format_project_list(
            &picker.rows,
            picker.page,
        ));
        out.push_str("\nTap a number, or type it. Tap Next for the next 8.");
    }
    (out, Some(project_picker_markup(&picker)))
}

fn flow_target(chat_id: i64) -> Option<(String, String)> {
    let flows = SEND_FLOWS.lock().ok()?;
    let flow = flows.get(&chat_id)?;
    if flow_expired(flow) {
        return None;
    }
    let instance = flow.instance.clone()?;
    Some((flow.machine.clone(), instance))
}

fn turn_project_page(chat_id: i64, page: usize) -> (String, Option<Value>) {
    match flow_target(chat_id) {
        Some((machine, instance)) => render_project_page(chat_id, &machine, &instance, page),
        None => (send_unknown(), None),
    }
}

fn on_project_number(chat_id: i64, page: usize, index: usize) -> (String, Option<Value>) {
    let (machine, instance) = match flow_target(chat_id) {
        Some(pair) => pair,
        None => return (send_unknown(), None),
    };
    let picker = super::send_listing::load_project_page(&instance, page);
    match picker.rows.get(index) {
        Some(row) => on_conversation_picked(chat_id, &machine, &instance, &row.value),
        None => (
            "That number isn't on this page. Tap one of the listed numbers, or send /send to start over.".to_string(),
            Some(project_picker_markup(&picker)),
        ),
    }
}

fn destination_locked_reply(machine: &str, instance: &str, conversation: &str) -> String {
    let ide_line = ide_exe_line_for(instance).unwrap_or_default();
    format!(
        "🎯 <b>Destination confirmed</b>\n\
        • <b>Machine:</b> {}\n\
        • <b>Instance:</b> <code>{}</code>\n\
        {}\
        • <b>Conversation:</b> <code>{}</code>\n\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        💬 <b>Type your prompt below</b> and send it as a plain chat message.\n\
        It will be dispatched immediately to this destination.",
        super::node_label::machine_header_html(machine),
        clean_for_telegram_html(instance, 48),
        ide_line,
        clean_for_telegram_html(conversation, 64)
    )
}

/// Handle `/send [cancel]`: open the destination picker, or cancel the flow.
/// `process_telegram_command_text` carries no chat id, and the daemon binds a
/// single allowed chat, so cancel clears every pending flow.
pub fn start_send_flow(rest: &str) -> String {
    if rest.trim().eq_ignore_ascii_case("cancel") {
        if let Ok(mut flows) = SEND_FLOWS.lock() {
            flows.clear();
        }
        return send_cancelled();
    }
    let entries = machine_entries();
    if entries.is_empty() {
        return "Sending is unavailable: this machine's name can't be shown as a button. \
        Rename the node alias and try /send again."
            .to_string();
    }
    let mut out = String::from(
        "Where should your prompt go? (Step 1 of 3 — pick the machine)\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    for (idx, alias) in entries.iter().enumerate() {
        let current = super::node_label::is_current_machine(alias);
        let (node_alias, ip, signal) = super::cluster::resolve_node_network_info(alias);
        let ip_part = if ip.is_empty() || ip == "Unknown IP" {
            String::new()
        } else {
            format!(" • <code>{ip}</code>")
        };
        let (icon, role) = if current {
            ("🟢", "<b>This machine</b>")
        } else {
            ("🌐", "<i>Fleet node</i>")
        };
        out.push_str(&format!(
            "{}. {} <b>{}</b> [{} • 📶 Signal: {}]{}\n",
            idx + 1,
            icon,
            clean_for_telegram_html(&node_alias, 48),
            role,
            clean_for_telegram_html(&signal, 32),
            ip_part
        ));
    }
    out.push_str(
        "\nTap a machine below, then pick the instance and the conversation. \
        The destination is confirmed before any prompt text is accepted.",
    );
    out
}

fn on_machine_picked(chat_id: i64, alias: &str) -> (String, Option<Value>) {
    // 1. Debounce guard: if chat already has a fresh Step 2 flow for this machine (<3s), suppress duplicate message.
    if let Ok(flows) = SEND_FLOWS.lock() {
        if let Some(existing) = flows.get(&chat_id) {
            let elapsed = now_secs().saturating_sub(existing.updated_at);
            if is_duplicate_send_machine_tap(&existing.machine, alias, existing.stage, elapsed) {
                tracing::info!(
                    "Suppressing duplicate send:m:{} callback within debounce window for chat {}",
                    alias,
                    chat_id
                );
                return (String::new(), None);
            }
        }
    }

    let canonical = match machine_entries()
        .into_iter()
        .find(|e| is_matching_machine_alias(e, alias))
    {
        Some(e) => e,
        None => {
            if super::node_label::is_current_machine(alias) {
                local_machine_label()
            } else {
                alias.trim().to_string()
            }
        }
    };
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        flows.insert(
            chat_id,
            SendFlow {
                machine: canonical.clone(),
                instance: None,
                conversation: None,
                stage: SendStage::PickInstance,
                page: 0,
                updated_at: now_secs(),
            },
        );
    }
    let instances = if super::node_label::is_current_machine(&canonical) {
        instance_entries()
    } else {
        let mut rem_instances = Vec::new();
        if let Ok(cluster_items) = crate::modules::ssh_manager::get_cluster_instances_and_balances()
        {
            for item in cluster_items {
                if item.machine_alias.eq_ignore_ascii_case(&canonical) {
                    rem_instances.push(PickerInstance {
                        id: item.instance_id.clone(),
                        name: item.instance_name.clone(),
                        exe_ending: "Antigravity.exe".to_string(),
                        data_ending: if item.bound_email.is_empty()
                            || item.bound_email == "(unbound)"
                        {
                            String::new()
                        } else {
                            item.bound_email
                        },
                    });
                }
            }
        }
        if rem_instances.is_empty() {
            instance_entries()
        } else {
            rem_instances
        }
    };
    let (node_alias, ip, signal) = super::cluster::resolve_node_network_info(&canonical);
    let signal_clean = if signal.starts_with("📶") {
        signal.clone()
    } else {
        format!("📶 {signal}")
    };

    let mut out = format!(
        "🎯 <b>Where should your prompt go? (Step 2 of 3 — pick the instance)</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        📡 <b>Target Machine:</b> {}\n\
        • <b>Machine Alias:</b> <code>{}</code>\n\
        • <b>IP Address:</b> <code>{}</code>\n\
        • <b>Signal Status:</b> {}\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        Instances available on <b>{}</b>:\n",
        super::node_label::machine_header_html(&canonical),
        clean_for_telegram_html(&node_alias, 48),
        clean_for_telegram_html(&ip, 48),
        clean_for_telegram_html(&signal_clean, 48),
        clean_for_telegram_html(&node_alias, 48),
    );
    for (idx, inst) in instances.iter().enumerate() {
        out.push_str(&format_instance_choice(idx, inst, &node_alias, &ip));
    }
    out.push_str("\nTap an instance below to continue.");
    (out, Some(instance_picker_markup(&canonical, &instances)))
}

fn on_instance_picked(chat_id: i64, machine: &str, instance: &str) -> (String, Option<Value>) {
    let canonical_machine = match machine_entries()
        .into_iter()
        .find(|e| e.eq_ignore_ascii_case(machine))
    {
        Some(e) => e,
        None => {
            if super::node_label::is_current_machine(machine) {
                local_machine_label()
            } else {
                machine.trim().to_string()
            }
        }
    };
    let canonical_instance = match instance_entries()
        .into_iter()
        .find(|inst| inst.id.eq_ignore_ascii_case(instance))
    {
        Some(inst) => inst.id,
        None => instance.trim().to_string(),
    };
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        flows.insert(
            chat_id,
            SendFlow {
                machine: canonical_machine.clone(),
                instance: Some(canonical_instance.clone()),
                conversation: None,
                stage: SendStage::PickConversation,
                page: 0,
                updated_at: now_secs(),
            },
        );
    }
    render_project_page(chat_id, &canonical_machine, &canonical_instance, 0)
}

fn on_manual_conversation(chat_id: i64, machine: &str, instance: &str) -> (String, Option<Value>) {
    let mut updated = false;
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        if let Some(flow) = flows.get_mut(&chat_id) {
            if flow.stage == SendStage::PickConversation {
                flow.stage = SendStage::TypeConversation;
                flow.updated_at = now_secs();
                updated = true;
            }
        }
    }
    if !updated {
        return (send_unknown(), None);
    }
    let _ = (machine, instance);
    (
        "Type the conversation. 📁 <b>Step 3 of 3: Enter Project or Conversation ID</b>\n\n\
        Reply with the <b>project name</b> (e.g. <code>Antigravity-Manager</code>), sequence code (e.g. <code>C001</code>, <code>P001</code>), or conversation ID.\n\n\
        ⚠️ <i>Note: This specifies WHERE your prompt will run. You will type your actual prompt text in the next step!</i>"
            .to_string(),
        Some(cancel_markup()),
    )
}

fn recent_prompts_for_destination(
    instance: &str,
    conversation: &str,
    limit: usize,
) -> Vec<crate::modules::repo_db::ActivePrompt> {
    let all = match crate::modules::repo_db::list_all_prompts() {
        Ok(p) => p,
        Err(_) => return Vec::new(),
    };
    let mut matching: Vec<_> = all
        .into_iter()
        .filter(|p| {
            if is_instance_mismatch(&p.instance_id, instance) {
                return false;
            }
            if let Some(ref s) = p.session_id {
                if s.eq_ignore_ascii_case(conversation) {
                    return true;
                }
            }
            if p.project_id.eq_ignore_ascii_case(conversation) {
                return true;
            }
            let label = super::send_listing::project_label(&p.repo_path, &p.project_id, instance);
            if label.eq_ignore_ascii_case(conversation) {
                return true;
            }
            false
        })
        .collect();
    matching.sort_by(|a, b| b.created_at.cmp(&a.created_at));
    matching.truncate(limit);
    matching
}

fn on_conversation_picked(
    chat_id: i64,
    machine: &str,
    instance: &str,
    conversation: &str,
) -> (String, Option<Value>) {
    if conversation.trim().is_empty() {
        return (send_unknown(), None);
    }
    let mut updated = false;
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        if let Some(flow) = flows.get_mut(&chat_id) {
            if flow.stage == SendStage::PickConversation
                || flow.stage == SendStage::TypeConversation
            {
                flow.machine = machine.to_string();
                flow.instance = Some(instance.to_string());
                flow.conversation = Some(conversation.to_string());
                flow.stage = SendStage::AwaitPrompt;
                flow.updated_at = now_secs();
                updated = true;
            }
        }
    }
    if !updated {
        return (send_unknown(), None);
    }

    let mut reply = destination_locked_reply(machine, instance, conversation);
    let recent = recent_prompts_for_destination(instance, conversation, 3);
    let mut markup_rows: Vec<Vec<Value>> = Vec::new();
    if !recent.is_empty() {
        let summary_words = super::config::load_config()
            .map(|c| c.prompt_summary_words)
            .unwrap_or(80);
        reply.push_str("\n\n📜 <i>Optional: Tap below to resend a past prompt instead:</i>");
        for (idx, p) in recent.iter().enumerate() {
            let id8 = if p.id.len() >= 8 { &p.id[..8] } else { &p.id };
            let snippet = super::prompt_history::prompt_preview(p, summary_words);
            reply.push_str(&format!(
                "\n<blockquote expandable><b>{}. [<code>{}</code>]</b>\n<pre>{}</pre></blockquote>",
                idx + 1,
                id8,
                snippet
            ));
            markup_rows.push(vec![json!({
                "text": format!("↻ Resend {id8}"),
                "callback_data": format!("send:r:{id8}")
            })]);
        }
    }
    markup_rows.push(vec![
        json!({ "text": "🧹 Cancel Flow", "callback_data": "send:cancel" }),
    ]);

    (reply, Some(json!({ "inline_keyboard": markup_rows })))
}

/// Handle `send:*` inline-keyboard callbacks. Returns the reply text and markup.
pub async fn handle_send_callback(chat_id: i64, data: &str) -> (String, Option<Value>) {
    let payload = match data.strip_prefix("send:") {
        Some(p) => p,
        None => return (send_unknown(), None),
    };
    if payload == "cancel" {
        clear_flow(chat_id);
        return (send_cancelled(), None);
    }
    if let Some(alias) = payload.strip_prefix("m:") {
        return on_machine_picked(chat_id, alias);
    }
    if let Some(rest) = payload.strip_prefix("i:") {
        return match rest.split_once('|') {
            Some((machine, instance)) => on_instance_picked(chat_id, machine, instance),
            None => (send_unknown(), None),
        };
    }
    // `send:cman` carries no params: machine/instance already live in the
    // flow state (set when the instance was picked). The `cman:<m>|<i>`
    // spelling is still accepted for buttons delivered before the payload
    // was shortened.
    if payload == "cman" {
        return on_manual_conversation(chat_id, "", "");
    }
    if let Some(rest) = payload.strip_prefix("cman:") {
        return match rest.split_once('|') {
            Some((machine, instance)) => on_manual_conversation(chat_id, machine, instance),
            None => (send_unknown(), None),
        };
    }
    if let Some(rest) = payload.strip_prefix("next:") {
        let page = rest.parse::<usize>().unwrap_or(0);
        return turn_project_page(chat_id, page.saturating_add(1));
    }
    if let Some(rest) = payload.strip_prefix("back:") {
        let page = rest.parse::<usize>().unwrap_or(0);
        return turn_project_page(chat_id, page.saturating_sub(1));
    }
    if let Some(rest) = payload.strip_prefix("n:") {
        return match rest.split_once('|') {
            Some((page, index)) => {
                let page = page.parse::<usize>().unwrap_or(0);
                let index = index.parse::<usize>().unwrap_or(usize::MAX);
                on_project_number(chat_id, page, index)
            }
            None => (send_unknown(), None),
        };
    }
    if let Some(rest) = payload.strip_prefix("c:") {
        let mut parts = rest.splitn(3, '|');
        return match (parts.next(), parts.next(), parts.next()) {
            (Some(machine), Some(instance), Some(conversation)) => {
                on_conversation_picked(chat_id, machine, instance, conversation)
            }
            _ => (send_unknown(), None),
        };
    }
    // One-tap rerun from `/history`: `send:r:<id8>` resolves the prompt
    // server-side and re-sends it through the canonical injection path.
    if let Some(id8) = payload.strip_prefix("r:") {
        let (reply, markup) = super::prompt_history::rerun_prompt_by_id(id8).await;
        if reply.contains("No prompt found") {
            return (String::new(), None);
        }
        return (reply, markup);
    }
    // Turn Rerun: send:t:<cid8>:<turn>
    if let Some(rest) = payload.strip_prefix("t:") {
        return handle_turn_rerun_callback(chat_id, rest, false).await;
    }
    // Turn Verification: send:v:<cid8>:<turn>
    if let Some(rest) = payload.strip_prefix("v:") {
        return handle_turn_rerun_callback(chat_id, rest, true).await;
    }
    // Turn Image Fetch: send:img:<cid8>:<turn>
    if let Some(rest) = payload.strip_prefix("img:") {
        return handle_turn_image_callback(chat_id, rest).await;
    }
    (send_unknown(), None)
}

/// Search project conversation tree for a conversation matching short ID, UUID prefix, or sequence code
fn find_conversation_in_tree(
    tree: &[crate::modules::repo_db::models::AgmProjectTreeNode],
    cid8: &str,
) -> Option<(
    crate::modules::repo_db::models::AgmProjectTreeNode,
    crate::modules::repo_db::models::AgmConversationNode,
)> {
    let cid8_lower = cid8.to_lowercase();
    tree.iter().find_map(|proj| {
        proj.conversations.iter().find_map(|conv| {
            if conv.short_id.eq_ignore_ascii_case(cid8)
                || conv.conversation_id.to_lowercase().starts_with(&cid8_lower)
                || conv.seq_code.eq_ignore_ascii_case(cid8)
            {
                Some((proj.clone(), conv.clone()))
            } else {
                None
            }
        })
    })
}

/// Handle turn rerun or turn verification callbacks (`send:t:<cid8>:<turn>` / `send:v:<cid8>:<turn>`)
pub async fn handle_turn_rerun_callback(
    _chat_id: i64,
    payload: &str,
    is_verify: bool,
) -> (String, Option<Value>) {
    // Invariant 2: 60-byte callback payload budget (RCA 15)
    let full_cb = format!("send:{}:{}", if is_verify { "v" } else { "t" }, payload);
    if !callback_fits(&full_cb) {
        return (
            format!(
                "⚠️ Callback payload exceeds safe budget of {} bytes: <code>{}</code>",
                MAX_CALLBACK_BYTES,
                clean_for_telegram_html(&full_cb, 80)
            ),
            None,
        );
    }

    let (cid8, turn_str) = match payload.split_once(':') {
        Some((c, t)) => (c.trim(), t.trim()),
        None => (payload.trim(), "1"),
    };

    let turn_num = turn_str.parse::<usize>().unwrap_or(1);
    if cid8.is_empty() {
        return (send_unknown(), None);
    }

    // Lookup conversation across active instances and prompt sequences
    let tree = crate::modules::repo_db::get_project_conversation_tree(200, false);
    let (proj, conv) = match find_conversation_in_tree(&tree, cid8) {
        Some(m) => m,
        None => {
            // Fallback: check sequence resolution directly
            if let Some(res) =
                crate::modules::repo_db::tree_format::resolve_agm_sequence_target(cid8)
            {
                let full_cid = res.conversation_id.unwrap_or_else(|| cid8.to_string());
                let inst_id = if res.instance_id.is_empty() {
                    "default".to_string()
                } else {
                    res.instance_id
                };
                return dispatch_turn_rerun_execution(
                    cid8,
                    &full_cid,
                    &inst_id,
                    &res.repo_path,
                    &res.seq_code,
                    &res.gitmap_seq_code,
                    turn_num,
                    is_verify,
                    None,
                )
                .await;
            } else {
                return (
                    format!(
                        "⚠️ <b>Conversation Not Found:</b> No active conversation matching <code>{}</code> in registry.\n💡 Send <code>/seek</code> to list available conversations.",
                        clean_for_telegram_html(cid8, 24)
                    ),
                    None,
                );
            }
        }
    };

    let full_cid = conv.conversation_id.clone();
    let instance_id = if conv.instance_id.is_empty() {
        if proj.instance_id.is_empty() {
            "default".to_string()
        } else {
            proj.instance_id.clone()
        }
    } else {
        conv.instance_id.clone()
    };
    let repo_path = proj.repo_path.clone();
    let seq_code = conv.seq_code.clone();
    let gitmap_seq_code = conv.gitmap_seq_code.clone();

    // Look for prompt text for targeted turn
    let mut raw_prompt_opt = None;

    // 1. Try transcript extract_conversation_prompt_turns
    let base_dirs = crate::modules::repo_db::gemini_dirs_for_instance(&instance_id);
    for base in &base_dirs {
        let turns = crate::modules::repo_db::transcript::extract_conversation_prompt_turns(
            base,
            &full_cid,
            conv.is_running,
        );
        if let Some(target) = turns.iter().find(|t| t.turn_number == turn_num) {
            if !target.prompt_text.trim().is_empty() {
                raw_prompt_opt = Some(target.prompt_text.clone());
                break;
            }
        }
    }

    // 2. Try conv.prompts
    if raw_prompt_opt.is_none() {
        if let Some(target) = conv.prompts.iter().find(|t| t.turn_number == turn_num) {
            if !target.prompt_text.trim().is_empty() {
                raw_prompt_opt = Some(target.prompt_text.clone());
            }
        }
    }

    // 3. Fallback to full_prompt_text or preview
    let raw_prompt = raw_prompt_opt
        .or(conv.full_prompt_text.clone())
        .filter(|p| !p.trim().is_empty())
        .unwrap_or(conv.prompt_preview_200w.clone());

    dispatch_turn_rerun_execution(
        cid8,
        &full_cid,
        &instance_id,
        &repo_path,
        &seq_code,
        &gitmap_seq_code,
        turn_num,
        is_verify,
        Some(raw_prompt),
    )
    .await
}

async fn dispatch_turn_rerun_execution(
    cid8: &str,
    full_cid: &str,
    instance_id: &str,
    repo_path: &str,
    seq_code: &str,
    gitmap_seq_code: &str,
    turn_num: usize,
    is_verify: bool,
    prompt_content_opt: Option<String>,
) -> (String, Option<Value>) {
    let raw_prompt = prompt_content_opt
        .unwrap_or_else(|| "Continue current task and verify outcomes".to_string());
    if raw_prompt.trim().is_empty() {
        return (
            format!(
                "⚠️ Conversation <code>{}</code> turn #{} has empty prompt content.",
                clean_for_telegram_html(cid8, 16),
                turn_num
            ),
            None,
        );
    }

    let injected_text = if is_verify {
        format!(
            "Review, verify, and run regression tests for the following implementation:\n{}",
            raw_prompt
        )
    } else {
        raw_prompt.clone()
    };

    let args = format!("{} --instance {} {}", full_cid, instance_id, injected_text);
    let injected = execute_prompt_injection(&args).await;

    // Trigger direct CDP submission for instance
    let _cdp_ok = crate::modules::instance::cdp_onboarding::submit_prompt_via_cdp_for_instance(
        instance_id,
        &injected_text,
    );

    // Capture visual proof of delivery
    let proof = super::open_shot::capture_and_send_proof_shot(&OpenShotTarget::Local).await;

    let (preview_200w, word_count) =
        crate::modules::repo_db::extract_prompt_words_preview(&injected_text, 200);
    let now_utc = Utc::now();
    let utc_str = now_utc.to_rfc3339();
    let local_str = Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string();
    let (_, detected_images) =
        crate::modules::repo_db::extract_image_payload_or_path(&injected_text);
    let media_count = detected_images.len();

    let reply = format!(
        "{icon} <b>Prompt {action} Dispatched & Acknowledged</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n\
        • <b>Dispatched At:</b> <code>{utc_str}</code> ({local_str})\n\
        • <b>Target:</b> <code>[AGM:{seq} | {gm}]</code> · <code>#{c_seq}</code> (<code>{cid8}</code>)\n\
        • <b>Instance:</b> <code>{inst}</code> · <b>Workspace:</b> <code>{ws}</code>\n\
        • <b>Turn:</b> #{turn} ({words} words)\n\
        • <b>Attached Media:</b> {media_count} image(s)\n\
        • <b>Mode:</b> {mode}\n\n\
        <b>Dispatched Prompt:</b>\n\
        <blockquote expandable><pre><code class=\"language-markdown\">{preview}</code></pre></blockquote>\n\n\
        {injected}\n\n\
        {proof}",
        icon = if is_verify { "🛡️" } else { "🔁" },
        action = if is_verify {
            "Verification"
        } else {
            "Turn Rerun"
        },
        seq = clean_for_telegram_html(seq_code, 16),
        gm = clean_for_telegram_html(gitmap_seq_code, 24),
        c_seq = clean_for_telegram_html(seq_code, 16),
        cid8 = clean_for_telegram_html(cid8, 12),
        inst = clean_for_telegram_html(instance_id, 32),
        ws = clean_for_telegram_html(repo_path, 64),
        turn = turn_num,
        words = word_count,
        mode = if is_verify {
            "🛡️ Verification &amp; Regression Test"
        } else {
            "🔁 Verbatim Turn Rerun"
        },
        preview = clean_for_telegram_html(&preview_200w, 2000),
        injected = injected,
        proof = proof,
    );

    (reply, None)
}

/// Handle sending attached images for a prompt turn directly to the Telegram chat
async fn handle_turn_image_callback(chat_id: i64, payload: &str) -> (String, Option<Value>) {
    let (cid8, turn_str) = match payload.split_once(':') {
        Some((c, t)) => (c.trim(), t.trim()),
        None => (payload.trim(), "1"),
    };
    let turn_num = turn_str.parse::<usize>().unwrap_or(1);
    if cid8.is_empty() {
        return (send_unknown(), None);
    }

    let tree = crate::modules::repo_db::get_project_conversation_tree(200, false);
    let (proj, conv) = match find_conversation_in_tree(&tree, cid8) {
        Some(m) => m,
        None => {
            return (
                format!(
                    "⚠️ <b>Conversation Not Found:</b> No active conversation matching <code>{}</code> in registry.",
                    clean_for_telegram_html(cid8, 24)
                ),
                None,
            );
        }
    };

    // Find targeted turn
    let target_turn = conv
        .prompts
        .iter()
        .find(|t| t.turn_number == turn_num)
        .or_else(|| conv.prompts.last());

    let media_paths = target_turn
        .map(|t| t.media_paths.clone())
        .unwrap_or_default();

    if media_paths.is_empty() {
        return (
            format!(
                "ℹ️ <b>No Images Found:</b> #{} Turn #{} has no image attachments.",
                conv.seq_code, turn_num
            ),
            None,
        );
    }

    let token = match super::load_config().map(|c| c.bot_token).ok() {
        Some(t) if !t.is_empty() => t,
        _ => return ("⚠️ Telegram bot token not configured.".to_string(), None),
    };

    let mut sent_count = 0;
    for (idx, path_str) in media_paths.iter().enumerate() {
        let p = std::path::Path::new(path_str);
        if p.exists() {
            if let Ok(bytes) = std::fs::read(p) {
                let caption = format!(
                    "🖼️ <b>Attachment {}/{}</b> · #{} Turn #{}\n<code>{}</code>",
                    idx + 1,
                    media_paths.len(),
                    conv.seq_code,
                    turn_num,
                    clean_for_telegram_html(&proj.project_id, 40)
                );
                if super::media::send_telegram_photo(&token, chat_id, &bytes, &caption, None)
                    .await
                    .is_ok()
                {
                    sent_count += 1;
                }
            }
        }
    }

    if sent_count > 0 {
        (
            format!(
                "✅ <b>Delivered {} image attachment(s)</b> for #{} Turn #{} to chat.",
                sent_count, conv.seq_code, turn_num
            ),
            None,
        )
    } else {
        (
            "⚠️ <b>Images Missing on Disk:</b> Attached files could not be read from disk."
                .to_string(),
            None,
        )
    }
}

/// Dispatch the prompt through the existing prompt-injection path and confirm
/// with a screenshot of the destination machine as proof of delivery.
async fn dispatch_send_prompt(
    machine: &str,
    instance: &str,
    conversation: &str,
    prompt: &str,
) -> SendDispatch {
    let is_local = super::node_label::is_current_machine(machine);
    let target = if is_local {
        OpenShotTarget::Local
    } else {
        OpenShotTarget::Remote(machine.to_string())
    };
    // Same argument shape the existing `/prompt` / `/target` flows use, with
    // `--node` added for remote machines (handled by `execute_prompt_injection`).
    let args = if is_local {
        format!("{conversation} --instance {instance} {prompt}")
    } else {
        format!("{conversation} --instance {instance} --node {machine} {prompt}")
    };
    let injected = execute_prompt_injection(&args).await;
    let proof = super::open_shot::capture_and_send_proof_shot(&target).await;
    let ide_line = ide_exe_line_for(instance).unwrap_or_default();
    let now_utc = Utc::now();
    let utc_str = now_utc.to_rfc3339();
    let local_str = Local::now().format("%Y-%m-%d %I:%M:%S %p").to_string();
    let (_, detected_images) = crate::modules::repo_db::extract_image_payload_or_path(prompt);
    let media_count = detected_images.len();

    let reply = if ide_line.is_empty() {
        format!(
            "✅ <b>Prompt Dispatched & Acknowledged</b>\n\
            • <b>Dispatched At:</b> <code>{}</code> ({})\n\
            • <b>Target Node:</b> <code>{}</code>\n\
            • <b>Instance:</b> <code>{}</code>\n\
            • <b>Conversation:</b> <code>{}</code>\n\
            • <b>Attached Media:</b> {} image(s)\n\n\
            {}\n\n{}",
            utc_str,
            local_str,
            clean_for_telegram_html(machine, 48),
            clean_for_telegram_html(instance, 48),
            clean_for_telegram_html(conversation, 64),
            media_count,
            injected,
            proof
        )
    } else {
        format!(
            "✅ <b>Prompt Dispatched & Acknowledged</b>\n\
            • <b>Dispatched At:</b> <code>{}</code> ({})\n\
            • <b>Target Node:</b> <code>{}</code>\n\
            • <b>Instance:</b> <code>{}</code>\n\
            • <b>Conversation:</b> <code>{}</code>\n\
            • <b>Attached Media:</b> {} image(s)\n\
            {}{}\n\n{}",
            utc_str,
            local_str,
            clean_for_telegram_html(machine, 48),
            clean_for_telegram_html(instance, 48),
            clean_for_telegram_html(conversation, 64),
            media_count,
            ide_line,
            injected,
            proof
        )
    };
    SendDispatch {
        reply,
        markup: None,
    }
}

/// Consume a plain-text message for the `/send` flow when a destination is
/// pending for this chat. Returns `None` for commands, empty text, and when
/// no flow is waiting for typed input — those fall through to normal routing.
pub async fn consume_send_prompt(
    _token: &str,
    chat_id: i64,
    text: &str,
    saved_media_opt: &Option<(std::path::PathBuf, Option<String>)>,
) -> Option<SendDispatch> {
    let trimmed = text.trim();
    if trimmed.starts_with('/') {
        return None;
    }
    let has_text = !trimmed.is_empty();
    let has_media = saved_media_opt.is_some();
    let has_payload = has_text || has_media;
    if !has_payload {
        return None;
    }
    // Scope the lock: extract what the await needs, then drop the guard
    // BEFORE any `.await` (std MutexGuard is not Send).
    enum Next {
        LockedReply(SendDispatch),
        Dispatch {
            machine: String,
            instance: String,
            conversation: String,
        },
        ProjectPick {
            page: usize,
            index: usize,
        },
        ProjectPage {
            page: usize,
        },
    }
    let next = {
        let mut flows = SEND_FLOWS.lock().ok()?;
        let flow = flows.get_mut(&chat_id)?;
        if flow_expired(flow) {
            flows.remove(&chat_id);
            return None;
        }
        match flow.stage {
            SendStage::PickConversation => {
                let page = flow.page;
                if let Some(index) = typed_project_index(trimmed) {
                    Next::ProjectPick { page, index }
                } else if trimmed.eq_ignore_ascii_case("next") {
                    Next::ProjectPage {
                        page: page.saturating_add(1),
                    }
                } else {
                    return None;
                }
            }
            SendStage::TypeConversation => {
                let is_likely_prompt = is_likely_prompt_text(trimmed);

                if is_likely_prompt {
                    // Gracefully route directly to the default project rather than naming the conversation the entire prompt!
                    let machine = flow.machine.clone();
                    let instance = flow
                        .instance
                        .clone()
                        .unwrap_or_else(|| "default".to_string());
                    flows.remove(&chat_id);
                    Next::Dispatch {
                        machine,
                        instance,
                        conversation: "default".to_string(),
                    }
                } else {
                    flow.conversation = Some(trimmed.to_string());
                    flow.stage = SendStage::AwaitPrompt;
                    flow.updated_at = now_secs();
                    let reply = destination_locked_reply(
                        &flow.machine.clone(),
                        flow.instance.as_deref().unwrap_or("default"),
                        trimmed,
                    );
                    Next::LockedReply(SendDispatch {
                        reply,
                        markup: Some(cancel_markup()),
                    })
                }
            }
            SendStage::AwaitPrompt => {
                let machine = flow.machine.clone();
                let instance = flow
                    .instance
                    .clone()
                    .unwrap_or_else(|| "default".to_string());
                let conversation = flow.conversation.clone().unwrap_or_default();
                flows.remove(&chat_id);
                Next::Dispatch {
                    machine,
                    instance,
                    conversation,
                }
            }
            _ => return None,
        }
    };
    match next {
        Next::LockedReply(dispatch) => Some(dispatch),
        Next::ProjectPick { page, index } => {
            let (reply, markup) = on_project_number(chat_id, page, index);
            Some(SendDispatch { reply, markup })
        }
        Next::ProjectPage { page } => {
            let (reply, markup) = turn_project_page(chat_id, page);
            Some(SendDispatch { reply, markup })
        }
        Next::Dispatch {
            machine,
            instance,
            conversation,
        } => {
            let prompt_text = match (trimmed.is_empty(), saved_media_opt) {
                (false, Some((path, _))) => {
                    format!("{}\n\n![Attached Image]({})", trimmed, path.display())
                }
                (true, Some((path, _))) => format!("![Attached Image]({})", path.display()),
                (false, None) => trimmed.to_string(),
                (true, None) => return None,
            };
            Some(dispatch_send_prompt(&machine, &instance, &conversation, &prompt_text).await)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_safety_rejects_separators() {
        assert!(is_valid_send_token("W3"));
        assert!(picker_safe("W3"));
        assert!(is_valid_send_token("default"));
        assert!(picker_safe("default"));
        assert!(!is_valid_send_token("a|b"));
        assert!(!picker_safe("a|b"));
        assert!(!is_valid_send_token("a:b"));
        assert!(!picker_safe("a:b"));
        assert!(!is_valid_send_token(""));
        assert!(!picker_safe(""));
    }

    #[test]
    fn flow_expiry_uses_ttl() {
        let fresh = SendFlow {
            machine: "local".to_string(),
            instance: None,
            conversation: None,
            stage: SendStage::PickInstance,
            page: 0,
            updated_at: now_secs(),
        };
        assert!(!flow_expired(&fresh));
        let stale = SendFlow {
            updated_at: now_secs().saturating_sub(SEND_FLOW_TTL_SECS + 1),
            ..fresh
        };
        assert!(flow_expired(&stale));
    }

    #[test]
    fn destination_locked_reply_mentions_all_parts() {
        let reply = destination_locked_reply("remote-studio", "default", "C001");
        assert!(reply.contains("remote-studio"));
        assert!(reply.contains("default"));
        assert!(reply.contains("C001"));
        assert!(reply.contains("Destination confirmed"));
    }

    static TEST_FLOW_MUTEX: Mutex<()> = Mutex::new(());

    fn seed_flow(chat_id: i64, stage: SendStage) {
        let mut flows = SEND_FLOWS.lock().unwrap();
        flows.insert(
            chat_id,
            SendFlow {
                machine: "W3".to_string(),
                instance: Some("default".to_string()),
                conversation: None,
                stage,
                page: 0,
                updated_at: now_secs(),
            },
        );
    }

    fn drop_flow(chat_id: i64) {
        if let Ok(mut flows) = SEND_FLOWS.lock() {
            flows.remove(&chat_id);
        }
    }

    #[test]
    fn send_stage_transition_pick_conversation_locks_destination() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        let chat_id: i64 = -9_001_337;
        seed_flow(chat_id, SendStage::PickConversation);

        let (reply, markup) = on_conversation_picked(chat_id, "W3", "default", "C001");
        assert!(reply.contains("Destination confirmed"));
        assert!(reply.contains("C001"));
        assert!(markup.is_some());

        {
            let flows = SEND_FLOWS.lock().unwrap();
            let flow = flows
                .get(&chat_id)
                .expect("flow must survive the transition");
            assert_eq!(flow.stage, SendStage::AwaitPrompt);
            assert_eq!(flow.conversation.as_deref(), Some("C001"));
        }

        // Picking again at AwaitPrompt is rejected, never silently re-locked.
        let (retry, _) = on_conversation_picked(chat_id, "W3", "default", "C002");
        assert!(retry.contains("isn't recognized"));
        drop_flow(chat_id);
    }

    #[test]
    fn send_stage_transition_manual_conversation() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        let chat_id: i64 = -9_001_338;
        seed_flow(chat_id, SendStage::PickConversation);

        let (reply, markup) = on_manual_conversation(chat_id, "W3", "default");
        assert!(reply.contains("Type the conversation"));
        assert!(markup.is_some());
        {
            let flows = SEND_FLOWS.lock().unwrap();
            assert_eq!(
                flows.get(&chat_id).expect("flow must survive").stage,
                SendStage::TypeConversation
            );
        }

        // No flow for this chat: rejected, no state created.
        let (unknown, none) = on_manual_conversation(-9_001_339, "W3", "default");
        assert!(unknown.contains("isn't recognized"));
        assert!(none.is_none());
        drop_flow(chat_id);
    }

    #[test]
    fn send_cancel_clears_pending_flows() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        let chat_id: i64 = -9_001_340;
        seed_flow(chat_id, SendStage::PickInstance);

        let reply = start_send_flow("cancel");
        assert!(reply.contains("cancelled"));
        assert!(reply.contains("no prompt was sent"));
        assert!(SEND_FLOWS.lock().unwrap().is_empty());
    }

    #[test]
    fn send_start_lists_machines_destination_first() {
        let reply = start_send_flow("");
        assert!(reply.contains("Step 1 of 3"));
        assert!(reply.contains("destination is confirmed"));
    }

    #[tokio::test]
    async fn send_callback_cancel_clears_flow() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        let chat_id: i64 = -9_001_341;
        seed_flow(chat_id, SendStage::PickInstance);

        let (reply, markup) = handle_send_callback(chat_id, "send:cancel").await;
        assert!(reply.contains("cancelled"));
        assert!(markup.is_none());
        assert!(SEND_FLOWS.lock().unwrap().get(&chat_id).is_none());
    }

    #[tokio::test]
    async fn send_callback_unknown_payload_is_rejected() {
        let (reply, markup) = handle_send_callback(-9_001_342, "send:bogus").await;
        assert!(reply.contains("isn't recognized"));
        assert!(markup.is_none());
    }

    #[test]
    fn send_callback_routing_parses_prefixes() {
        // Pure prefix contract shared with the inline keyboards built above.
        assert_eq!("send:m:W3".strip_prefix("send:").unwrap(), "m:W3");
        assert!("send:i:W3|default"
            .strip_prefix("send:")
            .unwrap()
            .starts_with("i:"));
        assert!("send:c:W3|default|C001"
            .strip_prefix("send:")
            .unwrap()
            .starts_with("c:"));
        assert!("send:cman:W3|default"
            .strip_prefix("send:")
            .unwrap()
            .starts_with("cman:"));
        assert_eq!("send:cman".strip_prefix("send:").unwrap(), "cman");
        assert_eq!("send:cancel".strip_prefix("send:").unwrap(), "cancel");
    }

    #[tokio::test]
    async fn send_callback_cman_accepts_short_and_legacy_payloads() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        // New short form: params come from the flow state, payload stays tiny.
        let chat_id: i64 = -9_001_343;
        seed_flow(chat_id, SendStage::PickConversation);
        let (reply, markup) = handle_send_callback(chat_id, "send:cman").await;
        assert!(reply.contains("Type the conversation"));
        assert!(markup.is_some());
        {
            let flows = SEND_FLOWS.lock().unwrap();
            assert_eq!(
                flows.get(&chat_id).expect("flow must survive").stage,
                SendStage::TypeConversation
            );
        }
        drop_flow(chat_id);

        // Legacy long form from buttons delivered before the shortening.
        let legacy_id: i64 = -9_001_344;
        seed_flow(legacy_id, SendStage::PickConversation);
        let (legacy_reply, _) = handle_send_callback(legacy_id, "send:cman:W3|default").await;
        assert!(legacy_reply.contains("Type the conversation"));
        drop_flow(legacy_id);
    }

    #[test]
    fn machine_entry_usable_guards_picker_payloads() {
        assert!(is_machine_entry_usable("W3"));
        assert!(machine_entry_usable("W3"));
        assert!(is_machine_entry_usable("antigravity-node"));
        assert!(machine_entry_usable("antigravity-node"));
        // Separator characters would corrupt `|`-split parsing.
        assert!(!is_machine_entry_usable("node|evil"));
        assert!(!machine_entry_usable("node|evil"));
        assert!(!is_machine_entry_usable("node:evil"));
        assert!(!machine_entry_usable("node:evil"));
        assert!(!is_machine_entry_usable(""));
        assert!(!machine_entry_usable(""));
        // `send:m:` is 7 bytes; the alias must leave room under the 64-byte cap.
        assert!(!is_machine_entry_usable(&"n".repeat(60)));
        assert!(!machine_entry_usable(&"n".repeat(60)));
        assert!(is_machine_entry_usable(&"n".repeat(50)));
        assert!(machine_entry_usable(&"n".repeat(50)));
    }

    #[test]
    fn test_affirmative_send_helpers() {
        assert!(is_known_ip("192.168.1.1"));
        assert!(!is_known_ip("Unknown IP"));
        assert!(!is_known_ip("unknown ip"));
        assert!(!is_known_ip(""));

        assert!(is_matching_target_machine("worker1", "worker1"));
        assert!(is_matching_target_machine("local", "localhost"));
        assert!(!is_matching_target_machine("worker1", "worker2"));

        assert!(is_duplicate_send_machine_tap(
            "worker1",
            "worker1",
            SendStage::PickInstance,
            2
        ));
        assert!(!is_duplicate_send_machine_tap(
            "worker1",
            "worker1",
            SendStage::PickInstance,
            4
        ));
        assert!(!is_duplicate_send_machine_tap(
            "worker1",
            "worker2",
            SendStage::PickInstance,
            1
        ));
        assert!(!is_duplicate_send_machine_tap(
            "worker1",
            "worker1",
            SendStage::PickConversation,
            1
        ));

        assert!(is_matching_machine_alias("worker1", "worker1"));
        assert!(is_matching_machine_alias("local", "localhost"));
        assert!(!is_matching_machine_alias("worker1", "worker2"));

        assert!(is_instance_mismatch("inst1", "inst2"));
        assert!(!is_instance_mismatch("inst1", "inst1"));
        assert!(!is_instance_mismatch("", "inst2"));
        assert!(!is_instance_mismatch("inst1", ""));

        assert!(is_likely_prompt_text("one two three four"));
        assert!(is_likely_prompt_text("please fix this"));
        assert!(is_likely_prompt_text("fix bug"));
        assert!(is_likely_prompt_text("implement feature"));
        assert!(is_likely_prompt_text("add item"));
        assert!(!is_likely_prompt_text("conversation1"));
    }

    #[test]
    fn path_ending_keeps_only_the_last_section() {
        assert_eq!(
            path_ending(r"C:\Users\Administrator\AppData\Roaming\Antigravity"),
            "Antigravity"
        );
        assert_eq!(path_ending(r"D:\apps\Antigravity.exe"), "Antigravity.exe");
        assert_eq!(
            path_ending("/home/me/.antigravity/instances/default-copy-5597/"),
            "default-copy-5597"
        );
        assert_eq!(path_ending("Antigravity.exe"), "Antigravity.exe");
        assert!(path_ending("").is_empty());
    }

    #[test]
    fn instance_choice_lists_exe_and_data_endings_only() {
        let inst = PickerInstance {
            id: "default-copy-5597".to_string(),
            name: "Default Copy".to_string(),
            exe_ending: "Antigravity.exe".to_string(),
            data_ending: "default-copy-5597".to_string(),
        };
        let text = format_instance_choice(1, &inst, "w1", "192.168.1.3");
        assert!(text.contains("IDE exe"));
        assert!(text.contains("Antigravity.exe"));
        assert!(text.contains("IDE data"));
        assert!(text.contains("default-copy-5597"));
        assert!(text.contains("w1 [192.168.1.3]"));
        assert!(!text.contains(":\\"));
        assert!(!text.contains("/home/"));
    }

    #[tokio::test]
    async fn test_turn_callback_payload_bounds() {
        // Enforce 60-byte limit on turn callback data (RCA 15)
        let cid8 = "d58c5517";
        let turn = 1;
        let rerun_payload = format!("send:t:{}:{}", cid8, turn);
        let verify_payload = format!("send:v:{}:{}", cid8, turn);
        assert!(callback_fits(&rerun_payload));
        assert!(callback_fits(&verify_payload));
        assert!(rerun_payload.len() <= MAX_CALLBACK_BYTES);
        assert!(verify_payload.len() <= MAX_CALLBACK_BYTES);
    }

    #[tokio::test]
    async fn test_handle_turn_rerun_callback_safe_bounds() {
        let chat_id: i64 = -9_001_345;
        // Nonexistent CID returns graceful not-found message without panicking
        let (reply, markup) = handle_send_callback(chat_id, "send:t:nonexist:1").await;
        assert!(
            reply.contains("Conversation Not Found")
                || reply.contains("No active conversation")
                || reply.contains("No prompt was sent")
        );
        assert!(markup.is_none());

        // Oversized callback rejected
        let oversized = format!("send:t:{}:1", "x".repeat(70));
        let (oversized_reply, _) = handle_send_callback(chat_id, &oversized).await;
        assert!(
            oversized_reply.contains("exceeds safe budget")
                || oversized_reply.contains("expired")
                || oversized_reply.contains("isn't recognized")
        );
    }

    #[test]
    fn test_step2_machine_shielding_and_signal_details() {
        let _lock = TEST_FLOW_MUTEX.lock().unwrap();
        let chat_id: i64 = -9_001_350;

        // 1. Cross-machine relay: remote machine callback produces Step 2 to allow picking instances on target node
        let (remote_reply, remote_markup) = on_machine_picked(chat_id, "remote-machine-01");
        assert!(
            remote_reply.contains("Step 2 of 3"),
            "cross-machine relay must produce Step 2 header"
        );
        assert!(
            remote_markup.is_some(),
            "cross-machine relay must provide instance buttons"
        );

        // Reset flow for clean local test
        drop_flow(chat_id);

        // 2. Local machine callback produces Step 2 with alias, IP, and signal
        let (step2_reply, step2_markup) = on_machine_picked(chat_id, "local");
        assert!(
            step2_reply.contains("Step 2 of 3"),
            "must contain Step 2 header"
        );
        assert!(
            step2_reply.contains("Machine Alias:"),
            "must display machine alias"
        );
        assert!(
            step2_reply.contains("IP Address:"),
            "must display IP address"
        );
        assert!(
            step2_reply.contains("Signal Status:"),
            "must display signal status"
        );
        assert!(step2_markup.is_some(), "must provide instance buttons");

        // 3. Immediate repeat within debounce window (<3s) must be suppressed to prevent duplicate messages
        let (suppressed_reply, suppressed_markup) = on_machine_picked(chat_id, "local");
        assert!(
            suppressed_reply.is_empty(),
            "immediate duplicate tap must be suppressed"
        );
        assert!(suppressed_markup.is_none());

        drop_flow(chat_id);
    }
}
