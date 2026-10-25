//! Telegram sent-prompt history (`/history`) and one-tap rerun (`/rerun`).
//!
//! There is no per-instance sent-prompt history table. This surface lists
//! rows from `active_prompts` (via `repo_db::list_all_prompts`) newest-first
//! and offers a rerun button per row that re-dispatches through the FIFO
//! queue (`repo_db::enqueue_prompt_for_instance`).
//!
//! `ActivePrompt` carries no Telegram chat-id attribution. The bot serves a
//! single allowed chat, so history is per-bot rather than per-user — noted
//! here as the accepted scoping for this surface.

use serde_json::{json, Value};

use crate::modules::repo_db::ActivePrompt;

use super::formatting::clean_for_telegram_html;
use super::open_shot::{capture_and_send_proof_shot, local_machine_label, OpenShotTarget};
use super::send_listing::{picker_safe, project_label};

/// Maximum history rows rendered by `/history` (one rerun button per row).
const HISTORY_ROWS: usize = 8;
/// Rerun callbacks carry an id prefix this long: `send:r:<id8>`.
const ID_PREFIX_LEN: usize = 8;
/// `:` is the callback prefix delimiter, `|` the field separator. Prompt
/// text is NEVER embedded in callback payloads (RCA 15); the id prefix is
/// resolved server-side instead.
const RERUN_CALLBACK_PREFIX: &str = "send:r:";
/// Telegram caps `callback_data` at 64 bytes; `send_flow.rs` stays at a
/// 60-byte budget (`MAX_CALLBACK_BYTES`, RCA 15) and so does this module.
const MAX_CALLBACK_BYTES: usize = 60;
const PREVIEW_CHARS: usize = 140;
const META_CHARS: usize = 64;

/// Render the newest sent prompts as HTML plus an inline keyboard carrying
/// one `↻ Rerun` button per row. Buttons hold only a compact `send:r:<id8>`
/// payload; the full prompt is looked up server-side when tapped.
pub fn format_prompt_history(limit: usize) -> (String, Value) {
    let rows = limit.clamp(1, HISTORY_ROWS);
    let prompts = match load_recent_prompts(rows) {
        Ok(prompts) => prompts,
        Err(err) => {
            return (
                db_failure(
                    "Prompt history unavailable",
                    "load telegram prompt history",
                    err,
                ),
                json!({ "inline_keyboard": [] }),
            );
        }
    };
    if prompts.is_empty() {
        return (
            "📜 <b>Sent prompt history is empty.</b>\nPrompts dispatched through this bot will appear here with a one-tap re-run button.".to_string(),
            json!({ "inline_keyboard": [] }),
        );
    }
    let summary_words = super::config::load_config()
        .map(|c| c.prompt_summary_words)
        .unwrap_or(100);
    let mut text = "📜 <b>Sent Prompt History</b> — newest first, tap ↻ to resend\n".to_string();
    let mut keyboard: Vec<Vec<Value>> = Vec::new();
    for (index, prompt) in prompts.iter().enumerate() {
        let id8 = id_prefix(&prompt.id);
        let preview = prompt_preview(prompt, summary_words);
        let block = format!(
            "\n<b>{}.</b> <code>{}</code> <i>{}</i> • <i>{}</i>\n   {}\n   <pre>{}</pre>\n",
            index + 1,
            clean_for_telegram_html(&id8, 16),
            format_sent_at(prompt.created_at),
            clean_for_telegram_html(prompt.status.trim(), 24),
            prompt_destination(prompt),
            preview,
        );
        // Telegram 4000-char boundary guard preserving code block formatting
        if text.len() + block.len() > 3900 {
            text.push_str("\n<i>... (additional history truncated to prevent Telegram size limit rejection)</i>\n");
            break;
        }
        text.push_str(&block);
        // A row whose id cannot round-trip through a callback payload simply
        // gets no button instead of corrupting the keyboard.
        if let Some(payload) = rerun_callback(prompt) {
            keyboard.push(vec![json!({
                "text": format!("↻ Resend {id8}"),
                "callback_data": payload,
            })]);
        }
    }
    (text, json!({ "inline_keyboard": keyboard }))
}

