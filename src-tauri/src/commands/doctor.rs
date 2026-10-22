//! Imp-90: System Diagnostics Tauri IPC Command
use crate::modules::cli::doctor::generate_doctor_report;
use crate::modules::cli::doctor_types::DoctorReport;

#[tauri::command]
pub fn agm_run_doctor() -> Result<DoctorReport, String> {
    Ok(generate_doctor_report())
}
