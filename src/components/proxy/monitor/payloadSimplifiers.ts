// ==========================================
// 简要模式载荷简化器 (Payload Simplifiers)
// 按载荷关注点拆分的纯函数：工具声明 / 工具调用 / 消息内容 / 消息列表 /
// Gemini 轮次 / 系统提示词 / 用量信息 / 深度反转义。无副作用，可独立测试。
// ==========================================

// 工具声明 (完整保留 Schema，方便开发者查看工具拼接与入参定义)
export const simplifyTools = (tools: any): any => {
    if (!Array.isArray(tools)) return undefined;
    return tools;
};

// 简化工具调用 (统一规范为: id, type: 'function', function: { name, arguments })
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

// 简化消息内容 (Claude / OpenAI parts)
export const simplifyContent = (content: any): any => {
    if (typeof content === 'string') return content;
    if (Array.isArray(content)) {
        return content.map((item: any) => {
            if (typeof item === 'string') return item;
            if (!item || typeof item !== 'object') return item;
            // Claude tool_use 块
            if (item.type === 'tool_use') {
                return {
                    type: 'tool_use',
                    id: item.id,
                    name: item.name,
                    input: item.input !== undefined ? item.input : {}
                };
            }
            // Claude tool_result 块
            if (item.type === 'tool_result') {
                return {
                    type: 'tool_result',
                    tool_use_id: item.tool_use_id,
                    ...(item.content !== undefined ? { content: item.content } : {}),
                    ...(item.is_error !== undefined ? { is_error: item.is_error } : {})
                };
            }
            // Claude thinking 块与签名
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
            // Claude redacted_thinking 块
            if (item.type === 'redacted_thinking') {
                return {
                    type: 'redacted_thinking',
                    data: item.data
                };
            }
            // 文本块
            if (item.type === 'text') {
                return item;
            }
            return item;
        });
    }
    return content;
};

// 简化消息列表
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

// 简化 Gemini 轮次 (contents)
export const simplifyGeminiContents = (contents: any): any => {
    if (!Array.isArray(contents)) return undefined;
    return contents.map((c: any) => {
        if (!c || typeof c !== 'object') return c;
        const res: any = { role: c.role };
        if (Array.isArray(c.parts)) {
            res.parts = c.parts.map((p: any) => {
                if (!p || typeof p !== 'object') return p;

                // 1. 优先识别工具调用 (functionCall) 并保留其名称、ID、参数与携带的加密思考签名
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

                // 2. 优先识别工具响应 (functionResponse) 并保留其名称、ID、返回值与携带的签名
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

                // 3. 独立思考块 (纯思考过程，不带工具调用)
                if (p.thought !== undefined || p.thought_signature !== undefined || p.thoughtSignature !== undefined || p.signature !== undefined) {
                    const tPart: any = {};
                    if (p.thought !== undefined) tPart.thought = p.thought;
                    if (p.thought_signature !== undefined) tPart.thought_signature = p.thought_signature;
                    if (p.thoughtSignature !== undefined) tPart.thoughtSignature = p.thoughtSignature;
                    if (p.signature !== undefined) tPart.signature = p.signature;
                    if (p.text !== undefined) tPart.text = p.text;
                    return tPart;
                }

                // 4. 普通文本块
                if (p.text !== undefined) {
                    return { text: p.text };
                }

                return p;
            });
        }
        return res;
    });
};

// 简化系统提示词 (Gemini / Anthropic)
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

// 提取用量与缓存命中率
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

    // 计算全量上下文输入 Token (Total Context Input)
    // 1. Anthropic 官方协议: input_tokens 仅代表未缓存增量，总上下文 = input_tokens + cache_read_input_tokens
    // 2. 兼容历史日志: 若 cached > rawInput，说明 rawInput 存的是未缓存差值，做自愈加和
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
 * 递归深度反转义并反序列化嵌套在 JSON 字符串属性中的 JSON 内容
 * 例如将 "response": "{\"error\":{\"code\":400...}}" 自动展开为真实的嵌套对象
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
