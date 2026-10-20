# Antigravity Tools 🚀
> Professional AI Account Management & Protocol Proxy System (v4.241.0)

<div align="center">
  <img src="public/icon.png" alt="Antigravity Logo" width="120" height="120" style="border-radius: 24px; box-shadow: 0 10px 30px rgba(0,0,0,0.15);">

  <h3>Your Personal High-Performance AI Gateway</h3>
  <p>Not just account management, but the ultimate solution to break API barriers.</p>
  <p><em>Managed and enhanced by <strong>Md. Alim Ul Karim</strong> · Originally created by <strong>lbjlaq</strong> and upstream contributors</em></p>

  <p>
    <a href="https://github.com/alimtvnetwork/Antigravity-Manager/releases">
      <img src="https://img.shields.io/badge/Version-4.241.0-blue?style=flat-square" alt="Version">
    </a>
    <img src="https://img.shields.io/badge/Tauri-v2-orange?style=flat-square" alt="Tauri">
    <img src="https://img.shields.io/badge/Backend-Rust-red?style=flat-square" alt="Rust">
    <img src="https://img.shields.io/badge/Frontend-React-61DAFB?style=flat-square" alt="React">
    <img src="https://img.shields.io/badge/License-CC--BY--NC--SA--4.0-lightgrey?style=flat-square" alt="License">
  </p>

  <p>
    <a href="#-features">Features</a> •
    <a href="#-gui-overview">GUI Overview</a> •
    <a href="#-architecture">Architecture</a> •
    <a href="#-installation">Installation</a> •
    <a href="#-quick-integration">Integration</a>
  </p>

  <p>
    <strong>English</strong>
  </p>
</div>

---

**Antigravity Tools** is an all-in-one desktop application designed for developers and AI enthusiasts. It perfectly combines multi-account management, protocol conversion, and smart request scheduling to provide you with a stable, high-speed, and low-cost **Local AI Relay Station**.

By leveraging this app, you can transform common Web Sessions (Google/Anthropic) into standardized API interfaces, completely eliminating the protocol gap between different providers.

## 💖 Sponsors

