// ==========================================
// Payload Simplifiers
// Pure functions split by payload concerns: tool declaration / tool call / message content / message list /
// Gemini rounds / system prompt words / usage information / deep anti-escaping. No side effects and can be independently tested.
// ==========================================

// Tool declaration (the Schema is retained completely to facilitate developers to view tool splicing and input parameter definitions)
export const simplifyTools = (tools: any): any => {
    if (!Array.isArray(tools)) return undefined;
    return tools;
};

// Simplify tool calls (unified specification: id, type: 'function', function: { name, arguments })
export const simplifyToolCalls = (toolCalls: any): any => {
    if (!Array.isArray(toolCalls)) return undefined;
    return toolCalls.map((tc: any) => {
        if (!tc || typeof tc !== 'object') return tc;
        const res: any = {};
        if (tc.id) res.id = tc.id;
        res.type = tc.type || 'function';
        if (tc.function && typeof tc.function === 'object') {
            res.function = {
                name: tc.function.name,
                arguments: tc.function.arguments !== undefined ? tc.function.arguments : {}
            };
        } else {
            const name = tc.name || tc.function?.name || 'unknown';
            const args = tc.arguments !== undefined ? tc.arguments : (tc.args !== undefined ? tc.args : (tc.input !== undefined ? tc.input : {}));
            res.function = {
                name,
                arguments: args
            };
        }
        return res;
    });
};

// Simplify message content (Claude / OpenAI parts)
export const simplifyContent = (content: any): any => {
    if (typeof content === 'string') return content;
    if (Array.isArray(content)) {
        return content.map((item: any) => {
            if (typeof item === 'string') return item;
            if (!item || typeof item !== 'object') return item;
            // Claude tool_use block
            if (item.type === 'tool_use') {
                return {
                    type: 'tool_use',
                    id: item.id,
                    name: item.name,
                    input: item.input !== undefined ? item.input : {}
                };
            }
            // Claude tool_result block
            if (item.type === 'tool_result') {
                return {
                    type: 'tool_result',
                    tool_use_id: item.tool_use_id,
                    ...(item.content !== undefined ? { content: item.content } : {}),
                    ...(item.is_error !== undefined ? { is_error: item.is_error } : {})
                };
            }
            // Claude thinking blocks and signatures
            if (item.type === 'thinking') {
                return {
                    type: 'thinking',
                    thinking: item.thinking,
                    ...(item.signature !== undefined ? { signature: item.signature } : {}),
                    ...(item.thought_signature !== undefined ? { thought_signature: item.thought_signature } : {}),
                    ...(item.thoughtSignature !== undefined ? { thoughtSignature: item.thoughtSignature } : {}),
                    ...(item.thinking_signature !== undefined ? { thinking_signature: item.thinking_signature } : {})
                };
            }
            // Claude redacted_thinking block
            if (item.type === 'redacted_thinking') {
                return {
                    type: 'redacted_thinking',
                    data: item.data
                };
            }
            // text block
            if (item.type === 'text') {
                return item;
            }
            return item;
        });
    }
    return content;
};

// Simplified message list
export const simplifyMessages = (messages: any): any => {
    if (!Array.isArray(messages)) return undefined;
    return messages.map((m: any) => {
        if (!m || typeof m !== 'object') return m;
        const res: any = { role: m.role };
        if (m.content !== undefined) {
            res.content = simplifyContent(m.content);
        }
        if (m.reasoning_content !== undefined) {
            res.reasoning_content = m.reasoning_content;
        }
        if (m.thinking !== undefined) {
            res.thinking = m.thinking;
        }
        if (m.signature !== undefined) {
            res.signature = m.signature;
        }
        if (m.thought_signature !== undefined) {
            res.thought_signature = m.thought_signature;
        }
        if (m.thinking_signature !== undefined) {
            res.thinking_signature = m.thinking_signature;
        }
        if (m.tool_calls) {
            res.tool_calls = simplifyToolCalls(m.tool_calls);
        }
        if (m.tool_call_id) {
            res.tool_call_id = m.tool_call_id;
        }
        if (m.name) {
            res.name = m.name;
        }
        return res;
    });
};

