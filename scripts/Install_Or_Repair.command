#!/bin/bash
set -E

# Resolve script directory and app configuration
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_NAME="Antigravity Manager Tools"
BUNDLE_ID="com.lbjlaq.antigravity-tools"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m'

# Persistent diagnostic logging configuration
LOG_DIR="$HOME/Library/Logs/AntigravityManager"
mkdir -p "$LOG_DIR" 2>/dev/null || true
LOG_FILE="$LOG_DIR/dmg_install.log"
exec > >(tee -a "$LOG_FILE") 2>&1

echo -e "\n\033[1;30m[$(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)] Starting ${APP_NAME} 1-Click Installer & Repair\033[0m"

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
            local src="${BASH_SOURCE[$i]:-Install_Or_Repair.command}"
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

# Function to launch the installed application safely with multiple fallback layers
launch_installed_application() {
    local target_app="$1"
    local app_name="$2"
    local lsregister_bin="$3"
    local launch_ok=0
    local open_err=""

    echo ""
    echo -e "🚀 正在启动应用 / Launching application: ${CYAN}${target_app}${NC}..."

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
        echo -e "${GREEN}🎉 重试启动成功! / Launched successfully on retry!${NC}"
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
    echo -e "${YELLOW}提示: 如果 macOS 提示“无法打开未知名开发者”，请前往: 系统设置 -> 隐私与安全性 -> 点击【仍要打开】${NC}"
    echo -e "${YELLOW}Tip: If Gatekeeper blocks opening, go to: System Settings -> Privacy & Security -> Click 'Open Anyway'${NC}"
    return 1
}

echo -e "${CYAN}================================================================${NC}"
echo -e "${CYAN}   🚀 ${APP_NAME} - 一键安装与全自动修复 / One-Click Installer${NC}"
echo -e "${CYAN}================================================================${NC}"
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

# 3. 彻底清理废纸篓 (Trash) 冲突项与解除 LaunchServices 废纸篓锁定
echo -e "🧹 正在深度清理废纸篓冲突与反注册 LaunchServices / Purging Trash & LaunchServices..."
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

# 执行废纸篓物理删除与注销
for ta in "${TRASHED_APPS[@]}"; do
    if [ -n "$ta" ] && { [ -e "$ta" ] || [ -d "$ta" ]; }; then
        [ -n "$LSREGISTER" ] && "$LSREGISTER" -u "$ta" 2>/dev/null || true
        chflags -R nouchg,noschg "$ta" 2>/dev/null || true
        rm -rf "$ta" 2>/dev/null || sudo rm -rf "$ta" 2>/dev/null || true
    fi
done

# 运行 LaunchServices 垃圾回收以清除死路径 (跨 user, system, local 域)
if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    "$LSREGISTER" -gc -R -v -apps u,s,l 2>/dev/null || "$LSREGISTER" -gc 2>/dev/null || true
fi

# 4. 定位源应用程序包
SOURCE_APP=""
if [ -d "$DIR/${APP_NAME}.app" ]; then
    SOURCE_APP="$DIR/${APP_NAME}.app"
elif [ -d "$DIR/Antigravity Tools.app" ]; then
    SOURCE_APP="$DIR/Antigravity Tools.app"
elif [ -d "$DIR/agm-alim.app" ]; then
    SOURCE_APP="$DIR/agm-alim.app"
else
    SOURCE_APP=$(find "$DIR" -maxdepth 2 -name "*.app" -type d 2>/dev/null | head -n 1)
fi

if [ -z "$SOURCE_APP" ] || [ ! -d "$SOURCE_APP" ]; then
    echo -e "${RED}❌ 未在安装包中找到应用程序 / Source application bundle not found in $DIR${NC}"
    dump_diagnostic_state ""
    exit 1
fi

echo -e "📦 发现源应用包 / Source package: ${BLUE}$SOURCE_APP${NC}"

# 5. 目标路径选择 (优先 /Applications，无权限退回 $HOME/Applications)
DEST_DIR="/Applications"
if [ ! -d "$DEST_DIR" ] || [ ! -w "$DEST_DIR" ]; then
    DEST_DIR="$HOME/Applications"
    mkdir -p "$DEST_DIR" 2>/dev/null || true
fi
TARGET_APP="${DEST_DIR}/${APP_NAME}.app"

echo -e "🚀 正在安装到 / Installing to: ${CYAN}$TARGET_APP${NC}..."

