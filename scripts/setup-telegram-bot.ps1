<#
.SYNOPSIS
    Automated Setup and Verification Wizard for AGM Telegram Remote Control Daemon.

.DESCRIPTION
    Validates Telegram Bot Token via official Telegram Bot API, detects or listens
    for recent chat IDs, sends a live test notification to your Telegram client,
    and configures telegram_config.json in the AGM data directory.

.PARAMETER BotToken
    The Telegram Bot Token obtained from @BotFather.

.PARAMETER AllowedChatId
    Optional Telegram Chat ID to authorize for administrative commands.

.PARAMETER PollIntervalSecs
    Polling interval in seconds (default: 5).

.PARAMETER EnableNow
    Enables the daemon immediately upon writing the configuration.

.PARAMETER SendTest
    Dispatches a verification test ping to Telegram upon configuration (default: true).

.PARAMETER RunAgent
    Immediately starts the automated Telegram agent listener in PowerShell after setup.

.EXAMPLE
    .\scripts\setup-telegram-bot.ps1
    .\scripts\setup-telegram-bot.ps1 -BotToken "123456:ABC-DEF1234ghIkl-zyx57W2v1u123ew11" -EnableNow
    .\scripts\setup-telegram-bot.ps1 -RunAgent
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$BotToken,

    [Parameter(Position = 1)]
    [long]$AllowedChatId = 0,

    [Parameter(Position = 2)]
    [int]$PollIntervalSecs = 5,

    [switch]$EnableNow,

    [switch]$SendTest = $true,

    [switch]$RunAgent
)

$ErrorActionPreference = "Stop"

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "       AGM Telegram Bot Setup & Verification Wizard              " -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""

# Resolve AGM Data Directory & Existing Config
$dataDir = $env:ABV_DATA_DIR
if ([string]::IsNullOrWhiteSpace($dataDir)) {
    $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $dataDir = Join-Path $homeDir ".antigravity_tools"
}
$configPath = Join-Path $dataDir "telegram_config.json"

$existingConfig = $null
if (Test-Path $configPath) {
    try {
        $raw = Get-Content $configPath -Raw -ErrorAction SilentlyContinue
        $existingConfig = $raw | ConvertFrom-Json -ErrorAction SilentlyContinue
    } catch { }
}

# 1. Acquire Bot Token
if ([string]::IsNullOrWhiteSpace($BotToken)) {
    if ($existingConfig -and -not [string]::IsNullOrWhiteSpace($existingConfig.bot_token)) {
        Write-Host "[*] Found existing Bot Token in config: $($existingConfig.bot_token.Substring(0, [Math]::Min(12, $existingConfig.bot_token.Length)))..." -ForegroundColor Yellow
        $useExisting = Read-Host "Use existing saved Bot Token? (Y/n)"
        if ($useExisting -match '^[Yy]?$') {
            $BotToken = $existingConfig.bot_token
        }
    }
}

if ([string]::IsNullOrWhiteSpace($BotToken)) {
    Write-Host "[*] No Bot Token provided." -ForegroundColor Yellow
    Write-Host "    To get a token: Open Telegram -> Search @BotFather -> Send '/newbot'" -ForegroundColor Gray
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
    Write-Host "[ERROR] Failed to connect to Telegram Bot API: $_" -ForegroundColor Red
    Write-Host "        Token provided: '$BotToken'" -ForegroundColor Yellow
    Write-Host "        Tip: Ensure you copied the full HTTP API token from @BotFather." -ForegroundColor Yellow
    exit 1
}

if (-not $meResp.ok) {
    Write-Host "[ERROR] Telegram Bot API returned error: $($meResp.description)" -ForegroundColor Red
    exit 1
}

$botUser = $meResp.result
Write-Host "[+] Bot successfully verified!" -ForegroundColor Green
Write-Host "    Bot Username: @$($botUser.username)" -ForegroundColor White
Write-Host "    Bot Name:     $($botUser.first_name)" -ForegroundColor White
Write-Host "    Bot ID:       $($botUser.id)" -ForegroundColor White
Write-Host ""

