use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

/// Unified usage and caching calculation engine (CanonicalUsage)
/// Authoritatively converges Google Gemini native usageMetadata and provides standard diffusion methods across protocols.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct CanonicalUsage {
    /// Total prompt context input (Total Prompt Context = uncached + cached)
    pub total_input_tokens: u32,
    /// Uncached incremental input (Uncached Input = total_input_tokens - cached_tokens)
    pub uncached_input_tokens: u32,
    /// Cached tokens
    pub cached_tokens: u32,
    /// Total model output tokens (including reasoning chain)
    pub output_tokens: u32,
    /// Reasoning token consumption
    pub reasoning_tokens: u32,
    /// Total consumed tokens = total_input_tokens + output_tokens
    pub total_tokens: u32,
}

impl CanonicalUsage {
    /// Construct standard usage object from discrete fields
    pub fn new(
        total_input_tokens: u32,
        cached_tokens: u32,
        output_tokens: u32,
        reasoning_tokens: u32,
    ) -> Self {
        let uncached_input_tokens = total_input_tokens.saturating_sub(cached_tokens);
        let total_tokens = total_input_tokens.saturating_add(output_tokens);
        Self {
            total_input_tokens,
            cached_tokens,
            uncached_input_tokens,
            output_tokens,
            reasoning_tokens,
            total_tokens,
        }
    }

