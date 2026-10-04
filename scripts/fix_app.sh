#!/bin/bash
set -E

APP_NAME="Antigravity Manager Tools"
BUNDLE_ID="com.lbjlaq.antigravity-tools"

# Persistent diagnostic logging
LOG_DIR="$HOME/Library/Logs/AntigravityManager"
mkdir -p "$LOG_DIR" 2>/dev/null || true
LOG_FILE="$LOG_DIR/dmg_fix_app.log"
exec > >(tee -a "$LOG_FILE") 2>&1

echo -e "\n[$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)] Running fix_app.sh for ${APP_NAME}..."

# Error stack trace trap for maximum transparency and failure diagnostics
report_error_stack() {
    local exit_code="${3:-$?}"
    if [[ "$exit_code" -eq 0 ]]; then
        exit_code=1
    fi
    local line_no="${1:-$LINENO}"
    local bash_cmd="${2:-${BASH_COMMAND:-unknown}}"
    echo -e "\n\033[0;31m❌ [ERROR] Command '${bash_cmd}' failed with exit code ${exit_code} at line ${line_no}\033[0m" >&2
    local sys_info
    sys_info="$(uname -srm 2>/dev/null || echo "unknown")"
    if command -v sw_vers &>/dev/null; then
        local mac_ver
        mac_ver="$(sw_vers -productVersion 2>/dev/null || true)"
        sys_info="macOS ${mac_ver} (${sys_info})"
    fi
    echo -e "\033[1;33m   [SYSTEM CONTEXT] OS: ${sys_info} | User: $(whoami 2>/dev/null || id -un) | Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)\033[0m" >&2
    echo -e "\033[1;33m   [LOG FILE] Full diagnostic log: ${LOG_FILE}\033[0m" >&2
    if [[ ${#FUNCNAME[@]} -gt 1 ]]; then
        echo -e "\033[1;33m   [STACK TRACE]\033[0m" >&2
        for ((i = 1; i < ${#FUNCNAME[@]}; i++)); do
            local fn="${FUNCNAME[$i]}"
            local src="${BASH_SOURCE[$i]:-fix_app.sh}"
            local ln="${BASH_LINENO[$((i - 1))]}"
            echo -e "     -> at ${fn}() in ${src}:${ln}" >&2
        done
    fi
}
trap 'report_error_stack "$LINENO" "$BASH_COMMAND"' ERR

echo "🛠️  Fixing '${APP_NAME} is damaged or in Trash' issue..."

# 1. 查找 lsregister
LSREGISTER=""
for cand in \
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister" \
    "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister" \
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister" \
    "/System/Library/Frameworks/CoreServices.framework/Versions/Current/Frameworks/LaunchServices.framework/Versions/Current/Support/lsregister"; do
    if [ -x "$cand" ]; then
        LSREGISTER="$cand"
        break
    fi
done

# 2. 清理后台自启服务与终止旧进程
killall "agm-alim" 2>/dev/null || true
killall "${APP_NAME}" 2>/dev/null || true
killall "Antigravity Tools" 2>/dev/null || true
killall "agm" 2>/dev/null || true

LAUNCH_AGENT_PLIST="$HOME/Library/LaunchAgents/${BUNDLE_ID}.plist"
if [ -f "$LAUNCH_AGENT_PLIST" ]; then
    launchctl bootout "gui/$(id -u 2>/dev/null || echo 501)" "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    launchctl unload "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    rm -f "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
fi

# 3. 清理废纸篓冲突副本与注销旧注册
echo "🧹 Purging conflicting Trash copies and stale LaunchServices registrations..."
TRASHED_APPS=()

while IFS= read -r t_app; do
    if [ -n "$t_app" ] && [ -e "$t_app" ]; then
        TRASHED_APPS+=("$t_app")
    fi
done < <(find "$HOME/.Trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null || true)

if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    while IFS= read -r ls_path; do
        if [ -n "$ls_path" ]; then
            if [[ "$ls_path" == *".Trash"* || "$ls_path" == *".Trashes"* ]]; then
                [ -n "$LSREGISTER" ] && "$LSREGISTER" -u "$ls_path" 2>/dev/null || true
                TRASHED_APPS+=("$ls_path")
            fi
        fi
    done < <("$LSREGISTER" -dump 2>/dev/null | awk '
        BEGIN { RS = "--------------------------------------------------------------------------------"; FS = "\n" }
        /com\.lbjlaq\.antigravity-tools|[Aa]ntigravity|[Aa]gm/ {
            for (i = 1; i <= NF; i++) {
                if ($i ~ /^[[:space:]]*path:[[:space:]]*/) {
                    p = $i
                    sub(/^[[:space:]]*path:[[:space:]]*/, "", p)
                    sub(/[[:space:]]*\(0x[0-9a-fA-F]+\)[[:space:]]*$/, "", p)
                    if (length(p) > 0) print p
                }
            }
        }
    ' || true)
fi

# 绕过 TCC 限制：通过 AppleScript 将废纸篓内冲突项移出到 /tmp 并删除
osascript -e '
tell application "Finder"
    try
        set tMatches to (every item of trash whose name contains "Antigravity" or name contains "agm")
        repeat with tItem in tMatches
            try
                move tItem to (POSIX file "/tmp") with replacing
            end try
        end repeat
    end try
end tell' 2>/dev/null || true
rm -rf /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

for ta in "${TRASHED_APPS[@]}"; do
    if [ -n "$ta" ] && { [ -e "$ta" ] || [ -d "$ta" ]; }; then
        [ -n "$LSREGISTER" ] && "$LSREGISTER" -u "$ta" 2>/dev/null || true
        chflags -R nouchg,noschg "$ta" 2>/dev/null || true
        rm -rf "$ta" 2>/dev/null || sudo rm -rf "$ta" 2>/dev/null || true
    fi
done

if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    "$LSREGISTER" -gc 2>/dev/null || true
fi

# 4. 动态搜索有效应用路径
APP_PATH=""
CANDIDATES=(
    "/Applications/${APP_NAME}.app"
    "$HOME/Applications/${APP_NAME}.app"
    "/Applications/Antigravity Tools.app"
    "$HOME/Applications/Antigravity Tools.app"
    "/Applications/agm-alim.app"
    "$HOME/Applications/agm-alim.app"
)
for cand in "${CANDIDATES[@]}"; do
    if [ -d "$cand" ]; then
        APP_PATH="$cand"
        break
    fi
done

if [ -n "$APP_PATH" ] && [ -d "$APP_PATH" ]; then
    echo "📍 Found application: $APP_PATH"
    echo "🔑 Removing quarantine attributes and fixing permissions..."

    chmod -R u+rwX "$APP_PATH" 2>/dev/null || sudo chmod -R u+rwX "$APP_PATH" 2>/dev/null || true
    if [ -d "$APP_PATH/Contents/MacOS" ]; then
        chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || sudo chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || true
    fi

    xattr -cr "$APP_PATH" 2>/dev/null || true
    xattr -rd com.apple.quarantine "$APP_PATH" 2>/dev/null || true
    xattr -rd com.apple.provenance "$APP_PATH" 2>/dev/null || true
    find "$APP_PATH" -exec xattr -c {} + 2>/dev/null || true

    if command -v spctl &>/dev/null; then
        spctl --add "$APP_PATH" 2>/dev/null || sudo spctl --add "$APP_PATH" 2>/dev/null || true
    fi

    if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
        echo "🔄 Refreshing LaunchServices database..."
        "$LSREGISTER" -u "$APP_PATH" 2>/dev/null || true
        "$LSREGISTER" -gc 2>/dev/null || true
        "$LSREGISTER" -f -r "$APP_PATH" 2>/dev/null || true
        killall launchservicesd Finder Dock 2>/dev/null || true
        sleep 1
    fi

    echo "✅ Fix successful! The app should now launch normally."
    echo "Launching $APP_PATH..."
    if ! open "$APP_PATH" 2>/dev/null; then
        direct_bin="${APP_PATH}/Contents/MacOS/agm-alim"
        [ ! -x "$direct_bin" ] && direct_bin="${APP_PATH}/Contents/MacOS/${APP_NAME}"
        [ ! -x "$direct_bin" ] && direct_bin="${APP_PATH}/Contents/MacOS/agm"
        if [ -x "$direct_bin" ]; then
            echo "Directly launching core binary..."
            chmod +x "$direct_bin" 2>/dev/null || true
            xattr -cr "$direct_bin" 2>/dev/null || true
            local bin_log="$LOG_DIR/binary_launch.log"
            nohup "$direct_bin" >> "$bin_log" 2>&1 &
            local bin_pid=$!
            sleep 2
            if kill -0 "$bin_pid" 2>/dev/null; then
                echo "🎉 Core binary running (PID: $bin_pid)"
            else
                echo "❌ Core binary exited. Last log:"
                tail -n 20 "$bin_log" 2>/dev/null || true
            fi
        fi
    fi
else
    echo "⚠️  Application not found in /Applications."
    echo "   Please install '${APP_NAME}.app' first."
fi
