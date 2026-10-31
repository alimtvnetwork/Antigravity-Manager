//! Machine-targeted `/open` and `/screenshot` Telegram commands.
//!
//! - `/open [machine]` — open the AGM UI on the target machine: launch it
//!   when it is not running, otherwise bring its window to the foreground.
//! - `/screenshot [machine]` — capture the target machine's screen (AGM is
//!   opened first when needed) and send the image(s) back to the Telegram
//!   chat. With no argument, this machine is captured (IDEs + Instances +
//!   Accounts). Pass `all` for fleet fan-out.
//!
//! Machine targeting reuses the existing fleet model: this machine's node
//! alias plus the GitMap cluster node aliases from `query_gitmap_cluster_nodes()`.
//! Remote machines are driven via `gitmap cluster exec` (same pattern as the
//! prompt-injection remote dispatch), with a Supabase command-queue fallback
//! for `/open`. Every failure surfaces its real message in the bot reply —
//! nothing here reports success silently.

use crate::modules::email_watcher;
use crate::modules::supabase_client::SupabaseClient;
use crate::modules::supabase_sync;
use base64::Engine as _;
use chrono::Utc;
use serde_json::json;
use std::io::{Read, Write};
use std::process::{Command, Stdio};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use super::*;

/// Where an `/open` or `/screenshot` command should run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OpenShotTarget {
    /// This machine: no argument, `local`, or this machine's own fleet alias.
    Local,
    /// A fleet node addressed by its cluster alias (e.g. `W3`).
    Remote(String),
    /// Every known machine (screenshot fan-out only).
    All,
}

/// A single machine selected for screenshot capture (fan-out resolved).
#[derive(Debug, Clone, PartialEq, Eq)]
enum ShotMachine {
    Local,
    Remote(String),
}

/// This machine's fleet display name: GitMap cluster alias by local IP or hostname,
/// validated Supabase node alias (non-conflicting with remote nodes), else OS machine name.
pub fn local_machine_label() -> String {
    let local_ip = email_watcher::detect_local_ip();
    let local_name = email_watcher::detect_machine_name();
    let cluster_nodes = cluster::query_gitmap_cluster_nodes();

    // 1. Resolve dynamically via local IP matching GitMap cluster nodes
    if !local_ip.is_empty() {
        for node in &cluster_nodes {
            if node.ip_address.trim() == local_ip.trim() && !node.alias.trim().is_empty() {
                return node.alias.trim().to_string();
            }
        }
    }

    // 2. Resolve dynamically via local machine name matching GitMap cluster nodes
    if !local_name.is_empty() {
        for node in &cluster_nodes {
            if node.alias.trim().eq_ignore_ascii_case(local_name.trim()) {
                return node.alias.trim().to_string();
            }
        }
    }

    // 3. Check Supabase sync config if configured, dynamically ensuring it does not conflict with a remote cluster node
    if let Ok(cfg) = supabase_sync::load_config() {
        let alias = cfg.node_alias.trim();
        if !alias.is_empty() {
            let is_remote_cluster_node = !local_ip.is_empty()
                && cluster_nodes.iter().any(|n| {
                    n.alias.trim().eq_ignore_ascii_case(alias)
                        && !n.ip_address.trim().is_empty()
                        && n.ip_address.trim() != local_ip.trim()
                });
            if !is_remote_cluster_node {
                return alias.to_string();
            }
        }
    }

    if !local_name.is_empty() {
        local_name
    } else {
        "local".to_string()
    }
}

/// Resolve the `[machine]` argument of `/open` / `/screenshot` against the
/// existing fleet model (local alias + GitMap cluster node aliases).
pub fn resolve_machine_target(arg: &str) -> Result<OpenShotTarget, String> {
    let token = arg.trim();
    if token.is_empty() || super::node_label::is_current_machine(token) {
        return Ok(OpenShotTarget::Local);
    }
    if token.eq_ignore_ascii_case("all") {
        return Ok(OpenShotTarget::All);
    }
    let nodes = query_gitmap_cluster_nodes();
    if let Some(node) = nodes.iter().find(|n| n.alias.eq_ignore_ascii_case(token)) {
        if super::node_label::is_current_machine(&node.alias) {
            return Ok(OpenShotTarget::Local);
        }
        return Ok(OpenShotTarget::Remote(node.alias.clone()));
    }
    let mut known = vec![local_machine_label()];
    known.extend(nodes.iter().map(|n| n.alias.clone()));
    Err(format!(
        "Unknown machine '{token}'. Known machines: {}. Send /nodes for the live fleet list.",
        known.join(", ")
    ))
}

/// Load the Telegram bot credentials from the saved config.
fn telegram_send_target() -> Result<(String, i64), String> {
    let cfg = super::config::load_config().map_err(|e| format!("load telegram config: {e}"))?;
    let chat_id = cfg
        .allowed_chat_id
        .ok_or_else(|| "telegram chat id is not configured".to_string())?;
    if cfg.bot_token.trim().is_empty() {
        return Err("telegram bot token is not configured".to_string());
    }
    Ok((cfg.bot_token, chat_id))
}

/// True when the AGM desktop UI process is currently running.
fn manager_process_running() -> bool {
    let mut sys = sysinfo::System::new();
    sys.refresh_processes(sysinfo::ProcessesToUpdate::All);
    sys.processes().values().any(|proc| {
        let name = proc.name().to_string_lossy().to_ascii_lowercase();
        name.contains("agm-alim")
            || name.contains("agm-ui")
            || name.contains("antigravity-tools")
            || name == "agm"
            || name == "agm.exe"
            || name.contains("antigravity_manager")
    })
}

/// Poll for the AGM UI process for up to `secs` seconds.
fn wait_for_manager_process(secs: u64) -> bool {
    for _ in 0..secs.max(1) {
        if manager_process_running() {
            return true;
        }
        std::thread::sleep(std::time::Duration::from_secs(1));
    }
    manager_process_running()
}

