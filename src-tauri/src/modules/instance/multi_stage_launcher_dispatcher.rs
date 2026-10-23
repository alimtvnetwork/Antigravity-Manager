//! Multi-Stage Launcher Dispatcher (`Imp-628`).
//!
//! Enforces process spawning safety and multi-stage fallback per AGENTS.md:
//! - "Never unconditionally specify `CREATE_BREAKAWAY_FROM_JOB` (`0x01000000`)
//!   without a fallback ladder (`spawn_with_job_breakaway_fallback`). If running inside
//!   a restricted Windows Job Object, `CreateProcessW` fails with `os error 5: Access is denied`."
//! - "Never pass `.cmd` or `.bat` batch files directly as the executable path in
//!   `std::process::Command::new(...)`."
//! - "Multi-instance launchers must implement multi-stage fallback:
//!   target binary -> detected base binary -> Windows Shell dispatch (`cmd.exe /c start`)."

use serde::{Deserialize, Serialize};
use std::fmt;

/// Individual stage attempted during multi-stage process dispatch.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LaunchStage {
    /// Stage 1: Directly invoke the specified target binary.
    Stage1TargetBinary,
    /// Stage 2: Fall back to invoking the auto-detected base installation binary.
    Stage2BaseBinary,
    /// Stage 3: Fall back to Windows Shell dispatch (`cmd.exe /c start "" <args>`).
    Stage3WindowsShell,
}

impl LaunchStage {
    /// Returns true if this is Stage 1 (Target Binary).
    pub fn is_stage1(&self) -> bool {
        matches!(self, Self::Stage1TargetBinary)
    }

    /// Returns true if this is Stage 2 (Base Binary).
    pub fn is_stage2(&self) -> bool {
        matches!(self, Self::Stage2BaseBinary)
    }

    /// Returns true if this is Stage 3 (Windows Shell).
    pub fn is_stage3(&self) -> bool {
        matches!(self, Self::Stage3WindowsShell)
    }

    /// Returns the canonical kebab-case identifier for this stage.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Stage1TargetBinary => "stage_1_target_binary",
            Self::Stage2BaseBinary => "stage_2_base_binary",
            Self::Stage3WindowsShell => "stage_3_windows_shell",
        }
    }
}

impl fmt::Display for LaunchStage {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Mode of Windows Job Object breakaway handling applied to the dispatch plan.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum JobBreakawayMode {
    /// Request `CREATE_BREAKAWAY_FROM_JOB` flag when operating in standard environment.
    BreakawayRequested,
    /// Omit `CREATE_BREAKAWAY_FROM_JOB` flag to avoid OS error 5 inside restricted Job Objects.
    BreakawayDeniedFallback,
    /// Breakaway is not applicable (e.g. delegated via Windows Shell dispatch).
    BreakawayNotApplicable,
}

impl JobBreakawayMode {
    /// Returns true if breakaway was requested.
    pub fn is_breakaway_requested(&self) -> bool {
        matches!(self, Self::BreakawayRequested)
    }

    /// Returns true if breakaway was stepped down due to restricted job object.
    pub fn is_breakaway_denied_fallback(&self) -> bool {
        matches!(self, Self::BreakawayDeniedFallback)
    }

    /// Returns true if breakaway is not applicable for this dispatch mode.
    pub fn is_breakaway_not_applicable(&self) -> bool {
        matches!(self, Self::BreakawayNotApplicable)
    }

    /// Returns the canonical string identifier for this breakaway mode.
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::BreakawayRequested => "breakaway_requested",
            Self::BreakawayDeniedFallback => "breakaway_denied_fallback",
            Self::BreakawayNotApplicable => "breakaway_not_applicable",
        }
    }
}

impl fmt::Display for JobBreakawayMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

/// Security violation or fatal exhaustion encountered during launcher dispatch.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LauncherSecurityViolation {
    /// Prohibited attempt to execute a raw Windows batch script directly.
    DirectBatchFileExecutionDisallowed {
        path: String,
        extension: String,
    },
    /// All configured stages in the fallback ladder failed or were disallowed.
    AllLaunchStagesFailed {
        attempted_stages: Vec<LaunchStage>,
        error_details: Vec<String>,
    },
}

impl LauncherSecurityViolation {
    /// Returns true if this violation is a direct batch file execution attempt.
    pub fn is_batch_disallowed(&self) -> bool {
        matches!(self, Self::DirectBatchFileExecutionDisallowed { .. })
    }

    /// Returns true if this violation indicates all stages were exhausted.
    pub fn is_all_stages_failed(&self) -> bool {
        matches!(self, Self::AllLaunchStagesFailed { .. })
    }
}

