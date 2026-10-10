/**
 * Backward-compatible re-exports for PromptTreeViewModal.
 *
 * The implementation has been split into focused modules under
 * `./PromptTreeView/` (each ≤500 lines). This file preserves the original
 * public API so existing imports keep working.
 */
export { default } from './PromptTreeView/PromptTreeViewModal';
export type { PromptTreeViewModalProps } from './PromptTreeView/types';
export type {
    AgmConversationNode,
    AgmProjectTreeNode,
    PromptTier,
    TierClassificationResult,
    PromptCategory,
    PromptCategoryResult,
    HierarchicalConversationNode,
    ViewMode,
} from './PromptTreeView/types';
export type { TruncatedContextCalloutProps } from './PromptTreeView/TruncatedContextCallout';
export { TruncatedContextCallout } from './PromptTreeView/TruncatedContextCallout';
export {
    formatDualBadge,
    formatCleanSeqCode,
    formatSeqBadge,
} from './PromptTreeView/formatters';
export {
    isGhostConversation,
    isStaleOrEmptyConversation,
    isUntitledOrEmpty,
} from './PromptTreeView/conversationGuards';
export { copyPromptWithRichImages } from './PromptTreeView/clipboard';
export {
    classifyPromptTier,
    detectPromptCategory,
    assembleConversationHierarchy,
    normalizePromptForGrouping,
} from './PromptTreeView/tierClassification';
