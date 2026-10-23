# Stability and Search Functionality: A Guide to Testing Examples

In order to verify the recent fixes to API 400 errors and "Search File Error", you can run the following command in Claude CLI (Claude Code) for actual testing.

## 1. Verify search tool self-healing (Grep/Glob Fix)

In response to the previous "Error searching files" issue, these instructions will trigger the `Grep` and `Glob` tool calls and verify that the parameter mapping is correct.

### Test command example
* **Command A**: `Search the current directory for Rust files containing "fn handle_messages". `
    * *Verification point*: Check whether the proxy can correctly map `query` to `pattern` and inject the default `path: "."`.
* **Command B**: `List all .rs files in the src-tauri directory. `
    * *Verification point*: Verify whether the `Glob` tool name is correctly recognized and the path filtering logic is normal.

---

## 2. Verify protocol sequence and signature stability (Thinking/Signature Fix)

For previous `Found 'text'` and `Invalid signature` 400 errors.

### Test command example
* **Instruction A (Inference + Search)**: `Analyze the core logic of processing cloud requests in this project, summarize it in the order of calls, and give Grep search evidence of key lines of code. `
    * *Verification Point*: Verify that the order of blocks is correct in the "Think -> Tool Call -> Result -> Continue Thinking" loop.
* **Command B (history retry)**: Switch models frequently in long conversations and observe whether the system silently repairs the signature when a 400 error is reported and tries again.

---

## Appendix: In-depth error comparison and repair plan

| Error category | Specific error code (Error Detail) | Repair/coping logic adopted by the agent |
| :--- | :--- | :--- |
| **Message flow order violation** | `If an assistant message contains any thinking blocks... Found 'text'.` | **Fixed**: `streaming.rs` no longer allows illegal appending of thought blocks after text blocks. |
| **thought signature mismatch** | `Invalid signature in thinking block` | **Fixed**: Prefer retaining original names to protect Google backend signature verification. |
| **Thought signature missing** | `Function call is missing a thought_signature` | **FIXED**: Automatic injection of `skip_thought_signature_validator` placeholder. |
| **Illegal cache tag** | `thinking.cache_control: Extra inputs are not permitted` | **Fixed**: Globally cull `cache_control` tag in history messages. |
| **Plan Mode error** | `EnterPlanMode tool call: InputValidationError: Extra inputs are not permitted` | **Fixed**: `streaming.rs` forced clearing of tool parameters to comply with the official no-argument protocol. |
| **Continuous User Messages** | `Consecutive user messages are not allowed` | **Fixed**: `merge_consecutive_messages` automatically merges adjacent messages of the same role. |

---

## 3. Verify Claude Code Plan Mode and role alternation (Issue #813)

Aiming at the problem of protocol error caused by Plan Mode switching.

### A. Verify Plan Mode activation (UI status)
* **Command**: `Enter Plan Mode to investigate the directory structure of src-tauri. `
* **Expected results**:
    * A blue **`plan mode on`** label should immediately appear in the lower left corner of the terminal.
    * You should see `[Streaming] Tool Call: 'EnterPlanMode' Args: {}` in the log.

### B. Verify role replacement and self-healing (Consecutive Messages)
* **Command**: `Help me analyze the logic of proxy/mappers/claude/request.rs in Plan Mode, then exit Plan Mode and give a brief summary. `
* **Expected results**:
    * When the model switches modes (such as from Plan to Code), a 400 error will not be reported due to "two consecutive User messages".
    * The merge action of `merge_consecutive_messages` will be reflected in the log.

---

## 4. QuotaData field logical analysis

In the "Account Management" list in the settings page, the progress bar data below comes from `QuotaData`. The system checks account quotas before making requests and automatically rotates when the threshold is hit.

---

## Debugging suggestions
```bash
RUST_LOG=debug npm run tauri dev
```
Search for `[Claude-Request]` in the log and pay attention to the order of message roles.

---

## 5. Verify Thinking signature persistence and restart fault tolerance (Proxy Restart Test)

This test simulates the main logic of the proxy service: verifying whether historical messages carrying old signatures will cause a 400 error when the proxy is restarted (the memory signature cache is lost). This is the most efficient way to reproduce `Invalid signature`.

### Test process
1. **Generate Thinking (Step 1)**:
    * **Instruction**: `Analyze the code structure of proxy/mappers/claude/request.rs in detail, especially how it handles Thinking Block. Please show the thought process. `
    * *STATUS*: Claude CLI will receive a response containing Thinking and Signature.

2. **Simulating environment changes (Step 2)**:
    * **Action**: **Keep the current Claude CLI session from closing**.
    * **Action**: Completely restart Antigravity in another terminal (or `npm run tauri dev`).
    * *Principle*: Restarting will clear the "signature whitelist" in the agent's memory, which means that the signatures issued in Step 1 are now "unknown/untrusted" to the agent.

