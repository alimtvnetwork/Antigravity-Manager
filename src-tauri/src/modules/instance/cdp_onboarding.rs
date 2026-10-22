//! CDP-based auto-dismissal of the Antigravity "Security Notice & Data Use" /
//! "Terms of Service & Data Use" post-onboarding dialog.
//!
//! When switching accounts (from CLI or UI) or launching an instance, Antigravity
//! may present a modal dialog asking whether to share Interactions data with Google
//! alongside an "Accept and continue" button. This module connects to the instance's
//! local DevTools endpoint (`DevToolsActivePort`), ensures the Interactions/telemetry
//! data-sharing checkbox is unchecked (`checked = false`), and clicks the
//! "Accept and continue" button.

use std::fs;
use std::io::{Read, Write};
use std::net::{IpAddr, Ipv4Addr, SocketAddr, TcpStream};
use std::path::{Path, PathBuf};
use std::time::{Duration, Instant};

const DISMISS_SECURITY_NOTICE_JS: &str = r#"(async () => {
  if (window.__agmSecurityNoticeDismissed) {
    return { status: 'dismissed', alreadyDone: true };
  }
  if (window.__agmDismissingSecurityNotice) {
    return { status: 'in_progress' };
  }

  const findSecurityDialog = () => {
    const dialogs = Array.from(document.querySelectorAll('[role="dialog"], .modal, [data-state="open"]'));
    for (const d of dialogs) {
      const text = d.innerText || '';
      if (
        text.includes('Security Notice & Data Use') ||
        text.includes('Terms of Service & Data Use') ||
        (text.includes('Accept and continue') &&
          (text.includes('Interactions data') ||
            text.includes('security limitations') ||
            text.includes('security risks')))
      ) {
        return d;
      }
    }
    const buttons = Array.from(document.querySelectorAll('button'));
    for (const btn of buttons) {
      const label = (btn.innerText || btn.textContent || '').trim();
      if (label === 'Accept and continue') {
        return btn.closest('[role="dialog"]') || btn.parentElement?.parentElement || document.body;
      }
    }
    return null;
  };

  const dialog = findSecurityDialog();
  if (!dialog) {
    return { status: 'no_dialog' };
  }

  window.__agmDismissingSecurityNotice = true;
  try {
    // Allow uCb's getUserAccountSettings() to finish populating checkbox state
    await new Promise((r) => setTimeout(r, 450));

    const uncheckAll = (root) => {
      let uncheckedCount = 0;
      const checkboxes = Array.from(root.querySelectorAll('input[type="checkbox"]'));
      for (const cb of checkboxes) {
        if (cb.checked && !cb.disabled) {
          cb.click();
          uncheckedCount++;
        }
      }
      const roleCheckboxes = Array.from(root.querySelectorAll('[role="checkbox"]'));
      for (const rc of roleCheckboxes) {
        if (rc.getAttribute('aria-checked') === 'true') {
          rc.click();
          uncheckedCount++;
        }
      }
      return { uncheckedCount, totalCheckboxes: checkboxes.length + roleCheckboxes.length };
    };

    const activeDialog = findSecurityDialog() || dialog;
    const firstPass = uncheckAll(activeDialog);
    await new Promise((r) => setTimeout(r, 180));
    const secondPass = uncheckAll(findSecurityDialog() || activeDialog);

    const verifyRoot = findSecurityDialog() || activeDialog;
    for (const cb of Array.from(verifyRoot.querySelectorAll('input[type="checkbox"]'))) {
      if (cb.checked && !cb.disabled) {
        cb.click();
      }
    }
    await new Promise((r) => setTimeout(r, 120));

    const currentDialog = findSecurityDialog() || verifyRoot;
    let acceptBtn = Array.from(currentDialog.querySelectorAll('button')).find(
      (b) => (b.innerText || b.textContent || '').trim() === 'Accept and continue'
    );
    if (!acceptBtn) {
      acceptBtn = Array.from(document.querySelectorAll('button')).find(
        (b) => (b.innerText || b.textContent || '').trim() === 'Accept and continue'
      );
    }
    if (!acceptBtn) {
      return { status: 'no_button', firstPass, secondPass };
    }

    acceptBtn.click();
    await new Promise((r) => setTimeout(r, 200));
    const stillOpen = !!findSecurityDialog();
    if (!stillOpen) {
      window.__agmSecurityNoticeDismissed = true;
    }
    return {
      status: stillOpen ? 'still_open' : 'dismissed',
      unchecked: firstPass.uncheckedCount + secondPass.uncheckedCount,
      totalCheckboxes: firstPass.totalCheckboxes,
    };
  } finally {
    window.__agmDismissingSecurityNotice = false;
  }
})()"#;

