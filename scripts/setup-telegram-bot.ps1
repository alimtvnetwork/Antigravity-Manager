<#
.SYNOPSIS
    Automated Setup and Verification Wizard for AGM Telegram Remote Control Daemon.

.DESCRIPTION
    Validates Telegram Bot Token via official Telegram Bot API, detects recent chat IDs,
    and configures telegram_config.json in the AGM data directory.

.PARAMETER BotToken
    The Telegram Bot Token obtained from @BotFather.

.PARAMETER AllowedChatId
    Optional Telegram Chat ID to authorize for administrative commands.

.PARAMETER PollIntervalSecs
    Polling interval in seconds (default: 5).

.PARAMETER EnableNow
    Enables the daemon immediately upon writing the configuration.

.EXAMPLE
    .\scripts\setup-telegram-bot.ps1 -BotToken "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11" -EnableNow
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$BotToken,

    [Parameter(Position = 1)]
    [long]$AllowedChatId = 0,

    [Parameter(Position = 2)]
    [int]$PollIntervalSecs = 5,

    [switch]$EnableNow
)

$ErrorActionPreference = "Stop"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "       AGM Telegram Bot Setup & Verification Wizard              " -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Acquire Bot Token
if ([string]::IsNullOrWhiteSpace($BotToken)) {
    Write-Host "[*] No BotToken provided via CLI argument." -ForegroundColor Yellow
    Write-Host "    If you don't have a bot token, message @BotFather on Telegram to create one."
    $BotToken = Read-Host "Enter Telegram Bot Token"
    $BotToken = $BotToken.Trim()
}

if ([string]::IsNullOrWhiteSpace($BotToken)) {
    Write-Error "Bot token cannot be empty."
    exit 1
}

# 2. Validate Token via Telegram Bot API (getMe)
Write-Host "[*] Validating bot token with Telegram Bot API..." -ForegroundColor Gray
$getMeUrl = "https://api.telegram.org/bot$BotToken/getMe"

try {
    $meResp = Invoke-RestMethod -Uri $getMeUrl -Method Get -TimeoutSec 15
} catch {
    Write-Error "Failed to connect to Telegram Bot API: $_"
    exit 1
}

if (-not $meResp.ok) {
    Write-Error "Telegram Bot API returned error: $($meResp.description)"
    exit 1
}

$botUser = $meResp.result
Write-Host "[+] Bot successfully verified!" -ForegroundColor Green
Write-Host "    Bot Username: @$($botUser.username)" -ForegroundColor White
Write-Host "    Bot Name:     $($botUser.first_name)" -ForegroundColor White
Write-Host "    Bot ID:       $($botUser.id)" -ForegroundColor White
Write-Host ""

# 3. Detect Chat ID if not provided
if ($AllowedChatId -eq 0) {
    Write-Host "[*] Querying recent Telegram updates for authorized Chat ID..." -ForegroundColor Gray
    $updatesUrl = "https://api.telegram.org/bot$BotToken/getUpdates"
    try {
        $updatesResp = Invoke-RestMethod -Uri $updatesUrl -Method Get -TimeoutSec 10
        if ($updatesResp.ok -and $updatesResp.result.Count -gt 0) {
            $recentChats = @{}
            foreach ($up in $updatesResp.result) {
                if ($up.message -and $up.message.chat) {
                    $c = $up.message.chat
                    $chatKey = "$($c.id)"
                    if (-not $recentChats.ContainsKey($chatKey)) {
                        $sender = if ($up.message.from.username) { "@" + $up.message.from.username } else { $up.message.from.first_name }
                        $recentChats[$chatKey] = @{
                            Id = $c.id
                            Type = $c.type
                            Sender = $sender
                        }
                    }
                }
            }

            if ($recentChats.Count -gt 0) {
                Write-Host "[?] Detected the following recent chat senders:" -ForegroundColor Yellow
                $idx = 1
                $chatList = @($recentChats.Values)
                foreach ($item in $chatList) {
                    Write-Host "    [$idx] Chat ID: $($item.Id) | Sender: $($item.Sender) | Type: $($item.Type)" -ForegroundColor White
                    $idx++
                }
                Write-Host "    [0] Leave unconstrained (or enter custom ID manually)" -ForegroundColor Gray
                $choice = Read-Host "Select option (1-$($chatList.Count)) or press Enter to skip"
                if ($choice -match '^\d+$') {
                    $selectedIdx = [int]$choice
                    if ($selectedIdx -ge 1 -and $selectedIdx -le $chatList.Count) {
                        $AllowedChatId = $chatList[$selectedIdx - 1].Id
                        Write-Host "[+] Bound Allowed Chat ID to: $AllowedChatId" -ForegroundColor Green
                    }
                }
            }
        }
    } catch {
        Write-Host "[-] Note: Could not fetch updates (non-fatal): $_" -ForegroundColor DarkGray
    }
}

# 4. Resolve AGM Data Directory
$dataDir = $env:ABV_DATA_DIR
if ([string]::IsNullOrWhiteSpace($dataDir)) {
    $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $dataDir = Join-Path $homeDir ".antigravity_tools"
}

if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
}

$configPath = Join-Path $dataDir "telegram_config.json"

# 5. Build Configuration
$allowedIdValue = if ($AllowedChatId -ne 0) { $AllowedChatId } else { $null }
$configObj = [ordered]@{
    bot_token = $BotToken
    allowed_chat_id = $allowedIdValue
    is_enabled = [bool]$EnableNow.IsPresent
    poll_interval_secs = $PollIntervalSecs
}

$jsonText = $configObj | ConvertTo-Json -Depth 4
[System.IO.File]::WriteAllText($configPath, $jsonText, [System.Text.Encoding]::UTF8)

Write-Host ""
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "[+] Telegram Bot configuration saved successfully!" -ForegroundColor Green
Write-Host "    Config Location: $configPath" -ForegroundColor White
Write-Host "    Bot:             @$($botUser.username)" -ForegroundColor White
Write-Host "    Allowed Chat ID: $(if ($allowedIdValue) { $allowedIdValue } else { 'Any (Open)' })" -ForegroundColor White
Write-Host "    Enabled:         $($configObj.is_enabled)" -ForegroundColor White
Write-Host "    Poll Interval:   $PollIntervalSecs s" -ForegroundColor White
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "1. In Telegram, start a chat with @$($botUser.username)"
Write-Host "2. Send 'SNAPSHOT' or '/start' to inspect the Antigravity cluster nodes."
Write-Host "3. Send 'FF' to trigger fast-forward account rotation."
Write-Host "4. Send 'CMD:<node-alias>:<command>' to dispatch remote PowerShell/Bash instructions."
Write-Host ""
