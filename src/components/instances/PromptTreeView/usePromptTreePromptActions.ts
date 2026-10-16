import { useCallback } from 'react';
import {
    focusOrLaunchInstance,
    focusInstanceWorkspace,
    enqueuePrompt,
    sendPromptNow,
} from '../../../services/instanceService';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { persistClientCrash } from '../../../lib/error-listener';
import { auditRecord } from '../../audit/audit-client';
import type { AgmConversationNode } from './types';
import { isGhostConversation } from './conversationGuards';
import { detectPromptCategory } from './tierClassification';
import type { PromptTreeState } from './usePromptTreeState';

export async function copyPromptWithRichImages(
    promptText: string,
    onSuccess: (message: string) => void,
    onError: (err: unknown) => void
): Promise<boolean> {
    if (!promptText || !promptText.trim()) {
        return false;
    }

    try {
        // 1. Synthesize rich HTML payload converting Markdown images to styled HTML <img> tags
        const htmlContent = promptText
            .replace(/!\[(.*?)\]\((.*?)\)/g, '<img src="$2" alt="$1" style="max-width:100%; border-radius: 8px; margin: 8px 0;" /><br />')
            .replace(/\n/g, '<br />');

        const blobHtml = new Blob([htmlContent], { type: 'text/html' });
        const blobText = new Blob([promptText], { type: 'text/plain' });

        const item = new ClipboardItem({
            'text/html': blobHtml,
            'text/plain': blobText,
        });

        await navigator.clipboard.write([item]);
        onSuccess('Copied full prompt with embedded rich images to clipboard!');
        return true;
    } catch (err) {
        // Fallback ladder adhering to "No Silent Errors" rule
        useErrorStore.getState().trackWarning(err, {
            source: 'clipboard.copyPromptWithRichImages',
            triggerAction: 'navigator.clipboard.write',
        });

        const fallbackOk = await copyToClipboard(promptText);
        if (fallbackOk) {
            onSuccess('Copied clean prompt text to clipboard (rich media unsupported in current context)');
            return true;
        }

        onError(err);
        return false;
    }
}

export type PromptAiCopyKind = 'json' | 'text' | 'md';

export function resolvePromptBody(conv: AgmConversationNode, preferred?: string): string {
    const preferredBody = preferred?.trim();
    if (preferredBody) return preferredBody;
    return (conv.full_prompt_text || conv.prompt_preview_200w || '').trim();
}

/** JSON copy is only the AI fields already stored on the conversation. */
export function buildAiResultsJson(conv: AgmConversationNode): string {
    return JSON.stringify(
        {
            latest_response: conv.latest_response ?? '',
            execution_results: conv.execution_results ?? '',
        },
        null,
        2
    );
}

export function buildPromptAndAiPlainText(promptBody: string, conv: AgmConversationNode): string {
    const sections = [`Prompt\n${promptBody}`, `AI Response\n${conv.latest_response ?? ''}`];
    if (conv.execution_results) {
        sections.push(`Execution Results\n${conv.execution_results}`);
    }
    return sections.join('\n\n');
}

export function buildPromptAndAiMarkdown(promptBody: string, conv: AgmConversationNode): string {
    const sections = [`# Prompt\n\n${promptBody}`, `# AI Response\n\n${conv.latest_response ?? ''}`];
    if (conv.execution_results) {
        sections.push(`# Execution Results\n\n${conv.execution_results}`);
    }
    return sections.join('\n\n');
}

export function buildPromptAiCopyPayload(
    kind: PromptAiCopyKind,
    conv: AgmConversationNode,
    promptBody: string
): string {
    if (kind === 'json') return buildAiResultsJson(conv);
    if (kind === 'text') return buildPromptAndAiPlainText(promptBody, conv);
    return buildPromptAndAiMarkdown(promptBody, conv);
}

