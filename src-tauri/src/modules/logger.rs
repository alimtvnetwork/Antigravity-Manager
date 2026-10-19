use crate::modules::account::get_data_dir;
use std::fs;
use std::path::PathBuf;
use tracing::{error, info, warn};
use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

// Custom local timezone time formatter
struct LocalTimer;

impl tracing_subscriber::fmt::time::FormatTime for LocalTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        let now = chrono::Local::now();
        write!(w, "{}", now.to_rfc3339())
    }
}

/// Short clock for the console so a line fits without splitting the message.
struct ConsoleTimer;

impl tracing_subscriber::fmt::time::FormatTime for ConsoleTimer {
    fn format_time(&self, w: &mut tracing_subscriber::fmt::format::Writer<'_>) -> std::fmt::Result {
        write!(w, "{}", chrono::Local::now().format("%H:%M:%S"))
    }
}

pub fn get_log_dir() -> Result<PathBuf, String> {
    let data_dir = get_data_dir()?;
    let log_dir = data_dir.join("logs");

    if !log_dir.exists() {
        fs::create_dir_all(&log_dir)
            .map_err(|e| format!("Failed to create log directory: {}", e))?;
    }

    Ok(log_dir)
}

static LOGGER_INIT: std::sync::Once = std::sync::Once::new();

/// Initialize the log system (idempotent, runs at most once per process)
pub fn init_logger() {
    LOGGER_INIT.call_once(|| {
        init_logger_inner();
    });
}