3. **Trigger history replay (Step 3)**:
    * **Instruction**: `Based on the above analysis, summarize the core logic of signature verification. `
    * *Principle*: The CLI will send the Thinking Block + Signature in Step 1 as a history record to the restarted agent.

### Expected results (verify fix)
* **If it fails**: Error `Invalid signature in thinking block` (because the agent cannot verify the signature, it is directly transmitted to Google and rejected by Google).
* **If passed (current version)**: The agent finds that the signature is not in the memory cache, **automatically triggers the downgrade logic** (strips the Thinking Block or sends it as plain text), and the conversation continues normally without error.

---

## 6. Verify Dynamic Thinking Stripping

This test verifies whether the system can automatically strip useless historical Thinking Blocks under high Context pressure or signature invalidation scenarios, thereby solving "Prompt is too long" and "Invalid signature" errors.

### Preconditions
* Enable Debug log: `RUST_LOG=debug npm run tauri dev`
* Make sure to use a model that supports Thinking (such as `claude-3-7-sonnet` or the mapped `gemini-2.0-flash-thinking-exp`)

### Verification Scenario A: Simulate High Load

This scenario verifies whether the system will automatically clean up old Thinking when the conversation history approaches the Token limit.

1. **Constructing a long dialogue**:
    * **Method 1 (auto-generated)**: Run `docs/generate_long_payload.sh` to generate a 2MB test file.
        ```bash
        chmod +x docs/generate_long_payload.sh
        ./docs/generate_long_payload.sh
        cat docs/long_context_payload.txt | pbcopy
        ```
        The clipboard contents are then pasted to Claude multiple times until significant delay is perceived or a contextual warning is received.

    * **Method 2 (Deep Thinking Induction - Continuous Pressure)**:
        The following prompts are designed to induce extremely long thought reasoning in the model. Can be sent in turn:

        > **Round 1 (History)**: "Please analyze the history of computing from the abacus to quantum computers. For every major milestone (at least 20), perform a deep 'thinking' block simulating the thought process of the inventors. Detailed thinking is required. Aim for maximum output tokens."

        > **Round 2 (Math/Logic)**: "Prove the Riemann Hypothesis. Just kidding. But please perform a deep, step-by-step derivation of the Navier-Stokes existence and smoothness problem's core challenges. Explore 10 different mathematical approaches, evaluating the pros and cons of each in extreme detail."

        > **Round 3 (System Architecture)**: "Design a distributed system capable of handling 100 billion requests per second. Detail the consensus execution flow (Paxos/Raft) for a single transaction across 5000 nodes. Simulate the network partition handling logic in your 'thinking' process for at least 50 failure scenarios."

        > **Round 4 (Literature)**: "Write a recursive story where the protagonist is a recursive function. The story must nest at least 20 levels deep, and for each level, you must 'think' about the symbolic meaning of that recursion depth before writing the narrative part."

2. **Observation log**:
    * Search for `[ContextManager]` in the terminal.
    * **Expected log**:
        ```
        [INFO] [ContextManager] Context pressure: 95.0% (1900000 / 2000000), Strategy: Aggressive => Purifying history
        [DEBUG] History purified successfully
        ```

3. **Verification results**:
    * The Request was successfully sent to Gemini, but no "Prompt is too long" was reported.
    * The HTTP response header contains `X-Context-Purified: true`.
    * Claude CLI is user-agnostic (the history is still displayed locally in the CLI, but the server side has been sanitized).

### Verification scenario B: Signature Immunity via Stripping

This scenario verifies that even if retry logic is not triggered, active stripping under high load can incidentally solve the signature problem.

1. **Generate signed Thinking**:
    * **Instructions**: `Think about Rust’s ownership mechanism and write 500 words. `

2. **Restart Proxy and inject fake payload (optional)**:
    * Restart the agent (clear the signature cache).
    * Continue the conversation. At this point the Thinking with the old signature will be sent to the agent.

3. **Expected results**:
    * If Stripping is triggered due to high context pressure, or RetriedWithoutThinking is triggered due to a signature error, the system will strip off the Thinking Block.
    * **Key Point**: Once the Thinking Block is stripped off, `thought_signature` will also disappear.
    * Gemini receives plain text history and will never report Invalid Signature.

---

## 7. OpenCode (Claude Code CLI) multi-protocol access test

**Antigravity has fully supported OpenCode's multi-protocol access**, completely solving compatibility issues such as `AI_TypeValidationError`. You can choose any of the following ways to access according to your needs.

### Endpoint configuration table

