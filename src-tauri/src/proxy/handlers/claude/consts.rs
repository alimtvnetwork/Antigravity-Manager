use super::*;

const MAX_RETRY_ATTEMPTS: usize = 3;

// ===== Model Constants for Background Tasks =====
// These can be adjusted for performance/cost optimization or overridden by custom_mapping
pub(crate) const INTERNAL_BACKGROUND_TASK: &str = "internal-background-task"; // Unified virtual ID for all background tasks

// ===== Layer 3: XML Summary Prompt Template =====
// Borrowed from Practical-Guide-to-Context-Engineering + Claude Code official practice
// This prompt generates a structured 8-section XML summary for context compression
const CONTEXT_SUMMARY_PROMPT: &str = r#"You are a context compression specialist. Your task is to create a structured XML snapshot of the conversation history.

This snapshot will become the Agent's ONLY memory of the past. All key details, plans, errors, and user instructions MUST be preserved.

First, think through the entire history in a private <scratchpad>. Review the user's overall goal, the agent's actions, tool outputs, file modifications, and any unresolved issues. Identify every piece of information critical for future actions.

After reasoning, generate the final <state_snapshot> XML object. Information must be extremely dense. Omit any irrelevant conversational filler.

The structure MUST be as follows:

<state_snapshot>
  <overall_goal>
    <!-- Describe the user's high-level goal in one concise sentence -->
  </overall_goal>

  <technical_context>
    <!-- Tech stack: frameworks, languages, toolchain, dependency versions -->
  </technical_context>

  <file_system_state>
    <!-- List files that were created, read, modified, or deleted. Note their status -->
  </file_system_state>

  <code_changes>
    <!-- Key code snippets (preserve function signatures and important logic) -->
  </code_changes>

  <debugging_history>
    <!-- List all errors encountered, with stack traces, and how they were fixed -->
  </debugging_history>

  <current_plan>
    <!-- Step-by-step plan. Mark completed steps -->
  </current_plan>

  <user_preferences>
    <!-- User's work preferences for this project (test commands, code style, etc.) -->
  </user_preferences>

  <key_decisions>
    <!-- Critical architectural decisions and design choices -->
  </key_decisions>

  <latest_thinking_signature>
    <!-- [CRITICAL] Preserve the last valid thinking signature -->
    <!-- Format: base64-encoded signature string -->
    <!-- This MUST be copied exactly as-is, no modifications -->
  </latest_thinking_signature>
</state_snapshot>

**IMPORTANT**:
1. Code snippets must be complete, including function signatures and key logic
2. Error messages must be preserved verbatim, including line numbers and stacks
3. File paths must use absolute paths
4. The thinking signature must be copied exactly, no modifications
"#;

// ===== Jitter Configuration (REMOVED) =====
// Jitter was causing connection instability, reverted to fixed delays
// const JITTER_FACTOR: f64 = 0.2;

// ===== 统一退避策略模块 =====

// [REMOVED] apply_jitter function
// Jitter logic removed to restore stability (v3.3.16 fix)

// ===== 统一退避策略模块 =====
// 移除本地重复定义，使用 common 中的统一实现
use crate::proxy::handlers::common::{apply_retry_strategy, should_rotate_account, RetryStrategy};

// ===== 退避策略模块结束 =====
