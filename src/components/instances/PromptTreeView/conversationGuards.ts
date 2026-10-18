import type { AgmConversationNode } from './types';

export function isConversationOrTurnQueued(conv: AgmConversationNode): boolean {
    if (Boolean(conv.is_queued)) return true;
    if (conv.status && conv.status.toLowerCase().includes('queue')) return true;
    if (conv.prompts && conv.prompts.some((p) => Boolean(p.is_queued) || (p.status && p.status.toLowerCase().includes('queue')))) {
        return true;
    }
    return false;
}

export function isGhostConversation(conv: AgmConversationNode): boolean {
    const statusLower = (conv.status || '').toLowerCase();
    const isRunning = Boolean(conv.is_running) || statusLower.includes('run') || statusLower.includes('exec');
    const isQueued = isConversationOrTurnQueued(conv);
    if (isRunning || isQueued) {
        return false;
    }
    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();
    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;
    return isUntitled && isEmptyPrompt;
}

// Helper to check if a conversation is stale or empty (exempt if actively running or queued with content)
export function isStaleOrEmptyConversation(conv: AgmConversationNode): boolean {
    if (isGhostConversation(conv)) {
        return true;
    }
    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;

    // Running OR queued conversations must NEVER be treated as stale or empty
    const statusLower = (conv.status || '').toLowerCase();
    const isRunning = Boolean(conv.is_running) || statusLower.includes('run') || statusLower.includes('exec');
    const isQueued = isConversationOrTurnQueued(conv);
    if (isRunning || isQueued) {
        return false;
    }

    // 0-word empty untitled conversations must NEVER be treated as running or active
    if (isEmptyPrompt) {
        return true;
    }
    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();
    return isUntitled;
}

export const isUntitledOrEmpty = isStaleOrEmptyConversation;
