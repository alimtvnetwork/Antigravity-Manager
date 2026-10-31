import { PromptOriginTier } from './types';
import type { TierClassificationResult } from './types';
import { classifyPromptTier } from './tierClassification';

export { PromptOriginTier };
export type { TierClassificationResult };

/**
 * Imp-13: 4-Tier Origin Badge Classifier
 * Standardizes mapping from raw origin string or payload to the 4 canonical origin tiers:
 * - USER_PROMPT (Sky badge, User icon)
 * - SUBAGENT_INSTRUCTION (Purple badge, Bot icon)
 * - SYSTEM_MESSAGE (Zinc badge, Terminal icon)
 * - TOOL_OUTPUT (Amber badge, Wrench icon)
 */
export function classifyPromptOrigin(originStr?: string, payloadText?: string): TierClassificationResult {
    const cleanOrigin = (originStr || '').trim().toLowerCase();
    
    if (cleanOrigin.includes('tool') || cleanOrigin.includes('output') || cleanOrigin.includes('wrench')) {
        return {
            tier: PromptOriginTier.ToolOutput,
            category: PromptOriginTier.ToolOutput,
            label: 'Tool Output',
            roleBadge: 'TOOL',
            badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
            iconName: 'wrench',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.98,
            matchedPattern: 'tool_output_tier',
            description: 'Diagnostic tool execution results and execution steps',
        };
    }

    if (cleanOrigin.includes('subagent') || cleanOrigin.includes('worker') || cleanOrigin.includes('agent')) {
        return {
            tier: PromptOriginTier.SubagentInstruction,
            category: PromptOriginTier.SubagentInstruction,
            label: 'AI Subagent Instruction',
            roleBadge: 'SUBAGENT',
            badgeStyle: 'text-purple-900 dark:text-amber-200 bg-purple-500/20 border-purple-400/40',
            iconName: 'bot',
            isSubagent: true,
            isNonPrompt: false,
            confidence: 0.96,
            matchedPattern: 'subagent_tier',
            description: 'Autonomous worker prompt or subagent execution directive',
        };
    }

    if (cleanOrigin.includes('system') || cleanOrigin.includes('terminal')) {
        return {
            tier: PromptOriginTier.SystemMessage,
            category: PromptOriginTier.SystemMessage,
            label: 'System Message',
            roleBadge: 'SYSTEM',
            badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
            iconName: 'terminal',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.95,
            matchedPattern: 'system_message_tier',
            description: 'System initialization directives and environment preambles',
        };
    }

    // If payloadText is supplied, run full deep heuristic classifier
    if (payloadText) {
        return classifyPromptTier(payloadText, originStr);
    }

    // Default to USER_PROMPT
    return {
        tier: PromptOriginTier.UserPrompt,
        category: PromptOriginTier.UserPrompt,
        label: 'User Prompt',
        roleBadge: 'USER',
        badgeStyle: 'bg-sky-500/15 text-sky-700 dark:text-cyan-300 border-sky-400/30',
        iconName: 'user',
        isSubagent: false,
        isNonPrompt: false,
        confidence: 0.90,
        description: 'Direct human directive submitted via chat, CLI, or task request',
    };
}
