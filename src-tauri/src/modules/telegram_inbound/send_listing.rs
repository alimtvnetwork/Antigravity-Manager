//! Project picker pages for the Telegram `/send` flow.
//!
//! Eight projects per page, newest conversation first. `list_all_prompts`
//! keeps `ORDER BY created_at ASC, id ASC`. Newest-first order is a local
//! `Vec` sort and does not change that SQL.

use std::collections::HashSet;
use std::path::Path;

use crate::modules::repo_db::ActivePrompt;

const PAGE_SIZE: usize = 8;
const TITLE_CHARS: usize = 80;

#[derive(Clone)]
pub struct PickerProject {
    pub value: String,
    pub project_name: String,
    pub conversation_title: String,
    pub updated_at: i64,
}

pub struct PickerPage {
    pub rows: Vec<PickerProject>,
    pub page: usize,
    pub has_next: bool,
    pub has_back: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct PageWindow {
    pub page: usize,
    pub start: usize,
    pub end: usize,
    pub has_next: bool,
    pub has_back: bool,
}

pub fn load_project_page(instance_id: &str, page: usize) -> PickerPage {
    let prompts = match crate::modules::repo_db::list_all_prompts() {
        Ok(rows) => rows,
        Err(err) => {
            // Justification: a prompt-database read failure only blanks this
            // picker page; the send flow still offers manual conversation entry.
            crate::error::record_ignored(
                Err::<(), String>(err),
                "list prompts for telegram project picker",
            );
            return PickerPage {
                rows: Vec::new(),
                page: 0,
                has_next: false,
                has_back: false,
            };
        }
    };
    let projects = projects_for_instance(prompts, instance_id);
    let window = page_slice(projects.len(), page);
    PickerPage {
        rows: projects[window.start..window.end].to_vec(),
        page: window.page,
        has_next: window.has_next,
        has_back: window.has_back,
    }
}

pub fn format_project_list(rows: &[PickerProject], page: usize) -> String {
    let palette = crate::modules::telegram_inbound::palette::active_card_palette();
    let mut out = String::with_capacity(rows.len().saturating_mul(96).saturating_add(page));
    for (offset, row) in rows.iter().enumerate() {
        let n = offset + 1;
        let project = super::formatting::clean_for_telegram_html(&row.project_name, 160);
        let title =
            super::formatting::clean_for_telegram_html(&row.conversation_title, TITLE_CHARS);
        out.push_str(&format!(
            "{n}. {mark} <b>{project}</b>\n   {bullet} {title}\n",
            mark = palette.mark,
            bullet = palette.bullet,
        ));
    }
    out
}

/// `page` is 0-based. An out-of-range page clamps to the last page when
/// `total` is non-zero.
pub(crate) fn page_slice(total: usize, page: usize) -> PageWindow {
    if total == 0 {
        return PageWindow {
            page: 0,
            start: 0,
            end: 0,
            has_next: false,
            has_back: false,
        };
    }
    let last = (total - 1) / PAGE_SIZE;
    let page = page.min(last);
    let start = page * PAGE_SIZE;
    let end = (start + PAGE_SIZE).min(total);
    PageWindow {
        page,
        start,
        end,
        has_next: end < total,
        has_back: page > 0,
    }
}

/// File name of `repo_path` when it is non-empty. Otherwise `project_id`
/// with a trailing `__` plus `instance_id` removed when that suffix is present.
pub(crate) fn project_label(repo_path: &str, project_id: &str, instance_id: &str) -> String {
    let path = repo_path.trim();
    if !path.is_empty() {
        if let Some(name) = Path::new(path)
            .file_name()
            .and_then(|name| name.to_str())
            .map(str::trim)
            .filter(|name| !name.is_empty())
        {
            return name.to_string();
        }
    }
    strip_instance_suffix(project_id, instance_id)
}

fn projects_for_instance(prompts: Vec<ActivePrompt>, instance_id: &str) -> Vec<PickerProject> {
    let mut rows: Vec<ActivePrompt> = prompts
        .into_iter()
        .filter(|prompt| prompt.instance_id == instance_id)
        .collect();
    rows.sort_by(|left, right| {
        right
            .updated_at
            .cmp(&left.updated_at)
            .then_with(|| right.id.cmp(&left.id))
    });

    let mut seen = HashSet::new();
    let mut projects = Vec::new();
    for prompt in rows {
        if !seen.insert(prompt.project_id.clone()) {
            continue;
        }
        let Some(value) = picker_value(&prompt) else {
            continue;
        };
        let session_id = session_text(&prompt);
        projects.push(PickerProject {
            project_name: project_label(&prompt.repo_path, &prompt.project_id, instance_id),
            conversation_title: conversation_title(&session_id, &prompt.prompt_content),
            updated_at: prompt.updated_at,
            value,
        });
    }
    projects.sort_by(|left, right| right.updated_at.cmp(&left.updated_at));
    projects
}

fn picker_value(prompt: &ActivePrompt) -> Option<String> {
    let session_id = session_text(prompt);
    if !session_id.is_empty() && picker_safe(&session_id) {
        return Some(session_id);
    }
    if picker_safe(&prompt.project_id) {
        return Some(prompt.project_id.clone());
    }
    None
}

fn session_text(prompt: &ActivePrompt) -> String {
    prompt
        .session_id
        .as_deref()
        .map(str::trim)
        .filter(|session| !session.is_empty())
        .unwrap_or("")
        .to_string()
}

fn picker_safe(token: &str) -> bool {
    !token.is_empty() && !token.contains('|') && !token.contains(':')
}

fn strip_instance_suffix(project_id: &str, instance_id: &str) -> String {
    let instance_id = instance_id.trim();
    if instance_id.is_empty() {
        return project_id.to_string();
    }
    let suffix = format!("__{instance_id}");
    match project_id.strip_suffix(&suffix) {
        Some(rest) if !rest.is_empty() => rest.to_string(),
        _ => project_id.to_string(),
    }
}

fn conversation_title(session_id: &str, prompt_content: &str) -> String {
    if !session_id.is_empty() {
        if let Some(title) = crate::modules::repo_db::lookup_conversation_title(session_id) {
            let trimmed = title.trim();
            if !trimmed.is_empty() && trimmed != session_id {
                return trimmed.to_string();
            }
        }
    }
    first_prompt_line(prompt_content, session_id)
        .unwrap_or_else(|| "Untitled conversation".to_string())
}

fn first_prompt_line(prompt_content: &str, session_id: &str) -> Option<String> {
    for line in prompt_content.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        if !session_id.is_empty() && trimmed == session_id {
            continue;
        }
        let short: String = trimmed.chars().take(TITLE_CHARS).collect();
        if short.is_empty() {
            continue;
        }
        return Some(short);
    }
    None
}

#[cfg(test)]
mod tests {
    use super::{page_slice, project_label};

    #[test]
    fn project_label_uses_repo_file_name() {
        assert_eq!(
            project_label("repos/antigravity-manager", "ignored__alpha", "alpha"),
            "antigravity-manager"
        );
    }

    #[test]
    fn project_label_strips_trailing_instance_suffix() {
        assert_eq!(project_label("", "fleet-ui__alpha", "alpha"), "fleet-ui");
        assert_eq!(project_label("   ", "fleet-ui__alpha", "alpha"), "fleet-ui");
    }

    #[test]
    fn page_slice_of_nine_yields_eight_and_has_next() {
        let window = page_slice(9, 0);
        assert_eq!(window.end - window.start, 8);
        assert!(window.has_next);
        assert!(!window.has_back);
        assert_eq!(window.page, 0);
    }

    #[test]
    fn second_page_has_back() {
        let window = page_slice(9, 1);
        assert!(window.has_back);
        assert!(!window.has_next);
        assert_eq!(window.page, 1);
        assert_eq!(window.start, 8);
        assert_eq!(window.end, 9);
    }
}
