import { PromptOriginTier } from './types';
import type { AgmConversationNode, TierClassificationResult, HierarchicalConversationNode } from './types';
import { getConversationStatus } from './conversationGuards';

/**
 * Evaluates whether text and title indicate diagnostic or tool output.
 */
export function isToolOutputPayload(text: string, cleanTitle: string): boolean {
    const isResultPrefix = text.startsWith('[Tool Result]') || text.startsWith('Tool returned:');
    if (isResultPrefix) return true;

    const isJsonCallId = text.startsWith('{"tool_call_id":');
    if (isJsonCallId) return true;

    const hasToolTags = text.includes('<tool_response>') || text.includes('<tool_calls>');
    if (hasToolTags) return true;

    const isCodeBlock = text.startsWith('```');
    const hasOutputKeyword = cleanTitle.includes('output') || cleanTitle.includes('result');
    return isCodeBlock && hasOutputKeyword;
}

/**
 * Detects presence of explicit user request tags or section markers.
 */
export function hasUserContent(text: string): boolean {
    const hasUserTag = text.includes('<USER_REQUEST>') || text.includes('<user_request>');
    const hasUserHeader = text.includes('User Requests') || text.includes('# User Requests');
    return hasUserTag || hasUserHeader;
}

/**
 * Evaluates whether a payload constitutes a system-level initialization directive.
 */
export function isSystemMessageDirective(text: string, cleanTitle: string): boolean {
    const hasSystemPrefix = text.startsWith('<SYSTEM_MESSAGE>') || text.startsWith('System:');
    const hasTitlePrefix = cleanTitle.startsWith('system:');
    const isCandidate = hasSystemPrefix || hasTitlePrefix;

    if (!isCandidate) {
        return false;
    }
    if (hasUserContent(text)) {
        return false;
    }
    const isCallerPreamble = text.includes('invoked by a caller agent') || text.includes('You are');
    return !isCallerPreamble;
}

/**
 * Checks whether the normalized conversation title indicates a subagent or worker.
 */
export function isSubagentTitle(cleanTitle: string): boolean {
    const isSubagentPrefix =
        cleanTitle.startsWith('subagent:') ||
        cleanTitle.startsWith('subagent-') ||
        cleanTitle.startsWith('subagent ') ||
        cleanTitle.startsWith('[subagent]');
    if (isSubagentPrefix) return true;

    const isWorkerPrefix =
        cleanTitle.startsWith('worker:') ||
        cleanTitle.startsWith('worker-') ||
        cleanTitle.startsWith('worker ');
    return isWorkerPrefix;
}

/**
 * Detects multi-agent invocation signatures and operational reminders in prompt text.
 */
export function hasSubagentSignature(text: string): boolean {
    const hasInvocationMarker =
        text.includes('invoked by a caller agent') ||
        text.includes('<subagent_reminder>') ||
        text.includes('send_message to communicate all results');
    if (hasInvocationMarker) return true;

    const rolePattern = /Role:\s*(?:Codebase Researcher|Database Debugger|QA Tester|Subagent|Worker|Architect)/i;
    return rolePattern.test(text);
}

/**
 * Evaluates whether conversation metadata indicates a subagent.
 */
export function isSubagentMetadata(metadata?: Record<string, any>): boolean {
    const isMetadataFlagged = Boolean(metadata?.is_subagent);
    const isCategorySubagent = metadata?.prompt_category === 'subagent';
    return isMetadataFlagged || isCategorySubagent;
}

/**
 * Determines whether the message originates from an autonomous subagent worker.
 */
export function isSubagentDirective(
    text: string,
    cleanTitle: string,
    metadata?: Record<string, any>
): boolean {
    if (hasSubagentSignature(text)) {
        return true;
    }
    if (isSubagentTitle(cleanTitle)) {
        return true;
    }
    return isSubagentMetadata(metadata);
}

/**
 * Checks if a child conversation's timestamp falls within the correlation window of a root prompt.
 */
export function isRecentChildTurn(
    nodeTime: number,
    rootTime: number,
    maxGapMs: number = 2 * 3600 * 1000
): boolean {
    if (rootTime === 0 || nodeTime === 0) {
        return false;
    }
    const timeDelta = Math.abs(nodeTime - rootTime);
    return timeDelta <= maxGapMs;
}

/**
 * Evaluates whether a hierarchical root node or any attached subagents are actively running.
 */
export function isHierarchicalNodeRunning(node: HierarchicalConversationNode): boolean {
    const isPrimaryRunning = getConversationStatus(node.primaryNode).isRunning;
    if (isPrimaryRunning) {
        return true;
    }
    return node.subagents.some((s) => getConversationStatus(s.primaryNode).isRunning);
}

/**
 * Evaluates whether a hierarchical root node or any attached subagents are in queued status.
 */
export function isHierarchicalNodeQueued(node: HierarchicalConversationNode): boolean {
    const isPrimaryQueued = getConversationStatus(node.primaryNode).isQueued;
    if (isPrimaryQueued) {
        return true;
    }
    return node.subagents.some((s) => getConversationStatus(s.primaryNode).isQueued);
}

/**
 * Sorts hierarchical nodes by running status first, queued status second, and newest time third.
 */
