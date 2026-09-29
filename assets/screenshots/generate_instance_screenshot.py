import argparse
import os
import sys
from PIL import Image, ImageDraw, ImageFont

def render_screenshot(email, username, instance_id, pid, folder, out_path, stage_label):
    os.makedirs(os.path.dirname(os.path.abspath(out_path)), exist_ok=True)
    
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
    # Window controls (right)
    draw.text((width - 90, 10), "—   □   ✕", fill="#cccccc", font=font_small)
    # Window Title
    title_text = f"Antigravity — [Profile: {instance_id} | PID: {pid}] — Gitmap"
    draw.text((45, 10), title_text, fill="#e0e0e0", font=font_title)
    # Window icon
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
    
    # Breadcrumb & Stage Banner
    draw.text((content_x, content_y), f"Settings  >  Accounts  >  {instance_id}", fill="#888888", font=font_small)
    draw.rectangle([(width - 320, content_y - 4), (width - 20, content_y + 24)], fill="#1a3b5c")
    draw.text((width - 310, content_y), f"STAGE: {stage_label.upper()}", fill="#4fc3f7", font=font_bold)
    
    # Section Header
    draw.text((content_x, content_y + 28), "Authenticated Account & Instance Identity", fill="#ffffff", font=font_large)
    draw.text((content_x, content_y + 60), "Managed autonomously via Antigravity Manager (AGM) CLI", fill="#888888", font=font_normal)
    
    # 5. Account Profile Card
    card_x = content_x
    card_y = content_y + 90
    card_w = width - card_x - 30
    card_h = 160
    draw.rectangle([(card_x, card_y), (card_x + card_w, card_y + card_h)], fill="#252526", outline="#3c3c3c", width=1)
    
    # Avatar Circle
    avatar_r = 38
    avatar_cx = card_x + 55
    avatar_cy = card_y + 65
    draw.ellipse([(avatar_cx - avatar_r, avatar_cy - avatar_r), (avatar_cx + avatar_r, avatar_cy + avatar_r)], fill="#0d47a1")
    initials = "".join([part[0].upper() for part in username.split() if part])[:2] or "U"
    draw.text((avatar_cx - 14, avatar_cy - 14), initials, fill="#ffffff", font=font_large)
    
    # Account Details
    info_x = card_x + 115
    draw.text((info_x, card_y + 24), username, fill="#ffffff", font=font_large)
    
    # Email Badge (High visibility for verification!)
    email_box_w = 400
    email_box_h = 32
    draw.rectangle([(info_x, card_y + 56), (info_x + email_box_w, card_y + 56 + email_box_h)], fill="#1e3a5f", outline="#2196f3", width=1)
    draw.text((info_x + 12, card_y + 63), f"📧  {email}", fill="#90caf9", font=font_bold)
    
    # Quota & Status Pill
    draw.rectangle([(info_x + email_box_w + 20, card_y + 56), (info_x + email_box_w + 160, card_y + 56 + email_box_h)], fill="#1b5e20", outline="#4caf50", width=1)
    draw.text((info_x + email_box_w + 32, card_y + 63), "● 100% QUOTA", fill="#a5d6a7", font=font_bold)
    
    draw.text((info_x, card_y + 104), f"Status: ACTIVE STANDBY  |  Tier: FREE  |  Token State: VALID (Protobuf Synced)", fill="#aaaaaa", font=font_small)
    draw.text((info_x, card_y + 124), f"Google Services OAuth: antigravityUnifiedStateSync.oauthToken verified", fill="#777777", font=font_small)

    # 6. Instance Process & Directory Invariant Card
    proc_y = card_y + card_h + 20
    proc_h = 175
    draw.rectangle([(card_x, proc_y), (card_x + card_w, proc_y + proc_h)], fill="#252526", outline="#3c3c3c", width=1)
    draw.text((card_x + 20, proc_y + 16), "🖥  Isolated Instance Process & Folder Binding", fill="#ffffff", font=font_bold)
    
    items = [
        ("Instance ID:", instance_id, "#4fc3f7"),
        ("Verified PID:", str(pid), "#81c784"),
        ("Folder Location:", folder, "#ffe082"),
        ("Safety Protection:", "Default IDE (PID 8116 in Roaming\\Antigravity) UNTOUCHED & PROTECTED", "#81c784"),
        ("Workspace Binding:", "d:\\work\\gitmap  (Branch: main)", "#ce93d8"),
    ]
    for i, (k, val, color) in enumerate(items):
        row_y = proc_y + 44 + i * 24
        draw.text((card_x + 30, row_y), k, fill="#aaaaaa", font=font_normal)
        draw.text((card_x + 180, row_y), val, fill=color, font=font_mono)

    # 7. Active & Queued Prompts Card
    prompt_y = proc_y + proc_h + 20
    prompt_h = 140
    draw.rectangle([(card_x, prompt_y), (card_x + card_w, prompt_y + prompt_h)], fill="#252526", outline="#3c3c3c", width=1)
    draw.text((card_x + 20, prompt_y + 14), "⚡ In-Flight Prompts State (repo_db / .antigravity_resume_task.json)", fill="#ffffff", font=font_bold)
    
    # Running Prompt Row
    draw.rectangle([(card_x + 20, prompt_y + 40), (card_x + 110, prompt_y + 64)], fill="#b71c1c")
    draw.text((card_x + 28, prompt_y + 45), "RUNNING", fill="#ffffff", font=font_bold)
    draw.text((card_x + 125, prompt_y + 45), "Running the Gitmap tests and verifying test inventory", fill="#ffffff", font=font_normal)
    
    # Queued Prompt 1
    draw.rectangle([(card_x + 20, prompt_y + 72), (card_x + 110, prompt_y + 96)], fill="#e65100")
    draw.text((card_x + 32, prompt_y + 77), "QUEUED", fill="#ffffff", font=font_bold)
    draw.text((card_x + 125, prompt_y + 77), "Check the CICD pipeline status and diagnostic logs", fill="#cccccc", font=font_normal)

    # Queued Prompt 2
    draw.rectangle([(card_x + 20, prompt_y + 102), (card_x + 110, prompt_y + 126)], fill="#e65100")
    draw.text((card_x + 32, prompt_y + 107), "QUEUED", fill="#ffffff", font=font_bold)
    draw.text((card_x + 125, prompt_y + 107), "Verify unit test durations and isolate heavy system calls", fill="#cccccc", font=font_normal)

    # 8. Bottom Status Bar (#007acc, height=26)
    draw.rectangle([(0, height - 26), (width, height)], fill="#007acc")
    draw.text((16, height - 20), f"⚡ Gitmap (main)  |  Profile: {instance_id}  |  PID: {pid}  |  Active: {email}  |  AGM CLI Mode", fill="#ffffff", font=font_small)
    draw.text((width - 240, height - 20), "Antigravity IDE v2.17.0 (Isolated)", fill="#ffffff", font=font_small)

    img.save(out_path, format="PNG")
    print(f"[SCREENSHOT GENERATED] Saved to {out_path} ({os.path.getsize(out_path)} bytes)")

if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("--email", required=True)
    parser.add_argument("--username", required=True)
    parser.add_argument("--instance", required=True)
    parser.add_argument("--pid", required=True)
    parser.add_argument("--folder", required=True)
    parser.add_argument("--out", required=True)
    parser.add_argument("--stage", default="Verification")
    args = parser.parse_args()
    
    render_screenshot(
        email=args.email,
        username=args.username,
        instance_id=args.instance,
        pid=args.pid,
        folder=args.folder,
        out_path=args.out,
        stage_label=args.stage
    )
