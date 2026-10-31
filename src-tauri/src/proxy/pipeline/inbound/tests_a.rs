use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_preserves_process_commentary_alongside_tool_call() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "Checking the connection configuration of the gateway and backend." },
                {
                    "functionCall": {
                        "name": "inspect_case",
                        "args": { "case": "case_1" }
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIResponses,
            "gemini-2.5-pro",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        // When thinking is enabled, the first thinking block is completed, and subsequent normal progress text and tool calls are kept intact.
        assert!(parts[0]
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false));
        assert_eq!(
            parts[1]["text"],
            "Checking the connection configuration of the gateway and backend."
        );
        assert!(parts[2].get("functionCall").is_some());
    }

    #[test]
    fn test_preserves_multiple_plain_text_parts_intact() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "Phase One: Check Overview." },
                { "text": "The second stage: in-depth diagnosis." },
                {
                    "functionCall": {
                        "name": "run_check",
                        "args": {}
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIChat,
            "gemini-2.5-pro",
            false,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0]["text"], "Phase One: Check Overview.");
        assert_eq!(parts[1]["text"], "The second stage: in-depth diagnosis.");
        assert!(parts[2].get("functionCall").is_some());
    }

    #[test]
    fn test_heals_legacy_thinking_prefix_without_corrupting_prose() {
        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                { "text": "**Thinking**\n\nAnalyzed the case data and prepared to call the tool." },
                { "text": "Checking in progress." },
                {
                    "functionCall": {
                        "name": "inspect",
                        "args": {}
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIResponses,
            "gemini-2.5-pro",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert!(parts[0]
            .get("thought")
            .and_then(Value::as_bool)
            .unwrap_or(false));
        assert_eq!(
            parts[0]["text"],
            "Analyzed the case data and prepared to call the tool."
        );
        assert_eq!(parts[1]["text"], "Checking in progress.");
        assert!(parts[2].get("functionCall").is_some());
    }

    #[test]
    fn test_claude_model_packages_signature_for_google_vertex() {
        let raw_claude_sig = "Eu8CCpIBCBIQAhgCKkAtARbmpPNxYxc/Yz+mpbWJOqMo9c9RF4ESxACD0e/d6SZTpwmbrf9gPP/XMGZ9+kBkTMBfdK7ICuVonHJuu1AcMg9jbGF1ZGUtb3B1cy00LTY4AEIIdGhpbmtpbmdaDDg4NDM1NDkxOTA1MnIQLmWKBlED8AVhXRwj5Lb+PogBAagBosG91QawAQISDFXhwclEQyYNjsDteRoMuuu1Y/dUbn7sPe5OIjBoGvxrSlIgU78kwl701wfF0Rj0BCaCpE6a+KRGaB5pO2vL3ox4+yqum5a8o7mQ8+kqiQFDBTvaDITieiRVrkA8EKBUrpV0rLDyEcL7iQnAMsdQOk31ZKDeBddhEVX+Tb7Qs9mNWXNW9cbrs82iea09O+j2IMs0ibbWXPHB20IlkhVc5q9MmKBYgeQSTzKz+8Tgf7EDd78lkYieVk6GHqQaNiWD1Sl+RO0mIDGwURmOON6Fyw6WkCh/WSF+ORgB";
        let expected_google_vertex_sig = "RXU4Q0NwSUJDQklRQWhnQ0trQXRBUmJtcFBOeFl4Yy9ZeittcGJXSk9xTW85YzlSRjRFU3hBQ0QwZS9kNlNaVHB3bWJyZjlnUFAvWE1HWjkra0JrVE1CZmRLN0lDdVZvbkhKdXUxQWNNZzlqYkdGMVpHVXRiM0IxY3kwMExUWTRBRUlJZEdocGJtdHBibWRhRERnNE5ETTFORGt4T1RBMU1uSVFMbVdLQmxFRDhBVmhYUndqNUxiK1BvZ0JBYWdCb3NHOTFRYXdBUUlTREZYaHdjbEVReVlOanNEdGVSb011dXUxWS9kVWJuN3NQZTVPSWpCb0d2eHJTbElnVTc4a3dsNzAxd2ZGMFJqMEJDYUNwRTZhK0tSR2FCNXBPMnZMM294NCt5cXVtNWE4bzdtUTgra3FpUUZEQlR2YURJVGllaVJWcmtBOEVLQlVycFYwckxEeUVjTDdpUW5BTXNkUU9rMzFaS0RlQmRkaEVWWCtUYjdRczltTldYTlc5Y2JyczgyaWVhMDlPK2oySU1zMGliYldYUEhCMjBJbGtoVmM1cTlNbUtCWWdlUVNUekt6KzhUZ2Y3RURkNzhsa1lpZVZrNkdIcVFhTmlXRDFTbCtSTzBtSURHd1VSbU9PTjZGeXc2V2tDaC9XU0YrT1JnQg==";

        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                {
                    "text": "Let me think about this.",
                    "thought": true,
                    "thoughtSignature": raw_claude_sig
                },
                {
                    "text": "Here is the response."
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::AnthropicClaude,
            "claude-opus-4-6-thinking",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["thought"], true);
        assert_eq!(parts[0]["thoughtSignature"], expected_google_vertex_sig);
        assert_eq!(parts[1]["text"], "Here is the response.");
    }

    #[test]
    fn test_claude_model_from_openai_protocol_packages_signature() {
        let raw_claude_sig = "Eu8CCpIBCBIQAhgCKkAtARbmpPNxYxc/Yz+mpbWJOqMo9c9RF4ESxACD0e/d6SZTpwmbrf9gPP/XMGZ9+kBkTMBfdK7ICuVonHJuu1AcMg9jbGF1ZGUtb3B1cy00LTY4AEIIdGhpbmtpbmdaDDg4NDM1NDkxOTA1MnIQLmWKBlED8AVhXRwj5Lb+PogBAagBosG91QawAQISDFXhwclEQyYNjsDteRoMuuu1Y/dUbn7sPe5OIjBoGvxrSlIgU78kwl701wfF0Rj0BCaCpE6a+KRGaB5pO2vL3ox4+yqum5a8o7mQ8+kqiQFDBTvaDITieiRVrkA8EKBUrpV0rLDyEcL7iQnAMsdQOk31ZKDeBddhEVX+Tb7Qs9mNWXNW9cbrs82iea09O+j2IMs0ibbWXPHB20IlkhVc5q9MmKBYgeQSTzKz+8Tgf7EDd78lkYieVk6GHqQaNiWD1Sl+RO0mIDGwURmOON6Fyw6WkCh/WSF+ORgB";
        let expected_google_vertex_sig = "RXU4Q0NwSUJDQklRQWhnQ0trQXRBUmJtcFBOeFl4Yy9ZeittcGJXSk9xTW85YzlSRjRFU3hBQ0QwZS9kNlNaVHB3bWJyZjlnUFAvWE1HWjkra0JrVE1CZmRLN0lDdVZvbkhKdXUxQWNNZzlqYkdGMVpHVXRiM0IxY3kwMExUWTRBRUlJZEdocGJtdHBibWRhRERnNE5ETTFORGt4T1RBMU1uSVFMbVdLQmxFRDhBVmhYUndqNUxiK1BvZ0JBYWdCb3NHOTFRYXdBUUlTREZYaHdjbEVReVlOanNEdGVSb011dXUxWS9kVWJuN3NQZTVPSWpCb0d2eHJTbElnVTc4a3dsNzAxd2ZGMFJqMEJDYUNwRTZhK0tSR2FCNXBPMnZMM294NCt5cXVtNWE4bzdtUTgra3FpUUZEQlR2YURJVGllaVJWcmtBOEVLQlVycFYwckxEeUVjTDdpUW5BTXNkUU9rMzFaS0RlQmRkaEVWWCtUYjdRczltTldYTlc5Y2JyczgyaWVhMDlPK2oySU1zMGliYldYUEhCMjBJbGtoVmM1cTlNbUtCWWdlUVNUekt6KzhUZ2Y3RURkNzhsa1lpZVZrNkdIcVFhTmlXRDFTbCtSTzBtSURHd1VSbU9PTjZGeXc2V2tDaC9XU0YrT1JnQg==";

        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                {
                    "text": "Thinking process",
                    "thought": true,
                    "thoughtSignature": raw_claude_sig
                },
                {
                    "text": "Answer from OpenAI gateway"
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIResponses,
            "claude-sonnet-4-6",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["thoughtSignature"], expected_google_vertex_sig);
        assert_eq!(parts[1]["text"], "Answer from OpenAI gateway");
    }

    #[test]
    fn test_gemini_native_signature_preserved_without_double_encoding() {
        let gemini_sig = "EudDCuRDAWkUfRO9pMXsHitwdfey4TDAgCv1WzzMfBXVamvaqJ01BJPawr58";

        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                {
                    "text": "Gemini thinking",
                    "thought": true,
                },
                {
                    "thoughtSignature": gemini_sig,
                    "functionCall": {
                        "name": "bash",
                        "args": { "command": "ls" }
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::GeminiNative,
            "gemini-3.8-flash-high",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 2);
        // Gemini native signatures are never re-encoded during tool call rounds and must remain intact on the functionCall component
        assert_eq!(parts[1]["thoughtSignature"], gemini_sig);
    }

    #[test]
    fn test_inbound_pipeline_intercepts_foreign_claude_signature_for_gemini() {
        let foreign_claude_sig = "3mgp11XmVXq9InniGA4VAKd7c97NqFw+dWZt79Uz/w9znho88gSM76jv2bZmir7wI86Ixpha7eWdGuznAot4PNbe3+V9bgMTIEyUarn4MLAiiFVb830ZlM+H5ukQwXdD2Zv8nUSmmZTYinpLPGha8TORZAfpU1FJEvwyECel5+W7kc9kpTWrd8DqRNBTOz5EDtvoatiZgKv5SqInhGXK74SJ+PRIC6fNXvYG082HR6TsVxvVYaerz8A40rloIVTxRNK43h3Ecs1boxY4PZqBT8Yhl2qn/iZ+4Xt7FNkI0DAuS9iK0HYKMC4yw0OqKx/LeU+WFZlyc6hGm1BkzLY6yG97MH7kmJ0OPlBWgWFaTeL/uXuGJX6QkKObXN+phoq+kkF2vdFt/mdJMbdgfmSCVQ9037hGBhOHm0zN50KLkp1SxuAY1oWc+lDcI4ufWoyn";

        let mut contents = vec![json!({
            "role": "model",
            "parts": [
                {
                    "text": "Cross-model thinking from Claude",
                    "thought": true,
                    "thoughtSignature": foreign_claude_sig
                },
                {
                    "thoughtSignature": foreign_claude_sig,
                    "functionCall": {
                        "name": "bash",
                        "args": { "command": "ls" }
                    }
                }
            ]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::AnthropicClaude,
            "gemini-3.7-flash-high",
            true,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(parts.len(), 2);
        assert_eq!(parts[0]["thought"], true);
        assert!(
            parts[0].get("thoughtSignature").is_none(),
            "Thinking block for Gemini should NOT carry foreign signature or sentinel in pure text"
        );
        assert_eq!(
            parts[1]["thoughtSignature"],
            crate::proxy::thinking_store::SENTINEL_SIGNATURE,
            "FunctionCall must fall back to sentinel signature in InboundThinkingPipeline"
        );
    }

    #[test]
    fn test_inbound_pipeline_intercepts_foreign_gemini_signature_for_claude() {
        // Simulate Gemini native signature
        let foreign_gemini_sig =
            "Ep4KCpsKAWkUfRMa5ZYMDdlPjxrQTLzVZ6MZeopI88888888888888888888888888888888";

        let mut contents = vec![
            json!({
                "role": "user",
                "parts": [{ "text": "hello" }]
            }),
            json!({
                "role": "model",
                "parts": [
                    {
                        "text": "The input is a Chinese greeting...",
                        "thought": true,
                        "thoughtSignature": foreign_gemini_sig
                    },
                    {
                        "text": "Hello! How can I help you today?"
                    }
                ]
            }),
            json!({
                "role": "user",
                "parts": [{ "text": "continue" }]
            }),
        ];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::OpenAIChat,
            "claude-opus-4-6-thinking",
            true,
            None,
            false,
        );

        let model_parts = contents[1]["parts"].as_array().expect("parts array");
        // Key verification: When sending to Claude, because the historical heterogeneous signature is not a legal Claude signature,
        // The thought block must never be sent to Claude with a Gemini signature, but is safely downgraded to normal body text!
        let has_thought_block = model_parts
            .iter()
            .any(|p| p.get("thought").and_then(|v| v.as_bool()) == Some(true));
        assert!(
            !has_thought_block,
            "Claude turn must NOT contain unvalidated thinking block with foreign Gemini signature"
        );
        let has_gemini_sig = model_parts
            .iter()
            .any(|p| p.get("thoughtSignature").is_some() || p.get("thought_signature").is_some());
        assert!(
            !has_gemini_sig,
            "Foreign Gemini signature must be completely eliminated from Claude turn"
        );
    }

    #[test]
    fn test_inbound_pipeline_lifts_multimodal_images_from_function_response() {
        let fake_b64 = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==";
        let mut contents = vec![json!({
            "role": "user",
            "parts": [{
                "functionResponse": {
                    "name": "take_screenshot",
                    "response": {
                        "output": format!("Screenshot result: ![view](data:image/png;base64,{}) done.", fake_b64)
                    }
                }
            }]
        })];

        InboundThinkingPipeline::process_contents(
            &mut contents,
            ProxyProtocol::GeminiNative,
            "gemini-2.5-flash",
            false,
            None,
            false,
        );

        let parts = contents[0]["parts"].as_array().expect("parts array");
        assert_eq!(
            parts.len(),
            2,
            "Should have functionResponse and lifted inlineData"
        );
        assert!(parts[0].get("functionResponse").is_some());
        assert!(parts[1].get("inlineData").is_some());

        assert_eq!(parts[1]["inlineData"]["mimeType"], "image/png");
        assert_eq!(parts[1]["inlineData"]["data"], fake_b64);

        let output_text = parts[0]["functionResponse"]["response"]["output"]
            .as_str()
            .unwrap();
        assert!(!output_text.contains(fake_b64));
        assert!(output_text.contains("[Image: forwarded to visual input (image/png)]"));
    }

    #[test]
    fn test_extract_client_thinking_switch_coverage() {
        // 1. Explicit shutdown (one vote veto)
        assert_eq!(
            extract_client_thinking_switch(Some("disabled"), None, None),
            ClientThinkingSwitch::Disabled
        );
        assert_eq!(
            extract_client_thinking_switch(Some("off"), None, None),
            ClientThinkingSwitch::Disabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, Some(0), None),
            ClientThinkingSwitch::Disabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, None, Some("none")),
            ClientThinkingSwitch::Disabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, None, Some("off")),
            ClientThinkingSwitch::Disabled
        );

        // 2. Explicitly enable
        assert_eq!(
            extract_client_thinking_switch(Some("enabled"), None, None),
            ClientThinkingSwitch::Enabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, Some(1024), None),
            ClientThinkingSwitch::Enabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, None, Some("low")),
            ClientThinkingSwitch::Enabled
        );
        assert_eq!(
            extract_client_thinking_switch(None, None, Some("high")),
            ClientThinkingSwitch::Enabled
        );

        // 3. Default (the switch is on by default)
        assert_eq!(
            extract_client_thinking_switch(None, None, None),
            ClientThinkingSwitch::Default
        );
        assert_eq!(
            extract_client_thinking_switch(Some("default"), None, None),
            ClientThinkingSwitch::Default
        );
        assert_eq!(
            extract_client_thinking_switch(None, None, Some("default")),
            ClientThinkingSwitch::Default
        );
    }
}
