use crate::modules::email_vault_db::{self, EmailAccount};
use crate::modules::*;
use base64::prelude::*;

use super::*;

// ---------------------------------------------------------------------------
// Rich HTML Email Templates (With Node Alias & Local IPv4 Telemetry)
// ---------------------------------------------------------------------------

pub(crate) fn wrap_plaintext_email(
    title: &str,
    content: &str,
    machine_name: &str,
    machine_ip: &str,
) -> String {
    wrap_html_email_card(title, content, machine_name, machine_ip)
}

/// Render HTML email for low quota alerts
pub fn render_quota_drop_email(
    email: &str,
    current_quota: f64,
    threshold: u32,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] Low Quota Warning ({:.1}%) - {}",
        pkg_ver, machine_name, machine_ip, current_quota, email
    );
    let content = format!(
        "[!] CREDIT THRESHOLD TRIGGER\r\n\r\n\
         The active profile '{}' has dropped to {:.1}% remaining credit,\r\n\
         falling below the configured safety threshold of {}%.\r\n\r\n\
         If auto-profile switcher is enabled, Antigravity Manager will attempt\r\n\
         to migrate active workspaces to the next highest credit account.",
        email, current_quota, threshold
    );
    let body = wrap_plaintext_email("Low Credit Alert", &content, machine_name, machine_ip);
    (subject, body)
}

/// Render HTML email for automated workspace switch
pub fn render_workspace_switch_email(
    from_instance: &str,
    to_instance: &str,
    reason: &str,
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] [Notice] Workspace Auto-Switched: {}",
        pkg_ver, machine_name, machine_ip, to_instance
    );
    let content = format!(
        "[*] WORKSPACE SWITCHED\r\n\r\n\
         Antigravity Manager transitioned active tasks from '{}' to '{}'.\r\n\r\n\
         Reason: {}\r\n\r\n\
         Running prompts and repository states were safely snapshotted to repo_prompts.db.",
        from_instance, to_instance, reason
    );
    let body = wrap_plaintext_email(
        "Profile Migration Notice",
        &content,
        machine_name,
        machine_ip,
    );
    (subject, body)
}