# 清除目标位置已有版本与注销
if [ -d "$TARGET_APP" ]; then
    [ -n "$LSREGISTER" ] && "$LSREGISTER" -u "$TARGET_APP" 2>/dev/null || true
    rm -rf "$TARGET_APP" 2>/dev/null || sudo rm -rf "$TARGET_APP" 2>/dev/null || true
fi

# 复制新版本
cp -R "$SOURCE_APP" "$TARGET_APP" 2>/dev/null || sudo cp -R "$SOURCE_APP" "$TARGET_APP"

# 清理遗留历史包名
for legacy in "Antigravity Tools.app" "Anti-Gravity Tools.app" "agm-alim.app"; do
    if [ -d "${DEST_DIR}/${legacy}" ] && [ "${DEST_DIR}/${legacy}" != "$TARGET_APP" ]; then
        rm -rf "${DEST_DIR}/${legacy}" 2>/dev/null || true
    fi
done

# 6. 全面净化隔离属性并配置权限
echo -e "🔧 正在净化隔离属性与授予执行权限 / Stripping quarantine attributes & setting permissions..."
chmod -R u+rwX "$TARGET_APP" 2>/dev/null || sudo chmod -R u+rwX "$TARGET_APP" 2>/dev/null || true
if [ -d "$TARGET_APP/Contents/MacOS" ]; then
    chmod -R +x "$TARGET_APP/Contents/MacOS" 2>/dev/null || sudo chmod -R +x "$TARGET_APP/Contents/MacOS" 2>/dev/null || true
fi

xattr -cr "$TARGET_APP" 2>/dev/null || true
xattr -rd com.apple.quarantine "$TARGET_APP" 2>/dev/null || true
xattr -rd com.apple.provenance "$TARGET_APP" 2>/dev/null || true
find "$TARGET_APP" -exec xattr -c {} + 2>/dev/null || true
find "$TARGET_APP" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true
find "$TARGET_APP" -exec xattr -d com.apple.provenance {} + 2>/dev/null || true

# 7. 注册 Gatekeeper 与本地自签名
macos_major=0
if command -v sw_vers &>/dev/null; then
    macos_major=$(sw_vers -productVersion | cut -d. -f1)
fi

if [[ "$macos_major" -ge 13 ]] && command -v spctl &>/dev/null; then
    echo -e "🔐 向 Gatekeeper 注册信任 / Registering with Gatekeeper..."
    spctl --add "$TARGET_APP" 2>/dev/null || sudo spctl --add "$TARGET_APP" 2>/dev/null || true
fi

if command -v codesign &>/dev/null; then
    echo -e "🔏 补充本地自签名 / Applying local ad-hoc codesign..."
    codesign --force --deep --sign - "$TARGET_APP" 2>/dev/null || sudo codesign --force --deep --sign - "$TARGET_APP" 2>/dev/null || true
fi

# 8. 重构并刷新 LaunchServices
if [ -n "$LSREGISTER" ] && [ -x "$LSREGISTER" ]; then
    echo -e "🔄 刷新 LaunchServices 应用程序数据库 (全域重置) / Refreshing LaunchServices..."
    "$LSREGISTER" -u "$TARGET_APP" 2>/dev/null || true
    "$LSREGISTER" -gc -R -v -apps u,s,l 2>/dev/null || "$LSREGISTER" -gc 2>/dev/null || true
    "$LSREGISTER" -f -r "$TARGET_APP" 2>/dev/null || true
    killall Finder Dock 2>/dev/null || true
    sleep 1
fi

# 9. 配置命令行 CLI 软链接
MACOS_BIN_DIR="${TARGET_APP}/Contents/MacOS"
if [ -d "$MACOS_BIN_DIR" ]; then
    CLI_BIN=""
    for candidate_name in "agm" "agm-alim" "${APP_NAME}" "Antigravity Tools" "antigravity-tools"; do
        if [ -x "${MACOS_BIN_DIR}/${candidate_name}" ]; then
            CLI_BIN="${MACOS_BIN_DIR}/${candidate_name}"
            break
        fi
    done
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
echo -e "${GREEN}   🎉 安装与修复成功! / Installation Complete!${NC}"
echo -e "${GREEN}====================================================${NC}"
echo "应用程序位置 / Installed to: $TARGET_APP"

osascript -e "display notification \"安装与修复成功，正在启动...\" with title \"$APP_NAME\" sound name \"Glass\"" 2>/dev/null || true

# 10. 启动应用
launch_installed_application "$TARGET_APP" "$APP_NAME" "$LSREGISTER"

echo ""
echo "详细运行日志与诊断信息保存在 / Log file: $LOG_FILE"
echo "按任意键退出 / Press any key to exit..."
read -n 1 -s -r -p ""
echo ""