fn init_logger_inner() {
    // Capture log macro logs
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(tracing_log::LogTracer::init(), "init");

    let log_dir = match get_log_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("Failed to initialize log directory: {}", e);
            return;
        }
    };

    // 1. Set up file Appender (using tracing-appender for rolling logs)
    // Using a daily rolling strategy here
    let file_appender = tracing_appender::rolling::daily(log_dir, "app.log");
    let (non_blocking, _guard) = tracing_appender::non_blocking(file_appender);

    // 2. Console output layer (route to stderr when --json/-j is passed so stdout stays pure JSON)
    let is_json_cli = std::env::args().any(|a| a == "--json" || a == "-j");
    // Colors only when stdout is a console. Otherwise the layer stays plain text.
    let ansi = crate::modules::delegate_updater::enable_modern_console();
    let console_layer = fmt::Layer::new()
        .compact()
        .with_ansi(ansi)
        .with_writer(move || -> Box<dyn std::io::Write + Send> {
            if is_json_cli {
                Box::new(std::io::stderr())
            } else {
                Box::new(std::io::stdout())
            }
        })
        .with_target(false)
        .with_thread_ids(false)
        .with_level(true)
        .with_timer(ConsoleTimer);

    // 3. File output layer (disable ANSI formatting, use local timezone)
    let file_layer = fmt::Layer::new()
        .with_writer(non_blocking)
        .with_ansi(false)
        .with_target(true)
        .with_level(true)
        .with_timer(LocalTimer);

    // 4. Set filtering layer (default to INFO level to reduce log size)
    let filter_layer = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    // 6. Log bridge layer
    let bridge_layer = crate::modules::log_bridge::TauriLogBridgeLayer::new();

    // 5. Initialize global subscriber (use try_init to avoid crash on repeated initialization)
    // Justification: best-effort call; failure logged without changing control flow
    crate::error::record_ignored(
        tracing_subscriber::registry()
            .with(filter_layer)
            .with(console_layer)
            .with(file_layer)
            .with(bridge_layer)
            .try_init(),
        "registry",
    );

    // Leak _guard to ensure its lifetime lasts until program exit
    // Recommended practice when using tracing_appender::non_blocking (if manual flushing is not needed)
    std::mem::forget(_guard);

    // Install panic hook to capture panic stack traces into file, tracing, and stderr
    std::panic::set_hook(Box::new(|panic_info| {
        let backtrace = std::backtrace::Backtrace::force_capture();
        let location = panic_info
            .location()
            .map(|l| format!("{}:{}:{}", l.file(), l.line(), l.column()))
            .unwrap_or_else(|| "unknown location".to_string());
        let payload = if let Some(s) = panic_info.payload().downcast_ref::<&str>() {
            (*s).to_string()
        } else if let Some(s) = panic_info.payload().downcast_ref::<String>() {
            s.clone()
        } else {
            "unknown panic payload".to_string()
        };

        // Standard OS pipe closure (e.g. piped into `head`, `less`, or closed stdout) is normal
        // CLI termination and must not be logged as a critical panic or crash stack trace.
        if payload.contains("The pipe is being closed")
            || payload.contains("Broken pipe")
            || payload.contains("failed printing to stdout")
            || payload.contains("os error 232")
            || payload.contains("os error 32")
        {
            std::process::exit(0);
        }

        let timestamp = chrono::Local::now().to_rfc3339();
        let pid = std::process::id();
        let thread = std::thread::current();
        let thread_name = thread.name().unwrap_or("unnamed");
        let err_msg = format!(
            "[{timestamp}] [PID: {pid}] CRITICAL PANIC on thread '{thread_name}' at {location}: {payload}\n[STACK TRACE]\n{backtrace:?}\n"
        );
        eprintln!("{}", err_msg);
        tracing::error!("{}", err_msg);
        if let Ok(log_dir) = get_log_dir() {
            // Justification: best-effort file write; failure is logged and surfaces on the next read
            crate::error::record_ignored(
                std::fs::write(log_dir.join("panic.log"), &err_msg),
                "fs::write",
            );
            use std::io::Write;
            if let Ok(mut f) = std::fs::OpenOptions::new()
                .create(true)
                .append(true)
                .open(log_dir.join("crash_stacktrace.log"))
            {
                // Justification: best-effort call; failure logged without changing control flow
                crate::error::record_ignored(writeln!(f, "{err_msg}"), "operation");
            }
        }
        #[cfg(target_os = "macos")]
        {
            if let Ok(home) = std::env::var("HOME") {
                let mac_log_dir =
                    std::path::PathBuf::from(home).join("Library/Logs/AntigravityManager");
                // Justification: best-effort directory creation; later file ops fail loudly if the directory is actually needed
                crate::error::record_ignored(
                    std::fs::create_dir_all(&mac_log_dir),
                    "create_dir_all",
                );
                // Justification: best-effort file write; failure is logged and surfaces on the next read
                crate::error::record_ignored(
                    std::fs::write(mac_log_dir.join("panic.log"), &err_msg),
                    "fs::write",
                );
                use std::io::Write;
                if let Ok(mut f) = std::fs::OpenOptions::new()
                    .create(true)
                    .append(true)
                    .open(mac_log_dir.join("crash_stacktrace.log"))
                {
                    // Justification: best-effort call; failure logged without changing control flow
                    crate::error::record_ignored(writeln!(f, "{err_msg}"), "operation");
                }
            }
        }
    }));

    info!("Log system initialized (Console + File persistence + Panic stack trace capture)");

    // Auto-cleanup logs older than 7 days
    if let Err(e) = cleanup_old_logs(7) {
        warn!("Failed to cleanup old logs: {}", e);
    }
}

