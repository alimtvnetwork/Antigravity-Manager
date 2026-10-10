import { useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import {
    focusOrLaunchInstance,
    focusInstanceWorkspace,
    sendPromptNow,
} from '../../../services/instanceService';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import type { AgmConversationNode } from './types';
import { isGhostConversation } from './conversationGuards';
import { detectPromptCategory } from './tierClassification';
import type { PromptTreeState } from './usePromptTreeState';

export interface PromptTreePromptActions {
    handleCopyWithImagesRich: () => Promise<void>;
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
        setIsCopiedRaw,
        setIsResending,
        setIsEnqueueing,
        setIsFocusing,
        confirmationSuffix,
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

    const handleCopyWithImagesRich = useCallback(async () => {
        try {
            const htmlContent = activePromptText
                .replace(/!\[(.*?)\]\((.*?)\)/g, '<img src="$2" alt="$1" style="max-width:100%;" /><br />')
                .replace(/\n/g, '<br />');
            const blobHtml = new Blob([htmlContent], { type: 'text/html' });
            const blobText = new Blob([activePromptText], { type: 'text/plain' });
            const item = new ClipboardItem({
                'text/html': blobHtml,
                'text/plain': blobText,
            });
            await navigator.clipboard.write([item]);
            setIsCopiedRaw(true);
            setActionMsg('Copied rich prompt with embedded images to clipboard!');
            setTimeout(() => {
                setIsCopiedRaw(false);
                setActionMsg(null);
            }, 2500);
        } catch {
            await navigator.clipboard.writeText(activePromptText);
            setIsCopiedRaw(true);
            setTimeout(() => setIsCopiedRaw(false), 2000);
        }
    }, [activePromptText, setIsCopiedRaw, setActionMsg]);

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
            try {
                await sendPromptNow(targetInstId, repoPath, promptContent, conv?.conversation_id);
                dispatchOk = true;
            } catch (sendErr) {
                dispatchErr = sendErr;
                console.warn('sendPromptNow error', sendErr);
            }

            // 2. Copy prompt content to clipboard so user can paste immediately (manual fallback).
            // Use the robust helper: navigator.clipboard.writeText throws
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
                const errMsg = dispatchErr instanceof Error ? dispatchErr.message : String(dispatchErr);
                setError(`Send failed: ${errMsg || 'dispatch rejected'} — prompt copied to clipboard; paste it into the IDE manually.`);
            }
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : String(err) || 'Failed to dispatch prompt');
        } finally {
            setIsResending(false);
        }
    }, [
        selectedConversation, selectedProject, treeData, archivedProjectIds,
        editedPromptText, activePromptText, confirmationSuffix, instanceId,
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
            // Single invoke against the registered backend command. Previous
            // nested fallbacks (re-invoke + disconnected resume-file write)
            // masked real failures while still reporting success.
            await invoke('enqueue_prompt', {
                instanceId: targetInstId,
                repoPath,
                promptContent,
                conversationId: selectedConversation.conversation_id,
            });

            setActionMsg('Prompt enqueued into FIFO scheduler queue!');
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: unknown) {
            setError(err instanceof Error ? err.message : String(err) || 'Failed to enqueue prompt');
        } finally {
            setIsEnqueueing(false);
        }
    }, [
        selectedConversation, selectedProject, editedPromptText, activePromptText,
        instanceId, setIsEnqueueing, setActionMsg, setError,
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
        handleCopyWithImagesRich,
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
