//! Repo DB: tree text

use super::text_utils::format_friendly_workspace_label;
use super::tree::get_project_conversation_tree;
use base64::Engine;

/// Format the Project -> Conversation -> 200-Word Prompt Tree View for AGM CLI output
pub fn format_tree_view_cli(max_words: usize, only_running: bool) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree(word_cap, only_running);
    let mut out = String::new();

    let mode_label = if only_running {
        "RUNNING PROJECTS & CONVERSATIONS"
    } else {
        "ALL PROJECTS & CONVERSATIONS"
    };
    out.push_str(&format!(
        "🌳 AGM + GITMAP PROJECT → CONVERSATION → [≤{}w PROMPT] TREE ({})\n",
        word_cap, mode_label
    ));
    out.push_str(&"━".repeat(86));
    out.push('\n');

    if tree.is_empty() {
        out.push_str("  (No matching projects or active conversations detected)\n");
        return out;
    }

    for proj in &tree {
        let proj_badge = if proj.is_running { "🟢" } else { "⚪" };
        let label =
            format_friendly_workspace_label(&proj.project_id, &proj.repo_name, &proj.repo_path);
        let inst_seq_str = proj
            .instance_seq_num
            .map(|n| format!("#{} ", n))
            .unwrap_or_default();
        let email_str = proj
            .bound_email
            .as_deref()
            .filter(|e| !e.is_empty())
            .map(|e| format!(" ({})", e))
            .unwrap_or_default();

        out.push_str(&format!(
            "📁 #{} ({}) · {} ({}) — {} [Instance: {}{}{}]\n",
            proj.seq_code,
            proj.gitmap_seq_code,
            label,
            proj_badge,
            proj.repo_path,
            inst_seq_str,
            proj.instance_name,
            email_str
        ));

        if proj.conversations.is_empty() {
            out.push_str("   └─ (No conversations recorded for this workspace)\n");
            continue;
        }

        let conv_len = proj.conversations.len();
        for (idx, conv) in proj.conversations.iter().enumerate() {
            let is_last = idx + 1 == conv_len;
            let branch = if is_last { "└─" } else { "├─" };
            let sub_pipe = if is_last { "   " } else { "│  " };
            let c_badge = if conv.is_running { "🟢" } else { "⚪" };
            let steps_str = if conv.step_count > 0 {
                format!(" · {} steps", conv.step_count)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "   {} 💬 #{} ({}) · \"{}\" ({} {}{})\n",
                branch,
                conv.seq_code,
                conv.gitmap_seq_code,
                conv.title,
                c_badge,
                conv.status,
                steps_str
            ));

            if !conv.prompt_preview_200w.is_empty() {
                out.push_str(&format!(
                    "   {} └─ 📝 Prompt ≤{}w ({} words): \"{}\"\n",
                    sub_pipe, word_cap, conv.prompt_word_count, conv.prompt_preview_200w
                ));
            }
        }
    }

    out.push_str(&"━".repeat(86));
    out.push_str(
        "\n💡 Target by Dual AGM/GitMap Sequence, Instance, & Machine:\n\
         • AGM Conv:    `agm prompt C001 \"is it done?\"`\n\
         • AGM Inst:    `agm prompt P001 \"run tests\" --instance #2`\n\
         • Remote Node: `agm prompt C001 \"check status\" --instance default --node worker-1`\n\
         • GitMap CLI:  `gitmap agy prompt-project P001 -n is-done -t \"verify all\"`\n",
    );
    out
}

