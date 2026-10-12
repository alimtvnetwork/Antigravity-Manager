<#
.SYNOPSIS
  Comprehensive Verification Suite for Supabase Connection & UI/UX Enhancement
  Verifies progress bar green #1af18d, 5-slot grid, layout compaction, theme contrast,
  Prompt Tree dropdown/loading states, and dual Supabase cluster configuration.

.EXAMPLE
  powershell -ExecutionPolicy Bypass -File scripts/verify-supabase-and-ui-ux.ps1
#>

$ErrorActionPreference = "Stop"
$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$RootDir = Split-Path -Parent $ScriptDir
Set-Location $RootDir

Write-Host "=================================================================" -ForegroundColor Cyan
Write-Host "   Antigravity-Manager: Supabase & UI/UX Enhancement Verification" -ForegroundColor Cyan
Write-Host "=================================================================" -ForegroundColor Cyan

$Passed = 0
$Failed = 0

function Assert-Test {
    param (
        [string]$Name,
        [scriptblock]$Test
    )
    try {
        $result = & $Test
        if ($result -eq $true) {
            Write-Host "  [PASS] $Name" -ForegroundColor Green
            $script:Passed++
        } else {
            Write-Host "  [FAIL] $Name" -ForegroundColor Red
            $script:Failed++
        }
    } catch {
        Write-Host "  [FAIL] $Name (Exception: $_)" -ForegroundColor Red
        $script:Failed++
    }
}

Write-Host "`n1. Verifying Quota Progress Bar Tokens & Styles (#1af18d, 5 Slots, Blur Glow)..." -ForegroundColor Yellow

Assert-Test "Design token --ui-success set to #1af18d" {
    $content = Get-Content "src/styles/ui-tokens.css" -Raw
    $content -match "--ui-success:\s*#1af18d;" -and $content -match "--ui-success-glow:\s*rgb\(26 241 141"
}

Assert-Test "QuotaProgressBar color mapping includes #1af18d" {
    $content = Get-Content "src/components/accounts/QuotaProgressBar.tsx" -Raw
    $content -match "100:\s*'#1af18d'" -and $content -match "80:\s*'#1af18d'"
}

Assert-Test "QuotaProgressBar default checkpoints configured to exactly 5 slots [100, 80, 60, 40, 20]" {
    $content = Get-Content "src/components/accounts/QuotaProgressBar.tsx" -Raw
    $content -match "\[100,\s*80,\s*60,\s*40,\s*20\]"
}

Assert-Test "QuotaProgressBar removed legacy checkmark SVG path bolts" {
    $content = Get-Content "src/components/accounts/QuotaProgressBar.tsx" -Raw
    -not ($content -match "M2\.5 6\.5L4\.8 8\.8L9\.5 3\.5")
}

Assert-Test "QuotaProgressBar vibrant blur glow active at >= 95% threshold" {
    $content = Get-Content "src/components/accounts/QuotaProgressBar.tsx" -Raw
    $content -match "shadow-\[0_0_12px_rgba\(26,241,141,0\.8\)\]"
}

Write-Host "`n2. Verifying Compact Layout & Right-Hand Compacting..." -ForegroundColor Yellow

Assert-Test "QuotaProgressBar track uses flex-1 min-w-0 for fluid compaction" {
    $content = Get-Content "src/components/accounts/QuotaProgressBar.tsx" -Raw
    $content -match "flex-1 min-w-0" -and $content -match "shrink-0 ml-2"
}

Assert-Test "AccountRowContent aligns quota progress bars to the right-hand side" {
    $content = Get-Content "src/components/accounts/AccountTable/AccountRowContent.tsx" -Raw
    $content -match "flex justify-end w-full"
}

Assert-Test "InstanceRowCells aligns 4H and Weekly quota bars to the right-hand side" {
    $content = Get-Content "src/components/instances/table/InstanceRowCells.tsx" -Raw
    $content -match "flex justify-end w-full"
}

Write-Host "`n3. Verifying Theming Contrast, Email Privacy & CSS3 Animations..." -ForegroundColor Yellow

Assert-Test "InstanceCard resolved purple-on-purple defect with high-contrast text" {
    $content = Get-Content "src/pages/instances/InstanceCard.tsx" -Raw
    $content -match "dark:text-purple-100" -or $content -match "dark:text-\[var\(--ui-focus-ring\)\]"
}

Assert-Test "InstanceCard uses maskEmail with interactive hover/click reveal toggle" {
    $content = Get-Content "src/pages/instances/InstanceCard.tsx" -Raw
    $content -match "maskEmail\(displayEmail\)" -and ($content -match "isEmailRevealed" -or $content -match "isRevealed")
}

Assert-Test "InstanceCard has CSS3 hover elevation transitions" {
    $content = Get-Content "src/pages/instances/InstanceCard.tsx" -Raw
    $content -match "hover:-translate-y-0\.5" -and $content -match "hover:shadow-lg"
}