impl fmt::Display for LauncherSecurityViolation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::DirectBatchFileExecutionDisallowed { path, extension } => {
                write!(
                    f,
                    "Direct execution of Windows batch file is disallowed: '{}' (extension: '{}')",
                    path, extension
                )
            }
            Self::AllLaunchStagesFailed {
                attempted_stages,
                error_details,
            } => {
                write!(
                    f,
                    "All launch stages failed (attempted {:?}): {}",
                    attempted_stages,
                    error_details.join("; ")
                )
            }
        }
    }
}

impl std::error::Error for LauncherSecurityViolation {}

/// Target configuration and constraints provided to the multi-stage dispatcher.
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct LauncherTargetSpec {
    /// Primary target binary path requested.
    pub target_binary_path: Option<String>,
    /// Secondary detected base binary path used as fallback.
    pub detected_base_binary_path: Option<String>,
    /// Command-line arguments to forward to the launched process.
    pub arguments: Vec<String>,
    /// Whether Stage 3 Windows Shell dispatch is permitted.
    pub allow_shell_fallback: bool,
}

impl LauncherTargetSpec {
    /// Creates an empty target specification.
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets the primary target binary path.
    pub fn with_target(mut self, path: impl Into<String>) -> Self {
        self.target_binary_path = Some(path.into());
        self
    }

    /// Sets the secondary detected base binary path.
    pub fn with_base(mut self, path: impl Into<String>) -> Self {
        self.detected_base_binary_path = Some(path.into());
        self
    }

    /// Sets the arguments vector.
    pub fn with_args<I, S>(mut self, args: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        self.arguments = args.into_iter().map(Into::into).collect();
        self
    }

    /// Sets whether Windows shell dispatch fallback is permitted.
    pub fn with_shell_fallback(mut self, allow: bool) -> Self {
        self.allow_shell_fallback = allow;
        self
    }
}

/// Fully resolved dispatch plan ready for process execution.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchDispatchPlan {
    /// Stage that successfully satisfied the launch requirements.
    pub selected_stage: LaunchStage,
    /// Executable path or command to spawn.
    pub executable: String,
    /// Resolved argument vector.
    pub resolved_args: Vec<String>,
    /// Breakaway policy selected for Job Object management.
    pub breakaway_mode: JobBreakawayMode,
}

impl LaunchDispatchPlan {
    /// Returns true if this plan utilizes Stage 3 Windows Shell dispatch.
    pub fn is_shell_stage(&self) -> bool {
        self.selected_stage.is_stage3()
    }

    /// Returns true if job breakaway was requested.
    pub fn is_breakaway_requested(&self) -> bool {
        self.breakaway_mode.is_breakaway_requested()
    }

    /// Returns true if job breakaway was stepped down due to restricted job object.
    pub fn is_breakaway_denied_fallback(&self) -> bool {
        self.breakaway_mode.is_breakaway_denied_fallback()
    }
}

/// Structured outcome of a multi-stage dispatch evaluation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LaunchDispatchReport {
    /// Whether a viable launch plan was resolved without violations.
    pub is_success: bool,
    /// Resolved dispatch plan if successful.
    pub plan: Option<LaunchDispatchPlan>,
    /// Security violation or failure details if unsuccessful.
    pub violation: Option<LauncherSecurityViolation>,
}

impl LaunchDispatchReport {
    /// Creates a successful launch dispatch report.
    pub fn success(plan: LaunchDispatchPlan) -> Self {
        Self {
            is_success: true,
            plan: Some(plan),
            violation: None,
        }
    }

    /// Creates an unsuccessful launch dispatch report with a security violation.
    pub fn failure(violation: LauncherSecurityViolation) -> Self {
        Self {
            is_success: false,
            plan: None,
            violation: Some(violation),
        }
    }

    /// Returns true if the dispatch plan resolved successfully.
    pub fn is_successful(&self) -> bool {
        self.is_success
    }
}

/// Determines if an executable path represents a prohibited Windows batch script (.cmd or .bat).
pub fn is_disallowed_batch_script(path: &str) -> bool {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return false;
    }
    let lower = trimmed.to_ascii_lowercase();
    lower.ends_with(".cmd") || lower.ends_with(".bat")
}

/// Extracts normalized extension (.cmd or .bat) for a disallowed batch script.
pub fn extract_batch_extension(path: &str) -> String {
    let lower = path.trim().to_ascii_lowercase();
    if lower.ends_with(".cmd") {
        ".cmd".to_string()
    } else {
        ".bat".to_string()
    }
}

