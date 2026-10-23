# Professional version model 1.5/2.5 Pro automatic alignment and shunt test (v4.2.4)

## Test target

Verify the correctness, stability, and cost optimization of the three-layer progressive context compression function.

## Preparation

1. **Start the application**:
   ```bash
   cd /Users/lbjlaq/Desktop/xin
   npm run tauri dev
   ```

2. **Enable debug logs**:
   ```bash
   export RUST_LOG=debug
   ```

3. **Prepare test account**:
   - At least 1 Google account (for Gemini API)
   - Make sure the account has enough quota

## Test scenario

### Scenario 1: Layer 1 tool message clipping (60% pressure)

**Goal**: Verify tool message intelligent cropping function

**Steps**:
1. Use Claude Code CLI or Cherry Studio
2. Initiate a task that requires multiple tool calls (such as code search, file reading)
3. Continue the conversation until 60% contextual pressure is triggered

**Expected results**:
- `[Layer-1] Tool trimming triggered` appears in the log
- Keep the last 5 rounds of tool interactions
- Delete older tool messages
- **No 400 errors**
- **Response speed is normal**

**Verification command**:
```bash
# View log
tail -f ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep "Layer-1"
```

---

### Scenario 2: Layer 2 Thinking Compression (75% pressure)

**Goal**: Verify Thinking content compression + signature preservation

**Steps**:
1. Using the Claude 4.5 Opus/Sonnet Thinking model
2. Initiate complex reasoning tasks (such as code reconstruction, algorithm design)
3. Continue the conversation until 75% contextual pressure is triggered

**Expected results**:
- `[Layer-2] Thinking compression triggered` appears in the log
- Thinking block text is compressed to "..."
- **`signature` field remains intact**
- The last 4 messages are not compressed
- **NO 400 SIGNATURE ERROR**

**Verification command**:
```bash
# Check the signature retention status
tail -f ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep -E "(Layer-2|signature)"
```

---

### Scenario 3: Layer 3 Fork Session + XML Digest (90% Stress)

**Goal**: Validate XML digest generation and session fork

**Steps**:
1. Use any model for very long conversations
2. Continue the conversation until 90% contextual pressure is triggered

**Expected results**:
- `[Layer-3] Critical context pressure` appears in the log
- Call `gemini-2.5-flash-lite` to generate XML digest
- Create a new message sequence: `[User: XML summary] + [Assistant: Confirmation] + [User's latest message]`
- **Compression rate 86-97%**
- **No Prompt Cache corruption**
- **Signature chain complete**

**Verification command**:
```bash
# View Layer 3 triggering and summary generation
tail -f ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep -E "(Layer-3|XML summary|Fork)"
```

---

### Scenario 4: Progressive Trigger Test

**Goal**: Verify the progressive triggering mechanism of three-layer compression

**Steps**:
1. Start with an empty conversation
2. Continue the conversation and observe the triggering sequence of the compression levels.

**Expected results**:
- Trigger sequence: Layer 1 (60%) → Layer 2 (75%) → Layer 3 (90%)
- Re-estimate Token usage after each compression
- The triggers and effects of each layer are clearly recorded in the log

**Verification command**:
```bash
# View triggers at all levels
tail -f ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep -E "Layer-[123]"
```

---

### Scenario 5: Error handling test

**Goal**: Verify fault tolerance when Layer 3 fails

**Steps**:
1. Temporarily disable Gemini account or network
2. Trigger Layer 3 compression

**Expected results**:
- Return `BAD_REQUEST` error on Layer 3 failure
- Friendly error message: `Context too long and automatic compression failed`
- Prompt user to use `/compact` or switch models

**Verification command**:
```bash
# View error handling
tail -f ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep -E "(Layer-3.*failed|BAD_REQUEST)"
```

---

## Performance verification

### Token cost savings

**Test method**:
1. Record the Token usage before compression (extracted from the log)
2. Record the compressed Token usage
3. Calculate savings ratio

**Expected results**:
- Layer 1: 60-90% savings
- Layer 2: 70-95% savings
- Layer 3: 86-97% savings

### Response speed

**Test method**:
1. Use the `time` command to measure response time
2. Compare the response speed before and after compression

**Expected results**:
- Layer 1/2: No significant change in response speed
- Layer 3: The first digest generation may take 2-5 seconds, subsequent requests will be normal

---