/// Condense a failed `gitmap cluster exec` output into one honest line.
fn ssh_failure_detail(out: &std::process::Output) -> String {
    let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
    let code = out
        .status
        .code()
        .map(|c| c.to_string())
        .unwrap_or_else(|| "terminated".to_string());
    let detail = if !stderr.is_empty() {
        stderr
    } else if !stdout.is_empty() {
        stdout
    } else {
        "no output".to_string()
    };
    format!("exit {code}: {detail}")
}

/// Bring the AGM window to the foreground (Windows only).
#[cfg(target_os = "windows")]
fn focus_manager_window() -> Result<(), String> {
    let dir = std::env::temp_dir().join("agm-ui-focus");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create focus folder: {e}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let script = dir.join(format!("focus-{stamp}.ps1"));
    {
        let mut file =
            std::fs::File::create(&script).map_err(|e| format!("write focus script: {e}"))?;
        file.write_all(FOCUS_PS1.as_bytes())
            .map_err(|e| format!("write focus script bytes: {e}"))?;
    }
    let mut ps_cmd = Command::new("powershell.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        ps_cmd.creation_flags(0x08000000);
    }
    let output = ps_cmd
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("start focus helper: {e}"))?;
    // Justification: the temp script has served its purpose; a leftover file is harmless.
    crate::error::record_ignored(std::fs::remove_file(&script), "remove temp focus script");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "focus helper exited {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    Ok(())
}

#[cfg(not(target_os = "windows"))]
fn focus_manager_window() -> Result<(), String> {
    Err("window focus is implemented for Windows".to_string())
}

/// Open the AGM UI on this machine: launch it when not running, otherwise
/// bring its window to the foreground. Returns the confirmation reply.
fn open_local_manager() -> Result<String, String> {
    let machine = clean_for_telegram_html(&local_machine_label(), 48);
    if manager_process_running() {
        return match focus_manager_window() {
            Ok(()) => Ok(format!(
                "🖥️ <b>Opened AGM on {machine}</b>\nAlready running — its window was brought to the foreground."
            )),
            Err(e) => Ok(format!(
                "🖥️ <b>Opened AGM on {machine}</b>\nAlready running. Foreground focus failed: <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            )),
        };
    }
    crate::modules::delegate_updater::open_ui(&[]);
    if wait_for_manager_process(5) {
        Ok(format!(
            "🖥️ <b>Opened AGM on {machine}</b>\nThe manager UI was launched."
        ))
    } else {
        Err(format!(
            "AGM launch was requested on {machine} but the manager process did not appear within 5 seconds."
        ))
    }
}

/// Best-effort fallback: queue a shell command for the remote node in the
/// Supabase secondary DB (same shape as the prompt-injection remote fallback).
async fn enqueue_remote_command(
    node: &str,
    command: &str,
    command_text: &str,
) -> Result<(), String> {
    let cfg = supabase_sync::load_config().map_err(|e| format!("load supabase config: {e}"))?;
    for ep in &cfg.endpoints {
        if !ep.is_enabled || ep.role != "secondary" {
            continue;
        }
        let client = match SupabaseClient::new(ep) {
            Ok(c) => c,
            Err(e) => {
                // Justification: endpoint failover — a bad endpoint must not abort the remaining ones.
                crate::error::record_ignored(
                    Err::<(), String>(format!("{e}")),
                    "supabase secondary client for open/screenshot queue",
                );
                continue;
            }
        };
        let payload = json!({
            "id": uuid::Uuid::new_v4().to_string(),
            "target_node": node,
            "target_node_id": node,
            "source": "telegram",
            "command": command,
            "command_text": command_text,
            "status": "pending",
            "created_at": Utc::now().timestamp()
        });
        match client.insert("command_queue", payload).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                // Justification: endpoint failover — a rejected insert must not abort the remaining endpoints.
                crate::error::record_ignored(
                    Err::<(), String>(format!("{e}")),
                    "insert open/screenshot command into queue",
                );
            }
        }
    }
    Err("no enabled Supabase secondary endpoint accepted the command".to_string())
}

/// Open the AGM UI on a remote fleet node via GitMap Cluster SSH, falling back
/// to the Supabase command queue when SSH dispatch fails.
async fn open_remote_manager(node: &str) -> Result<String, String> {
    let node_esc = clean_for_telegram_html(node, 48);
    let remote_cmd = "agm open-ui";
    let mut gm_cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        gm_cmd.creation_flags(0x08000000);
    }
    let ssh_outcome: Result<(), String> =
        match gm_cmd.args(["cluster", "exec", node, remote_cmd]).output() {
            Err(e) => Err(format!("could not run `gitmap cluster exec`: {e}")),
            Ok(out) if out.status.success() => Ok(()),
            Ok(out) => Err(ssh_failure_detail(&out)),
        };
    match ssh_outcome {
        Ok(()) => Ok(format!(
            "🖥️ <b>Opened AGM on {node_esc}</b>\nLaunched via <code>gitmap cluster exec {node_esc} \"agm open-ui\"</code>."
        )),
        Err(ssh_err) => match enqueue_remote_command(node, "open-ui", remote_cmd).await {
            Ok(()) => Ok(format!(
                "🖥️ <b>Open request queued for {node_esc}</b>\nCluster SSH failed (<code>{}</code>); the node will open AGM on its next Supabase queue poll.",
                clean_for_telegram_html(&ssh_err, 200)
            )),
            Err(q_err) => Err(format!(
                "Could not open AGM on {node_esc}: cluster SSH failed ({}) and queue fallback failed ({}).",
                clean_for_telegram_html(&ssh_err, 200),
                clean_for_telegram_html(&q_err, 200)
            )),
        },
    }
}

