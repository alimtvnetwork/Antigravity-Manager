//! Per-instance Telegram status cards: compact running/queued prompts and
//! projects, native copy (256-char Bot API cap), full-prompt documents, rerun,
//! automation enqueue, fast-forward, and screenshot callbacks.
//!
//! Callback payloads stay inside the 60-byte AGM budget (`i:<op>:<id8>`).
//! Full prompt text is never embedded in `callback_data` or in `copy_text`
//! beyond `TELEGRAM_COPY_TEXT_MAX`.

use serde_json::{json, Value};

use crate::modules::email_watcher;
use crate::modules::instance;
use crate::modules::repo_db::{self, ActivePrompt};
use crate::modules::supabase_sync;

use super::formatting::{
    clamp_telegram_copy_text, clean_for_telegram_html, TELEGRAM_COPY_TEXT_MAX,
};
use super::media::send_telegram_document;
use super::open_shot;
use super::send_telegram_message_with_markup;

fn format_running_duration(created_at: i64) -> String {
    if created_at <= 0 {
        return String::new();
    }
    let now = chrono::Utc::now().timestamp();
    let secs = (now - created_at).max(0);
    if secs < 60 {
        format!("(running {secs}s)")
    } else {
        let mins = secs / 60;
        let rem = secs % 60;
        format!("(running {mins}m {rem}s)")
    }
}

fn shorten_project_name(name: &str) -> String {
    let lower = name.to_lowercase();
    if lower.contains("antigravity") && lower.contains("manager") {
        return "AGM".to_string();
    }
    if lower.contains("gitmap") {
        return "GitMap".to_string();
    }
    if let Some((stem, _)) = name.rsplit_once('-') {
        if stem.len() >= 3 && name.len() - stem.len() >= 6 {
            return stem.to_string();
        }
    }
    if name.chars().count() > 28 {
        return format!("{}…", name.chars().take(27).collect::<String>());
    }
    name.to_string()
}

const MAX_CALLBACK_BYTES: usize = 60;
const PREVIEW_WORDS: usize = 100;
const MAX_PROMPTS_SHOWN: usize = 3;
const AUTOMATION_DONE: &str = "Is it done?";
const AUTOMATION_RELEASED: &str = "Is it released?";
const AUTOMATION_RANDOM: &[&str] = &[
    "Please continue from where you left off and report status.",
    "Verify the last change compiled and summarize what shipped.",
    "If the task is complete, confirm it is released; otherwise continue.",
];

#[derive(Debug, Clone)]
pub struct PromptSnippet {
    pub id: String,
    pub preview: String,
    pub word_count: usize,
    pub status: String,
    pub project: String,
    pub created_at: i64,
}

#[derive(Debug, Clone)]
pub struct InstanceCard {
    pub instance_id: String,
    pub instance_name: String,
    pub running: bool,
    pub bound_email: String,
    pub pid: String,
    pub is_active: bool,
    pub running_prompts: Vec<PromptSnippet>,
    pub queued_prompts: Vec<PromptSnippet>,
    pub running_projects: Vec<String>,
    pub queued_projects: Vec<String>,
}

fn id8(id: &str) -> String {
    let t = id.trim();
    if t.chars().count() <= 8 {
        t.to_string()
    } else {
        t.chars().take(8).collect()
    }
}

fn is_running_status(status: &str) -> bool {
    matches!(
        status,
        "running" | "dispatched" | "executing" | "RUNNING" | "DISPATCHED" | "EXECUTING"
    )
}

fn is_queued_status(status: &str) -> bool {
    matches!(status, "queued" | "pending" | "QUEUED" | "PENDING")
}

fn snippet_from_prompt(p: &ActivePrompt) -> PromptSnippet {
    let (preview, wc) = repo_db::extract_prompt_words_preview(&p.prompt_content, PREVIEW_WORDS);
    let friendly = repo_db::format_friendly_workspace_label(&p.project_id, "", &p.repo_path);
    PromptSnippet {
        id: p.id.clone(),
        preview,
        word_count: wc,
        status: p.status.clone(),
        project: shorten_project_name(&friendly),
        created_at: p.created_at,
    }
}