| protocol type | Base URL (Antigravity) | Corresponding OpenCode Provider | Remark |
| :--- | :--- | :--- | :--- |
| **Anthropic (native)** | `http://localhost:8045/v1` | `anthropic` | **recommend**. Supports Thinking, tool calling, and Artifacts preview. |
| **OpenAI (Standard)** | `http://localhost:8045/v1` | `openai` | Supports common OpenAI client logic. |
| **OA-Compatible** | `http://localhost:8045/v1` | `openai-compatible` | Suitable for scenarios where non-standard model names are forced to be specified. |
| **Google Gemini** | `http://localhost:8045/v1` | `gemini` | Directly uses the Gemini protocol and supports Google's native SDK features. |

### A. Method 1: Anthropic native protocol (recommended)

This method provides the best Claude native experience and supports Thinking signature protection and Beta features.

1. **Configuration**:
    ```bash
    # Set Base URL (note: OpenCode’s anthropic provider sometimes requires a full path)
    export ANTHROPIC_BASE_URL="http://localhost:8045/v1"
    # Set API Key (Antigravity’s key)
    export ANTHROPIC_API_KEY="sk-antigravity-key"
    ```

2. **Test Instructions**:
    ```bash
    claude "Please use Thinking to analyze the Cargo.toml dependency structure in the current directory."
    ```

3. **Verification Point**:
    * **Thinking**: Can you see the blue thinking block output?
    * **Signature**: Check the Antigravity log, it should show `Cached signature to session ... [FIFO: true]`.
    * **No errors**: No `Invalid signature` error was reported throughout the process.

### B. Method 2: OpenAI protocol (including Compatible)

Suitable for users who are accustomed to using the OpenAI ecosystem or need specific model mapping.

1. **Configuration**:
    ```bash
    # Set Base URL
    export OPENAI_BASE_URL="http://localhost:8045/v1"
    export OPENAI_API_KEY="sk-antigravity-key"
    ```

2. **Start OpenCode**:
    ```bash
    claude --provider openai --model gemini-2.0-flash
    # Or use compatible mode
    claude --provider openai-compatible --model gemini-2.0-flash
    ```

3. **Verification Point**:
    * **JSON Error**: When trying to intentionally disconnect or use an invalid Key, OpenCode should display a friendly JSON error message (such as `{"error": {"message": "..."}}`) instead of crashing.
    * **Non-Streaming Compatible**: Some OpenCode tool calls may use non-streaming requests, verify that they can parse the JSON response properly.

### C. Method 3: Google Gemini native protocol

Antigravity v4.1.4 added support.

1. **Configuration**:
    ```bash
    export GEMINI_API_KEY="sk-antigravity-key"
    # If OpenCode supports GEMINI_BASE_URL (usually requires anti-generation tools such as cloudflared or modified config):
    export GEMINI_BASE_URL="http://localhost:8045/v1"
    ```

2. **Verification Point**:
    * **Adapter Detection**: Antigravity logs should show `[Gemini] Client Adapter detected`.
    * **Let It Crash**: When a 403/404 error is encountered, the response should be returned immediately rather than letting OpenCode hang waiting for a retry.

### D. Troubleshooting common problems

* **Q: Error `AI_TypeValidationError` reported? **
    * **A**: Please make sure to upgrade Antigravity to v4.1.2+. Older versions returned incorrect formats (plain text) that failed OpenCode's Zod validation.

* **Q: Is the Thinking block displayed as `[Redacted]` or disappears directly? **
    * **A**: This is normal. To protect Google's signatures from being compromised, Antigravity may proactively strip blocks of thought in certain situations, such as high contextual pressure or when signature verification fails. As long as the conversation continues, the "Dynamic Stripping" mechanism is working.

---

## 8. Multiple rounds of continuous conversation stress test (Continuous Conversation Stress Test)

This test is designed to verify **Signed Session Stability** (signed session stability) under high-frequency, multi-round interactions. Please perform the following steps **continuously** within an OpenCode session without restarting or clearing the context.

### Scenario: Rust project reconstruction in practice

#### Round 1: In-depth Code Review (Initial Analysis)
* **instruction**:
    ```bash
    claude "Please review the src-tauri/src/proxy/handlers/claude.rs file in detail. Pay attention to the handle_messages function and analyze how it handles Beta Headers injection. Please use a thought chain to list your analysis steps."
    ```
* **Verification Point**:
    * You must see the logic analysis of injecting Header through `ClientAdapter`.
    * The response contains the complete Thinking Block.

#### Round 2: Refactoring Proposal
* **instruction**:
    ```bash
    claude "Based on your analysis, if I want to add an adapter named 'CherryStudio', which files should be modified? Please give a specific implementation plan and do not modify the files directly."
    ```
* **Verification Point**:
    * Claude can accurately reference the context of round 1 (proof that the Session ID is passed properly).
    * The Thinking signature has not been lost (if an error message of `Invalid signature` is reported, it means that the signature cache is invalid).

