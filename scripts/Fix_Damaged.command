#!/bin/bash
set -E

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
    echo -e "\033[1;33m   [SYSTEM CONTEXT] OS: ${sys_info} | Date: $(date -u +"%Y-%m-%dT%H:%M:%SZ" 2>/dev/null || date)\033[0m" >&2
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

# 获取当前脚本所在目录
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_NAME="Antigravity Manager Tools"

# 定义颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

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

# 2. 检查废纸篓 (Trash) 是否存在被隔离或被重命名的残留应用 (如 "... 13-20-55-339.app")
echo -e "🔍 正在检查废纸篓残留 / Inspecting Trash for quarantined items..."
TRASHED_APPS=()
while IFS= read -r t_app; do
    if [ -n "$t_app" ] && [ -d "$t_app" ]; then
        TRASHED_APPS+=("$t_app")
    fi
done < <(find "$HOME/.Trash" -maxdepth 2 \( -iname "*antigravity*.app" -o -iname "*agm*.app" \) -type d 2>/dev/null || true)

if [ ${#TRASHED_APPS[@]} -gt 0 ]; then
    echo -e "${YELLOW}⚠️  在废纸篓中发现 ${#TRASHED_APPS[@]} 个被误移入的应用副本 / Found trashed app bundle(s):${NC}"
    for ta in "${TRASHED_APPS[@]}"; do
        echo -e "   🗑️  $ta"
    done
fi

# 3. 动态查找有效应用程序路径
APP_PATH=""

# 3.1 优先在系统或用户 Applications 目录查找已安装版本
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

# 3.2 如果未在系统目录找到，在当前脚本同级目录查找 (例如 DMG 卷或解压目录)
if [ -z "$APP_PATH" ]; then
    if [ -d "$DIR/${APP_NAME}.app" ]; then
        APP_PATH="$DIR/${APP_NAME}.app"
    elif [ -d "$DIR/Antigravity Tools.app" ]; then
        APP_PATH="$DIR/Antigravity Tools.app"
    elif [ -d "$DIR/agm-alim.app" ]; then
        APP_PATH="$DIR/agm-alim.app"
    else
        LOCAL_APP=$(find "$DIR" -maxdepth 2 -name "*.app" -type d 2>/dev/null | head -n 1)
        if [ -n "$LOCAL_APP" ] && [ -d "$LOCAL_APP" ]; then
            APP_PATH="$LOCAL_APP"
        fi
    fi
fi

# 3.3 兜底模糊匹配 Applications 目录
if [ -z "$APP_PATH" ]; then
    SEARCH_APP=$(find /Applications "$HOME/Applications" -maxdepth 2 \( -iname "*antigravity*.app" -o -iname "*agm*.app" \) -type d 2>/dev/null | head -n 1)
    if [ -n "$SEARCH_APP" ] && [ -d "$SEARCH_APP" ]; then
        APP_PATH="$SEARCH_APP"
    fi
fi

# 3.4 如果系统中完全未找到任何已安装应用，但废纸篓中有被移入的应用，自动尝试从废纸篓恢复
if [ -z "$APP_PATH" ] && [ ${#TRASHED_APPS[@]} -gt 0 ]; then
    RESTORE_SOURCE="${TRASHED_APPS[0]}"
    TARGET_DEST_DIR="/Applications"
    if [ ! -w "$TARGET_DEST_DIR" ]; then
        TARGET_DEST_DIR="$HOME/Applications"
        mkdir -p "$TARGET_DEST_DIR" 2>/dev/null || true
    fi
    RESTORED_TARGET="${TARGET_DEST_DIR}/${APP_NAME}.app"
    echo -e "📦 正在从废纸篓恢复应用到 ${TARGET_DEST_DIR}... / Restoring app from Trash..."
    rm -rf "$RESTORED_TARGET" 2>/dev/null || true
    if mv "$RESTORE_SOURCE" "$RESTORED_TARGET" 2>/dev/null; then
        APP_PATH="$RESTORED_TARGET"
        echo -e "${GREEN}✅ 已成功恢复 / Successfully restored: $APP_PATH${NC}"
        # 从数组中移除已恢复的条目
        TRASHED_APPS=("${TRASHED_APPS[@]:1}")
    fi
fi

# 4. 清理废纸篓中剩余的冲突残留副本，防止 LaunchServices 继续指向 Trash
if [ ${#TRASHED_APPS[@]} -gt 0 ]; then
    echo -e "🧹 正在清理废纸篓冲突副本以避免 LaunchServices 锁定 / Purging lingering trashed copies..."
    for ta in "${TRASHED_APPS[@]}"; do
        if [ -n "$LSREGISTER" ]; then
            "$LSREGISTER" -u "$ta" 2>/dev/null || true
        fi
        chflags -R nouchg "$ta" 2>/dev/null || true
        rm -rf "$ta" 2>/dev/null || sudo rm -rf "$ta" 2>/dev/null || true
        echo -e "   🗑️  已移除冲突副本 / Removed: $(basename "$ta")"
    done
fi

# 5. 如果当前运行在 DMG 卷内，且系统 Applications 未安装，协助复制到 Applications
if [[ "$APP_PATH" == /Volumes/* && ! -d "/Applications/${APP_NAME}.app" && ! -d "$HOME/Applications/${APP_NAME}.app" ]]; then
    TARGET_DEST="/Applications"
    if [ ! -w "$TARGET_DEST" ]; then
        TARGET_DEST="$HOME/Applications"
        mkdir -p "$TARGET_DEST" 2>/dev/null || true
    fi
    echo -e "🚀 正在将应用安装到 ${TARGET_DEST}... / Copying app to ${TARGET_DEST}..."
    if cp -R "$APP_PATH" "${TARGET_DEST}/"; then
        APP_PATH="${TARGET_DEST}/$(basename "$APP_PATH")"
        echo -e "${GREEN}✅ 安装完成 / Installed: $APP_PATH${NC}"
    fi
fi

# 6. 执行权限与隔离修复
if [ -n "$APP_PATH" ] && [ -d "$APP_PATH" ]; then
    TARGET_NAME=$(basename "$APP_PATH" .app)
    echo -e "📍 目标应用路径 / Target application path: ${BLUE}$APP_PATH${NC}"

    macos_major=0
    if command -v sw_vers &>/dev/null; then
        macos_major=$(sw_vers -productVersion | cut -d. -f1)
    fi

    echo "🔧 正在清除隔离属性 / Clearing quarantine attributes..."
    # 先尝试用户级清除隔离属性
    chmod -R u+rwX "$APP_PATH" 2>/dev/null || true
    if [ -d "$APP_PATH/Contents/MacOS" ]; then
        chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || true
    fi
    find "$APP_PATH" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true
    xattr -cr "$APP_PATH" 2>/dev/null || true
    xattr -d com.apple.quarantine "$APP_PATH" 2>/dev/null || true

    # 检查是否仍有隔离属性，若有则提示 sudo
    if xattr -r "$APP_PATH" 2>/dev/null | grep -q "com.apple.quarantine"; then
        echo -e "${YELLOW}提示: 当前目录权限受限，请输入开机密码授权 (输入时不显示):${NC}"
        echo -e "${YELLOW}Note: Permissions restricted. Please enter login password (keystrokes hidden):${NC}"
        sudo chmod -R u+rwX "$APP_PATH" 2>/dev/null || true
        sudo chmod -R +x "$APP_PATH/Contents/MacOS" 2>/dev/null || true
        sudo find "$APP_PATH" -exec xattr -d com.apple.quarantine {} + 2>/dev/null || true
        sudo xattr -cr "$APP_PATH" 2>/dev/null || true
        sudo xattr -d com.apple.quarantine "$APP_PATH" 2>/dev/null || true
    fi
    echo -e "${GREEN}✅ 隔离属性已清除 / Quarantine cleared.${NC}"

    # 向 Gatekeeper 注册 (macOS 13+ Ventura / Sonoma / Sequoia)
    if [[ "$macos_major" -ge 13 ]] && command -v spctl &>/dev/null; then
        echo "🔐 正在向看门人注册应用 / Registering app with Gatekeeper (spctl)..."
        spctl --add "$APP_PATH" 2>/dev/null || sudo spctl --add "$APP_PATH" 2>/dev/null || true
        echo -e "${GREEN}✅ 看门人注册完成 / Gatekeeper registration done.${NC}"
    fi

    # 应用本地自签名 (macOS 15 以前版本应用)
    if [[ "$macos_major" -lt 15 ]] && command -v codesign &>/dev/null; then
        echo "🔏 正在应用本地自签名 / Applying local ad-hoc codesign..."
        codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || sudo codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || true
        echo -e "${GREEN}✅ 本地签名完成 / Ad-hoc codesign done.${NC}"
    fi

    # 重建 LaunchServices 数据库，确保 LaunchServices 刷新应用注册并不再指向废纸篓
    if [ -n "$LSREGISTER" ]; then
        echo "🔄 正在刷新 LaunchServices 应用程序数据库 / Refreshing LaunchServices database..."
        "$LSREGISTER" -f "$APP_PATH" 2>/dev/null || true
        "$LSREGISTER" -kill -r -domain local -domain system -domain user 2>/dev/null || true
        killall Finder 2>/dev/null || true
        killall Dock 2>/dev/null || true
        echo -e "${GREEN}✅ LaunchServices 数据库已刷新 / LaunchServices refreshed.${NC}"
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

            # 确保 $HOME/.local/bin 存在于 shell 配置文件中
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
    echo -e "${GREEN}   ✅ 修复成功! / Repair complete!${NC}"
    echo -e "${GREEN}====================================================${NC}"
    echo "您现在可以像往常一样打开 $TARGET_NAME 了。"
    echo "You can now open $TARGET_NAME normally."

    # 尝试通过 AppleScript 弹窗通知
    osascript -e "display notification \"修复成功，现在可以正常打开应用了\" with title \"$TARGET_NAME\" sound name \"Glass\"" 2>/dev/null || true

    echo ""
    echo -e "🚀 正在尝试直接启动应用 / Launching ${TARGET_NAME}..."
    local open_output
    if open_output=$(open "$APP_PATH" 2>&1); then
        echo -e "${GREEN}🎉 应用已成功启动! / ${TARGET_NAME} launched successfully!${NC}"
    else
        echo -e "${YELLOW}⚠️  启动提示 / Launch notice: ${open_output}${NC}"
        echo "您也可以直接在访达中打开: $APP_PATH"
    fi
else
    echo -e "${RED}⚠️  未找到应用文件 / Application bundle not found${NC}"
    echo "请确保将 '${APP_NAME}.app' 放置在 /Applications 或与本修复脚本位于同一目录下。"
    echo "Please ensure '${APP_NAME}.app' is in /Applications or in the same folder as this script."
fi

echo ""
echo "按任意键退出 / Press any key to exit..."
read -n 1 -s -r -p ""
echo ""