export function compareHierarchicalNodes(
    a: HierarchicalConversationNode,
    b: HierarchicalConversationNode
): number {
    const isARunning = isHierarchicalNodeRunning(a);
    const isBRunning = isHierarchicalNodeRunning(b);
    if (isARunning !== isBRunning) {
        return isARunning ? -1 : 1;
    }

    const isAQueued = isHierarchicalNodeQueued(a);
    const isBQueued = isHierarchicalNodeQueued(b);
    if (isAQueued !== isBQueued) {
        return isAQueued ? -1 : 1;
    }

    const aTime = new Date(a.primaryNode.last_modified).getTime() || 0;
    const bTime = new Date(b.primaryNode.last_modified).getTime() || 0;
    return bTime - aTime;
}

export function classifyPromptTier(
    text?: string,
    title?: string,
    metadata?: Record<string, any>
): TierClassificationResult {
    const raw = (text || '').trim();
    const cleanTitle = (title || '').trim().toLowerCase();

    // 1. Tool Output Classification (Highest Specificity)
    if (isToolOutputPayload(raw, cleanTitle)) {
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
            matchedPattern: 'tool_output_marker',
            description: 'Diagnostic tool execution results and execution steps',
        };
    }

    // 2. System Message Classification
    if (isSystemMessageDirective(raw, cleanTitle)) {
        const isRunning = Boolean(metadata?.is_running);
        const isQueued = Boolean(metadata?.is_queued);
        const isMetadataActive = isRunning || isQueued;

        return {
            tier: PromptOriginTier.SystemMessage,
            category: PromptOriginTier.SystemMessage,
            label: 'System Message',
            roleBadge: 'SYSTEM',
            badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
            iconName: 'terminal',
            isSubagent: false,
            isNonPrompt: !isMetadataActive,
            confidence: 0.95,
            matchedPattern: 'system_message_tag',
            description: 'System initialization directives and environment preambles',
        };
    }

    // 3. AI Subagent Instruction Classification
    if (isSubagentDirective(raw, cleanTitle, metadata)) {
        const subagentRoleMatch =
            raw.match(/Role:\s*([A-Za-z0-9_\-\s]{3,30})/i) ||
            raw.match(/You are (?:the )?([A-Za-z0-9_\-\s]{3,30}) for Task/i) ||
            raw.match(/You are an? ([A-Za-z0-9_\-\s]{3,30}) agent/i);

        let detectedRole = 'Subagent';
        if (subagentRoleMatch) {
            detectedRole = subagentRoleMatch[1].trim();
        } else if (cleanTitle.startsWith('worker')) {
            detectedRole = 'Worker';
        } else if (cleanTitle.includes('research') || raw.includes('Researcher')) {
            detectedRole = 'Researcher';
        } else if (cleanTitle.includes('debug') || raw.includes('Debugger')) {
            detectedRole = 'Debugger';
        } else if (cleanTitle.includes('tester') || raw.includes('QA Tester')) {
            detectedRole = 'QA Tester';
        } else if (cleanTitle.includes('architect') || raw.includes('Architect')) {
            detectedRole = 'Architect';
        }

        return {
            tier: PromptOriginTier.SubagentInstruction,
            category: PromptOriginTier.SubagentInstruction,
            label: 'AI Subagent Instruction',
            roleBadge: detectedRole,
            badgeStyle: 'text-purple-900 dark:text-amber-200 bg-purple-500/20 border-purple-400/40',
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

        if (classification.tier === PromptOriginTier.SubagentInstruction) {
            const rootTime = currentRoot ? (new Date(currentRoot.primaryNode.last_modified).getTime() || 0) : 0;
            const nodeTime = new Date(conv.last_modified).getTime() || 0;
            const isRecentChild = currentRoot !== null && isRecentChildTurn(nodeTime, rootTime);

            if (isRecentChild && currentRoot) {
                // Attach as child sub-point to active root prompt
                currentRoot.subagents.push(node);
            } else {
                // If no root has occurred yet or gap > 2 hours, treat as standalone root
                rootNodes.push(node);
            }
        } else {
            // New User Prompt, System Message or Tool Output becomes new root
            rootNodes.push(node);
            if (classification.tier === PromptOriginTier.UserPrompt) {
                currentRoot = node;
            }
        }
    }

    // Reverse rootNodes so newest user prompt appears at top,
    // and sort running and queued roots to the top
    return rootNodes.reverse().sort(compareHierarchicalNodes);
}

export function normalizePromptForGrouping(conv: AgmConversationNode): string {
    const text = conv.prompt_preview_200w || conv.title || conv.short_id || conv.conversation_id;
    return text.trim().toLowerCase().replace(/\s+/g, ' ');
}

export function isAutomatedConversation(conv: AgmConversationNode): boolean {
    const preview = (conv.prompt_preview_200w || '').toLowerCase();
    const title = (conv.title || '').toLowerCase();
    const cat = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);

    const isSystemTier = cat.tier === PromptOriginTier.SystemMessage || cat.tier === PromptOriginTier.ToolOutput;
    if (isSystemTier) {
        return true;
    }
    const hasSystemMarker = preview.includes('<system_message>') || preview.includes('system_message');
    if (hasSystemMarker) {
        return true;
    }
    const hasHeartbeat = title.includes('heartbeat') || preview.includes('heartbeat');
    const isSystemTitle = title.startsWith('system:');
    if (hasHeartbeat || isSystemTitle) {
        return true;
    }
    const hasTelemetryPreview = preview.includes('token_usage') || preview.includes('telemetry');
    const hasTelemetryTitle = title.includes('telemetry');
    return hasTelemetryPreview || hasTelemetryTitle;
}
