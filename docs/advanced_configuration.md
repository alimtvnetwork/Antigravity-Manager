# Advanced Configuration and Experimental Features (Advanced Configuration)

Antigravity v3.3.35 introduces `ExperimentalConfig`, which is a set of experimental function switches that are turned on by default, aiming to improve the robustness and compatibility of the system. These configurations are located in `src-tauri/src/proxy/config.rs` and are not currently exposed to the UI interface.

## Feature list

### 1. Double-layer signature cache (Signature Cache)
* **Configuration item**: `enable_signature_cache`
* **Default value**: `true`
* **Description**: When enabled, the system will cache the mapping relationship between `ToolUse ID` and `Thought Signature`.
* **Function**: Solve the problem that some clients (such as Claude Desktop CLI, Cherry Studio) may lose historical Tool Call signatures in multiple rounds of conversations. When the upstream API reports an error "Missing signature", the system can automatically restore it from the cache to avoid interruption of the conversation.

### 2. Tool Loop Recovery
* **Configuration item**: `enable_tool_loop_recovery`
* **Default value**: `true`
* **Note**: When enabled, the system will monitor the conversation status in real time and detect the "infinite loop" mode.
* **Trigger condition**: Continuous `ToolUse` -> `ToolResult` loop is detected, and the `Thinking` block is missing from the `Assistant` message (usually stripping due to signature verification failure).
* **Behavior**: Automatically inject synthetic messages (`Assistant: Tool execution completed.` -> `User: Proceed.`) to break the infinite loop and force the model to enter the next round of thinking.

### 3. Cross-Model Checks
* **Configuration item**: `enable_cross_model_checks`
* **Default value**: `true`
* **Description**: Prevent signature errors caused when switching different series of models (such as Claude -> Gemini) in the same session.
* **Function**: When it is detected that the signature in the historical message belongs to an incompatible model family (such as `claude-3-5` vs `gemini-2.0`), the system will automatically discard the old signature to prevent the API from rejecting the request.

## Custom configuration

Currently these configuration items can be adjusted by modifying the `default_true` default value in `src-tauri/src/proxy/config.rs`, or waiting for future versions to be integrated into the "Settings -> Advanced" interface.

```rust
// src-tauri/src/proxy/config.rs

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExperimentalConfig {
    #[serde(default = "default_true")]
    pub enable_signature_cache: bool,
    // ...
}
```