fn escape_tg_html_local(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

/// Format the Project -> Conversation -> 200-Word Prompt Tree View for Telegram HTML output
pub fn format_tree_view_telegram_html(max_words: usize, only_running: bool) -> String {
    let word_cap = if max_words == 0 { 200 } else { max_words };
    let tree = get_project_conversation_tree(word_cap, only_running);
    let mut out = String::new();

    let title = if only_running {
        "🌳 <b>AGM + GitMap Running Tree (Project → Conv → [≤200w Prompt])</b>"
    } else {
        "🌳 <b>AGM + GitMap Full Workspace Tree (Project → Conv → [≤200w Prompt])</b>"
    };
    out.push_str(title);
    out.push_str("\n━━━━━━━━━━━━━━━━━━━━\n");

    if tree.is_empty() {
        out.push_str("<i>No matching running projects or conversations found.</i>\n");
        out.push_str("\n💡 Try <code>/tree all</code> to view idle workspaces too.");
        return out;
    }

    for proj in &tree {
        let p_icon = if proj.is_running { "🟢" } else { "⚪" };
        let label =
            format_friendly_workspace_label(&proj.project_id, &proj.repo_name, &proj.repo_path);
        let inst_seq_str = proj
            .instance_seq_num
            .map(|n| format!("#{} ", n))
            .unwrap_or_default();
        let email_str = proj
            .bound_email
            .as_deref()
            .filter(|e| !e.is_empty())
            .map(|e| format!(" · {}", e))
            .unwrap_or_default();

        out.push_str(&format!(
            "\n📁 <b>#{}</b> (<code>{}</code>) · {} <b>{}</b>\n   🖥️ <i>Instance: {}{}{}</i> · 📂 <code>{}</code>\n",
            escape_tg_html_local(&proj.seq_code),
            escape_tg_html_local(&proj.gitmap_seq_code),
            p_icon,
            escape_tg_html_local(&label),
            escape_tg_html_local(&inst_seq_str),
            escape_tg_html_local(&proj.instance_name),
            escape_tg_html_local(&email_str),
            escape_tg_html_local(&proj.repo_path),
        ));

        if proj.conversations.is_empty() {
            out.push_str("   └─ <i>No recent conversations</i>\n");
            continue;
        }

        let conv_len = proj.conversations.len();
        for (idx, conv) in proj.conversations.iter().enumerate() {
            let is_last = idx + 1 == conv_len;
            let branch = if is_last { "└─" } else { "├─" };
            let sub_pipe = if is_last { "   " } else { "│  " };
            let c_icon = if conv.is_running { "🟢" } else { "⚪" };
            let steps_str = if conv.step_count > 0 {
                format!(" · {} steps", conv.step_count)
            } else {
                String::new()
            };

            out.push_str(&format!(
                "   {} 💬 <b>#{}</b> (<code>{}</code>) · <b>{}</b> ({} {}{})\n",
                branch,
                escape_tg_html_local(&conv.seq_code),
                escape_tg_html_local(&conv.gitmap_seq_code),
                escape_tg_html_local(&conv.title),
                c_icon,
                escape_tg_html_local(&conv.status),
                escape_tg_html_local(&steps_str),
            ));

            if !conv.prompt_preview_200w.is_empty() {
                out.push_str(&format!(
                    "   {} └─ 📝 <b>Prompt ≤{}w ({}w):</b> <i>\"{}\"</i>\n",
                    sub_pipe,
                    word_cap,
                    conv.prompt_word_count,
                    escape_tg_html_local(&conv.prompt_preview_200w),
                ));
            }
        }
    }

    out.push_str("\n━━━━━━━━━━━━━━━━━━━━\n");
    out.push_str("💡 <b>Prompt by Dual Seq / Instance / Node:</b>\n");
    out.push_str("• <code>/prompt C001 Is it done?</code>\n");
    out.push_str("• <code>/prompt P001 --instance #2 Run pre-flight checks</code>\n");
    out.push_str("• <code>/prompt C001 --instance default --node worker-1 Check build</code>\n");
    out.push_str("• <code>/agy prompt-project P001 -n is-done -t \"verify\"</code>");
    out
}

// ---------------------------------------------------------------------------
// 5-Second Prompt Heartbeat Goal Engine & Watchdog Recovery
// ---------------------------------------------------------------------------