pub(crate) fn candidate_devtools_port_files(data_dir: &Path) -> Vec<PathBuf> {
    let mut files = vec![
        data_dir.join("DevToolsActivePort"),
        data_dir
            .join("AppData")
            .join("Roaming")
            .join("Antigravity")
            .join("DevToolsActivePort"),
    ];
    if let Some(parent) = data_dir.parent() {
        files.push(
            parent
                .join("home")
                .join("AppData")
                .join("Roaming")
                .join("Antigravity")
                .join("DevToolsActivePort"),
        );
    }
    files
}

fn read_active_ports(data_dir: &Path) -> Vec<u16> {
    let mut ports = Vec::new();
    for file in candidate_devtools_port_files(data_dir) {
        if let Ok(content) = fs::read_to_string(&file) {
            if let Some(first_line) = content.lines().next() {
                if let Ok(port) = first_line.trim().parse::<u16>() {
                    if port > 0 && !ports.contains(&port) {
                        ports.push(port);
                    }
                }
            }
        }
    }
    ports
}

fn fetch_page_ws_urls(port: u16) -> Vec<String> {
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let Ok(mut stream) = TcpStream::connect_timeout(&addr, Duration::from_millis(600)) else {
        return Vec::new();
    };
    // Justification: best-effort socket timeout configuration on loopback CDP discovery
    crate::error::record_ignored(
        stream.set_read_timeout(Some(Duration::from_millis(800))),
        "set CDP HTTP read timeout",
    );
    // Justification: best-effort socket timeout configuration on loopback CDP discovery
    crate::error::record_ignored(
        stream.set_write_timeout(Some(Duration::from_millis(800))),
        "set CDP HTTP write timeout",
    );

    let req = format!(
        "GET /json HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nConnection: close\r\n\r\n",
        port
    );
    if stream.write_all(req.as_bytes()).is_err() {
        return Vec::new();
    }

    let mut resp = Vec::new();
    let mut buf = [0u8; 4096];
    loop {
        match stream.read(&mut buf) {
            Ok(0) => break,
            Ok(n) => {
                resp.extend_from_slice(&buf[..n]);
                if let Some((_, body)) = std::str::from_utf8(&resp)
                    .ok()
                    .and_then(|s| s.split_once("\r\n\r\n"))
                {
                    if serde_json::from_str::<serde_json::Value>(body.trim()).is_ok() {
                        break;
                    }
                }
            }
            Err(e)
                if e.kind() == std::io::ErrorKind::TimedOut
                    || e.kind() == std::io::ErrorKind::WouldBlock
                    || e.raw_os_error() == Some(10060)
                    || e.to_string().contains("10060")
                    || e.to_string().contains("timed out") =>
            {
                break;
            }
            Err(e) => {
                // Justification: partial read or failure during local CDP discovery is handled by fallback loop
                crate::error::record_ignored(
                    Err::<(), std::io::Error>(e),
                    "read CDP /json response",
                );
                break;
            }
        }
    }
    let text = String::from_utf8_lossy(&resp);
    let Some((_, body)) = text.split_once("\r\n\r\n") else {
        return Vec::new();
    };

    let Ok(targets) = serde_json::from_str::<serde_json::Value>(body.trim()) else {
        return Vec::new();
    };
    let Some(arr) = targets.as_array() else {
        return Vec::new();
    };

    let mut urls = Vec::new();
    for item in arr {
        let target_type = item.get("type").and_then(|v| v.as_str()).unwrap_or("");
        if target_type != "page" && target_type != "webview" {
            continue;
        }
        if let Some(ws_url) = item.get("webSocketDebuggerUrl").and_then(|v| v.as_str()) {
            if ws_url.starts_with("ws://127.0.0.1:") || ws_url.starts_with("ws://localhost:") {
                urls.push(ws_url.to_string());
            }
        }
    }
    urls
}