/// Re-dispatch a previously sent prompt by enqueuing into the FIFO queue:
/// routes through `repo_db::enqueue_prompt_for_instance`.
/// `ActivePrompt` has no machine field, so reruns always target
/// the LOCAL instance — remote-node reruns are out of scope.
pub async fn rerun_prompt_by_id(id8: &str) -> (String, Option<Value>) {
    let key = id8.trim();
    if key.is_empty() || !picker_safe(key) {
        return (
            format!(
                "⚠️ <b>Invalid rerun id:</b> <code>{}</code>",
                clean_for_telegram_html(key, 32)
            ),
            None,
        );
    }
    // Check if key is a 1-based index from /history (e.g. "/rerun 3" or "/rerun #3")
    let prompt_opt: Option<ActivePrompt> = if let Some(idx) = key
        .strip_prefix('#')
        .unwrap_or(key)
        .parse::<usize>()
        .ok()
        .filter(|&i| i >= 1)
    {
        match load_recent_prompts(HISTORY_ROWS) {
            Ok(recent) if idx <= recent.len() => Some(recent[idx - 1].clone()),
            _ => None,
        }
    } else {
        None
    };

    let prompt = if let Some(p) = prompt_opt {
        p
    } else {
        let mut matches: Vec<ActivePrompt> = match crate::modules::repo_db::list_all_prompts() {
            Ok(rows) => rows
                .into_iter()
                .filter(|prompt| prompt.id.starts_with(key))
                .collect(),
            Err(err) => {
                return (
                    db_failure("Rerun failed", "load prompts for telegram rerun", err),
                    None,
                );
            }
        };
        // An 8-char prefix of a UUID-style id is effectively unique. If several
        // rows share it (short custom ids), rerun the newest and name it below.
        matches.sort_by(|left, right| {
            right
                .created_at
                .cmp(&left.created_at)
                .then_with(|| right.id.cmp(&left.id))
        });
        let Some(p) = matches.into_iter().next() else {
            return (
                format!(
                    "⚠️ <b>No prompt found</b> for id or row <code>{}</code>. Send <code>/history</code> to pick one.",
                    clean_for_telegram_html(key, 32)
                ),
                None,
            );
        };
        p
    };

    let conversation = rerun_conversation(&prompt);
    let instance = prompt.instance_id.trim();
    let instance = if instance.is_empty() {
        "default"
    } else {
        instance
    };
    let session_opt = prompt
        .session_id
        .as_deref()
        .filter(|s| !s.trim().is_empty());

    let mut enqueued = match crate::modules::repo_db::enqueue_prompt_for_instance(
        instance,
        prompt.repo_path.trim(),
        &prompt.prompt_content,
        session_opt,
    ) {
        Ok(p) => p,
        Err(err) => {
            return (
                format!(
                    "⚠️ <b>Failed to enqueue rerun:</b> <code>{}</code>",
                    clean_for_telegram_html(&err, 200)
                ),
                None,
            );
        }
    };

    // Preserve original image payload if present
    if let Some(ref img) = prompt.image_payload {
        let clean_img = img.trim();
        if !clean_img.is_empty() {
            enqueued.image_payload = Some(clean_img.to_string());
            // Justification: Persisting preserved image payload to SQLite queue is best-effort cache sync
            crate::error::record_ignored(
                crate::modules::repo_db::save_or_requeue_prompt(&enqueued),
                "persist preserved image payload for rerun prompt",
            );
        }
    }

    // Wake background scheduler immediately
    crate::modules::scheduler::notify_prompt_queue();

    // Immediate dispatch check if the instance is currently idle
    // Justification: Idle dispatch check is best-effort; background scheduler picks up prompt if busy
    crate::error::record_ignored(
        crate::modules::repo_db::check_and_dispatch_enqueued_prompts(Some(instance)),
        "check and dispatch enqueued rerun prompt immediately if idle",
    );

    // Send immediate confirmation to Telegram chat
    let (token, chat_id) = match super::config::load_config() {
        Ok(cfg) => (
            cfg.bot_token.trim().to_string(),
            cfg.allowed_chat_id.unwrap_or(0),
        ),
        Err(_) => (String::new(), 0),
    };
    if !token.is_empty() && chat_id != 0 {
        // Justification: immediate rerun confirmation for sub-second user responsiveness
        crate::error::record_ignored(
            super::formatting::send_telegram_message(
                &token,
                chat_id,
                "✅ Prompt enqueued for execution (FIFO queue)!",
            )
            .await,
            "send immediate rerun confirmation",
        );
    }

    // Detach screenshot proof capture into an async background task to eliminate 3-12s blocking wait
    tauri::async_runtime::spawn(async move {
        let proof_res = capture_and_send_proof_shot(&OpenShotTarget::Local).await;
        tracing::info!("Background rerun proof shot completed: {proof_res}");
    });
    let proof = "📸 <i>Delivery proof screenshot is being captured in background...</i>";
    let image_note = enqueued
        .image_payload
        .as_deref()
        .map(str::trim)
        .filter(|payload| !payload.is_empty())
        .map(|_| "\n• <b>Image:</b> <i>original image attachment preserved and queued</i>")
        .unwrap_or("");
    let summary_words = super::config::load_config()
        .map(|c| c.prompt_summary_words)
        .unwrap_or(100);
    let preview = prompt_preview(&enqueued, summary_words);
    let reply = format!(
        "🔁 <b>Prompt Re-Run Enqueued</b>\n\
        • <b>Queue ID:</b> <code>{queue_id}</code>\n\
        • <b>Status:</b> <code>queued</code> (FIFO Order)\n\
        • <b>Machine:</b> <code>{machine}</code>\n\
        • <b>Instance:</b> <code>{instance}</code>\n\
        • <b>Conversation:</b> <code>{conversation}</code>\n\
        • <b>Source prompt:</b> <code>{source}</code> <i>{sent_at}</i>{image_note}\n\n\
        <b>Enqueued Prompt Preview:</b>\n\
        <pre>{preview}</pre>\n\n\
        {proof}",
        queue_id = clean_for_telegram_html(&enqueued.id, 24),
        machine = clean_for_telegram_html(&local_machine_label(), META_CHARS),
        instance = clean_for_telegram_html(instance, META_CHARS),
        conversation = clean_for_telegram_html(&conversation, META_CHARS),
        source = clean_for_telegram_html(&id_prefix(&prompt.id), 16),
        sent_at = format_sent_at(prompt.created_at),
        preview = preview,
        proof = proof,
    );
    (reply, None)
}

