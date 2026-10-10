import type { AgmConversationNode, TierClassificationResult, HierarchicalConversationNode, PromptTier, PromptCategory, PromptCategoryResult } from './types';

export function classifyPromptTier(
    text?: string,
    title?: string,
    metadata?: Record<string, unknown>
): TierClassificationResult {
    const raw = (text || '').trim();
    const cleanTitle = (title || '').trim().toLowerCase();

    // 1. Tool Output Classification (Highest Specificity)
    const isToolOutput =
        raw.startsWith('[Tool Result]') ||
        raw.startsWith('{"tool_call_id":') ||
        raw.startsWith('Tool returned:') ||
        raw.includes('<tool_response>') ||
        raw.includes('<tool_calls>') ||
        (raw.startsWith('```') && (cleanTitle.includes('output') || cleanTitle.includes('result')));

    if (isToolOutput) {
        return {
            tier: 'TOOL_OUTPUT',
            category: 'TOOL_OUTPUT',
            label: 'Tool Output',
            roleBadge: 'TOOL',
            badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
            iconName: 'wrench',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.98,
            matchedPattern: 'tool_output_marker',
            description: 'Diagnostic tool execution results and execution steps',
        };
    }

    // 2. System Message Classification
    const isSystemMessage =
        raw.startsWith('<SYSTEM_MESSAGE>') ||
        raw.includes('<conversation_transcript>') ||
        raw.includes('<artifacts>') ||
        cleanTitle.startsWith('system:');

    if (isSystemMessage && !raw.includes('invoked by a caller agent') && !raw.includes('You are')) {
        return {
            tier: 'SYSTEM_MESSAGE',
            category: 'SYSTEM_MESSAGE',
            label: 'System Message',
            roleBadge: 'SYSTEM',
            badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
            iconName: 'terminal',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.95,
            matchedPattern: 'system_message_tag',
            description: 'System initialization directives and environment preambles',
        };
    }

    // 3. AI Subagent Instruction Classification
    const subagentRoleMatch =
        raw.match(/Role:\s*([A-Za-z0-9_\-\s]{3,30})/i) ||
        raw.match(/You are (?:the )?([A-Za-z0-9_\-\s]{3,30}) for Task/i) ||
        raw.match(/You are an? ([A-Za-z0-9_\-\s]{3,30}) agent/i);

    const isSubagent =
        raw.includes('invoked by a caller agent') ||
        raw.includes('<subagent_reminder>') ||
        raw.includes('send_message to communicate all results') ||
        raw.includes('Execute enhanced Read Memory protocol') ||
        raw.includes('You are pair programming with a USER') ||
        raw.includes('Project Structure Memory Analysis') ||
        raw.includes('Safe Removal Architect') ||
        raw.includes('Spec Author') ||
        raw.includes('Codebase Researcher') ||
        raw.includes('Database Debugger') ||
        raw.includes('QA Tester') ||
        raw.includes('Read & Understand') ||
        /Role:\s*(?:Codebase Researcher|Database Debugger|QA Tester|Subagent|Worker|Architect)/i.test(raw) ||
        cleanTitle.includes('subagent') ||
        cleanTitle.includes('worker-') ||
        cleanTitle.includes('worker ') ||
        cleanTitle.includes('author-') ||
        cleanTitle.includes('researcher') ||
        cleanTitle.includes('debugger') ||
        cleanTitle.includes('memory') ||
        cleanTitle.includes('analysis') ||
        cleanTitle.includes('spec') ||
        cleanTitle.includes('tester') ||
        cleanTitle.includes('qa') ||
        metadata?.is_subagent === true ||
        metadata?.prompt_category === 'subagent';

    if (isSubagent) {
        let detectedRole = 'Subagent';
        if (subagentRoleMatch) {
            detectedRole = subagentRoleMatch[1].trim();
        } else if (cleanTitle.includes('memory') || raw.includes('Memory Analysis')) {
            detectedRole = 'Memory Analysis';
        } else if (cleanTitle.includes('spec') || raw.includes('Spec Author')) {
            detectedRole = 'Spec Author';
        } else if (cleanTitle.includes('research') || raw.includes('Researcher')) {
            detectedRole = 'Researcher';
        } else if (cleanTitle.includes('worker')) {
            detectedRole = 'Worker';
        } else if (cleanTitle.includes('debug') || raw.includes('Debugger')) {
            detectedRole = 'Debugger';
        } else if (cleanTitle.includes('tester') || cleanTitle.includes('qa') || raw.includes('QA Tester')) {
            detectedRole = 'QA Tester';
        } else if (cleanTitle.includes('architect') || raw.includes('Architect')) {
            detectedRole = 'Architect';
        } else if (cleanTitle.includes('analysis') || raw.includes('Analysis')) {
            detectedRole = 'Analysis';
        }

        return {
            tier: 'SUBAGENT_INSTRUCTION',
            category: 'SUBAGENT_INSTRUCTION',
            label: 'AI Subagent Instruction',
            roleBadge: detectedRole,
            badgeStyle: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-400/30',
            iconName: 'bot',
            isSubagent: true,
            isNonPrompt: false,
            confidence: 0.96,
            matchedPattern: 'subagent_signature',
            subagentRole: detectedRole,
            description: 'Autonomous worker prompt or subagent execution directive',
        };
    }

    // 4. Default: USER_PROMPT (Direct Human Request)
    return {
        tier: 'USER_PROMPT',
        category: 'USER_PROMPT',
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

export function detectPromptCategory(
    text?: string,
    title?: string,
    conv?: AgmConversationNode
): TierClassificationResult {
    return classifyPromptTier(text, title, conv as unknown as Record<string, unknown> | undefined);
}


export function assembleConversationHierarchy(
    conversations: AgmConversationNode[]
): HierarchicalConversationNode[] {
    // Sort chronologically ascending (oldest first) so that a user prompt precedes the subagents it spawned
    const chronological = [...conversations].sort((a, b) => {
        const aTime = new Date(a.last_modified).getTime() || 0;
        const bTime = new Date(b.last_modified).getTime() || 0;
        return aTime - bTime;
    });

    const rootNodes: HierarchicalConversationNode[] = [];
    let currentRoot: HierarchicalConversationNode | null = null;

    for (const conv of chronological) {
        const classification = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);

        const node: HierarchicalConversationNode = {
            primaryNode: conv,
            classification,
            subagents: [],
            isExpanded: true,
        };

        if (classification.tier === 'SUBAGENT_INSTRUCTION') {
            if (currentRoot) {
                // Attach as child sub-point to active root prompt
                currentRoot.subagents.push(node);
            } else {
                // If no root has occurred yet, treat as standalone root
                rootNodes.push(node);
            }
        } else {
            // New User Prompt, System Message or Tool Output becomes new root
            rootNodes.push(node);
            if (classification.tier === 'USER_PROMPT') {
                currentRoot = node;
            }
        }
    }

    // Reverse rootNodes so the newest user prompt appears at top,
    // and sort running roots to the top
    return rootNodes.reverse().sort((a, b) => {
        const aRunning = Boolean(a.primaryNode.is_running) || a.subagents.some((s) => Boolean(s.primaryNode.is_running));
        const bRunning = Boolean(b.primaryNode.is_running) || b.subagents.some((s) => Boolean(s.primaryNode.is_running));
        if (aRunning !== bRunning) return aRunning ? -1 : 1;
        return 0;
    });
}

export function normalizePromptForGrouping(conv: AgmConversationNode): string {
    const text = conv.prompt_preview_200w || conv.title || conv.short_id || conv.conversation_id;
    return text.trim().toLowerCase().replace(/\s+/g, ' ');
}

// Inline Markdown parser (Headings, bold, italic, code blocks, inline code, lists, images, blockquotes, truncated callouts)