fn parse_ws_url(ws_url: &str) -> Option<(u16, String)> {
    let rest = ws_url
        .strip_prefix("ws://127.0.0.1:")
        .or_else(|| ws_url.strip_prefix("ws://localhost:"))?;
    let (port_str, path) = match rest.find('/') {
        Some(idx) => (&rest[..idx], &rest[idx..]),
        None => (rest, "/"),
    };
    let port = port_str.parse::<u16>().ok()?;
    Some((port, path.to_string()))
}

fn write_masked_ws_text_frame(stream: &mut TcpStream, payload: &[u8]) -> std::io::Result<()> {
    let mask: [u8; 4] = [0x37, 0xfa, 0x21, 0x3d];
    let mut frame = Vec::with_capacity(14 + payload.len());
    frame.push(0x81); // FIN + text frame
    if payload.len() < 126 {
        frame.push(0x80 | (payload.len() as u8));
    } else if payload.len() <= 65535 {
        frame.push(0x80 | 126);
        frame.extend_from_slice(&(payload.len() as u16).to_be_bytes());
    } else {
        frame.push(0x80 | 127);
        frame.extend_from_slice(&(payload.len() as u64).to_be_bytes());
    }
    frame.extend_from_slice(&mask);
    for (i, &b) in payload.iter().enumerate() {
        frame.push(b ^ mask[i & 3]);
    }
    stream.write_all(&frame)?;
    stream.flush()
}

fn read_ws_frame(stream: &mut TcpStream) -> std::io::Result<(u8, Vec<u8>)> {
    let mut hdr = [0u8; 2];
    stream.read_exact(&mut hdr)?;
    let opcode = hdr[0] & 0x0f;
    let masked = (hdr[1] & 0x80) != 0;
    let mut len = (hdr[1] & 0x7f) as usize;
    if len == 126 {
        let mut ext = [0u8; 2];
        stream.read_exact(&mut ext)?;
        len = u16::from_be_bytes(ext) as usize;
    } else if len == 127 {
        let mut ext = [0u8; 8];
        stream.read_exact(&mut ext)?;
        len = u64::from_be_bytes(ext) as usize;
    }
    let mut mask = [0u8; 4];
    if masked {
        stream.read_exact(&mut mask)?;
    }
    let mut data = vec![0u8; len];
    stream.read_exact(&mut data)?;
    if masked {
        for (i, b) in data.iter_mut().enumerate() {
            *b ^= mask[i & 3];
        }
    }
    Ok((opcode, data))
}

fn connect_ws_stream(ws_url: &str) -> Option<TcpStream> {
    let (port, path) = parse_ws_url(ws_url)?;
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::LOCALHOST), port);
    let mut stream = TcpStream::connect_timeout(&addr, Duration::from_millis(800)).ok()?;
    // Justification: best-effort socket timeout configuration on loopback CDP WebSocket
    crate::error::record_ignored(
        stream.set_read_timeout(Some(Duration::from_millis(2500))),
        "set CDP WS read timeout",
    );
    // Justification: best-effort socket timeout configuration on loopback CDP WebSocket
    crate::error::record_ignored(
        stream.set_write_timeout(Some(Duration::from_millis(1000))),
        "set CDP WS write timeout",
    );

    let handshake = format!(
        "GET {} HTTP/1.1\r\nHost: 127.0.0.1:{}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n",
        path, port
    );
    stream.write_all(handshake.as_bytes()).ok()?;

    let mut header_bytes = Vec::with_capacity(256);
    let mut byte = [0u8; 1];
    while !header_bytes.ends_with(b"\r\n\r\n") {
        if stream.read(&mut byte).ok()? == 0 {
            return None;
        }
        header_bytes.push(byte[0]);
        if header_bytes.len() > 8192 {
            return None;
        }
    }
    let header_str = String::from_utf8_lossy(&header_bytes);
    if !header_str.contains("101") {
        return None;
    }
    Some(stream)
}

fn evaluate_dismiss_on_ws(ws_url: &str) -> Option<serde_json::Value> {
    let mut stream = connect_ws_stream(ws_url)?;

    let cmd = serde_json::json!({
        "id": 1,
        "method": "Runtime.evaluate",
        "params": {
            "expression": DISMISS_SECURITY_NOTICE_JS,
            "awaitPromise": true,
            "returnByValue": true
        }
    });
    let payload = serde_json::to_vec(&cmd).ok()?;
    write_masked_ws_text_frame(&mut stream, &payload).ok()?;

    for _ in 0..16 {
        let (opcode, data) = read_ws_frame(&mut stream).ok()?;
        if opcode == 0x8 {
            return None;
        }
        if opcode != 0x1 {
            continue;
        }
        if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&data) {
            if msg.get("id").and_then(|v| v.as_i64()) == Some(1) {
                return msg
                    .get("result")
                    .and_then(|r| r.get("result"))
                    .and_then(|r| r.get("value"))
                    .cloned();
            }
        }
    }
    None
}

