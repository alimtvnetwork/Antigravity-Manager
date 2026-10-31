# Claude 4.6 Adaptive Thinking Mode: A Guide to Testing Examples

In order to verify the integration effect of Claude 4.6 Adaptive Thinking mode, especially the effectiveness of the `effort` parameter and the automatic adjustment of Token limits, please refer to the following test scenario.

## 1. Verify Adaptive mode activation and Effort control

This test verifies that the system can correctly pass the `thinking: { type: "adaptive", effort: "..." }` parameter and observes differences in model behavior.

### Preconditions
* Make sure to use a model ID that supports Adaptive Thinking (such as `claude-opus-4-6-thinking` or a mapped ID).
* Set "Thinking Budget" mode to **"Adaptive"** in settings.

### Test command example

#### Scenario A: Low Effort (low intensity thinking)
* **Configuration**: Set Effort to `Low`.
* **Instructions**: `Write a Hello World program and compile it with Rust. `
* **Expected results**:
    * Thinking blocks should be short and the model thinks it is a simple task that does not require in-depth reasoning.
    * The response speed is faster.

#### Scenario B: High Effort (high-intensity thinking)
* **Configuration**: Set Effort to `High`.
* **Instructions**: `Please analyze Rust’s async/await state machine generation mechanism in detail and compare it with Go’s Goroutine scheduling model. Please deduce the difference in memory overhead between the two through the chain of thought. `
* **Expected results**:
    * Thinking blocks should be very long and detailed (maybe over 5k tokens).
    * The model attempts to perform in-depth comparative analysis and inference.
    * **Key Verification Point**: Check Antigravity logs to confirm that `generationConfig` contains `thinkingConfig: { type: "adaptive", effort: "high" }`.

---

## 2. Verify Adaptive state maintenance in multiple rounds of dialogue

Verify that Adaptive mode continues to take effect across multiple rounds of conversations and that the Token limit (128k) works properly.

### Scenario: Complex algorithm design iteration

#### Round 1: Initial Design
* **instruction**:
    ```bash
    claude "Design a distributed high-concurrency flash sale system. Core issues such as cache consistency, inventory anti-overselling, and anti-brushing interfaces need to be considered. Please use High Effort for in-depth thinking."
    ```
* **Verification Point**:
    * Design documents containing architectural diagrams and detailed logic were generated.
    * The Thinking process details the trade-offs of different options (such as Redis Lua vs database pessimistic locking).
    * Verify in the response headers or logs that `maxOutputTokens` is raised to **128,000** (or higher) to accommodate long output.

#### Round 2: Solution Challenge (Simulated User Feedback)
* **instruction**:
    ```bash
    claude "In your design, if a split-brain occurs in the Redis cluster, how to ensure the strong consistency of the inventory data? Please rethink and correct the plan."
    ```
* **Verification Point**:
    * The Thinking block continues deep reasoning, analyzing the suitability of Redlock or other consensus algorithms.
    * **Signature verification**: Ensure that the signature verification of Thinking Block in multiple rounds of dialogue passes (no `Invalid signature` error is reported).

#### Round 3: Code Implementation
* **instruction**:
    ```bash
    claude "Please provide the Rust code implementation of the core logic of inventory deduction."
    ```
* **Verification Point**:
    * Can generate code that conforms to previous design ideas.
    * Under high context pressure, does the system automatically trigger Thinking stripping (if dynamic stripping is configured), or can it continue to generate the complete history normally.

---

## 3. Verify automatic switching between Budget mode and Adaptive mode

This test verifies that the backend converts parameters correctly when the user switches between Fixed Budget and Adaptive modes.

### Test process
1. **Set to Fixed Budget**: Select "Custom" in settings and set Budget to `16384`.
    * Send a request.
    * *Validation*: Backend requests should only contain `thinkingConfig: { budget: 16384 }` and **should not contain** `effort`.

2. **Switch to Adaptive**: Select "Adaptive" in settings and set Effort to `Medium`.
    * Send a request.
    * *Validation*: Backend requests should only contain `thinkingConfig: { type: "adaptive", effort: "medium" }` and **should not contain** `budget`.

---

## 4. Debugging suggestions

When running the above test, it is recommended to turn on the Debug log to observe the parameter passing:

```bash
RUST_LOG=debug npm run tauri dev
```

Search keywords in logs:
* `[Claude-Request]`: View the converted request body.
* `thinkingConfig`: Confirm configuration injection.
* `maxOutputTokens`: Confirm the adjustment of the Token upper limit.