/// Resolve an instance id/name prefix uniquely against the registry.
pub fn resolve_instance_prefix(prefix: &str) -> Option<String> {
    let key = prefix.trim();
    if key.is_empty() {
        return None;
    }
    let reg = instance::load_registry().unwrap_or_default();
    if key.eq_ignore_ascii_case("default") {
        if let Some(found) = reg
            .instances
            .iter()
            .find(|i| i.is_default || i.id == "default")
        {
            return Some(found.id.clone());
        }
        return Some("default".to_string());
    }
    if let Some(found) = reg
        .instances
        .iter()
        .find(|i| i.id.eq_ignore_ascii_case(key) || i.name.eq_ignore_ascii_case(key))
    {
        return Some(found.id.clone());
    }
    let matches: Vec<&instance::InstanceConfig> = reg
        .instances
        .iter()
        .filter(|i| {
            i.id.starts_with(key)
                || i.name.starts_with(key)
                || i.id.to_lowercase().starts_with(&key.to_lowercase())
        })
        .collect();
    if matches.len() == 1 {
        return Some(matches[0].id.clone());
    }
    None
}

fn prompt_belongs_to_instance(prompt: &ActivePrompt, instance_id: &str, is_default: bool) -> bool {
    let pid = prompt.instance_id.trim();
    if pid.eq_ignore_ascii_case(instance_id) {
        return true;
    }
    if is_default && (pid.is_empty() || pid.eq_ignore_ascii_case("default") || pid == "__default__")
    {
        return true;
    }
    false
}

/// Collect one card per registered instance (or a synthetic default).
pub fn collect_instance_cards() -> Vec<InstanceCard> {
    let reg = instance::load_registry().unwrap_or_default();
    let prompts = repo_db::list_all_prompts().unwrap_or_default();
    let mut cards = Vec::new();

    let instances: Vec<instance::InstanceConfig> = if reg.instances.is_empty() {
        vec![instance::InstanceConfig {
            id: "default".to_string(),
            name: "Default".to_string(),
            data_dir: String::new(),
            executable_path: None,
            extensions_dir: None,
            bound_account_id: None,
            bound_email: None,
            created_at: 0,
            last_used: 0,
            is_default: true,
            pid: None,
            seq_num: Some(1),
        }]
    } else {
        reg.instances.clone()
    };

    for inst in &instances {
        let pids = instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default);
        let running =
            !pids.is_empty() || instance::is_instance_running(&inst.id, &inst.data_dir, inst.pid);
        let pid_str = if pids.is_empty() {
            inst.pid
                .map(|p| p.to_string())
                .unwrap_or_else(|| "-".to_string())
        } else {
            pids.iter()
                .map(|p| p.to_string())
                .collect::<Vec<_>>()
                .join(",")
        };
        let bound = inst
            .bound_email
            .clone()
            .unwrap_or_else(|| "(none)".to_string());

        let mut owned: Vec<&ActivePrompt> = prompts
            .iter()
            .filter(|p| prompt_belongs_to_instance(p, &inst.id, inst.is_default))
            .collect();
        owned.sort_by(|a, b| {
            a.created_at
                .cmp(&b.created_at)
                .then_with(|| a.id.cmp(&b.id))
        });

        let running_prompts: Vec<PromptSnippet> = owned
            .iter()
            .filter(|p| is_running_status(&p.status))
            .rev()
            .take(MAX_PROMPTS_SHOWN)
            .map(|p| snippet_from_prompt(p))
            .collect();
        let queued_prompts: Vec<PromptSnippet> = owned
            .iter()
            .filter(|p| is_queued_status(&p.status))
            .take(MAX_PROMPTS_SHOWN)
            .map(|p| snippet_from_prompt(p))
            .collect();

        let mut running_projects: Vec<String> = running_prompts
            .iter()
            .map(|s| s.project.clone())
            .filter(|s| !s.is_empty())
            .collect();
        running_projects.sort();
        running_projects.dedup();
        let mut queued_projects: Vec<String> = queued_prompts
            .iter()
            .map(|s| s.project.clone())
            .filter(|s| !s.is_empty())
            .collect();
        queued_projects.sort();
        queued_projects.dedup();

        cards.push(InstanceCard {
            instance_id: inst.id.clone(),
            instance_name: inst.name.clone(),
            running,
            bound_email: bound,
            pid: pid_str,
            is_active: reg.active_instance_id == inst.id,
            running_prompts,
            queued_prompts,
            running_projects,
            queued_projects,
        });
    }
    cards
}