/// Poll the instance's CDP `DevToolsActivePort` up to `timeout`, uncheck the
/// "Yes, I agree to help improve Antigravity by allowing Google to collect and
/// use my Interactions data..." checkbox, and click "Accept and continue".
pub fn dismiss_security_notice_dialog_for_data_dir(
    data_dir: &Path,
    bound_email: Option<&str>,
    timeout: Duration,
) -> bool {
    let deadline = Instant::now() + timeout;
    while Instant::now() < deadline {
        let ports = read_active_ports(data_dir);
        for port in ports {
            let ws_urls = fetch_page_ws_urls(port);
            for ws_url in ws_urls {
                if let Some(val) = evaluate_dismiss_on_ws(&ws_url) {
                    let status = val.get("status").and_then(|v| v.as_str()).unwrap_or("");
                    if status == "dismissed" {
                        let unchecked = val.get("unchecked").and_then(|v| v.as_u64()).unwrap_or(0);
                        crate::modules::logger::log_info(&format!(
                            "[Instance] Auto-dismissed 'Security Notice & Data Use' dialog via CDP (unchecked_share={}, data_dir={:?})",
                            unchecked, data_dir
                        ));
                        super::storage::record_onboarding_acceptance_fallback(
                            data_dir,
                            bound_email,
                        );
                        return true;
                    }
                }
            }
        }
        std::thread::sleep(Duration::from_millis(250));
    }

    super::storage::record_onboarding_acceptance_fallback(data_dir, bound_email);
    false
}

/// Spawn a detached background thread that watches for and auto-dismisses the
/// "Security Notice & Data Use" dialog after an instance launch or switch.
pub fn spawn_security_notice_dismisser(
    data_dir: PathBuf,
    bound_email: Option<String>,
    timeout: Duration,
) {
    std::thread::spawn(move || {
        dismiss_security_notice_dialog_for_data_dir(&data_dir, bound_email.as_deref(), timeout);
    });
}

fn cdp_send_and_recv(
    stream: &mut TcpStream,
    id: i64,
    method: &str,
    params: serde_json::Value,
) -> Option<serde_json::Value> {
    let cmd = serde_json::json!({
        "id": id,
        "method": method,
        "params": params
    });
    let payload = serde_json::to_vec(&cmd).ok()?;
    write_masked_ws_text_frame(stream, &payload).ok()?;

    for _ in 0..24 {
        let (opcode, data) = read_ws_frame(stream).ok()?;
        if opcode == 0x8 {
            return None;
        }
        if opcode != 0x1 {
            continue;
        }
        if let Ok(msg) = serde_json::from_slice::<serde_json::Value>(&data) {
            if msg.get("id").and_then(|v| v.as_i64()) == Some(id) {
                return Some(msg);
            }
        }
    }
    None
}

const CANCEL_ACTIVE_TASK_JS: &str = r#"(() => {
    const cancelBtn = document.querySelector('button[data-tooltip-id="input-send-button-cancel-tooltip"], button[aria-label*="Cancel"]');
    if (cancelBtn) { cancelBtn.click(); return 'cancelled_via_button'; }
    const el = document.querySelector('[data-testid="send-button"]') || document.querySelector('[aria-label="Message input"]');
    if (el) {
        const key = Object.keys(el).find(k => k.startsWith('__reactFiber$'));
        let fiber = key ? el[key] : null;
        while (fiber) {
            if (fiber.memoizedProps?.cancelInvocation && fiber.memoizedProps?.isRunning) {
                fiber.memoizedProps.cancelInvocation();
                return 'cancelled_via_fiber';
            }
            fiber = fiber.return;
        }
    }
    return 'not_running';
})()"#;

