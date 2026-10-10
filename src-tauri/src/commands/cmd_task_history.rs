use super::*;

#[tauri::command]
pub fn list_task_history(
    offset: u32,
    limit: u32,
) -> Result<modules::task_history_db::TaskHistoryPage, String> {
    modules::task_history_db::list_page(offset, limit.clamp(1, 500))
}

#[tauri::command]
pub fn get_task_history_detail(id: String) -> Result<modules::task_history_db::TaskDetail, String> {
    modules::task_history_db::get_detail(&id)
}