# 3. Detect Chat ID if not provided
if ($AllowedChatId -eq 0 -and $existingConfig -and $existingConfig.allowed_chat_id) {
    $AllowedChatId = [long]$existingConfig.allowed_chat_id
    Write-Host "[*] Using previously saved Allowed Chat ID: $AllowedChatId" -ForegroundColor Yellow
}

if ($AllowedChatId -eq 0) {
    Write-Host "[*] Querying recent Telegram updates to detect your Chat ID..." -ForegroundColor Gray
    $updatesUrl = "https://api.telegram.org/bot$BotToken/getUpdates"
    $foundChats = @{}

    try {
        $updatesResp = Invoke-RestMethod -Uri $updatesUrl -Method Get -TimeoutSec 10
        if ($updatesResp.ok -and $updatesResp.result.Count -gt 0) {
            foreach ($up in $updatesResp.result) {
                if ($up.message -and $up.message.chat) {
                    $c = $up.message.chat
                    $chatKey = "$($c.id)"
                    if (-not $foundChats.ContainsKey($chatKey)) {
                        $sender = if ($up.message.from.username) { "@" + $up.message.from.username } else { $up.message.from.first_name }
                        $foundChats[$chatKey] = @{
                            Id = $c.id
                            Type = $c.type
                            Sender = $sender
                        }
                    }
                }
            }
        }
    } catch {
        Write-Host "[-] Note: Could not query getUpdates: $_" -ForegroundColor DarkGray
    }

    if ($foundChats.Count -gt 0) {
        Write-Host "[?] Detected the following recent chat senders:" -ForegroundColor Yellow
        $idx = 1
        $chatList = @($foundChats.Values)
        foreach ($item in $chatList) {
            Write-Host "    [$idx] Chat ID: $($item.Id) | Sender: $($item.Sender) | Type: $($item.Type)" -ForegroundColor White
            $idx++
        }
        Write-Host "    [M] Enter custom numeric Chat ID manually" -ForegroundColor Gray
        Write-Host "    [0] Leave unconstrained (Any user with bot handle can send commands)" -ForegroundColor Gray
        $choice = Read-Host "Select option (1-$($chatList.Count), M, or 0) [Default: 1]"
        if ([string]::IsNullOrWhiteSpace($choice)) { $choice = "1" }

        if ($choice -match '^\d+$') {
            $selectedIdx = [int]$choice
            if ($selectedIdx -ge 1 -and $selectedIdx -le $chatList.Count) {
                $AllowedChatId = $chatList[$selectedIdx - 1].Id
                Write-Host "[+] Bound Allowed Chat ID to: $AllowedChatId" -ForegroundColor Green
            }
        } elseif ($choice -match '^[Mm]$') {
            $manualInput = Read-Host "Enter numeric Telegram Chat ID"
            if ($manualInput -match '^\d+$') {
                $AllowedChatId = [long]$manualInput
                Write-Host "[+] Set Allowed Chat ID to: $AllowedChatId" -ForegroundColor Green
            }
        }
    } else {
        Write-Host "[!] No incoming messages found yet for @$($botUser.username)." -ForegroundColor Yellow
        Write-Host "    Action required:" -ForegroundColor Gray
        Write-Host "    1. Open Telegram on your phone or PC." -ForegroundColor Gray
        Write-Host "    2. Search for: @$($botUser.username)" -ForegroundColor Gray
        Write-Host "    3. Click 'START' or send any message (e.g. 'hello')." -ForegroundColor Gray
        Write-Host ""
        $listenChoice = Read-Host "Wait and listen for your message now? (Y/n/manual)"
        if ($listenChoice -match '^[Yy]?$') {
            Write-Host "[*] Listening for incoming messages from @$($botUser.username) (up to 30s)..." -ForegroundColor Cyan
            $stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
            while ($stopwatch.ElapsedMilliseconds -lt 30000 -and $AllowedChatId -eq 0) {
                Write-Host "." -NoNewline -ForegroundColor Gray
                Start-Sleep -Seconds 2
                try {
                    $liveResp = Invoke-RestMethod -Uri $updatesUrl -Method Get -TimeoutSec 5
                    if ($liveResp.ok -and $liveResp.result.Count -gt 0) {
                        $lastMsg = $liveResp.result[-1].message
                        if ($lastMsg -and $lastMsg.chat) {
                            $AllowedChatId = [long]$lastMsg.chat.id
                            $senderName = if ($lastMsg.from.username) { "@" + $lastMsg.from.username } else { $lastMsg.from.first_name }
                            Write-Host ""
                            Write-Host "[+] Successfully detected incoming chat from $senderName (Chat ID: $AllowedChatId)!" -ForegroundColor Green
                            break
                        }
                    }
                } catch { }
            }
            Write-Host ""
        } elseif ($listenChoice -match 'manual') {
            $manualInput = Read-Host "Enter numeric Telegram Chat ID"
            if ($manualInput -match '^\d+$') {
                $AllowedChatId = [long]$manualInput
            }
        }
    }
}