/// Load the newest prompts. `list_all_prompts` keeps its FIFO
/// `ORDER BY created_at ASC, id ASC` (AGENTS.md mandates FIFO for DISPATCH);
/// newest-first here is a local sort for display only and never feeds a
/// dispatch query.
fn load_recent_prompts(limit: usize) -> Result<Vec<ActivePrompt>, String> {
    let mut prompts = crate::modules::repo_db::list_all_prompts()?;
    prompts.sort_by(|left, right| {
        right
            .created_at
            .cmp(&left.created_at)
            .then_with(|| right.id.cmp(&left.id))
    });
    prompts.truncate(limit);
    Ok(prompts)
}

fn db_failure(title: &str, context: &str, err: String) -> String {
    // Justification: the DB failure is recorded for diagnostics; the chat
    // still needs an answer, so the failure is rendered inline instead of
    // silently returning an empty list (same pattern as send_listing.rs).
    crate::error::record_ignored(Err::<(), String>(err.clone()), context);
    format!(
        "⚠️ <b>{title}:</b> <code>{}</code>",
        clean_for_telegram_html(&err, 200)
    )
}

fn id_prefix(id: &str) -> String {
    id.chars().take(ID_PREFIX_LEN).collect()
}

/// Compact rerun callback payload for one history row, or `None` when the
/// row's id cannot round-trip through a `send:r:<id8>` payload (separator
/// characters or the 60-byte budget).
fn rerun_callback(prompt: &ActivePrompt) -> Option<String> {
    let id8 = id_prefix(&prompt.id);
    if !picker_safe(&id8) {
        return None;
    }
    let payload = format!("{RERUN_CALLBACK_PREFIX}{id8}");
    if payload.len() > MAX_CALLBACK_BYTES {
        return None;
    }
    Some(payload)
}

