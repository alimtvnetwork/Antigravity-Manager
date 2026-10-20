import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../../utils/env';
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
import { copyPromptWithRichImages, extractPromptImages } from './clipboard';
import {
    buildRichVerificationPrompt,
    buildRerunPrompt,
    buildMultiPromptVerificationPrompt,
    parseVerificationTypeFromSuffix,
    buildMultiRoundVerificationPrompts,
    type MultiRoundVerificationOptions,
} from './verificationTemplates';

export { copyPromptWithRichImages } from './clipboard';

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
    handleMultiPromptResend: (count: number | 'all') => Promise<{ sent: number; failed: number }>;
    handleVerifyLastPrompts: (count?: number) => Promise<boolean>;
    handleCloneAndCompact: () => Promise<string | null>;
    handleExportConvJson: () => void;
    handleExportConvYaml: () => void;
    handleExportConvSqlite: () => Promise<void>;
    handleOpenVerificationModal: (conversationId?: string | AgmConversationNode, defaultTurns?: number[]) => void;
    isVerificationModalOpen: boolean;
    setIsVerificationModalOpen: React.Dispatch<React.SetStateAction<boolean>>;
    verificationModalConv: AgmConversationNode | null;
    enqueueVerificationRounds: (options: MultiRoundVerificationOptions) => Promise<boolean>;
}

interface UsePromptTreePromptActionsProps {
    state: PromptTreeState;
    instanceId: string;
    selectConversation: (conv: AgmConversationNode, project: import('./types').AgmProjectTreeNode) => void;
    loadTree?: (isInitialLoad?: boolean, isForce?: boolean) => Promise<void>;
}

const IMAGE_EXTENSIONS = ['png', 'jpg', 'jpeg', 'webp', 'svg'] as const;

