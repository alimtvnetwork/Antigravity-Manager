use super::*;

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proxy::mappers::claude::models::{Message, SystemPrompt, Tool};

    #[test]
    fn test_sanitize_user_text_for_fingerprint() {
        let text1 =
            "hello\n\n<system-reminder>\nCurrent date: 2026-09-08 (Tue)\n</system-reminder>";
        let text2 =
            "hello\n\n<system-reminder>\nCurrent date: 2026-09-09 (Wed)\n</system-reminder>";
        let text3 = "hello [System: Tool execution completed successfully]";

        assert_eq!(sanitize_user_text_for_fingerprint(text1), "hello");
        assert_eq!(sanitize_user_text_for_fingerprint(text2), "hello");
        assert_eq!(sanitize_user_text_for_fingerprint(text3), "hello");
    }

    #[test]
    fn test_same_user_text_different_system_prompt_isolated() {
        let req_project_a = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: Some(SystemPrompt::String(
                "Project A Workspace: /src/backend".to_string(),
            )),
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };

        let req_project_b = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: Some(SystemPrompt::String(
                "Project B Workspace: /src/frontend".to_string(),
            )),
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };

        let sid_a = SessionManager::extract_session_id(&req_project_a);
        let sid_b = SessionManager::extract_session_id(&req_project_b);

        // Even though both messages are "hello", different workspace/system instructions yield completely isolated sessions!
        assert_ne!(sid_a, sid_b);
    }

    #[test]
    fn test_same_user_text_different_tools_isolated() {
        let req_with_tools = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: None,
            tools: Some(vec![Tool {
                name: Some("bash".to_string()),
                description: None,
                input_schema: None,
                type_: None,
            }]),
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };

        let req_pure_chat = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: None,
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };

        let sid_tools = SessionManager::extract_session_id(&req_with_tools);
        let sid_chat = SessionManager::extract_session_id(&req_pure_chat);

        assert_ne!(sid_tools, sid_chat);
    }

    #[test]
    fn test_same_conversation_multi_turn_stability() {
        let req_turn1 = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: Some(SystemPrompt::String("Project A".to_string())),
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };

        let mut req_turn2 = req_turn1.clone();
        req_turn2.messages.push(Message {
            role: "assistant".to_string(),
            content: MessageContent::String("Hello! How can I help you?".to_string()),
        });
        req_turn2.messages.push(Message {
            role: "user".to_string(),
            content: MessageContent::String("Write a fibonacci sequence".to_string()),
        });

        let sid1 = SessionManager::extract_session_id(&req_turn1);
        let sid2 = SessionManager::extract_session_id(&req_turn2);

        // Within the same conversation, multi-turn session ID remains 100% stable!
        assert_eq!(sid1, sid2);
    }

    #[test]
    fn test_utf8_char_boundary_at_512_bytes_never_panics() {
        // Construct a system prompt where byte index 512 lands exactly inside a 3-byte UTF-8 character (bytes 511..514)
        let prefix = "a".repeat(511);
        let malicious_sys = format!("{}\u{5355}\u{6e05}\u{5355}\u{6e05}\u{5355}", prefix);
        assert!(
            !malicious_sys.is_char_boundary(512),
            "Byte 512 must be inside multibyte char to test the regression"
        );

        // 1. Claude Request
        let claude_req = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: Some(SystemPrompt::String(malicious_sys.clone())),
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };
        let sid_claude = SessionManager::extract_session_id(&claude_req);
        assert!(sid_claude.starts_with("sid-"));

        // 2. OpenAI Request
        let openai_req: OpenAIRequest = serde_json::from_value(serde_json::json!({
            "model": "gpt-4o",
            "messages": [
                { "role": "system", "content": malicious_sys },
                { "role": "user", "content": "hello" }
            ]
        }))
        .unwrap();
        let sid_openai = SessionManager::extract_openai_session_id(&openai_req);
        assert!(sid_openai.starts_with("sid-"));

        // 3. Gemini Native Request
        let gemini_req = serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{ "text": "hello" }]
            }],
            "system_instruction": {
                "parts": [{ "text": malicious_sys }]
            }
        });
        let sid_gemini = SessionManager::extract_gemini_session_id(&gemini_req, "gemini-2.5-pro");
        assert!(sid_gemini.starts_with("sid-"));

        // 4. Real User Prompt reported in issue
        let user_prompt = "You are a remote server operations expert....".repeat(20);
        let claude_user_req = ClaudeRequest {
            model: "claude-3-7-sonnet".to_string(),
            messages: vec![Message {
                role: "user".to_string(),
                content: MessageContent::String("hello".to_string()),
            }],
            system: Some(SystemPrompt::String(user_prompt)),
            tools: None,
            stream: false,
            max_tokens: None,
            temperature: None,
            top_p: None,
            top_k: None,
            thinking: None,
            metadata: None,
            output_config: None,
            size: None,
            quality: None,
        };
        let sid_real = SessionManager::extract_session_id(&claude_user_req);
        assert!(sid_real.starts_with("sid-"));
    }
}
