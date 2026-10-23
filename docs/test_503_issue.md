#503 Error (Service Unavailable) Fix Verification Guide

This guide provides test verification examples for recently reported 503 errors (Issue #1794 and backend capacity limitations).

## 1. Verify automatic rollback after failure to obtain Project ID (Issue #1794)

**Scene description**:
Some accounts (especially Free accounts or restricted accounts) will report an error `The account is not eligible to obtain the official cloudaicompanionProject` when calling the official interface to obtain the project ID. Before the repair, the system will directly skip the account and eventually return 503; after the repair, the system will automatically use the universal Project ID (`bamboo-precept-lgxtn`).

### A. Use `curl` for basic connectivity testing
Please use the API Key corresponding to an account that previously reported error 503 (or directly through a proxy):

```bash
curl http://localhost:8045/v1/chat/completions \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer sk-antigravity-key" \
  -d '{
    "model": "gemini-2.0-flash",
    "messages": [
      {"role": "user", "content": "Hello, please confirm your work status."}
    ],
    "stream": false
  }'
```

### B. Observe server logs (npm run tauri dev)
**Expected Phenomenon**:
When the system detects a permission problem, the following **Warn** information will appear in the log, but the request ** should not report an error 503**, but continue to execute:

```text
WARN Failed to fetch project_id for user@example.com, using fallback: Account is not eligible for official cloudaicompanionProject
DEBUG [TokenManager] Using project_id: bamboo-precept-lgxtn for request
```

---

## 2. Verify Quota Protection’s prevention of 503

**Scene description**:
When the account quota is exhausted or the backend returns 503 due to high load, the system should correctly identify and try to rotate the account instead of directly transparently transmitting 503 to the client.

### Test Command (Claude CLI)
```bash
claude "Where is the bug in this code? [Attached is a long code]"
```

**Verification Point**:
- If the current account returns 503, the log should display `[RetryStrategy] Status 503 detected, rotating account...`.
- The system should automatically try the next available account until a successful response is obtained or the number of retries is exhausted.

---

## 3. Distinguish between "code bugs" and "backend capacity limitations" (Opus 4.6)

**Scene description**:
Since the `claude-opus-4-6-thinking` model is currently in an experimental stage, the Google backend often returns `No capacity available` (503).

### Test instructions
```bash
claude --model claude-opus-4-6-thinking "Perform an in-depth inference task comparing Rust and C++'s asynchronous memory models."
```

**Expected results analysis**:
1. **If 503 is returned and the message contains "No capacity available"**:
   - This is a **Google backend capacity limit**, not a bug in this software.
   - The agent will automatically try other accounts through the retry policy, but if all accounts encounter capacity limits, this 503 will eventually be passed through.
   - **Recommendation**: Switch to `gemini-2.0-flash-thinking-exp` or `claude-3-7-sonnet` for testing during this load peak period.

2. **If the return is successful**:
   - Indicates that the current backend capacity is sufficient.

---

## Debugging assistance skills

If you want to force simulation of Project ID failure scenarios for code-level verification, you can temporarily modify the simulation logic in `src-tauri/src/proxy/token_manager.rs`. But in most cases, you can confirm that the fix has taken effect by observing whether the words `using fallback: ...` appear in the log.
