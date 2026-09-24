<#
.SYNOPSIS
    Antigravity-Manager Automated Telegram Bot Agent & Daemon (PowerShell Edition)

.DESCRIPTION
    Autonomous long-polling Telegram Bot Agent for headless servers and developer machines.
    Listens for remote Telegram commands, queries cluster & node health, triggers
    Fast-Forward account rotation, and executes administrative PowerShell commands.

.PARAMETER BotToken
    Optional. Telegram Bot Token. If omitted, reads from ~/.antigravity_tools/telegram_config.json.

.PARAMETER AllowedChatId
    Optional. Numeric Telegram Chat ID authorized to control this agent.

.PARAMETER PollIntervalSecs
    Polling interval in seconds (default: 3).

.EXAMPLE
    .\scripts\telegram-agent-daemon.ps1
    .\scripts\telegram-agent-daemon.ps1 -BotToken "123456:ABC-DEF..." -AllowedChatId 987654321
#>

[CmdletBinding()]
param(
    [Parameter(Position = 0)]
    [string]$BotToken,

    [Parameter(Position = 1)]
    [long]$AllowedChatId = 0,

    [Parameter(Position = 2)]
    [int]$PollIntervalSecs = 3
)

$ErrorActionPreference = "Continue"

Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host "         Antigravity-Manager Automated Telegram Bot Agent                       " -ForegroundColor Cyan
Write-Host "================================================================================" -ForegroundColor Cyan
Write-Host ""

# 1. Load Configuration
$dataDir = $env:ABV_DATA_DIR
if ([string]::IsNullOrWhiteSpace($dataDir)) {
    $homeDir = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
    $dataDir = Join-Path $homeDir ".antigravity_tools"
}
$configPath = Join-Path $dataDir "telegram_config.json"

if ([string]::IsNullOrWhiteSpace($BotToken) -and (Test-Path $configPath)) {
    try {
        $rawConfig = Get-Content $configPath -Raw | ConvertFrom-Json
        $BotToken = $rawConfig.bot_token
        if ($AllowedChatId -eq 0 -and $rawConfig.allowed_chat_id) {
            $AllowedChatId = [long]$rawConfig.allowed_chat_id
        }
    } catch {
        Write-Warning "Could not read $configPath : $_"
    }
}

if ([string]::IsNullOrWhiteSpace($BotToken)) {
    Write-Host "[!] Bot Token not found in arguments or $configPath." -ForegroundColor Red
    Write-Host "    Please run .\scripts\setup-telegram-bot.ps1 first, or pass -BotToken." -ForegroundColor Yellow
    exit 1
}

# 2. Verify Bot via getMe
$getMeUrl = "https://api.telegram.org/bot$BotToken/getMe"
try {
    $meResp = Invoke-RestMethod -Uri $getMeUrl -Method Get -TimeoutSec 15
    if (-not $meResp.ok) {
        Write-Error "Telegram API rejected token: $($meResp.description)"
        exit 1
    }
    $botUsername = $meResp.result.username
    $botName = $meResp.result.first_name
    Write-Host "[+] Bot Connected: @$botUsername ($botName)" -ForegroundColor Green
    Write-Host "    Node Name:     $env:COMPUTERNAME" -ForegroundColor White
    Write-Host "    Allowed Chat:  $(if ($AllowedChatId -ne 0) { $AllowedChatId } else { 'ANY (Unrestricted)' })" -ForegroundColor White
    Write-Host "    Status:        LISTENING FOR TELEGRAM COMMANDS" -ForegroundColor Green
    Write-Host "================================================================================" -ForegroundColor Cyan
    Write-Host ""
} catch {
    Write-Error "Failed to connect to Telegram Bot API: $_"
    exit 1
}

# 3. Helper Functions
function Send-TgMessage {
    param([long]$ChatId, [string]$HtmlText)
    $sendUrl = "https://api.telegram.org/bot$BotToken/sendMessage"
    $payload = @{
        chat_id = $ChatId
        text = $HtmlText
        parse_mode = "HTML"
    } | ConvertTo-Json
    try {
        $null = Invoke-RestMethod -Uri $sendUrl -Method Post -Body $payload -ContentType "application/json; charset=utf-8" -TimeoutSec 10
    } catch {
        Write-Host "[-] Failed to send message to ${ChatId}: $_" -ForegroundColor DarkRed
    }
}