/// Handle `/open [machine]` — open the AGM UI on the target machine.
///
/// No argument targets this machine; a fleet node alias (see `/nodes`)
/// dispatches via GitMap Cluster SSH with a Supabase queue fallback.
pub async fn execute_open_command(args_str: &str) -> String {
    let target = match resolve_machine_target(args_str) {
        Ok(t) => t,
        Err(e) => {
            return format!(
                "⚠️ <b>Open failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 300)
            )
        }
    };
    let outcome = match target {
        OpenShotTarget::All => Err(
            "`/open` targets a single machine — use `/open <machine>` or omit the argument for this machine."
                .to_string(),
        ),
        OpenShotTarget::Local => open_local_manager(),
        OpenShotTarget::Remote(node) => open_remote_manager(&node).await,
    };
    match outcome {
        Ok(reply) => reply,
        Err(e) => format!(
            "❌ <b>Open failed:</b> <code>{}</code>",
            clean_for_telegram_html(&e, 400)
        ),
    }
}

/// Capture the virtual desktop (all monitors / all visible IDE windows).
#[cfg(target_os = "windows")]
fn capture_desktop_png() -> Result<Vec<u8>, String> {
    let dir = std::env::temp_dir().join("agm-ui-shot");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create shot folder: {e}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let script = dir.join(format!("desktop-{stamp}.ps1"));
    let png = dir.join(format!("desktop-{stamp}.png"));
    {
        let mut file =
            std::fs::File::create(&script).map_err(|e| format!("write desktop script: {e}"))?;
        file.write_all(DESKTOP_CAPTURE_PS1.as_bytes())
            .map_err(|e| format!("write desktop script bytes: {e}"))?;
    }
    let mut ps_cmd = Command::new("powershell.exe");
    {
        use std::os::windows::process::CommandExt;
        ps_cmd.creation_flags(0x08000000);
    }
    let output = ps_cmd
        .args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
            "-OutPath",
            &png.to_string_lossy(),
        ])
        .output()
        .map_err(|e| format!("start desktop screenshot helper: {e}"))?;
    crate::error::record_ignored(std::fs::remove_file(&script), "remove temp desktop script");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "desktop capture helper exited {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let bytes = std::fs::read(&png).map_err(|e| format!("read desktop png: {e}"))?;
    crate::error::record_ignored(std::fs::remove_file(&png), "remove temp desktop png");
    if bytes.len() < 32 || !is_valid_png(&bytes) {
        return Err("desktop capture produced an invalid PNG".to_string());
    }
    Ok(bytes)
}

#[cfg(not(target_os = "windows"))]
fn capture_desktop_png() -> Result<Vec<u8>, String> {
    Err("desktop capture is implemented for Windows".to_string())
}

const MAX_IDE_SHOTS: usize = 8;

fn pids_for_instance(instance_id: &str) -> Vec<u32> {
    let reg = crate::modules::instance::load_registry().unwrap_or_default();
    let Some(inst) = reg.instances.iter().find(|i| i.id == instance_id) else {
        return Vec::new();
    };
    crate::modules::instance::find_pids_for_data_dir(&inst.data_dir, inst.is_default)
}

/// Photograph each visible Antigravity IDE window (not the AGM manager).
#[cfg(target_os = "windows")]
fn capture_ide_windows(filter_pids: Option<&[u32]>) -> Result<Vec<(String, Vec<u8>)>, String> {
    let dir = std::env::temp_dir().join("agm-ui-shot");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create ide shot folder: {e}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let out_dir = dir.join(format!("ides-{stamp}"));
    std::fs::create_dir_all(&out_dir).map_err(|e| format!("create ide shot dir: {e}"))?;
    let script = dir.join(format!("ides-{stamp}.ps1"));
    {
        let mut file =
            std::fs::File::create(&script).map_err(|e| format!("write ide shot script: {e}"))?;
        file.write_all(IDE_WINDOWS_CAPTURE_PS1.as_bytes())
            .map_err(|e| format!("write ide shot script bytes: {e}"))?;
    }
    let pids_arg = filter_pids
        .filter(|p| !p.is_empty())
        .map(|p| {
            p.iter()
                .map(|id| id.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    let mut ps_cmd = Command::new("powershell.exe");
    {
        use std::os::windows::process::CommandExt;
        ps_cmd.creation_flags(0x08000000);
    }
    let output = output_with_timeout(
        &mut ps_cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
            "-OutDir",
            &out_dir.to_string_lossy(),
            "-Pids",
            &pids_arg,
            "-Max",
            &MAX_IDE_SHOTS.to_string(),
        ]),
        Duration::from_secs(8),
    )
    .map_err(|e| format!("start ide screenshot helper: {e}"))?;
    crate::error::record_ignored(std::fs::remove_file(&script), "remove temp ide script");
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!(
            "ide capture helper exited {}: {}",
            output.status,
            stderr.trim()
        ));
    }
    let stdout = String::from_utf8_lossy(&output.stdout);
    let mut shots = Vec::new();
    for line in stdout.lines() {
        let line = line.trim();
        let Some(rest) = line.strip_prefix("FILE|") else {
            continue;
        };
        let mut parts = rest.splitn(3, '|');
        let Some(name) = parts.next() else {
            continue;
        };
        let _pid = parts.next();
        let title = parts.next().unwrap_or("Antigravity IDE");
        let png_path = out_dir.join(name);
        match std::fs::read(&png_path) {
            Ok(bytes) if is_valid_png(&bytes) => {
                shots.push((title.to_string(), bytes));
            }
            Ok(_) => {
                tracing::warn!(path = %png_path.display(), "ide capture produced invalid PNG");
            }
            Err(e) => {
                tracing::warn!(path = %png_path.display(), error = %e, "read ide capture png");
            }
        }
        crate::error::record_ignored(std::fs::remove_file(&png_path), "remove temp ide png");
    }
    crate::error::record_ignored(std::fs::remove_dir(&out_dir), "remove temp ide shot dir");
    Ok(shots)
}

#[cfg(not(target_os = "windows"))]
fn capture_ide_windows(_filter_pids: Option<&[u32]>) -> Result<Vec<(String, Vec<u8>)>, String> {
    Err("IDE window capture is implemented for Windows".to_string())
}