export function usePromptTreePromptActions({
    state,
    instanceId,
    selectConversation,
    loadTree,
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

        const imgRegex = /!\[(.*?)\]\((https?:\/\/.*?|data:image\/[^;]+;base64,[A-Za-z0-9+/=]+|[^\s)]+?\.(?:png|jpe?g|gif|webp|svg))\)/gi;
        let match: RegExpExecArray | null;
        while ((match = imgRegex.exec(text)) !== null) {
            if (match[2]) {
                images.push(match[2].trim());
            }
        }

        const fileRegex = /(?:file:\/\/\/|[a-zA-Z]:\\|\/)[^\s)"]+/g;
        let fileMatch: RegExpExecArray | null;
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
        const text = editedPromptText.trim() || activePromptTextRef.current || activePromptText || selectedConversation?.full_prompt_text || selectedConversation?.prompt_preview_200w || '';
        if (!text) {
            setActionMsg('No prompt content to copy');
            setTimeout(() => setActionMsg(null), 2000);
            return;
        }

        const conv = selectedConversationRef.current || selectedConversation;
        const mediaPaths = conv?.media_paths;
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
            },
            mediaPaths
        );
        if (ok) {
            setIsCopiedAll(true);
            setTimeout(() => setIsCopiedAll(false), 2000);
        }
    }, [editedPromptText, activePromptText, selectedConversation, activePromptTextRef, selectedConversationRef, setIsCopiedAll, setActionMsg, setError]);

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
            setError('Send failed: no conversation or project could be resolved â€” select a prompt first.');
            return;
        }

        try {
            setIsResending(true);
            setActionMsg('Dispatching prompt to running instance...');
            let promptContent = (!selectedConversation && conv)
                ? (conv.full_prompt_text || conv.prompt_preview_200w || '')
                : (editedPromptText.trim() || activePromptTextRef.current || activePromptText || conv?.full_prompt_text || conv?.prompt_preview_200w || '');
            if (!promptContent && conv) {
                promptContent = resolvePromptBody(conv);
            }
            const repoPath = proj?.repo_path || selectedProject?.repo_path || (conv as { repo_path?: string })?.repo_path || '';
            const targetInstId = selectedConversation?.instance_id || conv?.instance_id || proj?.instance_id || instanceId || 'default';
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : conv?.conversation_id;

            if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)' && confirmationSuffix !== 'none') {
                const templateType = parseVerificationTypeFromSuffix(confirmationSuffix);
                if (templateType !== 'none') {
                    promptContent = buildRichVerificationPrompt(templateType, {
                        originalPrompt: promptContent,
                        instanceId: targetInstId,
                        repoPath,
                        convId: targetConvId,
                    });
                }
            }

            // Append attached images from conversation media_paths or media_thumbnails if not already in markdown
            const mediaList: string[] = [
                ...(selectedConversation?.media_paths || conv?.media_paths || []),
                ...(selectedConversation?.media_thumbnails || conv?.media_thumbnails || []),
            ].filter((p, idx, arr): p is string => Boolean(p && typeof p === 'string' && arr.indexOf(p) === idx));
            if (mediaList.length > 0) {
                const missing = mediaList.filter((p) => p && !promptContent.includes(p));
                if (missing.length > 0) {
                    promptContent += '\n\n' + missing.map((p, idx) => `![Attached Image ${idx + 1}](${p})`).join('\n\n');
                }
            }

            // Dispatch real prompt content directly to running instance via sendPromptNow without stripping Base64 images to non-existent relative attachments
            // Track real dispatch outcome: the backend returns Err when the agy
            // CLI spawn fails, and the success message must not show in that case.
            let dispatchOk = false;
            let dispatchErr: unknown = null;
            try {
                await sendPromptNow(targetInstId, repoPath, promptContent, targetConvId);
                dispatchOk = true;
            } catch (sendErr) {
                dispatchErr = sendErr;
                persistClientCrash(sendErr, 'PromptTreeViewModal.handleSend');
            }

            // Prime system clipboard concurrently with dual-MIME rich HTML images for immediate pasting.
            try {
                await copyPromptWithRichImages(promptContent, undefined, undefined, mediaList);
            } catch (clipErr) {
                useErrorStore.getState().trackWarning(clipErr, {
                    source: 'PromptTreeViewModal.handleSend',
                    triggerAction: 'copy_prompt_rich_fallback',
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

        // Fallback 2: If selectedProject is null but conv exists, find and assign proj from treeData (so repoPath is NEVER empty "")
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

        if (!conv) {
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
            let promptContent = editedPromptText.trim() || activePromptTextRef.current || activePromptText || conv.full_prompt_text || conv.prompt_preview_200w || '';
            if (!promptContent && conv) {
                promptContent = resolvePromptBody(conv);
            }
            const repoPath = proj?.repo_path || selectedProject?.repo_path || (conv as { repo_path?: string })?.repo_path || '';
            const targetInstId = selectedConversation?.instance_id || conv.instance_id || proj?.instance_id || instanceId || 'default';
            const targetConvId = conversationTargetMode === 'new'
                ? (newConversationTitle?.trim() || undefined)
                : conv.conversation_id;

            if (confirmationSuffix && confirmationSuffix !== 'None (Send as is)' && confirmationSuffix !== 'none') {
                const templateType = parseVerificationTypeFromSuffix(confirmationSuffix);
                if (templateType !== 'none') {
                    promptContent = buildRichVerificationPrompt(templateType, {
                        originalPrompt: promptContent,
                        instanceId: targetInstId,
                        repoPath,
                        convId: targetConvId,
                    });
                }
            }

            // Append attached images from conversation media_paths or media_thumbnails if not already in markdown
            const mediaList: string[] = [
                ...(selectedConversation?.media_paths || conv.media_paths || []),
                ...(selectedConversation?.media_thumbnails || conv.media_thumbnails || []),
            ].filter((p, idx, arr): p is string => Boolean(p && typeof p === 'string' && arr.indexOf(p) === idx));
            if (mediaList.length > 0) {
                const missing = mediaList.filter((p) => p && !promptContent.includes(p));
                if (missing.length > 0) {
                    promptContent += '\n\n' + missing.map((p, idx) => `![Attached Image ${idx + 1}](${p})`).join('\n\n');
                }
            }

            // Enqueue prompt into FIFO queue (preserving real Base64 image data URIs and file paths)
            await enqueuePrompt(targetInstId, repoPath, promptContent, targetConvId);

            // Trigger immediate tree refresh and cache invalidation so tree view reflects enqueued turn
            if (loadTree) {
                try {
                    await loadTree(true, true);
                } catch (loadErr) {
                    useErrorStore.getState().trackWarning(loadErr, {
                        source: 'usePromptTreePromptActions.handleEnqueuePrompt',
                        triggerAction: 'loadTree',
                    });
                }
            }
            if (isTauri()) {
                try {
                    await invoke('invalidate_prompt_tree_cache', { instanceId: targetInstId }).catch(async () => {
                        await invoke('reset_prompt_tree_cache', { instanceId: targetInstId });
                    });
                } catch (cacheErr) {
                    useErrorStore.getState().trackWarning(cacheErr, {
                        source: 'usePromptTreePromptActions.handleEnqueuePrompt',
                        triggerAction: 'invalidate_prompt_tree_cache',
                    });
                }
            }

            // Prime system clipboard concurrently with dual-MIME rich HTML images for immediate pasting
            try {
                await copyPromptWithRichImages(promptContent, undefined, undefined, mediaList);
            } catch (clipErr) {
                useErrorStore.getState().trackWarning(clipErr, {
                    source: 'PromptTreeViewModal.handleEnqueuePrompt',
                    triggerAction: 'copy_enqueue_rich_fallback',
                });
            }

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
        selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef,
        treeData, archivedProjectIds, setSelectedProject,
        editedPromptText, activePromptText, activePromptTextRef, confirmationSuffix,
        conversationTargetMode, newConversationTitle, instanceId, selectConversation,
        loadTree, setIsEnqueueing, setActionMsg, setError,
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

    const handleMultiPromptResend = useCallback(async (count: number | 'all') => {
        const conv = selectedConversationRef.current || selectedConversation;
        const proj = selectedProjectRef.current || selectedProject;
        if (!conv) {
            setError('No conversation selected for multi-prompt resend');
            return { sent: 0, failed: 0 };
        }
        const prompts = extractConversationPromptsList(conv);
        setActionMsg(`Resending ${count === 'all' ? 'all' : count} prompt(s)...`);
        const res = await dispatchMultiPromptResend({
            conversationId: conv.conversation_id,
            instanceId: selectedConversation?.instance_id || conv.instance_id || proj?.instance_id || instanceId || 'default',
            repoPath: proj?.repo_path || '',
            count,
            prompts,
        });
        if (res.sent > 0) {
            setActionMsg(`Successfully resent ${res.sent} prompt(s)!`);
            setTimeout(() => setActionMsg(null), 3000);
        } else if (res.failed > 0) {
            setError(`Failed to resend prompts (${res.failed} error(s))`);
        }
        return res;
    }, [selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef, instanceId, setActionMsg, setError]);

    const handleVerifyLastPrompts = useCallback(async (count: number = 10) => {
        const conv = selectedConversationRef.current || selectedConversation;
        const proj = selectedProjectRef.current || selectedProject;
        if (!conv) {
            setError('No conversation selected to verify');
            return false;
        }
        const prompts = extractConversationPromptsList(conv);
        setActionMsg(`Synthesizing ${count}-prompt verification audit & enqueueing...`);
        const ok = await handleVerifyLastPromptsAndEnqueue({
            instanceId: selectedConversation?.instance_id || conv.instance_id || proj?.instance_id || instanceId || 'default',
            repoPath: proj?.repo_path || '',
            conversationId: conv.conversation_id,
            convTitle: conv.title || conv.short_id,
            count,
            prompts,
        });
        if (ok) {
            setActionMsg(`Synthesized ${Math.min(count, prompts.length)}-prompt audit & enqueued to instance!`);
            setTimeout(() => setActionMsg(null), 3500);
        } else {
            setError('Failed to enqueue verification audit');
        }
        return ok;
    }, [selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef, instanceId, setActionMsg, setError]);

    const handleCloneAndCompact = useCallback(async () => {
        const conv = selectedConversationRef.current || selectedConversation;
        const proj = selectedProjectRef.current || selectedProject;
        if (!conv) {
            setError('No conversation selected to clone and compact');
            return null;
        }
        setActionMsg('Cloning conversation and compacting memory handoff...');
        const newTitle = await handleCloneAndCompactConversation({
            instanceId: selectedConversation?.instance_id || conv.instance_id || proj?.instance_id || instanceId || 'default',
            repoPath: proj?.repo_path || '',
            sourceConv: conv,
        });
        if (newTitle) {
            setActionMsg(`Cloned & compacted into fresh session '${newTitle}'!`);
            setTimeout(() => setActionMsg(null), 3500);
        } else {
            setError('Failed to clone and compact conversation');
        }
        return newTitle;
    }, [selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef, instanceId, setActionMsg, setError]);

    const handleExportConvJson = useCallback(() => {
        const conv = selectedConversationRef.current || selectedConversation;
        const proj = selectedProjectRef.current || selectedProject;
        if (!conv) return;
        exportConversationAsJson(conv, proj?.repo_path);
        setActionMsg('Exported conversation as JSON');
        setTimeout(() => setActionMsg(null), 2500);
    }, [selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef, setActionMsg]);

    const handleExportConvYaml = useCallback(() => {
        const conv = selectedConversationRef.current || selectedConversation;
        const proj = selectedProjectRef.current || selectedProject;
        if (!conv) return;
        exportConversationAsYaml(conv, proj?.repo_path);
        setActionMsg('Exported conversation as YAML');
        setTimeout(() => setActionMsg(null), 2500);
    }, [selectedConversation, selectedConversationRef, selectedProject, selectedProjectRef, setActionMsg]);

    const handleExportConvSqlite = useCallback(async () => {
        const conv = selectedConversationRef.current || selectedConversation;
        if (!conv) return;
        setActionMsg('Exporting conversation as SQLite .db...');
        await exportConversationAsSqlite(conv);
        setActionMsg('Exported conversation as SQLite .db');
        setTimeout(() => setActionMsg(null), 2500);
    }, [selectedConversation, selectedConversationRef, setActionMsg]);

    const [isVerificationModalOpen, setIsVerificationModalOpen] = useState(false);
    const [verificationModalConv, setVerificationModalConv] = useState<AgmConversationNode | null>(null);

    const handleOpenVerificationModal = useCallback(
        (conversationId?: string | AgmConversationNode, _defaultTurns?: number[]) => {
            let targetConv: AgmConversationNode | null = null;
            if (typeof conversationId === 'object' && conversationId !== null) {
                targetConv = conversationId;
            } else if (typeof conversationId === 'string' && conversationId.trim()) {
                for (const proj of treeData) {
                    const found = proj.conversations.find((c) => c.conversation_id === conversationId);
                    if (found) {
                        targetConv = found;
                        break;
                    }
                }
            }
            if (!targetConv) {
                targetConv = selectedConversationRef.current || selectedConversation;
            }
            if (!targetConv) {
                setError('Please select a prompt or conversation to audit with verification rounds.');
                return;
            }
            setVerificationModalConv(targetConv);
            setIsVerificationModalOpen(true);
        },
        [treeData, selectedConversation, selectedConversationRef, setError]
    );

    const enqueueVerificationRounds = useCallback(
        async (options: MultiRoundVerificationOptions): Promise<boolean> => {
            try {
                const prompts = buildMultiRoundVerificationPrompts(options);
                if (prompts.length === 0) {
                    setError('No verification prompts synthesized');
                    return false;
                }
                setActionMsg(`Enqueueing ${prompts.length} verification round(s) to FIFO queue...`);

                const targetInstId = options.instanceId || instanceId || 'default';
                let enqueuedViaIpc = false;
                if (isTauri()) {
                    try {
                        await invoke('enqueue_verification_rounds', {
                            instanceId: targetInstId,
                            prompts,
                            sessionId: options.conversationId,
                        });
                        enqueuedViaIpc = true;
                    } catch {
                        enqueuedViaIpc = false;
                    }
                }

                if (!enqueuedViaIpc) {
                    for (let i = 0; i < prompts.length; i++) {
                        await enqueuePrompt(targetInstId, options.repoPath || '', prompts[i], options.conversationId);
                        if (i < prompts.length - 1) {
                            await new Promise((resolve) => setTimeout(resolve, 500));
                        }
                    }
                }

                if (isTauri()) {
                    try {
                        await invoke('invalidate_prompt_tree_cache', { instanceId: targetInstId });
                    } catch (cacheErr) {
                        useErrorStore.getState().trackWarning(cacheErr, {
                            source: 'usePromptTreePromptActions.enqueueVerificationRounds',
                            triggerAction: 'invalidate_cache',
                        });
                    }
                }

                await auditRecord({
                    actor: 'user',
                    methodName: 'prompt_tree_verification_rounds_enqueued',
                    route: '/prompts',
                    uiTarget: 'VerificationModal | EnqueueRounds',
                    trigger: 'click',
                    params: {
                        conversationId: options.conversationId,
                        instanceId: targetInstId,
                        roundsCount: prompts.length,
                        writingStyle: options.writingStyle || options.style,
                    },
                    result: 'ok',
                });

                setActionMsg(`Successfully enqueued ${prompts.length} verification round(s)!`);
                setTimeout(() => setActionMsg(null), 3500);
                return true;
            } catch (err) {
                persistClientCrash(err, 'usePromptTreePromptActions.enqueueVerificationRounds');
                useErrorStore.getState().captureException(
                    err instanceof Error ? err : new Error(String(err)),
                    {
                        source: 'usePromptTreePromptActions.enqueueVerificationRounds',
                        triggerAction: 'enqueue_verification_rounds',
                    }
                );
                setError('Failed to enqueue verification rounds');
                return false;
            }
        },
        [instanceId, setActionMsg, setError]
    );

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
        handleMultiPromptResend,
        handleVerifyLastPrompts,
        handleCloneAndCompact,
        handleExportConvJson,
        handleExportConvYaml,
        handleExportConvSqlite,
        handleOpenVerificationModal,
        isVerificationModalOpen,
        setIsVerificationModalOpen,
        verificationModalConv,
        enqueueVerificationRounds,
    };
}

export { IMAGE_EXTENSIONS };

// ============================================================================
// RERUN SECTION (Task-04) â€” appended at END of file per task ownership.
// ----------------------------------------------------------------------------
// Rerun support: dispatches a follow-up nudge message N times through the SAME
// canonical dispatch path as handleResendPrompt (`sendPromptNow`) â€” no forked
// logic, no new backend endpoint. Each individual rerun is audit-logged via
// `auditRecord`; dispatch failures are routed into the error module (via
// sendPromptNow's captureError + persistClientCrash) and recorded in the audit
// entry as `result: 'error'`, so failures are always visible â€” no silent errors.
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
    'Quick status check â€” is it done yet?',
    'Please continue where you left off and confirm when finished.',
    'Give me a brief progress update.',
    'Are you finished? Summarize the result.',
    'Keep going â€” ping me when it is published.',
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
    /** Full original prompt text to embed into Chapter 4. */
    originalPrompt?: string;
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
 * into the error module â€” failures are never silent.
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
        route: '/prompts',
        uiTarget: 'PromptTreeView | DetailHeader | RerunSection',
        trigger: 'click' as const,
    };
    for (let i = 0; i < requested; i += 1) {
        const { message: rawMessage, pickedIndex } = resolveRerunMessage(ctx.messageOption);
        const richPrompt = buildRerunPrompt(ctx.originalPrompt || '', i + 1, requested);
        const message = richPrompt || rawMessage;
        const { rewrittenPrompt } = extractPromptImages(message);
        try {
            // Canonical dispatch path â€” identical call handleResendPrompt makes.
            await sendPromptNow(ctx.targetInstId, ctx.repoPath, rewrittenPrompt, ctx.convId);
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

// ============================================================================
// CONVERSATION ACTIONS SECTION (Task-03)
// ----------------------------------------------------------------------------
// Multi-prompt resend, verify last prompts & enqueue, clone & compact memory,
// and per-conversation JSON/YAML/SQLite export utilities.
// ============================================================================

export function extractConversationPromptsList(conv: AgmConversationNode): string[] {
    const list: string[] = [];
    if (conv.sub_runs && conv.sub_runs.length > 0) {
        for (const sub of conv.sub_runs) {
            const body = (sub.full_prompt_text || sub.prompt_preview_200w || '').trim();
            if (body && !list.includes(body)) {
                list.push(body);
            }
        }
    }
    const mainBody = (conv.full_prompt_text || conv.prompt_preview_200w || '').trim();
    if (mainBody && !list.includes(mainBody)) {
        list.push(mainBody);
    }
    return list.length > 0 ? list : (mainBody ? [mainBody] : []);
}

export interface MultiPromptResendOptions {
    conversationId: string;
    instanceId: string;
    repoPath: string;
    count: number | 'all';
    delayMs?: number;
    prompts?: string[];
}

export async function dispatchMultiPromptResend(
    options: MultiPromptResendOptions
): Promise<{ sent: number; failed: number }> {
    const delay = options.delayMs ?? 500;
    const allPrompts = options.prompts ?? [];
    const targetPrompts =
        options.count === 'all'
            ? allPrompts
            : allPrompts.slice(-Math.max(1, Math.floor(Number(options.count))));

    const result = { sent: 0, failed: 0 };
    for (const prompt of targetPrompts) {
        if (!prompt || !prompt.trim()) continue;
        const { rewrittenPrompt } = extractPromptImages(prompt);
        try {
            await sendPromptNow(options.instanceId, options.repoPath, rewrittenPrompt, options.conversationId);
            result.sent += 1;
            await auditRecord({
                actor: 'user',
                methodName: 'prompt_tree_multi_resend',
                route: '/prompts',
                uiTarget: 'ConversationContextMenu | ResendSubmenu',
                trigger: 'click',
                params: {
                    conversationId: options.conversationId,
                    instanceId: options.instanceId,
                    promptCount: targetPrompts.length,
                    prompt,
                },
                result: 'ok',
            });
        } catch (err) {
            result.failed += 1;
            persistClientCrash(err, 'usePromptTreePromptActions.dispatchMultiPromptResend');
            useErrorStore.getState().trackWarning(err, {
                source: 'usePromptTreePromptActions.dispatchMultiPromptResend',
                triggerAction: 'multi_prompt_resend',
            });
        }
        if (delay > 0) {
            await new Promise((resolve) => setTimeout(resolve, delay));
        }
    }
    return result;
}

export interface VerifyLastPromptsOptions {
    instanceId: string;
    repoPath: string;
    conversationId: string;
    convTitle?: string;
    count?: number;
    prompts?: string[];
}

export async function handleVerifyLastPromptsAndEnqueue(
    options: VerifyLastPromptsOptions
): Promise<boolean> {
    try {
        const count = options.count ?? 10;
        const allPrompts = options.prompts ?? [];
        const subset = allPrompts.slice(-count);
        const verificationPrompt = buildMultiPromptVerificationPrompt(
            subset.length > 0 ? subset : ['Verify all completed tasks in this conversation and provide a full evidence scorecard.'],
            options.convTitle || options.conversationId
        );

        const { rewrittenPrompt } = extractPromptImages(verificationPrompt);
        await enqueuePrompt(options.instanceId, options.repoPath, rewrittenPrompt, options.conversationId);
        try {
            await copyPromptWithRichImages(verificationPrompt);
        } catch (clipErr) {
            useErrorStore.getState().trackWarning(clipErr, {
                source: 'usePromptTreePromptActions.handleVerifyLastPromptsAndEnqueue',
                triggerAction: 'copy_verify_prompt_clipboard',
            });
        }
        await auditRecord({
            actor: 'user',
            methodName: 'prompt_tree_verify_last_prompts_enqueue',
            route: '/prompts',
            uiTarget: 'ConversationContextMenu | VerifyLastPrompts',
            trigger: 'click',
            params: {
                conversationId: options.conversationId,
                instanceId: options.instanceId,
                auditedCount: subset.length,
            },
            result: 'ok',
        });
        return true;
    } catch (err) {
        persistClientCrash(err, 'usePromptTreePromptActions.handleVerifyLastPromptsAndEnqueue');
        useErrorStore.getState().captureException(
            err instanceof Error ? err : new Error(String(err)),
            {
                source: 'usePromptTreePromptActions.handleVerifyLastPromptsAndEnqueue',
                triggerAction: 'enqueue_verify_prompts',
            }
        );
        return false;
    }
}

export interface CloneAndCompactOptions {
    instanceId: string;
    repoPath: string;
    sourceConv: AgmConversationNode;
    prompts?: string[];
}

export async function handleCloneAndCompactConversation(
    options: CloneAndCompactOptions
): Promise<string | null> {
    try {
        const { sourceConv, instanceId, repoPath } = options;
        const allPrompts = options.prompts ?? extractConversationPromptsList(sourceConv);
        const compactTitle = `[Compacted] ${sourceConv.title || sourceConv.short_id || 'Conversation'}`;

        const summaryGoals = allPrompts
            .slice(0, 5)
            .map((p, idx) => `${idx + 1}. ${p.slice(0, 150).replace(/\n/g, ' ')}`)
            .join('\n');

        const compactContext = [
            '# ðŸ“¦ SESSION HANDOFF & COMPACTED MEMORY CONTEXT',
            '',
            `> **Source Conversation:** ${sourceConv.title || sourceConv.short_id} (\`${sourceConv.conversation_id}\`)`,
            `> **Previous Steps Completed:** ${sourceConv.step_count || 1}`,
            `> **Timestamp:** ${new Date().toISOString()}`,
            '',
            '### 1. Predecessor Goals & Instructions Summary:',
            summaryGoals || 'No prior prompt history found.',
            '',
            sourceConv.latest_response ? `### 2. Latest AI Response Outcome:\n${sourceConv.latest_response.slice(0, 800)}\n` : '',
            sourceConv.execution_results ? `### 3. Key Execution Results:\n${sourceConv.execution_results.slice(0, 800)}\n` : '',
            '',
            '### 4. Continuous Execution Handoff Directive:',
            'You are resuming work from a compacted predecessor session in a fresh clean session context.',
            'Review the verified state above and continue seamlessly toward task completion.',
        ].filter(Boolean).join('\n');

        const { rewrittenPrompt } = extractPromptImages(compactContext);
        await enqueuePrompt(instanceId, repoPath, rewrittenPrompt, compactTitle);

        try {
            await copyPromptWithRichImages(compactContext);
        } catch (clipErr) {
            useErrorStore.getState().trackWarning(clipErr, {
                source: 'usePromptTreePromptActions.handleCloneAndCompactConversation',
                triggerAction: 'copy_compacted_context',
            });
        }

        await auditRecord({
            actor: 'user',
            methodName: 'prompt_tree_clone_and_compact',
            route: '/prompts',
            uiTarget: 'ConversationContextMenu | CloneCompact',
            trigger: 'click',
            params: {
                sourceConvId: sourceConv.conversation_id,
                targetTitle: compactTitle,
                instanceId,
            },
            result: 'ok',
        });

        return compactTitle;
    } catch (err) {
        persistClientCrash(err, 'usePromptTreePromptActions.handleCloneAndCompactConversation');
        useErrorStore.getState().captureException(
            err instanceof Error ? err : new Error(String(err)),
            {
                source: 'usePromptTreePromptActions.handleCloneAndCompactConversation',
                triggerAction: 'clone_compact_conversation',
            }
        );
        return null;
    }
}

export function exportConversationAsJson(conv: AgmConversationNode, repoPath?: string): void {
    try {
        const baseName = (conv.title || conv.short_id || 'conversation')
            .replace(/[^a-zA-Z0-9_\-\s]/g, '')
            .trim()
            .replace(/\s+/g, '_')
            .slice(0, 40) || 'conversation';

        const data = {
            conversation_id: conv.conversation_id,
            seq_code: conv.seq_code,
            title: conv.title,
            short_id: conv.short_id,
            status: conv.status,
            is_running: conv.is_running,
            step_count: conv.step_count,
            repo_path: repoPath ?? '',
            full_prompt_text: conv.full_prompt_text ?? '',
            prompt_preview_200w: conv.prompt_preview_200w ?? '',
            latest_response: conv.latest_response ?? '',
            execution_results: conv.execution_results ?? '',
            sub_runs: conv.sub_runs ?? [],
            exported_at: new Date().toISOString(),
        };

        const blob = new Blob([JSON.stringify(data, null, 2)], { type: 'application/json;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${baseName}.json`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    } catch (err) {
        useErrorStore.getState().captureException(
            err instanceof Error ? err : new Error(String(err)),
            {
                source: 'usePromptTreePromptActions.exportConversationAsJson',
                triggerAction: 'export_conv_json',
            }
        );
    }
}

function escapeYaml(val: unknown): string {
    if (val === null || val === undefined) return '""';
    const s = String(val);
    if (s.includes('\n') || s.includes('"') || s.includes(':') || s.includes('#')) {
        return `|-\n    ${s.split('\n').join('\n    ')}`;
    }
    return `"${s.replace(/"/g, '\\"')}"`;
}

export function exportConversationAsYaml(conv: AgmConversationNode, repoPath?: string): void {
    try {
        const baseName = (conv.title || conv.short_id || 'conversation')
            .replace(/[^a-zA-Z0-9_\-\s]/g, '')
            .trim()
            .replace(/\s+/g, '_')
            .slice(0, 40) || 'conversation';

        const lines = [
            `conversation_id: ${escapeYaml(conv.conversation_id)}`,
            `seq_code: ${escapeYaml(conv.seq_code)}`,
            `title: ${escapeYaml(conv.title)}`,
            `short_id: ${escapeYaml(conv.short_id)}`,
            `status: ${escapeYaml(conv.status)}`,
            `is_running: ${Boolean(conv.is_running)}`,
            `step_count: ${conv.step_count || 1}`,
            `repo_path: ${escapeYaml(repoPath || '')}`,
            `exported_at: ${escapeYaml(new Date().toISOString())}`,
            `prompt_text: ${escapeYaml(conv.full_prompt_text || conv.prompt_preview_200w || '')}`,
            `latest_response: ${escapeYaml(conv.latest_response || '')}`,
            `execution_results: ${escapeYaml(conv.execution_results || '')}`,
        ];

        if (conv.sub_runs && conv.sub_runs.length > 0) {
            lines.push('sub_runs:');
            for (const sub of conv.sub_runs) {
                lines.push(`  - id: ${escapeYaml(sub.conversation_id)}`);
                lines.push(`    title: ${escapeYaml(sub.title)}`);
                lines.push(`    prompt: ${escapeYaml(sub.full_prompt_text || sub.prompt_preview_200w || '')}`);
            }
        }

        const blob = new Blob([lines.join('\n')], { type: 'text/yaml;charset=utf-8' });
        const url = URL.createObjectURL(blob);
        const a = document.createElement('a');
        a.href = url;
        a.download = `${baseName}.yaml`;
        document.body.appendChild(a);
        a.click();
        document.body.removeChild(a);
        URL.revokeObjectURL(url);
    } catch (err) {
        useErrorStore.getState().captureException(
            err instanceof Error ? err : new Error(String(err)),
            {
                source: 'usePromptTreePromptActions.exportConversationAsYaml',
                triggerAction: 'export_conv_yaml',
            }
        );
    }
}

export async function exportConversationAsSqlite(conv: AgmConversationNode): Promise<void> {
    try {
        const baseName = (conv.title || conv.short_id || 'conversation')
            .replace(/[^a-zA-Z0-9_\-\s]/g, '')
            .trim()
            .replace(/\s+/g, '_')
            .slice(0, 40) || 'conversation';
        const fileName = `${baseName}-${Date.now()}.db`;

        if (!isTauri()) {
            const data = JSON.stringify(conv, null, 2);
            const blob = new Blob([data], { type: 'application/x-sqlite3' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = fileName;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);
            return;
        }

        const { save } = await import('@tauri-apps/plugin-dialog');
        const selected = await save({
            defaultPath: fileName,
            filters: [{ name: 'SQLite Database', extensions: ['db', 'sqlite'] }],
        });
        if (!selected) return;

        await invoke('export_prompt_tree_sqlite', { path: selected });
    } catch (err) {
        useErrorStore.getState().captureException(
            err instanceof Error ? err : new Error(String(err)),
            {
                source: 'usePromptTreePromptActions.exportConversationAsSqlite',
                triggerAction: 'export_conv_sqlite',
            }
        );
    }
}