## Compatibility testing

### Client Compatibility

Test the following clients:
- ✅ Claude Code CLI
- ✅ Cherry Studio
- ✅ Cursor
- ✅ Python OpenAI SDK
- ✅ Kilo Code

### Model Compatibility

Test the following models:
- ✅ Gemini 3 Flash
- ✅ Gemini 3 Pro High
- ✅ Claude 4.5 Sonnet
- ✅ Claude 4.5 Opus Thinking

---

## Regression testing

### Signature chain integrity

**Verification Point**:
- Signature is not lost after Layer 2 compression
- Signatures are restored correctly after Layer 3 Fork
- No 400 signature errors

### Tool call chain

**Verification Point**:
- Tool calls still work properly after compression
- Tool results are passed correctly
- No tool call interruption

---

## Log analysis

### Key log mode

```bash
# Layer 1 trigger
grep "Layer-1.*Tool trimming" antigravity.log

# Layer 2 trigger
grep "Layer-2.*Thinking compression" antigravity.log

# Layer 3 trigger
grep "Layer-3.*Fork successful" antigravity.log

# Token save statistics
grep "Compression result.*saved" antigravity.log
```

---

## Test report template

```markdown
## Test results

### Scenario 1: Layer 1 tool message clipping
- [ ] Triggered successfully
- [ ] keep the last 5 rounds
- [ ] No 400 errors
- [ ] Response speed is normal

### Scenario 2: Layer 2 Thinking Compression
- [ ] Triggered successfully
- [ ] Signature kept intact
- [ ] No signature error
- [ ] Compression rate meets standard

### Scenario 3: Layer 3 Fork session
- [ ] Triggered successfully
- [ ] XML digest generation
- [ ] Compression rate 86-97%
- [ ] No Cache corruption

### Scenario 4: Progressive triggering
- [ ] in correct order (1→2→3)
- [ ] Token revaluation
- [ ] clear log

### Scenario 5: Error handling
- [ ] Friendly prompts when failure occurs
- [ ] No crashes
- [ ] Recommendations are clear

### Performance verification
- Token savings: ____%
- Response speed: normal/slow (___ms)

### Compatibility
- Claude Code: ✅/❌
- Cherry Studio: ✅/❌
- Cursor: ✅/❌
- Python SDK: ✅/❌

### Regression testing
- Complete signature chain: ✅/❌
- Tool calling is normal: ✅/❌

## Problem record

(Record problems found during testing)

## Conclusion

(Overall comments and suggestions)
```

---

## Quick test script

```bash
#!/bin/bash
# Quickly test three-layer compression

echo "=== Test Layer 1 (Tool Cropping) ==="
# Use Claude Code to perform multiple file searches
claude "Please search all .rs files in the project and read the contents of 5 of them"

echo "=== Test Layer 2 (Thinking compression) ==="
#Use Thinking model for complex reasoning
claude --model claude-opus-4-5-thinking "Please analyze the performance bottleneck of this code in detail and propose an optimization plan"

echo "=== Test Layer 3 (Fork session) ==="
# Extra long dialogue triggers Fork
for i in {1..20}; do
  claude "Continue the previous topic, please provide more details (round $i)"
done

echo "=== View log ==="
tail -100 ~/Library/Application\ Support/com.antigravity.tools/logs/antigravity.log | grep -E "Layer-[123]"
```

---

## Notes

1. **Test environment**: Make sure to test in a clean environment to avoid interference from other factors
2. **Log level**: `RUST_LOG=debug` must be set to see detailed logs
3. **Account Quota**: Make sure the account has sufficient quota before testing
4. **Backup data**: Back up important data before testing
5. **Version Confirmation**: Confirm that you are running version v4.2.4

---

## Troubleshooting

### Problem 1: Layer 1 is not triggered
- Check if dialogue reaches 60% stress
- Check whether the Token estimate is accurate

### Problem 2: Layer 2 signature is lost
- Check the `compress_thinking_preserve_signature` function
- Verify signature extraction logic

### Issue 3: Layer 3 digest failed
- Check if Gemini account is available
- Verify `call_gemini_sync` function
- View upstream API errors

### Question 4: 400 error
- Check whether the signature chain is complete
- Verify tool call parameters
- View upstream API responses

---

## Contact information

If you have any questions, please raise an issue on GitHub:
https://github.com/lbjlaq/Antigravity-Manager/issues
