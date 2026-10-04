#!/bin/bash
set -E

# Resolve script directory and app configuration
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_NAME="Antigravity Manager Tools"
BUNDLE_ID="com.lbjlaq.antigravity-tools"

# Formatting colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Persistent diagnostic logging configuration
LOG_DIR="$HOME/Library/Logs/AntigravityManager"
mkdir -p "$LOG_DIR" 2>/dev/null || true
LOG_FILE="$LOG_DIR/dmg_repair.log"
exec > >(tee -a "$LOG_FILE") 2>&1

echo -e "\n\033[1;30m[$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)] Starting ${APP_NAME} Quick Repair Tool\033[0m"

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
    echo -e "\033[1;33m   [SYSTEM CONTEXT] OS: ${sys_info} | User: $(whoami 2>/dev/null || id -un) (UID: $(id -u 2>/dev/null || true)) | Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)\033[0m" >&2
    echo -e "\033[1;33m   [LOG FILE] Full diagnostic log: ${LOG_FILE}\033[0m" >&2
    if [[ ${#FUNCNAME[@]} -gt 1 ]]; then
        echo -e "\033[1;33m   [STACK TRACE]\033[0m" >&2
        for ((i = 1; i < ${#FUNCNAME[@]}; i++)); do
            local fn="${FUNCNAME[$i]}"
            local src="${BASH_SOURCE[$i]:-Fix_Damaged.command}"
            local ln="${BASH_LINENO[$((i - 1))]}"
            echo -e "     -> at ${fn}() in ${src}:${ln}" >&2
        done
    fi
}
trap 'report_error_stack "$LINENO" "$BASH_COMMAND"' ERR

# Diagnostic state dump function
dump_diagnostic_state() {
    local target="${1:-}"
    echo -e "\n\033[1;36m==================== [SYSTEM DIAGNOSTICS & STACK TRACE] ====================\033[0m"
    echo -e "\033[1;33m[1] System Information & Hardware Architecture:\033[0m"
    if command -v sw_vers &>/dev/null; then
        sw_vers 2>/dev/null | sed 's/^/    /' || true
    fi
    echo "    Kernel : $(uname -a 2>/dev/null || true)"
    echo "    Machine: $(uname -m 2>/dev/null || true)"
    if command -v sysctl &>/dev/null; then
        local cpu_brand
        cpu_brand="$(sysctl -n machdep.cpu.brand_string 2>/dev/null || true)"
        [ -n "$cpu_brand" ] && echo "    CPU    : $cpu_brand"
        local translated
        translated="$(sysctl -n sysctl.proc_translated 2>/dev/null || true)"
        [ -n "$translated" ] && echo "    Rosetta: Translated=$translated (1=x86 binary running via Rosetta on Apple Silicon)"
    fi
    echo "    User   : $(whoami 2>/dev/null || true) (UID: $(id -u 2>/dev/null || true), GID: $(id -g 2>/dev/null || true))"
    echo "    Console: $(stat -f '%Su' /dev/console 2>/dev/null || echo "unknown")"
    echo "    Shell  : ${SHELL:-bash} (BASH_VERSION: ${BASH_VERSION:-unknown})"
    echo "    Log    : ${LOG_FILE}"

    echo -e "\033[1;33m[2] Gatekeeper Status:\033[0m"
    if command -v spctl &>/dev/null; then
        spctl --status 2>&1 | sed 's/^/    /' || true
        if [ -n "$target" ] && [ -d "$target" ]; then
            echo "    Assessment for target bundle:"
            spctl -a -vvvv "$target" 2>&1 | sed 's/^/      /' || true
        fi
    else
        echo "    spctl not available"
    fi

    echo -e "\033[1;33m[3] Target Application Attributes & Signatures:\033[0m"
    if [ -n "$target" ] && [ -d "$target" ]; then
        echo "    Path: $target"
        echo "    Extended Attributes (xattr -lv):"
        xattr -lv "$target" 2>&1 | sed 's/^/      /' || echo "      (none)"
        if command -v codesign &>/dev/null; then
            echo "    Codesign Verification (codesign -vvvv --deep --strict):"
            codesign -vvvv --deep --strict "$target" 2>&1 | sed 's/^/      /' || true
            echo "    Codesign Entitlements:"
            codesign -d --entitlements :- "$target" 2>&1 | sed 's/^/      /' || true
        fi
    else
        echo "    Target bundle not found at path: '$target'"
    fi

    echo -e "\033[1;33m[4] LaunchServices Registrations for ${BUNDLE_ID}:\033[0m"
    if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
        "$LSREGISTER" -dump 2>/dev/null | awk '
            BEGIN { RS = "--------------------------------------------------------------------------------"; FS = "\n" }
            /com\.lbjlaq\.antigravity-tools|[Aa]ntigravity|[Aa]gm/ {
                for (i = 1; i <= NF; i++) {
                    if ($i ~ /^[[:space:]]*(path|identifier|name|version|flags):/) print "    " $i
                }
            }
        ' || echo "    (none registered)"
    else
        echo "    lsregister binary not located"
    fi

    echo -e "\033[1;33m[5] Lingering Trash Artifacts across All Volumes:\033[0m"
    local found_trash=0
    for t_dir in "$HOME/.Trash" "/.Trashes" /Volumes/*/.Trashes; do
        if [ -d "$t_dir" ]; then
            while IFS= read -r t_item; do
                if [ -n "$t_item" ]; then
                    found_trash=1
                    echo "    Trashed item: $t_item"
                    ls -ld "$t_item" 2>/dev/null | sed 's/^/      /' || true
                fi
            done < <(find "$t_dir" -maxdepth 2 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null || true)
        fi
    done
    if [ "$found_trash" -eq 0 ]; then
        echo "    (clean - no Antigravity items in Trash)"
    fi

    echo -e "\033[1;33m[6] Active LaunchAgents:\033[0m"
    find "$HOME/Library/LaunchAgents" -iname "*antigravity*" 2>/dev/null | sed 's/^/    /' || echo "    (none)"

    echo -e "\033[1;33m[7] Active Antigravity Processes:\033[0m"
    pgrep -l -i "agm|antigravity" 2>/dev/null | sed 's/^/    /' || echo "    (none running)"

    echo -e "\033[1;33m[8] Recent Crash Reports & Stack Traces (~/Library/Logs/DiagnosticReports):\033[0m"
    local found_crash=0
    for cr_dir in "$HOME/Library/Logs/DiagnosticReports" "/Library/Logs/DiagnosticReports"; do
        if [ -d "$cr_dir" ]; then
            while IFS= read -r cr_path; do
                if [ -n "$cr_path" ] && [ -f "$cr_path" ]; then
                    found_crash=1
                    echo "    Crash Report: $cr_path"
                    head -n 45 "$cr_path" 2>/dev/null | sed 's/^/      /' || true
                fi
            done < <(find "$cr_dir" -maxdepth 1 \( -iname "*agm*" -o -iname "*antigravity*" \) -mmin -120 2>/dev/null | head -n 3 || true)
        fi
    done
    if [ "$found_crash" -eq 0 ]; then
        echo "    (no recent crash reports found in last 120 minutes)"
    fi

    echo -e "\033[1;33m[9] macOS Unified System Log Extracts (LaunchServices & Process Lifecycle):\033[0m"
    if command -v log &>/dev/null; then
        log show --predicate 'process == "agm-alim" || process == "agm" || process == "open" || subsystem == "com.apple.LaunchServices"' --last 3m --style compact 2>/dev/null | tail -n 20 | sed 's/^/    /' || echo "    (no log entries)"
    else
        echo "    (log command not available)"
    fi

    echo -e "\033[1;33m[10] Binary Architecture & Dynamic Linkage:\033[0m"
    if [ -n "$target" ] && [ -d "$target" ]; then
        local t_bin="${target}/Contents/MacOS/agm-alim"
        [ ! -f "$t_bin" ] && t_bin="${target}/Contents/MacOS/agm"
        [ ! -f "$t_bin" ] && t_bin=$(find "${target}/Contents/MacOS" -type f 2>/dev/null | head -n 1 || true)
        if [ -n "$t_bin" ] && [ -f "$t_bin" ]; then
            echo "    Binary : $t_bin"
            echo "    Format : $(file "$t_bin" 2>/dev/null || true)"
            if command -v otool &>/dev/null; then
                echo "    Dynamic libraries (otool -L):"
                otool -L "$t_bin" 2>/dev/null | head -n 15 | sed 's/^/      /' || true
            fi
        fi
    fi

    echo -e "\033[1;33m[11] Direct Binary Launch Log Tail:\033[0m"
    if [ -f "$LOG_DIR/binary_launch.log" ]; then
        tail -n 25 "$LOG_DIR/binary_launch.log" 2>/dev/null | sed 's/^/    /' || echo "    (empty)"
    else
        echo "    (no direct binary launch log recorded)"
    fi
    echo -e "\033[1;36m============================================================================\033[0m\n"
}

# Function to launch the repaired application safely with multiple fallback layers
launch_repaired_application() {
    local target_app="$1"
    local app_name="$2"
    local lsregister_bin="$3"
    local launch_ok=0
    local open_err=""

    echo ""
    echo -e "🚀 正在尝试直接启动应用 / Launching ${app_name}: ${CYAN}${target_app}${NC}..."

    if open_err=$(open "$target_app" 2>&1); then
        echo -e "${GREEN}🎉 应用已顺利启动! / Launched successfully via LaunchServices!${NC}"
        return 0
    fi

    echo -e "${YELLOW}⚠️  LaunchServices 启动遇到障碍 (open exited with error): ${open_err}${NC}"
    echo -e "${YELLOW}🔄 执行全方位废纸篓脱困、LaunchServices 深度重置与反注册恢复...${NC}"

    # 1. 绕过 TCC 限制：通过 AppleScript 将废纸篓内冲突项移出到 /private/tmp 并彻底销毁
    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true
    osascript -e '
    tell application "Finder"
        try
            set destFolder to (POSIX file "/private/tmp") as alias
            repeat with anItem in (every item of trash)
                try
                    set n to name of anItem as text
                    if n contains "Antigravity" or n contains "agm" then
                        move anItem to destFolder with replacing
                    end if
                end try
            end repeat
        end try
    end tell' 2>/dev/null || true
    rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

    # 2. 深度清理本地 Trash 目录与解除锁定
    find "$HOME/.Trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null | while read -r t_item; do
        if [ -n "$t_item" ]; then
            [ -n "$lsregister_bin" ] && "$lsregister_bin" -u "$t_item" 2>/dev/null || true
            chflags -R nouchg,noschg "$t_item" 2>/dev/null || true
            rm -rf "$t_item" 2>/dev/null || true
        fi
    done

    # 3. 运行 LaunchServices 垃圾回收与强制重置注册 (跨所有 domain: user, system, local)
    if [ -n "$lsregister_bin" ] && [ -x "$lsregister_bin" ]; then
        echo -e "🧹 正在执行 LaunchServices 垃圾回收与重置 (lsregister -gc -R -apps u,s,l)..."
        "$lsregister_bin" -u "$target_app" 2>/dev/null || true
        "$lsregister_bin" -gc -R -v -apps u,s,l 2>/dev/null || "$lsregister_bin" -gc 2>/dev/null || true
        "$lsregister_bin" -f -r "$target_app" 2>/dev/null || true
        killall Finder Dock 2>/dev/null || true
    fi

    sleep 1
    if open_err=$(open -n "$target_app" 2>&1) || open_err=$(open "$target_app" 2>&1); then
        echo -e "${GREEN}🎉 二次重试启动成功! / Launched on retry!${NC}"
        return 0
    fi
    echo -e "${YELLOW}⚠️  二次重试提示: ${open_err}${NC}"

    # 4. 终极保险：通过底层核心二进制直接启动 (彻底绕过 Finder / LaunchServices 废纸篓拦截)
    local internal_bin="${target_app}/Contents/MacOS/agm-alim"
    [ ! -x "$internal_bin" ] && internal_bin="${target_app}/Contents/MacOS/${app_name}"
    [ ! -x "$internal_bin" ] && internal_bin="${target_app}/Contents/MacOS/agm"
    if [ -x "$internal_bin" ]; then
        echo -e "🚀 正在通过核心直接启动 (绕过 LaunchServices/Finder 拦截) / Launching direct binary: ${CYAN}${internal_bin}${NC}..."
        chmod +x "$internal_bin" 2>/dev/null || true
        xattr -cr "$internal_bin" 2>/dev/null || true
        local bin_log="$LOG_DIR/binary_launch.log"
        nohup "$internal_bin" >> "$bin_log" 2>&1 &
        local bin_pid=$!
        sleep 2
        if kill -0 "$bin_pid" 2>/dev/null; then
            echo -e "${GREEN}🎉 核心进程已成功启动 (PID: $bin_pid)! / Core binary running!${NC}"
            return 0
        fi
        echo -e "${RED}❌ 核心直接启动后未能持续运行，进程退出。核心日志:${NC}"
        tail -n 25 "$bin_log" 2>/dev/null | sed 's/^/    /' || true
    fi

    dump_diagnostic_state "$target_app"
    echo -e "${YELLOW}如果系统弹出 Gatekeeper 提示，请前往: 系统设置 -> 隐私与安全性 -> 点击【仍要打开】${NC}"
    echo -e "${YELLOW}If prompted by Gatekeeper, navigate to: System Settings -> Privacy & Security -> Click 'Open Anyway'${NC}"
    return 1
}

echo -e "${GREEN}====================================================${NC}"
echo -e "${GREEN}   ${APP_NAME} - 快速修复助手 / Quick Repair${NC}"
echo -e "${GREEN}====================================================${NC}"
echo ""

# 1. 查找 LaunchServices 注册工具 lsregister
LSREGISTER=""
LSREGISTER_CANDIDATES=(
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Support/lsregister"
    "/System/Library/Frameworks/CoreServices.framework/Versions/A/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"
    "/System/Library/Frameworks/CoreServices.framework/Frameworks/LaunchServices.framework/Versions/A/Support/lsregister"
    "/System/Library/Frameworks/CoreServices.framework/Versions/Current/Frameworks/LaunchServices.framework/Versions/Current/Support/lsregister"
)
for cand in "${LSREGISTER_CANDIDATES[@]}"; do
    if [ -x "$cand" ]; then
        LSREGISTER="$cand"
        break
    fi
done
if [ -z "$LSREGISTER" ]; then
    LSREGISTER=$(find /System/Library/Frameworks/CoreServices.framework -name "lsregister" -type f 2>/dev/null | head -n 1 || true)
fi

# 2. 终止正在运行的旧进程与卸载遗留后台自启服务，防止文件锁定
echo -e "🛑 正在检查正在运行的旧进程与后台自启服务 / Checking for active processes & services..."
killall "agm-alim" 2>/dev/null || true
killall "${APP_NAME}" 2>/dev/null || true
killall "Antigravity Tools" 2>/dev/null || true
killall "agm" 2>/dev/null || true

LAUNCH_AGENT_PLIST="$HOME/Library/LaunchAgents/${BUNDLE_ID}.plist"
if [ -f "$LAUNCH_AGENT_PLIST" ]; then
    echo -e "🔄 正在卸载冲突的 LaunchAgent / Unloading stale LaunchAgent..."
    launchctl bootout "gui/$(id -u 2>/dev/null || echo 501)" "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    launchctl unload "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
    rm -f "$LAUNCH_AGENT_PLIST" 2>/dev/null || true
fi

# 3. 深度扫描并清理废纸篓 (Trash) 与 LaunchServices 中的冲突残留副本 (如 "... 13-20-55-339.app")
echo -e "🔍 正在检查废纸篓残留 / Inspecting Trash for quarantined items..."
TRASHED_APPS=()

# 扫描用户废纸篓顶层条目
while IFS= read -r t_app; do
    if [ -n "$t_app" ] && [ -e "$t_app" ]; then
        TRASHED_APPS+=("$t_app")
    fi
done < <(find "$HOME/.Trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null || true)

# 扫描挂载卷与外部磁盘中的废纸篓
for ext_trash in "/Volumes"/*/.Trashes/"$(id -u 2>/dev/null || echo 501)" "/.Trashes/$(id -u 2>/dev/null || echo 501)"; do
    if [ -d "$ext_trash" ]; then
        while IFS= read -r t_app; do
            if [ -n "$t_app" ] && [ -e "$t_app" ]; then
                TRASHED_APPS+=("$t_app")
            fi
        done < <(find "$ext_trash" -maxdepth 1 \( -iname "*antigravity*" -o -iname "*agm*" \) 2>/dev/null || true)
    fi
done

# 反查 LaunchServices 注册在 Trash 内或已失效的路径
if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    while IFS= read -r ls_path; do
        if [ -n "$ls_path" ]; then
            if [[ "$ls_path" == *".Trash"* || "$ls_path" == *".Trashes"* ]]; then
                echo -e "${YELLOW}⚠️  检测到 LaunchServices 指向废纸篓路径 / Stale LaunchServices Trash entry: $ls_path${NC}"
                "$LSREGISTER" -u "$ls_path" 2>/dev/null || true
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

# 绕过 TCC 限制：通过 AppleScript 将废纸篓内冲突项移出到 /private/tmp 并物理清理
rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true
osascript -e '
tell application "Finder"
    try
        set destFolder to (POSIX file "/private/tmp") as alias
        repeat with anItem in (every item of trash)
            try
                set n to name of anItem as text
                if n contains "Antigravity" or n contains "agm" then
                    move anItem to destFolder with replacing
                end if
            end try
        end repeat
    end try
end tell' 2>/dev/null || true
rm -rf /private/tmp/*[Aa]ntigravity* /private/tmp/*[Aa]gm* /tmp/*[Aa]ntigravity* /tmp/*[Aa]gm* 2>/dev/null || true

# 彻底清理废纸篓副本与解除文件锁定
if [ ${#TRASHED_APPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠️  发现 ${#TRASHED_APPS[@]} 个废纸篓冲突副本，正在彻底清除并注销注册... / Purging trashed duplicates:${NC}"
    for ta in "${TRASHED_APPS[@]}"; do
        if [ -n "$ta" ] && { [ -e "$ta" ] || [ -d "$ta" ]; }; then
            echo -e "   🗑️  正在注销与清理 / Purging: $ta"
            if [ -n "$LSREGISTER" ]; then
                "$LSREGISTER" -u "$ta" 2>/dev/null || true
            fi
            chflags -R nouchg,noschg "$ta" 2>/dev/null || true
            rm -rf "$ta" 2>/dev/null || sudo rm -rf "$ta" 2>/dev/null || true
        fi
    done
    echo -e "${GREEN}✅ 废纸篓冲突副本已全部清理完毕 / Trash duplicates purged successfully.${NC}"
fi

# 运行 LaunchServices 垃圾回收以清除死路径 (跨 user, system, local 域)
if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    "$LSREGISTER" -gc -R -v -apps u,s,l 2>/dev/null || "$LSREGISTER" -gc 2>/dev/null || true
fi

# 4. 动态查找有效应用程序路径
APP_PATH=""

# 4.1 优先在系统或用户 Applications 目录查找已安装版本
CANDIDATES=(
    "/Applications/${APP_NAME}.app"
    "$HOME/Applications/${APP_NAME}.app"
    "/Applications/Antigravity Tools.app"
    "$HOME/Applications/Antigravity Tools.app"
    "/Applications/Anti-Gravity Tools.app"
    "$HOME/Applications/Anti-Gravity Tools.app"
    "/Applications/agm-alim.app"
    "$HOME/Applications/agm-alim.app"
)
for candidate in "${CANDIDATES[@]}"; do
    if [ -d "$candidate" ]; then
        APP_PATH="$candidate"
        break
    fi
done

# 4.2 如果在系统目录未找到，或者当前运行在 DMG / 解压目录中，定位本地源应用
SOURCE_APP_IN_DIR=""
if [ -d "$DIR/${APP_NAME}.app" ]; then
    SOURCE_APP_IN_DIR="$DIR/${APP_NAME}.app"
elif [ -d "$DIR/Antigravity Tools.app" ]; then
    SOURCE_APP_IN_DIR="$DIR/Antigravity Tools.app"
elif [ -d "$DIR/agm-alim.app" ]; then
    SOURCE_APP_IN_DIR="$DIR/agm-alim.app"
else
    LOCAL_APP=$(find "$DIR" -maxdepth 2 -name "*.app" -type d 2>/dev/null | head -n 1)
    if [ -n "$LOCAL_APP" ] && [ -d "$LOCAL_APP" ]; then
        SOURCE_APP_IN_DIR="$LOCAL_APP"
    fi
fi

# 4.3 如果 /Applications 中未安装应用，但同级目录存在源应用，自动安装到 /Applications
if [ -z "$APP_PATH" ] || [ ! -d "$APP_PATH" ]; then
    if [ -n "$SOURCE_APP_IN_DIR" ] && [ -d "$SOURCE_APP_IN_DIR" ]; then
        TARGET_DEST="/Applications"
        if [ ! -w "$TARGET_DEST" ]; then
            TARGET_DEST="$HOME/Applications"
            mkdir -p "$TARGET_DEST" 2>/dev/null || true
        fi
        echo -e "📦 正在将应用安全复制到 ${TARGET_DEST}... / Copying app to ${TARGET_DEST}..."
        APP_PATH="${TARGET_DEST}/${APP_NAME}.app"
        rm -rf "$APP_PATH" 2>/dev/null || sudo rm -rf "$APP_PATH" 2>/dev/null || true
        cp -R "$SOURCE_APP_IN_DIR" "$APP_PATH" || sudo cp -R "$SOURCE_APP_IN_DIR" "$APP_PATH"
        echo -e "${GREEN}✅ 已成功安装到 / Installed to: $APP_PATH${NC}"
    fi
fi

# 4.4 兜底模糊匹配 Applications 目录
if [ -z "$APP_PATH" ] || [ ! -d "$APP_PATH" ]; then
    SEARCH_APP=$(find /Applications "$HOME/Applications" -maxdepth 2 \( -iname "*antigravity*.app" -o -iname "*agm*.app" \) -type d 2>/dev/null | head -n 1)
    if [ -n "$SEARCH_APP" ] && [ -d "$SEARCH_APP" ]; then
        APP_PATH="$SEARCH_APP"
    fi
fi

# 5. 执行权限与隔离属性深度修复
if [ -n "$APP_PATH" ] && [ -d "$APP_PATH" ]; then
    TARGET_NAME=$(basename "$APP_PATH" .app)
    echo -e "📍 目标应用路径 / Target application path: ${BLUE}$APP_PATH${NC}"

    macos_major=0
    if command -v sw_vers &>/dev/null; then
        macos_major=$(sw_vers -productVersion | cut -d. -f1)
    fi

    echo "🔧 正在递归清除 Gatekeeper 隔离属性与文件锁定 / Clearing quarantine attributes..."
    chmod -R u+rwX "$APP_PATH" 2>/dev/null || sudo chmod -R u+rwX "$APP_PATH" 2>/dev/null || true
    if [ -d "$APP_PATH/Contents/MacOS" ]; then
        chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || sudo chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || true
    fi

    # 清除各种 macOS 隔离与溯源标记
    xattr -cr "$APP_PATH" 2>/dev/null || true
    xattr -rd com.apple.quarantine "$APP_PATH" 2>/dev/null || true
    xattr -rd com.apple.provenance "$APP_PATH" 2>/dev/null || true
    find "$APP_PATH" -exec xattr -c {} + 2>/dev/null || true
    find "$APP_PATH" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true
    find "$APP_PATH" -exec xattr -d com.apple.provenance {} + 2>/dev/null || true

    # 若仍有顽固隔离标记则请求 sudo 一次性彻底清除
    if xattr -r "$APP_PATH" 2>/dev/null | grep -qE "com.apple.quarantine|com.apple.provenance"; then
        echo -e "${YELLOW}提示: 当前目录权限受限，请输入开机密码授权 (输入时不显示):${NC}"
        echo -e "${YELLOW}Note: Permissions restricted. Please enter login password (keystrokes hidden):${NC}"
        sudo chmod -R u+rwX "$APP_PATH" 2>/dev/null || true
        sudo chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || true
        sudo xattr -cr "$APP_PATH" 2>/dev/null || true
        sudo xattr -rd com.apple.quarantine "$APP_PATH" 2>/dev/null || true
        sudo xattr -rd com.apple.provenance "$APP_PATH" 2>/dev/null || true
        sudo find "$APP_PATH" -exec xattr -c {} + 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ 隔离属性与权限已完全净化 / Quarantine attributes cleared.${NC}"

    # 向 Gatekeeper 注册 (macOS 13+ Ventura / Sonoma / Sequoia)
    if [[ "$macos_major" -ge 13 ]] && command -v spctl &>/dev/null; then
        echo "🔐 正在向看门人注册应用 / Registering app with Gatekeeper (spctl)..."
        spctl --add "$APP_PATH" 2>/dev/null || sudo spctl --add "$APP_PATH" 2>/dev/null || true
        echo -e "${GREEN}✅ 看门人注册完成 / Gatekeeper registration done.${NC}"
    fi

    if command -v codesign &>/dev/null; then
        echo "🔏 正在应用本地自签名 / Applying local ad-hoc codesign..."
        codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || sudo codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || true
        echo -e "${GREEN}✅ 本地签名完成 / Ad-hoc codesign done.${NC}"
    fi

    # 正确重构 LaunchServices 数据库：强制重新注册，通知 Finder/Dock
    if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
        echo "🔄 正在重构并刷新 LaunchServices 应用程序数据库 (全域重置) / Refreshing LaunchServices database..."
        "$LSREGISTER" -u "$APP_PATH" 2>/dev/null || true
        "$LSREGISTER" -gc -R -v -apps u,s,l 2>/dev/null || "$LSREGISTER" -gc 2>/dev/null || true
        "$LSREGISTER" -f -r "$APP_PATH" 2>/dev/null || true
        killall Finder Dock 2>/dev/null || true
        sleep 1
        echo -e "${GREEN}✅ LaunchServices 数据库已重构注册 / LaunchServices refreshed.${NC}"
    fi

    # 检查并创建 CLI 软链接
    MACOS_BIN_DIR="${APP_PATH}/Contents/MacOS"
    if [ -d "$MACOS_BIN_DIR" ]; then
        CLI_BIN=""
        for candidate_name in "agm" "agm-alim" "${APP_NAME}" "Antigravity Tools" "antigravity-tools"; do
            if [ -x "${MACOS_BIN_DIR}/${candidate_name}" ]; then
                CLI_BIN="${MACOS_BIN_DIR}/${candidate_name}"
                break
            fi
        done
        if [ -z "$CLI_BIN" ]; then
            for f in "${MACOS_BIN_DIR}"/*; do
                if [ -f "$f" ] && [ -x "$f" ]; then
                    CLI_BIN="$f"
                    break
                fi
            done
        fi
        if [ -n "$CLI_BIN" ]; then
            chmod +x "$CLI_BIN" 2>/dev/null || true
            USER_BIN="$HOME/.local/bin"
            mkdir -p "$USER_BIN" 2>/dev/null || true
            ln -sf "$CLI_BIN" "${USER_BIN}/agm" 2>/dev/null || true
            ln -sf "$CLI_BIN" "${USER_BIN}/agm-alim" 2>/dev/null || true
            echo -e "🔗 已配置命令行工具 / Configured CLI symlinks: ${CYAN}${USER_BIN}/agm, ${USER_BIN}/agm-alim${NC}"

            SHELL_RCS=()
            [ -f "$HOME/.zshrc" ] && SHELL_RCS+=("$HOME/.zshrc")
            [ -f "$HOME/.bashrc" ] && SHELL_RCS+=("$HOME/.bashrc")
            [ -f "$HOME/.bash_profile" ] && SHELL_RCS+=("$HOME/.bash_profile")
            [ ${#SHELL_RCS[@]} -eq 0 ] && SHELL_RCS+=("$HOME/.zshrc")

            for rc in "${SHELL_RCS[@]}"; do
                if [ -f "$rc" ] && grep -qF "${USER_BIN}" "$rc" 2>/dev/null; then
                    continue
                fi
                echo "" >> "$rc" 2>/dev/null || true
                echo "export PATH=\"${USER_BIN}:\$PATH\"" >> "$rc" 2>/dev/null || true
                echo "➕ 已添加 ${USER_BIN} 到 PATH / Added ${USER_BIN} to PATH in $rc"
            done
        fi
    fi

    echo ""
    echo -e "${GREEN}====================================================${NC}"
    echo -e "${GREEN}   ✅ 修复完成! / Repair complete!${NC}"
    echo -e "${GREEN}====================================================${NC}"
    echo "您现在可以正常打开 $TARGET_NAME 了。"
    echo "You can now open $TARGET_NAME normally."

    osascript -e "display notification \"修复成功，现在可以正常打开应用了\" with title \"$TARGET_NAME\" sound name \"Glass\"" 2>/dev/null || true

    # 启动应用
    launch_repaired_application "$APP_PATH" "$APP_NAME" "$LSREGISTER"
else
    echo -e "${RED}⚠️  未找到应用文件 / Application bundle not found${NC}"
    echo "请确保将 '${APP_NAME}.app' 放置在 /Applications 或与本修复脚本位于同一目录下。"
    echo "Please ensure '${APP_NAME}.app' is in /Applications or in the same folder as this script."
    dump_diagnostic_state ""
fi

echo ""
echo "详细运行日志与诊断信息保存在 / Log file: $LOG_FILE"
echo "按任意键退出 / Press any key to exit..."
read -n 1 -s -r -p ""
echo ""
