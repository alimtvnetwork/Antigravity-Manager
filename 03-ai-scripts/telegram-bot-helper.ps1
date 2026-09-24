<#
.SYNOPSIS
    Antigravity-Manager Telegram Bot Setup & Interactive Verification Script
.DESCRIPTION
    Step-by-step helper to verify your Telegram Bot Token, automatically detect your
    numeric Allowed Chat ID from recent messages, and send an interactive plaintext
    test notification from PowerShell.
.EXAMPLE
    .\03-ai-scripts\telegram-bot-helper.ps1 -BotToken "123456:ABC-DEF..."
    .\03-ai-scripts\telegram-bot-helper.ps1 -BotToken "123456:ABC-DEF..." -ChatId "987654321"
#>

param(
    [Parameter(Mandatory = $false)]
    [string]$BotToken = $env:AGM_TELEGRAM_BOT_TOKEN,

    [Parameter(Mandatory = $false)]
    [string]$ChatId = $env:AGM_TELEGRAM_CHAT_ID,

    [Parameter(Mandatory = $false)]
    [string]$TestMessage = "AGM Interactive Telegram Verification | Node: $env:COMPUTERNAME | Status: Ready"
)

$ErrorActionPreference = "Stop"

Write-Host "============================================================" -ForegroundColor Cyan
Write-Host " Antigravity-Manager Telegram Bot Step-by-Step Helper       " -ForegroundColor Cyan
Write-Host "============================================================" -ForegroundColor Cyan

if ([string]::IsNullOrWhiteSpace($BotToken)) {
    Write-Host "`n[Step 1] How to get a Telegram Bot Token:" -ForegroundColor Yellow
    Write-Host "  1. Open Telegram and message @BotFather"
    Write-Host "  2. Send '/newbot' and follow the prompts to name your bot"
    Write-Host "  3. Copy the HTTP API Token (e.g. 123456789:ABCdefGHIjklMNO...)"
    Write-Host "  4. Re-run this script with: .\03-ai-scripts\telegram-bot-helper.ps1 -BotToken '<YOUR_TOKEN>'"
    exit 0
}

Write-Host "`n[Step 1] Verifying Bot Token via Telegram getMe API..." -ForegroundColor Cyan
$getMeUrl = "https://api.telegram.org/bot$BotToken/getMe"
try {
    $meResp = Invoke-RestMethod -Uri $getMeUrl -Method Get -TimeoutSec 15
    if (-not $meResp.ok) {
        Write-Error "Telegram getMe returned not ok."
    }
    Write-Host "  [OK] Connected to Bot: @$($meResp.result.username) (ID: $($meResp.result.id))" -ForegroundColor Green
} catch {
    Write-Host "  [ERROR] Invalid Bot Token or network unreachable: $_" -ForegroundColor Red
    exit 1
}

if ([string]::IsNullOrWhiteSpace($ChatId)) {
    Write-Host "`n[Step 2] Discovering your Numeric Allowed Chat ID via getUpdates..." -ForegroundColor Cyan
    $updatesUrl = "https://api.telegram.org/bot$BotToken/getUpdates"
    try {
        $updatesResp = Invoke-RestMethod -Uri $updatesUrl -Method Get -TimeoutSec 15
        $chats = @()
        foreach ($u in $updatesResp.result) {
            if ($null -ne $u.message -and $null -ne $u.message.chat) {
                $chats += $u.message.chat
            }
        }
        if ($chats.Count -eq 0) {
            Write-Host "  [ACTION NEEDED] No messages found yet!" -ForegroundColor Yellow
            Write-Host "  Please open Telegram, search for @$($meResp.result.username), click START or send 'hello', and run this script again."
            exit 0
        }
        $latestChat = $chats[-1]
        $ChatId = [string]$latestChat.id
        Write-Host "  [OK] Discovered Chat ID: $ChatId (User: $($latestChat.username) / $($latestChat.first_name))" -ForegroundColor Green
    } catch {
        Write-Host "  [ERROR] Failed to query getUpdates: $_" -ForegroundColor Red
        exit 1
    }
}

Write-Host "`n[Step 3] Sending Test Notification to Chat ID $ChatId..." -ForegroundColor Cyan
$sendUrl = "https://api.telegram.org/bot$BotToken/sendMessage"
$payload = @{
    chat_id = $ChatId
    text    = $TestMessage
} | ConvertTo-Json

try {
    $sendResp = Invoke-RestMethod -Uri $sendUrl -Method Post -ContentType "application/json; charset=utf-8" -Body $payload -TimeoutSec 15
    if ($sendResp.ok) {
        Write-Host "  [SUCCESS] Telegram message delivered to Chat ID $ChatId!" -ForegroundColor Green
        Write-Host "  Paste these values into Antigravity-Manager -> Email / Notifications -> Telegram Bot Integration:" -ForegroundColor Cyan
        Write-Host "    Telegram Bot Token        : $BotToken"
        Write-Host "    Allowed Chat ID (Numeric) : $ChatId"
    }
} catch {
    Write-Host "  [ERROR] Failed to send Telegram message: $_" -ForegroundColor Red
    exit 1
}
