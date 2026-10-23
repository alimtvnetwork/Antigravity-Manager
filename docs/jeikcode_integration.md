# Guide to connecting JeikCode to Antigravity-Manager

[JeikCode](https://github.com/jeikl/JeikCode) is a modern terminal AI Coding Agent tool (based on the native Rust kernel) created by the core maintainer of this project. It is deeply compatible with the Antigravity-Manager gateway protocol. Through the in-depth collaboration between the two, an ultra-high KV-Cache cache hit rate of **95%+**, full-protocol multi-modal interaction, and the ultimate inference streaming experience can be achieved.

---

## 🚀 Method 1: One-click synchronization through GUI interface (most recommended)

If you are running the Antigravity-Manager desktop client or web management backend, you can use the built-in one-click synchronization function **Zero-configuration second-level access**:

1. **Start service**:
   - Open Antigravity-Manager and make sure the proxy service is started on the **API Anti-Generation** page (green running status is displayed in the upper right corner).
2. **Enter one-click configuration**:
   - Click the top navigation tab **`>_ Agent tool one-click configuration`**.
3. **One-click synchronization of JeikCode**:
   - In the first card **JeikCode** (marked with `🌟 Recommended · Best Matched`):
     - The system has automatically detected the locally installed JeikCode path and the currently running gateway address (such as `http://127.0.0.1:8045/v1` or a custom port).
     - Select the model you want to use by default in the **Select synchronization model** drop-down list (for example, `gemini-3.8-flash-high` or `claude-sonnet-4-6-thinking`).
     - Click **`🔄 to synchronize configuration`** now.
4. **Start and use JeikCode**:
   - Open the terminal and run `jeikcode` directly to start the native TUI interactive interface.
   - **🌐 Highly recommended to experience WebUI**: Enter **`/webui`** in the terminal TUI, you can open the beautiful web page in the browser with one click, and enjoy a more intuitive and comfortable visual multi-round programming and Diff experience, right out of the box!

> [!TIP]
> **Automatic configuration content description**:
> After clicking "Sync Now", the system will automatically update `~/.jeikcode/config.toml`, automatically inject the `antigravity-manager` provider account, pre-configure the gateway core model list (including optimization parameters such as context windows, thinking gears, image support), and generate backup configurations, supporting one-click restoration at any time (click the restore icon in the lower left corner of the card).

---

## 🛠️ Method 2: Manually modify the configuration file (Headless/Linux/Docker recommended)

For users without a desktop environment, remote servers, Docker deployments, or who wish to manually fine-tune the configuration, the JeikCode configuration file can be edited directly:

- **Configuration file path**:
  - Linux / macOS：`~/.jeikcode/config.toml`
  - Windows: `C:\Users\<your username>\.jeikcode\config.toml`
  - (or `config.toml` in the directory specified by the environment variable `JEIKCODE_HOME`)

Add or replace the following in `config.toml` (replace `http://127.0.0.1:8045` and `sk-your-api-key` with your real gateway address and API Key):

```toml
#The model used by default
default_model = "gemini-3.8-flash-high"
default_provider = "gemini-3.8-flash-high"

# ─── Provider account definition ───
[provider_accounts.antigravity-manager]
provider = "anthropic" # Anthropic protocol access using gateway in-depth optimization
base_url = "http://127.0.0.1:8045/v1" # Antigravity-Manager listening port and path
api_key = "sk-your-api-key" # API Key configured by the gateway

# ─── Model definition (you can add more as needed) ───

# Gemini 3.8 Flash (high thinking level, recommended for daily use)
[models."gemini-3.8-flash-high"]
account = "antigravity-manager"
model = "gemini-3.8-flash-high"
context_window = 1048576
image_input = true
reasoning_model = true
reasoning_history = "exclude"
reasoning_effort = "high"
reasoning_levels = ["low", "medium", "high"]

# Claude 3.7 Sonnet Thinking (deep logic and reconstruction tool)
[models."claude-sonnet-4-6-thinking"]
account = "antigravity-manager"
model = "claude-sonnet-4-6-thinking"
context_window = 200000
image_input = true
reasoning_model = true
reasoning_history = "exclude"
reasoning_effort = "high"
reasoning_levels = ["low", "medium", "high"]

# Claude 3.7 Opus Thinking (ultra-high complexity architecture planning)
[models."claude-opus-4-6-thinking"]
account = "antigravity-manager"
model = "claude-opus-4-6-thinking"
context_window = 200000
image_input = true
reasoning_model = true
reasoning_history = "exclude"
reasoning_effort = "high"
reasoning_levels = ["low", "medium", "high"]
```

---

## ⚡ Method 3: Temporarily activate terminal environment variables

If you want to quickly test ad hoc or call it in a script, you can directly override the startup via environment variables:

```bash
#Set the gateway address and authentication key (Anthropic compatible protocol)
export ANTHROPIC_BASE_URL="http://127.0.0.1:8045"
export ANTHROPIC_API_KEY="sk-your-api-key"

# Start and specify the model
jeikcode --model claude-sonnet-4-6-thinking
```

---

## 🌟 Core features and in-depth tuning

### 1. Deep collaboration: 95%+ KV-Cache cache hit
Antigravity-Manager has **Prefix Stability** and **Dynamic Thinking Stripping** mechanisms, and combined with JeikCode's native Rust context management and read-only tool concurrent calls, can increase the code prompt word cache hit rate of large projects to **more than 95%**, significantly shorten the first word response delay (TTFT) and reduce Token consumption.

### 2. Think intensity adjustment (`/effort` & `Ctrl+T`)
In the JeikCode Interactive Interface (TUI):
- Enter `/effort` or directly press the shortcut key **`Ctrl + T`** to quickly switch between `low`, `medium`, and `high` thinking gears.
- The Antigravity-Manager gateway will automatically normalize the thinking budget (Thinking Budget / Thinking Level) in the request body and then dispatch it to the corresponding upstream model.

### 3. Interactive switching between multiple models and providers
- Enter **`/model`**: quickly switch between different models under the current gateway (such as switching from `gemini-3.8-flash-high` to `claude-opus-4-6-thinking` in seconds).
- Enter **`/provider`**: View the connection details and context window status of the current gateway.

### 4. 🌐 Highly recommended: Use JeikCode WebUI for the best programming experience
In addition to the extremely responsive native Rust terminal TUI, JeikCode also has a built-in modern WebUI with a strong sense of technology:
- **One-click startup**: Run `jeikcode` directly on the terminal. After starting TUI, type **`/webui`** in the input box, and the system will automatically open the visual interface in the default browser.
- **Panorama Visualization**: Compared with pure character terminals, WebUI provides richer and more intuitive code Diff highlight comparison, file tree expansion, multi-round dialogue folding and Markdown rendering, which greatly improves the comfort and work efficiency of complex project code reading and review!

---

## ❓ Frequently Asked Questions (FAQ)

### Q1: Prompt `Connection Refused` or unable to connect?
- Check if the API anti-representation service is in "Running" status in the Antigravity-Manager client.
- Confirm whether the port numbers match: The default commonly used port on the client is `8045` or `8046`. Please refer to the client interface or Docker mapped port, whichever comes first.
- If running JeikCode in Docker / WSL, you need to replace `127.0.0.1` with the host LAN IP or `host.docker.internal`.

### Q2: Prompt `401 Unauthorized`?
- Confirm that `api_key` is exactly the same as `API_KEY` set in Antigravity-Manager's **API Antigeneration** page.
- Note: If you enable the independent management background login password `WEB_PASSWORD`, API requests still need to use `API_KEY` for authentication.

### Q3: How to customize and add other experimental models?
- Add a new model block under `[models."<your alias>"]` in `config.toml`, point `account` to `"antigravity-manager"`, and `model` fill in the model name supported by the upstream (such as `gemini-3-flash`, `gemini-2.5-pro`). It will take effect immediately after saving, without restarting the JeikCode daemon process.