export async function copyPromptAiPayload(
    kind: PromptAiCopyKind,
    conv: AgmConversationNode,
    promptBody: string,
    source: string
): Promise<boolean> {
    try {
        const ok = await copyToClipboard(buildPromptAiCopyPayload(kind, conv, promptBody));
        if (!ok) {
            throw new Error('clipboard copy returned false');
        }
        return true;
    } catch (err) {
        const errorObj = err instanceof Error ? err : new Error(String(err));
        useErrorStore.getState().captureException(errorObj, {
            source,
            triggerAction: `copy_prompt_ai_${kind}`,
        });
        return false;
    }
}

export interface PromptTreePromptActions {
    handleCopyAllWithRichImages: () => Promise<void>;
    handleCopyWithImagesRich: () => Promise<void>;
    handleCopyAiResultsJson: () => Promise<void>;
    handleCopyPromptAndAiText: () => Promise<void>;
    handleCopyPromptAndAiMarkdown: () => Promise<void>;
    handleSaveImages: () => void;
    handleResendPrompt: () => Promise<void>;
    handleExport: (format: 'md' | 'json') => void;
    handleEnqueuePrompt: () => Promise<void>;
    handleFocusIde: () => Promise<void>;
    extractAssets: (text: string) => { images: string[]; files: string[] };
    openInspector: (conv: AgmConversationNode, repoPath: string) => void;
}

interface UsePromptTreePromptActionsProps {
    state: PromptTreeState;
    instanceId: string;
    selectConversation: (conv: AgmConversationNode, project: import('./types').AgmProjectTreeNode) => void;
}

const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'svg'] as const;