fn submit_prompt_on_ws(ws_url: &str, prompt_text: &str, cancel_running_first: bool) -> bool {
    let mut stream = match connect_ws_stream(ws_url) {
        Some(s) => s,
        None => return false,
    };

    let mut msg_id = 1;

    // When cancel_running_first == true (Send mode):
    // Execute JavaScript in the Antigravity IDE webview to cancel the active task before typing
    if cancel_running_first {
        if let Some(cancel_res) = cdp_send_and_recv(
            &mut stream,
            msg_id,
            "Runtime.evaluate",
            serde_json::json!({
                "expression": CANCEL_ACTIVE_TASK_JS,
                "returnByValue": true
            }),
        ) {
            let cancel_val = cancel_res
                .get("result")
                .and_then(|r| r.get("result"))
                .and_then(|r| r.get("value"))
                .and_then(|v| v.as_str())
                .unwrap_or("");
            if cancel_val == "cancelled_via_button" || cancel_val == "cancelled_via_fiber" {
                crate::modules::logger::log_info(&format!(
                    "[Instance] Cancelled active task via CDP ({cancel_val}) before dispatching prompt"
                ));
                // If cancelled, sleep 250ms for the agent to cancel and UI to settle to idle
                std::thread::sleep(Duration::from_millis(250));
            }
        }
        msg_id += 1;
    }

    // 1. Focus the message input and clear any draft text
    let focus_js = r#"(() => {
        const selectorLadder = [
            '[aria-label="Message input"]',
            '[role="combobox"][contenteditable="true"]',
            'div.monaco-editor [contenteditable="true"]',
            'div.interactive-input-editor [contenteditable="true"]',
            '[role="textbox"][contenteditable="true"]',
            'textarea.monaco-mouse-cursor-text',
            'textarea[placeholder*="Ask"]',
            'div[contenteditable="true"]',
        ];
        let el = null;
        for (const sel of selectorLadder) {
            el = document.querySelector(sel);
            if (el) break;
        }
        if (!el) return 'no_input';
        el.focus();
        try {
            const sel = window.getSelection();
            if (sel) {
                const range = document.createRange();
                range.selectNodeContents(el);
                sel.removeAllRanges();
                sel.addRange(range);
            }
        } catch (_) {}
        return 'focused';
    })()"#;
    let Some(focus_res) = cdp_send_and_recv(
        &mut stream,
        msg_id,
        "Runtime.evaluate",
        serde_json::json!({ "expression": focus_js, "returnByValue": true }),
    ) else {
        return false;
    };
    msg_id += 1;
    let focus_val = focus_res
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(|r| r.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    if focus_val != "focused" {
        return false;
    }

    // 2. Insert text via CDP Input.insertText so Lexical/React state updates properly
    if cdp_send_and_recv(
        &mut stream,
        msg_id,
        "Input.insertText",
        serde_json::json!({ "text": prompt_text }),
    )
    .is_none()
    {
        return false;
    }
    msg_id += 1;

    std::thread::sleep(Duration::from_millis(120));

    // 3. Click the Send message button or dispatch Enter key
    let send_js = r#"(() => {
        const btnSelectors = [
            'button[aria-label="Send message"]',
            'button[aria-label="Send Prompt"]',
            'button[aria-label="Send"]',
            'button[title*="Send"]',
            'button.codicon-send',
        ];
        for (const sel of btnSelectors) {
            const btn = document.querySelector(sel);
            if (btn && !btn.disabled) {
                btn.click();
                return 'sent';
            }
        }
        // Fallback: Dispatch Enter key event to active element
        const active = document.activeElement;
        if (active) {
            active.dispatchEvent(new KeyboardEvent('keydown', { key: 'Enter', code: 'Enter', keyCode: 13, which: 13, bubbles: true }));
            return 'sent';
        }
        return 'not_ready';
    })()"#;
    let Some(send_res) = cdp_send_and_recv(
        &mut stream,
        msg_id,
        "Runtime.evaluate",
        serde_json::json!({ "expression": send_js, "returnByValue": true }),
    ) else {
        return false;
    };
    let send_val = send_res
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(|r| r.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    send_val == "sent"
}

