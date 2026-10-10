import type { AgmConversationNode } from './types';

export function isGhostConversation(conv: AgmConversationNode): boolean {
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

// Helper to check if a conversation is stale or empty (exempt if actively running with content)
export function isStaleOrEmptyConversation(conv: AgmConversationNode): boolean {
    if (isGhostConversation(conv)) {
        return true;
    }
    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;
    // 0-word empty untitled conversations must NEVER be treated as running or active
    if (isEmptyPrompt) {
        return true;
    }
    if (Boolean(conv.is_running)) {
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
    return isUntitled;
}

export const isUntitledOrEmpty = isStaleOrEmptyConversation;