fn format_prompt_block(snips: &[PromptSnippet], empty: &str) -> String {
    if snips.is_empty() {
        return format!("   <i>{}</i>\n", empty);
    }
    let mut out = String::new();
    for s in snips {
        let dur = format_running_duration(s.created_at);
        let dur_clean = dur
            .strip_prefix("(running ")
            .and_then(|x| x.strip_suffix(')'))
            .unwrap_or(dur.as_str());
        let id = id8(&s.id);
        out.push_str(&format!(
            "   <code>{}</code> · {} · {}w {}\n   <blockquote expandable>{}</blockquote>\n",
            clean_for_telegram_html(&id, 12),
            clean_for_telegram_html(&s.project, 28),
            s.word_count,
            clean_for_telegram_html(dur_clean, 24),
            clean_for_telegram_html(&s.preview, 700)
        ));
    }
    out
}

pub fn format_instance_card_html(card: &InstanceCard) -> String {
    let dot = if card.running { "🟢" } else { "⚪" };
    let active = if card.is_active {
        " · <i>ACTIVE</i>"
    } else {
        ""
    };
    let run_proj = if card.running_projects.is_empty() {
        "(none)".to_string()
    } else {
        card.running_projects.join(", ")
    };
    let q_proj = if card.queued_projects.is_empty() {
        "(none)".to_string()
    } else {
        card.queued_projects.join(", ")
    };
    format!(
        "{dot} <b>{}</b> (<code>{}</code>){active}\n\
         👤 <code>{}</code> · PID <code>{}</code>\n\
         🏃 Running projects: <b>{}</b>\n\
         📥 Queued projects: <b>{}</b>\n\
         ⚡ Running prompts ({})\n{}\
         ⏳ Queued prompts ({})\n{}",
        clean_for_telegram_html(&card.instance_name, 40),
        clean_for_telegram_html(&card.instance_id, 40),
        clean_for_telegram_html(&card.bound_email, 48),
        clean_for_telegram_html(&card.pid, 32),
        clean_for_telegram_html(&run_proj, 80),
        clean_for_telegram_html(&q_proj, 80),
        card.running_prompts.len(),
        format_prompt_block(&card.running_prompts, "none running"),
        card.queued_prompts.len(),
        format_prompt_block(&card.queued_prompts, "queue empty"),
    )
}

fn cb(op: &str, id: &str) -> Option<String> {
    let payload = format!("i:{}:{}", op, id8(id));
    if payload.len() > MAX_CALLBACK_BYTES {
        return None;
    }
    Some(payload)
}

fn copy_source_prompt(card: &InstanceCard) -> Option<&PromptSnippet> {
    card.running_prompts
        .first()
        .or_else(|| card.queued_prompts.first())
}

/// Lookup the full prompt text for a card (running first, else queued).
fn full_prompt_text(card: &InstanceCard) -> String {
    let Some(snip) = copy_source_prompt(card) else {
        return String::new();
    };
    if let Ok(all) = repo_db::list_all_prompts() {
        if let Some(p) = all
            .iter()
            .find(|p| p.id == snip.id || p.id.starts_with(&snip.id))
        {
            return p.prompt_content.clone();
        }
    }
    snip.preview.clone()
}