/// Submit a prompt directly into the running Antigravity IDE's conversation composer
/// via CDP (`Input.insertText` + clicking `button[aria-label="Send message"]`).
/// When `cancel_running_first` is true, cancels any active task before typing.
pub fn submit_prompt_via_cdp_for_instance(
    instance_id: &str,
    prompt_text: &str,
    cancel_running_first: bool,
) -> Result<(), String> {
    let trimmed = prompt_text.trim();
    if trimmed.is_empty() {
        return Err("Prompt text is empty".to_string());
    }
    let data_dir =
        if instance_id.is_empty() || instance_id == "default" || instance_id == "__default__" {
            super::storage::get_default_antigravity_data_dir()
        } else if let Ok(reg) = super::load_registry() {
            reg.instances
                .iter()
                .find(|i| i.id == instance_id || i.name == instance_id)
                .map(|i| PathBuf::from(&i.data_dir))
                .unwrap_or_else(super::storage::get_default_antigravity_data_dir)
        } else {
            super::storage::get_default_antigravity_data_dir()
        };

    let ports = read_active_ports(&data_dir);
    if ports.is_empty() {
        return Err(format!(
            "No active DevTools port found for instance '{}' (data_dir: {:?})",
            instance_id, data_dir
        ));
    }

    for port in ports {
        for ws_url in fetch_page_ws_urls(port) {
            if submit_prompt_on_ws(&ws_url, trimmed, cancel_running_first) {
                crate::modules::logger::log_info(&format!(
                    "[Instance] Submitted prompt ({} chars, cancel_first: {}) into live Antigravity conversation via CDP for instance '{}'",
                    trimmed.len(),
                    cancel_running_first,
                    instance_id
                ));
                return Ok(());
            }
        }
    }
    Err(format!(
        "Failed to submit prompt via CDP for instance '{}': DevTools page targets did not accept input",
        instance_id
    ))
}

/// Cancel any currently active task/generating turn via CDP by clicking
/// 'Stop Generating' / 'Stop' button or dispatching Escape key.
pub fn cancel_active_task_on_ws(ws_url: &str) -> bool {
    let mut stream = match connect_ws_stream(ws_url) {
        Some(s) => s,
        None => return false,
    };

    let cancel_js = r#"(() => {
        const stopSelectors = [
            'button[aria-label="Stop Generating"]',
            'button[aria-label="Stop generation"]',
            'button[aria-label="Stop"]',
            'button[title*="Stop Generating"]',
            'button[title*="Stop"]',
            'button.codicon-stop-circle',
            'button.codicon-debug-stop'
        ];
        for (const sel of stopSelectors) {
            const btn = document.querySelector(sel);
            if (btn && !btn.disabled) {
                btn.click();
                return 'stopped';
            }
        }
        // Fallback: Dispatch Escape key event to active element to cancel execution
        const active = document.activeElement;
        if (active) {
            active.dispatchEvent(new KeyboardEvent('keydown', { key: 'Escape', code: 'Escape', keyCode: 27, which: 27, bubbles: true }));
            return 'escaped';
        }
        return 'not_found';
    })()"#;

    let Some(eval_res) = cdp_send_and_recv(
        &mut stream,
        1,
        "Runtime.evaluate",
        serde_json::json!({ "expression": cancel_js, "returnByValue": true }),
    ) else {
        return false;
    };
    let val = eval_res
        .get("result")
        .and_then(|r| r.get("result"))
        .and_then(|r| r.get("value"))
        .and_then(|v| v.as_str())
        .unwrap_or("");
    val == "stopped" || val == "escaped"
}

/// Cancel active tasks for an instance via live Chrome DevTools Protocol.
pub fn cancel_active_task_via_cdp_for_instance(instance_id: &str) -> bool {
    let data_dir =
        if instance_id.is_empty() || instance_id == "default" || instance_id == "__default__" {
            super::storage::get_default_antigravity_data_dir()
        } else if let Ok(reg) = super::load_registry() {
            reg.instances
                .iter()
                .find(|i| i.id == instance_id || i.name == instance_id)
                .map(|i| PathBuf::from(&i.data_dir))
                .unwrap_or_else(super::storage::get_default_antigravity_data_dir)
        } else {
            super::storage::get_default_antigravity_data_dir()
        };

    let mut cancelled = false;
    for port in read_active_ports(&data_dir) {
        for ws_url in fetch_page_ws_urls(port) {
            if cancel_active_task_on_ws(&ws_url) {
                crate::modules::logger::log_info(&format!(
                    "[Instance] Cancelled active task via CDP for instance '{}'",
                    instance_id
                ));
                cancelled = true;
            }
        }
    }
    cancelled
}