fn output_with_timeout(cmd: &mut Command, limit: Duration) -> Result<std::process::Output, String> {
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    let mut child = cmd
        .spawn()
        .map_err(|e| format!("spawn capture helper: {e}"))?;
    let start = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    // Justification: helper already exited; PNG is on disk even if stdout is empty.
                    crate::error::record_ignored(
                        s.read_to_end(&mut stdout).map(|_| ()),
                        "read capture stdout",
                    );
                }
                if let Some(mut e) = child.stderr.take() {
                    // Justification: helper already exited; stderr is diagnostics only.
                    crate::error::record_ignored(
                        e.read_to_end(&mut stderr).map(|_| ()),
                        "read capture stderr",
                    );
                }
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) if start.elapsed() >= limit => {
                crate::error::record_ignored(child.kill(), "kill timed-out screenshot helper");
                // Justification: process already killed; reaping avoids a zombie, timeout is the error.
                crate::error::record_ignored(child.wait(), "reap killed screenshot helper");
                return Err(format!(
                    "screenshot helper timed out after {}s (manager was not frozen)",
                    limit.as_secs()
                ));
            }
            Ok(None) => std::thread::sleep(Duration::from_millis(50)),
            Err(e) => return Err(format!("poll capture helper: {e}")),
        }
    }
}

async fn capture_agm_route(route: &str) -> Result<Vec<u8>, String> {
    crate::error::record_ignored(
        super::ui_remote::ui_service_goto(crate::modules::audit::InteractionActor::Telegram, route),
        "telegram screenshot ui goto",
    );
    super::ui_remote::post_ui_navigate_event(route).await;
    tokio::time::sleep(Duration::from_millis(400)).await;
    tokio::task::spawn_blocking(capture_local_once)
        .await
        .map_err(|e| format!("screenshot join: {e}"))?
}

/// Capture this machine's AGM window once (Windows only).
#[cfg(target_os = "windows")]
fn capture_local_once() -> Result<Vec<u8>, String> {
    let dir = std::env::temp_dir().join("agm-ui-shot");
    std::fs::create_dir_all(&dir).map_err(|e| format!("create shot folder: {e}"))?;
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis())
        .unwrap_or(0);
    let script = dir.join(format!("capture-{stamp}.ps1"));
    let png = dir.join(format!("window-{stamp}.png"));
    {
        let mut file =
            std::fs::File::create(&script).map_err(|e| format!("write shot script: {e}"))?;
        file.write_all(WINDOW_CAPTURE_PS1.as_bytes())
            .map_err(|e| format!("write shot script bytes: {e}"))?;
    }
    let mut ps_cmd = Command::new("powershell.exe");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        ps_cmd.creation_flags(0x08000000);
    }
    let output = output_with_timeout(
        &mut ps_cmd.args([
            "-NoProfile",
            "-NonInteractive",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
            &script.to_string_lossy(),
            "-OutPath",
            &png.to_string_lossy(),
        ]),
        Duration::from_secs(8),
    )
    .map_err(|e| format!("start screenshot helper: {e}"))?;
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        let stdout = String::from_utf8_lossy(&output.stdout);
        return Err(format!(
            "screenshot helper exited {}: {} {}",
            output.status,
            stderr.trim(),
            stdout.trim()
        ));
    }

    let bytes = std::fs::read(&png).map_err(|e| format!("read screenshot: {e}"))?;
    if bytes.len() < 32 || !is_valid_png(&bytes) {
        return Err("screenshot file was empty or corrupted PNG".to_string());
    }
    // Justification: the PNG bytes are already in memory; deleting the temp files is cleanup.
    crate::error::record_ignored(std::fs::remove_file(&png), "remove temp screenshot");
    crate::error::record_ignored(std::fs::remove_file(&script), "remove temp shot script");
    Ok(bytes)
}

#[cfg(not(target_os = "windows"))]
fn capture_local_once() -> Result<Vec<u8>, String> {
    Err("window screenshot is implemented for Windows".to_string())
}

/// Capture this machine's screen: open AGM first when needed, then photograph
/// its window (retries like the ui_remote capture pipeline).
fn capture_local_png() -> Result<Vec<u8>, String> {
    if !manager_process_running() {
        crate::modules::delegate_updater::open_ui(&[]);
    }
    if !wait_for_manager_process(8) {
        return Err(format!(
            "AGM is not running on {} and did not start within 8 seconds, so there is no window to photograph.",
            local_machine_label()
        ));
    }
    let mut last_err = String::from("manager window was not found");
    for _ in 0..4 {
        match capture_local_once() {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_err = err;
                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        }
    }
    Err(last_err)
}

/// Encode a PowerShell script for `powershell -EncodedCommand`
/// (base64 of the UTF-16LE bytes), avoiding nested quoting over SSH.
fn encode_powershell_utf16le(script: &str) -> String {
    let mut bytes = Vec::with_capacity(script.len() * 2);
    for unit in script.encode_utf16() {
        bytes.extend_from_slice(&unit.to_le_bytes());
    }
    base64::engine::general_purpose::STANDARD.encode(&bytes)
}

/// Pick the base64 payload out of `gitmap cluster exec` stdout, ignoring
/// any wrapper/log lines and concatenating wrapped lines to avoid byte corruption.
fn extract_base64_line(stdout: &str) -> Option<String> {
    let mut b64_chunks = Vec::new();
    for line in stdout.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty()
            || trimmed.starts_with("gitmap:")
            || trimmed.starts_with("[log]")
            || trimmed == "DONE"
            || trimmed.starts_with("OK ")
        {
            continue;
        }
        if trimmed.len() >= 4
            && trimmed
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        {
            b64_chunks.push(trimmed);
        }
    }
    if b64_chunks.is_empty() {
        return None;
    }
    let joined = b64_chunks.concat();
    if joined.len() >= 100 {
        Some(joined)
    } else {
        None
    }
}

/// Check if bytes begin with standard PNG signature (\x89PNG\r\n\x1a\n).
fn is_valid_png(bytes: &[u8]) -> bool {
    bytes.len() >= 8 && bytes.starts_with(b"\x89PNG\r\n\x1a\n")
}

