//! Telegram commands that open the Antigravity Manager window, photograph it,
//! and remember where the next plain-text prompt should go.

use serde_json::{json, Value};
use std::io::Write;
use std::process::Command;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

use super::injection::execute_prompt_injection;

const TARGET_TTL_SECS: u64 = 600;

struct PromptAim {
    instance: String,
    conversation: String,
    set_at: u64,
}

static PROMPT_AIM: Mutex<Option<PromptAim>> = Mutex::new(None);

#[derive(Debug, Clone, PartialEq, Eq)]
enum UiVerb {
    Open,
    Shot,
    ShowAim,
    ClearAim,
    SetAim {
        instance: String,
        conversation: String,
    },
    AimedPrompt {
        text: String,
    },
}

pub struct LocalUiAction {
    pub reply: String,
    pub markup: Option<Value>,
    pub png: Option<Vec<u8>>,
    pub caption: String,
}

/// Copy-paste affordances for the essentials: every button carries the command
/// template as native `copy_text`, so the user taps, copies, edits, and sends.
pub fn help_copy_markup() -> Value {
    json!({
        "inline_keyboard": [
            [
                { "text": "📋 /send", "copy_text": { "text": "/send" } },
                { "text": "📋 /prompt", "copy_text": { "text": "/prompt C001 " } },
                { "text": "📋 /open", "copy_text": { "text": "/open " } }
            ],
            [
                { "text": "📋 /screenshot", "copy_text": { "text": "/screenshot " } },
                { "text": "📋 /shot", "copy_text": { "text": "/shot" } },
                { "text": "📋 /ui", "copy_text": { "text": "/ui" } }
            ],
            [
                { "text": "📋 /target", "copy_text": { "text": "/target default C001" } },
                { "text": "📋 /tree", "copy_text": { "text": "/tree" } },
                { "text": "📋 /fleet", "copy_text": { "text": "/fleet" } }
            ],
            [
                { "text": "📋 /status", "copy_text": { "text": "/status" } },
                { "text": "📋 /active", "copy_text": { "text": "/active" } },
                { "text": "📋 /nodes", "copy_text": { "text": "/nodes" } }
            ],
            [
                { "text": "📋 /backup", "copy_text": { "text": "/backup" } },
                { "text": "📋 /update", "copy_text": { "text": "/update" } },
                { "text": "📋 /help", "copy_text": { "text": "/help" } }
            ]
        ]
    })
}

pub fn window_command_section() -> &'static str {
    "\n\
    🎨 <b>Window, screenshot, and where the next prompt goes</b>\n\
    <blockquote>🔵 <b>This machine.</b> <code>/ui</code> opens Antigravity Manager when it is closed. <code>/shot</code> opens it only if needed, then sends a picture of the current manager window back to this chat. Each machine that runs the manager answers for its own screen.</blockquote>\n\
    <pre>/ui\n/shot\n/target default C001</pre>\n\
    <blockquote>🟢 Send <code>/target &lt;instance&gt; &lt;C001&gt;</code> first — the next plain message is the prompt and goes to that conversation. <code>/target clear</code> forgets the aim. For a guided machine → instance → conversation picker with screenshot proof of delivery, use <code>/send</code>.</blockquote>\n\
    <blockquote>🟡 <code>/open &lt;machine&gt;</code> opens AGM on a fleet machine; <code>/screenshot &lt;machine&gt;</code> photographs it. Bare <code>/open</code> / <code>/screenshot</code> stay local.</blockquote>\n\
    <blockquote>🟣 Voice notes stay audio. Telegram does not turn them into text. Type the prompt after <code>/target</code> or <code>/send</code>.</blockquote>\n"
}

/// Handle `/ui`, `/shot`, `/target`, and a follow-up prompt. Other text returns None.
pub async fn prepare_local_ui_action(text: &str) -> Option<LocalUiAction> {
    let verb = classify_ui_verb(text)?;
    Some(run_ui_verb(verb).await)
}