pub fn build_instance_card_markup(card: &InstanceCard) -> Value {
    let mut rows: Vec<Vec<Value>> = Vec::new();
    let full = full_prompt_text(card);
    let copy_preview = clamp_telegram_copy_text(&full, TELEGRAM_COPY_TEXT_MAX);
    let mut copy_row = Vec::new();
    if !copy_preview.trim().is_empty() {
        copy_row.push(json!({
            "text": "📋 Copy",
            "copy_text": { "text": copy_preview }
        }));
    }
    if let Some(snip) = copy_source_prompt(card) {
        if let Some(payload) = cb("cp", &snip.id) {
            copy_row.push(json!({ "text": "📄 Full", "callback_data": payload }));
        }
    }
    if let Some(payload) = cb("rr", &card.instance_id) {
        copy_row.push(json!({ "text": "↻ Rerun", "callback_data": payload }));
    }
    if !copy_row.is_empty() {
        rows.push(copy_row);
    }

    let mut auto_row = Vec::new();
    if let Some(payload) = cb("ad", &card.instance_id) {
        auto_row.push(json!({ "text": "✅ Done?", "callback_data": payload }));
    }
    if let Some(payload) = cb("ap", &card.instance_id) {
        auto_row.push(json!({ "text": "🚀 Released?", "callback_data": payload }));
    }
    if let Some(payload) = cb("ax", &card.instance_id) {
        auto_row.push(json!({ "text": "🎲 Auto", "callback_data": payload }));
    }
    if !auto_row.is_empty() {
        rows.push(auto_row);
    }

    let mut act_row = Vec::new();
    if let Some(payload) = cb("ff", &card.instance_id) {
        act_row.push(json!({ "text": "⏩ FF", "callback_data": payload }));
    }
    if let Some(payload) = cb("sc", &card.instance_id) {
        act_row.push(json!({ "text": "📸 Shot", "callback_data": payload }));
    }
    if let Some(payload) = cb("st", &card.instance_id) {
        act_row.push(json!({ "text": "🔄 Refresh", "callback_data": payload }));
    }
    if !act_row.is_empty() {
        rows.push(act_row);
    }

    json!({ "inline_keyboard": rows })
}

pub fn format_status_header() -> (String, Value) {
    let machine = email_watcher::detect_machine_name();
    let ip = email_watcher::detect_local_ip();
    let alias = supabase_sync::load_config()
        .ok()
        .map(|c| c.node_alias)
        .unwrap_or_default();
    let cards = collect_instance_cards();
    let running_n = cards.iter().filter(|c| c.running).count();
    let run_p: usize = cards.iter().map(|c| c.running_prompts.len()).sum();
    let q_p: usize = cards.iter().map(|c| c.queued_prompts.len()).sum();
    let html = format!(
        "📊 <b>Instance status</b> · <code>{}</code> (<code>{}</code>)\n\
         🏷️ Node <code>{}</code> · {}/{} instances running\n\
         ⚡ {} running prompt(s) · ⏳ {} queued\n\
         <i>One card per instance follows. Copy is a 256-char preview; Full sends the whole prompt.</i>",
        clean_for_telegram_html(&machine, 40),
        clean_for_telegram_html(&ip, 40),
        clean_for_telegram_html(if alias.trim().is_empty() { "—" } else { alias.trim() }, 40),
        running_n,
        cards.len(),
        run_p,
        q_p
    );
    let markup = json!({
        "inline_keyboard": [
            [
                { "text": "💰 Balance", "callback_data": "cmd:/balance" },
                { "text": "📸 All shots", "callback_data": "cmd:/screenshot" }
            ],
            [
                { "text": "📥 Instances CSV", "callback_data": "cmd:/export instances csv" },
                { "text": "📥 Prompts CSV", "callback_data": "cmd:/export prompts csv" }
            ],
            [
                { "text": "🔄 Refresh all", "callback_data": "cmd:/status" }
            ]
        ]
    });
    (html, markup)
}

fn is_status_command(text: &str) -> bool {
    let t = text.trim();
    let (cmd, _) = match t.split_once(char::is_whitespace) {
        Some((c, r)) => (c, r),
        None => (t, ""),
    };
    let base = cmd
        .strip_prefix('/')
        .unwrap_or(cmd)
        .split('@')
        .next()
        .unwrap_or(cmd)
        .to_lowercase();
    matches!(base.as_str(), "status" | "observe")
}

pub fn should_fanout_status(text: &str) -> bool {
    is_status_command(text)
}

