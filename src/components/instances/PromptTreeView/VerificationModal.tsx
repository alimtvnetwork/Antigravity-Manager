import { useState, useEffect, useMemo, useCallback } from 'react';
import {
    X,
    ShieldCheck,
    HelpCircle,
    FileCheck,
    AlertOctagon,
    Check,
    Copy,
    Send,
    Loader2,
    Layers,
    ListChecks,
    Eye,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { isTauri } from '../../../utils/env';
import { invoke } from '@tauri-apps/api/core';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { enqueuePrompt } from '../../../services/instanceService';
import { auditRecord } from '../../audit/audit-client';
import type { AgmConversationNode } from './types';
import {
    type VerificationWritingStyle,
    type MultiRoundVerificationOptions,
    WRITING_STYLES,
    buildMultiRoundVerificationPrompts,
} from './verificationTemplates';

export interface VerificationModalProps {
    isOpen: boolean;
    onClose: () => void;
    conversation: AgmConversationNode | null;
    instanceId: string;
    repoPath: string;
    onSuccess?: (enqueuedCount: number) => void;
}

interface ExtractedTurn {
    id: string;
    index: number;
    turnBadge: string;
    timestamp: string;
    text: string;
    preview: string;
}

export function VerificationModal({
    isOpen,
    onClose,
    conversation,
    instanceId,
    repoPath,
    onSuccess,
}: VerificationModalProps) {
    const [selectedIndices, setSelectedIndices] = useState<Set<number>>(new Set());
    const [rounds, setRounds] = useState<1 | 2 | 3>(2);
    const [style, setStyle] = useState<VerificationWritingStyle>('root-cause-auditor');
    const [previewRound, setPreviewRound] = useState<number>(1);
    const [customDirectives, setCustomDirectives] = useState<string>('');
    const [isDispatching, setIsDispatching] = useState<boolean>(false);
    const [feedbackMsg, setFeedbackMsg] = useState<string | null>(null);
    const [copiedPreview, setCopiedPreview] = useState<boolean>(false);

    // Extract turn items from conversation
    const turns: ExtractedTurn[] = useMemo(() => {
        if (!conversation) return [];
        const result: ExtractedTurn[] = [];

        // 1. If conversation has explicit prompts array
        if (conversation.prompts && conversation.prompts.length > 0) {
            conversation.prompts.forEach((p, idx) => {
                const text = (p.prompt_text || p.preview_200w || '').trim();
                if (text) {
                    result.push({
                        id: `turn-prompt-${p.turn_number || idx + 1}`,
                        index: idx,
                        turnBadge: `T${p.turn_number || idx + 1}`,
                        timestamp: p.timestamp || '',
                        text,
                        preview: p.preview_200w || text.slice(0, 180),
                    });
                }
            });
            return result;
        }

        // 2. If conversation has sub_runs
        if (conversation.sub_runs && conversation.sub_runs.length > 0) {
            conversation.sub_runs.forEach((sub, idx) => {
                const text = (sub.full_prompt_text || sub.prompt_preview_200w || '').trim();
                if (text) {
                    result.push({
                        id: `subrun-${sub.conversation_id || idx}`,
                        index: idx,
                        turnBadge: `T${idx + 1}`,
                        timestamp: sub.last_modified || '',
                        text,
                        preview: sub.prompt_preview_200w || text.slice(0, 180),
                    });
                }
            });
        }

        // 3. Parent prompt
        const mainText = (conversation.full_prompt_text || conversation.prompt_preview_200w || '').trim();
        if (mainText && !result.some((t) => t.text === mainText)) {
            result.push({
                id: `main-${conversation.conversation_id}`,
                index: result.length,
                turnBadge: `T${result.length + 1}`,
                timestamp: conversation.last_modified || '',
                text: mainText,
                preview: conversation.prompt_preview_200w || mainText.slice(0, 180),
            });
        }

        return result;
    }, [conversation]);

    // Initialize turn selection when modal opens or turns change
    useEffect(() => {
        if (isOpen && turns.length > 0) {
            // Default select all if <= 3, otherwise select last 3
            const initial = new Set<number>();
            if (turns.length <= 3) {
                turns.forEach((_, idx) => initial.add(idx));
            } else {
                for (let i = Math.max(0, turns.length - 3); i < turns.length; i++) {
                    initial.add(i);
                }
            }
            setSelectedIndices(initial);
            setPreviewRound(1);
            setFeedbackMsg(null);
        }
    }, [isOpen, turns]);

    // Handle Escape key listener
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape' && isOpen) {
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    // Keep preview round within bounds
    useEffect(() => {
        if (previewRound > rounds) {
            setPreviewRound(rounds);
        }
    }, [rounds, previewRound]);

    // Toggle individual turn selection
    const toggleTurn = useCallback((index: number) => {
        setSelectedIndices((prev) => {
            const next = new Set(prev);
            if (next.has(index)) {
                next.delete(index);
            } else {
                next.add(index);
            }
            return next;
        });
    }, []);

    // Quick selection helpers
    const selectAll = useCallback(() => {
        const next = new Set<number>();
        turns.forEach((_, idx) => next.add(idx));
        setSelectedIndices(next);
    }, [turns]);

    const selectLastN = useCallback((n: number) => {
        const next = new Set<number>();
        const start = Math.max(0, turns.length - n);
        for (let i = start; i < turns.length; i++) {
            next.add(i);
        }
        setSelectedIndices(next);
    }, [turns]);

    const clearSelection = useCallback(() => {
        setSelectedIndices(new Set());
    }, []);

    // Synthesize prompts for preview and dispatch
    const synthesizedPrompts = useMemo(() => {
        const selectedTexts = turns
            .filter((_, idx) => selectedIndices.has(idx))
            .map((t) => t.text);

        const options: MultiRoundVerificationOptions = {
            conversationId: conversation?.conversation_id || 'default-session',
            conversationTitle: conversation?.title || conversation?.short_id || 'Current Task',
            instanceId: instanceId || 'default',
            repoPath: repoPath || '',
            roundsCount: rounds,
            rounds,
            writingStyle: style,
            style,
            prompts: selectedTexts,
            turns: selectedTexts,
            customDirectives: customDirectives.trim() || undefined,
        };

        return buildMultiRoundVerificationPrompts(options);
    }, [turns, selectedIndices, conversation, instanceId, repoPath, rounds, style, customDirectives]);

    // Active round preview markdown
    const activePreviewMarkdown = useMemo(() => {
        const roundIdx = Math.max(0, Math.min(synthesizedPrompts.length - 1, previewRound - 1));
        return synthesizedPrompts[roundIdx] || '';
    }, [synthesizedPrompts, previewRound]);

    // Copy live preview
    const handleCopyPreview = useCallback(async () => {
        if (!activePreviewMarkdown) return;
        const ok = await copyToClipboard(activePreviewMarkdown);
        if (ok) {
            setCopiedPreview(true);
            setTimeout(() => setCopiedPreview(false), 2000);
        }
    }, [activePreviewMarkdown]);

    // Dispatch verification queue
    const handleDispatch = useCallback(async () => {
        if (synthesizedPrompts.length === 0 || selectedIndices.size === 0) {
            setFeedbackMsg('Please select at least one prompt turn to audit.');
            setTimeout(() => setFeedbackMsg(null), 3000);
            return;
        }

        try {
            setIsDispatching(true);
            setFeedbackMsg(`Enqueueing ${synthesizedPrompts.length} verification round(s)...`);

            const targetInstId = instanceId || 'default';
            const targetConvId = conversation?.conversation_id;

            // 1. Attempt native Tauri IPC first if available
            let enqueuedViaIpc = false;
            if (isTauri()) {
                try {
                    await invoke('enqueue_verification_rounds', {
                        instanceId: targetInstId,
                        prompts: synthesizedPrompts,
                        sessionId: targetConvId,
                    });
                    enqueuedViaIpc = true;
                } catch {
                    // Fallback to enqueue_prompt ladder below
                    enqueuedViaIpc = false;
                }
            }

            // 2. Sequential fallback: enqueuePrompt with Invariant I2 (500ms safety sleep)
            if (!enqueuedViaIpc) {
                for (let i = 0; i < synthesizedPrompts.length; i++) {
                    const roundPrompt = synthesizedPrompts[i];
                    await enqueuePrompt(targetInstId, repoPath, roundPrompt, targetConvId);
                    if (i < synthesizedPrompts.length - 1) {
                        await new Promise((resolve) => setTimeout(resolve, 500));
                    }
                }
            }

            // 3. Copy Round 1 prompt to clipboard for immediate user access
            if (synthesizedPrompts.length > 0) {
                try {
                    await copyToClipboard(synthesizedPrompts[0]);
                } catch (clipErr) {
                    useErrorStore.getState().trackWarning(clipErr, {
                        source: 'VerificationModal.handleDispatch',
                        triggerAction: 'copy_round_1_clipboard',
                    });
                }
            }

            // 4. Record interaction in audit log
            await auditRecord({
                actor: 'user',
                methodName: 'dispatch_verification_rounds',
                uiTarget: 'VerificationModal | DispatchQueue',
                trigger: 'click',
                params: {
                    instanceId: targetInstId,
                    conversationId: targetConvId,
                    roundsCount: synthesizedPrompts.length,
                    writingStyle: style,
                    turnsCount: selectedIndices.size,
                },
                result: 'ok',
            });

            setFeedbackMsg(`Successfully enqueued ${synthesizedPrompts.length} verification round(s)!`);
            onSuccess?.(synthesizedPrompts.length);

            setTimeout(() => {
                onClose();
            }, 1200);
        } catch (err: unknown) {
            const errorObj = err instanceof Error ? err : new Error(String(err) || 'Failed to dispatch verification rounds');
            useErrorStore.getState().captureException(errorObj, {
                source: 'VerificationModal.handleDispatch',
                triggerAction: 'dispatch_verification_rounds',
            });
            setFeedbackMsg(`Dispatch failed: ${errorObj.message}`);
        } finally {
            setIsDispatching(false);
        }
    }, [synthesizedPrompts, selectedIndices, instanceId, conversation, repoPath, style, onSuccess, onClose]);

    if (!isOpen) return null;

    const convTitle = conversation?.title || conversation?.short_id || 'Current Task';
    const seqCode = conversation?.seq_code || 'C001';

    return (
        <div
            className="fixed inset-0 z-50 flex items-center justify-center p-4 bg-black/80 backdrop-blur-md animate-fade-in"
            onClick={onClose}
        >
            <div
                className="relative w-full max-w-4xl max-h-[90vh] flex flex-col bg-slate-950/90 border border-slate-700/60 rounded-2xl shadow-2xl overflow-hidden backdrop-blur-xl text-slate-100"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between px-6 py-4 border-b border-slate-800/80 bg-slate-900/50">
                    <div className="flex items-center space-x-3">
                        <div className="flex items-center justify-center w-9 h-9 rounded-xl bg-indigo-500/10 border border-indigo-500/30 text-indigo-400">
                            <ShieldCheck className="w-5 h-5" />
                        </div>
                        <div>
                            <div className="flex items-center space-x-2">
                                <h2 className="text-base font-semibold text-slate-100">
                                    🛡️ Verification Segment & AI Auditor Studio
                                </h2>
                                <span className="px-2 py-0.5 text-xs font-mono font-medium rounded-full bg-indigo-500/20 text-indigo-300 border border-indigo-500/30">
                                    [AGM:{seqCode}]
                                </span>
                            </div>
                            <p className="text-xs text-slate-400 truncate max-w-lg mt-0.5">
                                Task: <span className="text-slate-200 font-medium">{convTitle}</span>
                                {instanceId && <span className="ml-2 text-slate-400">({instanceId})</span>}
                            </p>
                        </div>
                    </div>
                    <button
                        onClick={onClose}
                        className="p-1.5 text-slate-400 hover:text-slate-200 hover:bg-slate-800/60 rounded-lg transition-colors"
                        title="Close (Esc)"
                    >
                        <X className="w-5 h-5" />
                    </button>
                </div>

                {/* Body Content */}
                <div className="flex-1 overflow-y-auto px-6 py-5 space-y-6 custom-scrollbar">
                    {/* Section 1: Turn Selection */}
                    <div className="space-y-2.5">
                        <div className="flex items-center justify-between">
                            <label className="text-xs font-semibold tracking-wider uppercase text-slate-300 flex items-center space-x-1.5">
                                <ListChecks className="w-3.5 h-3.5 text-indigo-400" />
                                <span>1. Select Instruction Turns to Audit ({selectedIndices.size}/{turns.length})</span>
                            </label>
                            <div className="flex items-center space-x-1 text-xs">
                                <button
                                    onClick={selectAll}
                                    className="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-slate-100 transition-colors"
                                >
                                    Select All
                                </button>
                                <button
                                    onClick={() => selectLastN(3)}
                                    className="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-slate-100 transition-colors"
                                >
                                    Last 3
                                </button>
                                <button
                                    onClick={() => selectLastN(5)}
                                    className="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-slate-300 hover:text-slate-100 transition-colors"
                                >
                                    Last 5
                                </button>
                                <button
                                    onClick={clearSelection}
                                    className="px-2 py-1 rounded bg-slate-800/80 hover:bg-slate-700 text-slate-400 hover:text-slate-200 transition-colors"
                                >
                                    Clear
                                </button>
                            </div>
                        </div>

                        <div className="max-h-44 overflow-y-auto rounded-xl border border-slate-800/90 bg-slate-900/40 divide-y divide-slate-800/50 custom-scrollbar">
                            {turns.length === 0 ? (
                                <div className="p-4 text-center text-xs text-slate-500">
                                    No prompt turns recorded for this conversation yet.
                                </div>
                            ) : (
                                turns.map((turn) => {
                                    const isChecked = selectedIndices.has(turn.index);
                                    return (
                                        <div
                                            key={turn.id}
                                            onClick={() => toggleTurn(turn.index)}
                                            className={cn(
                                                'flex items-start space-x-3 px-3.5 py-2.5 cursor-pointer transition-colors',
                                                isChecked
                                                    ? 'bg-indigo-950/20 hover:bg-indigo-950/30'
                                                    : 'hover:bg-slate-800/30 text-slate-400'
                                            )}
                                        >
                                            <input
                                                type="checkbox"
                                                checked={isChecked}
                                                onChange={() => {}}
                                                className="mt-0.5 rounded border-slate-700 bg-slate-900 text-indigo-500 focus:ring-0 focus:ring-offset-0 cursor-pointer"
                                            />
                                            <div className="flex-1 min-w-0">
                                                <div className="flex items-center space-x-2">
                                                    <span className="px-1.5 py-0.5 text-[11px] font-mono font-semibold rounded bg-slate-800 text-slate-300 border border-slate-700">
                                                        {turn.turnBadge}
                                                    </span>
                                                    {turn.timestamp && (
                                                        <span className="text-[11px] text-slate-500">
                                                            {turn.timestamp}
                                                        </span>
                                                    )}
                                                </div>
                                                <p className="text-xs text-slate-300 mt-1 line-clamp-2 font-mono leading-relaxed">
                                                    {turn.preview}
                                                </p>
                                            </div>
                                        </div>
                                    );
                                })
                            )}
                        </div>
                    </div>

                    {/* Section 2: Repetition Rounds Selector */}
                    <div className="space-y-2">
                        <label className="text-xs font-semibold tracking-wider uppercase text-slate-300 flex items-center space-x-1.5">
                            <Layers className="w-3.5 h-3.5 text-indigo-400" />
                            <span>2. Verification Rounds Escalation</span>
                        </label>
                        <div className="inline-flex p-1 rounded-full bg-slate-900/80 border border-slate-800 shadow-inner">
                            <button
                                type="button"
                                onClick={() => setRounds(1)}
                                className={cn(
                                    'px-4 py-1.5 text-xs font-medium rounded-full transition-all duration-200',
                                    rounds === 1
                                        ? 'bg-indigo-600 text-white shadow-md'
                                        : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                1x Single Audit
                            </button>
                            <button
                                type="button"
                                onClick={() => setRounds(2)}
                                className={cn(
                                    'px-4 py-1.5 text-xs font-medium rounded-full transition-all duration-200',
                                    rounds === 2
                                        ? 'bg-indigo-600 text-white shadow-md'
                                        : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                2x (Recommended)
                            </button>
                            <button
                                type="button"
                                onClick={() => setRounds(3)}
                                className={cn(
                                    'px-4 py-1.5 text-xs font-medium rounded-full transition-all duration-200',
                                    rounds === 3
                                        ? 'bg-indigo-600 text-white shadow-md'
                                        : 'text-slate-400 hover:text-slate-200'
                                )}
                            >
                                3x Full Release Gate
                            </button>
                        </div>
                    </div>

                    {/* Section 3: Writing Style Selector */}
                    <div className="space-y-2.5">
                        <label className="text-xs font-semibold tracking-wider uppercase text-slate-300 flex items-center space-x-1.5">
                            <Eye className="w-3.5 h-3.5 text-indigo-400" />
                            <span>3. Choose AI Auditor Writing Style</span>
                        </label>
                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                            {WRITING_STYLES.map((st) => {
                                const isSelected = style === st.id;
                                const IconComponent =
                                    st.id === 'root-cause-auditor'
                                        ? ShieldCheck
                                        : st.id === 'socratic-skeptic'
                                          ? HelpCircle
                                          : st.id === 'spec-gatekeeper'
                                            ? FileCheck
                                            : AlertOctagon;

                                return (
                                    <div
                                        key={st.id}
                                        onClick={() => setStyle(st.id)}
                                        className={cn(
                                            'p-3.5 rounded-xl border cursor-pointer transition-all duration-200 text-left flex items-start space-x-3',
                                            isSelected
                                                ? 'bg-indigo-950/30 border-indigo-500/80 ring-2 ring-indigo-500/30 shadow-lg'
                                                : 'bg-slate-900/40 border-slate-800/80 hover:border-slate-700/80'
                                        )}
                                    >
                                        <div
                                            className={cn(
                                                'p-2 rounded-lg shrink-0 mt-0.5',
                                                isSelected
                                                    ? 'bg-indigo-500/20 text-indigo-400'
                                                    : 'bg-slate-800 text-slate-400'
                                            )}
                                        >
                                            <IconComponent className="w-4 h-4" />
                                        </div>
                                        <div className="flex-1 min-w-0">
                                            <div className="flex items-center justify-between">
                                                <h4 className="text-xs font-semibold text-slate-100">
                                                    {st.title}
                                                </h4>
                                                <span className="px-1.5 py-0.5 text-[10px] font-mono rounded bg-slate-800/90 text-slate-300 border border-slate-700/50">
                                                    {st.badge}
                                                </span>
                                            </div>
                                            <p className="text-[11px] font-medium text-slate-400 mt-0.5">
                                                {st.subtitle}
                                            </p>
                                            <p className="text-[11px] text-slate-500 mt-1 line-clamp-2 leading-relaxed">
                                                {st.description}
                                            </p>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </div>

                    {/* Section 4: Live Preview Tabs */}
                    <div className="space-y-2">
                        <div className="flex items-center justify-between">
                            <div className="flex items-center space-x-1.5">
                                <label className="text-xs font-semibold tracking-wider uppercase text-slate-300">
                                    4. Synthesized Prompt Preview
                                </label>
                                <span className="text-xs text-slate-500">
                                    (Viewing Round {previewRound} of {rounds})
                                </span>
                            </div>
                            <button
                                type="button"
                                onClick={handleCopyPreview}
                                className="flex items-center space-x-1 text-xs text-slate-400 hover:text-slate-200 transition-colors"
                            >
                                {copiedPreview ? (
                                    <>
                                        <Check className="w-3.5 h-3.5 text-emerald-400" />
                                        <span className="text-emerald-400">Copied!</span>
                                    </>
                                ) : (
                                    <>
                                        <Copy className="w-3.5 h-3.5" />
                                        <span>Copy Preview</span>
                                    </>
                                )}
                            </button>
                        </div>

                        {/* Round Tabs */}
                        <div className="flex items-center space-x-1 border-b border-slate-800">
                            {Array.from({ length: rounds }, (_, i) => i + 1).map((r) => (
                                <button
                                    key={r}
                                    type="button"
                                    onClick={() => setPreviewRound(r)}
                                    className={cn(
                                        'px-3.5 py-1.5 text-xs font-medium border-b-2 transition-colors',
                                        previewRound === r
                                            ? 'border-indigo-500 text-indigo-400 bg-indigo-950/20'
                                            : 'border-transparent text-slate-400 hover:text-slate-200'
                                    )}
                                >
                                    Round {r}
                                </button>
                            ))}
                        </div>

                        {/* Preview Box */}
                        <div className="relative rounded-xl border border-slate-800/90 bg-slate-900/60 p-3.5 font-mono text-xs text-slate-300 max-h-56 overflow-y-auto whitespace-pre-wrap select-text custom-scrollbar">
                            {activePreviewMarkdown || (
                                <span className="text-slate-500 italic">
                                    Select prompt turns above to generate preview.
                                </span>
                            )}
                        </div>
                    </div>

                    {/* Section 5: Optional Custom Directives */}
                    <div className="space-y-1.5">
                        <label className="text-xs font-semibold tracking-wider uppercase text-slate-400">
                            Custom Operator Directives (Optional)
                        </label>
                        <input
                            type="text"
                            value={customDirectives}
                            onChange={(e) => setCustomDirectives(e.target.value)}
                            placeholder="e.g. Focus particularly on memory safety and 500ms safety sleep Invariant I2..."
                            className="w-full px-3 py-2 text-xs rounded-lg border border-slate-800 bg-slate-900/70 text-slate-200 placeholder-slate-500 focus:outline-none focus:border-indigo-500/80 transition-colors"
                        />
                    </div>
                </div>

                {/* Footer Controls */}
                <div className="flex items-center justify-between px-6 py-4 border-t border-slate-800/80 bg-slate-900/50">
                    <div className="text-xs text-slate-400">
                        {feedbackMsg ? (
                            <span className={cn(
                                'font-medium',
                                feedbackMsg.includes('failed') ? 'text-rose-400' : 'text-emerald-400'
                            )}>
                                {feedbackMsg}
                            </span>
                        ) : (
                            <span>
                                {selectedIndices.size} turn(s) selected • {rounds} round(s) scheduled
                            </span>
                        )}
                    </div>

                    <div className="flex items-center space-x-3">
                        <button
                            type="button"
                            onClick={onClose}
                            disabled={isDispatching}
                            className="px-4 py-2 text-xs font-medium rounded-lg border border-slate-700/80 text-slate-300 hover:bg-slate-800/60 transition-colors disabled:opacity-50"
                        >
                            Cancel
                        </button>
                        <button
                            type="button"
                            onClick={handleDispatch}
                            disabled={isDispatching || selectedIndices.size === 0}
                            className={cn(
                                'flex items-center space-x-2 px-5 py-2 text-xs font-semibold rounded-lg shadow-md transition-all duration-200',
                                selectedIndices.size === 0
                                    ? 'bg-slate-800 text-slate-500 cursor-not-allowed'
                                    : 'bg-indigo-600 hover:bg-indigo-500 text-white shadow-indigo-500/20 active:scale-[0.98]'
                            )}
                        >
                            {isDispatching ? (
                                <>
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                    <span>Dispatching...</span>
                                </>
                            ) : (
                                <>
                                    <Send className="w-3.5 h-3.5" />
                                    <span>⚡ Dispatch Verification Queue ({rounds} Round{rounds > 1 ? 's' : ''})</span>
                                </>
                            )}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}