/// Cleanup log files older than specified days OR if total size exceeds limit
pub fn cleanup_old_logs(days_to_keep: u64) -> Result<(), String> {
    use std::time::{SystemTime, UNIX_EPOCH};

    let log_dir = get_log_dir()?;
    if !log_dir.exists() {
        return Ok(());
    }

    // Constants for size-based cleanup
    const MAX_TOTAL_SIZE_BYTES: u64 = 1024 * 1024 * 1024; // 1GB
    const TARGET_SIZE_BYTES: u64 = 512 * 1024 * 1024; // 512MB

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|e| format!("Failed to get system time: {}", e))?
        .as_secs();

    let cutoff_time = now.saturating_sub(days_to_keep * 24 * 60 * 60);

    let mut entries_info = Vec::new();
    let entries =
        fs::read_dir(&log_dir).map_err(|e| format!("Failed to read log directory: {}", e))?;

    for entry in entries {
        if let Ok(entry) = entry {
            let path = entry.path();
            if !path.is_file() {
                continue;
            }

            if let Ok(metadata) = fs::metadata(&path) {
                let modified = metadata.modified().unwrap_or(SystemTime::now());
                let modified_secs = modified
                    .duration_since(UNIX_EPOCH)
                    .map(|d| d.as_secs())
                    .unwrap_or(0);

                let size = metadata.len();
                entries_info.push((path, size, modified_secs));
            }
        }
    }

    let mut deleted_count = 0;
    let mut total_size_freed = 0u64;

    // 1. First pass: Delete files older than cutoff_time
    let mut remaining_entries = Vec::new();
    for (path, size, modified_secs) in entries_info {
        if modified_secs < cutoff_time {
            if let Err(e) = fs::remove_file(&path) {
                warn!("Failed to delete old log file {:?}: {}", path, e);
                remaining_entries.push((path, size, modified_secs));
            } else {
                deleted_count += 1;
                total_size_freed += size;
                info!("Deleted old log file (expired): {:?}", path.file_name());
            }
        } else {
            remaining_entries.push((path, size, modified_secs));
        }
    }

    // 2. Second pass: If total size still exceeds limit, delete oldest files
    let mut current_total_size: u64 = remaining_entries.iter().map(|(_, size, _)| *size).sum();

    if current_total_size > MAX_TOTAL_SIZE_BYTES {
        info!(
            "Log directory size ({} MB) exceeds limit (1024 MB), starting size-based cleanup...",
            current_total_size / 1024 / 1024
        );

        // Sort remaining entries by modification time (oldest first)
        remaining_entries.sort_by_key(|(_, _, modified)| *modified);

        for (path, size, _) in remaining_entries {
            if current_total_size <= TARGET_SIZE_BYTES {
                break;
            }

            // Try to delete. Skip if it's the most recent file and it fails (might be active)
            if let Err(e) = fs::remove_file(&path) {
                warn!(
                    "Failed to delete log file during size cleanup {:?}: {}",
                    path, e
                );
            } else {
                deleted_count += 1;
                total_size_freed += size;
                current_total_size -= size;
                info!("Deleted log file (size limit): {:?}", path.file_name());
            }
        }
    }

    if deleted_count > 0 {
        let size_mb = total_size_freed as f64 / 1024.0 / 1024.0;
        info!(
            "Log cleanup completed: deleted {} files, freed {:.2} MB space",
            deleted_count, size_mb
        );
    }

    Ok(())
}

/// Clear log cache (using truncation mode to keep file handles valid)
pub fn clear_logs() -> Result<(), String> {
    let log_dir = get_log_dir()?;
    if log_dir.exists() {
        // Iterate through all files in directory and truncate instead of deleting directory
        let entries =
            fs::read_dir(&log_dir).map_err(|e| format!("Failed to read log directory: {}", e))?;
        for entry in entries {
            if let Ok(entry) = entry {
                let path = entry.path();
                if path.is_file() {
                    // Open file in truncation mode, set size to 0
                    // Justification: best-effort call; failure logged without changing control flow
                    crate::error::record_ignored(
                        fs::OpenOptions::new().write(true).truncate(true).open(path),
                        "new",
                    );
                }
            }
        }
    }
    Ok(())
}

/// Log info message (backward compatibility)
pub fn log_info(message: &str) {
    info!("{}", message);
}

/// Log warn message (backward compatibility)
pub fn log_warn(message: &str) {
    warn!("{}", message);
}

/// Log error message (backward compatibility)
pub fn log_error(message: &str) {
    error!("{}", message);
}

/// Mask sensitive email addresses for logs, telemetry, and external notifications.
/// Example: `riseup.asia.team@gmail.com` -> `rise***@***`
pub fn mask_email(email: &str) -> String {
    let trimmed = email.trim();
    if trimmed.is_empty() {
        return "***".to_string();
    }
    if let Some((local, _domain)) = trimmed.split_once('@') {
        if local.len() <= 2 {
            format!("{}***@***", &local[..1.min(local.len())])
        } else {
            let shown = 4.min(local.len().saturating_sub(2)).max(1);
            format!("{}***@***", &local[..shown])
        }
    } else if trimmed.len() <= 3 {
        "***".to_string()
    } else {
        format!("{}***", &trimmed[..3])
    }
}