#### Round 3: High-frequency concurrent testing (Concurrent Simulation)
* **Background**: In this round, we simulate a rapid succession of queries to test the robustness of the FIFO signature queue.
* **Command (please execute 3 times in quick succession)**:
    ```bash
    # Quickly enter the following short instructions to simulate the user's urgent questioning
    claude "In the plan just now, does StreamingState need to be changed?"
    claude "What about the ClientAdapter trait?"
    claude "Does Cargo.toml need to add dependencies?"
    ```
* **Verification Point**:
    * **Out-of-order tolerance**: Even though responses may arrive in an order inconsistent with the request, the client should not crash.
    * **Queue Depth**: The Antigravity log should show that the Signature Cache is updated normally, and there is no pre-order signature failure caused by overwriting.

#### Round 4: Long Text Generation (Output Token Limit)
* **instruction**:
    ```bash
    claude "Please write a detailed developer documentation (Markdown format) for the ClientAdapter trait, including detailed comments on all methods and best practice example code for three different scenarios. The word count is required to be more than 2000 words."
    ```
* **Verification Point**:
    * Verify that SSE flow is stable under large output volumes.
    * Observe in the log whether active purification of `ContextManager` is triggered and whether the signature is safely stripped.

---

## 6. Intelligent context compression level test (Compression Level Test)

For the `Intelligent Contextual Compression Levels` (Disabled / Low / Medium / High) introduced in the new version, you can use the following steps to verify the purification and noise reduction performance of each level.

### A. Basic test data preparation

Starting a new session in a client (such as Cline or Cherry Studio), we will prepare a text that is a mixture of redundant polite language and typical repetitive build logs as the first round of requests:

```text
Hello, could you please help me build this package? Actually, basically, I think probably it is failed.

Progress: 10%
Progress: 20%
Progress: 30%
Progress: 40%
Progress: 50%
Error: compilation failed at src/main.rs:25
```

After the model replies, it will then conduct a second round of questioning (for example: `How to fix this error?`). At this time, the first round of messages will become "historical messages", thus triggering the corresponding static cleaning rules.

---

### B. Verify Low level (log noise reduction only)

1. **Settings**: In the Settings panel, select `Smart Context Compression Level` to **`Low - Log Noise Reduction`**.
2. **Test**: Execute the above two rounds of dialogue.
3. **Log and effect verification**:
    * In the console of `npm run tauri:debug`, observe the payload when the second round of requests is sent:
        * **RTK log denoising takes effect**: `Progress: 10% ... 50%` in the first round of messages will be automatically purified and collapsed into placeholders similar to `[Collapsed 3 similar lines]`.
        * **Safe recovery of error reports**: The `Error: compilation failed...` error message at the bottom is completely retained.
        * **Spoken language remains intact**: The `Hello, could you please...` polite words at the beginning of the first round of messages are still intact in the payload and have not been streamlined.

---

### C. Verify Medium level (log noise reduction + spoken language purification)

1. **Settings**: In the Settings panel, select `Smart Contextual Compression Level` to **`Medium (Log + Spoken)`**.
2. **Test**: Reopen the session and perform the above two rounds of dialogue.
3. **Log and effect verification**:
    * Observe the payload when the second round of request is sent in the console:
        * **RTK log denoising takes effect**: Duplicate progress lines continue to be successfully collapsed.
        * **Caveman spoken language purification takes effect**: The opening text in the historical message has been greatly purified. Modifying modal particles such as `Hello, could you please...` and `Actually, basically, I think probably...` that have no substantive meaning have been completely removed, and converted into a minimalist primitive style.
        * **Code Isolation Security**: All code snippets and stack paths (such as `src/main.rs:25`) are not broken.

---

### D. Verify High level (log + spoken + dynamic anti-riot)

1. **Settings**: In the Settings panel, select `Smart Context Compression Level` to **`High (High - Dynamic Vandal Protection)`**.
2. **Test**: Send a long piece of project code or contain tens of thousands of Tokens to increase context usage.
3. **Log and effect verification**:
    * **Dynamic three-level pressure adaptation**: You will be able to see in the console that `ContextManager` estimates the pressure ratio of the current context in real time (such as `Context pressure: 45.2%`).
    * **Grade trigger log**:
        * L1 threshold exceeded: trigger `[Layer-1] Tool trimming` trimming legacy toolkit.
        * L2 threshold exceeded: Trigger `[Layer-2] Thinking compression` to compress historical thinking while replaying and protecting signatures.
        * Exceeding the L3 threshold: triggering the `[Layer-3] Fork+Summary` ultimate reset, the console will display the session restart and summary generation, and the injected `cache_control: {"type": "ephemeral"}` tag will be displayed in the payload of the first summary (in subsequent chats, you will observe a geometric surge in the Prompt Cache hit rate on the client, and extremely fast response).