| Sponsor | Description |
| :---: | :--- |
| <img src="docs/images/packycode_logo.png" width="200" alt="PackyCode Logo"> | Thanks to **PackyCode** for sponsoring this project! PackyCode is a reliable and efficient API relay service provider, offering relays for various services such as Claude Code, Codex, and Gemini. PackyCode provides a special offer for users of this project: Register using [this link](https://www.packyapi.com/register?aff=Ctrler) and enter the **"Ctrler"** coupon code when topping up to enjoy a **10% discount**. |
| <img src="docs/images/APIKEYFUN.png" width="200" alt="APIKEYFUN Logo"> | Thanks to **APIKEY.FUN** for sponsoring this project! APIKEY.FUN is a professional enterprise-grade AI relay station, dedicated to providing stable, efficient, and low-cost AI model API access services for enterprise and individual developers. The platform supports mainstream popular models such as Claude, OpenAI, and Gemini, with prices as low as 7% of the official original price. Register through [this exclusive link](https://apikey.fan/register?aff=Ctrler) for this project to enjoy an exclusive offer of up to **permanent 5% off on top-ups**. |
| <img src="docs/images/claudeapilogo.png" width="200" alt="Claude API Logo"> | Thanks to **Claude API** for supporting this project! claudeapi.com is a **Claude API** relay station built on **official and AWS channels**, focused exclusively on Claude, delivering high stability and low latency with full support for Claude Code. Exclusive offer: register via this [exclusive link](https://console.claudeapi.com/register?source=antigravity) to get **free trial credits — zero setup, get started instantly**; enjoy an extra **5% off** when you top up (Contact Support). |
| <img src="docs/images/AICodeMirror.jpg" width="200" alt="AICodeMirror Logo"> | Thanks to **AICodeMirror** for sponsoring this project! AICodeMirror provides official high-stability relay services for Claude Code / Codex / Gemini CLI, supporting enterprise-grade concurrency, fast invoicing, and 24/7 dedicated technical support. Claude Code / Codex / Gemini official channels at 38% / 2% / 9% of original price, with extra discounts on top-ups! AICodeMirror offers special benefits for Antigravity-Manager users: register via [this link](https://aicodemirror.ai/register?invitecode=MV5XUM) to enjoy 20% off your first top-up, and enterprise customers can get up to 25% off! |

### ☕ Support

If you find this project helpful, feel free to buy me a coffee!

<a href="https://www.buymeacoffee.com/Ctrler" target="_blank"><img src="https://cdn.buymeacoffee.com/buttons/v2/default-green.png" alt="Buy Me A Coffee" style="height: 60px !important; width: 217px !important;"></a>

| Alipay | WeChat Pay | Buy Me a Coffee |
| :---: | :---: | :---: |
| ![Alipay](./docs/images/donate_alipay.png) | ![WeChat](./docs/images/donate_wechat.png) | ![Coffee](./docs/images/donate_coffee.png) |

## 🚀 Recommended Projects

If you like this project, you might also be interested in:

*   **[Antigravity-Tools-LS](https://github.com/lbjlaq/Antigravity-Tools-LS)**: A Language Server Protocol (LSP) designed for AI protocols, providing you with smarter code completion, diagnostics, and protocol debugging experiences.

## 🌟 Detailed Feature Matrix

### 1. 🎛️ Smart Account Dashboard
*   **Global Real-time Monitoring**: Instant insight into the health of all accounts, including average remaining quotas for Gemini Pro, Gemini Flash, Claude, and Gemini Image generation.
*   **Smart Recommendation**: The system uses a real-time algorithm to filter and recommend the "Best Account" based on quota redundancy, supporting **one-click switching**.
*   **Active Account Snapshot**: Visually displays the specific quota percentage and the last synchronization time of the currently active account.

### 2. 🔐 Professional AI Account Management & Proxy System
*   **OAuth 2.0 Authorization (Auto/Manual)**: Pre-generates a copyable authorization URL so you can finish auth in any browser; after the callback, the app auto-completes and saves the account (use “I already authorized, continue” if needed).
*   **Multi-dimensional Import**: Supports single token entry, JSON batch import, and automatic hot migration from V1 legacy databases.
*   **Gateway-level Views**: Supports switching between "List" and "Grid" views. Provides 403 Forbidden detection, automatically marking and skipping accounts with permission anomalies.

### 3.  Protocol Conversion & Relay (API Proxy)
*   **Multi-Protocol Adaptation (Multi-Sink)**:
    *   **OpenAI Format**: Provides `/v1/chat/completions` endpoint, compatible with 99% of existing AI apps.
    *   **Anthropic Format**: Provides native `/v1/messages` interface, supporting all features of **Claude Code CLI** (e.g., chain-of-thought, system prompts).
    *   **Gemini Format**: Supports direct calls from official Google AI SDKs.
*   **Smart Self-healing**: When a request encounters `429 (Too Many Requests)` or `401 (Expired)`, the backend triggers **millisecond-level automatic retry and silent rotation**, ensuring business continuity.

### 4. 🔀 Model Router Center
*   **Series-based Mapping**: Classify complex original model IDs into "Series Groups" (e.g., routing all GPT-4 requests uniformly to `gemini-3-pro-high`).
*   **Expert Redirection**: Supports custom regex-level model mapping for precise control over every request's landing model.
*   **Tiered Routing [New]**: Automatically prioritizes models based on account tiers (Ultra/Pro/Free) and reset frequencies to ensure stability for high-volume users.
*   **Silent Background Downgrading [New]**: Intelligently identifies background tasks (e.g., Claude CLI title generation) and reroutes them to Flash models to preserve premium quota.

### 5. 🎨 Multimodal & Imagen 3 Support
*   **Advanced Image Control**: Supports precise control over image generation tasks via OpenAI `size` (e.g., `1024x1024`, `16:9`) parameters or model name suffixes.
*   **Enhanced Payload Support**: The backend supports payloads up to **100MB** (configurable), more than enough for 4K HD image recognition and processing.

##  GUI Overview

| | |
| :---: | :---: |
| ![Dashboard - Global Quota Monitoring & One-click Switch](docs/images/dashboard-light.png) <br> Dashboard | ![Account List - High-density Quota Display & Smart 403 Labeling](docs/images/accounts-light.png) <br> Account List |
| ![About Page - About Antigravity Tools](docs/images/dashboard-modern.png) <br> Modern Dashboard | ![API Proxy - Service Control](docs/images/v3/proxy-settings.png) <br> API Proxy |
| ![Settings - General Config](docs/images/settings-dark.png) <br> Settings | |

### 💡 Usage Examples

| | |
| :---: | :---: |
| ![Claude Code Web Search - Structured source and citation display](docs/images/usage/claude-code-search.png) <br> Claude Code Web Search | ![Cherry Studio Deep Integration - Native echo of search citations and source links](docs/images/usage/cherry-studio-citations.png) <br> Cherry Studio Integration |
| ![Imagen 3 Advanced Drawing - Perfect restoration of Prompt artistic conception and details](docs/images/usage/image-gen-nebula.png) <br> Imagen 3 Advanced Drawing | ![Kilo Code Integration - Multi-account high-speed rotation and model penetration](docs/images/usage/kilo-code-integration.png) <br> Kilo Code Integration |

## 🏗️ Architecture

```mermaid
graph TD
    Client([External Apps: Claude Code/NextChat]) -->|OpenAI/Anthropic| Gateway[Antigravity Axum Server]
    Gateway --> Middleware[Middleware: Auth/Rate Limit/Logs]
    Middleware --> Router[Model Router: ID Mapping]
    Router --> Dispatcher[Dispatcher: Rotation/Weights]
    Dispatcher --> Mapper[Request Mapper]
    Mapper --> Upstream[Upstream: Google/Anthropic API]
    Upstream --> ResponseMapper[Response Mapper]
    ResponseMapper --> Client
```

## 📥 Installation

### Option A: Terminal Installation (Recommended)

#### Cross-Platform One-Line Install Scripts

Automatically detects your OS, architecture, and package manager — one command to download and install.

#### Windows (PowerShell 5.1+)

**Bar 1: Latest Version (Auto-Updating)**
```powershell
irm https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.ps1 | iex
```

**Bar 2: Version-Based Installation (v4.72.0)**
```powershell
& ([scriptblock]::Create((irm https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.72.0/install.ps1))) -Version "4.72.0"
```

---

#### Linux & macOS (Bash)

**Bar 1: Latest Version (Auto-Updating)**
```bash
curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash
```

**Bar 2: Version-Based Installation (v4.72.0)**
```bash
curl -fsSL https://github.com/alimtvnetwork/Antigravity-Manager/releases/download/v4.72.0/install.sh | bash -s -- --version "4.72.0"
```

> **Supported formats**: Linux (`.deb` / `.rpm` / `.AppImage`) | macOS (`.dmg`) | Windows (NSIS `.exe`)
>
> **Advanced usage**: Install a specific version `curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --version 4.66.0`, dry-run mode `curl -fsSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/install.sh | bash -s -- --dry-run`

#### macOS - Homebrew
If you have [Homebrew](https://brew.sh/) installed, you can also install via:

```bash
# 1. Tap the repository
brew tap alimtvnetwork/antigravity-manager https://github.com/alimtvnetwork/Antigravity-Manager

# 2. Install the app
brew install --cask antigravity-tools
```

#### Arch Linux
You can choose to install via the one-click script or Homebrew:

**Option 1: One-click script (Recommended)**
```bash
curl -sSL https://raw.githubusercontent.com/alimtvnetwork/Antigravity-Manager/main/deploy/arch/install.sh | bash
```

**Option 2: via Homebrew** (If you have [Linuxbrew](https://sh.brew.sh/) installed)
```bash
brew tap alimtvnetwork/antigravity-manager https://github.com/alimtvnetwork/Antigravity-Manager
brew install --cask antigravity-tools
```

#### Other Linux Distributions
The AppImage will be automatically symlinked to your binary path with executable permissions.

### Option B: Manual Download
Download from [GitHub Releases](https://github.com/alimtvnetwork/Antigravity-Manager/releases):
*   **macOS**: `.dmg` (Universal, Apple Silicon & Intel)
*   **Windows**: `.msi` or portable `.zip`
*   **Linux**: `.deb` or `AppImage`

### Option C: Docker Deployment (Recommended for NAS/Servers)
If you prefer running in a containerized environment, we provide a native Docker image. This image supports the v4.0.3 Native Headless architecture, automatically hosts frontend static resources, and allows for direct browser-based management.

#### Option 1: Direct Run (Recommended)
- **API_KEY**: Required. Used for AI request authentication.
- **WEB_PASSWORD**: Optional. Used for Web UI login. Defaults to API_KEY if NOT set.

```bash
docker run -d --name antigravity-manager \
  -p 8045:8045 \
  -e API_KEY=sk-your-api-key \
  -e WEB_PASSWORD=your-login-password \
  -e ABV_MAX_BODY_SIZE=104857600 \
  -v ~/.antigravity_tools:/root/.antigravity_tools \
  alimtvnetwork/antigravity-manager:latest

# Forgot keys? Run `docker logs antigravity-manager` or `grep -E '"api_key"|"admin_password"' ~/.antigravity_tools/gui_config.json`
```

> [!TIP]
> **🧪 Pulling Beta / Preview Images**:
> To test the latest Beta pre-release features, specify the corresponding Beta version tag (pre-releases are published independently and will never overwrite the `latest` stable tag):
> ```bash
> # Pull a specific Beta pre-release version (check Docker Hub for all tags)
> docker pull lbjlaq/antigravity-manager:v4.8.2-beta.0
> 
> # Run Beta container
> docker run -d --name antigravity-manager-beta \
>   -p 8045:8045 \
>   -e API_KEY=sk-your-api-key \
>   -e WEB_PASSWORD=your-login-password \
>   -e ABV_MAX_BODY_SIZE=104857600 \
>   -v ~/.antigravity_tools:/root/.antigravity_tools \
>   lbjlaq/antigravity-manager:v4.8.2-beta.0
> ```
> View all published Beta tags on [Docker Hub Tags](https://hub.docker.com/r/lbjlaq/antigravity-manager/tags). If you wish to run the unreleased bleeding-edge `beta` branch directly, build locally: `docker build -t lbjlaq/antigravity-manager:beta -f docker/Dockerfile .`.

#### 🔐 Authentication Scenarios
*   **Scenario A: Only `API_KEY` is set**
    - **Web Login**: Use `API_KEY` to access the dashboard.
    - **API Calls**: Use `API_KEY` for AI request authentication.
*   **Scenario B: Both `API_KEY` and `WEB_PASSWORD` are set (Recommended)**
    - **Web Login**: **Must** use `WEB_PASSWORD`. Using API Key will be rejected (more secure).
    - **API Calls**: Continue to use `API_KEY`. This allows you to share the API Key with team members while keeping the password for administrative access only.

#### 🆙 Upgrade Guide for Older Versions
If you are upgrading from v4.0.1 or earlier, your installation won't have a `WEB_PASSWORD` set by default. You can add one using any of these methods:
1.  **Web UI (Recommended)**: Log in using your existing `API_KEY`, go to the **API Proxy Settings** page, find the **Web UI Management Password** section below the API Key, set your new password, and save.
2.  **Environment Variable (Docker)**: Stop the old container and start the new one with the added parameter `-e WEB_PASSWORD=your_new_password`. **Note: Environment variables have the highest priority and will override any changes in the UI.**
3.  **Config File (Persistent)**: Directly edit `~/.antigravity_tools/gui_config.json` and add/modify `"admin_password": "your_new_password"` inside the `proxy` object.
    - *Note: `WEB_PASSWORD` is the environment variable name, while `admin_password` is the JSON key in the config file.*

> [!TIP]
> **Priority Logic**:
> - **Environment Variable** (`WEB_PASSWORD`) has the highest priority. If set, the application will always use it and ignore values in the configuration file.
> - **Configuration File** (`gui_config.json`) is used for persistent storage. When you change the password via Web UI and save, it is written here.
> - **Fallback**: If neither is set, it falls back to `API_KEY`; if even `API_KEY` is missing, a random one is generated.

#### Option 2: Use Docker Compose
1. Enter the `docker` directory and start the service:
```bash
cd docker
docker compose up -d
```
> **Log rotation**: Compose limits JSON logs to `100m` per file and keeps `3` files by default to prevent unbounded growth.
> **Access URL**: `http://localhost:8045` (Admin Console) | `http://localhost:8045/v1` (API Base)
> **System Requirements**:
> - **RAM**: **1GB** recommended (minimum 256MB).
> - **Persistence**: Mount `/root/.antigravity_tools` to persist your data.
> - **Architecture**: Supports x86_64 and ARM64.
> **See**: [Docker Deployment Guide (docker)](./docker/README.md)

<details>
<summary><b>🛠️ Troubleshooting - Click to expand</b></summary>

#### macOS says "App is damaged"?
Due to macOS security gatekeeper, non-App Store apps might show this. Run this in Terminal to fix:
```bash
sudo xattr -rd com.apple.quarantine "/Applications/Antigravity Tools.app"
```

#### Linux window is black or empty?
On niri, Hyprland, Sway, and similar compositors, older builds forced `GDK_BACKEND=x11` whenever `DISPLAY` was set, and WebKit then drew a black window. Update to a build that includes this fix, or launch once with:

```bash
env WEBKIT_DISABLE_DMABUF_RENDERER=1 ANTIGRAVITY_FORCE_WAYLAND=1 antigravity-tools
```

- `ANTIGRAVITY_FORCE_WAYLAND=1`: keep native Wayland (do not force X11)
- `ANTIGRAVITY_FORCE_X11=1`: force X11 if you still need it
- `WEBKIT_DISABLE_DMABUF_RENDERER=1`: disable the WebKit DMA-BUF renderer

</details>

## 🔌 Quick Integration Examples

### 🔐 OAuth Authorization Flow (Add Account)
1. Go to `Accounts` → `Add Account` → `OAuth`.
2. The dialog pre-generates an authorization URL before you click any button. Click the URL to copy it to the system clipboard, then open it in the browser you prefer and complete authorization.
3. After consent, the browser opens a local callback page and shows “✅ Authorized successfully!”.
4. The app automatically continues the flow and saves the account; if it doesn’t, click “I already authorized, continue” to finish manually.

> Note: the auth URL contains a one-time local callback port. Always use the latest URL shown in the dialog. If the app isn’t running or the dialog is closed during auth, the browser may show `localhost refused connection`.

### How to use with JeikCode? (Recommended)
[JeikCode](https://github.com/jeikl/JeikCode) is a modern terminal AI Coding Agent crafted by the core maintainer of this project, featuring native deep integration with Antigravity-Manager, delivering **95%+ KV-Cache hit rate** and seamless reasoning level control.
1. **One-Click Sync via GUI (Recommended)**:
   - Launch Antigravity-Manager and ensure the **API Proxy** service is running.
   - Switch to the **`>_ Agent Tools One-Click Configuration`** tab.
   - Locate the **JeikCode** card, choose your desired default model (e.g. `gemini-3.8-flash-high`), and click **`🔄 Sync Now`**.
   - Simply run `jeikcode` in your terminal for the TUI, **and type `/webui` to instantly launch the modern WebUI in your browser for a richer visual coding experience out of the box!**
2. **Temporary Environment Variables**:
```bash
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
export ANTHROPIC_API_KEY="sk-antigravity"
jeikcode --model claude-sonnet-4-6-thinking
```
> For complete manual configuration and tuning, see the [JeikCode Integration Guide](./docs/jeikcode_integration.md).

### How to use with Claude Code CLI?
1. Start Antigravity service in the "API Proxy" tab.
2. In your terminal:
```bash
export ANTHROPIC_API_KEY="sk-antigravity"
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
claude
```

### How to use with OpenCode?
1. Go to **API Proxy** → **External Providers** → click the **OpenCode Sync** card.
2. Click **Sync** to generate `~/.config/opencode/opencode.json`:
    - Creates a dedicated provider `antigravity-manager` (does not overwrite google/anthropic providers)
    - Optional: Check **Sync accounts** to export `antigravity-accounts.json` (plugin-compatible v3 format) for the OpenCode plugin
3. Click **Clear Config** to remove Manager configuration and clean up legacy entries; click **Restore** to revert from backup.
4. On Windows, the path is `C:\Users\<User>\.config\opencode\` (same `~/.config/opencode` rule).

**Quick verification commands:**
```bash
# Test antigravity-manager provider (supports --variant)
opencode run "test" --model antigravity-manager/claude-sonnet-4-5-thinking --variant high

# If opencode-antigravity-auth is installed, verify google provider still works independently
opencode run "test" --model google/antigravity-claude-sonnet-4-5-thinking --variant max
```

### How to use in Python?
```python
import openai

client = openai.OpenAI(
    api_key="sk-antigravity",
    base_url="http://127.0.0.1:8045/v1"
)

response = client.chat.completions.create(
    model="gemini-3-flash",
    messages=[{"role": "user", "content": "Hello, please introduce yourself"}]
)
print(response.choices[0].message.content)
```

### How to use with Kilo Code?
1.  **Protocol Selection**: We recommend using the **Gemini protocol**.
2.  **Base URL**: Set it to `http://127.0.0.1:8045`.
3.  **Note**:
    - **OpenAI Protocol Limitation**: When using OpenAI mode, Kilo Code's request path will append `/v1/chat/completions/responses`, a non-standard path that will return 404 from Antigravity. Make sure to enter the Base URL and select Gemini mode.
    - **Model Mapping**: Model names in Kilo Code may differ from Antigravity's defaults. If you encounter connection issues, set up custom mappings on the "Model Mapping" page and check the **log files** for debugging.

### How to connect a Telegram Bot & CLI (`agm telegram`)?
1. **Create a Bot**: Message `@BotFather` on Telegram with `/newbot` to obtain an HTTP API token (`<BOT_ID>:<BOT_SECRET>`), then open your new bot in Telegram and send `/ping`.
2. **Connect via UI (Auto-Detect Chat ID)**:
   - Navigate to **Settings → Email & Alerts** (or **Settings → Supabase Sync**).
   - Paste your **Telegram Bot Token**, click **Auto-Detect Chat ID** to automatically populate your numeric `Allowed Chat ID`, and click **Save Telegram Settings**.
3. **Connect via Terminal CLI (One Command)**:
   ```bash
   agm telegram connect "<YOUR_BOT_TOKEN>"
   ```
4. **Remote Commands (Chat & Terminal)**:
   - **In Telegram Chat**: `/ping`, `/observe`, `/status`, `/gitmap pe`, `/agm status`, `/api`, `/backup` (`/backpack`), `/restore`, `/email [status|ping|help]`, `/ff`, `/snapshot`, `/help`.
   - **In Terminal**: `agm telegram [ping|observe|gitmap|api|backup|restore|email|send|poll|ls]`.

### How to use Image Generation (Imagen 3)?

#### Method 1: OpenAI Images API (Recommended)
```python
import openai

client = openai.OpenAI(
    api_key="***",
    base_url="http://127.0.0.1:8045/v1"
)

# Generate image
response = client.images.generate(
    model="gemini-3-pro-image",
    prompt="A futuristic cyberpunk city with neon lights",
    size="1920x1080",      # Supports any WIDTHxHEIGHT format, auto-calculates aspect ratio
    quality="hd",          # "standard" | "hd" | "medium"
    n=1,
    response_format="b64_json"
)

# Save image
import base64
image_data = base64.b64decode(response.data[0].b64_json)
with open("output.png", "wb") as f:
    f.write(image_data)
```

**Supported parameters**:
- **`size`**: Any `WIDTHxHEIGHT` format (e.g. `1280x720`, `1024x1024`, `1920x1080`), auto-calculates and maps to standard aspect ratios (21:9, 16:9, 9:16, 4:3, 3:4, 1:1)
- **`quality`**:
  - `"hd"` → 4K resolution (high quality)
  - `"medium"` → 2K resolution (medium quality)
  - `"standard"` → Default resolution (standard quality)
- **`n`**: Number of images to generate (1-10)
- **`response_format`**: `"b64_json"` or `"url"` (Data URI)

<details>
<summary><b>🎨 Expand to view more image generation methods & parameter mapping rules (Chat API / Model Suffix / Cherry Studio)</b></summary>

#### Method 2: Chat API + Parameters (✨ New)

**All protocols** (OpenAI, Claude) Chat APIs now support direct `size` and `quality` parameters:

```python
# OpenAI Chat API
response = client.chat.completions.create(
    model="gemini-3-pro-image",
    size="1920x1080",      # ✅ Supports any WIDTHxHEIGHT format
    quality="hd",          # ✅ "standard" | "hd" | "medium"
    messages=[{"role": "user", "content": "A futuristic city"}]
)
```

```bash
# Claude Messages API
curl -X POST http://127.0.0.1:8045/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: ***" \
  -d '{
    "model": "gemini-3-pro-image",
    "size": "1280x720",
    "quality": "hd",
    "messages": [{"role": "user", "content": "A cute cat"}]
  }'
```

**Parameter priority**: `imageSize` parameter > `quality` parameter > model suffix

**✨ New `imageSize` parameter support**:

In addition to the `quality` parameter, you can now also use Gemini's native `imageSize` parameter:

```python
# Using imageSize parameter (highest priority)
response = client.chat.completions.create(
    model="gemini-3-pro-image",
    size="16:9",           # Aspect ratio
    imageSize="4K",        # ✨ Direct resolution: "1K" | "2K" | "4K"
    messages=[{"role": "user", "content": "A futuristic city"}]
)
```

```bash
# Claude Messages API also supports imageSize
curl -X POST http://127.0.0.1:8045/v1/messages \
  -H "Content-Type: application/json" \
  -H "x-api-key: ***" \
  -d '{
    "model": "gemini-3-pro-image",
    "size": "1280x720",
    "imageSize": "4K",
    "messages": [{"role": "user", "content": "A cute cat"}]
  }'
```

**Parameter descriptions**:
- **`imageSize`**: Direct resolution specification (`"1K"` / `"2K"` / `"4K"`)
- **`quality`**: Infers resolution from quality level (`"standard"` → 1K, `"medium"` → 2K, `"hd"` → 4K)
- **Priority**: If both `imageSize` and `quality` are specified, the system prioritizes `imageSize`

#### Method 3: Chat API + Model Suffix
```python
response = client.chat.completions.create(
    model="gemini-3-pro-image-16-9-4k",  # Format: gemini-3-pro-image-[ratio]-[quality]
    messages=[{"role": "user", "content": "A futuristic city"}]
)
```

**Model suffix explanation**:
- **Aspect ratio**: `-16-9`, `-9-16`, `-4-3`, `-3-4`, `-21-9`, `-1-1`
- **Quality**: `-4k` (4K), `-2k` (2K), no suffix (standard)
- **Example**: `gemini-3-pro-image-16-9-4k` → 16:9 ratio + 4K resolution

#### Method 4: Cherry Studio & Other Client Settings
In clients that support OpenAI protocol (e.g., Cherry Studio), you can configure image generation parameters via the **Model Settings** page:

1. **Enter Model Settings**: Select the `gemini-3-pro-image` model
2. **Configure Parameters**:
   - **Size**: Enter any `WIDTHxHEIGHT` format (e.g. `1920x1080`, `1024x1024`)
   - **Quality**: Choose `standard` / `hd` / `medium`
   - **Number**: Set the number of images (1-10)
3. **Send Request**: Simply type your image description in the chat dialog

**Parameter mapping rules**:
- `size: "1920x1080"` → Auto-calculated as `16:9` aspect ratio
- `quality: "hd"` → Mapped to `4K` resolution
- `quality: "medium"` → Mapped to `2K` resolution

</details>

## 📝 Changelog

> Latest version **v4.241.0**: Native Win32 DIB Image Clipboard, Prompt Tree Send & Enqueue Restoral, DetailHeader Title Polish, SQLite Reset & Rescan & Fleet Lead Alignment — Repaired `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` by checking `data:image/` before path normalization, running PowerShell in `-Sta` mode with `[System.Windows.Forms.Clipboard]::SetDataObject($data, $true)` to ensure persistent native `CF_DIB`/`CF_BITMAP` clipboard memory for Microsoft OneNote, Google Antigravity IDE, Word, and Paint; hooked native clipboard priming into `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts`; restored attached media preservation during turn selection in `createTurnConv` (`src/components/instances/PromptTreeView/ConversationNode.tsx`); passed `loadTree` callback from `PromptTreeViewModal.tsx` to `usePromptTreePromptActions` to trigger immediate tree refresh upon enqueuing; added current working directory fallback in `src-tauri/src/modules/repo_db/agy.rs` for prompts with empty workspace paths; stripped noisy turn badges preceding conversation titles in `src/components/instances/PromptTreeView/DetailHeader.tsx` and enlarged title typography to `text-lg md:text-xl font-bold font-ubuntu tracking-tight`; enhanced `wipe_and_rescan_repo_db` in `src-tauri/src/modules/repo_db/agy.rs` to wipe `media_vault`, in-memory tracking maps, and dispatched prompts cache alongside `invalidatePromptTreeCache()` on reset; and aligned cluster lead node `main` (`192.168.1.20`) and local node `w1` (`192.168.1.3`) in `src/components/instances/fleet/useFleetMachines.ts` and `FleetMachineRow.tsx` to reflect the authentic Git origin `alimtvnetwork / Antigravity-Manager` with bound accounts `sakibagjdjg@gmail.com` and `cirajayu11@gmail.com`. (Thanks to @aukgit)

> Latest version **v4.240.1**: Prompt Tree Buttons Repair, Native OS DIB Image Clipboard, DetailHeader Title Prominence, Full SQLite Reset & Rescan, Fleet Lead Node Alignment & agy CLI Path Resolution — Restored independent `[ Send N | v ]` and `[ Enqueue | v ]` buttons in `src/components/instances/PromptTreeView/DetailHeader.tsx` (now in distinct Segment 2 and Segment 3 capsules), ensuring full prompt text `conv.full_prompt_text` is dispatched without premature truncation and preserving attached media paths; implemented `copy_image_to_clipboard` backend command in `src-tauri/src/commands/cmd_files.rs` and `src-tauri/src/lib/run.rs` setting native Windows `CF_DIB`/`CF_BITMAP` format via PowerShell `System.Windows.Forms.Clipboard::SetImage` alongside pure PNG binary Blob to guarantee pasted images render 100% reliably in Microsoft OneNote, Paint, Word, and Antigravity IDE; normalized single-image copy button in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` strictly to "Copy" (eliminating "Copy including image"), and implemented `handleCopyImagesUnified` on the main copy button so clicking it when multiple images exist automatically stitches all attached images into a single vertical composite PNG canvas and writes to the system clipboard; added `~/.gemini/bin/agy.exe` and `~/.antigravity/bin/agy.exe` to `get_antigravity_cli_executable_path` in `src-tauri/src/modules/process/locations.rs` so CLI prompt dispatch resolves the installed `agy` binary reliably; expanded CDP composer selector ladder (`div.monaco-editor`, `div.interactive-input-editor`, `textarea.monaco-mouse-cursor-text`) and send button selectors in `src-tauri/src/modules/instance/cdp_onboarding.rs`; polished `DetailHeader.tsx` by stripping all noisy turn prefix badges preceding the title, enlarging conversation title text (`text-base md:text-lg font-bold font-ubuntu`), keeping Copy Title button adjacent, and positioning the status capsule strictly after the title; connected `onResetTree` in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `wipe_and_rescan_repo_db` and frontend `loadTree(true, true)` to completely purge SQLite prompt caches, terminate stale workers, rescan disk `transcript_full.jsonl` files fresh, and rebuild the tree with live spinner feedback; and corrected cluster lead node `main` (`192.168.1.20`) in `src/components/instances/fleet/useFleetMachines.ts` and `FleetMachineRow.tsx` to reflect its authentic Git origin `gitmap / gitmap` and bound account `sakibagjdjg@gmail.com` without leaking local machine `w1` (`192.168.1.3`, `Antigravity-Manager`) accounts. (Thanks to @aukgit)

> Latest version **v4.240.0**: Prompt Tree Button Repairs, Pure PNG Clipboard Pipeline, DetailHeader Clean Title Polish, SQLite Database Full Wipe & Rescan, Fleet Lead Node Alignment & Telegram Prompt Inbound Fix — Restored prompt and image dispatch across Prompt Tree Send and Enqueue actions without stripping image URIs into non-existent paths, with project path resolution fallbacks; upgraded image clipboard copying to write strictly single-MIME `'image/png'` binary blobs without conflicting HTML to guarantee pasting into Microsoft OneNote, Paint, and Antigravity; added vertical composite multi-image canvas stitching and 1-click `[ Copy All Images ]` batch copying; cleaned DetailHeader by stripping prefix turn badges, enlarging conversation titles, and repositioning the status capsule; added backend `wipe_and_rescan_repo_db` command with live progress feedback to completely rebuild the SQLite prompt database and regenerate the tree view; aligned Fleet lead node `main` (`192.168.1.20`) and `w1` (`192.168.1.3`) SSH host alias mappings with Git repository badge display; and fixed Telegram inbound `/prompt` route collision where conversation seek shadowed prompt execution. (Thanks to @aukgit)

> Latest version **v4.239.0**: Fleet Machine Identity w1 Resolution, Cluster Reset, Running Prompt Detection for Default Instances, Conversation Rename Sync, Audit Log Semantic Action Naming with Interactive Step Breadcrumbs, Start All Instances, and Markdown View Polish - Resolved local machine identity matching `192.168.1.3` to cluster node `w1` using Cluster-IP primacy in `src-tauri/src/modules/ssh_manager/exec.rs` while preserving all 6 discovered GitMap nodes in `FleetMachinesTable.tsx`; added `reset_ssh_cluster` backend command and UI toolbar button `[ 🔄 Reset Cluster ]` to safely clear stale connection pools; fixed running prompt liveness in `src-tauri/src/modules/repo_db/liveness.rs` and `detection.rs` by including `dispatched` status and preserving empty instance records; ensured conversation title renames update synchronously across `conversations.title`, `conversation_summaries.summary`, and `conversations_fts.title` with live UI notification; replaced cryptic Tailwind utility class selectors (`div.grid`, `Button.P 1.5`, `div.group`, `span.truncate`, `scroll:/instances`) in `useAuditRecorder.ts` and `AuditPanel.tsx` with human-readable semantic action descriptors; added an interactive "Step Performed & Interaction Execution Path" inspection card in `AuditPanel.tsx` featuring action summaries, connected DOM hierarchy breadcrumbs with arrow glyphs, and a 4-part target context grid; relocated the Details button with healthy status icon to row 2; standardized "Copy All (Include Images)" in the prompt tree toolbar; updated the Markdown view background to `#19191c`; converted verbose SSH settings text to icon buttons with hover tooltips; and added a "Start All" instances button in the main instance management toolbar. (Thanks to @aukgit)

> Latest version **v4.238.7**: Multimodal Prompt Backup & Restore E2E Suite, GitMap Delegation Parity, 10s Settling Invariant, SQLite Constraint & File-Lock Fixes - Implemented a comprehensive 25-assertion, 8-gate end-to-end test suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) covering running prompt inspection, multimodal deep analysis image attachments (Base64 data URIs, local fixtures, screenshot tokens), Split SQLite snapshot backup (`backup_prompts.db:prompt_backups`), and strict FIFO queue restoration with status transitioning to `'dispatched'`; verified full command parity across `agm running`, `agm backup running`, `agm restore running`, `gitmap agy running`, `gitmap agy ff`, `gitmap backup-running-prompts`, and `gitmap restore-running-prompts`; enforced mandatory `post_switch_resume_delay_seconds = 10` in `src-tauri/src/modules/repo_db/post_switch_resume.rs` ensuring Electron webview, extension host, and CDP debugger endpoints (`DevToolsActivePort`) are ready before prompt re-injection; hardened caller process detection (`agm instances whoami`) with Exit Code 2 refusal on caller stopping or mutation to protect active developer workspaces; fixed SQLite runtime constraint failure in `src-tauri/src/modules/instance/create.rs` (`NOT NULL constraint failed: instance_processes.pid`) preserving historical process ID while clearing active state; implemented `remove_dir_all_force` in `src-tauri/src/utils/fs.rs` with Win32 attribute stripping (`FILE_ATTRIBUTE_NORMAL`), eliminating `os error 5 (Access is denied)` during Chromium cache removal; and passed 8/8 diagnostic probes in `agm errors fix` self-healing engine. (Thanks to @aukgit)

> Latest version **v4.238.0**: Pure PNG Clipboard Copy, Image Attachment Forwarding, Turn Status Badges & GitMap Node Resolution — Implemented `writePureImageToClipboard` in `src/components/instances/PromptTreeView/clipboard.ts` writing strictly single-MIME `'image/png'` with canvas-based PNG header transcoding (`transcodeSrcToPngBlob`), eliminating multi-MIME format collisions that previously caused Microsoft OneNote, Slack, Discord, and Telegram to strip images or paste empty placeholders; added 1-click `[ 📋 Copy Image (PNG) ]` and sequential `[ 📋 Copy Images (N) ]` buttons in the "Attached Images" gallery header, plus hover-activated 1-click copy badges on individual thumbnails in `PromptInstructionView.tsx`; decoupled immediate session injection (`handleSendPromptNow` / `sendPromptNow`) from queue scheduling (`handleEnqueuePrompt`) in `usePromptTreePromptActions.ts` and `DetailHeader.tsx`; preserved and appended attached conversation images (`media_paths`) in markdown image syntax (`![Attached Image N](<path>)`) across Rerun, Send, and Enqueue flows so IDE sessions receive complete image attachments without data loss; refactored status evaluation in `src-tauri/src/modules/repo_db/tree_collect.rs` so live transcript activity (`is_owning_inst_alive && inspection.is_recent_active && !inspection.is_terminal_done`) takes precedence over stale SQLite idle summaries, accurately displaying emerald pulse `RUNNING` for active turns and amber `QUEUED` for pending turns; upgraded `src-tauri/src/modules/telegram_inbound/node_label.rs` to parse `~/.gitmap/gitmap-ssh-nodes.json` and `./gitmap-ssh-nodes.json` when the machine alias is `none` or auto-generated `Node-*`, matching local IP `192.168.1.3` to alias `w1` and annotating as `w1 (collected from gitmap)`; separated `Machine:` and `Instance:` onto distinct lines across all Telegram status reports in `reports_b.rs`, `system.rs`, `telegram_alerts.rs`, and `telemetry.rs`; and upgraded preview, raw markdown, and edit mode containers in `PromptInstructionView.tsx` to `dark:bg-[#0c0514]/95` with `dark:border-purple-500/25` subtle borders. (Thanks to @aukgit)

> Latest version **v4.237.0**: App Update Verification, Prompt Tree Selection Stability, Image Clipboard & Import-Export Restoral — Added `verify_tag_has_platform_asset` in `src-tauri/src/modules/update_checker/channels.rs` performing HTTP HEAD requests against the GitHub Release API to ensure that native platform installer binaries (`agm-alim-setup.exe`, `.dmg`, `.deb`) actually exist and are downloadable before notifying users of updates, eliminating 404 installation errors when tags exist before assets finish compiling; locked the actively selected turn (`selected_turn_number`) and project selection across background SWR sync intervals in `src/components/instances/PromptTreeView/usePromptTreeData.ts` and `ConversationNode.tsx`, preventing title and prompt text from glitching or alternating between conversation summaries and raw turn prompts during active inspection; added explicit `onClick` and `onDoubleClick` backdrop event handlers on the backdrop container in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to cleanly dismiss the modal when clicking outside; upgraded image processing in `PromptInstructionView.tsx`, `markdown.tsx`, and `clipboard.ts` with local base64 and asset protocol fallback cards (`PromptThumbnailCard`), and support rich multi-format clipboard copying (`image/png` binary blob + `text/html`) through `copyPromptWithRichImages` with deduplicated URI extraction and Windows drive letter decoding; added direct combined export for both running and enqueued prompts (`Export Running & Queued Prompts (.json)`) and direct backup restoration in `src/components/instances/PromptTreeView/ImportExportMenu.tsx` and `usePromptTreeCoreActions.ts`, displaying an interactive 5-second countdown HUD banner before automatic prompt dispatching; preserved `saved_media_opt` during `/send` flows in `src-tauri/src/modules/telegram_inbound/send_flow.rs` and `injection.rs`, populating `active_prompts.image_payload` with attached media counts and timestamps; implemented an anti-aliased synthetic dispatch card in `open_shot.rs` when running under non-interactive Windows OpenSSH Session 0 without display station access; and reconciled `SshConnectionRecord` field schemas across `src-tauri/src/modules/supabase_sync/query.rs` and `src/components/instances/fleet/useFleetMachines.ts`, eliminating all Rust compilation, Clippy, and TypeScript linter errors across all supported OS matrix targets. (Thanks to @aukgit)

> Latest version **v4.236.0**: UI/UX Redesign, 24h Date Standards, Expand Full Persistence, Asset Protocol Capability, 5s Readiness Ladder & Account-Centric SSH Discovery — Converted session overview card in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` to be collapsed by default with persistent localStorage memory (`agm_prompt_overview_expanded`) and low-profile purple trigger (`ChevronDown`/`ChevronUp`); deduplicated redundant header and clarified sequential conversation index (`C211` = Conversation #211) and interaction turns (`turns` = user/assistant request-response cycles) in `src/components/instances/PromptTreeView/DetailsModal.tsx`; formatted `Last Modified` into localized 24-hour clock (`DD-MMM-YYYY HH:mm`) with relative delta ("ran 5 minutes ago", "ran 1 day ago"); fixed bug in `src/components/instances/PromptTreeView/DetailTabs.tsx` where switching from Input to Output hid the Expand Full button; implemented centralized date formatting in `src/utils/date.ts` supporting `DD-MMM-YYYY HH:mm` in 24-hour clock using local system timezone, with relative time indicators and duration seconds suppression; added user-configurable `date_format_mode` toggle with strictly typed `DateFormatModeType` enum (`TwentyFourHour = '24h'`, `TwelveHour = '12h'`, `Iso = 'iso'`) in `src/types/config.ts` and Settings; added Tauri v2 `core:asset:default` capability in `src-tauri/capabilities/default.json` and resolved relative image paths against `repo_path` in `clipboard.ts` and `PromptMarkdownPreview.tsx`; protected queued prompts from stale conversation demotion in `conversationGuards.ts`; synthesized running/queued projects from `active_prompts` in `src-tauri/src/modules/repo_db/tree_phases.rs`, ensuring the left panel is never empty when prompts are in flight; upgraded `src-tauri/src/modules/instance/smart_run.rs` and `src-tauri/src/modules/repo_db/agy.rs` with an adaptive 5-second process stabilization ladder and CDP retry mechanism for cold-started IDE instances, eliminating lost prompts when dispatching to closed IDEs; and upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via SSH, elevating Account Email and Instance Name to primary columns in `FleetMachinesTable.tsx` and `SshGitmapSection.tsx` while demoting raw IP into an interactive hover badge, and adding dynamic instance `<select>` dropdown and 1-click remote prompt dispatching. (Thanks to @aukgit)

> Latest version **v4.235.0**: GitMap Node, SSH Fleet Discovery, Prompt Tree Reliability, Supabase Cooldown & Telegram Routing — Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` with `extract_clean_json` to bypass leading logger noise, OS-specific CLI execution (`agm.exe` on Windows vs `agm` on Linux), and direct PowerShell/Bash file inspection over SSH (`instances.json`, `accounts.json`) to reliably extract remote instances and bound Google accounts; calculates accurate dual-window quotas via `extract_dual_window_quotas` instead of looking for nonexistent root fields; synchronizes encrypted node passwords to `~/.gitmap/gitmap-ssh-nodes.json` in `connections.rs` so GitMap's `fix-auth` engine possesses authentication credentials; forwards default identity key paths (`-i <pubkey>`) in `fix_node_auth_via_gitmap`; integrated automatic cooldown triggers in `src-tauri/src/modules/supabase_client/client.rs` across all client methods (`select`, `insert`, `upsert`, `update`, `delete`, `rpc`), automatically invoking `record_supabase_network_error()` on timeouts, connection errors, and 5xx responses, while fast-failing subsequent requests with `Err(AppError::Network(...))` during the 300-second cooldown; added circuit breaker guards to background sync workers in `fleet.rs`, `sync.rs`, and `supabase_pruner.rs` to stop repetitive connection retry loops; converted `onRefreshAll` and `onResetTree` in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` into async functions awaiting `loadTree(false, true)`, synchronizing cache invalidation (`invalidatePromptTreeCache`) and auto-dismissing visual status banners after 2500ms; enabled flexible instance dispatch in `src-tauri/src/modules/repo_db/dispatch.rs` so restored prompts dispatch cleanly across active and default instances with loopback CDP injection fallback (`submit_prompt_via_cdp_for_instance`); refactored `src/components/instances/PromptTreeView/ImportExportMenu.tsx` to mount its dropdown popup via React Portal (`createPortal(..., document.body)`) with `fixed z-[99999]` and a full-screen transparent capture backdrop (`fixed inset-0 z-[99998]`), completely eliminating CSS stacking context clipping and blur bleed-through from `ModalHeader` (`z-40`) and `TreePanel`; removed redundant plain `User Prompt` badges and verbose instance ID tokens (`default-copy-5597`) from `DetailHeader.tsx`, directing deep runtime telemetry to `DetailsModal.tsx`; updated `performAutoSelection` in `usePromptTreeData.ts` to preserve existing selected conversations on reloads; tightened skeleton shimmer mounting to avoid screen glitches during background refreshes; added explicit amber clock `QUEUED` badges to child subagents in `ProjectNode.tsx` when enqueued under parent prompts; enforced strict 3-tier ordering (Tier 1: Running, Tier 2: Queued, Tier 3: Newest Timestamp) across project trees; increased `FETCH_DEBOUNCE_MS` to 30s in `src/stores/useAccountStore.ts` with 5-minute idle throttling when `document.hidden`; implemented adaptive 60s/300s polling in `src/pages/instances/useInstancePage.ts` with a 2000ms coalescing debounce on live events; preserved `ACCOUNT_LIST_CACHE` in `src-tauri/src/modules/account/crud.rs` via in-place memory updates on quota saves; optimized `src-tauri/src/modules/repo_db/liveness.rs` to query `conversation_summaries.db` once per instance sweep into a `HashSet<String>` rather than opening the database 15-30 times for every workspace folder; cached `detect_running_projects` for 15s in `detection.rs`; demoted idle/negative audit logs (`IdleNoActiveTasks`) from `info!` to `tracing::debug!`; expanded `machine_entries()` in `src-tauri/src/modules/telegram_inbound/send_flow.rs` to discover SSH mesh and Supabase fleet nodes alongside GitMap nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to `agm.exe prompt <target> [flags] "<prompt>"` to correctly invoke targeted sequence resolution (`C001`, `P001`) instead of swallowing arguments; and added a 3-stage execution fallback ladder inspecting process exit status. (Thanks to @aukgit)

> Latest version **v4.234.0**: GitMap Node and Prompt System Issues with Instance Management Sync — Added `deduplicate_connections()` in `src-tauri/src/modules/ssh_manager/connections.rs` to eliminate duplicate cluster node records; resolved local node identity in `src-tauri/src/modules/ssh_manager/exec.rs` by matching local IPv4 against configured SSH connections (resolving local alias as `w1`), ensuring local instances are not excluded or misattributed to remote nodes; added dual-key matching (`machine_alias` or `ip_address`) in `src/components/settings/EmailNotifications/SshGitmapSection.tsx` to accurately display instances and bound accounts; restored multi-VM collision shielding guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` (`on_machine_picked`, `on_instance_picked`), ensuring only targeted nodes respond to machine/instance callbacks while non-target nodes drop replies cleanly to prevent multi-daemon message collisions; refined `is_current_machine` in `src-tauri/src/modules/telegram_inbound/node_label.rs` to prioritize authentic local hostnames (`detect_machine_name`) and local IPs before querying cluster nodes; standardized remote CLI dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to `agm prompt send --instance <inst> [--repo <repo>] "<prompt>"`; centralized conversation status detection in `src/components/instances/PromptTreeView/conversationGuards.ts` (`getConversationStatus`) and re-exported through `ConversationNode.tsx`; enforced strict 3-tier ordering in `ProjectNode.tsx`, `tierClassification.ts`, and `PromptTreeViewModal.tsx` so running prompts stay strictly at Tier 1 on top, queued prompts at Tier 2 with amber clock badges, and completed/historical prompts ordered by timestamp descending; and added document visibility change detection and 5-minute background throttling in `src/pages/instances/useInstancePage.ts` and `src/stores/useAccountStore.ts`, coalescing IPC refresh requests and eliminating CPU spikes during inactive background state. (Thanks to @aukgit)

> Latest version **v4.233.1**: Prompt Tree View Reliability, GitMap Node Discovery & Clipboard Pipeline — Centralized context menu state into `TreePanel.tsx` with a single `<PromptTreeContextMenu>` and transparent portal backdrop; added outside capture event listeners (`pointerdown`, `contextmenu`, `keydown`, `scroll`, `blur`) to dismiss immediately on any outside interaction; introduced `isConversationOrTurnQueued` to auto-expand queued child turns, render amber `QUEUED` badges with clock icons, and prioritize queued items in Tier 2 sorting; prioritized `gitmap cluster nodes --json` and `export-json` in `src-tauri/src/modules/ssh_manager/sync.rs` for reliable cluster node discovery across Windows/Linux machines; corrected offline SSH node quota fallbacks to 0.0% instead of 100.0% in `src-tauri/src/modules/ssh_manager/exec.rs`; added request debouncing and in-flight promise coalescing to `useAccountStore.ts` to prevent redundant IPC bursts; introduced `read_image_as_base64` in `src-tauri/src/commands/cmd_files.rs` and registered in `src-tauri/src/lib/run.rs` to read local image files securely with path canonicalization, system path protection, and a 100MB ceiling; and decoded Base64 data directly into binary PNG Blobs (`base64ToBlob`) in `src/components/instances/PromptTreeView/clipboard.ts`, eliminating offscreen HTML canvas CORS errors and WebView2 `SecurityError` canvas tainting, with rich event JSON clipboard copying and toast feedback. (Thanks to @aukgit)

> Latest version **v4.233.0**: GitMap Node, Prompt System & Instance Management Synchronization — Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via GitMap discovery and cluster metadata; forwarded identity public key `-i <pubkey>` in `fix_node_auth_via_gitmap` across `mesh.rs`, `commands/ssh.rs`, and CLI handlers, removing hardcoded single-node bypasses in favor of generalized credential vault auto-repair; removed fatal callback drop guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` so the receiving Telegram bot coordinator accurately routes prompts across cluster nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to canonical `agm prompt send <project> --instance <inst> "<prompt>"` with safe quoting and multi-line escaping; inverted prompt dispatch in `src-tauri/src/modules/repo_db/agy.rs` to submit directly via Chrome DevTools Protocol (CDP) into active IDE windows before falling back to spawning external `agy` CLI background workers; wired direct 1-click execution for Backup and Restore in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`; added spinning animation on the Refresh icon during active sync; provided instant visual notification feedback on Reset and Refresh; restricted animated skeleton loaders in `src/components/instances/PromptTreeView/TreePanel.tsx` and `PromptTreeViewModal.tsx` strictly to initial cold starts (`isLoading && treeData.length === 0`), keeping the active tree rendered during background polling to eliminate visual flashing; enforced 3-tier deterministic sorting in `ProjectNode.tsx` and `tierClassification.ts` with `RUNNING` prompts strictly at Tier 1, `QUEUED` prompts at Tier 2 with explicit amber badges, and timestamp descending at Tier 3; removed redundant `"User Prompt"` badge and duplicate `{instanceNameDisplay}` text token from `DetailHeader.tsx`, relocating deep runtime telemetry to the Details modal; added `relative z-40` to `ModalHeader.tsx` container and `relative z-50` to `ImportExportMenu.tsx` so menus stack cleanly above banners and header elements; implemented atomic `LAST_SUPABASE_ERROR_TIMESTAMP` circuit breaker in `src-tauri/src/modules/supabase_sync/types.rs` with a 300-second cooldown on network errors across `fleet.rs`, `query.rs`, and `release.rs`; relaxed fleet polling to 300s in `useFleetMachines.ts`; added a 30-second TTL cache for account listings in `src-tauri/src/modules/account/crud.rs` to stop repetitive reads of 46 JSON account files; removed circular cache invalidation sabotage in `src-tauri/src/modules/repo_db/detection.rs:254` to eliminate console log flood; replaced ambiguous status codes with an emerald pulse badge (`PID: <pid>`) and aligned executable tail and data directory tail together on the exact same horizontal flex row (`.../Antigravity.exe · .../data`) in `src/components/instances/table/InstanceRowCells.tsx`; upgraded `src/pages/instances/SwitchAccountDialog.tsx` with cohesive dark-glass UI/UX; enforced process tree killing via `taskkill /F /T /PID` before stopping or switching accounts in `src-tauri/src/commands/instance.rs`; hardened `detect_caller_instance_id` to protect the developer's active workspace session (`default-copy-5597`); and verified with transient test sandboxes (`agm instances test-create` and `agm instances test-clean`). (Thanks to @aukgit)

> Latest version **v4.231.0**: Instance Management, PID Inline List View Layout, Deterministic Process Termination Ladder, Account Switch Pre-Switch Eviction, Royal Violet Switch Account Modal & Sharded Audit Trail Visibility — Confirmed numeric status `1632` represents native OS Process ID (`PID`); merged separate `Status & PID` and `IDE EXE & DATA` table columns into a unified `Status, Process & Environment` cell in `src/components/instances/InstanceTable.tsx` and `src/components/instances/table/InstanceRowCells.tsx` with inline monospace PID badge (`PID: <pid>`), shortened executable tail, and shortened data directory tail separated by `·` with segmented copy action capsule; established 5-stage deterministic process termination ladder (`taskkill /F /T`, Win32 `TerminateProcess`, PowerShell `Stop-Process`, 200ms kernel settle delay, Chromium lockfile eviction) in `src-tauri/src/modules/instance/close.rs` and `create.rs`; enforced pre-switch PID eviction and 200ms settle delay in `switch.rs` and `credentials.rs` to eliminate Electron write-back race conditions; redesigned `SwitchAccountDialog.tsx` to 100% Royal Violet design tokens (`var(--ui-surface-1)`, `var(--ui-border-subtle)`, `var(--ui-interactive)`, `var(--ui-success)`) with segmented quota pill capsule; resolved empty audit trail in `switch_history.rs` and `InstanceAuditTrailModal.tsx` by expanding default instance aliases, guaranteeing response fields, and adding smart auto-tab fallback to operations; hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations) with exit code 2 immunity guard; and verified 100% green via `scripts/test-instance-lifecycle-e2e.ps1`. (Thanks to @aukgit)

> Latest version **v4.230.0**: Prompt Tree View Ergonomics, SSH GitMap Parity, Instance Prompt Isolation, Standalone JSON & Pinned Updater Resolution — Compacted Input and Output indicators into a unified segmented capsule alongside Expand Full on the right-hand side of Row 2 in `DetailTabs.tsx`; streamlined View Modes (`Preview`, `Raw`, `Edit`, `Details`) into an icon-only compact capsule (`rounded-lg` container, `rounded-md` buttons) on the left-hand side with hover tooltips; elevated Copy, Send, and Rerun action controls to Row 1 with dual rich copy options (`includes image` binary PNG + HTML vs `copy as text`); resolved prompt image rendering in WebView2 using Tauri asset URLs (`convertFileSrc`) to prevent image loading failures; added right-click context menus for project headers, conversation nodes, and prompt turns; introduced a comprehensive Conversation Session Summary Dashboard when selecting conversation roots and turn breadcrumb badges (`↳ Turn X of Y`) on individual prompt turns; restored the Shared Cluster section in instance management to discover and display registered local SSH mesh nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) with TCP liveness probing even when Supabase Cloud Sync is disabled; resolved SSH connection failures to remote node `w2` (192.168.1.7:22) by integrating GitMap Go in-process SSH fallback and vaulted credential delegation (`rtyrty123@`); enforced Explicit Idle Supremacy in `tree_collect.rs` and `liveness.rs` where explicit idle statuses (`CASCADE_RUN_STATUS_IDLE`) override unreset `not_fully_idle` engine artifacts; restricted instance card active task indicators strictly to genuine conversation owners (`c.instance_id === inst.config.id`); upgraded prompt tree JSON export to v2.0 envelope schema with portable base64 embedded images and relative `${workDir}` path normalization; elevated modal overlay z-index to `z-[100050]` so export/import dialogs are never trapped behind the main prompt tree modal; and resolved updater version reporting by writing `DisplayVersion` to Windows Registry branding in `install.ps1` (fixing false `v4.190` reports), prioritizing PE executable `ProductVersion` inspection, and enforcing strict `-isPinned` contracts preventing silent fallback to unrequested release versions. (Thanks to @aukgit)

> Latest version **v4.229.0**: Process Termination Ladder, List View PID Layout, Switch Account Sync & Audit Trail Restoration — Fixed stop instance button and account switching synchronization failures by establishing a multi-stage process termination ladder (`taskkill /F /T` with fallback to Win32 `TerminateProcess`), conscious PID matching against cached process instances, and deterministic lockfile eviction (`code.lock`, `lockfile`, `DevToolsActivePort`) before updating `state.vscdb`; hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations and matching workspace paths) across `agm ff`, `switch_account_to_instance`, `stop_instance`, and `close_instance` with `refuse_if_caller_instance` guards to prevent CLI commands and automated routines from self-terminating the developer's active workspace session; fixed empty instance audit trail by refining SQL filter logic to eliminate global account crud pollution from `default` instance records, instrumenting missing lifecycle operations (`RestartInstance`, `CloneInstance`, `CreateInstance`) into `task_history_db`, and expanding frontend action code mapping for lifecycle operations; aligned running instance status (`● PID: <pid>`), executable tail, and data directory tail inline on the same horizontal row with monospace formatting, enhanced tooltips, and contiguous segmented copy actions; redesigned switch account modal with Royal Violet tokens (`#171025`, `#40285e`, `#1af18d`), circular radio selection indicators, standard `<TierBadge />` integration, search filter clear button, and full keyboard accessibility; and verified end-to-end through `scripts/test-instance-lifecycle-e2e.ps1` covering caller self-preservation guards, transient test sandbox creation, PID eviction, switch synchronization, and clean filesystem teardown. (Thanks to @aukgit)

> Latest version **v4.228.0**: Instance Management, UI/UX Synchronization & CLI Self-Preservation — resolved instance process termination and account switching synchronization failures by implementing Win32 process tree escalation (`taskkill /F /T /PID` and `TerminateProcess`), 8.3 path normalization (`GetShortPathNameW`), and a 200ms kernel settle delay to guarantee process death and lockfile release before credential rewrites in `state.vscdb`; hardened `detect_caller_instance_id` to trace cloned executables (`Antigravity-<id>.exe`), `ANTIGRAVITY_APP_DATA_DIR`, and `VSCODE_PID`, enforcing caller immunity guards across `rm`, `switch`, `ff`, and `stop` to prevent CLI operations from terminating active IDE developer windows; explicitly formatted running instance status as `● Running (PID: <pid>)` with pulsing emerald indicator; aligned executable tail and data directory tail together inline on the same horizontal row with monospace pill styling and dual copy actions; completely overhauled switch account modal with AGM dark-glass styling (`rounded-2xl`, Royal Violet surfaces, sky/amber badges, `.ui-interactive-row` selection, Escape key dismissal, and segmented capsule controls); extended `AuditAction` enum with instance lifecycle actions (`LaunchInstance`, `StopInstance`, `KillInstance`, `CreateInstance`, `CloneInstance`, `DeleteInstance`, `InstanceError`), instrumented backend lifecycle logging into `task_history_db`, and added automatic fallback to operations tab to eliminate empty audit states; and added `agm instances current`, `agm instances create-test`, and `agm instances rm-test` for sub-50ms transient data-only testing, and hardened test suite (`scripts/test-instance-autoswitch-e2e.ps1`) with 30-second bounded timeouts. (Thanks to @aukgit)

> Latest version **v4.227.0**: Error E9001 Sibling Bound Isolation, Loop Throttling, Riseup Yellow 3D Switch & Flat Idle Controls, Table Dark-Glass Hover Contrast, Green Quota Readability, Prompt Parameter Context Menu, and GitMap JWT Bridge — resolved Error E9001 root-cause by protecting actively running sibling instances (`is_running == true`) from candidate theft during Fast-Forward and manual rotation in `instances.rs`, excluding bound accounts in `manual.rs`, and shielding candidates in `selection.rs`; enforced strict 30-second minimum floor on CLI poller loops (`fpug` and `sug` in `green_cmds.rs`) and `MIN_GOOGLE_QUOTA_CHECK_SECONDS`; eliminated infinite 5-second queue retry loops when workspace directories are missing in `queue.rs`; enforced a 5-minute (300s) backoff on Supabase heartbeat/connection failures; redesigned Account and Instance toolbars by replacing the 4th Tag icon with Fast-Forward, moving Tag into the dropdown, flattening idle Switch/Delete buttons, and applying authoritative Riseup Yellow 3D gradients (`from-[#FDE047] via-[#FACC15] to-[#EAB308]`, border `#CA8A04`, depth shadow `[0_2px_0_0_#A16207]`); fixed table row hover by replacing pure white washes with balanced dark-glass styling (`hover:bg-slate-100/80 dark:hover:bg-slate-800/60`) and making inner `td` cell backgrounds transparent; upgraded `default-copy` instance badge contrast (`bg-indigo-100 text-indigo-950 font-black`); improved green duration readability by enclosing `#1af18d` in high-contrast micro-pill badges; flattened nested JSX ternaries in `InstanceCard.tsx` into clean subcomponents with early returns; added right-click parameter context menu in `DetailHeader.tsx`; added 1-click Reset buttons to instance search and SSH prompt dispatcher; upgraded Win32 updater console to 18pt Consolas (`font_family = 54`); and architected GitMap <-> AGM mutual HMAC-SHA256 JWT communication bridge. (Thanks to @aukgit)
> Latest version **v4.225.0**: Cross-System 30-Second Minimum Interval Guard & Broadcast Storm Suppression Across Antigravity Manager & GitMap — enforced an architectural invariant across Antigravity Manager and GitMap mandating that all periodic background broadcasts, polling schedulers, worker tickers, and notification dispatchers operate at intervals of not less than 30 seconds (`>= 30s`) to prevent CPU spikes, database lock contention, and notification storms; clamped the detached prompt goal heartbeat worker loop in `goal_heartbeat.rs` to a strict 30-second minimum (`interval_secs.max(30)`, default 30s); expanded the duplicate switch notification broadcast suppression window in `channels.rs` from 5s to 30s (`(now_ts - guard.1).abs() <= 30`), completely preventing rapid notification storms across Email, Telegram, and Supabase; and aligned GitMap task synchronization defaults (`TaskMinInterval = 30`, `TaskDefaultInterval = 30` in `constants_task.go`), clamped async periodic monitors (`gitmap async -t`) to `>= 30s`, clamped macro runner intervals to `>= 30s`, and enforced minimum 30s cluster distribution heartbeat loops. (Thanks to @aukgit)
> Latest version **v4.224.0**: Compact Telegram Alerts with Next-Line White Text Values, Step 2 Multi-Machine Collision Shielding, Node IP & Signal Attribution, and Action-Only Keyboards — modernized all Telegram notification cards (Account Rotation, Post-Switch Liveness, Emergency Alerts, and System Updates) to be ~40% more vertically compact with key values indented on the next line in high-contrast bright white text (`kv_block`, `kv_white`); removed duplicate header and workspace fields; clamped project lists to top-3; eliminated duplicate Step 2 prompt wizard dispatch in Telegram via `node_label::is_current_machine()` destination matching and <3s debounce guard; enriched instance selectors with explicit node alias, IP, and signal indicators (`🏷️ {name} ({alias} • {ip})`) in single-column layout; and streamlined Account Rotation inline keyboards to functional action-only buttons. (Thanks to @aukgit)
> Latest version **v4.223.0**: Prompt Tree Custom Right-Click Menu, Right-Aligned Primary Navigation Tabs, Header Dead-Space Removal, Top Toolbar Copy All Consolidation & Telegram Cluster Shielding — added right-click context menus directly to project headers (`ProjectContextMenu.tsx`) with Import, Export (JSON/SQLite/YAML), and Rerun actions; wired right-click triggers and keyboard navigation (`Enter`, `Space`, `F2`) across conversations, subagents, and individual execution turns with active selection highlights; relocated primary navigation tabs ("Prompt Instruction" vs "AI Results & Outputs") to the right side of Row 2 in `DetailHeader.tsx` and eliminated the redundant 3rd row in `DetailTabs.tsx` to reclaim ~44px of vertical height; consolidated all copy actions into the top-level `[ Copy All | v ]` segmented capsule and purged redundant footer copy buttons in `PromptInstructionView.tsx`; eliminated cluster-wide prompt broadcast storms by binding `target_node_id` in Supabase `command_queue` payloads across `injection.rs`, `open_shot.rs`, and `processing.rs`; unified local machine identification via `node_label::is_current_machine()`, deduplicated "This machine" buttons in Telegram picker, added native CLI support for `agm prompt <target> [flags] <prompt_text>`, and redesigned destination locked UX so direct message typing is front-and-center while folding past prompts into expandable blockquotes. (Thanks to @aukgit)
> Latest version **v4.221.0**: Multi-Machine Fleet Synchronization & W2 Connectivity Recovery (extended TCP probe timeout to 1500ms with a 2-stage retry ladder and latency/error diagnostics to resolve false-offline W2 status), Multi-Machine Matrix Dynamic IP & Placeholder Normalization (serialized `machine_ip` and `ip_address` to eliminate empty `()` display; normalized `${adminUser}` to `Administrator` and `${linuxUser}` to `root`), Settings SSH & GitMap Section UI Overhaul (contiguous segmented pill capsule for fleet actions, sleek metadata badges replacing raw path dumps, and segmented terminal install controls with copy toast feedback), and Multi-Agent Concurrency Upgrade to A = 3, H = 2 (6 concurrent operations) across skills and coding guidelines. (Thanks to @aukgit)
> Latest version **v4.220.0**: Accounts Table Delete Remediation & Modal Mounting (resolved non-responsive single-item and bulk Delete buttons by mounting dedicated destructive `ModalDialog` components in `AccountsDialogs.tsx`), Position-Locked Table Row Hover Inversion (inverts row to pure white background and black text with static geometry to eliminate layout jitter), Action Buttons Color Scheme Overhaul (prominent yellow highlighter Switch button `⇄` and solid red Remove button `🗑️` with crisp white icon), and Account Disable / Enable Lifecycle (selection toolbar `<Ban>` button, action menu toggle, `disabled` model persistence, and 100% auto-switcher candidate exclusion with permissive manual user switching). (Thanks to @aukgit)
> Latest version **v4.219.0**: Prompt Tree Rich Verification Chapter Engine (`verificationTemplates.ts` with 5-chapter structure for "Is it done?", "Is it released?", "Are you confident?", "Edge Cases & Tests"), Rerun Chapter Engine with progression counter (`Round ${round} of ${total}`) and full prompt preservation, and Dual-MIME rich image clipboard priming on prompt resend/enqueue. (Thanks to @aukgit)
> Latest version **v4.218.0**: Repository Issues & System Hardening — Added `agm clean-other-versions` (`agm ov`) CLI command to purge stale/legacy binaries while strictly preserving SQLite databases; dynamic Telegram versioning; internal ID stripping from prompts; configurable 100/200-word summaries with resend buttons; combined IDE Exe & Data directory column; PromptTree DetailHeader overlap resolution & live disk sync; rich clipboard media copying; 5-tier color-segmented quota bar; interaction telemetry attributes; and clean process exit. (Thanks to @aukgit)
> Latest version **v4.217.0**: Instance Audit Trail Visibility (split-DB & `hot_tasks_cache` alias resolution, dual-tab 'Switch Cycles' vs 'All Operations' with auto-fallback), Switch Account Dialog UI/UX Consistency (dark-glass modal, top-right 'X' close button, row hover highlights, Current checkmark badge, color-coded quota badges), Active Caller IDE Detection & Self-Preservation Guard (`detect_caller_instance_id` via ancestor process command-line inspection and env checks, exit code 2 refusal on self-mutation), and Lightweight Test Instance Lifecycle (`agm instances create-test` / `rm-test` transient sandboxes without binary duplication). (Thanks to @aukgit)
> Latest version **v4.216.2**: Build fix — `ui_cmds.rs` now passes `None` for the new `stack_trace` parameter of `audit::append_entry` (E0061 broke the v4.216.1 build). (Thanks to @aukgit)
> Latest version **v4.216.1**: Build fix — `post_switch_resume.rs` timer registry now uses `tauri::async_runtime::JoinHandle` to match the `tauri::async_runtime::spawn` return type (E0308 broke the v4.216.0 build). (Thanks to @aukgit)
> Latest version **v4.216.0**: Post-switch prompt resume (~10s, configurable, cancellable) rides the RCA-15 restore pipeline; durable prompt backup re-sendable later via `agm prompts resend-queued` CLI or the UI button (FIFO, deduplicated); rerun section in the prompts tree send area (count selector, "is it done?" / "is it published?" / randomized messages); audit entries now capture stack traces (function + file + line, Rust and frontend). (Thanks to @aukgit)
> Latest version **v4.215.1**: Telegram `/expand` format-string fix; same Telegram enhancements as v4.215.0 on a green tree. (Thanks to @aukgit)
> Latest version **v4.231.0**: Instance Management & UI/UX Synchronization — Confirmed numeric status `1632` represents native OS Process ID (`PID`); merged separate `Status & PID` and `IDE EXE & DATA` table columns into a unified `Status, Process & Environment` cell with inline monospace PID badge, shortened executable tail, and shortened data directory tail separated by `·` with segmented copy action capsule; established 5-stage deterministic process termination ladder (`taskkill /F /T`, Win32 `TerminateProcess`, PowerShell `Stop-Process`, 200ms kernel settle delay, Chromium lockfile eviction) in `close.rs` and `create.rs`; enforced pre-switch PID eviction and 200ms settle delay in `switch.rs` and `credentials.rs` to eliminate Electron write-back race conditions; redesigned `SwitchAccountDialog.tsx` to 100% Royal Violet design tokens with segmented quota pill capsule; resolved empty audit trail in `switch_history.rs` and `InstanceAuditTrailModal.tsx` by expanding default instance aliases, guaranteeing response fields, and adding smart auto-tab fallback to operations; hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations) with exit code 2 immunity guard; eliminated Error Manager warnings (`NotAllowedError: Document is not focused`) via a 3-stage clipboard fallback ladder (`ensureDocumentFocus`, Async Clipboard API, `document.execCommand`, native OS helper) in `clipboard.ts`; eliminated intermittent registry deserialization errors (`expected value at line 1 column 1`) via atomic `write_atomic` persistence and `.bak` auto-recovery in `registry.rs`; and verified 100% green via `scripts/test-instance-lifecycle-e2e.ps1`. (Thanks to @aukgit)
> Latest version **v4.230.0**: Prompt Tree View Ergonomics, SSH GitMap Parity, Instance Prompt Isolation, Standalone JSON & Pinned Updater Resolution — Compacted Input and Output indicators into a unified segmented capsule alongside Expand Full on the right-hand side of Row 2 in `DetailTabs.tsx`; streamlined View Modes (`Preview`, `Raw`, `Edit`, `Details`) into an icon-only compact capsule (`rounded-lg` container, `rounded-md` buttons) on the left-hand side with hover tooltips; elevated Copy, Send, and Rerun action controls to Row 1 with dual rich copy options (`includes image` binary PNG + HTML vs `copy as text`); resolved prompt image rendering in WebView2 using Tauri asset URLs (`convertFileSrc`); added right-click context menus for project headers, conversation nodes, and prompt turns; introduced a comprehensive Conversation Session Summary Dashboard when selecting conversation roots and turn breadcrumb badges (`↳ Turn X of Y`) on individual prompt turns; restored the Shared Cluster section in instance management to discover and display registered local SSH mesh nodes (`w1`, `w2`, `w3`, `w4`, `u1`, `main`) with TCP liveness probing even when Supabase Cloud Sync is disabled; resolved SSH connection failures to remote node `w2` (192.168.1.7:22) by integrating GitMap Go in-process SSH fallback and vaulted credential delegation (`rtyrty123@`); enforced Explicit Idle Supremacy in `tree_collect.rs` and `liveness.rs` where explicit idle statuses (`CASCADE_RUN_STATUS_IDLE`) override unreset `not_fully_idle` engine artifacts; restricted instance card active task indicators strictly to genuine conversation owners (`c.instance_id === inst.config.id`); upgraded prompt tree JSON export to v2.0 envelope schema with portable base64 embedded images and relative `${workDir}` path normalization; elevated modal overlay z-index to `z-[100050]`; and resolved updater version reporting by writing `DisplayVersion` to Windows Registry branding in `install.ps1` (fixing false `v4.190` reports), prioritizing PE executable `ProductVersion` inspection, and enforcing strict `-isPinned` contracts. (Thanks to @aukgit)
> Latest version **v4.229.0**: Process Termination Ladder, List View PID Layout, Switch Account Synchronization & Audit Trail Restoration — Resolved instance process termination and account switching synchronization failures by establishing a multi-stage process termination ladder (`taskkill /F /T` with fallback to Win32 `TerminateProcess`), conscious PID matching against cached process instances, and deterministic lockfile eviction (`code.lock`, `lockfile`, `DevToolsActivePort`) before updating `state.vscdb`; hardened caller IDE detection (`detect_caller_instance_id` walking 24 parent process generations and matching workspace paths) across `agm ff`, `switch_account_to_instance`, `stop_instance`, and `close_instance` with `refuse_if_caller_instance` guards to prevent CLI commands and automated routines from self-terminating the developer's active workspace session; fixed empty instance audit trail by refining SQL filter logic to eliminate global account crud pollution from `default` instance records, instrumenting missing lifecycle operations (`RestartInstance`, `CloneInstance`, `CreateInstance`) into `task_history_db`, and expanding frontend action code mapping for lifecycle operations; aligned running instance status (`● PID: <pid>`), executable tail, and data directory tail inline on the same horizontal row with monospace formatting, enhanced tooltips, and contiguous segmented copy actions; redesigned switch account modal with Royal Violet tokens (`#171025`, `#40285e`, `#1af18d`), circular radio selection indicators, standard `<TierBadge />` integration, search filter clear button, and full keyboard accessibility; and implemented comprehensive E2E verification through `scripts/test-instance-lifecycle-e2e.ps1`. (Thanks to @aukgit)
> Latest version **v4.214.0**: Telegram `/send` step 2 lists each instance with `IDE exe` and `IDE data` subpoints. Only the last path section is shown, not the full absolute path. (Thanks to @aukgit)
> Latest version **v4.213.0**: The Send menu Destination hover stays inside the menu. A browser tooltip on that right-edge menu was clipped to `Destination conversation mo`. Do not put `title` back on those controls. The note is `.ai-memory/rca/20-dispatch-menu-hover-clipped.md`. (Thanks to @aukgit)
> Latest version **v4.212.0**: The Telegram `/send` picker root cause, the file map, the mistakes that reorder FIFO or put the conversation id back in the button, and the do-not-break checklist are in `.ai-memory/rca/18-telegram-project-listing-and-node-cache.md`, indexed from `.ai-memory/what-to-read.md`. (Thanks to @aukgit)
> Latest version **v4.211.0**: Telegram project pages list eight projects at a time, newest conversation first, with the project name and the conversation title. A number selects that row, and Next loads the next eight. The current machine is labeled This machine with a green button. A generated node alias is resolved through GitMap and cached in SQLite for 24 hours. (Thanks to @aukgit)
> Latest version **v4.209.0**: Instance list shows `PID` and the process id, and path cells keep the filename ending. Stop and account switch kill that process and fail if it is still alive. The audit dialog reads `recent_switches`. `agm instances current` refuses to stop, switch, or delete the instance that owns the command. The Switch Account dialog uses the same dark-glass layout as the instance list. (Thanks to @aukgit)

> Latest version **v4.208.0**: Auto-Switch 403 Fallback & Live Quota Sync, Fast-Forward Pool Unblocking, Complete Subprocess Window Suppression (CREATE_NO_WINDOW), Audit Log Clipboard Copy & Call Stack Inspection, Strict Email Masking, Max Backup Prompts Retention Cap & 18pt Updater Console — fixed `retrieveUserQuotaSummary` in `fetch_more.rs` with instant `{}` retry on HTTP 403 to prevent false 100% quota fallbacks when IDE 5-hour quota is depleted; broadcasted `accounts://refreshed` events in `daemon.rs` for live instance card synchronization; unblocked manual Fast-Forward by refining candidate exclusions in `manual.rs` and offloading relaunch verification to detached async tasks; suppressed popup console windows across 14 external subprocess call sites using `CREATE_NO_WINDOW` (`0x08000000`); replaced non-functional `<a download>` DOM clicks with direct clipboard copying (`copyToClipboard`) with visual `Copied!` badges (2000ms) and toast feedback in `AuditPanel.tsx`; added `#[serde(rename_all = "camelCase")]` on `InteractionEntry` and `RecordInteractionInput` to fix blank `METHOD` and `TARGET` audit columns; captured interactive elements and filtered call stacks (`new Error().stack`) in `useAuditRecorder.ts`; logged auto-switch candidate evaluations and cooldown skips to SQLite audit; strictly masked emails (`ris***@***`) across UI and backend logs; added configurable 100-prompt FIFO retention cap in Settings; and configured Win32 console with UTF-8, ANSI escape sequences, and 18pt Consolas font for the updater. (Thanks to @aukgit)
> Latest version **v4.207.0**: Window TitleBar Double-Click Maximize, Compact Status Pill, QuotaProgressBar Polish (-10% Width & 4 Checkpoints), Resizable Email Column, PromptTreeView Toolbar Compaction & New Conversation Flow, Fleet Remote JSON Telemetry — resolved Win32 modal drag-event interception on window header by guarding drag initiation (`e.detail > 1`) and invoking `toggle_maximize_window` directly on double click in `TitleBar.tsx` and `Navbar.tsx`; compacted loose `IDLE` status and `Log` buttons into a single 28px (`h-7`) dark-glass segmented pill capsule in `RecordingIndicator.tsx`; softened QuotaProgressBar ambient/hover neon glow, reduced bar width by 10% (`min-w-[126px]`), aligned checkpoints into 4 equal 20% intervals (`[20, 40, 60, 80]`), and unified filled checkpoints in emerald green `#10b981`; added user-resizable Email column with min/max bounds (`[220px, 550px]`), drag handle with double-click reset, and `localStorage` persistence in `AccountTable.tsx`; compacted Prompt Tree View toolbar into a single contiguous segmented dark-glass group, combined Send and Enqueue into a primary split button with unified dropdown, and introduced a New Conversation dispatch flow with custom name input; extended `fleet_sync.rs` with pure JSON telemetry generation and ingestion supporting Telegram, Email, and Supabase; and documented comprehensive 4-part Root Cause Analysis for historical prompt send and enqueue resolution in `02-spec/21-app/ui-and-functionality-enhancements/02-backend-fleet-and-rca-spec.md`. (Thanks to @aukgit)
> Latest version **v4.206.0**: Prompt Tree Send Dropdown Enqueue, Tree Import/Export (SQLite+JSON), Telegram Button Fixes — Send button dropdown now contains an "Enqueue to FIFO Queue" entry wired to the same dispatch path as the standalone Enqueue button; prompt tree gains copy-all-tree-prompts plus a unified Import/Export section (single-file SQLite DB export/import and JSON export/import); Telegram `cmd:` callback double-slash dispatch bug fixed so send-prompt, find-instance, and credits-check buttons work, 64-byte callback cap guarded, and unknown button taps show a visible notice instead of failing silently (RCAs 14-16). (Thanks to @aukgit)
> Latest version **v4.205.0**: Responsive Layout Hardening, Table Minimum Width Floor, Sticky Column Opaque Shield, Dual-Axis Viewport Scroll, Navbar Responsive Compaction & Window Minima — enforced `min-w-[1060px]` on `src/components/accounts/AccountTable.tsx` and updated `src/pages/accounts/AccountsContent.tsx` to `overflow-auto`, preventing Weekly Quota from collapsing into `ACTIONS` (`WEEKLY QUOCACTIONS`) and surfacing immediate horizontal scrolling on narrow viewports; rendered sticky `ACTIONS` header and cells with 100% solid opaque backgrounds to eliminate text bleed-through; enforced `min-w-[780px]` on `src/components/instances/InstanceTable.tsx`; made `NavLogo.tsx` switch to "AGM" below `lg` (1024px) to reclaim ~95px of navbar space and lowered total navbar minimum width to 670px, ensuring window controls (`btn-window-minimize`, `btn-window-maximize`, `btn-window-close`) and Fast-Forward are never pushed off-screen; configured `"minWidth": 800` and `"minHeight": 520` in `src-tauri/tauri.conf.json`; and added a 4-tier responsive simulator toolbar (`100% Fluid`, `1280px`, `1024px`, `800px`) in `scripts/ui-builder.html`. (Thanks to @aukgit)
> Latest version **v4.204.1**: TitleBar Close Button Padding, Icon-Only Settings, Triangle Capsule Trigger, QuotaProgressBar Overhaul, Modal z-index Fix, Browser UI Builder & Binary Target Alignment — added safe-area right inset padding (`pr-2.5 sm:pr-3`) and centered `overflow-visible` cross in `src/components/layout/TitleBar.tsx` and `src/components/navbar/NavSettings.tsx` to eliminate close button clipping; streamlined `src/components/navbar/NavMenu.tsx` by making Settings icon-only and replacing the trailing round hamburger button with an integrated micro-triangle chevron trigger (`rounded-r-full`); hardened `handleCheckUpdates` with non-silent toast feedback ("Checking for updates..." -> "No updates available (latest version installed)"); overhauled `src/components/accounts/QuotaProgressBar.tsx` with a 10px track, 20px protruding nodes with symmetrical `v` chevrons, zero boundary dots at 0% and 100%, 5 internal splits (`[20, 40, 60, 80, 90]`), 6-stop spatial gradient (`#7f1d1d` to `#1af18d`), 85% resting opacity, and table row hover neon glow; removed `isolation: 'isolate'` from `src/components/navbar/Navbar.tsx` and standardized modal z-index to `z-[99999]`; created browser-based UI builder workbench (`scripts/ui-builder.html` / `npm run ui:builder`) with JSON export/import; and aligned desktop UI executable target to `agm-ui` / `Antigravity Manager (AGM)-ui.exe` while preserving CLI `agm`. (Thanks to @aukgit)
> Latest version **v4.204.0**: Prompt Tree 3-Capsule Segmented Toolbar, Send vs Enqueue Decoupling, One-Shot Rich Copy (Content + Images), CLI Dispatch Parity & CDP DOM Insertion Fallback — separated prompt execution in `src/components/instances/PromptTreeView/DetailHeader.tsx` into independent segmented dark-glass capsules (`[ Send N | v ]` and `[ Enqueue | v ]`) with transparent borderless `<select>` styling to eliminate nested white pill defects; ensured "Send" directly dispatches to the active IDE conversation composer instead of acting as an enqueue operation, while "Enqueue" preserves strict FIFO queueing; implemented `copyPromptWithRichImages` in `src/components/instances/PromptTreeView/clipboard.ts` and added a standalone "Copy All (Content + Images)" button copying dual-MIME `ClipboardItem` (`text/html` with embedded `<img>` tags and `text/plain` markdown fallback) with visual feedback; added `agm queue`, `agm enqueue`, and `agm qe` top-level aliases, `--queue` / `-q` flags, sequence token resolution (`C001`, `P001`, `GM:#1`), and empty prompt validation in `src-tauri/src/bin/agm/main.rs`, `prompt_cmds.rs`, and `common.rs`; hardened `src-tauri/src/modules/repo_db/agy.rs` and `prompt_dispatch.rs` with automatic fallback to loopback CDP DOM insertion (`submit_prompt_via_cdp_for_instance`) when the `agy` CLI binary is absent or fails; and aligned `ssh_cmds.rs` with `SshGitmapStatus`. (Thanks to @aukgit)
> Latest version **v4.203.0**: Enhanced Telegram UI & Bot (HTML Entity Fix, Click-to-Copy Power Commands, Inline Action Buttons, `/balance` Credit Dashboard & CSV/JSON File Exports), GitMap Multi-Machine SSH Control, Auto-Install Helper & RSA-2048 Encrypted Password Vault — replaced unsupported `&nbsp;&nbsp;` and `&bull;` HTML entities in `src-tauri/src/modules/notification_hub/telegram_alerts.rs` and `src-tauri/src/modules/telegram_inbound/formatting.rs` with Unicode figure spaces and bullet glyphs; added interactive inline buttons on switch and liveness alerts, categorized click-to-copy `<code>` command examples in `/help` and `src/components/settings/EmailNotifications/TelegramSection.tsx`, `/balance` (`/credits`, `/quota`) account credit inspection, downloadable `.csv` and `.json` file exports via `send_telegram_document` (`/export` and `--csv`/`--json`), and shorthand `/prompt <machine:instance> <text>` dispatching; implemented RSA-2048 hybrid password encryption (`ag_rsa_v1_`) in `src-tauri/src/utils/crypto.rs` and `src-tauri/src/modules/ssh_manager/` (zero plaintext passwords on disk) with cross-machine encrypted bundle deployment, `agm ssh import-gitmap`, `agm ssh install-gitmap` PowerShell auto-installer, `agm balance [--csv|--json]`, and the `src/components/settings/EmailNotifications/SshGitmapSection.tsx` (`/settings/ssh`) fleet control panel. (Thanks to @aukgit)
> Latest version **v4.202.0**: Automatic CDP Dismissal of Antigravity "Security Notice & Data Use" Modal, Live CDP Conversation Prompt Submission & Startup Purge Persistence Fix — added `src-tauri/src/modules/instance/cdp_onboarding.rs` and `--remote-debugging-port=0` on instance launches to automatically uncheck the Interactions data-sharing checkbox and click **"Accept and continue"** via CDP when switching accounts or launching instances while persisting `hasAcceptedTermsOfService = true` and `shareDataToImproveAntigravity = false` in `app_storage.json` and `state.vscdb`; routed `agm prompt` (`prompt_dispatch.rs`), `project_cmds.rs`, and `repo_db/backup.rs` through `save_or_requeue_prompt` so active and queued prompts are never wiped by `purge_corrupted_running_projects` across CLI calls; added `submit_prompt_via_cdp_for_instance` so dispatched, queued, and restored prompts are submitted directly into the live Antigravity IDE conversation composer; and wired `/settings/:tab` routing for `agm ui goto`. (Thanks to @aukgit)
> Latest version **v4.201.0**: Post-Switch `agy` Silent Auth (`antigravity-oauth-token`), Quota Summary 403 `VALIDATION_REQUIRED` Detection, Synchronous Prompt Restoration & `agm ui record` CLI Audit Recorder — provisioned `antigravity-oauth-token`, `antigravity-cli-oauth-token`, and `antigravity-ide-oauth-token` across isolated instance `.gemini` subdirectories (`antigravity-cli`, `antigravity`, `antigravity-ide`, `cache`) so `agy -p` print mode authenticates silently without hanging on OAuth; detected HTTP 403 `VALIDATION_REQUIRED` on `retrieveUserQuotaSummary` and persisted `validation_blocked = true` so `auto-switch` and `ff` skip verification-blocked accounts; prevented the background `PromptQueueScheduler` from stealing `'backed_up'` prompts mid-switch and synchronously re-dispatched backed-up prompts upon IDE relaunch verification; enabled live `agy` worker replacement on explicit `Send`/`Rerun` with automatic FIFO queue drain on worker completion; added `agm ui record start/stop/status/list/clear/export`; targeted `activeInstanceId` from the Accounts table switch button; and refined progress bar checkpoint glow. (Thanks to @aukgit)
> Latest version **v4.200.1**: Build fix — added the missing `tauri::Emitter` trait import in `src-tauri/src/proxy/server/admin_ui.rs` (E0599 broke the v4.200.0 build). (Thanks to @aukgit)
> Latest version **v4.200.0**: UI interaction audit recorder, `agm ui` CLI bridge, and Telegram `/ui` commands — the recorder captures every UI interaction (click, navigation, input, scroll) with method name, target, trigger, and timestamp in SQLite, with a navbar indicator and filterable audit panel; `agm ui open/goto/ls/scroll/click` drives any UI surface from the CLI with actionable help; backend `/api/ui/*` endpoints navigate the live UI via Tauri events; Telegram `/ui goto/ls/scroll` mirrors the CLI. (Thanks to @aukgit)
> Latest version **v4.232.0**: UI/UX Session Overview Collapsible, 24h Date Standards, Expand Toggle Persistence, Asset Protocol Capability, 5s Readiness Ladder & Account-Centric SSH Discovery — Converted session overview card in `src/components/instances/PromptTreeView/PromptInstructionView.tsx` to be collapsed by default with persistent localStorage memory (`agm_prompt_overview_expanded`) and low-profile purple trigger (`ChevronDown`/`ChevronUp`); deduplicated redundant header and clarified sequential conversation index (`C211` = Conversation #211) and interaction turns (`turns` = user/assistant request-response cycles) in `src/components/instances/PromptTreeView/DetailsModal.tsx`; fixed bug in `src/components/instances/PromptTreeView/DetailTabs.tsx` where switching from Input to Output hid the Expand Full button; implemented centralized date formatting in `src/utils/date.ts` supporting `DD-MMM-YYYY HH:mm` in 24-hour clock using local Windows timezone, with relative time indicators ("ran 5 minutes ago") and duration seconds suppression; added user-configurable `date_format_mode` toggle (`24h` / `12h` / `iso`) in Settings; added Tauri v2 `core:asset:default` capability in `src-tauri/capabilities/default.json` and resolved relative image paths against `repo_path` in `clipboard.ts` and `PromptMarkdownPreview.tsx`; protected queued prompts from stale conversation demotion in `conversationGuards.ts`; synthesized running/queued projects from `active_prompts` in `src-tauri/src/modules/repo_db/tree_phases.rs`, ensuring the left panel is never empty when prompts are in flight; upgraded `src-tauri/src/modules/instance/smart_run.rs` and `src-tauri/src/modules/repo_db/agy.rs` with an adaptive 5-second process stabilization ladder and CDP retry mechanism for cold-started IDE instances, eliminating lost prompts when dispatching to closed IDEs; and upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via SSH, elevating Account Email and Instance Name to primary columns in `FleetMachinesTable.tsx` and `SshGitmapSection.tsx` while demoting raw IP into an interactive hover badge, and adding dynamic instance `<select>` dropdown and 1-click remote prompt dispatching. (Thanks to @aukgit)
> Latest version **v4.199.0**: Prompt send, account switch, and Telegram window commands — Send starts `agy` with `--conversation` and surfaces an immediate exit; Backup and Restore use a native dialog and Restore dispatches; a live Antigravity window counts as a successful switch while a dead saved process id does not; a copy switch leaves the Default login in the shared keyring; Telegram `/ui`, `/shot`, and `/target` open the manager, return a screenshot, and aim the next message. The UI session recorder guideline is written and the recorder is not implemented. (Thanks to @aukgit)
> Latest version **v4.198.0**: Prompt Tree WebView2 OOM Crash Fix (`0xe0000008`), Platform Asset-Gated Update Notification & CLI Prompt Copy/Resend Parity — Resolved the infinite `useEffect` render loop in `src/components/instances/PromptTreeView/usePromptTreeData.ts` and serialized uncached workspace scans in `src-tauri/src/modules/repo_db/tree.rs`, eliminating the 565-thread spike and WebView2 `0xe0000008` crash while keeping warm Prompt Tree loads under `200 ms`; gated **Update Available** notifications in `src-tauri/src/modules/update_checker/channels.rs` and `src/stores/use-update-store.ts` on verified platform installer assets (`.exe`/`.msi`, `.dmg`/`.app.tar.gz`, `.AppImage`/`.deb`/`.rpm`) and restyled `src/components/UpdateNotification.tsx` with high-contrast dark-glass and `#1af18d` accents; nulled detached IDE stdio handles in `src-tauri/src/modules/process/launch.rs` so `agm prompts restore` and `agm auto-switch test` complete in `<200 ms` without pipe hangs; and added `agm prompts copy`, `agm prompts resend`, `agm instances open-ui`, and pure JSON stdout when `--json` is passed. (Thanks to @aukgit)
> Latest version **v4.197.1**: Prompt tree cache and crash log — startup keeps `prompt_tree_cache` for 120 seconds, the CLI tree reads that cache, a tree panic returns a backtrace, and the tree header plus `agm prompts reset-tree` clear only the cache. (Thanks to @aukgit)
> Latest version **v4.197.0**: CI/CD Formatting Gate Fix & Dispatch Hardening — Resolved the `cargo fmt` signature-formatting discrepancy in `src-tauri/src/commands/instance.rs`, restoring green pipeline checks across all OS matrix runners; includes instance error-modal deduplication with poisoned prompt-lock prevention and pending AI end-to-end test cases for prompt dispatch. (Thanks to @aukgit)
> Latest version **v4.196.0**: Accounts Quota Column Layout Fix — Fixed the Accounts table quota layout so the 4H Model Quota and Weekly Quota bars expand to fill their columns instead of floating as narrow right-aligned bars with dead space; 5-slot checkpoints, `#1af18d` progress green, full-width spatial gradient with mask reveal, and compact bar heights preserved — layout-width change only. (Thanks to @aukgit)
> Latest version **v4.238.7**: Multimodal Prompt Backup & Restore E2E Suite, GitMap Delegation Parity, 10s Settling Invariant, SQLite Constraint & File-Lock Fixes — Implemented a comprehensive 25-assertion, 8-gate end-to-end test suite (`scripts/test-agm-gitmap-running-backup-e2e.ps1`) covering running prompt inspection, multimodal deep analysis image attachments (Base64 data URIs, local fixtures, screenshot tokens), Split SQLite snapshot backup (`backup_prompts.db:prompt_backups`), and strict FIFO queue restoration with status transitioning to `'dispatched'`; verified full command parity across `agm running`, `agm backup running`, `agm restore running`, `gitmap agy running`, `gitmap agy ff`, `gitmap backup-running-prompts`, and `gitmap restore-running-prompts`; enforced mandatory `post_switch_resume_delay_seconds = 10` in `src-tauri/src/modules/repo_db/post_switch_resume.rs` ensuring Electron webview, extension host, and CDP debugger endpoints (`DevToolsActivePort`) are ready before prompt re-injection; hardened caller process detection (`agm instances whoami`) with Exit Code 2 refusal on caller stopping or mutation to protect active developer workspaces; fixed SQLite runtime constraint failure in `src-tauri/src/modules/instance/create.rs` (`NOT NULL constraint failed: instance_processes.pid`) preserving historical process ID while clearing active state; implemented `remove_dir_all_force` in `src-tauri/src/utils/fs.rs` with Win32 attribute stripping (`FILE_ATTRIBUTE_NORMAL`), eliminating `os error 5 (Access is denied)` during Chromium cache removal; and passed 8/8 diagnostic probes in `agm errors fix` self-healing engine. (Thanks to @aukgit)
> Latest version **v4.234.0**: GitMap Node and Prompt System Issues with Instance Management Sync — Added `deduplicate_connections()` in `src-tauri/src/modules/ssh_manager/connections.rs` to eliminate duplicate cluster node records; resolved local node identity in `src-tauri/src/modules/ssh_manager/exec.rs` by matching local IPv4 against configured SSH connections (resolving local alias as `w1`), ensuring local instances are not excluded or misattributed to remote nodes; added dual-key matching (`machine_alias` or `ip_address`) in `src/components/settings/EmailNotifications/SshGitmapSection.tsx` to accurately display instances and bound accounts; restored multi-VM collision shielding guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` (`on_machine_picked`, `on_instance_picked`), ensuring only targeted nodes respond to machine/instance callbacks while non-target nodes drop replies cleanly to prevent multi-daemon message collisions; refined `is_current_machine` in `src-tauri/src/modules/telegram_inbound/node_label.rs` to prioritize authentic local hostnames (`detect_machine_name`) and local IPs before querying cluster nodes; standardized remote CLI dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to `agm prompt send --instance <inst> [--repo <repo>] "<prompt>"`; centralized conversation status detection in `src/components/instances/PromptTreeView/conversationGuards.ts` (`getConversationStatus`) and re-exported through `ConversationNode.tsx`; enforced strict 3-tier ordering in `ProjectNode.tsx`, `tierClassification.ts`, and `PromptTreeViewModal.tsx` so running prompts stay strictly at Tier 1 on top, queued prompts at Tier 2 with amber clock badges, and completed/historical prompts ordered by timestamp descending; and added document visibility change detection and 5-minute background throttling in `src/pages/instances/useInstancePage.ts` and `src/stores/useAccountStore.ts`, coalescing IPC refresh requests and eliminating CPU spikes during inactive background state. (Thanks to @aukgit)
> Latest version **v4.233.1**: Prompt Tree View Reliability, GitMap Node Discovery & Clipboard Pipeline — Centralized context menu state into `TreePanel.tsx` with a single `<PromptTreeContextMenu>` and transparent portal backdrop; added outside capture event listeners (`pointerdown`, `contextmenu`, `keydown`, `scroll`, `blur`) to dismiss immediately on any outside interaction; introduced `isConversationOrTurnQueued` to auto-expand queued child turns, render amber `QUEUED` badges with clock icons, and prioritize queued items in Tier 2 sorting; prioritized `gitmap cluster nodes --json` and `export-json` in `src-tauri/src/modules/ssh_manager/sync.rs` for reliable cluster node discovery across Windows/Linux machines; corrected offline SSH node quota fallbacks to 0.0% instead of 100.0% in `src-tauri/src/modules/ssh_manager/exec.rs`; added request debouncing and in-flight promise coalescing to `useAccountStore.ts` to prevent redundant IPC bursts; introduced `read_image_as_base64` in `src-tauri/src/commands/cmd_files.rs` and registered in `src-tauri/src/lib/run.rs` to read local image files securely with path canonicalization, system path protection, and a 100MB ceiling; and decoded Base64 data directly into binary PNG Blobs (`base64ToBlob`) in `src/components/instances/PromptTreeView/clipboard.ts`, eliminating offscreen HTML canvas CORS errors and WebView2 `SecurityError` canvas tainting, with rich event JSON clipboard copying and toast feedback. (Thanks to @aukgit)
> Latest version **v4.233.0**: GitMap Node, Prompt System & Instance Management Synchronization — Upgraded `get_cluster_instances_and_balances` in `src-tauri/src/modules/ssh_manager/exec.rs` to query real multi-instances and bound Google account emails via GitMap discovery and cluster metadata; forwarded identity public key `-i <pubkey>` in `fix_node_auth_via_gitmap` across `mesh.rs`, `commands/ssh.rs`, and CLI handlers, removing hardcoded single-node bypasses in favor of generalized credential vault auto-repair; removed fatal callback drop guards in `src-tauri/src/modules/telegram_inbound/send_flow.rs` so the receiving Telegram bot coordinator accurately routes prompts across cluster nodes; standardized remote dispatch syntax in `src-tauri/src/modules/telegram_inbound/injection.rs` to canonical `agm prompt send <project> --instance <inst> "<prompt>"` with safe quoting and multi-line escaping; inverted prompt dispatch in `src-tauri/src/modules/repo_db/agy.rs` to submit directly via Chrome DevTools Protocol (CDP) into active IDE windows before falling back to spawning external `agy` CLI background workers; wired direct 1-click execution for Backup and Restore in `src/components/instances/PromptTreeView/PromptTreeViewModal.tsx` to `coreActions.handleBackup()` and `coreActions.handleRestore()`; added spinning animation on the Refresh icon during active sync; provided instant visual notification feedback on Reset and Refresh; restricted animated skeleton loaders in `src/components/instances/PromptTreeView/TreePanel.tsx` and `PromptTreeViewModal.tsx` strictly to initial cold starts (`isLoading && treeData.length === 0`), keeping the active tree rendered during background polling to eliminate visual flashing; enforced 3-tier deterministic sorting in `ProjectNode.tsx` and `tierClassification.ts` with `RUNNING` prompts strictly at Tier 1, `QUEUED` prompts at Tier 2 with explicit amber badges, and timestamp descending at Tier 3; removed redundant `"User Prompt"` badge and duplicate `{instanceNameDisplay}` text token from `DetailHeader.tsx`, relocating deep runtime telemetry to the Details modal; added `relative z-40` to `ModalHeader.tsx` container and `relative z-50` to `ImportExportMenu.tsx` so menus stack cleanly above banners and header elements; implemented atomic `LAST_SUPABASE_ERROR_TIMESTAMP` circuit breaker in `src-tauri/src/modules/supabase_sync/types.rs` with a 300-second cooldown on network errors across `fleet.rs`, `query.rs`, and `release.rs`; relaxed fleet polling to 300s in `useFleetMachines.ts`; added a 30-second TTL cache for account listings in `src-tauri/src/modules/account/crud.rs` to stop repetitive reads of 46 JSON account files; removed circular cache invalidation sabotage in `src-tauri/src/modules/repo_db/detection.rs:254` to eliminate console log flood; replaced ambiguous status codes with an emerald pulse badge (`PID: <pid>`) and aligned executable tail and data directory tail together on the exact same horizontal flex row (`.../Antigravity.exe · .../data`) in `src/components/instances/table/InstanceRowCells.tsx`; upgraded `src/pages/instances/SwitchAccountDialog.tsx` with cohesive dark-glass UI/UX; enforced process tree killing via `taskkill /F /T /PID` before stopping or switching accounts in `src-tauri/src/commands/instance.rs`; hardened `detect_caller_instance_id` to protect the developer's active workspace session (`default-copy-5597`); and verified with transient test sandboxes (`agm instances test-create` and `agm instances test-clean`). (Thanks to @aukgit)
> Latest version **v4.188.0**: Auto-Switch & Fast-Forward PID Contamination Fix, CLI Version Downgrades, IDE Executable Path & Headless Fleet Sync — Eliminated cross-instance PID contamination in `pid_scan.rs` and unblocked candidate evaluation for sub-100% quota accounts; enabled CLI version downgrades and rollbacks (`agm update <ver>`) with automatic 2-part SemVer normalization; optimized launch latency by eliminating synchronous 6s blocking sleeps in `wait_for_instance_prompt_channel`; added dedicated IDE Executable path column in instance list view; introduced decentralized headless fleet synchronization over Email and Telegram without Supabase; and audited/eliminated silent error discards across instance lifecycle and frontend settings. (Thanks to @aukgit)
> Latest version **v4.187.0**: Full Task Manager Termination & Profile Switch Spawn Fallback — Enforced full application and background thread process termination in Windows Task Manager on window close and tray quit by routing all teardowns through `exit_app_fully(&AppHandle)` and cleanly halting Cloudflared tunnels, Admin Server listeners, and Proxy instances before invoking `std::process::exit(0)`; resolved profile and account switch crashes (`Access is denied (os error 5)`) in restricted Windows Job Objects with a 4-stage launch fallback ladder (direct breakaway fallback -> detected base executable -> `cmd.exe /s /c start` -> PowerShell `Start-Process` with `ShellExecuteEx`); upgraded Telegram notifications with structured HTML cards, `<pre><code>` prompt blocks, native inline copy buttons, free image hosting, and standalone Fleet Identity discovery (`/fleet`, `/whoami`, `/instances`, `/accounts`). (Thanks to @aukgit)
> Latest version **v4.186.0**: Update CLI Command & Pipeline Fixes — Universal Windows self-delegation for `agm update` to detached temp executables (`%TEMP%\agm-updater\agm-update-cli-<pid>.exe`) with immediate caller exit code 0 to prevent Windows executable image lock (`os error 5` / sharing violation `0x20`); enhanced installer process discovery for `antigravity-manager.exe` and `agm.exe`; fixed prompt pipeline dispatch and FIFO queueing (`agm prompt send`, `agm prompt queue`, `qe` alias, and `agm instance prompt`) with destination conversation target resolution (`--conversation <id>`, `--new`, `--existing`), working directory auto-fallback, and Tier 2 graceful degradation; unified Prompt Tree View UI with an inline segmented capsule selector for Existing Conv vs New Conv; resolved switch and auto-rotation prompt loss by expanding backup query status coverage (`running`, `in_flight`, `dispatched`), enabling cross-instance prompt re-attribution on profile rotation, and restoring `backed_up` prompts upon recovery. (Thanks to @aukgit)
> Latest version **v4.185.0**: Comprehensive compiler and pipeline error resolution — resolved 600+ compilation and visibility errors from god-module refactorings across proxy rate limiting, signature cache, token manager, session store, upstream client, and mappers; restored Axum 0.7 ServiceExt router compatibility; added Windows process flag gating; pre-flight cargo fmt and manifest synchronization verified clean. (Thanks to @aukgit)
> Latest version **v4.183.0**: Switch/fast-forward reliability hardening — new `close_instance_verified` kills, polls the OS process table and retries stragglers instead of bricking on surviving orphan PIDs; credential injection now retries over the dying IDE's SQLite-lock release window; fast-forward errors are captured into the Error Manager for diagnostics. Also fixed the clipboard `Document is not focused` warnings in the prompt send flow via the shared copy helper with execCommand fallback. (Thanks to @aukgit)
> Latest version **v4.182.0**: Restart & fast-forward reliability fix — restart now hard-closes the IDE, verifies the kill, refreshes the bound account token and relaunches with the same account; fast-forward verifies the close before credential injection. Best-account tier multipliers are now configurable (Ultra ×4, Pro ×2, Free ×1 defaults) via a new Settings → Algorithm section; scoring spec updated; Remote Fleet section verified with tests. (Thanks to @aukgit)
> Latest version **v4.181.0**: Quota UI compaction with stronger progress bars; Tauri window ACL fix for `set_decorations`/`set_background_color`; Supabase connectivity diagnostics with honest `Unreachable` badge; Settings General tab redesign (segmented Theme pill, Launch-at-Startup toggle); Send/Queue silent no-op hardening; Windows instance executable staleness refresh on base updates; navbar segmented capsule with a pinned Check-for-Updates section. (Thanks to @aukgit)
> Latest version **v4.180.0**: Release pipeline completion — re-ran the v4.179.0 macOS universal artifact upload after a transient GitHub artifact-service timeout so the release carries the full asset set; CI green across all platforms. No code changes. (Thanks to @aukgit)
> Latest version **v4.177.0**: Smart Process Cache Closed-PID Recovery, Foreign Key Purge Resilience, Adaptive Queue Ticker & UI Tag Compaction — initialized `warm_up_smart_process_cache` on startup to pre-warm cached PIDs across instances; added closed-PID recovery and zero-relaunch guarantee in `ensure_instance_running_smart` to eliminate repeated IDE launches; fixed SQLite foreign key constraint failure in `purge_corrupted_running_projects` and purged stale un-namespaced rows; added adaptive FIFO background prompt queue scheduler (5s active / 30s idle) with canonical `resume_task_document(...)` serialization; compacted Prompt Tree View sidebar tags (removing `{totalProjectPrompts} prompts` and `{conv.step_count} stp`, setting project actions to hover-only), stripped bracket noise (`#P001`, `C001`), eliminated redundant client-side focus race condition, and eradicated ghost running indicators on idle projects and instance cards. (Thanks to @aukgit)
> Latest version **v4.176.0**: Smart Process Cache Warm-Up, Closed-PID Recovery & Adaptive Queue Ticker — initialized `warm_up_smart_process_cache` on startup to pre-warm cached PIDs across instances; added closed-PID recovery and zero-relaunch guarantee in `ensure_instance_running_smart` to eliminate repeated IDE launches; added adaptive FIFO background prompt queue scheduler (5s active / 30s idle) with canonical `resume_task_document(...)` serialization; compacted Prompt Tree View sidebar tags (removing `{totalProjectPrompts} prompts` and `{conv.step_count} stp`, setting project actions to hover-only), stripped bracket noise (`#P001`, `C001`), and eradicated ghost running indicators on idle projects and instance cards. (Thanks to @aukgit)
> Latest version **v4.179.0**: Accounts table compact density pass — reduced row and header cell padding, thinner quota bars with proportionally smaller checkpoint markers and tighter label spacing (accounts table only); gradient, checkpoints, and behavior unchanged. (Thanks to @aukgit)
> Latest version **v4.178.0**: CI fix — removed a duplicate `version` entry in `src-tauri/Cargo.lock` that broke cargo parsing and failed all Rust jobs, and hardened the version-bump script so it can never reintroduce the duplicate. (Thanks to @aukgit)
> Latest version **v4.175.0**: UI refinement — centralized semantic color tokens, quota bars rebuilt as one continuous red→green scale, account/email hover fixed, calmer active-instance treatment; plus Prompt Tree Send/Queue fix — the buttons reported false success while the IDE dispatch silently failed, now surfacing real errors. (Thanks to @aukgit)
> Latest version **v4.174.0**: CI green — fixed `error[E0609]: no field 'instance' on type 'CliContext'` at two call sites in `src-tauri/src/bin/agm.rs` (stale field name left by the instance-process-cache refactor; corrected to `instance_id`), and re-enabled GitHub Actions on the fork (GitHub had auto-disabled workflows because it is a fork) — CI now passes all 7 jobs. (Thanks to @aukgit)
> Latest version **v4.173.0**: Updater honesty fixes — corrected the in-app update checker's download link (it pointed at the upstream `lbjlaq` repo instead of this fork) and removed phantom binary-asset URLs from `releases-manifest.json` for the notes-only releases v4.168.0–v4.172.0 so the installer no longer chases dead download links. (Thanks to @aukgit)
> Latest version **v4.172.0**: Smart Multi-Instance Process Cache Multi-PID Survival & Deep Transcript Scan — enhanced `check_cached_pid_alive` with surviving child PID promotion, Windows 8.3 short-path expansion (`GetShortPathNameW`), path normalization, and zero-relaunch guarantee; guarded against killing running IDE sessions in `launch_instance_inner_with_extra_workspaces`; deepened reverse transcript scan to 25 lines with telemetry noise immunity and 45s prompt TTLs to eradicate ghost running items; stripped literal bracket tokens (`[Collapse Full Text]`, `[Expand Full Text]`) and word count clutter across Prompt Tree View. (Thanks to @aukgit)
> Latest version **v4.171.0**: Smart Multi-Instance Process Cache & Relaunch Prevention — resolved instance relaunch bug via canonical instance ID resolution (`is_instance_process_running_smart`), closed-PID OS re-scan, and zero-relaunch guarantee; unified `enqueue_prompt` IPC handler with strict FIFO queueing; eradicated ghost running indicators (`WHERE status IN ('queued', 'pending')`); compacted Prompt Tree View badges (`P001 · #1`, `C001 · <cid>`) and stripped bracket clutter; 100% verified via host-shielded E2E testing. (Thanks to @aukgit)
> Latest version **v4.170.0**: New Toolchain settings page — install dev tools from the UI via backend endpoints with live progress; installer script v3 adds stackable profiles, `--dry-run` preview, `--troubleshoot`, `--ssh` remote install, and new items (Node.js, pnpm, sccache, cargo-watch, Tauri CLI, gh) for shell and PowerShell. (Thanks to @aukgit)
> Latest version **v4.169.0**: Fixed CI build failure (`E0428`) from duplicate `enqueue_prompt` / `enqueue_prompt_for_instance` definitions — removed the stale command variants, migrated `agm prompts enqueue` to the canonical `enqueue_prompt_for_instance_full` route (GUI/CLI parity), and de-duplicated the command registration in `lib.rs`. (Thanks to @aukgit)
> Latest version **v4.168.0**: Curated the model inventory — removed the outdated Gemini 2.5 series, kept the nano tier (gemini-3.1-flash-lite), added 5 newer open-source models in a new 'Open Source' group (gpt-oss-20b, deepseek-v4, qwen3-max, qwen3-8b, llama-4-maverick); every model now carries priority, capabilities, and speed annotations. (Thanks to @aukgit)
> Latest version **v4.167.0**: Implemented smart multi-instance process cache with targeted PID vitality checks, preventing duplicate IDE window launches on prompt dispatch; wired `enqueue_prompt` IPC command with strict FIFO queueing; eliminated false-positive running prompt indicators using terminal planner state detection; compacted prompt tree view badges (`P001 · #1`, `C001 · <cid>`) and stripped bracket clutter; expanded AGM CLI prompt verbs (`status`, `send`, `running`, `queue`) with unified JSON envelopes and 100% verified safe host-shielded E2E testing. (Thanks to @aukgit)
> Latest version **v4.166.0**: Implemented 3-step smart instance process caching with closed-PID OS re-scan and multi-instance process counting; aligned `--user-data-dir` CLI prompt dispatch with cross-platform clipboard synchronization; eliminated bracket noise in Prompt Tree View; eradicated ghost running indicators; and delivered CLI parity for `agm prompts` and `agm doctor`. (Thanks to @aukgit)

👉 **[View Full Changelog → CHANGELOG_EN.md](CHANGELOG_EN.md)**


<details open>
<summary><b>👥 Contributors & Maintainer</b></summary>

<a href="https://github.com/aukgit"><img src="https://github.com/aukgit.png" width="60px" style="border-radius: 50%; margin: 8px 0;" alt="aukgit" title="aukgit (Maintainer & Lead Developer)"/></a>

<p><em>Maintained and developed by <a href="https://github.com/aukgit"><strong>Md. Alim Ul Karim (aukgit)</strong></a></em></p>

</details>

<details>
<summary><b>🤝 Special Thanks - Click to expand</b></summary>

This project has referenced or learned from the ideas or code of the following excellent open-source projects during its development (in no particular order):

*   [learn-claude-code](https://github.com/shareAI-lab/learn-claude-code)
*   [Practical-Guide-to-Context-Engineering](https://github.com/WakeUp-Jin/Practical-Guide-to-Context-Engineering)
*   [CLIProxyAPI](https://github.com/router-for-me/CLIProxyAPI)
*   [OmniRoute](https://github.com/diegosouzapw/OmniRoute)
*   [antigravity-claude-proxy](https://github.com/badrisnarayanan/antigravity-claude-proxy)
*   [aistudio-gemini-proxy](https://github.com/zhongruichen/aistudio-gemini-proxy)
*   [gcli2api](https://github.com/su-kaka/gcli2api)
*   [agent-vibes](https://github.com/funny-vibes/agent-vibes)

</details>

*   **License**: **CC BY-NC-SA 4.0**. Strictly for non-commercial use.
*   **Security**: All account data is encrypted and stored locally in a SQLite database. Data never leaves your device unless sync is enabled.

---

<div align="center">
  <p>If you find this tool helpful, please give it a ⭐️ on GitHub!</p>
  <p>Managed and enhanced by Md. Alim Ul Karim &amp; Riseup Asia LLC · Special thanks and full credit to the original upstream project <a href="https://github.com/lbjlaq/Antigravity-Manager">lbjlaq/Antigravity-Manager</a> and original contributors.</p>
  <p>Copyright © 2024-2026 Antigravity Team.</p>
</div>