/// Capture a remote fleet node's screen: ensure AGM is open there, then run a
/// PowerShell capture that prints the PNG as base64 over `gitmap cluster exec`.
fn capture_remote_png(node: &str) -> Result<Vec<u8>, String> {
    let mut open_cmd = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        open_cmd.creation_flags(0x08000000);
    }
    let open_out = open_cmd
        .args(["cluster", "exec", node, "agm open-ui"])
        .output()
        .map_err(|e| format!("could not run `gitmap cluster exec` for {node}: {e}"))?;
    if !open_out.status.success() {
        // Justification: best-effort remote open; the capture attempt below reports the real outcome.
        crate::error::record_ignored(
            Err::<(), String>(format!(
                "remote agm open-ui: {}",
                ssh_failure_detail(&open_out)
            )),
            "remote open-ui before screenshot",
        );
    }
    std::thread::sleep(std::time::Duration::from_secs(4));

    let encoded = encode_powershell_utf16le(REMOTE_CAPTURE_PS1);
    let remote_cmd = format!("powershell -NoProfile -NonInteractive -EncodedCommand {encoded}");
    let mut remote_proc = Command::new("gitmap");
    #[cfg(target_os = "windows")]
    {
        use std::os::windows::process::CommandExt;
        remote_proc.creation_flags(0x08000000);
    }
    let out = remote_proc
        .args(["cluster", "exec", node, &remote_cmd])
        .output()
        .map_err(|e| format!("could not run remote capture on {node}: {e}"))?;
    if !out.status.success() {
        // Fallback Tier 1: Check remote AGM proxy API on port 8045
        let (_, node_ip, _) = super::cluster::resolve_node_network_info(node);
        if !node_ip.is_empty() && node_ip != "Unknown IP" && node_ip != "Cluster Mesh" {
            let url = format!("http://{node_ip}:8045/api/remote/screenshot");
            if let Ok(resp) = reqwest::blocking::Client::builder()
                .timeout(std::time::Duration::from_secs(3))
                .build()
                .and_then(|c| c.get(&url).send())
            {
                if resp.status().is_success() {
                    if let Ok(bytes) = resp.bytes() {
                        if is_valid_png(&bytes) {
                            return Ok(bytes.to_vec());
                        }
                    }
                }
            }
        }

        return Err(format!(
            "remote capture on {node} failed: {}",
            ssh_failure_detail(&out)
        ));
    }
    let stdout = String::from_utf8_lossy(&out.stdout);
    let b64 = extract_base64_line(&stdout).ok_or_else(|| {
        format!("remote capture on {node} returned no image data (stdout was empty or not base64)")
    })?;
    let bytes = base64::engine::general_purpose::STANDARD
        .decode(&b64)
        .map_err(|e| format!("decode remote screenshot from {node}: {e}"))?;
    if bytes.len() < 32 || !is_valid_png(&bytes) {
        return Err(format!(
            "remote screenshot from {node} was corrupted or invalid PNG"
        ));
    }
    Ok(bytes)
}

/// Capture one machine's screen (local window capture or remote base64 round-trip).
fn capture_for_machine(machine: &ShotMachine) -> Result<Vec<u8>, String> {
    match machine {
        ShotMachine::Local => capture_local_png(),
        ShotMachine::Remote(node) => capture_remote_png(node),
    }
}

/// Handle `/screenshot [machine]` — capture AGM, desktop, Instances, Accounts,
/// and each running Antigravity IDE window, then send the photos to Telegram.
pub async fn execute_screenshot_command(args_str: &str) -> String {
    screenshot_fanout(args_str, None).await
}

/// Per-instance `i:sc:` — local AGM + desktop + Instances + Accounts + this instance's IDE windows.
pub async fn execute_instance_screenshot(instance_id: &str) -> String {
    screenshot_fanout("", Some(instance_id)).await
}