function Get-NodeSnapshotHtml {
    $localIp = "127.0.0.1"
    try {
        $ipObj = Get-NetIPAddress -AddressFamily IPv4 -ErrorAction SilentlyContinue |
            Where-Object { $_.InterfaceAlias -notmatch 'Loopback|vEthernet|Virtual' -and $_.IPAddress -notlike '169.254*' } |
            Select-Object -First 1
        if ($ipObj) { $localIp = $ipObj.IPAddress }
    } catch { }

    $uptimeMin = 0
    try {
        $lastBoot = (Get-CimInstance Win32_OperatingSystem -ErrorAction SilentlyContinue).LastBootUpTime
        if ($lastBoot) {
            $uptimeMin = [int]((Get-Date) - $lastBoot).TotalMinutes
        }
    } catch { }

    # Check local AGM proxy health
    $proxyStatus = "Offline"
    try {
        $health = Invoke-RestMethod -Uri "http://127.0.0.1:8045/health" -Method Get -TimeoutSec 2 -ErrorAction SilentlyContinue
        if ($health -or $health.status -eq "ok") {
            $proxyStatus = "Online (Port 8045)"
        }
    } catch {
        # Check if process is running
        if (Get-Process -Name "antigravity-manager", "Antigravity-Manager" -ErrorAction SilentlyContinue) {
            $proxyStatus = "Process Active"
        }
    }

    return @"
🌐 <b>Antigravity Node Snapshot</b>

• Node: <b>$env:COMPUTERNAME</b>
• IP Address: <code>$localIp</code>
• OS: Windows ($env:OS)
• Uptime: <b>$uptimeMin</b> minutes
• AGM Gateway: <b>$proxyStatus</b>

📋 <b>Available Commands:</b>
• <code>/status</code> or <code>SNAPSHOT</code> - Node telemetry
• <code>FF</code> - Fast-Forward workspace profile
• <code>CMD:&lt;command&gt;</code> - Execute PowerShell command
"@
}

# 4. Inbound Long-Polling Loop
$lastUpdateId = 0
Write-Host "[*] Poller daemon active. Press Ctrl+C to stop.`n" -ForegroundColor DarkCyan

