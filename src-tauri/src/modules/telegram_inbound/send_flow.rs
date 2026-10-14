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
use std::collections::{HashMap, HashSet};
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::open_shot::{local_machine_label, OpenShotTarget};
use super::*;

const SEND_FLOW_TTL_SECS: u64 = 600;
const MAX_PICK_BUTTONS: usize = 8;
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

fn callback_fits(text: &str) -> bool {
    text.len() <= MAX_CALLBACK_BYTES
}

/// Fleet machines the picker can address: this machine plus GitMap cluster nodes.
fn machine_entries() -> Vec<String> {
    let local = local_machine_label();
    let mut entries = vec![local.clone()];
    for node in query_gitmap_cluster_nodes() {
        let alias = node.alias.trim();
        if picker_safe(alias)
            && !entries.iter().any(|e| e.eq_ignore_ascii_case(alias))
            && callback_fits(&format!("send:m:{alias}"))
        {
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

/// (value, label) conversation targets for an instance, from live tracked prompts.
fn conversation_entries(instance_id: &str) -> Vec<(String, String)> {
    let prompts = match crate::modules::repo_db::list_all_prompts() {
        Ok(p) => p,
        // Justification: an unreadable prompt DB only empties the picker; the
        // manual-entry path stays available and the error is logged.
        Err(e) => {
            crate::error::record_ignored(
                Err::<(), String>(e.to_string()),
                "list prompts for /send conversation picker",
            );
            return Vec::new();
        }
    };
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for p in prompts.iter().filter(|p| p.instance_id == instance_id) {
        let value = p
            .session_id
            .clone()
            .filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| p.project_id.clone());
        if !picker_safe(&value) || !seen.insert(value.clone()) {
            continue;
        }
        let label = match &p.session_id {
            Some(s) if !s.trim().is_empty() => {
                let short: String = s.chars().take(8).collect();
                format!("{} · {}", p.project_id, short)
            }
            _ => p.project_id.clone(),
        };
        if !callback_fits(&format!("send:c:x|x|{value}")) {
            continue;
        }
        out.push((value, label));
        if out.len() >= MAX_PICK_BUTTONS {
            break;
        }
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
            json!({
                "text": format!("🖥️ {}", alias),
                "callback_data": format!("send:m:{alias}"),
            })
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

fn conversation_picker_markup(machine: &str, instance: &str, convs: &[(String, String)]) -> Value {
    let mut rows: Vec<Value> = Vec::new();
    let buttons: Vec<Value> = convs
        .iter()
        .filter(|(value, _)| callback_fits(&format!("send:c:{machine}|{instance}|{value}")))
        .map(|(value, label)| {
            json!({
                "text": format!("💬 {}", label),
                "callback_data": format!("send:c:{machine}|{instance}|{value}"),
            })
        })
        .collect();
    push_buttons(&mut rows, buttons, 2);
    rows.push(json!([
        { "text": "✏️ Type it manually", "callback_data": format!("send:cman:{machine}|{instance}") },
        { "text": "🧹 Cancel", "callback_data": "send:cancel" }
    ]));
    json!({ "inline_keyboard": rows })
}

fn destination_locked_reply(machine: &str, instance: &str, conversation: &str) -> String {
    format!(
        "🎯 <b>Destination locked</b>\n\
        • <b>Machine:</b> <code>{}</code>\n\
        • <b>Instance:</b> <code>{}</code>\n\
        • <b>Conversation:</b> <code>{}</code>\n\n\
        ✍️ Now send the prompt text as a plain message. It will go to exactly this destination — nothing is guessed.",
        clean_for_telegram_html(machine, 48),
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
    let mut out = String::from(
        "📮 <b>Send a prompt — step 1 of 3: pick the destination machine</b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
    );
    for (idx, alias) in entries.iter().enumerate() {
        let marker = if idx == 0 {
            "📍 <i>this machine</i>"
        } else {
            "🌐 <i>fleet node</i>"
        };
        out.push_str(&format!(
            "{}. 🖥️ <code>{}</code> {}\n",
            idx + 1,
            clean_for_telegram_html(alias, 48),
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
                updated_at: now_secs(),
            },
        );
    }
    let instances = instance_entries();
    let mut out = format!(
        "📮 <b>Send a prompt — step 2 of 3: pick the instance on <code>{}</code></b>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        clean_for_telegram_html(&canonical, 48)
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
                updated_at: now_secs(),
            },
        );
    }
    let convs = conversation_entries(&canonical_instance);
    let mut out = format!(
        "📮 <b>Send a prompt — step 3 of 3: pick the conversation</b>\n\
        • <b>Machine:</b> <code>{}</code>\n\
        • <b>Instance:</b> <code>{}</code>\n\
        ━━━━━━━━━━━━━━━━━━━━━━━━\n",
        clean_for_telegram_html(&canonical_machine, 48),
        clean_for_telegram_html(&canonical_instance, 48)
    );
    if convs.is_empty() {
        out.push_str(
            "No live conversations were found for this instance. \
            Tap <b>✏️ Type it manually</b> and reply with the conversation Seq ID \
            (e.g. <code>C001</code>), project name, or conversation id.",
        );
    } else {
        for (idx, (_, label)) in convs.iter().enumerate() {
            out.push_str(&format!(
                "{}. 💬 <code>{}</code>\n",
                idx + 1,
                clean_for_telegram_html(label, 64)
            ));
        }
        out.push_str("\n👆 Tap a conversation, or type the Seq ID manually.");
    }
    (
        out,
        Some(conversation_picker_markup(
            &canonical_machine,
            &canonical_instance,
            &convs,
        )),
    )
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
    if let Some(rest) = payload.strip_prefix("cman:") {
        return match rest.split_once('|') {
            Some((machine, instance)) => on_manual_conversation(chat_id, machine, instance),
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
    }
    let next = {
        let mut flows = SEND_FLOWS.lock().ok()?;
        let flow = flows.get_mut(&chat_id)?;
        if flow_expired(flow) {
            flows.remove(&chat_id);
            return None;
        }
        match flow.stage {
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
        let reply = destination_locked_reply("W3", "default", "C001");
        assert!(reply.contains("W3"));
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
        assert_eq!("send:cancel".strip_prefix("send:").unwrap(), "cancel");
    }
}
