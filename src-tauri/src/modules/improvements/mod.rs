//! 100-improvements flywheel: plan 100 items as 10 batches of 10, then
//! dispatch `01-prompts/30-improvements/` discovery or next-10 prompts.

use crate::error::{AppError, AppResult};
use crate::modules::repo_db;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

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
        }
    }
}

pub fn repo_root() -> AppResult<PathBuf> {
    let mut cmd = Command::new("git");
    cmd.args(["rev-parse", "--show-toplevel"]);
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        cmd.creation_flags(0x08000000);
    }
    if let Ok(out) = cmd.output() {
        if out.status.success() {
            let text = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if !text.is_empty() {
                let root = PathBuf::from(text);
                if root.join("01-prompts/30-improvements").is_dir() {
                    return Ok(root);
                }
            }
        }
    }
    let mut dir = std::env::current_dir()
        .map_err(|e| AppError::Io(std::io::Error::other(format!("cwd: {e}"))))?;
    loop {
        if dir.join("01-prompts/30-improvements").is_dir() {
            return Ok(dir);
        }
        if !dir.pop() {
            break;
        }
    }
    Err(AppError::Config(
        "could not locate repository root containing 01-prompts/30-improvements".to_string(),
    ))
}

fn ledger_path(root: &Path) -> PathBuf {
    root.join(LEDGER_REL)
}

pub fn load_state() -> AppResult<FlywheelState> {
    let root = repo_root()?;
    let path = ledger_path(&root);
    if !path.exists() {
        return Ok(FlywheelState::empty(ImprovementKind::System, 1));
    }
    let raw = fs::read_to_string(&path)
        .map_err(|e| AppError::Io(std::io::Error::other(format!("read {LEDGER_REL}: {e}"))))?;
    serde_json::from_str(&raw).map_err(|e| AppError::Config(format!("parse {LEDGER_REL}: {e}")))
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
) -> String {
    let batch_line = match batch {
        Some(b) => format!(
            "Batch {}/{} (items {}–{}) — follow 10-at-a-time sequence.\n",
            b.index, BATCH_COUNT, b.item_start, b.item_end
        ),
        None => "Plan exactly 100 improvements as 10 batches of 10.\n".to_string(),
    };
    format!(
        "# AGM Improvements Flywheel\n\n\
         Read and follow `{prompt_rel}` end to end.\n\n\
         Kind: {kind_label}\n\
         Wave: {wave}\n\
         {batch_line}\
         Ledger: `{LEDGER_REL}`\n\n\
         Use execute-parent-task-with-n-steps-v6. GitMap-only search. Relative Git paths only.\n\
         Do not stop Default or v2-3127. Isolated test-create/test-clean only.\n"
    )
}

fn dispatch_prompt(instance_id: &str, prompt: &str) -> AppResult<String> {
    let root = repo_root()?;
    let repo = root.to_string_lossy().to_string();
    let inst = if instance_id.trim().is_empty() {
        "default"
    } else {
        instance_id.trim()
    };
    let queued = repo_db::enqueue_prompt_for_instance(inst, &repo, prompt, None)
        .map_err(|e| AppError::Unknown(format!("enqueue improvements prompt: {e}")))?;
    Ok(queued.id)
}

pub fn plan_flywheel(
    kind: ImprovementKind,
    dry_run: bool,
    instance_id: &str,
) -> AppResult<FlywheelState> {
    let root = repo_root()?;
    let prev_wave = load_state().ok().map(|s| s.wave).unwrap_or(0);
    let mut state = FlywheelState::empty(kind, prev_wave.saturating_add(1).max(1));
    state.last_action = if dry_run {
        "plan-dry-run".to_string()
    } else {
        "plan".to_string()
    };
    state.updated_at = Utc::now().timestamp();
    if !dry_run {
        let body = build_dispatch_prompt(kind.as_str(), kind.discovery_prompt(), None, state.wave);
        state.dispatched_prompt_id = Some(dispatch_prompt(instance_id, &body)?);
        state.last_action = format!("plan-dispatched:{}", kind.discovery_prompt());
    }
    save_state(&root, &state)?;
    Ok(state)
}

pub fn next_batch(dry_run: bool, instance_id: &str) -> AppResult<FlywheelState> {
    let root = repo_root()?;
    let mut state = load_state()?;
    let pending = state
        .batches
        .iter()
        .position(|b| b.status == "pending")
        .ok_or_else(|| AppError::Config("all 10 batches are already dispatched".to_string()))?;
    let batch_index = state.batches[pending].index;
    if dry_run {
        state.last_action = format!("next-dry-run:{batch_index}");
        state.dispatched_prompt_id = None;
        state.updated_at = Utc::now().timestamp();
        return Ok(state);
    }
    let kind = state.kind.clone();
    let wave = state.wave;
    let prompt_body = {
        let batch = &state.batches[pending];
        build_dispatch_prompt(&kind, PROMPT_NEXT, Some(batch), wave)
    };
    state.batches[pending].status = "dispatched".to_string();
    state.next_batch = (batch_index + 1).min(BATCH_COUNT);
    state.completed_batches = state
        .batches
        .iter()
        .filter(|b| b.status == "dispatched" || b.status == "done")
        .count() as u32;
    state.updated_at = Utc::now().timestamp();
    state.dispatched_prompt_id = Some(dispatch_prompt(instance_id, &prompt_body)?);
    state.last_action = format!("next-dispatched:{batch_index}");
    save_state(&root, &state)?;
    Ok(state)
}

pub fn detail_flywheel(dry_run: bool, instance_id: &str) -> AppResult<FlywheelState> {
    let root = repo_root()?;
    let mut state = load_state()?;
    state.updated_at = Utc::now().timestamp();
    if dry_run {
        state.last_action = "detail-dry-run".to_string();
        state.dispatched_prompt_id = None;
    } else {
        let body = build_dispatch_prompt(&state.kind, PROMPT_DETAIL, None, state.wave);
        state.dispatched_prompt_id = Some(dispatch_prompt(instance_id, &body)?);
        state.last_action = format!("detail-dispatched:{PROMPT_DETAIL}");
    }
    save_state(&root, &state)?;
    Ok(state)
}

pub fn check_pending() -> AppResult<FlywheelState> {
    let mut state = load_state()?;
    state.last_action = format!("check:{PROMPT_CHECK}");
    state.updated_at = Utc::now().timestamp();
    state.completed_batches = state
        .batches
        .iter()
        .filter(|b| b.status == "dispatched" || b.status == "done")
        .count() as u32;
    let root = repo_root()?;
    save_state(&root, &state)?;
    Ok(state)
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
         <i>Plan 100, then Next 10 until the wave is done.</i>",
        escape_html(&state.kind, 24),
        state.wave,
        state.completed_batches,
        BATCH_COUNT,
        pending,
        escape_html(next, 80),
        LEDGER_REL
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
    ];
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
                { "text": "📊 Status", "callback_data": "im:st" },
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
        "check" => check_pending(),
        _ => load_state(),
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
        for key in ["im:pl", "im:n10", "im:st", "im:ck"] {
            assert!(key.len() <= 60);
            assert!(parse_improvements_callback(key).is_some());
        }
        assert!(parse_improvements_callback("i:st:abc").is_none());
    }
}
