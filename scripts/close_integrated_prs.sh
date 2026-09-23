#!/bin/bash

# Script to close integrated PRs
# Before running, ensure GitHub CLI is installed and authenticated: brew install gh && gh auth login

REPO="alimtvnetwork/Antigravity-Manager"
VERSION="v4.65.0"

# Thank you comment template
THANK_YOU_MESSAGE="Thank you for your contribution! 🎉

The changes from this PR have been integrated into the project codebase.

The updates are documented in:
- README.md changelog
- Contributors list

Thank you again for your support of the Antigravity Manager Tools project!"

echo "================================================"
echo "Closing PRs integrated into ${VERSION}"
echo "================================================"
echo ""

# PR List format: "PR_NUMBER|AUTHOR|TITLE"
PRS_LIST=(
    "825|IamAshrafee|[Internationalization] Device Fingerprint Dialog localization"
    "822|Koshikai|[Japanese] Add missing translations and refine terminology"
    "798|vietnhatthai|[Translation Fix] Correct spelling error in Vietnamese settings"
    "846|lengjingxu|[Core Feature] Client Hot Update & Token Stats System"
    "949|lbjlaq|Streaming chunks order fix"
    "950|lbjlaq|[Fix] Remove redundant code and update README"
    "973|Mag1cFall|fix: Fix Windows platform startup parameters not taking effect"
)

# Check if GitHub CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI is not installed"
    echo ""
    echo "Please install GitHub CLI first:"
    echo "  brew install gh"
    echo ""
    echo "Then authenticate:"
    echo "  gh auth login"
    echo ""
    exit 1
fi

# Verify authentication
echo "Checking GitHub CLI authentication status..."
if ! gh auth status &> /dev/null; then
    echo "❌ GitHub CLI is not authenticated"
    echo "Please run 'gh auth login' to authenticate"
    exit 1
fi

echo "✅ GitHub CLI is authenticated"
echo ""

# Process each PR
SUCCESS_COUNT=0
SKIP_COUNT=0
FAIL_COUNT=0

for pr_info in "${PRS_LIST[@]}"; do
    IFS="|" read -r pr_number pr_author pr_title <<< "$pr_info"
    
    echo "----------------------------------------"
    echo "Processing PR #${pr_number}: ${pr_title}"
    echo "Author: @${pr_author}"
    
    # Check PR status
    pr_state=$(gh pr view "$pr_number" --repo "$REPO" --json state --jq .state 2>/dev/null)
    
    if [ $? -ne 0 ]; then
        echo "⚠️  Unable to fetch PR #${pr_number} status (may not exist or lacks permission)"
        ((FAIL_COUNT++))
        continue
    fi
    
    if [ "$pr_state" == "CLOSED" ] || [ "$pr_state" == "MERGED" ]; then
        echo "ℹ️  PR #${pr_number} is already closed or merged (${pr_state}), skipping"
        ((SKIP_COUNT++))
        continue
    fi
    
    # Add comment and close PR
    echo "Adding thank-you comment and closing PR #${pr_number}..."
    if gh pr comment "$pr_number" --repo "$REPO" --body "$THANK_YOU_MESSAGE" && \
       gh pr close "$pr_number" --repo "$REPO"; then
        echo "✅ PR #${pr_number} closed successfully!"
        ((SUCCESS_COUNT++))
    else
        echo "❌ Failed to close PR #${pr_number}"
        ((FAIL_COUNT++))
    fi
    
    # Polite delay to prevent API rate limiting
    sleep 2
done

echo ""
echo "================================================"
echo "Summary"
echo "================================================"
echo "Successfully closed: ${SUCCESS_COUNT}"
echo "Skipped: ${SKIP_COUNT}"
echo "Failed: ${FAIL_COUNT}"
echo "Total processed: ${#PRS_LIST[@]}"
echo "================================================"