fn format_pid_value(process_pid: Option<u32>) -> String {
    match process_pid {
        Some(pid) => pid.to_string(),
        None => "none".to_string(),
    }
}

/// Format structured audit log string for instance prompt and project liveness evaluation
pub fn format_instance_prompt_audit(
    instance_id: &str,
    resolved_name: &str,
    project_name: &str,
    repo_path: &str,
    db_path_evaluated: &str,
    process_pid: Option<u32>,
    criteria_evaluated: &str,
    is_running: bool,
    rationale: &str,
) -> String {
    let pid_str = format_pid_value(process_pid);
    format!(
        "[InstancePromptAudit] instance_id='{instance_id}' resolved_name='{resolved_name}' project='{project_name}' repo_path='{repo_path}' db_path='{db_path_evaluated}' pid={pid_str} criteria='{criteria_evaluated}' is_running={is_running} rationale='{rationale}'"
    )
}

/// Emit structured audit log for instance prompt and project liveness evaluation
pub fn log_instance_prompt_audit(
    instance_id: &str,
    resolved_name: &str,
    project_name: &str,
    repo_path: &str,
    db_path_evaluated: &str,
    process_pid: Option<u32>,
    criteria_evaluated: &str,
    is_running: bool,
    rationale: &str,
) {
    let line = format_instance_prompt_audit(
        instance_id,
        resolved_name,
        project_name,
        repo_path,
        db_path_evaluated,
        process_pid,
        criteria_evaluated,
        is_running,
        rationale,
    );
    info!("{line}");
}

static FILTER_NOISY_TELEMETRY: std::sync::atomic::AtomicBool =
    std::sync::atomic::AtomicBool::new(true);

/// Enable or disable filtering of noisy system telemetry (HEARTBEAT, TOKEN_USAGE, TELEMETRY).
pub fn set_filter_noisy_telemetry(enabled: bool) {
    FILTER_NOISY_TELEMETRY.store(enabled, std::sync::atomic::Ordering::SeqCst);
}

/// Check if noisy system telemetry filtering is enabled.
pub fn is_filter_noisy_telemetry_enabled() -> bool {
    FILTER_NOISY_TELEMETRY.load(std::sync::atomic::Ordering::SeqCst)
}

/// Check if a log line or message represents noisy background telemetry.
pub fn is_noisy_system_telemetry(msg: &str) -> bool {
    let lower = msg.to_ascii_lowercase();
    lower.contains("heartbeat")
        || lower.contains("token_usage")
        || lower.contains("telemetry")
        || lower.contains("[heartbeat]")
        || lower.contains("[token_usage]")
        || lower.contains("[telemetry]")
}

/// Check if a log message should be filtered out from active consoles and streams.
pub fn should_filter_log(msg: &str) -> bool {
    if is_filter_noisy_telemetry_enabled() {
        is_noisy_system_telemetry(msg)
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_noisy_system_telemetry() {
        assert!(is_noisy_system_telemetry(
            "Heartbeat ping received from node W1"
        ));
        assert!(is_noisy_system_telemetry("TOKEN_USAGE: 1250 tokens used"));
        assert!(is_noisy_system_telemetry(
            "[TELEMETRY] Reported 5 instances"
        ));
        assert!(!is_noisy_system_telemetry(
            "Switched account to user@example.com"
        ));
        assert!(!is_noisy_system_telemetry("Prompt dispatched successfully"));
    }

    #[test]
    fn test_should_filter_log() {
        set_filter_noisy_telemetry(true);
        assert!(should_filter_log("HEARTBEAT tick"));
        assert!(!should_filter_log("Normal operation completed"));

        set_filter_noisy_telemetry(false);
        assert!(!should_filter_log("HEARTBEAT tick"));
        set_filter_noisy_telemetry(true); // reset
    }
}