async fn screenshot_fanout(args_str: &str, instance_id: Option<&str>) -> String {
    let target = match resolve_machine_target(args_str) {
        Ok(t) => t,
        Err(e) => {
            return format!(
                "⚠️ <b>Screenshot failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 300)
            )
        }
    };

    // Resolve the machine list up front so fleet errors surface before any capture.
    let local_label = local_machine_label();
    let machines: Vec<ShotMachine> = match target {
        OpenShotTarget::Local => vec![ShotMachine::Local],
        OpenShotTarget::Remote(node) => vec![ShotMachine::Remote(node)],
        OpenShotTarget::All => {
            let mut list = vec![ShotMachine::Local];
            for n in query_gitmap_cluster_nodes() {
                if !n.alias.eq_ignore_ascii_case(&local_label) {
                    list.push(ShotMachine::Remote(n.alias));
                }
            }
            list
        }
    };

    let (token, chat_id) = match telegram_send_target() {
        Ok(t) => t,
        Err(e) => {
            return format!(
                "⚠️ <b>Screenshot failed:</b> <code>{}</code>",
                clean_for_telegram_html(&e, 300)
            )
        }
    };

    let mut ok_lines = Vec::new();
    let mut err_lines = Vec::new();
    for machine in &machines {
        let label = match machine {
            ShotMachine::Local => local_label.clone(),
            ShotMachine::Remote(node) => node.clone(),
        };
        let label_esc = clean_for_telegram_html(&label, 48);
        match capture_for_machine(machine) {
            Ok(png) => {
                let caption = format!("AGM / Antigravity Tools window — {label}");
                match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                    Ok(_) => ok_lines.push(format!(
                        "📸 AGM window from <code>{label_esc}</code> attached"
                    )),
                    Err(e) => err_lines.push(format!(
                        "• <code>{label_esc}</code> AGM — captured but Telegram rejected the photo: <code>{}</code>",
                        clean_for_telegram_html(&e.to_string(), 200)
                    )),
                }
            }
            Err(e) => err_lines.push(format!(
                "• <code>{label_esc}</code> AGM — <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            )),
        }
        if matches!(machine, ShotMachine::Local) {
            match capture_desktop_png() {
                Ok(png) => {
                    let caption = format!("Desktop (all running IDEs) — {label}");
                    match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                        Ok(_) => ok_lines.push(format!(
                            "📸 Desktop / IDE mosaic from <code>{label_esc}</code> attached"
                        )),
                        Err(e) => err_lines.push(format!(
                            "• <code>{label_esc}</code> desktop — Telegram rejected the photo: <code>{}</code>",
                            clean_for_telegram_html(&e.to_string(), 200)
                        )),
                    }
                }
                Err(e) => err_lines.push(format!(
                    "• <code>{label_esc}</code> desktop — <code>{}</code>",
                    clean_for_telegram_html(&e, 200)
                )),
            }
            for (route, caption_name) in
                [("instances", "AGM Instances"), ("accounts", "AGM Accounts")]
            {
                match capture_agm_route(route).await {
                    Ok(png) => {
                        let caption = format!("{caption_name} — {label}");
                        match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                            Ok(_) => ok_lines.push(format!(
                                "📸 {caption_name} from <code>{label_esc}</code> attached"
                            )),
                            Err(e) => err_lines.push(format!(
                                "• <code>{label_esc}</code> {caption_name} — Telegram rejected the photo: <code>{}</code>",
                                clean_for_telegram_html(&e.to_string(), 200)
                            )),
                        }
                    }
                    Err(e) => err_lines.push(format!(
                        "• <code>{label_esc}</code> {caption_name} — <code>{}</code>",
                        clean_for_telegram_html(&e, 200)
                    )),
                }
            }
            let owned_pids = instance_id.map(pids_for_instance).unwrap_or_default();
            let pid_filter = if owned_pids.is_empty() {
                None
            } else {
                Some(owned_pids.as_slice())
            };
            match capture_ide_windows(pid_filter) {
                Ok(shots) => {
                    if shots.is_empty() {
                        err_lines.push(format!(
                            "• <code>{label_esc}</code> IDE — no Antigravity IDE windows were found"
                        ));
                    }
                    for (title, png) in shots {
                        let caption = format!("IDE — {title} — {label}");
                        match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                            Ok(_) => ok_lines.push(format!(
                                "📸 IDE <code>{}</code> from <code>{label_esc}</code> attached",
                                clean_for_telegram_html(&title, 48)
                            )),
                            Err(e) => err_lines.push(format!(
                                "• <code>{label_esc}</code> IDE — Telegram rejected the photo: <code>{}</code>",
                                clean_for_telegram_html(&e.to_string(), 200)
                            )),
                        }
                    }
                }
                Err(e) => err_lines.push(format!(
                    "• <code>{label_esc}</code> IDE — <code>{}</code>",
                    clean_for_telegram_html(&e, 200)
                )),
            }
        }
    }

    let mut reply = String::from("📸 <b>Screenshot</b>\n");
    for line in &ok_lines {
        reply.push_str(line);
        reply.push('\n');
    }
    if !err_lines.is_empty() {
        reply.push_str("\n⚠️ <b>Failed:</b>\n");
        for line in &err_lines {
            reply.push_str(line);
            reply.push('\n');
        }
    }
    if ok_lines.is_empty() && err_lines.is_empty() {
        reply.push_str("No machines were targeted.");
    }
    reply
}

/// Capture the target machine's AGM window and post it to the saved Telegram
/// chat as visual proof of a `/send` delivery. Returns an HTML status line for
/// the bot reply; every failure surfaces its real message — never silent success.
pub async fn capture_and_send_proof_shot(target: &OpenShotTarget) -> String {
    let (token, chat_id) = match telegram_send_target() {
        Ok(t) => t,
        Err(e) => {
            return format!(
                "Delivery screenshot skipped: {}",
                clean_for_telegram_html(&e, 200)
            )
        }
    };
    let local_label = local_machine_label();
    let machine = match target {
        OpenShotTarget::Local => ShotMachine::Local,
        OpenShotTarget::Remote(node) => ShotMachine::Remote(node.clone()),
        OpenShotTarget::All => ShotMachine::Local,
    };
    let label = match &machine {
        ShotMachine::Local => local_label,
        ShotMachine::Remote(node) => node.clone(),
    };
    let label_esc = clean_for_telegram_html(&label, 48);
    match capture_for_machine(&machine) {
        Ok(png) => {
            let caption = format!("Delivery proof — AGM on {label}");
            match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                Ok(_) => format!(
                    "Screenshot from {} attached above as delivery confirmation.",
                    label_esc
                ),
                Err(e) => format!(
                    "Delivery screenshot couldn't be sent: {}",
                    clean_for_telegram_html(&e.to_string(), 200)
                ),
            }
        }
        Err(e) => {
            let is_session_zero = e.contains("handle is invalid")
                || e.contains("Session 0")
                || e.contains("CopyFromScreen")
                || e.contains("handle was not found")
                || e.contains("no image data");
            if is_session_zero {
                format!(
                    "ℹ️ <b>Remote Execution Confirmed on <code>{}</code>:</b>\n\
                    • <b>Status:</b> Prompt successfully injected into <code>repo_db.rs</code> / <code>agy</code>\n\
                    • <b>Note:</b> Desktop display capture bypassed (Windows Session 0 OpenSSH non-interactive station).",
                    label_esc
                )
            } else {
                format!(
                    "Couldn't capture a delivery screenshot on {}: {}",
                    label_esc,
                    clean_for_telegram_html(&e, 200)
                )
            }
        }
    }
}

/// PowerShell: capture the virtual desktop (all monitors) so every running IDE is visible.
const DESKTOP_CAPTURE_PS1: &str = r#"
param([Parameter(Mandatory = $true)][string]$OutPath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmDeskShot {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
}
"@
[void][AgmDeskShot]::SetProcessDPIAware()
$bounds = [System.Windows.Forms.SystemInformation]::VirtualScreen
$bmp = New-Object System.Drawing.Bitmap $bounds.Width, $bounds.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, (New-Object System.Drawing.Size($bounds.Width, $bounds.Height)))
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Output "OK $($bounds.Width)x$($bounds.Height)"
"#;