/// Send fleet header + one message per instance. Returns the header text.
pub async fn deliver_instance_status_fanout(token: &str, chat_id: i64) -> Option<String> {
    let (header, header_markup) = format_status_header();
    crate::error::record_ignored(
        send_telegram_message_with_markup(token, chat_id, &header, Some(header_markup), false)
            .await,
        "send instance status header",
    );
    let cards = collect_instance_cards();
    for card in &cards {
        let html = format_instance_card_html(card);
        let markup = build_instance_card_markup(card);
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &html, Some(markup), false).await,
            "send instance status card",
        );
        tokio::time::sleep(std::time::Duration::from_millis(80)).await;
    }
    Some(header)
}

fn latest_prompt_for_instance(instance_id: &str) -> Option<ActivePrompt> {
    let all = repo_db::list_all_prompts().ok()?;
    let mut owned: Vec<ActivePrompt> = all
        .into_iter()
        .filter(|p| {
            p.instance_id.trim().eq_ignore_ascii_case(instance_id)
                || (instance_id == "default"
                    && (p.instance_id.trim().is_empty()
                        || p.instance_id.trim().eq_ignore_ascii_case("default")))
        })
        .collect();
    owned.sort_by(|a, b| {
        b.created_at
            .cmp(&a.created_at)
            .then_with(|| b.id.cmp(&a.id))
    });
    owned.into_iter().next()
}

fn enqueue_text(
    instance_id: &str,
    repo_path: &str,
    session: Option<&str>,
    text: &str,
) -> Result<ActivePrompt, String> {
    repo_db::enqueue_prompt_for_instance(instance_id, repo_path, text, session)
}

fn random_followup() -> &'static str {
    let idx = (std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0)
        % AUTOMATION_RANDOM.len() as u128) as usize;
    AUTOMATION_RANDOM[idx]
}

async fn send_card_refresh(token: &str, chat_id: i64, instance_id: &str) {
    let cards = collect_instance_cards();
    if let Some(card) = cards.iter().find(|c| c.instance_id == instance_id) {
        let html = format_instance_card_html(card);
        let markup = build_instance_card_markup(card);
        crate::error::record_ignored(
            send_telegram_message_with_markup(token, chat_id, &html, Some(markup), false).await,
            "refresh instance status card",
        );
    }
}

async fn send_full_prompt_document(token: &str, chat_id: i64, pid8: &str) -> String {
    let all = match repo_db::list_all_prompts() {
        Ok(v) => v,
        Err(e) => {
            return format!(
                "❌ Could not load prompts: <code>{}</code>",
                clean_for_telegram_html(&e, 160)
            );
        }
    };
    let Some(p) = all
        .iter()
        .find(|p| p.id.starts_with(pid8) || id8(&p.id) == pid8)
    else {
        return format!(
            "⚠️ No prompt matching <code>{}</code>.",
            clean_for_telegram_html(pid8, 16)
        );
    };
    let file_name = format!("agm-prompt-{}.txt", id8(&p.id));
    let caption = format!(
        "📄 Full prompt <code>{}</code> · {} · {} chars",
        clean_for_telegram_html(&id8(&p.id), 12),
        clean_for_telegram_html(&p.status, 16),
        p.prompt_content.chars().count()
    );
    match send_telegram_document(
        token,
        chat_id,
        &file_name,
        p.prompt_content.as_bytes(),
        &caption,
        None,
    )
    .await
    {
        Ok(()) => caption,
        Err(e) => format!(
            "{}\n⚠️ Document upload failed: <code>{}</code>\n<blockquote expandable>{}</blockquote>",
            caption,
            clean_for_telegram_html(&e, 160),
            clean_for_telegram_html(&p.prompt_content, 3500)
        ),
    }
}