    /// Authoritatively extract and converge from Google upstream native usageMetadata or any JSON containing it
    pub fn from_gemini(raw: &Value) -> Self {
        let usage = raw.get("usageMetadata").unwrap_or(raw);

        let total_input = usage
            .get("promptTokenCount")
            .or_else(|| usage.get("prompt_tokens"))
            .or_else(|| usage.get("input_tokens"))
            .or_else(|| usage.get("total_input_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let cached = usage
            .get("cache_read_input_tokens")
            .or_else(|| usage.get("cachedContentTokenCount"))
            .or_else(|| usage.get("cached_content_token_count"))
            .or_else(|| usage.get("total_cached_tokens"))
            .or_else(|| usage.get("cached_tokens"))
            .or_else(|| {
                usage
                    .get("prompt_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
            })
            .or_else(|| {
                usage
                    .get("input_tokens_details")
                    .and_then(|d| d.get("cached_tokens"))
            })
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        // Extract reasoning tokens (supports candidatesTokensDetails / total_thought_tokens / completion_tokens_details)
        let mut reasoning = 0u32;
        if let Some(details) = usage
            .get("candidatesTokensDetails")
            .and_then(|d| d.as_array())
        {
            for item in details {
                let modality = item.get("modality").and_then(|m| m.as_str()).unwrap_or("");
                if modality.eq_ignore_ascii_case("THINKING")
                    || modality.eq_ignore_ascii_case("REASONING")
                {
                    reasoning = reasoning.saturating_add(
                        item.get("tokenCount").and_then(|v| v.as_u64()).unwrap_or(0) as u32,
                    );
                }
            }
        }
        if reasoning == 0 {
            if let Some(r) = usage
                .get("total_thought_tokens")
                .or_else(|| usage.get("totalThoughtTokens"))
                .or_else(|| usage.get("thoughtsTokenCount"))
                .or_else(|| {
                    usage
                        .get("completion_tokens_details")
                        .and_then(|d| d.get("reasoning_tokens"))
                })
                .or_else(|| {
                    usage
                        .get("output_tokens_details")
                        .and_then(|d| d.get("reasoning_tokens"))
                })
                .and_then(|v| v.as_u64())
            {
                reasoning = r as u32;
            }
        }

        let raw_output = usage
            .get("total_output_tokens")
            .or_else(|| usage.get("candidatesTokenCount"))
            .or_else(|| usage.get("candidates_token_count"))
            .or_else(|| usage.get("completion_tokens"))
            .or_else(|| usage.get("output_tokens"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let has_new_format = usage.get("total_output_tokens").is_some();
        let tool_use = usage
            .get("total_tool_use_tokens")
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as u32;

        let output = if has_new_format {
            raw_output
                .saturating_add(reasoning)
                .saturating_add(tool_use)
        } else {
            raw_output
        };

        // Calculate total and incremental
        // If input data is in Anthropic format (where raw_input stores uncached), sum them to restore total
        let (canonical_total_input, uncached_input) = if let Some(cr) = usage
            .get("cache_read_input_tokens")
            .and_then(|v| v.as_u64())
        {
            let cc = usage
                .get("cache_creation_input_tokens")
                .and_then(|v| v.as_u64())
                .unwrap_or(0) as u32;
            let total = total_input.saturating_add(cr as u32).saturating_add(cc);
            (total, total_input)
        } else if cached > 0 && total_input < cached {
            (total_input.saturating_add(cached), total_input)
        } else {
            (total_input, total_input.saturating_sub(cached))
        };

        let total = usage
            .get("total_tokens")
            .or_else(|| usage.get("totalTokenCount"))
            .and_then(|v| v.as_u64())
            .map(|v| v as u32)
            .unwrap_or_else(|| canonical_total_input.saturating_add(output));

        Self {
            total_input_tokens: canonical_total_input,
            uncached_input_tokens: uncached_input,
            cached_tokens: cached,
            output_tokens: output,
            reasoning_tokens: reasoning,
            total_tokens: total,
        }
    }

    /// Scaling algorithm (aligned with Claude Code context threshold): scale in tiers when exceeding 30k and scaling is enabled
    pub fn scale_claude_tokens(total_raw: u32, context_limit: u32) -> u32 {
        if total_raw <= 30_000 {
            return total_raw;
        }
        const TARGET_MAX: f64 = 195_000.0;
        let ratio = total_raw as f64 / context_limit.max(1) as f64;
        if ratio <= 0.5 {
            let display_ratio = ratio * 0.6;
            (display_ratio * TARGET_MAX) as u32
        } else if ratio <= 0.7 {
            let progress = (ratio - 0.5) / 0.2;
            let display_ratio = 0.3 + progress * 0.2;
            (display_ratio * TARGET_MAX) as u32
        } else if ratio <= 0.85 {
            let progress = (ratio - 0.7) / 0.15;
            let display_ratio = 0.5 + progress * 0.2;
            (display_ratio * TARGET_MAX) as u32
        } else {
            let progress = (ratio - 0.85) / 0.15;
            let display_ratio = 0.7 + progress * 0.27;
            (display_ratio.min(0.97) * TARGET_MAX) as u32
        }
    }

    /// Diffuse to Anthropic Claude format:
    /// Claude specification requires input_tokens to express only uncached increment; cached tokens are placed in cache_read_input_tokens
    pub fn to_claude_usage(&self, scaling_enabled: bool, context_limit: u32) -> Value {
        let (reported_input, reported_cache) = if scaling_enabled
            && self.total_input_tokens > 30_000
        {
            let scaled_total = Self::scale_claude_tokens(self.total_input_tokens, context_limit);
            if self.total_input_tokens > 0 {
                let cache_ratio = (self.cached_tokens as f64) / (self.total_input_tokens as f64);
                let sc_cache = (scaled_total as f64 * cache_ratio) as u32;
                (scaled_total.saturating_sub(sc_cache), Some(sc_cache))
            } else {
                (scaled_total, None)
            }
        } else {
            (
                self.uncached_input_tokens,
                if self.cached_tokens > 0 {
                    Some(self.cached_tokens)
                } else {
                    None
                },
            )
        };

        json!({
            "input_tokens": reported_input,
            "output_tokens": self.output_tokens,
            "cache_read_input_tokens": reported_cache,
            "cache_creation_input_tokens": 0,
        })
    }

    /// Diffuse to OpenAI Chat format:
    /// OpenAI specification requires prompt_tokens to be total input (including cache); cached tokens are placed in prompt_tokens_details.cached_tokens
    pub fn to_openai_chat_usage(&self) -> Value {
        let mut prompt_details = serde_json::Map::new();
        if self.cached_tokens > 0 {
            prompt_details.insert("cached_tokens".into(), json!(self.cached_tokens));
        }

        let mut completion_details = serde_json::Map::new();
        if self.reasoning_tokens > 0 {
            completion_details.insert("reasoning_tokens".into(), json!(self.reasoning_tokens));
        }

        let mut out = json!({
            "prompt_tokens": self.total_input_tokens,
            "completion_tokens": self.output_tokens,
            "total_tokens": self.total_tokens,
        });

        if let Some(obj) = out.as_object_mut() {
            if !prompt_details.is_empty() {
                obj.insert(
                    "prompt_tokens_details".into(),
                    Value::Object(prompt_details),
                );
            }
            if !completion_details.is_empty() {
                obj.insert(
                    "completion_tokens_details".into(),
                    Value::Object(completion_details),
                );
            }
        }
        out
    }

    /// Diffuse to OpenAI Responses format
    pub fn to_openai_responses_usage(&self) -> Value {
        let mut input_details = serde_json::Map::new();
        if self.cached_tokens > 0 {
            input_details.insert("cached_tokens".into(), json!(self.cached_tokens));
        }

        let mut output_details = serde_json::Map::new();
        if self.reasoning_tokens > 0 {
            output_details.insert("reasoning_tokens".into(), json!(self.reasoning_tokens));
        }

        let mut out = json!({
            "input_tokens": self.total_input_tokens,
            "output_tokens": self.output_tokens,
            "total_tokens": self.total_tokens,
        });

        if let Some(obj) = out.as_object_mut() {
            if !input_details.is_empty() {
                obj.insert("input_tokens_details".into(), Value::Object(input_details));
            }
            if !output_details.is_empty() {
                obj.insert(
                    "output_tokens_details".into(),
                    Value::Object(output_details),
                );
            }
        }
        out
    }

    /// Diffuse to Gemini Native format
    pub fn to_gemini_usage_metadata(&self) -> Value {
        let mut obj = json!({
            "promptTokenCount": self.total_input_tokens,
            "candidatesTokenCount": self.output_tokens,
            "totalTokenCount": self.total_tokens,
        });
        if let Some(map) = obj.as_object_mut() {
            if self.cached_tokens > 0 {
                map.insert("cachedContentTokenCount".into(), json!(self.cached_tokens));
            }
            if self.reasoning_tokens > 0 {
                map.insert(
                    "candidatesTokensDetails".into(),
                    json!([
                        {
                            "modality": "THINKING",
                            "tokenCount": self.reasoning_tokens
                        }
                    ]),
                );
            }
        }
        obj
    }

    /// Diffuse to ProxyMonitor brief mode and screenshot reference format
    pub fn to_brief_audit_usage(&self) -> Value {
        let cache_rate = if self.total_input_tokens > 0 {
            format!(
                "{:.1}%",
                (self.cached_tokens as f64 / self.total_input_tokens as f64) * 100.0
            )
        } else {
            "0.0%".to_string()
        };

        let mut obj = json!({
            "input_tokens": self.total_input_tokens,
            "output_tokens": self.output_tokens,
            "total_tokens": self.total_tokens,
        });

        if let Some(map) = obj.as_object_mut() {
            if self.cached_tokens > 0 {
                map.insert("cached_tokens".into(), json!(self.cached_tokens));
                map.insert("cache_hit_rate".into(), json!(cache_rate));
            }
            if self.reasoning_tokens > 0 {
                map.insert("reasoning_tokens".into(), json!(self.reasoning_tokens));
            }
        }
        obj
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_canonical_usage_from_gemini_and_diffusion() {
        let gemini_raw = json!({
            "usageMetadata": {
                "promptTokenCount": 99000,
                "cachedContentTokenCount": 94000,
                "candidatesTokenCount": 4100,
                "candidatesTokensDetails": [
                    { "modality": "TEXT", "tokenCount": 3100 },
                    { "modality": "THINKING", "tokenCount": 1000 }
                ]
            }
        });

        let usage = CanonicalUsage::from_gemini(&gemini_raw);
        assert_eq!(usage.total_input_tokens, 99000);
        assert_eq!(usage.cached_tokens, 94000);
        assert_eq!(usage.uncached_input_tokens, 5000);
        assert_eq!(usage.output_tokens, 4100);
        assert_eq!(usage.reasoning_tokens, 1000);
        assert_eq!(usage.total_tokens, 103100);

        // Verify diffusion to Claude
        let claude_val = usage.to_claude_usage(false, 200000);
        assert_eq!(claude_val["input_tokens"], 5000);
        assert_eq!(claude_val["output_tokens"], 4100);
        assert_eq!(claude_val["cache_read_input_tokens"], 94000);

        // Verify diffusion to OpenAI Chat
        let chat_val = usage.to_openai_chat_usage();
        assert_eq!(chat_val["prompt_tokens"], 99000);
        assert_eq!(chat_val["completion_tokens"], 4100);
        assert_eq!(chat_val["prompt_tokens_details"]["cached_tokens"], 94000);
        assert_eq!(
            chat_val["completion_tokens_details"]["reasoning_tokens"],
            1000
        );

        // Verify diffusion to OpenAI Responses
        let resp_val = usage.to_openai_responses_usage();
        assert_eq!(resp_val["input_tokens"], 99000);
        assert_eq!(resp_val["output_tokens"], 4100);
        assert_eq!(resp_val["input_tokens_details"]["cached_tokens"], 94000);
        assert_eq!(resp_val["output_tokens_details"]["reasoning_tokens"], 1000);

        // Verify diffusion to brief audit format
        let brief = usage.to_brief_audit_usage();
        assert_eq!(brief["input_tokens"], 99000);
        assert_eq!(brief["cached_tokens"], 94000);
        assert_eq!(brief["output_tokens"], 4100);
        assert_eq!(brief["reasoning_tokens"], 1000);
    }

    #[test]
    fn test_canonical_usage_healing_anthropic_input() {
        // Simulate Anthropic legacy format: input_tokens holds uncached portion (5000), cached holds (94000)
        let raw = json!({
            "input_tokens": 5000,
            "cached_tokens": 94000,
            "output_tokens": 110
        });
        let usage = CanonicalUsage::from_gemini(&raw);
        assert_eq!(usage.total_input_tokens, 99000);
        assert_eq!(usage.cached_tokens, 94000);
        assert_eq!(usage.uncached_input_tokens, 5000);
        assert_eq!(usage.output_tokens, 110);
    }
}
