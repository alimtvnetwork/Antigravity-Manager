//! Two-step `/send` flow for Telegram: pick a destination
//! (machine → instance → conversation) via inline-keyboard taps, then send the
//! prompt text as a plain message. Delivery is confirmed with a screenshot of
//! the destination machine as proof.
//!
//! Destination-first is enforced: prompt text is only accepted after the full
//! destination is locked in — the destination is never guessed. State is
//! process-local, keyed by Telegram chat id, with the same 10-minute TTL as
//! the `/target` aim in `ui_remote.rs`.

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

fn clear_flow(chat_id: i64) {
    if let Ok(mut flows) = SEND_FLOWS.lock() {
        flows.remove(&chat_id);
    }
}

fn send_unknown() -> String {
    "⚠️ <b>Unknown send action.</b> Send <code>/send</code> to start over.".to_string()
}

fn cancel_markup() -> Value {
    json!({
        "inline_keyboard": [[{ "text": "🧹 Cancel", "callback_data": "send:cancel" }]]
    })
}

/// Callback payloads use `|` as separator; entries containing it (or `:`)
/// would corrupt parsing, so they are excluded from pickers.
fn picker_safe(token: &str) -> bool {
    !token.is_empty() && !token.contains('|') && !token.contains(':')
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
fn machine_entry_usable(label: &str) -> bool {
    picker_safe(label) && callback_fits(&format!("send:m:{label}"))
}

/// Fleet machines the picker can address: this machine plus GitMap cluster nodes.
fn machine_entries() -> Vec<String> {
    let local = local_machine_label();
    let mut entries = Vec::new();
    // The local entry anchors the whole flow, but an unusable label must not
    // silently poison the picker: it is held to the same guards as fleet
    // nodes, and the empty state is reported to the user by `start_send_flow`.
    if machine_entry_usable(&local) {
        entries.push(local);
    } else {
        tracing::warn!(
            "local machine label is not picker-safe; /send destination picker will be empty"
        );
    }
    for node in query_gitmap_cluster_nodes() {
        let alias = node.alias.trim();
        if machine_entry_usable(alias) && !entries.iter().any(|e| e.eq_ignore_ascii_case(alias)) {
            entries.push(alias.to_string());
        }
    }
    entries
}

/// (id, display name) pairs from the existing instance registry.
fn instance_entries() -> Vec<(String, String)> {
    let mut out: Vec<(String, String)> = match crate::modules::instance::load_registry() {
        Ok(reg) => reg
            .instances
            .iter()
            .filter(|i| picker_safe(&i.id))
            .map(|i| (i.id.clone(), i.name.clone()))
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
        out.push(("default".to_string(), "default".to_string()));
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

fn instance_picker_markup(machine: &str, instances: &[(String, String)]) -> Value {
    let mut rows: Vec<Value> = Vec::new();
    let buttons: Vec<Value> = instances
        .iter()
        .filter(|(id, _)| callback_fits(&format!("send:i:{machine}|{id}")))
        .map(|(id, name)| {
            json!({
                "text": format!("🏷️ {}", name),
                "callback_data": format!("send:i:{machine}|{id}"),
            })
        })
        .collect();
    push_buttons(&mut rows, buttons, 2);
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
        { "text": "✏️ Type it manually", "callback_data": "send:cman" },
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
        "📮 <b>Send a prompt — step 3 of 3: pick the project</b>\n\
        • <b>Machine:</b> {}\n\
        • <b>Instance:</b> <code>{}</code>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        super::node_label::machine_header_html(machine),
        clean_for_telegram_html(instance, 48)
    );
    if picker.rows.is_empty() {
        out.push_str(
            "No projects were found for this instance. \
            Tap <b>✏️ Type it manually</b> and reply with the conversation Seq ID \
            (e.g. <code>C001</code>), project name, or conversation id.",
        );
    } else {
        out.push_str(&super::send_listing::format_project_list(
            &picker.rows,
            picker.page,
        ));
        out.push_str(
            "\n👆 Tap a number, or type that number. <b>Next</b> shows the next eight projects.",
        );
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
            "⚠️ <b>That number is not on this page.</b> Tap a number that is listed, or send <code>/send</code> to start over.".to_string(),
            Some(project_picker_markup(&picker)),
        ),
    }
}

fn destination_locked_reply(machine: &str, instance: &str, conversation: &str) -> String {
    format!(
        "🎯 <b>Destination locked</b>\n\
        • <b>Machine:</b> {}\n\
        • <b>Instance:</b> <code>{}</code>\n\
        • <b>Conversation:</b> <code>{}</code>\n\n\
        ✍️ Now send the prompt text as a plain message. It will go to exactly this destination — nothing is guessed.",
        super::node_label::machine_header_html(machine),
        clean_for_telegram_html(instance, 48),
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
        return "🧹 <b>Send flow cancelled.</b> Nothing was dispatched.".to_string();
    }
    let entries = machine_entries();
    if entries.is_empty() {
        return "⚠️ <b>Send unavailable:</b> this machine's label cannot be shown \
        in the destination picker (unsafe characters or too long for a button). \
        Rename the node alias and try <code>/send</code> again."
            .to_string();
    }
    let mut out = String::from(
        "📮 <b>Send a prompt — step 1 of 3: pick the destination machine</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    for (idx, alias) in entries.iter().enumerate() {
        let current = super::node_label::is_current_machine(alias);
        let marker = if current {
            "🟢 <b>this machine</b>"
        } else {
            "🌐 <i>fleet node</i>"
        };
        let name = super::node_label::machine_button_text(alias);
        out.push_str(&format!(
            "{}. <b>{}</b> {}\n",
            idx + 1,
            clean_for_telegram_html(&name, 48),
            marker
        ));
    }
    out.push_str(
        "\n👆 Tap a machine below, then pick the instance and the conversation. \
        The destination locks <b>before</b> any prompt text is accepted.",
    );
    out
}

fn on_machine_picked(chat_id: i64, alias: &str) -> (String, Option<Value>) {
    let canonical = match machine_entries()
        .into_iter()
        .find(|e| e.eq_ignore_ascii_case(alias))
    {
        Some(e) => e,
        None => {
            return (
                format!(
                    "⚠️ <b>Unknown machine:</b> <code>{}</code>. Send <code>/send</code> to pick again.",
                    clean_for_telegram_html(alias, 48)
                ),
                None,
            )
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
    let instances = instance_entries();
    let mut out = format!(
        "📮 <b>Send a prompt — step 2 of 3: pick the instance on {}</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        super::node_label::machine_header_html(&canonical)
    );
    for (idx, (id, name)) in instances.iter().enumerate() {
        out.push_str(&format!(
            "{}. 🏷️ <b>{}</b> (<code>{}</code>)\n",
            idx + 1,
            clean_for_telegram_html(name, 48),
            clean_for_telegram_html(id, 48)
        ));
    }
    out.push_str("\n👆 Tap an instance. Instance ids resolve on the target machine.");
    (out, Some(instance_picker_markup(&canonical, &instances)))
}

fn on_instance_picked(chat_id: i64, machine: &str, instance: &str) -> (String, Option<Value>) {
    let canonical_machine = match machine_entries()
        .into_iter()
        .find(|e| e.eq_ignore_ascii_case(machine))
    {
        Some(e) => e,
        None => {
            return (
                format!(
                    "⚠️ <b>Unknown machine:</b> <code>{}</code>. Send <code>/send</code> to start over.",
                    clean_for_telegram_html(machine, 48)
                ),
                None,
            )
        }
    };
    let canonical_instance = match instance_entries()
        .into_iter()
        .find(|(id, _)| id.eq_ignore_ascii_case(instance))
    {
        Some((id, _)) => id,
        None => {
            return (
                format!(
                    "⚠️ <b>Unknown instance:</b> <code>{}</code>. Send <code>/send</code> to start over.",
                    clean_for_telegram_html(instance, 48)
                ),
                None,
            )
        }
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
        "✏️ <b>Type the conversation</b>\n\
        Reply with the conversation Seq ID (e.g. <code>C001</code>), project name, \
        or conversation id as a plain message. The destination locks before any prompt text is accepted."
            .to_string(),
        Some(cancel_markup()),
    )
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
    (
        destination_locked_reply(machine, instance, conversation),
        Some(cancel_markup()),
    )
}

/// Handle `send:*` inline-keyboard callbacks. Returns the reply text and markup.
pub async fn handle_send_callback(chat_id: i64, data: &str) -> (String, Option<Value>) {
    let payload = match data.strip_prefix("send:") {
        Some(p) => p,
        None => return (send_unknown(), None),
    };
    if payload == "cancel" {
        clear_flow(chat_id);
        return (
            "🧹 <b>Send flow cancelled.</b> Nothing was dispatched.".to_string(),
            None,
        );
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
    (send_unknown(), None)
}

/// Dispatch the prompt through the existing prompt-injection path and confirm
/// with a screenshot of the destination machine as proof of delivery.
async fn dispatch_send_prompt(
    machine: &str,
    instance: &str,
    conversation: &str,
    prompt: &str,
) -> SendDispatch {
    let local = local_machine_label();
    let is_local = machine.eq_ignore_ascii_case(&local) || machine.eq_ignore_ascii_case("local");
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
    let reply = format!(
        "✅ <b>Prompt sent</b>\n\
        • <b>Machine:</b> <code>{}</code>\n\
        • <b>Instance:</b> <code>{}</code>\n\
        • <b>Conversation:</b> <code>{}</code>\n\n\
        {}\n\n{}",
        clean_for_telegram_html(machine, 48),
        clean_for_telegram_html(instance, 48),
        clean_for_telegram_html(conversation, 64),
        injected,
        proof
    );
    SendDispatch {
        reply,
        markup: None,
    }
}

/// Consume a plain-text message for the `/send` flow when a destination is
/// pending for this chat. Returns `None` for commands, empty text, and when
/// no flow is waiting for typed input — those fall through to normal routing.
pub async fn consume_send_prompt(_token: &str, chat_id: i64, text: &str) -> Option<SendDispatch> {
    let trimmed = text.trim();
    if trimmed.is_empty() || trimmed.starts_with('/') {
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
        } => Some(dispatch_send_prompt(&machine, &instance, &conversation, trimmed).await),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn picker_safety_rejects_separators() {
        assert!(picker_safe("W3"));
        assert!(picker_safe("default"));
        assert!(!picker_safe("a|b"));
        assert!(!picker_safe("a:b"));
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
        assert!(reply.contains("Destination locked"));
    }

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
        let chat_id: i64 = -9_001_337;
        seed_flow(chat_id, SendStage::PickConversation);

        let (reply, markup) = on_conversation_picked(chat_id, "W3", "default", "C001");
        assert!(reply.contains("Destination locked"));
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
        assert!(retry.contains("Unknown send action"));
        drop_flow(chat_id);
    }

    #[test]
    fn send_stage_transition_manual_conversation() {
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
        assert!(unknown.contains("Unknown send action"));
        assert!(none.is_none());
        drop_flow(chat_id);
    }

    #[test]
    fn send_cancel_clears_pending_flows() {
        let chat_id: i64 = -9_001_340;
        seed_flow(chat_id, SendStage::PickInstance);

        let reply = start_send_flow("cancel");
        assert!(reply.contains("cancelled"));
        assert!(reply.contains("Nothing was dispatched"));
        assert!(SEND_FLOWS.lock().unwrap().is_empty());
    }

    #[test]
    fn send_start_lists_machines_destination_first() {
        let reply = start_send_flow("");
        assert!(reply.contains("step 1 of 3"));
        assert!(reply.contains("destination locks"));
    }

    #[tokio::test]
    async fn send_callback_cancel_clears_flow() {
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
        assert!(reply.contains("Unknown send action"));
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
        assert!(machine_entry_usable("W3"));
        assert!(machine_entry_usable("antigravity-node"));
        // Separator characters would corrupt `|`-split parsing.
        assert!(!machine_entry_usable("node|evil"));
        assert!(!machine_entry_usable("node:evil"));
        assert!(!machine_entry_usable(""));
        // `send:m:` is 7 bytes; the alias must leave room under the 64-byte cap.
        assert!(!machine_entry_usable(&"n".repeat(60)));
        assert!(machine_entry_usable(&"n".repeat(50)));
    }
}
