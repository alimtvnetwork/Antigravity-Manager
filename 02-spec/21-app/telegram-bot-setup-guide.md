# Telegram Bot Notification Setup Guide

This guide describes how to configure and deploy a Telegram Bot for Antigravity-Manager remote alerts, status queries, and command execution.

## 1. Creating a Telegram Bot via @BotFather

1. Open Telegram and search for [`@BotFather`](https://t.me/BotFather).
2. Start a chat and send the command:
   ```text
   /newbot
   ```
3. Follow the prompts to provide:
   - **Bot Display Name**: e.g. `My AGM Alerts Bot`
   - **Bot Username**: Must end with `bot`, e.g. `my_agm_alerts_bot`
4. BotFather will provide an **HTTP API Token**:
   ```text
   123456789:ABCdefGhIJKlmNoPQRsTUVwxyZ
   ```
   Save this token securely.

---

## 2. Obtaining Your Telegram Chat ID

1. Search for [`@userinfobot`](https://t.me/userinfobot) or [`@getmyid_bot`](https://t.me/getmyid_bot) in Telegram.
2. Send `/start`. The bot will respond with your **Id** (a 9-10 digit number like `987654321`).
3. Send a message directly to your newly created bot so it is authorized to message you.

---

## 3. Configuring Telegram in Antigravity-Manager

1. In Antigravity-Manager, navigate to **Settings** -> **Email & Alerts** -> **Telegram Bot & Alerts**.
2. Enter:
   - **Bot Token**: Your API token from @BotFather.
   - **Chat ID**: Your numeric user or group Chat ID.
   - Toggle **Enable Telegram Notifications** to active.
3. Click **Test Bot Connection** to verify API communication.
4. Click **Send Test Alert** to confirm delivery to your Telegram client.

---

## 4. Automated Setup with PowerShell Script

An automated PowerShell script is provided at `03-ai-scripts/setup-telegram-bot.ps1` to test credentials directly from the command line:

```powershell
.\03-ai-scripts\setup-telegram-bot.ps1 -BotToken "YOUR_BOT_TOKEN" -ChatId "YOUR_CHAT_ID"
```

The script verifies token validity with `https://api.telegram.org/bot<token>/getMe`, sends a confirmation test message, and outputs status details.