/// Handle `i:<op>:<id8>` callbacks. Returns true when consumed.
pub async fn handle_instance_callback(token: &str, chat_id: i64, data: &str) -> bool {
    let rest = match data.strip_prefix("i:") {
        Some(r) => r,
        None => return false,
    };
    let (op, id) = match rest.split_once(':') {
        Some((o, i)) => (o.trim(), i.trim()),
        None => return false,
    };
    if op.is_empty() || id.is_empty() {
        return false;
    }

    match op {
        "st" => {
            if let Some(inst) = resolve_instance_prefix(id) {
                send_card_refresh(token, chat_id, &inst).await;
            } else {
                crate::error::record_ignored(
                    send_telegram_message_with_markup(
                        token,
                        chat_id,
                        &format!(
                            "⚠️ Unknown instance prefix <code>{}</code>.",
                            clean_for_telegram_html(id, 24)
                        ),
                        None,
                        false,
                    )
                    .await,
                    "unknown instance prefix",
                );
            }
        }
        "cp" => {
            let reply = send_full_prompt_document(token, chat_id, id).await;
            crate::error::record_ignored(
                send_telegram_message_with_markup(token, chat_id, &reply, None, false).await,
                "send full prompt ack",
            );
        }
        "rr" => {
            let reply = match resolve_instance_prefix(id) {
                Some(inst) => rerun_latest(&inst).await,
                None => format!(
                    "⚠️ Unknown instance <code>{}</code>.",
                    clean_for_telegram_html(id, 24)
                ),
            };
            crate::error::record_ignored(
                send_telegram_message_with_markup(token, chat_id, &reply, None, false).await,
                "send rerun ack",
            );
        }
        "ad" | "ap" | "ax" => {
            let reply = match resolve_instance_prefix(id) {
                Some(inst) => enqueue_automation(&inst, op).await,
                None => format!(
                    "⚠️ Unknown instance <code>{}</code>.",
                    clean_for_telegram_html(id, 24)
                ),
            };
            crate::error::record_ignored(
                send_telegram_message_with_markup(token, chat_id, &reply, None, false).await,
                "send automation ack",
            );
        }
        "ff" => {
            let reply = match resolve_instance_prefix(id) {
                Some(inst) => {
                    match crate::modules::auto_switcher::trigger_manual_rotation_for_instance(Some(
                        &inst,
                    ))
                    .await
                    {
                        Ok(msg) => format!(
                            "⏩ <b>Fast-forward</b> <code>{}</code>\n{}",
                            clean_for_telegram_html(&inst, 40),
                            clean_for_telegram_html(&msg, 800)
                        ),
                        Err(e) => format!(
                            "❌ Fast-forward failed on <code>{}</code>: <code>{}</code>",
                            clean_for_telegram_html(&inst, 40),
                            clean_for_telegram_html(&e, 200)
                        ),
                    }
                }
                None => format!(
                    "⚠️ Unknown instance <code>{}</code>.",
                    clean_for_telegram_html(id, 24)
                ),
            };
            crate::error::record_ignored(
                send_telegram_message_with_markup(token, chat_id, &reply, None, false).await,
                "send ff ack",
            );
        }
        "sc" => {
            let reply = match resolve_instance_prefix(id) {
                Some(inst) => open_shot::execute_instance_screenshot(&inst).await,
                None => open_shot::execute_screenshot_command("").await,
            };
            crate::error::record_ignored(
                send_telegram_message_with_markup(token, chat_id, &reply, None, false).await,
                "send screenshot ack",
            );
        }
        _ => return false,
    }
    true
}

async fn rerun_latest(instance_id: &str) -> String {
    let Some(p) = latest_prompt_for_instance(instance_id) else {
        return format!(
            "⚠️ No prompt history on <code>{}</code> to rerun.",
            clean_for_telegram_html(instance_id, 40)
        );
    };
    let session = p.session_id.as_deref().filter(|s| !s.trim().is_empty());
    match enqueue_text(instance_id, p.repo_path.trim(), session, &p.prompt_content) {
        Ok(enqueued) => format!(
            "↻ Reran <code>{}</code> on <code>{}</code> → queued <code>{}</code>",
            clean_for_telegram_html(&id8(&p.id), 12),
            clean_for_telegram_html(instance_id, 40),
            clean_for_telegram_html(&id8(&enqueued.id), 12)
        ),
        Err(e) => format!(
            "❌ Rerun failed: <code>{}</code>",
            clean_for_telegram_html(&e, 200)
        ),
    }
}