pub(crate) fn prompt_preview(prompt: &ActivePrompt, max_words: usize) -> String {
    let stripped = crate::modules::repo_db::strip_internal_id_wrappers(&prompt.prompt_content);
    let clamped = super::formatting::clamp_words(&stripped, max_words);
    clean_for_telegram_html(&clamped, 1000)
}

/// `instance → conversation/project`. (Task-02 owns IDE exe display — the
/// instance id alone is shown here.)
fn prompt_destination(prompt: &ActivePrompt) -> String {
    let conversation = prompt
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|session| !session.is_empty())
        .map(str::to_string)
        .unwrap_or_else(|| {
            project_label(&prompt.repo_path, &prompt.project_id, &prompt.instance_id)
        });
    format!(
        "<code>{}</code> → <code>{}</code>",
        clean_for_telegram_html(prompt.instance_id.trim(), META_CHARS),
        clean_for_telegram_html(&conversation, META_CHARS)
    )
}

/// Target conversation token for the rerun args string. Prefers the stored
/// session id, then the project id, then `"default"`. Tokens carrying
/// callback separators or whitespace would corrupt the args-string parse,
/// so unsafe values fall through to the next option.
fn rerun_conversation(prompt: &ActivePrompt) -> String {
    let session = prompt.session_id.as_deref().map(str::trim).unwrap_or("");
    if !session.is_empty() && picker_safe(session) && !session.chars().any(char::is_whitespace) {
        return session.to_string();
    }
    let project = prompt.project_id.trim();
    if !project.is_empty() && picker_safe(project) && !project.chars().any(char::is_whitespace) {
        return project.to_string();
    }
    "default".to_string()
}

fn format_sent_at(ts: i64) -> String {
    chrono::DateTime::from_timestamp(ts, 0)
        .map(|dt| dt.format("%Y-%m-%d %H:%M UTC").to_string())
        .unwrap_or_else(|| ts.to_string())
}

#[cfg(test)]
mod tests {
    use super::{format_sent_at, id_prefix, rerun_callback, rerun_conversation};
    use crate::modules::repo_db::ActivePrompt;

    fn sample_prompt() -> ActivePrompt {
        ActivePrompt {
            id: "abcdef1234567890".to_string(),
            project_id: "fleet-ui__default".to_string(),
            instance_id: "default".to_string(),
            repo_path: "repos/fleet-ui".to_string(),
            prompt_content: "  \nFix the build\nsecond line".to_string(),
            model: None,
            session_id: Some("sess-1".to_string()),
            status: "dispatched".to_string(),
            created_at: 1_760_000_000,
            updated_at: 1_760_000_000,
            image_payload: None,
        }
    }

    #[test]
    fn id_prefix_takes_eight_chars() {
        assert_eq!(id_prefix("abcdef1234567890"), "abcdef12");
    }

    #[test]
    fn rerun_callback_carries_compact_id() {
        let payload = rerun_callback(&sample_prompt()).expect("button payload");
        assert_eq!(payload, "send:r:abcdef12");
        assert!(payload.len() <= 60);
    }

    #[test]
    fn rerun_conversation_prefers_session_id() {
        assert_eq!(rerun_conversation(&sample_prompt()), "sess-1");
    }

    #[test]
    fn format_sent_at_renders_utc() {
        assert_eq!(format_sent_at(1_760_000_000), "2025-10-09 08:53 UTC");
    }

    #[test]
    fn prompt_preview_strips_internal_ids_and_clamps() {
        let mut p = sample_prompt();
        p.prompt_content =
            "Id: sess-abc\nFirst word second word third word fourth word".to_string();
        let preview = super::prompt_preview(&p, 3);
        assert!(!preview.contains("Id:"));
        assert!(preview.contains("First word second..."));
    }
}
