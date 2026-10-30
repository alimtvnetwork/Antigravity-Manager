//! 100-improvements flywheel: plan 100 items as 10 batches of 10, then
//! dispatch `01-prompts/30-improvements/` discovery or next-10 prompts.

mod roots;

use crate::error::{AppError, AppResult};
use crate::modules::repo_db;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};

pub use roots::{
    collect_instance_task_hints, collect_workspace_task_hints, dispatch_root, ledger_root,
    normalize_instance_id, repo_root,
};

pub const BATCH_SIZE: u32 = 10;
pub const TOTAL_ITEMS: u32 = 100;
pub const BATCH_COUNT: u32 = TOTAL_ITEMS / BATCH_SIZE;
pub const LEDGER_REL: &str = ".ai-memory/improvements/flywheel.json";

const PROMPT_UI: &str = "01-prompts/30-improvements/01-next-100-ui-improvements.md";
const PROMPT_CODE: &str = "01-prompts/30-improvements/02-next-100-code-improvements.md";
const PROMPT_SYSTEM: &str = "01-prompts/30-improvements/03-next-100-whole-system-improvements.md";
const PROMPT_DETAIL: &str = "01-prompts/30-improvements/04-next-detail-out-100-improvements.md";
const PROMPT_IDEAS: &str = "01-prompts/30-improvements/05-next-100-ideas-for-improvements.md";
const PROMPT_CHECK: &str = "01-prompts/30-improvements/06-check-pending-improvements.md";
const PROMPT_NEXT: &str = "01-prompts/30-improvements/07-implement-next-10-improvements.md";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImprovementKind {
    Ui,
    Code,
    System,
    Ideas,
}

impl ImprovementKind {
    pub fn parse(raw: &str) -> AppResult<Self> {
        match raw.trim().to_ascii_lowercase().as_str() {
            "" | "all" | "system" | "sys" => Ok(Self::System),
            "ui" | "ux" => Ok(Self::Ui),
            "code" | "codebase" => Ok(Self::Code),
            "ideas" | "idea" => Ok(Self::Ideas),
            other => Err(AppError::Config(format!(
                "unknown improvements kind '{other}' (use ui, code, system, ideas)"
            ))),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Ui => "ui",
            Self::Code => "code",
            Self::System => "system",
            Self::Ideas => "ideas",
        }
    }