async fn enqueue_automation(instance_id: &str, op: &str) -> String {
    let Some(p) = latest_prompt_for_instance(instance_id) else {
        return format!(
            "⚠️ No conversation on <code>{}</code> to attach automation.",
            clean_for_telegram_html(instance_id, 40)
        );
    };
    let session = p.session_id.as_deref().filter(|s| !s.trim().is_empty());
    let repo = p.repo_path.trim();
    let mut lines = Vec::new();
    let texts: Vec<&str> = match op {
        "ad" => vec![AUTOMATION_DONE],
        "ap" => vec![AUTOMATION_RELEASED],
        "ax" => vec![AUTOMATION_DONE, AUTOMATION_RELEASED, random_followup()],
        _ => vec![AUTOMATION_DONE],
    };
    for t in texts {
        match enqueue_text(instance_id, repo, session, t) {
            Ok(enqueued) => lines.push(format!(
                "• queued <code>{}</code> — {}",
                clean_for_telegram_html(&id8(&enqueued.id), 12),
                clean_for_telegram_html(t, 80)
            )),
            Err(e) => lines.push(format!(
                "• failed {}: <code>{}</code>",
                clean_for_telegram_html(t, 40),
                clean_for_telegram_html(&e, 120)
            )),
        }
    }
    format!(
        "🤖 Automation on <code>{}</code>\n{}",
        clean_for_telegram_html(instance_id, 40),
        lines.join("\n")
    )
}

/// Parse `i:op:id` without executing — used by unit tests.
pub fn parse_instance_callback(data: &str) -> Option<(String, String)> {
    let rest = data.strip_prefix("i:")?;
    let (op, id) = rest.split_once(':')?;
    if op.is_empty() || id.is_empty() {
        return None;
    }
    Some((op.to_string(), id.to_string()))
}

pub fn instance_callback_within_budget(op: &str, id: &str) -> bool {
    cb(op, id).is_some()
}

/// Counts used by instances CSV export.
pub fn instance_prompt_counts(instance_id: &str, is_default: bool) -> (usize, usize, usize, usize) {
    let prompts = repo_db::list_all_prompts().unwrap_or_default();
    let owned: Vec<&ActivePrompt> = prompts
        .iter()
        .filter(|p| prompt_belongs_to_instance(p, instance_id, is_default))
        .collect();
    let run_p = owned
        .iter()
        .filter(|p| is_running_status(&p.status))
        .count();
    let q_p = owned.iter().filter(|p| is_queued_status(&p.status)).count();
    let mut run_proj: Vec<String> = owned
        .iter()
        .filter(|p| is_running_status(&p.status))
        .map(|p| p.project_id.clone())
        .collect();
    run_proj.sort();
    run_proj.dedup();
    let mut q_proj: Vec<String> = owned
        .iter()
        .filter(|p| is_queued_status(&p.status))
        .map(|p| p.project_id.clone())
        .collect();
    q_proj.sort();
    q_proj.dedup();
    (run_p, q_p, run_proj.len(), q_proj.len())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn callback_payloads_fit_budget() {
        assert!(instance_callback_within_budget("ff", "default"));
        assert!(instance_callback_within_budget("rr", "v2-3127"));
        assert!(instance_callback_within_budget("cp", "p-239ec5"));
        assert!(instance_callback_within_budget("ad", "v3-7391"));
        let payload = cb("ff", "default").expect("payload");
        assert!(payload.len() <= MAX_CALLBACK_BYTES);
        assert_eq!(payload, "i:ff:default");
    }

    #[test]
    fn parse_instance_callback_roundtrip() {
        assert_eq!(
            parse_instance_callback("i:rr:v2-3127"),
            Some(("rr".to_string(), "v2-3127".to_string()))
        );
        assert!(parse_instance_callback("cmd:/status").is_none());
        assert!(parse_instance_callback("i:").is_none());
    }

    #[test]
    fn copy_text_never_exceeds_telegram_cap() {
        let long = "word ".repeat(400);
        let clamped = clamp_telegram_copy_text(&long, TELEGRAM_COPY_TEXT_MAX);
        assert!(clamped.chars().count() <= TELEGRAM_COPY_TEXT_MAX);
    }

    #[test]
    fn id8_truncates() {
        assert_eq!(id8("default"), "default");
        assert_eq!(id8("abcdefghij"), "abcdefgh");
    }
}
