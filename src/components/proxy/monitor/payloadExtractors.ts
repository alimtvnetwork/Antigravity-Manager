import {
    deepUnescapeJsonValue,
    simplifyContent,
    simplifyGeminiContents,
    simplifyMessages,
    simplifySystemInstruction,
    simplifyToolCalls,
    simplifyTools,
    simplifyUsage,
} from './payloadSimplifiers';
import type { ProxyRequestLog } from './types';

// ==========================================
// Brief pattern intelligent extraction and mapping algorithm
// ==========================================
export function extractConcisePayload(
    rawStr: string | undefined,
    kind: 'request' | 'upstream' | 'response',
    log?: ProxyRequestLog | null
): string {
    if (!rawStr) return '';
    let obj: any;
    try {
        obj = JSON.parse(rawStr);
    } catch {
        return rawStr;
    }
    if (!obj || typeof obj !== 'object') {
        return rawStr;
    }

    const concise: any = {};

    // Reserve a single line identifier used to identify the think block/session (supports requestId, sessionId, trace_id, etc.)
    const candidateSessionId =
        obj.requestId ||
        obj.request?.sessionId ||
        obj._session_id ||
        obj.session_id ||
        (log?.id ? log.id : undefined);

    if (candidateSessionId) {
        concise._session_thinking_id = candidateSessionId;
    }

    // Model
    if (obj.model) concise.model = obj.model;

    // Think about model configuration (start, budget, effort, summary)
    if (obj.thinking !== undefined) concise.thinking = obj.thinking;
    if (obj.reasoning_effort !== undefined) concise.reasoning_effort = obj.reasoning_effort;
    if (obj.reasoning !== undefined) concise.reasoning = obj.reasoning;
    if (obj.summary !== undefined) concise.summary = obj.summary;
    if (obj.generationConfig?.thinkingConfig !== undefined) {
        concise.thinkingConfig = obj.generationConfig.thinkingConfig;
    } else if (obj.thinkingConfig !== undefined) {
        concise.thinkingConfig = obj.thinkingConfig;
    }

    // System prompt word
    if (obj.system !== undefined) concise.system = obj.system;
    if (obj.systemInstruction !== undefined) concise.systemInstruction = simplifySystemInstruction(obj.systemInstruction);

    // Conversation Subject (OpenAI / Claude)
    if (obj.messages) {
        concise.messages = simplifyMessages(obj.messages);
    }

    // Conversation subject (Gemini)
    if (obj.contents) {
        concise.contents = simplifyGeminiContents(obj.contents);
    }

    // Tool declaration
    if (obj.tools) {
        concise.tools = simplifyTools(obj.tools);
    }

    // Antigravity's dedicated request nesting packaging layer (core: correctly maps the nesting level of the original forwarded message)
    if (obj.request && typeof obj.request === 'object') {
        const innerReq: any = {};

        // Single line session ID
        if (obj.request.sessionId) {
            innerReq.sessionId = obj.request.sessionId;
        }

        // thinking configuration (thinkingConfig / generationConfig)
        if (obj.request.generationConfig?.thinkingConfig !== undefined) {
            innerReq.thinkingConfig = obj.request.generationConfig.thinkingConfig;
        } else if (obj.request.thinkingConfig !== undefined) {
            innerReq.thinkingConfig = obj.request.thinkingConfig;
        }

        // System prompt word
        if (obj.request.systemInstruction !== undefined) {
            innerReq.systemInstruction = simplifySystemInstruction(obj.request.systemInstruction);
        }

        // Conversation bodies and thought blocks (Gemini contents or Claude messages)
        if (obj.request.contents) {
            innerReq.contents = simplifyGeminiContents(obj.request.contents);
        }
        if (obj.request.messages) {
            innerReq.messages = simplifyMessages(obj.request.messages);
        }

        // Tool declaration
        if (obj.request.tools) {
            innerReq.tools = simplifyTools(obj.request.tools);
        }

        concise.request = innerReq;
    }

    // Responsive: Think Blocks vs. Think Signatures (top-level responsive or non-streaming)
    if (obj.thinking !== undefined) concise.thinking = obj.thinking;
    if (obj.thinking_signature !== undefined) concise.thinking_signature = obj.thinking_signature;
    if (obj.thought_signature !== undefined) concise.thought_signature = obj.thought_signature;
    if (obj.signature !== undefined) concise.signature = obj.signature;
    if (obj.thoughtSignature !== undefined) concise.thoughtSignature = obj.thoughtSignature;
    if (obj._timing !== undefined) concise._timing = obj._timing;

    // 🌟 Standardized extraction of response messages: If it is response, priority is given to flattening the choices / candidates / content array into a top-level unified structure
    if (kind === 'response') {
        if (obj.choices && Array.isArray(obj.choices) && obj.choices.length > 0) {
            const first = obj.choices[0];
            const msg = first?.message || first?.delta;
            if (msg) {
                if (concise.thinking === undefined) {
                    const th = msg.reasoning_content || msg.thinking;
                    if (th) concise.thinking = th;
                }
                if (concise.thinking_signature === undefined) {
                    const sig = msg.thoughtSignature || msg.thought_signature || msg.signature;
                    if (sig) concise.thinking_signature = sig;
                }
                if (concise.content === undefined && msg.content !== undefined) {
                    concise.content = typeof msg.content === 'string' ? msg.content : simplifyContent(msg.content);
                }
                if (concise.tool_calls === undefined && msg.tool_calls) {
                    concise.tool_calls = simplifyToolCalls(msg.tool_calls);
                }
            }
        } else if (obj.candidates && Array.isArray(obj.candidates) && obj.candidates.length > 0) {
            const parts = obj.candidates[0]?.content?.parts;
            if (Array.isArray(parts)) {
                let thText = '';
                let normalText = '';
                let sigText = '';
                const extractedTools: any[] = [];
                for (const p of parts) {
                    if (p.text) {
                        if (p.thought) thText += p.text;
                        else normalText += p.text;
                    }
                    const s = p.thoughtSignature || p.thought_signature || p.signature || p.functionCall?.thoughtSignature || p.functionCall?.thought_signature;
                    if (s) {
                        if (!sigText) sigText = s;
                    }
                    if (p.functionCall) {
                        extractedTools.push({
                            id: p.functionCall.id || '',
                            type: 'function',
                            function: {
                                name: p.functionCall.name || 'unknown',
                                arguments: p.functionCall.args !== undefined ? (typeof p.functionCall.args === 'string' ? p.functionCall.args : JSON.stringify(p.functionCall.args)) : '{}'
                            }
                        });
                    }
                }
                if (concise.thinking === undefined && thText) concise.thinking = thText;
                if (concise.thinking_signature === undefined && sigText) concise.thinking_signature = sigText;
                if (concise.content === undefined && normalText) concise.content = normalText;
                if (concise.tool_calls === undefined && extractedTools.length > 0) concise.tool_calls = simplifyToolCalls(extractedTools);
            }
        } else if (Array.isArray(obj.content)) {
            if (!obj.messages && !obj.choices) {
            let thText = '';
            let sigText = '';
            let normalText = '';
            const extractedTools: any[] = [];
            for (const item of obj.content) {
                if (item && typeof item === 'object') {
                    if (item.type === 'thinking') {
                        if (item.thinking) thText += item.thinking;
                        const s = item.signature || item.thought_signature || item.thoughtSignature;
                        if (s) {
                        if (!sigText) sigText = s;
                    }
                    } else if (item.type === 'text' && item.text) {
                        normalText += item.text;
                    } else if (item.type === 'tool_use') {
                        extractedTools.push({
                            id: item.id || '',
                            type: 'function',
                            function: {
                                name: item.name || 'unknown',
                                arguments: item.input !== undefined ? (typeof item.input === 'string' ? item.input : JSON.stringify(item.input)) : '{}'
                            }
                        });
                    }
                }
            }
            if (concise.thinking === undefined && thText) concise.thinking = thText;
            if (concise.thinking_signature === undefined && sigText) concise.thinking_signature = sigText;
            if (concise.content === undefined && normalText) concise.content = normalText;
            if (concise.tool_calls === undefined && extractedTools.length > 0) concise.tool_calls = simplifyToolCalls(extractedTools);
            }
        }
    }

    // Response: Choices / Candidates / Aggregated response (if it is a request or a response that is not flattened and extracted, keep choices/candidates)
    if (kind !== 'response' || (!concise.content && !concise.tool_calls && !concise.thinking)) {
        if (obj.choices && Array.isArray(obj.choices)) {
            concise.choices = obj.choices.map((c: any) => {
                const choiceRes: any = { index: c.index };
                if (c.finish_reason) choiceRes.finish_reason = c.finish_reason;
                if (c.message) {
                    choiceRes.message = {
                        role: c.message.role,
                        ...(c.message.reasoning_content !== undefined ? { reasoning_content: c.message.reasoning_content } : {}),
                        ...(c.message.thinking !== undefined ? { thinking: c.message.thinking } : {}),
                        ...(c.message.thinking_signature !== undefined ? { thinking_signature: c.message.thinking_signature } : {}),
                        ...(c.message.thought_signature !== undefined ? { thought_signature: c.message.thought_signature } : {}),
                        ...(c.message.signature !== undefined ? { signature: c.message.signature } : {}),
                        ...(c.message.content !== undefined ? { content: c.message.content } : {}),
                        ...(c.message.tool_calls ? { tool_calls: simplifyToolCalls(c.message.tool_calls) } : {})
                    };
                } else if (c.delta) {
                    choiceRes.delta = {
                        role: c.delta.role,
                        ...(c.delta.reasoning_content !== undefined ? { reasoning_content: c.delta.reasoning_content } : {}),
                        ...(c.delta.thinking !== undefined ? { thinking: c.delta.thinking } : {}),
                        ...(c.delta.thinking_signature !== undefined ? { thinking_signature: c.delta.thinking_signature } : {}),
                        ...(c.delta.thought_signature !== undefined ? { thought_signature: c.delta.thought_signature } : {}),
                        ...(c.delta.signature !== undefined ? { signature: c.delta.signature } : {}),
                        ...(c.delta.content !== undefined ? { content: c.delta.content } : {}),
                        ...(c.delta.tool_calls ? { tool_calls: simplifyToolCalls(c.delta.tool_calls) } : {})
                    };
                }
                return choiceRes;
            });
        }

        if (obj.candidates && Array.isArray(obj.candidates)) {
            concise.candidates = obj.candidates.map((cand: any) => {
                const candRes: any = {};
                if (cand.finishReason) candRes.finishReason = cand.finishReason;
                if (cand.content) {
                    candRes.content = simplifyGeminiContents([cand.content])?.[0] || cand.content;
                }
                return candRes;
            });
        }
    }

    if (obj.input !== undefined) {
        concise.input = typeof obj.input === 'string' ? obj.input : (Array.isArray(obj.input) ? simplifyMessages(obj.input) : obj.input);
    }
    if (obj.output !== undefined) {
        concise.output = obj.output;
    }
    if (obj.prompt !== undefined) {
        concise.prompt = obj.prompt;
    }
    if (obj.instructions !== undefined) {
        concise.instructions = obj.instructions;
    }

    if (obj.content !== undefined) {
        if (!obj.messages && !obj.choices && !obj.request) {
            concise.content = simplifyContent(obj.content);
        }
    }
    if (obj.reasoning_content !== undefined) {
        if (!obj.messages && !obj.choices) {
            concise.reasoning_content = obj.reasoning_content;
        }
    }
    if (obj.tool_calls) {
        if (!obj.messages && !obj.choices) {
            concise.tool_calls = simplifyToolCalls(obj.tool_calls);
        }
    }

    // Error response purification: Do not castrate double-layer error reporting, fully present gateway diagnosis and upstream original errors
    if (obj.type !== undefined && !obj.messages && !obj.choices) concise.type = obj.type;
    if (obj.code !== undefined && !obj.messages && !obj.choices) concise.code = obj.code;
    if (obj.status !== undefined && !obj.messages && !obj.choices) concise.status = obj.status;
    if (obj.error !== undefined) {
        concise.error = deepUnescapeJsonValue(obj.error);
    }
    if (obj.gateway_error !== undefined) {
        concise.gateway_error = deepUnescapeJsonValue(obj.gateway_error);
    }
    if (obj.upstream_error !== undefined) {
        concise.upstream_error = deepUnescapeJsonValue(obj.upstream_error);
    }

    // Usage and cache
    const usage = simplifyUsage(obj.usage || obj.usageMetadata);
    if (usage) {
        concise.usage = usage;
    } else if (kind === 'response' && (log?.input_tokens || log?.output_tokens)) {
        const totalIn = (log.cached_tokens && log.cached_tokens > (log.input_tokens || 0))
            ? (log.input_tokens || 0) + log.cached_tokens
            : (log.input_tokens || 0);
        concise.usage = {
            input_tokens: totalIn,
            output_tokens: log.output_tokens,
            total_tokens: totalIn + (log.output_tokens || 0),
            ...(log.cached_tokens != null ? {
                cached_tokens: log.cached_tokens,
                cache_hit_rate: totalIn > 0 ? `${Math.min(100, Math.max(0, (log.cached_tokens / totalIn) * 100)).toFixed(1)}%` : undefined
            } : {})
        };
    }

    const substantiveKeys = Object.keys(concise).filter(k => k !== '_session_thinking_id');
    if (substantiveKeys.length === 0) {
        try {
            let parsed = JSON.parse(rawStr);
            if (typeof parsed === 'string') {
                try {
                    parsed = JSON.parse(parsed);
                } catch {
                    // Justification: format probe — a non-JSON string here is the expected common case
                    // (plain-text payloads), not an error; the original string is formatted as-is below.
                }
            }
            return JSON.stringify(deepUnescapeJsonValue(parsed), null, 2);
        } catch {
            return rawStr;
        }
    }

    return JSON.stringify(concise, null, 2);
}