// Simplifying Gemini rounds (contents)
export const simplifyGeminiContents = (contents: any): any => {
    if (!Array.isArray(contents)) return undefined;
    return contents.map((c: any) => {
        if (!c || typeof c !== 'object') return c;
        const res: any = { role: c.role };
        if (Array.isArray(c.parts)) {
            res.parts = c.parts.map((p: any) => {
                if (!p || typeof p !== 'object') return p;

                // 1. Prioritize the identification of tool calls (functionCall) and retain their names, IDs, parameters and encrypted thinking signatures
                if (p.functionCall) {
                    const fcPart: any = {
                        functionCall: {
                            name: p.functionCall.name,
                            ...(p.functionCall.id ? { id: p.functionCall.id } : {}),
                            args: p.functionCall.args !== undefined ? p.functionCall.args : {}
                        }
                    };
                    if (p.thought !== undefined) fcPart.thought = p.thought;
                    if (p.thoughtSignature !== undefined) fcPart.thoughtSignature = p.thoughtSignature;
                    if (p.thought_signature !== undefined) fcPart.thought_signature = p.thought_signature;
                    if (p.signature !== undefined) fcPart.signature = p.signature;
                    return fcPart;
                }

                // 2. Prioritize the tool response (functionResponse) and retain its name, ID, return value and carried signature
                if (p.functionResponse) {
                    const frPart: any = {
                        functionResponse: {
                            name: p.functionResponse.name,
                            ...(p.functionResponse.id ? { id: p.functionResponse.id } : {}),
                            response: p.functionResponse.response !== undefined ? p.functionResponse.response : {}
                        }
                    };
                    if (p.thought !== undefined) frPart.thought = p.thought;
                    if (p.thoughtSignature !== undefined) frPart.thoughtSignature = p.thoughtSignature;
                    if (p.thought_signature !== undefined) frPart.thought_signature = p.thought_signature;
                    if (p.signature !== undefined) frPart.signature = p.signature;
                    return frPart;
                }

                // 3. Independent thinking block (pure thinking process, without tool invocation)
                if (p.thought !== undefined || p.thought_signature !== undefined || p.thoughtSignature !== undefined || p.signature !== undefined) {
                    const tPart: any = {};
                    if (p.thought !== undefined) tPart.thought = p.thought;
                    if (p.thought_signature !== undefined) tPart.thought_signature = p.thought_signature;
                    if (p.thoughtSignature !== undefined) tPart.thoughtSignature = p.thoughtSignature;
                    if (p.signature !== undefined) tPart.signature = p.signature;
                    if (p.text !== undefined) tPart.text = p.text;
                    return tPart;
                }

                // 4. Ordinary text block
                if (p.text !== undefined) {
                    return { text: p.text };
                }

                return p;
            });
        }
        return res;
    });
};

// Simplified system prompt words (Gemini / Anthropic)
export const simplifySystemInstruction = (sys: any): any => {
    if (!sys || typeof sys !== 'object') return sys;
    if (Array.isArray(sys.parts)) {
        return {
            parts: sys.parts.map((p: any) => {
                if (typeof p === 'string') return { text: p };
                if (p && typeof p === 'object' && p.text !== undefined) return { text: p.text };
                return p;
            })
        };
    }
    return sys;
};

// Fetch usage and cache hit rate
export const simplifyUsage = (usage: any): any => {
    if (!usage || typeof usage !== 'object') return undefined;
    const res: any = {};
    const rawInput = usage.prompt_tokens ?? usage.input_tokens ?? usage.promptTokenCount;
    const output = usage.completion_tokens ?? usage.output_tokens ?? usage.candidatesTokenCount;

    let cached = usage.cached_tokens ?? usage.cache_read_input_tokens ?? usage.cachedContentTokenCount;
    if (cached == null && usage.prompt_tokens_details?.cached_tokens != null) {
        cached = usage.prompt_tokens_details.cached_tokens;
    }
    if (cached == null && usage.input_tokens_details?.cached_tokens != null) {
        cached = usage.input_tokens_details.cached_tokens;
    }

    // Calculate the total context input Token (Total Context Input)
    // 1. Anthropic official protocol: input_tokens only represents uncached increments, total context = input_tokens + cache_read_input_tokens
    // 2. Compatible with historical logs: If cached > rawInput, it means that rawInput stores uncached differences, and performs self-healing and summing
    let totalInput = rawInput != null ? Number(rawInput) : undefined;
    if (cached != null && totalInput != null && cached > totalInput) {
        totalInput = totalInput + Number(cached);
    } else if (usage.cache_read_input_tokens != null && usage.prompt_tokens == null && usage.promptTokenCount == null) {
        totalInput = Number(usage.input_tokens || 0) + Number(cached || 0);
    }

    const total = usage.total_tokens ?? usage.totalTokenCount ?? (totalInput != null && output != null ? totalInput + Number(output) : undefined);

    if (totalInput != null) res.input_tokens = totalInput;
    if (output != null) res.output_tokens = Number(output);
    if (total != null) res.total_tokens = Number(total);
    if (cached != null) {
        res.cached_tokens = Number(cached);
        if (totalInput != null && totalInput > 0) {
            const rate = Math.min(100, Math.max(0, (Number(cached) / totalInput) * 100));
            res.cache_hit_rate = `${rate.toFixed(1)}%`;
        }
    }
    if (usage.cache_creation_input_tokens != null) {
        res.cache_creation_input_tokens = usage.cache_creation_input_tokens;
    }
    if (usage.completion_tokens_details?.reasoning_tokens != null) {
        res.reasoning_tokens = usage.completion_tokens_details.reasoning_tokens;
    }
    if (usage.output_tokens_details?.reasoning_tokens != null) {
        res.reasoning_tokens = usage.output_tokens_details.reasoning_tokens;
    }
    return res;
};

/**
 * Recursively deeply unescape and deserialize JSON content nested within JSON string properties
 * For example, "response": "{\"error\":{\"code\":400...}}" will automatically expand into a real nested object
 */
export const deepUnescapeJsonValue = (val: any): any => {
    if (typeof val === 'string') {
        const trimmed = val.trim();
        if ((trimmed.startsWith('{') && trimmed.endsWith('}')) || (trimmed.startsWith('[') && trimmed.endsWith(']'))) {
            try {
                const parsed = JSON.parse(trimmed);
                return deepUnescapeJsonValue(parsed);
            } catch {
                return val;
            }
        }
        return val;
    }
    if (Array.isArray(val)) {
        return val.map(deepUnescapeJsonValue);
    }
    if (val && typeof val === 'object') {
        const res: Record<string, any> = {};
        for (const [k, v] of Object.entries(val)) {
            res[k] = deepUnescapeJsonValue(v);
        }
        return res;
    }
    return val;
};