Assert-Test "SortableAccountRow uses semantic ui-interactive-row token for smooth hover" {
    $content = Get-Content "src/components/accounts/AccountTable/SortableAccountRow.tsx" -Raw
    $content -match "ui-interactive-row"
}

Write-Host "`n4. Verifying Prompt Tree Modal Loading & Action Capsule..." -ForegroundColor Yellow

Assert-Test "PromptTreeViewModal displays top indeterminate progress bar during loading" {
    $content = Get-Content "src/components/instances/PromptTreeView/PromptTreeViewModal.tsx" -Raw
    $content -match "bg-gradient-to-r from-transparent via-\[#1af18d\] to-transparent animate-pulse"
}

Assert-Test "PromptTreeViewModal displays shimmer skeleton when isLoading without conversation" {
    $content = Get-Content "src/components/instances/PromptTreeView/PromptTreeViewModal.tsx" -Raw
    $content -match ":\s*isLoading\s*\?\s*\(\s*<div[^>]*animate-pulse"
}

Assert-Test "PromptTreeViewModal wraps details pane in PromptTreeErrorBoundary" {
    $content = Get-Content "src/components/instances/PromptTreeView/PromptTreeViewModal.tsx" -Raw
    $content -match "PromptTreeErrorBoundary"
}

Assert-Test "DetailHeader consolidates actions into segmented split-dropdown capsule" {
    $content = Get-Content "src/components/instances/PromptTreeView/DetailHeader.tsx" -Raw
    ($content -match "isActionMenuOpen" -or $content -match "isActionsMenuOpen") -and $content -match "onEnqueue" -and $content -match "onFocusIde"
}

Assert-Test "Prompt actions log stack traces with captureException" {
    $content = Get-Content "src/components/instances/PromptTreeView/usePromptTreePromptActions.ts" -Raw
    $content -match "captureException\("
}

Write-Host "`n5. Verifying Dual Supabase Configuration & Stack Trace Logging..." -ForegroundColor Yellow

Assert-Test "config_a.rs seeds Primary Lovable endpoint" {
    $content = Get-Content "src-tauri/src/modules/supabase_sync/config_a.rs" -Raw
    $content -match "https://pezjuuddecbyfmqxytrv\.supabase\.co" -and $content -match "sb_publishable_cJdJyIEeXc8bpym9TIOU7w_vVU0_Y69"
}

Assert-Test "config_a.rs seeds Secondary Personal endpoint" {
    $content = Get-Content "src-tauri/src/modules/supabase_sync/config_a.rs" -Raw
    $content -match "https://ikwmurmjynhxdpmhzekt\.supabase\.co" -and $content -match "sb_publishable_barsshQom3VcUw1l5soE_A_R1FwQz8o"
}

Assert-Test "SupabaseClient logs transport errors with stack traces" {
    $content = Get-Content "src-tauri/src/modules/supabase_client/client.rs" -Raw
    $content -match "log_transport_error" -and $content -match "Backtrace::capture"
}

Assert-Test "Frontend error-store captures stackTrace on exceptions" {
    $content = Get-Content "src/stores/error-store.ts" -Raw
    $content -match "stackTrace\?: string" -and $content -match "buildCapturedError"
}

Write-Host "`n6. Verifying Specifications and Execution Plans..." -ForegroundColor Yellow

Assert-Test "Architecture specification exists under 02-spec/21-app/" {
    Test-Path "02-spec/21-app/supabase-connection-and-ui-ux-enhancement/01-architecture-spec.md"
}

Assert-Test "Component specification exists under 02-spec/21-app/" {
    Test-Path "02-spec/21-app/supabase-connection-and-ui-ux-enhancement/02-component-spec.md"
}

Assert-Test "Master execution plan exists in .ai-memory/plans/" {
    Test-Path ".ai-memory/plans/supabase-connection-and-ui-ux-enhancement.md"
}

Assert-Test "All subtask plans exist under .ai-memory/plans/subtasks/" {
    (Test-Path ".ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/01-supabase-dual-cluster-and-stacktrace.md") -and
    (Test-Path ".ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/02-quota-progress-bar-and-layout.md") -and
    (Test-Path ".ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/03-theming-contrast-and-hover-animations.md") -and
    (Test-Path ".ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/04-prompt-tree-loading-and-actions.md") -and
    (Test-Path ".ai-memory/plans/subtasks/supabase-connection-and-ui-ux-enhancement/05-image-sharing-public-url.md")
}

Write-Host "`n-----------------------------------------------------------------" -ForegroundColor Cyan
Write-Host "   Verification Summary: $Passed Passed, $Failed Failed" -ForegroundColor $(if ($Failed -eq 0) { "Green" } else { "Red" })
Write-Host "-----------------------------------------------------------------" -ForegroundColor Cyan

if ($Failed -gt 0) {
    exit 1
} else {
    exit 0
}
