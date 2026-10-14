//! Machine-targeted `/open` and `/screenshot` Telegram commands.
//!
//! - `/open [machine]` — open the AGM UI on the target machine: launch it
//!   when it is not running, otherwise bring its window to the foreground.
//! - `/screenshot [machine]` — capture the target machine's screen (AGM is
//!   opened first when needed) and send the image(s) back to the Telegram
//!   chat. With no argument, every known machine is captured.
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
use std::io::Write;
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

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

/// This machine's fleet display name: Supabase node alias, else the OS machine name.
pub fn local_machine_label() -> String {
    match supabase_sync::load_config() {
        Ok(cfg) if !cfg.node_alias.trim().is_empty() => cfg.node_alias.trim().to_string(),
        Ok(_) => email_watcher::detect_machine_name(),
        Err(e) => {
            // Justification: the alias is display-only; an unreadable config falls back to the OS machine name.
            crate::error::record_ignored(Err::<(), _>(e), "load supabase config for node alias");
            email_watcher::detect_machine_name()
        }
    }
}

/// Resolve the `[machine]` argument of `/open` / `/screenshot` against the
/// existing fleet model (local alias + GitMap cluster node aliases).
pub fn resolve_machine_target(arg: &str) -> Result<OpenShotTarget, String> {
    let token = arg.trim();
    let local = local_machine_label();

    if token.is_empty() || token.eq_ignore_ascii_case("local") || token.eq_ignore_ascii_case(&local)
    {
        return Ok(OpenShotTarget::Local);
    }
    if token.eq_ignore_ascii_case("all") {
        return Ok(OpenShotTarget::All);
    }
    let nodes = query_gitmap_cluster_nodes();
    if let Some(node) = nodes.iter().find(|n| n.alias.eq_ignore_ascii_case(token)) {
        if node.alias.eq_ignore_ascii_case(&local) {
            return Ok(OpenShotTarget::Local);
        }
        return Ok(OpenShotTarget::Remote(node.alias.clone()));
    }
    let mut known = vec![local];
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
        proc.name()
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains("agm-alim")
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
    let output = Command::new("powershell.exe")
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
    let ssh_outcome: Result<(), String> = match Command::new("gitmap")
        .args(["cluster", "exec", node, remote_cmd])
        .output()
    {
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
    let output = Command::new("powershell.exe")
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
    if bytes.len() < 32 {
        return Err("screenshot file was empty".to_string());
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

/// Pick the base64 payload line out of `gitmap cluster exec` stdout, ignoring
/// any wrapper/log lines the transport may add.
fn extract_base64_line(stdout: &str) -> Option<String> {
    stdout
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.len() >= 100
                && line
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=')
        })
        .last()
        .map(str::to_string)
}

/// Capture a remote fleet node's screen: ensure AGM is open there, then run a
/// PowerShell capture that prints the PNG as base64 over `gitmap cluster exec`.
fn capture_remote_png(node: &str) -> Result<Vec<u8>, String> {
    let open_out = Command::new("gitmap")
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
    let out = Command::new("gitmap")
        .args(["cluster", "exec", node, &remote_cmd])
        .output()
        .map_err(|e| format!("could not run remote capture on {node}: {e}"))?;
    if !out.status.success() {
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
    if bytes.len() < 32 {
        return Err(format!("remote screenshot from {node} was empty"));
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

/// Handle `/screenshot [machine]` — capture the target machine's screen and
/// send the image(s) back to the Telegram chat.
///
/// No argument captures every known machine; AGM is opened first when needed.
/// Photos are sent to the chat directly; the returned text is the bot reply.
/// Failures surface their real messages — never silent success.
pub async fn execute_screenshot_command(args_str: &str) -> String {
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
                let caption = format!("Screenshot from {label}");
                match send_telegram_photo(&token, chat_id, &png, &caption, None).await {
                    Ok(_) => ok_lines.push(format!(
                        "📸 Screenshot from <code>{label_esc}</code> attached"
                    )),
                    Err(e) => err_lines.push(format!(
                        "• <code>{label_esc}</code> — captured but Telegram rejected the photo: <code>{}</code>",
                        clean_for_telegram_html(&e.to_string(), 200)
                    )),
                }
            }
            Err(e) => err_lines.push(format!(
                "• <code>{label_esc}</code> — <code>{}</code>",
                clean_for_telegram_html(&e, 200)
            )),
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
                "📸 <b>Proof screenshot skipped:</b> <code>{}</code>",
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
                    "📸 <b>Proof:</b> screenshot from <code>{label_esc}</code> attached above."
                ),
                Err(e) => format!(
                    "📸 <b>Proof screenshot failed:</b> captured but Telegram rejected the photo: <code>{}</code>",
                    clean_for_telegram_html(&e.to_string(), 200)
                ),
            }
        }
        Err(e) => format!(
            "📸 <b>Proof screenshot failed on <code>{label_esc}</code>:</b> <code>{}</code>",
            clean_for_telegram_html(&e, 200)
        ),
    }
}

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
$proc = Get-Process -Name 'agm-alim' -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Select-Object -First 1
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
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h, int n);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
$proc = Get-Process -Name 'agm-alim' -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
if (-not $proc) { throw 'Antigravity Manager window handle was not found' }
$h = $proc.MainWindowHandle
[void][AgmWinShot]::ShowWindow($h, 9)
[void][AgmWinShot]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 350
$rect = New-Object AgmWinShot+RECT
if (-not [AgmWinShot]::GetWindowRect($h, [ref]$rect)) { throw 'GetWindowRect failed' }
$w = $rect.R - $rect.L
$ht = $rect.B - $rect.T
if ($w -lt 40 -or $ht -lt 40) { throw "window is too small (${w}x${ht})" }
$bmp = New-Object System.Drawing.Bitmap $w, $ht
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
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
  public struct RECT { public int L; public int T; public int R; public int B; }
}
"@
$proc = Get-Process -Name 'agm-alim' -ErrorAction SilentlyContinue |
  Where-Object { $_.MainWindowHandle -ne [IntPtr]::Zero } |
  Sort-Object -Property WorkingSet64 -Descending |
  Select-Object -First 1
if (-not $proc) { throw 'Antigravity Manager window handle was not found' }
$h = $proc.MainWindowHandle
[void][AgmRemoteShot]::ShowWindow($h, 9)
[void][AgmRemoteShot]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 350
$rect = New-Object AgmRemoteShot+RECT
if (-not [AgmRemoteShot]::GetWindowRect($h, [ref]$rect)) { throw 'GetWindowRect failed' }
$w = $rect.R - $rect.L
$ht = $rect.B - $rect.T
if ($w -lt 40 -or $ht -lt 40) { throw "window is too small (${w}x${ht})" }
$bmp = New-Object System.Drawing.Bitmap $w, $ht
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($rect.L, $rect.T, 0, 0, (New-Object System.Drawing.Size($w, $ht)))
$ms = New-Object System.IO.MemoryStream
$bmp.Save($ms, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose()
$bmp.Dispose()
[Convert]::ToBase64String($ms.ToArray())
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