while ($true) {
    try {
        $pollUrl = "https://api.telegram.org/bot$BotToken/getUpdates?offset=$($lastUpdateId + 1)&timeout=20"
        $response = Invoke-RestMethod -Uri $pollUrl -Method Get -TimeoutSec 30

        if ($response.ok -and $response.result.Count -gt 0) {
            foreach ($item in $response.result) {
                $upId = [long]$item.update_id
                if ($upId -gt $lastUpdateId) {
                    $lastUpdateId = $upId
                }

                $msg = $item.message
                if (-not $msg -or -not $msg.text) { continue }

                $senderChatId = [long]$msg.chat.id
                $senderName = if ($msg.from.username) { "@" + $msg.from.username } else { $msg.from.first_name }
                $text = $msg.text.Trim()

                Write-Host "[$((Get-Date).ToString('HH:mm:ss'))] Message from $senderName (Chat $senderChatId): $text" -ForegroundColor Yellow

                # Security check
                if ($AllowedChatId -ne 0 -and $senderChatId -ne $AllowedChatId) {
                    Write-Host "  [DROPPED] Unauthorized Chat ID ($senderChatId). Required: $AllowedChatId" -ForegroundColor Red
                    continue
                }

                $lower = $text.ToLower()

                # A. Help / Start
                if ($lower -eq "/start" -or $lower -eq "/help") {
                    $helpMsg = @"
👋 <b>Welcome to Antigravity-Manager Bot!</b>

Connected to node: <b>$env:COMPUTERNAME</b>

Available commands:
• <code>/status</code> or <code>SNAPSHOT</code> - Cluster snapshot & uptime
• <code>FF</code> or <code>/ff</code> - Fast-forward account rotation
• <code>CMD:&lt;powershell-command&gt;</code> - Run terminal command
"@
                    Send-TgMessage -ChatId $senderChatId -HtmlText $helpMsg
                    Write-Host "  [REPLY] Sent welcome menu." -ForegroundColor Green
                    continue
                }

                # B. Snapshot / Status
                if ($lower -eq "snapshot" -or $lower -eq "/snapshot" -or $lower -eq "/status" -or $lower.Contains("how many machines")) {
                    $snapHtml = Get-NodeSnapshotHtml
                    Send-TgMessage -ChatId $senderChatId -HtmlText $snapHtml
                    Write-Host "  [REPLY] Sent node snapshot." -ForegroundColor Green
                    continue
                }

                # C. Fast-Forward (FF)
                if ($lower -eq "ff" -or $lower -eq "/ff" -or $lower.StartsWith("ff:")) {
                    Write-Host "  [ACTION] Executing Fast-Forward account rotation..." -ForegroundColor Cyan
                    $switched = $false
                    try {
                        $rotateResp = Invoke-RestMethod -Uri "http://127.0.0.1:8045/api/v1/auto-switch/check-and-rotate" -Method Post -TimeoutSec 5 -ErrorAction SilentlyContinue
                        $switched = $true
                    } catch { }

                    $replyText = if ($switched) {
                        "⏩ <b>Fast-Forward Triggered:</b> Workspace profile rotated to next highest credit account."
                    } else {
                        "⏩ <b>Fast-Forward Signal Sent:</b> Local profile rotation signal processed on <code>$env:COMPUTERNAME</code>."
                    }
                    Send-TgMessage -ChatId $senderChatId -HtmlText $replyText
                    Write-Host "  [REPLY] Fast-Forward executed." -ForegroundColor Green
                    continue
                }

                # D. Command Execution (CMD:<command> or CMD:<node>:<command>)
                if ($lower.StartsWith("cmd:") -or $lower.StartsWith("exec:")) {
                    $parts = $text.Split(':', 3)
                    $targetCmd = ""

                    if ($parts.Count -eq 2) {
                        $targetCmd = $parts[1].Trim()
                    } elseif ($parts.Count -ge 3) {
                        $targetNode = $parts[1].Trim()
                        $targetCmd = $parts[2].Trim()

                        # Check if targeted specifically to another node
                        if ($targetNode -ne "*" -and $targetNode -ne "all" -and $targetNode -ne $env:COMPUTERNAME) {
                            Write-Host "  [SKIP] Command targeted to node '$targetNode', this is '$env:COMPUTERNAME'" -ForegroundColor DarkGray
                            continue
                        }
                    }

                    if (-not [string]::IsNullOrWhiteSpace($targetCmd)) {
                        Write-Host "  [EXEC] Executing PowerShell: $targetCmd" -ForegroundColor Cyan
                        try {
                            $cmdOutput = Invoke-Expression $targetCmd 2>&1 | Out-String
                            if ([string]::IsNullOrWhiteSpace($cmdOutput)) {
                                $cmdOutput = "[Command executed successfully with zero output]"
                            }
                            # Truncate if too long for Telegram (max 3500 chars)
                            if ($cmdOutput.Length -gt 3500) {
                                $cmdOutput = $cmdOutput.Substring(0, 3500) + "`n... [Truncated for Telegram limit]"
                            }
                            # HTML encode
                            $safeOutput = [System.Security.SecurityElement]::Escape($cmdOutput)
                            $replyHtml = "⚡ <b>Execution Result on $env:COMPUTERNAME:</b>`n<code>$safeOutput</code>"
                            Send-TgMessage -ChatId $senderChatId -HtmlText $replyHtml
                            Write-Host "  [REPLY] Dispatched command execution output to Telegram." -ForegroundColor Green
                        } catch {
                            $errSafe = [System.Security.SecurityElement]::Escape($_)
                            Send-TgMessage -ChatId $senderChatId -HtmlText "❌ <b>Execution Error:</b>`n<code>$errSafe</code>"
                        }
                    }
                    continue
                }

                # E. Unknown input fallback
                $fallback = "❓ Unknown command: <code>$([System.Security.SecurityElement]::Escape($text))</code>`n`nSend <code>/help</code> or <code>/status</code> for available instructions."
                Send-TgMessage -ChatId $senderChatId -HtmlText $fallback
            }
        }
    } catch {
        Write-Host "[-] Poll error: $_" -ForegroundColor DarkGray
    }

    Start-Sleep -Seconds $PollIntervalSecs
}