fn classify_ui_verb(text: &str) -> Option<UiVerb> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return None;
    }
    if !trimmed.starts_with('/') {
        return aimed_prompt_verb(trimmed);
    }
    let (cmd_raw, rest) = match trimmed.split_once(char::is_whitespace) {
        Some((cmd, rest)) => (cmd, rest.trim()),
        None => (trimmed, ""),
    };
    let cmd = cmd_raw
        .trim_start_matches('/')
        .split('@')
        .next()
        .unwrap_or("")
        .to_ascii_lowercase();
    match cmd.as_str() {
        "ui" => Some(UiVerb::Open),
        "open" | "open-ui" | "launch-ui" => open_target_verb(rest),
        "shot" => Some(UiVerb::Shot),
        "screenshot" | "screen" => shot_target_verb(rest),
        "where" | "aim" => Some(parse_target_rest(rest)),
        "target" => Some(parse_target_rest(rest)),
        "agm" => match rest.split_once(char::is_whitespace) {
            Some((head, tail)) => match head.to_ascii_lowercase().as_str() {
                "ui" | "open" | "open-ui" => Some(UiVerb::Open),
                "shot" | "screenshot" | "screen" => Some(UiVerb::Shot),
                "target" | "where" | "aim" => Some(parse_target_rest(tail.trim())),
                _ => None,
            },
            None => match rest.to_ascii_lowercase().as_str() {
                "ui" | "open" | "open-ui" => Some(UiVerb::Open),
                "shot" | "screenshot" | "screen" => Some(UiVerb::Shot),
                "target" | "where" | "aim" => Some(UiVerb::ShowAim),
                _ => None,
            },
        },
        _ => None,
    }
}

/// Machine-targeted `/open` / `/screenshot` belong to the command router's
/// `open_shot` handlers: when the argument names a NON-local fleet machine,
/// decline here so `processing.rs` routes it to the machine-targeted handler.
/// Bare commands, local aliases, and unknown tokens keep the existing local
/// behavior exactly as before.
fn open_target_verb(rest: &str) -> Option<UiVerb> {
    match super::open_shot::resolve_machine_target(rest) {
        Ok(super::open_shot::OpenShotTarget::Remote(_)) => None,
        _ => Some(UiVerb::Open),
    }
}

fn shot_target_verb(rest: &str) -> Option<UiVerb> {
    match super::open_shot::resolve_machine_target(rest) {
        Ok(super::open_shot::OpenShotTarget::Remote(_)) => None,
        _ => Some(UiVerb::Shot),
    }
}

fn parse_target_rest(rest: &str) -> UiVerb {
    let rest = rest.trim();
    if rest.is_empty() || rest.eq_ignore_ascii_case("show") || rest.eq_ignore_ascii_case("ls") {
        return UiVerb::ShowAim;
    }
    if rest.eq_ignore_ascii_case("clear") || rest.eq_ignore_ascii_case("reset") {
        return UiVerb::ClearAim;
    }
    let mut parts = rest.split_whitespace();
    match (parts.next(), parts.next()) {
        (Some(instance), Some(conversation)) => UiVerb::SetAim {
            instance: instance.to_string(),
            conversation: conversation.to_string(),
        },
        _ => UiVerb::ShowAim,
    }
}

fn aimed_prompt_verb(text: &str) -> Option<UiVerb> {
    let aim = PROMPT_AIM
        .lock()
        .ok()?
        .as_ref()
        .map(|aim| (aim.instance.clone(), aim.conversation.clone(), aim.set_at))?;
    if now_secs().saturating_sub(aim.2) > TARGET_TTL_SECS {
        if let Ok(mut guard) = PROMPT_AIM.lock() {
            *guard = None;
        }
        return None;
    }
    Some(UiVerb::AimedPrompt {
        text: text.to_string(),
    })
}

