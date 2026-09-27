#!/usr/bin/env python3
"""
Standalone Telegram Bot Connection & Chat ID Auto-Discovery Script for Antigravity Manager (AGM).
Zero external dependencies (uses standard library urllib, json, os, sys, pathlib).

Usage:
    python 03-ai-scripts/42-telegram-connect.py <BOT_TOKEN> [CHAT_ID]
    python 03-ai-scripts/42-telegram-connect.py --status
    python 03-ai-scripts/42-telegram-connect.py --help
"""

import sys
import os
import json
import urllib.request
import urllib.parse
import urllib.error
from pathlib import Path


def get_agm_data_dir() -> Path:
    """Resolve AGM data directory across Windows, Linux, and macOS."""
    env_dir = os.environ.get("ABV_DATA_DIR")
    if env_dir and env_dir.strip():
        p = Path(env_dir.strip())
        p.mkdir(parents=True, exist_ok=True)
        return p

    home = Path.home()
    default_dir = home / ".antigravity_tools"
    default_dir.mkdir(parents=True, exist_ok=True)
    return default_dir


def call_telegram_api(token: str, method: str, payload: dict = None) -> dict:
    """Make HTTP POST or GET to Telegram Bot API."""
    url = f"https://api.telegram.org/bot{token}/{method}"
    headers = {"Content-Type": "application/json"}

    if payload is not None:
        data = json.dumps(payload).encode("utf-8")
        req = urllib.request.Request(url, data=data, headers=headers, method="POST")
    else:
        req = urllib.request.Request(url, headers=headers, method="GET")

    try:
        with urllib.request.urlopen(req, timeout=15) as resp:
            body = resp.read().decode("utf-8")
            return json.loads(body)
    except urllib.error.HTTPError as e:
        err_body = e.read().decode("utf-8", errors="ignore")
        try:
            parsed = json.loads(err_body)
            desc = parsed.get("description", err_body)
        except Exception:
            desc = err_body
        raise RuntimeError(f"Telegram API {method} error (HTTP {e.code}): {desc}")
    except Exception as e:
        raise RuntimeError(f"Network error contacting Telegram API ({method}): {e}")


def register_bot_commands(token: str) -> None:
    """Register interactive slash commands via setMyCommands."""
    commands = [
        {"command": "ping", "description": "Instant node liveness, version & Git telemetry"},
        {"command": "observe", "description": "Inspect active account, quota & prompt queue"},
        {"command": "status", "description": "Active account & proxy gateway status"},
        {"command": "gitmap", "description": "Run GitMap CI/CD pipeline check (pe)"},
        {"command": "backup", "description": "Backup active prompts to split SQLite DB"},
        {"command": "restore", "description": "Restore backed-up prompts to workspace"},
        {"command": "email", "description": "Check email dispatch & remote watcher status"},
        {"command": "ff", "description": "Fast-forward rotate to next highest-quota account"},
        {"command": "snapshot", "description": "Capture live desktop screenshot"},
        {"command": "help", "description": "Display full command cheat-sheet"},
    ]
    try:
        call_telegram_api(token, "setMyCommands", {"commands": commands})
        print("  [+] Registered 10 slash commands in Telegram bot menu.")
    except Exception as e:
        print(f"  [!] Note: Could not register bot menu commands: {e}")


def detect_chat_id(token: str) -> tuple[int, str, str]:
    """Auto-detect chat ID from getUpdates."""
    updates = call_telegram_api(token, "getUpdates", {"offset": -10, "limit": 10})
    if not updates.get("ok"):
        raise RuntimeError(f"getUpdates returned false: {updates}")

    results = updates.get("result", [])
    if not results:
        raise RuntimeError(
            "No recent messages found on bot.\n"
            "Please open your bot in Telegram, send '/start' or '/ping', and rerun this command."
        )

    for update in reversed(results):
        msg = update.get("message") or update.get("channel_post") or update.get("edited_message")
        if msg and "chat" in msg:
            chat = msg["chat"]
            cid = chat["id"]
            user = msg.get("from", {})
            user_name = user.get("username") or user.get("first_name") or str(cid)
            text = msg.get("text", "")
            return cid, user_name, text

    raise RuntimeError("Could not find a valid message object in recent bot updates.")


