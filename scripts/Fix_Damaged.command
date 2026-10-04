#!/bin/bash

# 获取当前脚本所在目录
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"
APP_NAME="Antigravity Manager Tools"

# 定义颜色
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${GREEN}==============================================${NC}"
echo -e "${GREEN}   ${APP_NAME} - 快速修复助手 / Quick Repair${NC}"
echo -e "${GREEN}==============================================${NC}"
echo ""

# 动态查找应用路径
APP_PATH=""

# 1. 优先在当前脚本所在目录查找
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

# 2. 如果未在当前目录找到，在系统或用户 Applications 目录查找
if [ -z "$APP_PATH" ]; then
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
fi

# 3. 兜底模糊匹配 Applications 目录下的相关应用
if [ -z "$APP_PATH" ]; then
    SEARCH_APP=$(find /Applications "$HOME/Applications" -maxdepth 2 \( -iname "*antigravity*.app" -o -iname "*agm*.app" \) -type d 2>/dev/null | head -n 1)
    if [ -n "$SEARCH_APP" ] && [ -d "$SEARCH_APP" ]; then
        APP_PATH="$SEARCH_APP"
    fi
fi

if [ -n "$APP_PATH" ] && [ -d "$APP_PATH" ]; then
    TARGET_NAME=$(basename "$APP_PATH" .app)
    echo -e "📍 发现应用 / Discovered app: ${BLUE}$APP_PATH${NC}"
    echo "🔑 正在清除 Gatekeeper 隔离属性并重署签名..."
    echo "🔑 Clearing Gatekeeper quarantine attributes & applying ad-hoc signature..."
    echo ""

    # 先尝试用户级清除隔离属性 (无需 sudo)
    xattr -cr "$APP_PATH" 2>/dev/null || true
    xattr -r -d com.apple.quarantine "$APP_PATH" 2>/dev/null || true

    # 检查是否仍有隔离属性，若有则使用 sudo
    if xattr -r "$APP_PATH" 2>/dev/null | grep -q "com.apple.quarantine"; then
        echo "提示: 当前目录权限受限，请输入开机密码授权 (输入时不显示):"
        echo "Note: Permissions restricted. Please enter login password (keystrokes hidden):"
        sudo xattr -cr "$APP_PATH" 2>/dev/null || true
        sudo xattr -rd com.apple.quarantine "$APP_PATH" 2>/dev/null || true
    fi

    # 应用本地临时签名
    if command -v codesign &>/dev/null; then
        echo "正在应用本地自签名 (codesign)..."
        echo "Applying local ad-hoc codesign..."
        codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || sudo codesign --force --deep --sign - "$APP_PATH" 2>/dev/null || true
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
            USER_BIN="$HOME/.local/bin"
            mkdir -p "$USER_BIN" 2>/dev/null || true
            ln -sf "$CLI_BIN" "${USER_BIN}/agm" 2>/dev/null || true
            ln -sf "$CLI_BIN" "${USER_BIN}/agm-alim" 2>/dev/null || true
            echo "🔗 已配置命令行工具 / Configured CLI symlinks: ${USER_BIN}/agm, ${USER_BIN}/agm-alim"

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
    echo -e "${GREEN}✅ 修复成功! / Repair complete!${NC}"
    echo "您现在可以像往常一样打开 $TARGET_NAME 了。"
    echo "You can now open $TARGET_NAME normally."

    # 尝试通过 AppleScript 弹窗通知
    osascript -e "display notification \"修复成功，现在可以正常打开应用了\" with title \"$TARGET_NAME\" sound name \"Glass\"" 2>/dev/null || true
else
    echo -e "${RED}⚠️  未找到应用文件 / Application bundle not found${NC}"
    echo "请确保将 '${APP_NAME}.app' 放置在 /Applications 或与本修复脚本位于同一目录下。"
    echo "Please ensure '${APP_NAME}.app' is in /Applications or in the same folder as this script."
fi

echo ""
echo "按任意键退出 / Press any key to exit..."
read -n 1 -s -r -p ""
echo ""
