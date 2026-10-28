//! Tauri IPC for the 100-improvements flywheel (CLI/UI/Telegram share the engine).

use crate::error::AppResult;
use crate::modules::improvements::{self, FlywheelState, ImprovementKind};

#[tauri::command]
pub fn improvements_plan(
    kind: Option<String>,
    dry_run: Option<bool>,
    instance_id: Option<String>,
) -> AppResult<FlywheelState> {
    let parsed = ImprovementKind::parse(kind.as_deref().unwrap_or("system"))?;
    let inst = instance_id.unwrap_or_else(|| "default".to_string());
    improvements::plan_flywheel(parsed, dry_run.unwrap_or(false), &inst)
}

#[tauri::command]
pub fn improvements_next(
    dry_run: Option<bool>,
    instance_id: Option<String>,
) -> AppResult<FlywheelState> {
    let inst = instance_id.unwrap_or_else(|| "default".to_string());
    improvements::next_batch(dry_run.unwrap_or(false), &inst)
}

#[tauri::command]
pub fn improvements_status() -> AppResult<FlywheelState> {
    improvements::load_state()
}

#[tauri::command]
pub fn improvements_check() -> AppResult<FlywheelState> {
    improvements::check_pending()
}