def main():
    args = sys.argv[1:]

    if not args or args[0] in ("-h", "--help", "help"):
        print("==================================================================")
        print("  Antigravity Manager (AGM) - Telegram Bot Connect Tool")
        print("==================================================================")
        print("Usage:")
        print("  python 03-ai-scripts/42-telegram-connect.py <BOT_TOKEN> [CHAT_ID]")
        print("  python 03-ai-scripts/42-telegram-connect.py --status")
        print("  python 03-ai-scripts/42-telegram-connect.py --help\n")
        print("Workflow:")
        print("  1. Message @BotFather on Telegram -> /newbot to create your bot.")
        print("  2. Open your new bot and send '/ping' or '/start'.")
        print("  3. Run: python 03-ai-scripts/42-telegram-connect.py <YOUR_BOT_TOKEN>")
        print("  (Automatically discovers your Chat ID, saves config, and links AGM!)\n")
        sys.exit(0)

    data_dir = get_agm_data_dir()
    cfg_path = data_dir / "telegram_config.json"

    if args[0] in ("--status", "status", "ls"):
        if not cfg_path.exists():
            print(f"[-] No telegram_config.json found at {cfg_path}")
            sys.exit(0)
        try:
            cfg = json.loads(cfg_path.read_text(encoding="utf-8"))
            print(f"[+] AGM Telegram Configuration ({cfg_path}):")
            print(f"    - Enabled: {cfg.get('is_enabled', False)}")
            raw_token = cfg.get("bot_token", "")
            masked = f"{raw_token[:6]}...{raw_token[-4:]}" if len(raw_token) > 10 else "None"
            print(f"    - Bot Token: {masked}")
            print(f"    - Allowed Chat ID: {cfg.get('allowed_chat_id')}")
            print(f"    - Polling Interval: {cfg.get('poll_interval_secs', 5)}s")
        except Exception as e:
            print(f"[-] Error reading config: {e}")
        sys.exit(0)

    token = args[0].strip()
    chat_id = None

    if len(args) > 1:
        try:
            chat_id = int(args[1].strip())
        except ValueError:
            print(f"[ERROR] Chat ID must be a numeric integer, got: {args[1]}")
            sys.exit(1)

    print(f"[*] Validating Telegram Bot Token with Telegram API...")
    try:
        me = call_telegram_api(token, "getMe")
        if not me.get("ok"):
            print(f"[ERROR] Token validation failed: {me}")
            sys.exit(1)
        bot_user = me.get("result", {})
        bot_name = bot_user.get("first_name", "Bot")
        bot_username = bot_user.get("username", "")
        print(f"[SUCCESS] Bot verified: {bot_name} (@{bot_username})")
    except Exception as e:
        print(f"[ERROR] Failed to verify bot token: {e}")
        sys.exit(1)

    if chat_id is None:
        print("[*] Auto-detecting Chat ID from recent bot messages (/ping or /start)...")
        try:
            detected_id, user_name, last_text = detect_chat_id(token)
            chat_id = detected_id
            print(f"[SUCCESS] Discovered Chat ID: {chat_id} from user @{user_name} (Last message: '{last_text}')")
        except Exception as e:
            print(f"\n[WARN] {e}")
            print("\nPlease open your bot in Telegram, send '/ping' or '/start', and re-run:")
            print(f"  python 03-ai-scripts/42-telegram-connect.py {token}\n")
            sys.exit(1)

    register_bot_commands(token)

    config_data = {
        "bot_token": token,
        "allowed_chat_id": chat_id,
        "is_enabled": True,
        "poll_interval_secs": 5,
        "notify_on_system_update": True,
    }

    try:
        cfg_path.write_text(json.dumps(config_data, indent=2), encoding="utf-8")
        print(f"[SUCCESS] Saved Telegram configuration to: {cfg_path}")
    except Exception as e:
        print(f"[ERROR] Failed to save configuration to {cfg_path}: {e}")
        sys.exit(1)

    welcome_text = (
        f"<b>🚀 Antigravity Manager Connected!</b>\n\n"
        f"Node is linked to Telegram bot <b>@{bot_username}</b>.\n"
        f"<b>Allowed Chat ID:</b> <code>{chat_id}</code>\n"
        f"<b>Status:</b> Ready for telemetry, alert notifications, and remote commands.\n\n"
        f"<i>Type /help or /ping to test interaction.</i>"
    )
    try:
        call_telegram_api(
            token,
            "sendMessage",
            {
                "chat_id": chat_id,
                "text": welcome_text,
                "parse_mode": "HTML",
            },
        )
        print(f"[SUCCESS] Delivered welcome confirmation to Telegram chat {chat_id}!")
    except Exception as e:
        print(f"[WARN] Config saved, but could not deliver greeting message: {e}")

    print("\n==================================================================")
    print("  TELEGRAM BOT CONNECTION COMPLETE & VERIFIED")
    print("==================================================================")
    print(f"  Bot:      @{bot_username}")
    print(f"  Chat ID:  {chat_id}")
    print(f"  Config:   {cfg_path}")
    print("  You can now launch AGM GUI or run: agm telegram status")
    print("==================================================================\n")


if __name__ == "__main__":
    main()
