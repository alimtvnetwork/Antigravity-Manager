<#
.SYNOPSIS
    Antigravity-Manager: Telegram Bot Setup & Verification Utility

.DESCRIPTION
    Validates a Telegram Bot Token via the Telegram API, retrieves bot metadata,
    and sends a verification test ping to the designated Chat ID.

.PARAMETER BotToken
    The Telegram bot token obtained from @BotFather.

.PARAMETER ChatId
    The Telegram chat ID (user or group ID) to receive alerts.

.EXAMPLE
    .\03-ai-scripts\setup-telegram-bot.ps1 -BotToken "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11" -ChatId "987654321"
#>

param(
    [Parameter(Mandatory = $true)]
    [string]$BotToken,

    [Parameter(Mandatory = $true)]
    [string]$ChatId
)

$ErrorActionPreference = "Stop"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "             Antigravity-Manager Telegram Bot Verification                      " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan

# 1. Test Bot Token Validity
Write-Host "`n[*] Verifying Bot Token..." -ForegroundColor Yellow
$getMeUrl = "https://api.telegram.org/bot$BotToken/getMe"

try {
    $meResponse = Invoke-RestMethod -Uri $getMeUrl -Method Get -TimeoutSec 10
    if ($meResponse.ok) {
        $botUser = $meResponse.result.username
        $botName = $meResponse.result.first_name
        Write-Host "  [OK] Bot Authenticated Successfully!" -ForegroundColor Green
        Write-Host "       Name:     $botName" -ForegroundColor Gray
        Write-Host "       Username: @$botUser" -ForegroundColor Gray
    } else {
        Write-Host "  [ERROR] Bot token rejected by Telegram: $($meResponse.description)" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  [ERROR] Failed to connect to Telegram API: $_" -ForegroundColor Red
    exit 1
}

# 2. Send Test Alert Message
Write-Host "`n[*] Sending Test Alert to Chat ID '$ChatId'..." -ForegroundColor Yellow
$sendMessageUrl = "https://api.telegram.org/bot$BotToken/sendMessage"
$text = "🚀 *Antigravity-Manager Alert Test*`n`nBot verification completed successfully!`nNode: " + $env:COMPUTERNAME + "`nTimestamp: " + (Get-Date -Format "yyyy-MM-dd HH:mm:ss")

$body = @{
    chat_id = $ChatId
    text = $text
    parse_mode = "Markdown"
} | ConvertTo-Json

try {
    $sendResponse = Invoke-RestMethod -Uri $sendMessageUrl -Method Post -Body $body -ContentType "application/json" -TimeoutSec 10
    if ($sendResponse.ok) {
        Write-Host "  [OK] Test alert delivered successfully!" -ForegroundColor Green
        Write-Host "`n[SUCCESS] Your Telegram bot is fully configured and ready for AGM alerts." -ForegroundColor Green
    } else {
        Write-Host "  [ERROR] Failed to send message: $($sendResponse.description)" -ForegroundColor Red
        exit 1
    }
} catch {
    Write-Host "  [ERROR] Message delivery failed: $_" -ForegroundColor Red
    Write-Host "  Tip: Ensure you have started a chat with the bot (@$botUser) in Telegram before testing." -ForegroundColor Yellow
    exit 1
}