async fn run_ui_verb(verb: UiVerb) -> LocalUiAction {
    match verb {
        UiVerb::Open => {
            let note = ensure_manager_window();
            LocalUiAction {
                reply: format!("🖥️ <b>Antigravity Manager</b>\n{note}"),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::Shot => match capture_manager_png() {
            Ok(png) => LocalUiAction {
                reply: format!(
                    "📸 <b>Screenshot sent</b> from <code>{}</code>.\nThe picture is the Antigravity Manager window on this machine.",
                    clean_machine()
                ),
                markup: Some(help_copy_markup()),
                caption: format!("AGM window on {}", clean_machine()),
                png: Some(png),
            },
            Err(err) => LocalUiAction {
                reply: format!("❌ <b>Screenshot failed:</b> <code>{}</code>", escape_html(&err)),
                markup: None,
                png: None,
                caption: String::new(),
            },
        },
        UiVerb::ShowAim => LocalUiAction {
            reply: aim_status_html(),
            markup: Some(help_copy_markup()),
            png: None,
            caption: String::new(),
        },
        UiVerb::ClearAim => {
            if let Ok(mut guard) = PROMPT_AIM.lock() {
                *guard = None;
            }
            LocalUiAction {
                reply: "🧹 <b>Aim cleared.</b> The next plain message is no longer sent as a prompt."
                    .to_string(),
                markup: None,
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::SetAim {
            instance,
            conversation,
        } => {
            if let Ok(mut guard) = PROMPT_AIM.lock() {
                *guard = Some(PromptAim {
                    instance: instance.clone(),
                    conversation: conversation.clone(),
                    set_at: now_secs(),
                });
            }
            LocalUiAction {
                reply: format!(
                    "🎯 <b>Next prompt goes here</b>\n\
                    • <b>Instance:</b> <code>{}</code>\n\
                    • <b>Conversation:</b> <code>{}</code>\n\
                    Send the prompt as the next plain message. It stays aimed for 10 minutes. <code>/target clear</code> forgets it.",
                    escape_html(&instance),
                    escape_html(&conversation)
                ),
                markup: Some(help_copy_markup()),
                png: None,
                caption: String::new(),
            }
        }
        UiVerb::AimedPrompt { text } => {
            let (instance, conversation) = PROMPT_AIM
                .lock()
                .ok()
                .and_then(|g| {
                    g.as_ref()
                        .map(|aim| (aim.instance.clone(), aim.conversation.clone()))
                })
                .unwrap_or_else(|| ("default".to_string(), "C001".to_string()));
            let injected = execute_prompt_injection(&format!(
                "{conversation} --instance {instance} {text}"
            ))
            .await;
            let (png, note) = match capture_manager_png() {
                Ok(bytes) => (
                    Some(bytes),
                    "📸 The manager window on this machine is attached. This does not click a conversation inside the IDE.".to_string(),
                ),
                Err(err) => (
                    None,
                    format!(
                        "Screenshot was not attached: <code>{}</code>",
                        escape_html(&err)
                    ),
                ),
            };
            LocalUiAction {
                reply: format!(
                    "🎯 <b>Prompt sent to</b> <code>{}</code> on <code>{}</code>\n{}\n\n{}",
                    escape_html(&conversation),
                    escape_html(&instance),
                    note,
                    injected
                ),
                markup: None,
                caption: format!(
                    "AGM after prompt to {} on {}",
                    conversation, instance
                ),
                png,
            }
        }
    }
}

fn aim_status_html() -> String {
    match PROMPT_AIM.lock() {
        Ok(guard) => match guard.as_ref() {
            Some(aim) => format!(
                "🎯 <b>Current aim</b>\n• <b>Instance:</b> <code>{}</code>\n• <b>Conversation:</b> <code>{}</code>\nThe next plain message is the prompt.",
                escape_html(&aim.instance),
                escape_html(&aim.conversation)
            ),
            None => "🎯 <b>No aim set.</b> Send <code>/target default C001</code> first, then the prompt text.".to_string(),
        },
        Err(_) => "🎯 <b>Aim lock failed.</b> Send <code>/target default C001</code> again.".to_string(),
    }
}

fn ensure_manager_window() -> String {
    if manager_process_running() {
        return format!(
            "Already open on <code>{}</code>. Use <code>/shot</code> to photograph this screen.",
            clean_machine()
        );
    }
    crate::modules::delegate_updater::open_ui(&[]);
    format!(
        "Launch requested on <code>{}</code>. If the window is still starting, send <code>/shot</code> again in a few seconds.",
        clean_machine()
    )
}

pub fn capture_manager_png_cli() -> Result<usize, String> {
    capture_manager_png().map(|bytes| bytes.len())
}

/// Capture the manager window and upload it to the saved Telegram chat.
pub async fn send_window_shot_to_saved_chat() -> Result<usize, String> {
    let png = capture_manager_png()?;
    let len = png.len();
    let cfg = super::config::load_config().map_err(|e| e.to_string())?;
    let chat_id = cfg
        .allowed_chat_id
        .ok_or_else(|| "telegram chat id is not configured".to_string())?;
    if cfg.bot_token.trim().is_empty() {
        return Err("telegram bot token is not configured".to_string());
    }
    super::media::send_telegram_photo(
        &cfg.bot_token,
        chat_id,
        &png,
        "AGM window on this machine",
        None,
    )
    .await
    .map_err(|e| e.to_string())?;
    Ok(len)
}

fn capture_manager_png() -> Result<Vec<u8>, String> {
    let opened = if manager_process_running() {
        false
    } else {
        crate::modules::delegate_updater::open_ui(&[]);
        true
    };
    if opened {
        std::thread::sleep(std::time::Duration::from_secs(3));
    }
    let mut last_err = String::from("manager window was not found");
    for _ in 0..4 {
        match capture_once() {
            Ok(bytes) => return Ok(bytes),
            Err(err) => {
                last_err = err;
                std::thread::sleep(std::time::Duration::from_millis(700));
            }
        }
    }
    Err(last_err)
}

fn capture_once() -> Result<Vec<u8>, String> {
    #[cfg(not(target_os = "windows"))]
    {
        return Err("window screenshot is implemented for Windows".to_string());
    }
    #[cfg(target_os = "windows")]
    {
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
            file.write_all(CAPTURE_PS1.as_bytes())
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
}

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

fn clean_machine() -> String {
    escape_html(&crate::modules::email_watcher::detect_machine_name())
}

fn escape_html(text: &str) -> String {
    text.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
}

fn now_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

const CAPTURE_PS1: &str = r#"
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

#[cfg(test)]
mod tests {
    use super::classify_ui_verb;
    use super::UiVerb;

    #[test]
    fn classifies_window_commands_and_aim() {
        assert_eq!(classify_ui_verb("/ui"), Some(UiVerb::Open));
        assert_eq!(classify_ui_verb("/open"), Some(UiVerb::Open));
        assert_eq!(classify_ui_verb("/screenshot"), Some(UiVerb::Shot));
        assert_eq!(classify_ui_verb("/shot@MyBot"), Some(UiVerb::Shot));
        assert_eq!(classify_ui_verb("/agm ui"), Some(UiVerb::Open));
        // Unknown machine tokens keep the existing local behavior untouched.
        assert_eq!(
            classify_ui_verb("/open definitely-not-a-machine-xyz"),
            Some(UiVerb::Open)
        );
        assert_eq!(
            classify_ui_verb("/screenshot definitely-not-a-machine-xyz"),
            Some(UiVerb::Shot)
        );
        assert_eq!(
            classify_ui_verb("/target default C001"),
            Some(UiVerb::SetAim {
                instance: "default".to_string(),
                conversation: "C001".to_string(),
            })
        );
        assert_eq!(classify_ui_verb("/target clear"), Some(UiVerb::ClearAim));
        assert_eq!(classify_ui_verb("/help"), None);
        assert_eq!(classify_ui_verb("/send"), None);
        assert_eq!(classify_ui_verb("just a sentence"), None);
    }
}