    pub fn discovery_prompt(self) -> &'static str {
        match self {
            Self::Ui => PROMPT_UI,
            Self::Code => PROMPT_CODE,
            Self::System => PROMPT_SYSTEM,
            Self::Ideas => PROMPT_IDEAS,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchRecord {
    pub index: u32,
    pub status: String,
    pub title: String,
    pub prompt_rel: String,
    pub item_start: u32,
    pub item_end: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FlywheelState {
    pub wave: u32,
    pub kind: String,
    pub total: u32,
    pub batch_size: u32,
    pub next_batch: u32,
    pub completed_batches: u32,
    pub last_action: String,
    pub updated_at: i64,
    pub ledger_rel: String,
    pub dispatched_prompt_id: Option<String>,
    pub batches: Vec<BatchRecord>,
    #[serde(default)]
    pub workspace_root: String,
    #[serde(default)]
    pub task_hints: Vec<String>,
}

impl FlywheelState {
    fn empty(kind: ImprovementKind, wave: u32) -> Self {
        let batches = (1..=BATCH_COUNT)
            .map(|index| {
                let item_start = (index - 1) * BATCH_SIZE + 1;
                let item_end = index * BATCH_SIZE;
                BatchRecord {
                    index,
                    status: "pending".to_string(),
                    title: format!("Batch {index:02} — items {item_start:02}–{item_end:02}"),
                    prompt_rel: PROMPT_NEXT.to_string(),
                    item_start,
                    item_end,
                }
            })
            .collect();
        Self {
            wave,
            kind: kind.as_str().to_string(),
            total: TOTAL_ITEMS,
            batch_size: BATCH_SIZE,
            next_batch: 1,
            completed_batches: 0,
            last_action: "unplanned".to_string(),
            updated_at: Utc::now().timestamp(),
            ledger_rel: LEDGER_REL.to_string(),
            dispatched_prompt_id: None,
            batches,
            workspace_root: String::new(),
            task_hints: Vec::new(),
        }
    }
}

fn ledger_path(root: &Path) -> PathBuf {
    root.join(LEDGER_REL)
}

fn attach_workspace(mut state: FlywheelState, instance_id: &str) -> FlywheelState {
    let root = dispatch_root(instance_id);
    state.workspace_root = root.to_string_lossy().to_string();
    state.task_hints = collect_instance_task_hints(instance_id);
    state
}

pub fn load_state() -> AppResult<FlywheelState> {
    load_state_for("default")
}

pub fn load_state_for(instance_id: &str) -> AppResult<FlywheelState> {
    let inst = normalize_instance_id(instance_id);
    let root = ledger_root(&inst);
    let path = ledger_path(&root);
    if !path.exists() {
        return Ok(attach_workspace(
            FlywheelState::empty(ImprovementKind::System, 1),
            &inst,
        ));
    }
    let raw = match fs::read_to_string(&path) {
        Ok(body) => body,
        Err(e) => {
            tracing::warn!("improvements ledger read failed, using empty: {e}");
            return Ok(attach_workspace(
                FlywheelState::empty(ImprovementKind::System, 1),
                &inst,
            ));
        }
    };
    match serde_json::from_str(&raw) {
        Ok(state) => Ok(attach_workspace(state, &inst)),
        Err(e) => {
            tracing::warn!("improvements ledger parse failed, using empty: {e}");
            Ok(attach_workspace(
                FlywheelState::empty(ImprovementKind::System, 1),
                &inst,
            ))
        }
    }
}

fn save_state(root: &Path, state: &FlywheelState) -> AppResult<()> {
    let path = ledger_path(root);
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|e| {
            AppError::Io(std::io::Error::other(format!(
                "create .ai-memory/improvements: {e}"
            )))
        })?;
    }
    let body = serde_json::to_string_pretty(state)
        .map_err(|e| AppError::Config(format!("serialize flywheel: {e}")))?;
    fs::write(&path, body)
        .map_err(|e| AppError::Io(std::io::Error::other(format!("write {LEDGER_REL}: {e}"))))?;
    Ok(())
}

fn build_dispatch_prompt(
    kind_label: &str,
    prompt_rel: &str,
    batch: Option<&BatchRecord>,
    wave: u32,
    workspace: &Path,
    action: &str,
    task_hints: &[String],
) -> String {
    let batch_line = match batch {
        Some(b) => format!(
            "Batch {}/{} (items {}–{}) — follow 10-at-a-time sequence.\n",
            b.index, BATCH_COUNT, b.item_start, b.item_end
        ),
        None => "Plan exactly 100 improvements as 10 batches of 10.\n".to_string(),
    };
    let workspace_disp = workspace.to_string_lossy();
    let prompt_exists = workspace.join(prompt_rel).is_file();
    let task_block = if task_hints.is_empty() {
        "No `.ai-memory/plans` or issues found. Inventory the current project tasks, tickets, and unfinished work, then break those down.\n".to_string()
    } else {
        format!(
            "Pending tasks in this instance (any bound workspace):\n{}\n",
            task_hints
                .iter()
                .map(|h| format!("- {h}"))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };
    let source_hint = if prompt_exists {
        format!("Read and follow `{prompt_rel}` end to end in this workspace.\n")
    } else {
        format!(
            "`{prompt_rel}` is not in this workspace. Follow the fallback for `{action}`:\n\
             - plan: inventory current work and write 100 improvements as 10 batches of 10 under `.ai-memory/plans/`.\n\
             - next: implement the next 10 pending items listed below, then `.ai-memory/plans/`, `.ai-memory/plans/subtasks/`, or the current task backlog.\n\
             - detail: expand the tasks listed below (and any other current plans) into exhaustive 100-step breakdowns any AI can execute.\n"
        )
    };
    format!(
        "# AGM Improvements Flywheel\n\n\
         Work in this instance workspace (any project, not only AGM):\n\
         `{workspace_disp}`\n\n\
         {source_hint}\n\
         {task_block}\n\
         Kind: {kind_label}\n\
         Wave: {wave}\n\
         Action: {action}\n\
         {batch_line}\
         Ledger: `{LEDGER_REL}`\n\n\
         Use execute-parent-task-with-n-steps-v6. GitMap-only search. Relative Git paths only.\n\
         Do not stop Default or v2-3127. Isolated test-create/test-clean only.\n"
    )
}

fn dispatch_prompt(instance_id: &str, prompt: &str) -> AppResult<String> {
    let inst = normalize_instance_id(instance_id);
    let repo = dispatch_root(&inst).to_string_lossy().to_string();
    let queued = repo_db::enqueue_prompt_for_instance(&inst, &repo, prompt, None)
        .map_err(|e| AppError::Unknown(format!("enqueue improvements prompt: {e}")))?;
    Ok(queued.id)
}

pub fn plan_flywheel(
    kind: ImprovementKind,
    dry_run: bool,
    instance_id: &str,
) -> AppResult<FlywheelState> {
    let inst = normalize_instance_id(instance_id);
    let root = ledger_root(&inst);
    let prev_wave = load_state_for(&inst).ok().map(|s| s.wave).unwrap_or(0);
    let mut state = FlywheelState::empty(kind, prev_wave.saturating_add(1).max(1));
    state.last_action = if dry_run {
        "plan-dry-run".to_string()
    } else {
        "plan".to_string()
    };
    state.updated_at = Utc::now().timestamp();
    if !dry_run {
        let hints = collect_instance_task_hints(&inst);
        let body = build_dispatch_prompt(
            kind.as_str(),
            kind.discovery_prompt(),
            None,
            state.wave,
            &root,
            "plan",
            &hints,
        );
        state.dispatched_prompt_id = Some(dispatch_prompt(&inst, &body)?);
        state.last_action = format!("plan-dispatched:{}", kind.discovery_prompt());
    }
    save_state(&root, &state)?;
    Ok(attach_workspace(state, &inst))
}

pub fn next_batch(dry_run: bool, instance_id: &str) -> AppResult<FlywheelState> {
    let inst = normalize_instance_id(instance_id);
    let root = ledger_root(&inst);
    let mut state = load_state_for(&inst)?;
    let pending = state
        .batches
        .iter()
        .position(|b| b.status == "pending")
        .ok_or_else(|| {
            AppError::Unknown(
                "all 10 batches are already dispatched. Plan 100 to start a new wave.".to_string(),
            )
        })?;
    let batch_index = state.batches[pending].index;
    if dry_run {
        let hint_n = collect_instance_task_hints(&inst).len();
        state.last_action = format!("next-dry-run:{batch_index}:{hint_n}-tasks");
        state.dispatched_prompt_id = None;
        state.updated_at = Utc::now().timestamp();
        return Ok(attach_workspace(state, &inst));
    }
    let kind = state.kind.clone();
    let wave = state.wave;
    let prompt_body = {
        let batch = &state.batches[pending];
        let hints = collect_instance_task_hints(&inst);
        build_dispatch_prompt(&kind, PROMPT_NEXT, Some(batch), wave, &root, "next", &hints)
    };
    state.batches[pending].status = "dispatched".to_string();
    state.next_batch = (batch_index + 1).min(BATCH_COUNT);
    state.completed_batches = state
        .batches
        .iter()
        .filter(|b| b.status == "dispatched" || b.status == "done")
        .count() as u32;
    state.updated_at = Utc::now().timestamp();
    state.dispatched_prompt_id = Some(dispatch_prompt(&inst, &prompt_body)?);
    state.last_action = format!("next-dispatched:{batch_index}");
    save_state(&root, &state)?;
    Ok(attach_workspace(state, &inst))
}

pub fn detail_flywheel(dry_run: bool, instance_id: &str) -> AppResult<FlywheelState> {
    let inst = normalize_instance_id(instance_id);
    let root = ledger_root(&inst);
    let mut state = load_state_for(&inst)?;
    state.updated_at = Utc::now().timestamp();
    if dry_run {
        let hint_n = collect_instance_task_hints(&inst).len();
        state.last_action = format!("detail-dry-run:{hint_n}-tasks");
        state.dispatched_prompt_id = None;
    } else {
        let hints = collect_instance_task_hints(&inst);
        let body = build_dispatch_prompt(
            &state.kind,
            PROMPT_DETAIL,
            None,
            state.wave,
            &root,
            "detail",
            &hints,
        );
        state.dispatched_prompt_id = Some(dispatch_prompt(&inst, &body)?);
        state.last_action = format!("detail-dispatched:{PROMPT_DETAIL}");
    }
    save_state(&root, &state)?;
    Ok(attach_workspace(state, &inst))
}

pub fn check_pending() -> AppResult<FlywheelState> {
    check_pending_for("default")
}

pub fn check_pending_for(instance_id: &str) -> AppResult<FlywheelState> {
    let inst = normalize_instance_id(instance_id);
    let mut state = load_state_for(&inst)?;
    state.last_action = format!("check:{PROMPT_CHECK}");
    state.updated_at = Utc::now().timestamp();
    state.completed_batches = state
        .batches
        .iter()
        .filter(|b| b.status == "dispatched" || b.status == "done")
        .count() as u32;
    let root = ledger_root(&inst);
    save_state(&root, &state)?;
    Ok(attach_workspace(state, &inst))
}

fn escape_html(input: &str, max_chars: usize) -> String {
    let clipped: String = input.chars().take(max_chars).collect();
    clipped
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

pub fn format_telegram_html(state: &FlywheelState) -> String {
    let pending = state
        .batches
        .iter()
        .filter(|b| b.status == "pending")
        .count();
    let next = state
        .batches
        .iter()
        .find(|b| b.status == "pending")
        .map(|b| b.title.as_str())
        .unwrap_or("all batches dispatched");
    format!(
        "🛠 <b>100 Improvements</b>\n\
         Kind <code>{}</code> · wave <code>{}</code>\n\
         ✅ {}/{} batches · ⏳ {} pending\n\
         Next: <i>{}</i>\n\
         Ledger <code>{}</code>\n\
         Workspace <code>{}</code>\n\
         Tasks <code>{}</code>\n\
         <i>Plan 100, then Next 10, or Detail any pending tasks.</i>",
        escape_html(&state.kind, 24),
        state.wave,
        state.completed_batches,
        BATCH_COUNT,
        pending,
        escape_html(next, 80),
        LEDGER_REL,
        escape_html(&state.workspace_root, 80),
        state.task_hints.len()
    )
}

pub fn format_cli_text(state: &FlywheelState) -> String {
    let pending = state
        .batches
        .iter()
        .filter(|b| b.status == "pending")
        .count();
    let mut lines = vec![
        "AGM 100 Improvements Flywheel".to_string(),
        format!(
            "  kind={}  wave={}  batches={}/{} done  pending={}  action={}",
            state.kind,
            state.wave,
            state.completed_batches,
            BATCH_COUNT,
            pending,
            state.last_action
        ),
        format!("  ledger {}", state.ledger_rel),
        format!("  workspace {}", state.workspace_root),
        format!("  tasks {}", state.task_hints.len()),
    ];
    for hint in state.task_hints.iter().take(8) {
        lines.push(format!("    · {hint}"));
    }
    if let Some(id) = &state.dispatched_prompt_id {
        lines.push(format!("  dispatched_prompt_id {id}"));
    }
    lines.push("  batches:".to_string());
    for batch in &state.batches {
        let marker = if batch.status == "pending" {
            "·"
        } else {
            "✓"
        };
        lines.push(format!(
            "    {marker} {:02}  {:<11}  {}",
            batch.index, batch.status, batch.title
        ));
    }
    lines.push(
        "  next: agm improvements next --dry-run     # claim the next 10 without dispatching"
            .to_string(),
    );
    lines.join("\n")
}

pub fn telegram_markup() -> Value {
    json!({
        "inline_keyboard": [
            [
                { "text": "📋 Plan 100", "callback_data": "im:pl" },
                { "text": "⏭ Next 10", "callback_data": "im:n10" }
            ],
            [
                { "text": "🧩 Detail", "callback_data": "im:dt" },
                { "text": "📊 Status", "callback_data": "im:st" }
            ],
            [
                { "text": "🔎 Check", "callback_data": "im:ck" }
            ]
        ]
    })
}

pub fn is_improvements_command(text: &str) -> bool {
    let trimmed = text.trim().trim_start_matches('/');
    let base = trimmed
        .split('@')
        .next()
        .unwrap_or(trimmed)
        .split_whitespace()
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    matches!(
        base.as_str(),
        "improvements" | "improve" | "next10" | "next-10"
    )
}

pub fn parse_improvements_callback(data: &str) -> Option<&'static str> {
    match data.trim() {
        "im:pl" => Some("plan"),
        "im:n10" => Some("next"),
        "im:dt" => Some("detail"),
        "im:st" => Some("status"),
        "im:ck" => Some("check"),
        _ => None,
    }
}

pub fn handle_rest(rest: &str) -> AppResult<FlywheelState> {
    let mut kind = ImprovementKind::System;
    let mut dry_run = false;
    let mut instance = "default".to_string();
    let mut action = "status";
    let tokens: Vec<&str> = rest.split_whitespace().collect();
    let mut i = 0;
    while i < tokens.len() {
        let lower = tokens[i].to_ascii_lowercase();
        match lower.as_str() {
            "plan" => action = "plan",
            "next" | "next-10" | "next10" | "run" => action = "next",
            "status" | "st" => action = "status",
            "check" | "ck" => action = "check",
            "detail" => action = "detail",
            "ui" | "code" | "system" | "ideas" | "all" => {
                kind = ImprovementKind::parse(&lower)?;
            }
            "--dry-run" | "-n" => dry_run = true,
            "--instance" | "-i" if i + 1 < tokens.len() => {
                instance = tokens[i + 1].to_string();
                i += 1;
            }
            other if other.starts_with("--instance=") => {
                instance = other.trim_start_matches("--instance=").to_string();
            }
            _ => {}
        }
        i += 1;
    }
    match action {
        "plan" => plan_flywheel(kind, dry_run, &instance),
        "next" => next_batch(dry_run, &instance),
        "detail" => detail_flywheel(dry_run, &instance),
        "check" => check_pending_for(&instance),
        _ => load_state_for(&instance),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kind_parse_and_batch_math() {
        assert_eq!(ImprovementKind::parse("ui").unwrap(), ImprovementKind::Ui);
        assert_eq!(
            ImprovementKind::parse("all").unwrap(),
            ImprovementKind::System
        );
        assert_eq!(BATCH_COUNT, 10);
        assert_eq!(TOTAL_ITEMS, 100);
        let state = FlywheelState::empty(ImprovementKind::Ui, 1);
        assert_eq!(state.batches.len(), 10);
        assert_eq!(state.batches[0].item_start, 1);
        assert_eq!(state.batches[9].item_end, 100);
    }

    #[test]
    fn test_callback_budget() {
        for key in ["im:pl", "im:n10", "im:dt", "im:st", "im:ck"] {
            assert!(key.len() <= 60);
            assert!(parse_improvements_callback(key).is_some());
        }
        assert!(parse_improvements_callback("i:st:abc").is_none());
    }

    #[test]
    fn test_dispatch_prompt_fallback_mentions_any_workspace() {
        let tmp = std::env::temp_dir().join("agm-fw-no-prompts");
        let body = build_dispatch_prompt("system", PROMPT_NEXT, None, 1, &tmp, "next", &[]);
        assert!(body.contains("any project, not only AGM"));
        assert!(body.contains("fallback for `next`"));
        assert!(body.contains(LEDGER_REL));
        assert!(body.contains("Inventory the current project tasks"));
    }

    #[test]
    fn test_dispatch_prompt_lists_pending_tasks() {
        let tmp = std::env::temp_dir().join(format!(
            "agm-fw-tasks-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis())
                .unwrap_or(0)
        ));
        let pending = tmp.join(".ai-memory").join("plans").join("pending");
        std::fs::create_dir_all(&pending).expect("create pending");
        std::fs::write(pending.join("42-next-breakdown.md"), "# detail\n").expect("write");
        let hints = collect_workspace_task_hints(&tmp);
        let body = build_dispatch_prompt("system", PROMPT_DETAIL, None, 1, &tmp, "detail", &hints);
        assert!(body.contains("42-next-breakdown"));
        assert!(body.contains("Pending tasks in this instance"));
        assert!(body.contains("fallback for `detail`"));
        crate::error::record_ignored(
            std::fs::remove_dir_all(&tmp),
            "cleanup flywheel prompt fixture",
        );
    }

    #[test]
    fn test_status_never_requires_source_checkout() {
        let state = load_state_for("default").expect("status must succeed without AGM source cwd");
        assert_eq!(state.batches.len(), 10);
        assert!(!state.workspace_root.is_empty());
    }
}