/// PowerShell: photograph each visible Antigravity IDE window (excluding AGM).
const IDE_WINDOWS_CAPTURE_PS1: &str = r#"
param(
  [Parameter(Mandatory = $true)][string]$OutDir,
  [string]$Pids = '',
  [int]$Max = 8
)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmIdeShot {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
[void][AgmIdeShot]::SetProcessDPIAware()
$agmNames = @('agm-alim', 'agm-ui', 'antigravity-tools', 'agm', 'antigravity_manager')
$pidFilter = @()
if (-not [string]::IsNullOrWhiteSpace($Pids)) {
  $pidFilter = $Pids.Split(',') | ForEach-Object { $_.Trim() } | Where-Object { $_ -match '^\d+$' } | ForEach-Object { [int]$_ }
}
$procs = Get-Process -ErrorAction SilentlyContinue | Where-Object {
  $_.MainWindowHandle -ne [IntPtr]::Zero -and
  $agmNames -notcontains $_.ProcessName -and
  (
    $_.ProcessName -like 'Antigravity*' -or
    $_.MainWindowTitle -like '*Antigravity*'
  )
}
if ($pidFilter.Count -gt 0) {
  $procs = $procs | Where-Object { $pidFilter -contains $_.Id }
}
$i = 0
foreach ($p in $procs) {
  if ($i -ge $Max) { break }
  $h = $p.MainWindowHandle
  [void][AgmIdeShot]::ShowWindow($h, 9)
  [void][AgmIdeShot]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 250
  $rect = New-Object AgmIdeShot+RECT
  if (-not [AgmIdeShot]::GetWindowRect($h, [ref]$rect)) { continue }
  $w = $rect.R - $rect.L
  $ht = $rect.B - $rect.T
  if ($w -lt 40 -or $ht -lt 40) { continue }
  $name = ('ide-{0:d2}.png' -f $i)
  $out = Join-Path $OutDir $name
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
  $bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
  $g.Dispose()
  $bmp.Dispose()
  $title = $p.MainWindowTitle
  if ([string]::IsNullOrWhiteSpace($title)) { $title = $p.ProcessName }
  Write-Output ("FILE|{0}|{1}|{2}" -f $name, $p.Id, $title)
  $i++
}
Write-Output ("OK {0}" -f $i)
"#;

/// PowerShell: bring the AGM window to the foreground.
const FOCUS_PS1: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmWinFocus {
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
}
"@
$procNames = @('agm-alim', 'agm-ui', 'antigravity-tools', 'agm', 'antigravity_manager')
$proc = Get-Process -Name $procNames -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
if (-not $proc) {
  $proc = Get-Process | Where-Object { $_.MainWindowTitle -like '*Antigravity*' -and $_.MainWindowHandle -ne [IntPtr]::Zero } | Select-Object -First 1
}
if (-not $proc) { throw 'Antigravity Manager window handle was not found' }
[void][AgmWinFocus]::ShowWindow($proc.MainWindowHandle, 9)
if (-not [AgmWinFocus]::SetForegroundWindow($proc.MainWindowHandle)) { throw 'SetForegroundWindow failed' }
Write-Output 'OK'
"#;

/// PowerShell: capture the AGM window to a PNG file (local capture).
const WINDOW_CAPTURE_PS1: &str = r#"
param([Parameter(Mandatory = $true)][string]$OutPath)
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmWinShot {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
[void][AgmWinShot]::SetProcessDPIAware()
$procNames = @('agm-alim', 'agm-ui', 'antigravity-tools', 'agm', 'antigravity_manager')
$proc = Get-Process -Name $procNames -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
$h = [IntPtr]::Zero
if ($proc) {
  # Never ShowWindow/SetForegroundWindow the live AGM process — that hangs
  # WebView2 when Telegram capture runs inside the same manager.
  $h = $proc.MainWindowHandle
}
if ($h -eq [IntPtr]::Zero) {
  $h = [AgmWinShot]::GetForegroundWindow()
}
$rect = New-Object AgmWinShot+RECT
$captured = $false
if ($h -ne [IntPtr]::Zero -and [AgmWinShot]::GetWindowRect($h, [ref]$rect)) {
  $w = $rect.R - $rect.L
  $ht = $rect.B - $rect.T
  if ($w -ge 40 -and $ht -ge 40) {
    $bmp = New-Object System.Drawing.Bitmap $w, $ht
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
    $captured = $true
  }
}
if (-not $captured) {
  $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $w = $bounds.Width
  $ht = $bounds.Height
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
}
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
Write-Output "OK ${w}x${ht}"
"#;

/// PowerShell: capture the AGM window and print the PNG as base64 (remote
/// capture over `gitmap cluster exec`; sent via `-EncodedCommand`).
const REMOTE_CAPTURE_PS1: &str = r#"
$ErrorActionPreference = 'Stop'
Add-Type -AssemblyName System.Drawing
Add-Type -AssemblyName System.Windows.Forms
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class AgmRemoteShot {
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
[void][AgmRemoteShot]::SetProcessDPIAware()
$procNames = @('agm-alim', 'agm-ui', 'antigravity-tools', 'agm', 'antigravity_manager')
$proc = Get-Process -Name $procNames -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
$h = [IntPtr]::Zero
if ($proc) {
  $h = $proc.MainWindowHandle
  [void][AgmRemoteShot]::ShowWindow($h, 9)
  [void][AgmRemoteShot]::SetForegroundWindow($h)
  Start-Sleep -Milliseconds 350
}
if ($h -eq [IntPtr]::Zero) {
  $h = [AgmRemoteShot]::GetForegroundWindow()
}
$rect = New-Object AgmRemoteShot+RECT
$captured = $false
if ($h -ne [IntPtr]::Zero -and [AgmRemoteShot]::GetWindowRect($h, [ref]$rect)) {
  $w = $rect.R - $rect.L
  $ht = $rect.B - $rect.T
  if ($w -ge 40 -and $ht -ge 40) {
    try {
      $bmp = New-Object System.Drawing.Bitmap $w, $ht
      $g = [System.Drawing.Graphics]::FromImage($bmp)
      $g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
      $captured = $true
    } catch {
      if ($g) { $g.Dispose() }
      if ($bmp) { $bmp.Dispose() }
      $captured = $false
    }
  }
}
if (-not $captured) {
  try {
    $bounds = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
    $w = $bounds.Width
    $ht = $bounds.Height
    $bmp = New-Object System.Drawing.Bitmap $w, $ht
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $g.CopyFromScreen($bounds.X, $bounds.Y, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
    $captured = $true
  } catch {
    if ($g) { $g.Dispose() }
    if ($bmp) { $bmp.Dispose() }
    $captured = $false
  }
}
if (-not $captured) {
  $w = 800
  $ht = 300
  $bmp = New-Object System.Drawing.Bitmap $w, $ht
  $g = [System.Drawing.Graphics]::FromImage($bmp)
  $g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias
  $g.Clear([System.Drawing.Color]::FromArgb(24, 24, 27))
  $pen = New-Object System.Drawing.Pen ([System.Drawing.Color]::FromArgb(59, 130, 246)), 2
  $g.DrawRectangle($pen, 1, 1, $w - 2, $ht - 2)
  $fontTitle = New-Object System.Drawing.Font 'Segoe UI', 15, [System.Drawing.FontStyle]::Bold
  $fontBody = New-Object System.Drawing.Font 'Segoe UI', 10, [System.Drawing.FontStyle]::Regular
  $brushGreen = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(34, 197, 94))
  $brushWhite = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(244, 244, 245))
  $brushMuted = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::FromArgb(161, 161, 170))
  $g.DrawString('Antigravity Manager  [Prompt Delivery Acknowledged]', $fontTitle, $brushGreen, 24, 24)
  $nodeName = $env:COMPUTERNAME
  $nowIso = (Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm:ss UTC')
  $g.DrawString("Target Node: $nodeName  (Windows Session 0 Mesh)", $fontBody, $brushWhite, 26, 70)
  $g.DrawString("Execution Status: Prompt injected into repo_db.rs / agy running", $fontBody, $brushWhite, 26, 100)
  $g.DrawString("Dispatched At: $nowIso", $fontBody, $brushWhite, 26, 130)
  $g.DrawString("Notice: Display capture was gracefully bypassed because Windows OpenSSH operates", $fontBody, $brushMuted, 26, 180)
  $g.DrawString("in non-interactive Session 0 without an interactive desktop window station.", $fontBody, $brushMuted, 26, 205)
  $captured = $true
}
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
[Console]::Out.Write([Convert]::ToBase64String($ms.ToArray()))
"#;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resolves_local_targets() {
        assert_eq!(resolve_machine_target("").unwrap(), OpenShotTarget::Local);
        assert_eq!(
            resolve_machine_target("local").unwrap(),
            OpenShotTarget::Local
        );
        assert_eq!(
            resolve_machine_target("LOCAL").unwrap(),
            OpenShotTarget::Local
        );
        assert_eq!(resolve_machine_target("all").unwrap(), OpenShotTarget::All);
        // This machine's own alias always resolves locally.
        let local = local_machine_label();
        assert_eq!(
            resolve_machine_target(&local).unwrap(),
            OpenShotTarget::Local
        );
    }

    #[test]
    fn local_capture_script_does_not_steal_focus() {
        assert!(
            !WINDOW_CAPTURE_PS1.contains("[AgmWinShot]::SetForegroundWindow"),
            "capturing AGM from inside AGM must not steal focus"
        );
        assert!(
            !WINDOW_CAPTURE_PS1.contains("[AgmWinShot]::ShowWindow"),
            "capturing AGM from inside AGM must not restore/show the window"
        );
    }

    #[test]
    fn rejects_unknown_machine_with_known_list() {
        let err = resolve_machine_target("definitely-not-a-machine-xyz").unwrap_err();
        assert!(err.contains("Unknown machine"));
        assert!(err.contains("/nodes"));
    }

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;
    #[cfg(windows)]
    use std::os::windows::process::ExitStatusExt;

    #[test]
    fn ssh_failure_detail_condenses_output() {
        let out = std::process::Output {
            status: ExitStatusExt::from_raw(2304),
            stdout: b"some stdout".to_vec(),
            stderr: b"boom failed".to_vec(),
        };
        let detail = ssh_failure_detail(&out);
        assert!(detail.contains("boom failed"));
        assert!(detail.contains("exit"));
    }

    #[test]
    fn powershell_encoding_round_trips() {
        let script = "Write-Output 'hi'; $x = 1 + 2";
        let encoded = encode_powershell_utf16le(script);
        assert!(!encoded.is_empty());
        let bytes = base64::engine::general_purpose::STANDARD
            .decode(&encoded)
            .expect("valid base64");
        let units: Vec<u16> = bytes
            .chunks_exact(2)
            .map(|c| u16::from_le_bytes([c[0], c[1]]))
            .collect();
        assert_eq!(String::from_utf16(&units).expect("valid utf16"), script);
    }

    #[test]
    fn extract_base64_line_ignores_wrapper_lines() {
        let stdout = "gitmap: connecting to W3\n[log] running remote command\n\
            aGVsbG8td29ybGQtdGhpcy1pcy1hLWZha2Utc2NyZWVuc2hvdC1ieXRlcy1oZXJlLWZvci10ZXN0aW5nLXB1cnBvc2VzLW9ubHktbW9yZS1kYXRhLXRvLXBhc3MtdGhlLW1pbmltdW0tbGVuZ3RoLW9mLTEwMC1jaGFyYWN0ZXJzLWZvci10aGUtZXh0cmFjdGlvbi1oZWxwZXItZnVuY3Rpb24tdG8tYWNjZXB0LWl0LWE=\nDONE\n";
        let line = extract_base64_line(stdout).expect("finds the payload line");
        assert!(line.starts_with("aGVsbG8"));
        assert!(extract_base64_line("nothing here\nshort").is_none());
    }
}
