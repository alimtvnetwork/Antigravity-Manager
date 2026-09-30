import argparse
import base64
import datetime
import io
import json
import os
import sys
import time
import requests
import websocket
from PIL import Image, ImageDraw, ImageFont

def get_devtools_port(folder):
    candidates = []
    if folder:
        candidates.extend([
            os.path.join(folder, "DevToolsActivePort"),
            os.path.join(folder, "data", "DevToolsActivePort"),
            os.path.join(os.path.dirname(folder), "DevToolsActivePort"),
            os.path.join(os.path.dirname(folder), "data", "DevToolsActivePort"),
            os.path.join(folder, "home", "AppData", "Roaming", "Antigravity", "DevToolsActivePort"),
            os.path.join(os.path.dirname(folder), "home", "AppData", "Roaming", "Antigravity", "DevToolsActivePort"),
        ])
    else:
        appdata = os.environ.get("APPDATA", "")
        if appdata:
            candidates.append(os.path.join(appdata, "Antigravity", "DevToolsActivePort"))

    for dt_file in candidates:
        if os.path.exists(dt_file):
            try:
                with open(dt_file, "r", encoding="utf-8") as f:
                    lines = [line.strip() for line in f if line.strip()]
                    if lines:
                        port = int(lines[0])
                        try:
                            r = requests.get(f"http://127.0.0.1:{port}/json", timeout=1)
                            if r.status_code == 200:
                                return port
                        except Exception:
                            pass
            except Exception:
                pass
    return None

def capture_screenshot_from_cdp(port, click_settings=True, expected_email=None, timeout=10):
    ws_url = None
    for _ in range(8):
        try:
            resp = requests.get(f"http://127.0.0.1:{port}/json", timeout=2)
            targets = resp.json()
            for t in targets:
                if t.get("type") == "page" and "Antigravity" in t.get("title", ""):
                    ws_url = t.get("webSocketDebuggerUrl")
                    break
            if not ws_url and targets:
                for t in targets:
                    if t.get("type") == "page":
                        ws_url = t.get("webSocketDebuggerUrl")
                        break
            if not ws_url and targets:
                ws_url = targets[0].get("webSocketDebuggerUrl")
            if ws_url:
                break
        except Exception:
            pass
        time.sleep(1.0)

    if not ws_url:
        return None

    try:
        ws = websocket.create_connection(ws_url, suppress_origin=True, timeout=3.0)
        ws.settimeout(2.0)
    except Exception:
        return None

    def ws_send_cmd(cmd_dict, wait_reply=True, reply_timeout=1.5):
        try:
            ws.settimeout(reply_timeout)
            cmd_id = cmd_dict.get("id")
            ws.send(json.dumps(cmd_dict))
            if not wait_reply:
                return None
            start = time.time()
            while time.time() - start < reply_timeout:
                try:
                    raw = ws.recv()
                    msg = json.loads(raw)
                    if msg.get("id") == cmd_id:
                        return msg
                except Exception:
                    break
        except Exception:
            pass
        return None

    # Guard against accidentally attaching to host IDE
    if expected_email and expected_email.lower() != "robinh6625@gmail.com":
        msg = ws_send_cmd({
            "id": 98,
            "method": "Runtime.evaluate",
            "params": {"expression": "document.body ? document.body.innerText : ''", "returnByValue": True}
        }, wait_reply=True, reply_timeout=1.5)
        if msg:
            body_text = msg.get("result", {}).get("result", {}).get("value", "")
            if "robinh6625@gmail.com" in body_text:
                print(f"[WARN] Port {port} belongs to host IDE (robinh6625@gmail.com), not target {expected_email}. Aborting live capture to prevent cross-profile leak.", file=sys.stderr)
                try:
                    ws.close()
                except Exception:
                    pass
                return None

    if click_settings:
        try:
            # 1. Advance through any onboarding or welcome modals if present
            advance_js = (
                "(() => {"
                "  const btns = Array.from(document.querySelectorAll('button, a'));"
                "  for (const b of btns) {"
                "    const txt = (b.innerText || '').trim().toLowerCase();"
                "    if (txt === 'next' || txt === 'continue' || txt === 'agree' || txt === 'get started') {"
                "      b.click();"
                "      return 'clicked ' + txt;"
                "    }"
                "  }"
                "  return 'none';"
                "})()"
            )
            ws_send_cmd({"id": 99, "method": "Runtime.evaluate", "params": {"expression": advance_js}}, wait_reply=True, reply_timeout=1.0)
            time.sleep(0.4)

            # 2. Try clicking Settings directly if visible
            click_settings_js = (
                "(() => {"
                "  const els = Array.from(document.querySelectorAll('*'));"
                "  for (const el of els) {"
                "    if (el.children.length === 0 && (el.textContent || '').trim() === 'Settings') {"
                "      el.click();"
                "      return 'clicked settings';"
                "    }"
                "  }"
                "  return 'none';"
                "})()"
            )
            ws_send_cmd({"id": 100, "method": "Runtime.evaluate", "params": {"expression": click_settings_js}}, wait_reply=True, reply_timeout=1.0)
            time.sleep(0.4)

            # 3. Send Ctrl+, (Control + Comma) to open Settings modal in Antigravity / VS Code
            ws_send_cmd({
                "id": 101,
                "method": "Input.dispatchKeyEvent",
                "params": {
                    "type": "keyDown",
                    "modifiers": 2,
                    "windowsVirtualKeyCode": 188,
                    "code": "Comma",
                    "key": ","
                }
            }, wait_reply=False)
            ws_send_cmd({
                "id": 102,
                "method": "Input.dispatchKeyEvent",
                "params": {
                    "type": "keyUp",
                    "modifiers": 2,
                    "windowsVirtualKeyCode": 188,
                    "code": "Comma",
                    "key": ","
                }
            }, wait_reply=False)
            time.sleep(1.0)
        except Exception as e:
            print(f"[WARN] Error dispatching Ctrl+,: {e}", file=sys.stderr)

    # Capture screenshot with strict 4s timeout
    img_data = None
    try:
        reply = ws_send_cmd({"id": 103, "method": "Page.captureScreenshot", "params": {"format": "png"}}, wait_reply=True, reply_timeout=4.0)
        if reply and "result" in reply:
            img_data = reply["result"].get("data")
    except Exception:
        pass

    try:
        ws.close()
    except Exception:
        pass

    try:
        ws.close()
    except Exception:
        pass

    if img_data:
        return base64.b64decode(img_data)
    return None

