use super::*;
use crate::proxy::handlers::claude::consts::INTERNAL_BACKGROUND_TASK;

// ===== 后台任务检测辅助函数 =====

/// 后台任务类型
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq)]
enum BackgroundTaskType {
    TitleGeneration,    // 标题生成
    SimpleSummary,      // 简单摘要
    ContextCompression, // 上下文压缩
    PromptSuggestion,   // 提示建议
    SystemMessage,      // 系统消息
    EnvironmentProbe,   // 环境探测
}

#[allow(dead_code)]
const TITLE_KEYWORDS: &[&str] = &[
    "write a 5-10 word title",
    "Please write a 5-10 word title",
    "Respond with the title",
    "Generate a title for",
    "Create a brief title",
    "title for the conversation",
    "conversation title",
    "生成标题",
    "为对话起个标题",
];

#[allow(dead_code)]
const SUMMARY_KEYWORDS: &[&str] = &[
    "Summarize this coding conversation",
    "Summarize the conversation",
    "Concise summary",
    "in under 50 characters",
    "compress the context",
    "Provide a concise summary",
    "condense the previous messages",
    "shorten the conversation history",
    "extract key points from",
];

#[allow(dead_code)]
const SUGGESTION_KEYWORDS: &[&str] = &[
    "prompt suggestion generator",
    "suggest next prompts",
    "what should I ask next",
    "generate follow-up questions",
    "recommend next steps",
    "possible next actions",
];

#[allow(dead_code)]
const SYSTEM_KEYWORDS: &[&str] = &[
    "Warmup",
    "<system-reminder>",
    // Removed: "Caveat: The messages below were generated" - this is a normal Claude Desktop system prompt
    "This is a system message",
];

#[allow(dead_code)]
const PROBE_KEYWORDS: &[&str] = &[
    "check current directory",
    "list available tools",
    "verify environment",
    "test connection",
];

#[allow(dead_code)]
fn detect_background_task_type(request: &ClaudeRequest) -> Option<BackgroundTaskType> {
    let last_user_msg = extract_last_user_message_for_detection(request)?;
    let preview = last_user_msg.chars().take(500).collect::<String>();

    // 长度过滤：后台任务通常不超过 800 字符
    if last_user_msg.len() > 800 {
        return None;
    }

    // 按优先级匹配
    if matches_keywords(&preview, SYSTEM_KEYWORDS) {
        return Some(BackgroundTaskType::SystemMessage);
    }

    if matches_keywords(&preview, TITLE_KEYWORDS) {
        return Some(BackgroundTaskType::TitleGeneration);
    }

    if matches_keywords(&preview, SUMMARY_KEYWORDS) {
        if preview.contains("in under 50 characters") {
            return Some(BackgroundTaskType::SimpleSummary);
        }
        return Some(BackgroundTaskType::ContextCompression);
    }

    if matches_keywords(&preview, SUGGESTION_KEYWORDS) {
        return Some(BackgroundTaskType::PromptSuggestion);
    }

    if matches_keywords(&preview, PROBE_KEYWORDS) {
        return Some(BackgroundTaskType::EnvironmentProbe);
    }

    None
}

#[allow(dead_code)]
fn matches_keywords(text: &str, keywords: &[&str]) -> bool {
    keywords.iter().any(|kw| text.contains(kw))
}

#[allow(dead_code)]
fn extract_last_user_message_for_detection(request: &ClaudeRequest) -> Option<String> {
    request
        .messages
        .iter()
        .rev()
        .filter(|m| m.role == "user")
        .find_map(|m| {
            let content = match &m.content {
                crate::proxy::mappers::claude::models::MessageContent::String(s) => s.to_string(),
                crate::proxy::mappers::claude::models::MessageContent::Array(arr) => arr
                    .iter()
                    .filter_map(|block| match block {
                        crate::proxy::mappers::claude::models::ContentBlock::Text { text } => {
                            Some(text.as_str())
                        }
                        _ => None,
                    })
                    .collect::<Vec<_>>()
                    .join(" "),
            };

            if content.trim().is_empty()
                || content.starts_with("Warmup")
                || content.contains("<system-reminder>")
            {
                None
            } else {
                Some(content)
            }
        })
}

#[allow(dead_code)]
fn select_background_model(task_type: BackgroundTaskType) -> &'static str {
    match task_type {
        BackgroundTaskType::TitleGeneration => INTERNAL_BACKGROUND_TASK,
        BackgroundTaskType::SimpleSummary => INTERNAL_BACKGROUND_TASK,
        BackgroundTaskType::SystemMessage => INTERNAL_BACKGROUND_TASK,
        BackgroundTaskType::PromptSuggestion => INTERNAL_BACKGROUND_TASK,
        BackgroundTaskType::EnvironmentProbe => INTERNAL_BACKGROUND_TASK,
        BackgroundTaskType::ContextCompression => INTERNAL_BACKGROUND_TASK,
    }
}