/// Render HTML email for idle running projects alert
pub fn render_idle_projects_email(
    projects: &[crate::modules::repo_db::ProjectExecutionInfo],
    machine_name: &str,
    machine_ip: &str,
) -> (String, String) {
    let pkg_ver = format!("v{}", env!("CARGO_PKG_VERSION"));
    let subject = format!(
        "[AGM {} | {} | {}] [Prompt Request] Running Projects Idle - Ready for Instructions",
        pkg_ver, machine_name, machine_ip
    );

    let mut seen_repos = std::collections::HashSet::new();
    let mut proj_rows = String::new();
    for p in projects {
        let repo_clean = p.repo_name.trim().to_lowercase();
        if !seen_repos.insert(repo_clean) {
            continue;
        }
        let is_running = p.status.to_uppercase().contains("RUNNING");
        let (badge_bg, badge_fg, badge_border, badge_text) = if is_running {
            ("#ecfdf5", "#047857", "#a7f3d0", "RUNNING")
        } else {
            ("#f1f5f9", "#475569", "#cbd5e1", "IDLE / READY")
        };
        proj_rows.push_str(&format!(
            r#"<tr>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">{}</td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-family: 'Ubuntu Mono', monospace; font-size: 16px; font-weight: 700;">sub: {} | proj-{}</code></td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-family: 'Ubuntu Mono', monospace; font-size: 15px; color: #475569;">{}</td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><span style="background: {}; color: {}; border: 1px solid {}; padding: 6px 14px; border-radius: 6px; font-size: 15px; font-weight: 800;">{}</span></td>
</tr>"#,
            escape_html_entities(&p.repo_name),
            escape_html_entities(machine_name),
            escape_html_entities(&p.project_id),
            escape_html_entities(&p.repo_path),
            badge_bg,
            badge_fg,
            badge_border,
            badge_text,
        ));
    }

    if proj_rows.is_empty() {
        proj_rows.push_str(r#"<tr><td colspan="4" style="padding: 18px 20px; text-align: center; color: #64748b; font-size: 16px; font-style: italic;">No workspace projects currently registered</td></tr>"#);
    }

    let recent_prompts = crate::modules::repo_db::list_all_prompts().unwrap_or_default();
    let mut prompt_rows = String::new();
    for p in recent_prompts.iter().take(6) {
        let (preview, word_count) =
            crate::modules::notification_hub::extract_words_preview(&p.prompt_content, 200);
        let preview_clean = escape_html_entities(&preview);
        prompt_rows.push_str(&format!(
            r#"<tr>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-family: 'Ubuntu Mono', monospace; font-size: 15px; color: #0284c7; font-weight: 700; vertical-align: top;">{}</td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; font-weight: 600; vertical-align: top;">{}</td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; vertical-align: top;"><span style="background: #f1f5f9; color: #475569; padding: 4px 8px; border-radius: 4px; font-size: 14px; font-weight: 700;">{}</span><br><span style="font-size: 12px; color: #64748b; font-family: 'Ubuntu Mono', monospace;">({} words)</span></td>
  <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 15px; color: #334155; line-height: 1.6; font-family: 'Ubuntu Mono', monospace; vertical-align: top; max-width: 450px; word-break: break-word;">{}</td>
</tr>"#,
            escape_html_entities(&p.id),
            escape_html_entities(&p.project_id),
            escape_html_entities(&p.status),
            word_count,
            preview_clean,
        ));
    }

    let prompts_section = if !prompt_rows.is_empty() {
        format!(
            r#"<div style="font-weight: 800; font-size: 20px; text-transform: uppercase; color: #0f172a; margin-top: 28px; margin-bottom: 14px; letter-spacing: 0.05em;">
  Cached Prompts History &amp; Queue (≥200 Words Preview)
</div>
<table style="width: 100%; border-collapse: collapse; font-size: 16px; margin-bottom: 28px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #cbd5e1; box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
  <thead>
    <tr style="background: #334155; color: #ffffff;">
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em; width: 140px;">Prompt ID</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em; width: 140px;">Target Project</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em; width: 120px;">Status</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Instruction (200 Words Preview)</th>
    </tr>
  </thead>
  <tbody>
    {}
  </tbody>
</table>"#,
            prompt_rows
        )
    } else {
        String::new()
    };

    let content_html = format!(
        r#"<div style="background: #f8fafc; border: 2px solid #cbd5e1; border-radius: 12px; padding: 22px 26px; margin-bottom: 26px;">
  <div style="font-weight: 800; font-size: 20px; color: #0f172a; margin-bottom: 10px;">
    [*] IDLE WORKSPACE SENSOR &mdash; All Background Tasks Completed
  </div>
  <p style="margin: 0; color: #334155; font-size: 18px; line-height: 1.7;">
    There are currently no active prompts running across your workspaces on node <strong>{}</strong> (<code>{}</code>).
    The system is verified completely idle and standing by for instructions.
  </p>
</div>

<div style="font-weight: 800; font-size: 20px; text-transform: uppercase; color: #0f172a; margin-top: 28px; margin-bottom: 14px; letter-spacing: 0.05em;">
  Active Workspaces &amp; Reply Target Identifiers
</div>
<table style="width: 100%; border-collapse: collapse; font-size: 16px; margin-bottom: 28px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #cbd5e1; box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
  <thead>
    <tr style="background: #0f172a; color: #ffffff;">
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Project Name</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Email Reply Target Identifier</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Repository Path</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Status</th>
    </tr>
  </thead>
  <tbody>
    {}
  </tbody>
</table>

<div style="font-weight: 800; font-size: 20px; text-transform: uppercase; color: #0f172a; margin-top: 28px; margin-bottom: 14px; letter-spacing: 0.05em;">
  Available Commands &amp; Remote Email Formats
</div>
<table style="width: 100%; border-collapse: collapse; font-size: 16px; margin-bottom: 28px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #cbd5e1; box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
  <thead>
    <tr style="background: #1e293b; color: #ffffff;">
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Action / Goal</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Email Reply Subject Format</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">CLI Equivalent</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">Send Prompt to Project</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;project_name&gt;</code><br><span style="color: #64748b; font-size: 15px; display: inline-block; margin-top: 4px;">(Body: your prompt instruction)</span></td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm prompt -p "&lt;name&gt;" "&lt;text&gt;"</code></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">Broadcast to All</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">sub: {} | all</code><br><span style="color: #64748b; font-size: 15px; display: inline-block; margin-top: 4px;">(Dispatches prompt to every workspace)</span></td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm broadcast-email send-to-all</code></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">Check Quota &amp; Credits</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">cmd: {} | agm status</code></td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm status</code></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">Trigger Auto-Switch</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">cmd: {} | agm switch-if-low-credit</code></td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm swlc</code></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0f172a; font-size: 16px;">List Running Prompts</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">cmd: {} | agm running-prompts ls</code></td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm running-prompts ls</code></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; font-weight: 700; color: #0f172a; font-size: 16px;">Inbound Commands Manual</td>
      <td style="padding: 14px 20px;"><code style="background: #0f172a; color: #f8fafc; padding: 6px 12px; border-radius: 6px; font-weight: 700; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">Subject: help</code></td>
      <td style="padding: 14px 20px;"><code style="background: #f1f5f9; color: #0f172a; padding: 6px 12px; border-radius: 6px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">agm help</code></td>
    </tr>
  </tbody>
</table>

<div style="font-weight: 800; font-size: 20px; text-transform: uppercase; color: #0f172a; margin-top: 28px; margin-bottom: 14px; letter-spacing: 0.05em;">
  Usable Prompts Reference List (01-prompts Library)
</div>
<table style="width: 100%; border-collapse: collapse; font-size: 16px; margin-bottom: 28px; background: #ffffff; border-radius: 10px; overflow: hidden; border: 1px solid #cbd5e1; box-shadow: 0 1px 3px rgba(0,0,0,0.05);">
  <thead>
    <tr style="background: #1e293b; color: #ffffff;">
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Prompt Slug / Name</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">Description &amp; Intended Purpose</th>
      <th style="padding: 16px 20px; text-align: left; font-size: 16px; text-transform: uppercase; letter-spacing: 0.05em;">How to Dispatch via Email</th>
    </tr>
  </thead>
  <tbody>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">execute-pending-tasks</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; line-height: 1.5;">Autonomously executes queued tasks in .ai-memory/plans/pending/ with batched subagents and auto-looping.</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: execute-pending-tasks</span></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">execute-parent-task</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; line-height: 1.5;">Decomposes a large parent task into structured subtasks and runs continuous N-step self-loops until completion.</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: execute-parent-task &lt;goal&gt;</span></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">ci-cd-fix</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; line-height: 1.5;">Grounded 4-part RCA diagnosis, fixes broken pipeline scripts or linters, and verifies green builds.</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: ci-cd-fix</span></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">minor-bump</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; line-height: 1.5;">Synchronizes version across package manifests, updates CHANGELOG.md, and tags release commit.</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: minor-bump</span></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">coding-guidelines</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0; font-size: 16px; color: #334155; line-height: 1.5;">Enforces PascalCase database entities, AppError wrapper patterns, positive booleans, and enum standards.</td>
      <td style="padding: 14px 20px; border-bottom: 1px solid #e2e8f0;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: coding-guidelines</span></td>
    </tr>
    <tr>
      <td style="padding: 14px 20px; font-weight: 700; color: #0284c7; font-size: 16px; font-family: 'Ubuntu Mono', monospace;">smart-test-runner</td>
      <td style="padding: 14px 20px; font-size: 16px; color: #334155; line-height: 1.5;">Executes affected targeted test suites incrementally, isolating slow suites and preventing regressions.</td>
      <td style="padding: 14px 20px;"><code style="background: #0f172a; color: #f8fafc; padding: 4px 8px; border-radius: 4px; font-size: 15px; font-family: 'Ubuntu Mono', monospace;">sub: {} | proj-&lt;proj&gt;</code><br><span style="color: #64748b; font-size: 14px;">Body: smart-test-runner</span></td>
    </tr>
  </tbody>
</table>

{}"#,
        machine_name,
        machine_ip,
        proj_rows,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        machine_name,
        prompts_section
    );

    let body = wrap_html_email_card(
        "Idle Projects Notification",
        &content_html,
        machine_name,
        machine_ip,
    );
    (subject, body)
}