/// Resolves breakaway mode based on whether execution is inside a restricted job object.
pub fn resolve_job_breakaway_mode(in_restricted_job_object: bool) -> JobBreakawayMode {
    if in_restricted_job_object {
        JobBreakawayMode::BreakawayDeniedFallback
    } else {
        JobBreakawayMode::BreakawayRequested
    }
}

/// Attempts Stage 1 launch planning using the specified target binary path.
fn resolve_stage1_target(
    target: &str,
    args: &[String],
    in_restricted_job_object: bool,
) -> Result<Option<LaunchDispatchPlan>, LauncherSecurityViolation> {
    if is_disallowed_batch_script(target) {
        return Err(LauncherSecurityViolation::DirectBatchFileExecutionDisallowed {
            path: target.to_string(),
            extension: extract_batch_extension(target),
        });
    }
    if target.is_empty() {
        return Ok(None);
    }
    Ok(Some(LaunchDispatchPlan {
        selected_stage: LaunchStage::Stage1TargetBinary,
        executable: target.to_string(),
        resolved_args: args.to_vec(),
        breakaway_mode: resolve_job_breakaway_mode(in_restricted_job_object),
    }))
}

/// Attempts Stage 2 launch planning using the detected base binary path.
fn resolve_stage2_base(
    base: &str,
    args: &[String],
    in_restricted_job_object: bool,
) -> Result<Option<LaunchDispatchPlan>, LauncherSecurityViolation> {
    if is_disallowed_batch_script(base) {
        return Err(LauncherSecurityViolation::DirectBatchFileExecutionDisallowed {
            path: base.to_string(),
            extension: extract_batch_extension(base),
        });
    }
    if base.is_empty() {
        return Ok(None);
    }
    Ok(Some(LaunchDispatchPlan {
        selected_stage: LaunchStage::Stage2BaseBinary,
        executable: base.to_string(),
        resolved_args: args.to_vec(),
        breakaway_mode: resolve_job_breakaway_mode(in_restricted_job_object),
    }))
}

/// Attempts Stage 3 launch planning using Windows Shell dispatch (`cmd.exe /c start "" <args>`).
fn resolve_stage3_shell(
    allow_shell_fallback: bool,
    args: &[String],
) -> Option<LaunchDispatchPlan> {
    if !allow_shell_fallback {
        return None;
    }
    let mut resolved_args = vec!["/c".to_string(), "start".to_string(), "\"\"".to_string()];
    resolved_args.extend(args.iter().cloned());
    Some(LaunchDispatchPlan {
        selected_stage: LaunchStage::Stage3WindowsShell,
        executable: "cmd.exe".to_string(),
        resolved_args,
        breakaway_mode: JobBreakawayMode::BreakawayNotApplicable,
    })
}

/// Constructs an `AllLaunchStagesFailed` security violation detailing exhausted stages.
fn build_all_stages_failed_violation() -> LauncherSecurityViolation {
    LauncherSecurityViolation::AllLaunchStagesFailed {
        attempted_stages: vec![
            LaunchStage::Stage1TargetBinary,
            LaunchStage::Stage2BaseBinary,
            LaunchStage::Stage3WindowsShell,
        ],
        error_details: vec![
            "Stage 1 target binary not specified or empty".to_string(),
            "Stage 2 base binary not detected or empty".to_string(),
            "Stage 3 Windows Shell fallback disabled".to_string(),
        ],
    }
}

/// Evaluates and plans a multi-stage process launch adhering to job breakaway invariants.
pub fn plan_multi_stage_launch(
    spec: &LauncherTargetSpec,
    in_restricted_job_object: bool,
) -> Result<LaunchDispatchPlan, LauncherSecurityViolation> {
    if let Some(ref target) = spec.target_binary_path {
        if let Some(plan) = resolve_stage1_target(target, &spec.arguments, in_restricted_job_object)? {
            return Ok(plan);
        }
    }
    if let Some(ref base) = spec.detected_base_binary_path {
        if let Some(plan) = resolve_stage2_base(base, &spec.arguments, in_restricted_job_object)? {
            return Ok(plan);
        }
    }
    if let Some(plan) = resolve_stage3_shell(spec.allow_shell_fallback, &spec.arguments) {
        return Ok(plan);
    }
    Err(build_all_stages_failed_violation())
}

/// Convenience dispatcher returning a structured `LaunchDispatchReport`.
pub fn dispatch_and_report(
    spec: &LauncherTargetSpec,
    in_restricted_job_object: bool,
) -> LaunchDispatchReport {
    match plan_multi_stage_launch(spec, in_restricted_job_object) {
        Ok(plan) => LaunchDispatchReport::success(plan),
        Err(violation) => LaunchDispatchReport::failure(violation),
    }
}