export function usePromptTreePromptActions({
    state,
    instanceId,
    selectConversation,
}: UsePromptTreePromptActionsProps): PromptTreePromptActions {
    const {
        treeData,
        setError,
        setActionMsg,
        archivedProjectIds,
        selectedProject,
        setSelectedProject,
        selectedConversation,
        activePromptText,
        editedPromptText,
        setIsCopiedAll,
        setIsResending,
        setIsEnqueueing,
        setIsFocusing,
        confirmationSuffix,
        conversationTargetMode,
        newConversationTitle,
        selectedConversationRef,
        selectedProjectRef,
        activePromptTextRef,
        setShowAllWords,
        setInspectorPrompt,
    } = state;

    const extractAssets = useCallback((text: string) => {
        const images: string[] = [];
        const files: string[] = [];

        const imgRegex = /!\[(.*?)\]\((https?:\/\/.*?|data:image\/.*?;base64,.*?)\)/g;
        let match;
        while ((match = imgRegex.exec(text)) !== null) {
            images.push(match[1]);
        }

        const fileRegex = /(?:file:\/\/\/|[a-zA-Z]:\\|\/)[^\s)"]+/g;
        let fileMatch;
        while ((fileMatch = fileRegex.exec(text)) !== null) {
            if (!fileMatch[0].match(/\.(png|jpg|jpeg|gif|webp|svg)$/i)) {
                files.push(fileMatch[0]);
            } else {
                images.push(fileMatch[0]);
            }
        }

        return { images, files };
    }, []);

    const openInspector = useCallback(
        (conv: AgmConversationNode, repoPath: string) => {
            const text = activePromptText || conv.prompt_preview_200w || 'No prompt content preview available for this conversation.';
            const { images, files } = extractAssets(text);
            setShowAllWords(true);
            setInspectorPrompt({
                title: conv.title || conv.short_id,
                text,
                convId: conv.conversation_id,
                projectPath: repoPath,
                images,
                files,
            });
        },
        [activePromptText, extractAssets, setShowAllWords, setInspectorPrompt]
    );

    const handleCopyAllWithRichImages = useCallback(async () => {
        const text = editedPromptText.trim() || activePromptTextRef.current || activePromptText || selectedConversation?.prompt_preview_200w || '';
        if (!text) {
            setActionMsg('No prompt content to copy');
            setTimeout(() => setActionMsg(null), 2000);
            return;
        }

        const ok = await copyPromptWithRichImages(
            text,
            (msg) => {
                setIsCopiedAll(true);
                setActionMsg(msg);
                setTimeout(() => {
                    setIsCopiedAll(false);
                    setActionMsg(null);
                }, 2000);
            },
            (err) => {
                useErrorStore.getState().captureException(
                    err instanceof Error ? err : new Error(String(err)),
                    {
                        source: 'usePromptTreePromptActions.handleCopyAllWithRichImages',
                        triggerAction: 'copyPromptWithRichImages',
                    }
                );
                setError('Failed to copy prompt to clipboard');
            }
        );
        if (ok) {
            setIsCopiedAll(true);
            setTimeout(() => setIsCopiedAll(false), 2000);
        }
    }, [editedPromptText, activePromptText, selectedConversation, activePromptTextRef, setIsCopiedAll, setActionMsg, setError]);

    const handleCopyWithImagesRich = useCallback(async () => {
        await handleCopyAllWithRichImages();
    }, [handleCopyAllWithRichImages]);

    const copySelectedPromptAi = useCallback(async (kind: PromptAiCopyKind) => {
        const conv = selectedConversationRef.current || selectedConversation;
        if (!conv) {
            setActionMsg('No prompt selected to copy');
            setTimeout(() => setActionMsg(null), 2000);
            return;
        }
        const promptBody = resolvePromptBody(conv, editedPromptText.trim() || activePromptTextRef.current || activePromptText);
        const ok = await copyPromptAiPayload(
            kind,
            conv,
            promptBody,
            `usePromptTreePromptActions.copyPromptAi.${kind}`
        );
        if (!ok) {
            setError('Failed to copy prompt and AI results');
            return;
        }
        const label = kind === 'json'
            ? 'Copied AI results as JSON'
            : kind === 'text'
                ? 'Copied prompt and AI results as text'
                : 'Copied prompt and AI results as markdown';
        setActionMsg(label);
        setTimeout(() => setActionMsg(null), 2000);
    }, [
        selectedConversation, selectedConversationRef, editedPromptText, activePromptText,
        activePromptTextRef, setActionMsg, setError,
    ]);

    const handleCopyAiResultsJson = useCallback(async () => {
        await copySelectedPromptAi('json');
    }, [copySelectedPromptAi]);

    const handleCopyPromptAndAiText = useCallback(async () => {
        await copySelectedPromptAi('text');
    }, [copySelectedPromptAi]);

    const handleCopyPromptAndAiMarkdown = useCallback(async () => {
        await copySelectedPromptAi('md');
    }, [copySelectedPromptAi]);

    const handleSaveImages = useCallback(() => {
        const text = activePromptText || selectedConversation?.prompt_preview_200w || '';
        const { images } = extractAssets(text);
        if (images.length === 0) {
            setActionMsg('No embedded images found in this prompt.');
            setTimeout(() => setActionMsg(null), 2500);
            return;
        }

        images.forEach((imgUrl, index) => {
            const a = document.createElement('a');
            a.href = imgUrl;
            let ext = 'png';
            if (imgUrl.includes('image/jpeg') || imgUrl.endsWith('.jpg') || imgUrl.endsWith('.jpeg')) {
                ext = 'jpg';
            } else if (imgUrl.includes('image/webp') || imgUrl.endsWith('.webp')) {
                ext = 'webp';
            } else if (imgUrl.includes('image/svg') || imgUrl.endsWith('.svg')) {
                ext = 'svg';
            }
            a.download = `prompt-image-${selectedConversation?.short_id || 'img'}-${index + 1}.${ext}`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
        });

        setActionMsg(`Saved ${images.length} image(s) to downloads!`);
        setTimeout(() => setActionMsg(null), 3000);
    }, [activePromptText, selectedConversation, extractAssets, setActionMsg]);

    const handleResendPrompt = useCallback(async () => {
        let conv = selectedConversationRef.current || selectedConversation;
        let proj = selectedProjectRef.current || selectedProject;

        // Fallback 1: If selectedConversation is null, find most recent non-empty conversation in selectedProject
        if (!conv && proj && proj.conversations.length > 0) {
            const candidates = proj.conversations.filter(
                (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            if (candidates.length > 0) {
                const sorted = [...candidates].sort((a, b) => {
                    const aTime = new Date(a.last_modified).getTime() || 0;
                    const bTime = new Date(b.last_modified).getTime() || 0;
                    return bTime - aTime;
                });
                conv = sorted[0];
                selectConversation(conv, proj);
            }
        }

        // Fallback 2: If selectedProject is null but selectedConversation exists, resolve selectedProject from tree nodes
        if (!proj && conv) {
            for (const p of treeData) {
                if (p.conversations.some((c) => c.conversation_id === conv?.conversation_id)) {
                    proj = p;
                    setSelectedProject(p);
                    break;
                }
            }
        }

        // Fallback 3: If both conv and proj are null, pick newest valid conversation across non-archived projects
        if (!conv && !proj && treeData.length > 0) {
            for (const p of treeData) {
                if (archivedProjectIds.includes(p.project_id)) continue;
                const candidates = p.conversations.filter(
                    (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                );
                if (candidates.length > 0) {
                    proj = p;
                    const sorted = [...candidates].sort((a, b) => {
                        const aTime = new Date(a.last_modified).getTime() || 0;
                        const bTime = new Date(b.last_modified).getTime() || 0;
                        return bTime - aTime;
                    });
                    conv = sorted[0];
                    selectConversation(conv, proj);
                    break;
                }
            }
        }

        if (!conv && !proj) {
            // Visible no-op guard: previously a silent return that made the
            // Send button appear dead with no feedback.
            useErrorStore.getState().trackWarning('Send: no conversation or project resolved', {
                source: 'PromptTreeViewModal.handleResendPrompt',
                triggerAction: 'resolve_send_target',
            });
            setError('Send failed: no conversation or project could be resolved — select a prompt first.');
            return;
        }

        try {
            setIsResending(true);
            setActionMsg('Dispatching prompt to running instance...');
            let promptContent = (!selectedConversation && conv)
                ? (conv.prompt_preview_200w || '')
                : (editedPromptText.trim() || activePromptTextRef.current || activePromptText || conv?.prompt_preview_200w || '');
            if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)') {
                promptContent = `${promptContent}\n\n${confirmationSuffix}`;
            }
            const repoPath = proj?.repo_path || '';

            // 1. Dispatch prompt directly to running instance via sendPromptNow.
            // Track real dispatch outcome: the backend returns Err when the agy
            // CLI spawn fails, and the success message must not show in that case.
            const targetInstId = proj?.instance_id || instanceId || 'default';
            let dispatchOk = false;
            let dispatchErr: unknown = null;
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : conv?.conversation_id;
            try {
                await sendPromptNow(targetInstId, repoPath, promptContent, targetConvId);
                dispatchOk = true;
            } catch (sendErr) {
                dispatchErr = sendErr;
                persistClientCrash(sendErr, 'PromptTreeViewModal.handleSend');
            }

            // 2. Copy prompt content to clipboard so user can paste immediately (manual fallback).
            // Use the robust helper: copyToClipboard throws
            // NotAllowedError when the document is not focused (e.g. the IDE
            // window stole focus during dispatch) — the execCommand fallback
            // covers that case instead of just logging a warning.
            try {
                const copied = await copyToClipboard(promptContent);
                if (!copied) {
                    throw new Error('clipboard copy returned false');
                }
            } catch (clipErr) {
                // Tracked in the error module; send dispatch already attempted above, user pastes manually if needed.
                useErrorStore.getState().trackWarning(clipErr, {
                    source: 'PromptTreeViewModal.handleSend',
                    triggerAction: 'copy_prompt_fallback',
                });
            }

            if (dispatchOk) {
                setActionMsg("Prompt dispatched to IDE & copied to clipboard!");
                setTimeout(() => setActionMsg(null), 3000);
            } else {
                const errorObj = dispatchErr instanceof Error ? dispatchErr : new Error(String(dispatchErr) || 'dispatch rejected');
                setError(`Send failed: ${errorObj.message} — prompt copied to clipboard; paste it into the IDE manually.`);
            }
        } catch (err: unknown) {
            const errorObj = err instanceof Error ? err : new Error(String(err) || 'Failed to dispatch prompt');
            useErrorStore.getState().captureException(errorObj, {
                source: 'PromptTreeViewModal.handleResendPrompt',
                triggerAction: 'resend_prompt',
            });
            setError(`Send failed: ${errorObj.message}`);
        } finally {
            setIsResending(false);
        }
    }, [
        selectedConversation, selectedProject, treeData, archivedProjectIds,
        editedPromptText, activePromptText, confirmationSuffix, conversationTargetMode, newConversationTitle, instanceId,
        selectConversation, selectedConversationRef, selectedProjectRef,
        activePromptTextRef, setSelectedProject, setActionMsg, setError, setIsResending,
    ]);

    const handleExport = useCallback(
        (format: 'md' | 'json') => {
            if (!selectedConversation) return;
            const baseName = (selectedConversation.title || selectedConversation.short_id || 'prompt')
                .replace(/[^a-zA-Z0-9_\-\s]/g, '')
                .trim()
                .replace(/\s+/g, '_')
                .slice(0, 40) || 'prompt';
            const categoryInfo = detectPromptCategory(activePromptText, selectedConversation.title);

            if (format === 'md') {
                const mdContent = `---
title: "${selectedConversation.title || 'Untitled Prompt'}"
project: "${selectedProject?.repo_name || 'Unknown'}"
sequence_code: "${selectedConversation.seq_code || 'P001'}"
category: "${categoryInfo.label}"
exported_at: "${new Date().toISOString()}"
---

# ${selectedConversation.title || 'Prompt Instruction'}

${activePromptText}
`;
                const blob = new Blob([mdContent], { type: 'text/markdown;charset=utf-8' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `${baseName}.md`;
                a.click();
                URL.revokeObjectURL(url);
                setActionMsg(`Exported prompt as Markdown (${baseName}.md)`);
                setTimeout(() => setActionMsg(null), 2500);
            } else {
                const jsonContent = JSON.stringify({
                    title: selectedConversation.title || 'Untitled Prompt',
                    conversation_id: selectedConversation.conversation_id,
                    project_id: selectedProject?.project_id,
                    project_name: selectedProject?.repo_name,
                    seq_code: selectedConversation.seq_code,
                    category: categoryInfo.category,
                    category_label: categoryInfo.label,
                    step_count: selectedConversation.step_count,
                    exported_at: new Date().toISOString(),
                    prompt_text: activePromptText,
                }, null, 2);
                const blob = new Blob([jsonContent], { type: 'application/json;charset=utf-8' });
                const url = URL.createObjectURL(blob);
                const a = document.createElement('a');
                a.href = url;
                a.download = `${baseName}.json`;
                a.click();
                URL.revokeObjectURL(url);
                setActionMsg(`Exported prompt as JSON (${baseName}.json)`);
                setTimeout(() => setActionMsg(null), 2500);
            }
        },
        [selectedConversation, selectedProject, activePromptText, setActionMsg]
    );

    const handleEnqueuePrompt = useCallback(async () => {
        if (!selectedConversation) {
            // Visible no-op guard: previously a silent return that made the
            // Queue button appear dead with no feedback.
            useErrorStore.getState().trackWarning('Queue: no conversation selected', {
                source: 'PromptTreeViewModal.handleEnqueuePrompt',
                triggerAction: 'resolve_queue_target',
            });
            setError('Queue failed: no conversation selected — select a prompt first.');
            return;
        }
        try {
            setIsEnqueueing(true);
            setActionMsg('Enqueueing prompt into FIFO scheduler queue...');
            const promptContent = editedPromptText.trim() || activePromptText || selectedConversation.prompt_preview_200w || '';
            const repoPath = selectedProject?.repo_path || '';
            const targetInstId = selectedProject?.instance_id || instanceId || 'default';
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : selectedConversation.conversation_id;
            await enqueuePrompt(targetInstId, repoPath, promptContent, targetConvId);

            setActionMsg('Prompt enqueued into FIFO scheduler queue!');
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: unknown) {
            const errorObj = err instanceof Error ? err : new Error(String(err) || 'Failed to enqueue prompt');
            persistClientCrash(errorObj, 'PromptTreeViewModal.handleEnqueuePrompt');
            useErrorStore.getState().captureException(errorObj, {
                source: 'PromptTreeViewModal.handleEnqueuePrompt',
                triggerAction: 'enqueue_prompt',
            });
            setError(`Queue failed: ${errorObj.message}`);
        } finally {
            setIsEnqueueing(false);
        }
    }, [
        selectedConversation, selectedProject, editedPromptText, activePromptText,
        conversationTargetMode, newConversationTitle, instanceId, setIsEnqueueing, setActionMsg, setError,
    ]);

    const handleFocusIde = useCallback(async () => {
        try {
            setIsFocusing(true);
            const targetId = selectedProject?.instance_id || instanceId || 'default';
            const repoPath = selectedProject?.repo_path || '';
            const repoName = selectedProject?.repo_name || '';
            setActionMsg(`Focusing IDE workspace '${repoName || targetId}'...`);
            const wasFocused = await focusInstanceWorkspace(targetId, repoPath, repoName);
            if (wasFocused) {
                setActionMsg(`Workspace window '${repoName}' focused!`);
            } else {
                const launched = await focusOrLaunchInstance(targetId, repoPath);
                setActionMsg(launched ? 'Antigravity IDE window focused!' : 'Antigravity IDE launched!');
            }
            setTimeout(() => setActionMsg(null), 3000);
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : String(err) || 'Failed to focus IDE');
        } finally {
            setIsFocusing(false);
        }
    }, [selectedProject, instanceId, setIsFocusing, setActionMsg, setError]);

    return {
        handleCopyAllWithRichImages,
        handleCopyWithImagesRich,
        handleCopyAiResultsJson,
        handleCopyPromptAndAiText,
        handleCopyPromptAndAiMarkdown,
        handleSaveImages,
        handleResendPrompt,
        handleExport,
        handleEnqueuePrompt,
        handleFocusIde,
        extractAssets,
        openInspector,
    };
}

export { IMAGE_EXTENSIONS };

// ============================================================================
// RERUN SECTION (Task-04) — appended at END of file per task ownership.
// ----------------------------------------------------------------------------
// Rerun support: dispatches a follow-up nudge message N times through the SAME
// canonical dispatch path as handleResendPrompt (`sendPromptNow`) — no forked
// logic, no new backend endpoint. Each individual rerun is audit-logged via
// `auditRecord`; dispatch failures are routed into the error module (via
// sendPromptNow's captureError + persistClientCrash) and recorded in the audit
// entry as `result: 'error'`, so failures are always visible — no silent errors.
// ============================================================================

/** Message choice offered by the rerun UI picker. */
export type RerunMessageOption = 'is-it-done' | 'is-it-published' | 'randomized';

/** Picker rows rendered by the rerun UI. The randomized row is explicitly labeled. */
export const RERUN_MESSAGE_OPTIONS: ReadonlyArray<{ value: RerunMessageOption; label: string; hint: string }> = [
    { value: 'is-it-done', label: 'Is it done?', hint: 'Send a completion-check nudge' },
    { value: 'is-it-published', label: 'Is it published?', hint: 'Send a publish-status nudge' },
    {
        value: 'randomized',
        label: 'Randomized nudge (random pick)',
        hint: 'Every dispatch picks ONE message at random from the built-in set',
    },
];

/** Small built-in set of follow-up nudges used by the "randomized" option. */
export const RERUN_RANDOM_MESSAGE_SET: ReadonlyArray<string> = [
    'Quick status check — is it done yet?',
    'Please continue where you left off and confirm when finished.',
    'Give me a brief progress update.',
    'Are you finished? Summarize the result.',
    'Keep going — ping me when it is published.',
];

export const RERUN_FIXED_MESSAGES: Record<Exclude<RerunMessageOption, 'randomized'>, string> = {
    'is-it-done': 'Is it done?',
    'is-it-published': 'Is it published?',
};

export const RERUN_DEFAULT_COUNT = 2;
export const RERUN_MIN_COUNT = 1;
export const RERUN_MAX_COUNT = 5;

/** Resolve the actual message text for one rerun dispatch. */
export function resolveRerunMessage(option: RerunMessageOption): { message: string; pickedIndex: number | null } {
    if (option === 'randomized') {
        const pickedIndex = Math.floor(Math.random() * RERUN_RANDOM_MESSAGE_SET.length);
        return { message: RERUN_RANDOM_MESSAGE_SET[pickedIndex], pickedIndex };
    }
    return { message: RERUN_FIXED_MESSAGES[option], pickedIndex: null };
}

export interface RerunDispatchContext {
    /** Instance id the nudge is sent to (same resolution as handleResendPrompt). */
    targetInstId: string;
    /** Repo path of the target instance ('' falls back to instance default). */
    repoPath: string;
    /** Conversation the nudge lands in (undefined = instance default). */
    convId?: string;
    /** How many times the message is dispatched (clamped to 1..5). */
    count: number;
    messageOption: RerunMessageOption;
    /** Audit/source label, e.g. 'DetailHeader.rerunSection'. */
    source?: string;
}

export interface RerunDispatchResult {
    requested: number;
    dispatched: number;
    failed: number;
}

/**
 * Dispatch the chosen rerun message N times through the canonical `sendPromptNow`
 * path (the same function handleResendPrompt uses). Each rerun is audit-logged
 * individually; a failed rerun is audit-logged as `result: 'error'` and routed
 * into the error module — failures are never silent.
 */
export async function dispatchRerunMessages(ctx: RerunDispatchContext): Promise<RerunDispatchResult> {
    const requested = Math.min(
        RERUN_MAX_COUNT,
        Math.max(RERUN_MIN_COUNT, Math.floor(ctx.count) || RERUN_DEFAULT_COUNT)
    );
    const source = ctx.source ?? 'DetailHeader.rerunSection';
    const result: RerunDispatchResult = { requested, dispatched: 0, failed: 0 };
    const auditBase = {
        actor: 'user' as const,
        methodName: 'prompt_tree_rerun_dispatch',
        uiTarget: 'PromptTreeView | DetailHeader | RerunSection',
        trigger: 'click' as const,
    };
    for (let i = 0; i < requested; i += 1) {
        const { message, pickedIndex } = resolveRerunMessage(ctx.messageOption);
        try {
            // Canonical dispatch path — identical call handleResendPrompt makes.
            await sendPromptNow(ctx.targetInstId, ctx.repoPath, message, ctx.convId);
            result.dispatched += 1;
            await auditRecord({
                ...auditBase,
                params: {
                    source,
                    targetInstId: ctx.targetInstId,
                    convId: ctx.convId ?? null,
                    messageOption: ctx.messageOption,
                    rerunIndex: i + 1,
                    rerunTotal: requested,
                    messagePickedIndex: pickedIndex,
                    message,
                },
                result: 'ok',
            });
        } catch (err) {
            result.failed += 1;
            const errorObj = err instanceof Error ? err : new Error(String(err) || 'rerun dispatch failed');
            // sendPromptNow already routed this into the error store (captureError);
            // persist the crash record as well so the failure stays visible.
            persistClientCrash(errorObj, 'usePromptTreePromptActions.dispatchRerunMessages');
            await auditRecord({
                ...auditBase,
                params: {
                    source,
                    targetInstId: ctx.targetInstId,
                    convId: ctx.convId ?? null,
                    messageOption: ctx.messageOption,
                    rerunIndex: i + 1,
                    rerunTotal: requested,
                    messagePickedIndex: pickedIndex,
                    message,
                },
                result: 'error',
                message: errorObj.message,
            });
        }
    }
    return result;
}
// END RERUN SECTION (Task-04)