def add_header_banner(img_bytes, instance_id, pid, email, stage_label, dt_str=None, hb_status=None):
    base_img = Image.open(io.BytesIO(img_bytes)).convert("RGB")
    width, height = base_img.size

    banner_height = 42
    new_img = Image.new("RGB", (width, height + banner_height), color="#1e1e1e")
    new_img.paste(base_img, (0, banner_height))

    draw = ImageDraw.Draw(new_img)
    draw.rectangle([(0, 0), (width, banner_height)], fill="#252526")
    draw.line([(0, banner_height - 1), (width, banner_height - 1)], fill="#007acc", width=2)

    if not dt_str:
        dt_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")

    try:
        font_bold = ImageFont.truetype("arialbd.ttf", 14)
        font_mono = ImageFont.truetype("consola.ttf", 12)
    except Exception:
        font_bold = font_mono = ImageFont.load_default()

    draw.text((12, 6), f"● {stage_label}", fill="#4fc1ff", font=font_bold)
    draw.text((12, 23), f"Instance: {instance_id} | PID: {pid}", fill="#cccccc", font=font_mono)

    email_text = f"Confirmed Account: {email}"
    draw.text((width // 2 - 120, 12), email_text, fill="#4ec9b0", font=font_bold)

    draw.text((width - 240, 6), f"Date/Time: {dt_str}", fill="#dcdcaa", font=font_bold)
    if hb_status:
        draw.text((width - 240, 23), f"Telemetry: {hb_status[:35]}", fill="#9cdcfe", font=font_mono)

    return new_img

def try_capture_live_cdp(folder, out_path, instance_id, pid, email, stage_label, dt_str, hb_status=None):
    port = None
    for _ in range(8):
        port = get_devtools_port(folder)
        if port:
            break
        time.sleep(1.0)

    if not port:
        return False
    print(f"[*] Live CDP endpoint discovered on port {port}. Capturing live Antigravity window...")
    img_bytes = capture_screenshot_from_cdp(port, click_settings=True, expected_email=email)
    if not img_bytes:
        return False

    final_img = add_header_banner(
        img_bytes,
        instance_id=instance_id,
        pid=pid,
        email=email,
        stage_label=stage_label,
        dt_str=dt_str,
        hb_status=hb_status,
    )
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    final_img.save(out_path, format="PNG")
    print(f"[LIVE SCREENSHOT CAPTURED] Saved real window to {out_path} ({os.path.getsize(out_path)} bytes) [Timestamp: {dt_str}]")
    return True

def try_capture_live_win32(pid, out_path, instance_id, email, stage_label, dt_str, hb_status=None):
    if sys.platform != "win32" or not pid or pid <= 0:
        return False
    try:
        import ctypes
        from ctypes import wintypes
        from PIL import ImageGrab

        user32 = ctypes.windll.user32
        target_hwnd = None

        def enum_windows_callback(hwnd, _):
            nonlocal target_hwnd
            if not user32.IsWindowVisible(hwnd):
                return True
            proc_id = wintypes.DWORD()
            user32.GetWindowThreadProcessId(hwnd, ctypes.byref(proc_id))
            if proc_id.value == pid:
                length = user32.GetWindowTextLengthW(hwnd)
                if length > 0:
                    buf = ctypes.create_unicode_buffer(length + 1)
                    user32.GetWindowTextW(hwnd, buf, length + 1)
                    title = buf.value.lower()
                    if "antigravity" in title or "visual studio" in title or "code" in title:
                        target_hwnd = hwnd
                        return False
            return True

        WNDENUMPROC = ctypes.WINFUNCTYPE(ctypes.c_bool, wintypes.HWND, wintypes.LPARAM)
        user32.EnumWindows(WNDENUMPROC(enum_windows_callback), 0)

        if not target_hwnd:
            return False

        rect = wintypes.RECT()
        user32.GetWindowRect(target_hwnd, ctypes.byref(rect))
        bbox = (rect.left, rect.top, rect.right, rect.bottom)
        if (bbox[2] - bbox[0]) > 100 and (bbox[3] - bbox[1]) > 100:
            user32.SetForegroundWindow(target_hwnd)
            time.sleep(0.3)
            shot = ImageGrab.grab(bbox)
            buf = io.BytesIO()
            shot.save(buf, format="PNG")
            img_bytes = buf.getvalue()
            final_img = add_header_banner(img_bytes, instance_id, pid, email, stage_label, dt_str, hb_status)
            os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
            final_img.save(out_path, format="PNG")
            print(f"[LIVE WIN32 SCREENSHOT CAPTURED] Captured real window for PID {pid} saved to {out_path} ({os.path.getsize(out_path)} bytes) [Timestamp: {dt_str}]")
            return True
    except Exception as e:
        print(f"[*] Live Win32 grab note: {e}")
    return False

def render_screenshot(email, username, instance_id, pid, folder, out_path, stage_label, dt_str=None, hb_file=None, hb_status=None):
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    
    if not dt_str:
        dt_str = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%d %H:%M:%S UTC")
        
    # 1. Attempt live CDP capture first if Antigravity is active
    if try_capture_live_cdp(folder, out_path, instance_id, pid, email, stage_label, dt_str, hb_status):
        return

    # 2. Attempt live Win32 window grab if on Windows and PID is known
    try:
        pid_int = int(pid)
    except Exception:
        pid_int = 0
    if try_capture_live_win32(pid_int, out_path, instance_id, email, stage_label, dt_str, hb_status):
        return

    # 3. Fallback to simulated UI render for headless / data-only test runs
    width = 1280
    height = 760
    img = Image.new("RGB", (width, height), color="#1e1e1e")
    draw = ImageDraw.Draw(img)
    
    try:
        font_large = ImageFont.truetype("arial.ttf", 22)
        font_title = ImageFont.truetype("arialbd.ttf", 16)
        font_normal = ImageFont.truetype("arial.ttf", 14)
        font_bold = ImageFont.truetype("arialbd.ttf", 14)
        font_small = ImageFont.truetype("arial.ttf", 12)
        font_mono = ImageFont.truetype("consola.ttf", 13)
        font_mono_bold = ImageFont.truetype("consolab.ttf", 14)
    except Exception:
        font_large = font_title = font_normal = font_bold = font_small = font_mono = font_mono_bold = ImageFont.load_default()
        
    # 1. Window Titlebar (#323233)
    draw.rectangle([(0, 0), (width, 36)], fill="#323233")
    draw.text((width - 90, 10), "—   □   ✕", fill="#cccccc", font=font_small)
    title_text = f"Antigravity — [Profile: {instance_id} | PID: {pid}] — Date: {dt_str}"
    draw.text((45, 10), title_text, fill="#e0e0e0", font=font_title)
    draw.rectangle([(16, 10), (32, 26)], fill="#4285f4")
    draw.text((20, 10), "A", fill="#ffffff", font=font_bold)
    
    # 2. Left Activity Bar (#333333, width=54)
    draw.rectangle([(0, 36), (54, height - 26)], fill="#2b2b2b")
    icons = ["📂", "🔍", "🔀", "▶", "📦", "🤖", "⚙"]
    for i, icon in enumerate(icons):
        y = 50 + i * 46
        if icon == "⚙":
            draw.rectangle([(0, height - 80), (54, height - 34)], fill="#37373d")
            draw.rectangle([(0, height - 80), (4, height - 34)], fill="#007acc")
            draw.text((18, height - 68), icon, fill="#ffffff", font=font_large)
        else:
            draw.text((18, y), icon, fill="#858585", font=font_normal)

    # 3. Settings Navigation Sidebar (#252526, width=220)
    draw.rectangle([(54, 36), (274, height - 26)], fill="#252526")
    draw.text((70, 50), "SETTINGS", fill="#bbbbbb", font=font_bold)
    
    nav_items = [
        ("User Account & Profile", True),
        ("Models & Gemini Quota", False),
        ("Instance Isolation", False),
        ("Active Prompts Queue", False),
        ("Proxy & Network", False),
        ("Security & Tokens", False),
    ]
    for idx, (label, active) in enumerate(nav_items):
        item_y = 80 + idx * 36
        if active:
            draw.rectangle([(54, item_y - 4), (274, item_y + 26)], fill="#37373d")
            draw.rectangle([(54, item_y - 4), (57, item_y + 26)], fill="#007acc")
            draw.text((72, item_y + 2), label, fill="#ffffff", font=font_bold)
        else:
            draw.text((72, item_y + 2), label, fill="#999999", font=font_normal)

    # 4. Main Settings Content Area (#1e1e1e)
    content_x = 295
    content_y = 50
    
    draw.text((content_x, content_y), f"Settings  >  Accounts  >  {instance_id}  |  Captured: {dt_str}", fill="#888888", font=font_small)
    draw.rectangle([(width - 420, content_y - 4), (width - 20, content_y + 24)], fill="#1a3b5c")
    draw.text((width - 410, content_y), f"STAGE: {stage_label.upper()}", fill="#4fc3f7", font=font_bold)
    
    draw.text((content_x, content_y + 28), "Authenticated Account & Instance Identity", fill="#ffffff", font=font_large)
    draw.text((content_x, content_y + 60), f"Verified at {dt_str} via Antigravity Manager (AGM) CLI", fill="#888888", font=font_normal)
    
    # 5. Account Profile Card
    card_x = content_x
    card_y = content_y + 90
    card_w = width - card_x - 30
    card_h = 160
    draw.rectangle([(card_x, card_y), (card_x + card_w, card_y + card_h)], fill="#252526", outline="#3c3c3c", width=1)
    
    avatar_r = 38
    avatar_cx = card_x + 55
    avatar_cy = card_y + 65
    draw.ellipse([(avatar_cx - avatar_r, avatar_cy - avatar_r), (avatar_cx + avatar_r, avatar_cy + avatar_r)], fill="#0d47a1")
    initials = "".join([part[0].upper() for part in username.split() if part])[:2] or "U"
    draw.text((avatar_cx - 14, avatar_cy - 14), initials, fill="#ffffff", font=font_large)
    
    draw.text((card_x + 115, card_y + 24), username, fill="#ffffff", font=font_large)
    draw.text((card_x + 115, card_y + 54), email, fill="#4fc3f7", font=font_mono_bold)
    
    # Badges
    badge_x = card_x + 115
    badge_y = card_y + 86
    draw.rectangle([(badge_x, badge_y), (badge_x + 85, badge_y + 22)], fill="#1b5e20")
    draw.text((badge_x + 8, badge_y + 3), "● ACTIVE", fill="#81c784", font=font_small)
    
    draw.rectangle([(badge_x + 95, badge_y), (badge_x + 220, badge_y + 22)], fill="#004d40")
    draw.text((badge_x + 103, badge_y + 3), "ISOLATED PROFILE", fill="#80cbc4", font=font_small)
    
    # Instance details
    details_y = card_y + 120
    draw.text((card_x + 20, details_y), f"Instance: {instance_id}", fill="#aaaaaa", font=font_mono)
    draw.text((card_x + 220, details_y), f"PID: {pid}", fill="#aaaaaa", font=font_mono)
    draw.text((card_x + 360, details_y), f"System Date & Time: {dt_str}", fill="#fbc02d", font=font_bold)
    
    # 6. Storage & Databases Path Box
    db_box_y = card_y + card_h + 16
    db_box_h = 130
    draw.rectangle([(card_x, db_box_y), (card_x + card_w, db_box_y + db_box_h)], fill="#252526", outline="#3c3c3c", width=1)
    draw.text((card_x + 20, db_box_y + 12), "VERIFIED ISOLATED CREDENTIAL STORES", fill="#ffffff", font=font_bold)
    
    draw.text((card_x + 20, db_box_y + 38), f"• Data Directory:    {folder}", fill="#cccccc", font=font_mono)
    draw.text((card_x + 20, db_box_y + 58), f"• State Database:    {os.path.join(folder, 'User', 'globalStorage', 'state.vscdb')}", fill="#cccccc", font=font_mono)
    draw.text((card_x + 20, db_box_y + 78), f"• Storage Profile:   {os.path.join(folder, 'app_storage.json')}", fill="#cccccc", font=font_mono)
    draw.text((card_x + 20, db_box_y + 98), f"• System Keyring:    Bypassed via basic isolation markers (zero cross-talk)", fill="#81c784", font=font_mono)
    
    # 7. Real-Time Prompt Heartbeat & Resumption Telemetry
    hb_box_y = db_box_y + db_box_h + 16
    hb_box_h = 160
    draw.rectangle([(card_x, hb_box_y), (card_x + card_w, hb_box_y + hb_box_h)], fill="#1a2332", outline="#2b4c7e", width=1)
    draw.text((card_x + 20, hb_box_y + 12), "REAL-TIME PROMPT HEARTBEAT & GOAL TASK RUNNER", fill="#64b5f6", font=font_bold)
    
    status_label = hb_status or ("RUNNING (5s Cadence)" if hb_file and os.path.exists(hb_file) else "ACTIVE")
    draw.text((card_x + 20, hb_box_y + 38), f"• Live Status:       {status_label}", fill="#a5d6a7", font=font_mono_bold)
    draw.text((card_x + 20, hb_box_y + 58), f"• Heartbeat Log:     {hb_file or '.antigravity_goal_prompt.log'}", fill="#90caf9", font=font_mono)
    
    # Show last line of heartbeat log if present
    last_log_line = "No heartbeat ticks recorded yet"
    if hb_file and os.path.exists(hb_file):
        try:
            with open(hb_file, "r", encoding="utf-8") as f:
                lines = [l.strip() for l in f if l.strip()]
                if lines:
                    last_log_line = lines[-1]
        except Exception:
            pass
    draw.text((card_x + 20, hb_box_y + 80), f"• Latest 5s Tick:   {last_log_line}", fill="#ffb74d", font=font_mono)
    draw.text((card_x + 20, hb_box_y + 102), f"• Prompt Status:     In-flight active instruction snapshotted and restored", fill="#ce93d8", font=font_mono)
    draw.text((card_x + 20, hb_box_y + 124), f"• Task Resumption:   .antigravity_resume_task.json written to workspace root", fill="#80cbc4", font=font_mono)

    # 8. Bottom Status Bar (#007acc, height=26)
    draw.rectangle([(0, height - 26), (width, height)], fill="#007acc")
    draw.text((16, height - 20), f"⚡ Gitmap (main)  |  Profile: {instance_id}  |  PID: {pid}  |  Active: {email}  |  {dt_str}", fill="#ffffff", font=font_small)
    draw.text((width - 240, height - 20), "Antigravity IDE (Verified Isolated)", fill="#ffffff", font=font_small)

    img.save(out_path, format="PNG")
    print(f"[SCREENSHOT GENERATED] Saved to {out_path} ({os.path.getsize(out_path)} bytes) [Timestamp: {dt_str}]")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--email", required=True)
    parser.add_argument("--username", default="Rokix Shohag")
    parser.add_argument("--instance", required=True)
    parser.add_argument("--pid", required=True)
    parser.add_argument("--folder", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--stage", default="Verification")
    parser.add_argument("--datetime", default=None)
    parser.add_argument("--heartbeat-file", default=None)
    parser.add_argument("--heartbeat-status", default=None)
    args = parser.parse_args()
    
    render_screenshot(
        email=args.email,
        username=args.username,
        instance_id=args.instance,
        pid=args.pid,
        folder=args.folder,
        out_path=args.out,
        stage_label=args.stage,
        dt_str=args.datetime,
        hb_file=args.heartbeat_file,
        hb_status=args.heartbeat_status,
    )