# 4. Send Verification Test Message
if ($SendTest -and $AllowedChatId -ne 0) {
    Write-Host "[*] Dispatching test notification ping to Chat ID $AllowedChatId..." -ForegroundColor Gray
    $sendMessageUrl = "https://api.telegram.org/bot$BotToken/sendMessage"
    $nodeName = $env:COMPUTERNAME
    $timestamp = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
    $testText = "<b>Antigravity-Manager Alert Test</b>`n`nBot verification completed successfully!`nNode: <code>$nodeName</code>`nTimestamp: <code>$timestamp</code>`n`n<b>Commands available:</b>`n- <code>/status</code> or <code>SNAPSHOT</code> (Cluster status)`n- <code>FF</code> (Fast-Forward profile rotation)`n- <code>CMD:&lt;node-alias&gt;:&lt;command&gt;</code> (Execute remote instruction)"

    $testPayload = @{
        chat_id = $AllowedChatId
        text = $testText
        parse_mode = "HTML"
    } | ConvertTo-Json

    try {
        $sendResp = Invoke-RestMethod -Uri $sendMessageUrl -Method Post -Body $testPayload -ContentType "application/json; charset=utf-8" -TimeoutSec 10
        if ($sendResp.ok) {
            Write-Host "[+] Test notification ping delivered to your Telegram successfully!" -ForegroundColor Green
        }
    } catch {
        Write-Host "[-] Note: Test message could not be delivered: $_" -ForegroundColor Yellow
        Write-Host "    Make sure you have started a chat with @$($botUser.username) in Telegram." -ForegroundColor DarkGray
    }
}

# 5. Persist Configuration
if (-not (Test-Path $dataDir)) {
    New-Item -ItemType Directory -Path $dataDir -Force | Out-Null
}

$allowedIdValue = if ($AllowedChatId -ne 0) { $AllowedChatId } else { $null }
$isDaemonEnabled = if ($EnableNow.IsPresent) { $true } else { if ($existingConfig) { [bool]$existingConfig.is_enabled } else { $true } }

$configObj = [ordered]@{
    bot_token = $BotToken
    allowed_chat_id = $allowedIdValue
    is_enabled = $isDaemonEnabled
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
Write-Host "    Daemon Enabled:  $($configObj.is_enabled)" -ForegroundColor White
Write-Host "    Poll Interval:   $PollIntervalSecs s" -ForegroundColor White
Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host ""
Write-Host "Next Steps:" -ForegroundColor Yellow
Write-Host "1. In Telegram, start a chat with @$($botUser.username)"
Write-Host "2. Send 'SNAPSHOT' or '/start' to inspect the Antigravity cluster nodes."
Write-Host "3. Send 'FF' to trigger fast-forward account rotation."
Write-Host "4. Send 'CMD:<node-alias>:<command>' to dispatch remote PowerShell instructions."
Write-Host ""

# 6. Run Agent Daemon if requested
if ($RunAgent.IsPresent) {
    $daemonScript = Join-Path $PSScriptRoot "telegram-agent-daemon.ps1"
    if (Test-Path $daemonScript) {
        Write-Host "[*] Launching Telegram Agent Daemon..." -ForegroundColor Cyan
        & $daemonScript
    } else {
        Write-Host "[!] Telegram agent daemon script not found at $daemonScript" -ForegroundColor Yellow
    }
}
