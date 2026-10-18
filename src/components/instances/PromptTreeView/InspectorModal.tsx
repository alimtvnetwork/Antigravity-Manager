import { useState, useEffect, useMemo } from 'react';
import {
    X,
    Copy,
    Check,
    Image as ImageIcon,
    FileText,
    Folder,
    Maximize2,
    ZoomIn,
    ZoomOut,
    Sparkles,
    Terminal,
    Wrench,
    User,
    Bot,
    ChevronDown,
    ChevronRight,
    Columns,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { stripImagesFromPrompt, formatSeqBadge } from './formatters';
import { classifyPromptTier } from './tierClassification';
import { PromptMarkdownPreview } from './PromptMarkdownPreview';
import { generateVerificationPrompt } from './verificationTemplates';
import type { InspectorPrompt } from './usePromptTreeState';

export interface InspectorModalProps {
    inspectorPrompt: InspectorPrompt;
    showAllWords: boolean;
    setShowAllWords: (v: boolean) => void;
    isCopied: boolean;
    setIsCopied: (v: boolean) => void;
    isCopiedRaw: boolean;
    onClose: () => void;
    onCopyWithImages: () => void;
}

type ViewMode = 'split' | 'input' | 'output';

export function InspectorModal({
    inspectorPrompt,
    showAllWords,
    setShowAllWords,
    isCopied,
    setIsCopied,
    isCopiedRaw,
    onClose,
    onCopyWithImages,
}: InspectorModalProps) {
    const [viewMode, setViewMode] = useState<ViewMode>('split');
    const [lightboxImage, setLightboxImage] = useState<string | null>(null);
    const [lightboxZoom, setLightboxZoom] = useState(1);
    const [copiedFile, setCopiedFile] = useState<string | null>(null);
    const [copiedVerification, setCopiedVerification] = useState(false);
    const [generatedPrompt, setGeneratedPrompt] = useState<string | null>(null);
    const [showVerificationDrawer, setShowVerificationDrawer] = useState(false);
    const [isToolsExpanded, setIsToolsExpanded] = useState(false);

    // ESC key listener to dismiss lightbox first, or close modal
    useEffect(() => {
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (lightboxImage) {
                    setLightboxImage(null);
                    setLightboxZoom(1);
                } else {
                    onClose();
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [lightboxImage, onClose]);

    // Word and byte stats
    const promptText = inspectorPrompt.text || '';
    const wordCount = useMemo(() => promptText.trim().split(/\s+/).filter(Boolean).length, [promptText]);
    const byteCount = useMemo(() => new Blob([promptText]).size, [promptText]);
    const formattedBytes = useMemo(() => {
        if (byteCount >= 1024 * 1024) return `${(byteCount / (1024 * 1024)).toFixed(1)} MB`;
        if (byteCount >= 1024) return `${(byteCount / 1024).toFixed(1)} KB`;
        return `${byteCount} B`;
    }, [byteCount]);

    // Origin tier classification
    const tierInfo = useMemo(() => {
        return classifyPromptTier(promptText, inspectorPrompt.title);
    }, [promptText, inspectorPrompt.title]);

    // Check if prompt has embedded execution results or thinking
    const hasExecutionOutput = Boolean(
        (inspectorPrompt as any).latest_response ||
        (inspectorPrompt as any).execution_results ||
        promptText.includes('</thinking>') ||
        promptText.includes('</thought>')
    );

    const handleCopyCleanText = async () => {
        try {
            const cleanText = stripImagesFromPrompt(promptText);
            await copyToClipboard(cleanText);
            setIsCopied(true);
            setTimeout(() => setIsCopied(false), 2000);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InspectorModal.copyCleanText',
                triggerAction: 'copy_text',
            });
        }
    };

    const handleCopyFile = async (filePath: string) => {
        try {
            await copyToClipboard(filePath);
            setCopiedFile(filePath);
            setTimeout(() => setCopiedFile(null), 1500);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InspectorModal.copyFile',
                triggerAction: 'copy_file_path',
            });
        }
    };

    const handleGenerateVerificationPrompt = () => {
        try {
            const synthesized = generateVerificationPrompt({
                originalPrompt: promptText,
                kind: 'is-it-done',
                convId: inspectorPrompt.convId,
                convTitle: inspectorPrompt.title,
                repoPath: inspectorPrompt.projectPath,
                touchedFiles: inspectorPrompt.files,
            });
            setGeneratedPrompt(synthesized);
            setShowVerificationDrawer(true);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InspectorModal.generateVerificationPrompt',
                triggerAction: 'generate_verification_prompt',
            });
        }
    };

    const handleCopyVerificationPrompt = async () => {
        if (!generatedPrompt) {
            handleGenerateVerificationPrompt();
        }
        try {
            const toCopy = generatedPrompt || generateVerificationPrompt({
                originalPrompt: promptText,
                kind: 'is-it-done',
                convId: inspectorPrompt.convId,
                convTitle: inspectorPrompt.title,
                repoPath: inspectorPrompt.projectPath,
                touchedFiles: inspectorPrompt.files,
            });
            await copyToClipboard(toCopy);
            setCopiedVerification(true);
            setTimeout(() => setCopiedVerification(false), 2000);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'InspectorModal.copyVerificationPrompt',
                triggerAction: 'copy_verification_prompt',
            });
        }
    };

    return (
        <div className="fixed inset-0 z-[320] flex items-center justify-center bg-black/80 backdrop-blur-md p-3 sm:p-5 animate-in fade-in duration-200">
            <div className="flex h-[94vh] w-full max-w-7xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#120a1f] shadow-2xl">
                {/* 1. Header Toolbar */}
                <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#40285e] px-5 py-3 bg-slate-50 dark:bg-[#0c0617] shrink-0 gap-3 flex-wrap">
                    {/* Identity & Breadcrumbs */}
                    <div className="flex items-center gap-3 min-w-0">
                        {/* Sequence & Tier Badge */}
                        <span
                            className={cn(
                                'inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-xs font-mono font-bold border shrink-0',
                                tierInfo.tier === 'USER_PROMPT' && 'bg-sky-500/10 text-sky-600 dark:text-sky-300 border-sky-500/30',
                                tierInfo.tier === 'SUBAGENT_INSTRUCTION' && 'bg-purple-500/10 text-purple-600 dark:text-purple-300 border-purple-500/30',
                                tierInfo.tier === 'SYSTEM_MESSAGE' && 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-300 border-zinc-500/30',
                                tierInfo.tier === 'TOOL_OUTPUT' && 'bg-amber-500/10 text-amber-600 dark:text-amber-300 border-amber-500/30'
                            )}
                            title={tierInfo.tier}
                        >
                            {tierInfo.tier === 'USER_PROMPT' && <User className="w-3.5 h-3.5 text-sky-500" />}
                            {tierInfo.tier === 'SUBAGENT_INSTRUCTION' && <Bot className="w-3.5 h-3.5 text-purple-500" />}
                            {tierInfo.tier === 'SYSTEM_MESSAGE' && <Terminal className="w-3.5 h-3.5 text-zinc-500" />}
                            {tierInfo.tier === 'TOOL_OUTPUT' && <Wrench className="w-3.5 h-3.5 text-amber-500" />}
                            <span>{formatSeqBadge((inspectorPrompt as any).seq_code, 'C001')}</span>
                        </span>

                        <div className="min-w-0">
                            <h3 className="text-sm font-bold text-slate-900 dark:text-white truncate max-w-sm sm:max-w-md">
                                {inspectorPrompt.title}
                            </h3>
                            <div className="flex items-center gap-2 text-[11px] text-slate-500 dark:text-slate-400 font-mono truncate">
                                <span>Conv: {inspectorPrompt.convId}</span>
                                <span>·</span>
                                <span className="truncate">{inspectorPrompt.projectPath}</span>
                            </div>
                        </div>
                    </div>

                    {/* Controls: View Mode Switcher + Actions + Close */}
                    <div className="flex items-center gap-2.5 shrink-0 ml-auto">
                        {/* Segmented View Mode Capsule */}
                        <div className="inline-flex items-center rounded-full bg-slate-100 dark:bg-[#1a102b] p-0.5 border border-slate-200 dark:border-purple-500/30 text-xs">
                            <button
                                type="button"
                                onClick={() => setViewMode('split')}
                                className={cn(
                                    'flex items-center gap-1.5 px-3 py-1 rounded-full font-medium transition-colors cursor-pointer',
                                    viewMode === 'split'
                                        ? 'bg-white dark:bg-[#2c1a4d] text-purple-600 dark:text-purple-300 shadow-2xs font-semibold'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                                )}
                                title="Split view: side-by-side on wide screens, stacked on compact"
                            >
                                <Columns className="w-3 h-3" />
                                <span>Split</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => setViewMode('input')}
                                className={cn(
                                    'flex items-center gap-1.5 px-3 py-1 rounded-full font-medium transition-colors cursor-pointer',
                                    viewMode === 'input'
                                        ? 'bg-white dark:bg-[#2c1a4d] text-sky-600 dark:text-sky-300 shadow-2xs font-semibold'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                                )}
                                title="Focus only on prompt instruction input"
                            >
                                <FileText className="w-3 h-3" />
                                <span>Input</span>
                            </button>
                            <button
                                type="button"
                                onClick={() => setViewMode('output')}
                                className={cn(
                                    'flex items-center gap-1.5 px-3 py-1 rounded-full font-medium transition-colors cursor-pointer',
                                    viewMode === 'output'
                                        ? 'bg-white dark:bg-[#2c1a4d] text-emerald-600 dark:text-emerald-300 shadow-2xs font-semibold'
                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                                )}
                                title="Focus only on AI response and tool executions"
                            >
                                <Sparkles className="w-3 h-3" />
                                <span>Output</span>
                            </button>
                        </div>

                        {/* Quick Copy Capsule */}
                        <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#1a102b]/90 backdrop-blur-md border border-slate-200 dark:border-purple-500/30 p-0.5 divide-x divide-slate-200 dark:divide-purple-500/20 shadow-2xs">
                            <button
                                type="button"
                                onClick={handleCopyCleanText}
                                className="flex items-center gap-1.5 px-3 py-1 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-xs font-semibold transition-colors cursor-pointer"
                                title="Copy clean prompt instruction text"
                            >
                                {isCopied ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3" />}
                                <span>{isCopied ? 'Copied' : 'Copy'}</span>
                            </button>
                            <button
                                type="button"
                                onClick={onCopyWithImages}
                                className="flex items-center gap-1.5 px-2.5 py-1 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#40285e] text-xs font-semibold transition-colors cursor-pointer"
                                title="Copy text with images"
                            >
                                {isCopiedRaw ? <Check className="w-3 h-3 text-emerald-500" /> : <ImageIcon className="w-3 h-3 text-purple-400" />}
                                <span>+ Imgs</span>
                            </button>
                        </div>

                        {/* Close button */}
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex items-center justify-center h-8 w-8 rounded-lg text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 dark:hover:bg-rose-500/20 transition-colors cursor-pointer"
                            title="Close Inspector (Escape)"
                        >
                            <X className="h-4 w-4" />
                        </button>
                    </div>
                </div>

                {/* 2. Main Studio Workspace */}
                <div className="flex-1 overflow-y-auto p-4 sm:p-6 min-h-0 bg-slate-50/50 dark:bg-[#0e0719]/50">
                    <div
                        className={cn(
                            'gap-6 h-full',
                            viewMode === 'split' ? 'grid grid-cols-1 lg:grid-cols-2' : 'grid grid-cols-1'
                        )}
                    >
                        {/* Pane A: INPUT CARD (Prompt Instruction) */}
                        {(viewMode === 'split' || viewMode === 'input') && (
                            <div className="flex flex-col space-y-4 rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-sm">
                                {/* Card Header */}
                                <div className="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-[#40285e]/70">
                                    <div className="flex items-center gap-2">
                                        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-sky-500/10 text-sky-600 dark:text-sky-400 border border-sky-500/20">
                                            <FileText className="w-3.5 h-3.5" />
                                        </div>
                                        <div>
                                            <span className="text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-slate-100">
                                                Input: Prompt Instruction
                                            </span>
                                            <div className="text-[10px] text-slate-400 font-mono">
                                                {wordCount} words · {formattedBytes}
                                            </div>
                                        </div>
                                    </div>

                                    {/* 4-Tier Origin Badge */}
                                    <span
                                        className={cn(
                                            'px-2 py-0.5 rounded-full text-[10px] font-mono font-bold border flex items-center gap-1',
                                            tierInfo.tier === 'USER_PROMPT' && 'bg-sky-500/10 text-sky-600 dark:text-sky-300 border-sky-500/20',
                                            tierInfo.tier === 'SUBAGENT_INSTRUCTION' && 'bg-purple-500/10 text-purple-600 dark:text-purple-300 border-purple-500/20',
                                            tierInfo.tier === 'SYSTEM_MESSAGE' && 'bg-zinc-500/10 text-zinc-600 dark:text-zinc-300 border-zinc-500/20',
                                            tierInfo.tier === 'TOOL_OUTPUT' && 'bg-amber-500/10 text-amber-600 dark:text-amber-300 border-amber-500/20'
                                        )}
                                    >
                                        <span>{tierInfo.tier}</span>
                                    </span>
                                </div>

                                {/* Embedded Image Gallery with Lightbox Zoom */}
                                {inspectorPrompt.images && inspectorPrompt.images.length > 0 && (
                                    <div className="space-y-2">
                                        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
                                            <ImageIcon className="w-3.5 h-3.5 text-purple-500" />
                                            <span>Embedded Images ({inspectorPrompt.images.length})</span>
                                        </div>
                                        <div className="grid grid-cols-2 sm:grid-cols-3 gap-2.5">
                                            {inspectorPrompt.images.map((img, i) => (
                                                <div
                                                    key={i}
                                                    onClick={() => setLightboxImage(img)}
                                                    className="group relative h-28 overflow-hidden rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-100 dark:bg-[#10091d] cursor-pointer hover:border-purple-500/50 transition-all shadow-2xs"
                                                >
                                                    <img
                                                        src={img}
                                                        alt={`Thumbnail ${i + 1}`}
                                                        className="h-full w-full object-cover transition-transform duration-200 group-hover:scale-105"
                                                        loading="lazy"
                                                        onError={(e) => {
                                                            (e.target as HTMLElement).style.display = 'none';
                                                        }}
                                                    />
                                                    <div className="absolute inset-0 bg-black/40 opacity-0 group-hover:opacity-100 transition-opacity flex items-center justify-center text-white">
                                                        <Maximize2 className="w-4 h-4" />
                                                    </div>
                                                </div>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {/* Referenced File Chips */}
                                {inspectorPrompt.files && inspectorPrompt.files.length > 0 && (
                                    <div className="space-y-1.5">
                                        <div className="flex items-center gap-1.5 text-xs font-semibold text-slate-700 dark:text-slate-300">
                                            <Folder className="w-3.5 h-3.5 text-amber-500" />
                                            <span>Referenced Files ({inspectorPrompt.files.length})</span>
                                        </div>
                                        <div className="flex flex-wrap gap-1.5">
                                            {inspectorPrompt.files.map((file, i) => (
                                                <button
                                                    key={i}
                                                    type="button"
                                                    onClick={() => handleCopyFile(file)}
                                                    className={cn(
                                                        'inline-flex items-center gap-1.5 px-2 py-0.5 rounded-lg text-[10.5px] font-mono border transition-all cursor-pointer',
                                                        copiedFile === file
                                                            ? 'bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border-emerald-500/40'
                                                            : 'bg-slate-100 dark:bg-[#10091d] text-slate-700 dark:text-slate-300 border-slate-200 dark:border-[#40285e] hover:bg-slate-200/70 dark:hover:bg-[#40285e]'
                                                    )}
                                                    title="Click to copy file path"
                                                >
                                                    {copiedFile === file ? (
                                                        <Check className="w-3 h-3 text-emerald-500" />
                                                    ) : (
                                                        <FileText className="w-3 h-3 text-slate-400" />
                                                    )}
                                                    <span className="truncate max-w-[200px]">{file}</span>
                                                </button>
                                            ))}
                                        </div>
                                    </div>
                                )}

                                {/* Rendered Prompt Body with Charming Markdown */}
                                <div className="flex-1 flex flex-col space-y-1.5 min-h-[300px]">
                                    <span className="text-[11px] font-mono text-slate-400 uppercase tracking-wider">
                                        Instruction Body
                                    </span>
                                    <div className="flex-1 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#100a1c] p-4 text-xs text-slate-900 dark:text-slate-100 overflow-y-auto select-text shadow-inner">
                                        <PromptMarkdownPreview
                                            content={promptText}
                                            showAllWords={showAllWords}
                                            onToggleExpand={() => setShowAllWords(!showAllWords)}
                                            isTruncated={promptText.trim().endsWith('...') || promptText.trim().endsWith('…')}
                                        />
                                    </div>
                                </div>
                            </div>
                        )}

                        {/* Pane B: OUTPUT CARD (AI Execution, Thinking & Tools) */}
                        {(viewMode === 'split' || viewMode === 'output') && (
                            <div className="flex flex-col space-y-4 rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] p-5 shadow-sm">
                                {/* Card Header */}
                                <div className="flex items-center justify-between pb-3 border-b border-slate-200 dark:border-[#40285e]/70">
                                    <div className="flex items-center gap-2">
                                        <div className="flex h-7 w-7 items-center justify-center rounded-lg bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                                            <Sparkles className="w-3.5 h-3.5" />
                                        </div>
                                        <div>
                                            <span className="text-xs font-bold uppercase tracking-wider text-slate-800 dark:text-slate-100">
                                                Output: AI Execution &amp; Results
                                            </span>
                                            <div className="text-[10px] text-slate-400 font-mono">
                                                Transcript &amp; Assistant Trace
                                            </div>
                                        </div>
                                    </div>

                                    {/* Status Capsule */}
                                    {(inspectorPrompt as any).is_running ? (
                                        <span className="inline-flex items-center gap-1.5 px-2.5 py-0.5 rounded-md text-[10px] font-mono font-bold bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 animate-pulse">
                                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                                            <span>RUNNING</span>
                                        </span>
                                    ) : (
                                        <span className="inline-flex items-center gap-1 px-2.5 py-0.5 rounded-md text-[10px] font-mono font-bold bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20">
                                            <span>COMPLETED</span>
                                        </span>
                                    )}
                                </div>

                                {/* Content: Assistant Output & Tools */}
                                <div className="flex-1 overflow-y-auto space-y-3 min-h-[300px]">
                                    {/* 1. Assistant Text Response */}
                                    {(inspectorPrompt as any).latest_response ? (
                                        <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#100a1c] p-4 text-xs text-slate-800 dark:text-slate-200 shadow-inner">
                                            <div className="flex items-center gap-1.5 mb-2 pb-1.5 border-b border-slate-200/60 dark:border-[#40285e]/60 text-emerald-600 dark:text-emerald-400 font-bold text-xs">
                                                <Sparkles className="w-3.5 h-3.5" />
                                                <span>AI Assistant Response</span>
                                            </div>
                                            <PromptMarkdownPreview content={(inspectorPrompt as any).latest_response} />
                                        </div>
                                    ) : hasExecutionOutput ? (
                                        /* Extract response or markdown from prompt */
                                        <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#100a1c] p-4 text-xs text-slate-800 dark:text-slate-200 shadow-inner">
                                            <PromptMarkdownPreview content={promptText} />
                                        </div>
                                    ) : (
                                        /* Empty State */
                                        <div className="h-full flex flex-col items-center justify-center p-8 text-center rounded-xl border border-dashed border-slate-200 dark:border-[#40285e]/80 text-slate-400 space-y-2">
                                            <Terminal className="w-8 h-8 text-slate-400/60" />
                                            <div className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                                                No Output Stream Logged
                                            </div>
                                            <p className="text-[11px] max-w-xs text-slate-500">
                                                Execution results or assistant thoughts will appear here as the agent completes tasks.
                                            </p>
                                        </div>
                                    )}

                                    {/* 2. Tool Executions Pipeline Accordion */}
                                    {Boolean((inspectorPrompt as any).execution_results || (inspectorPrompt as any).tool_calls_summary) && (
                                        <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-900 text-slate-100 overflow-hidden shadow-xs">
                                            <button
                                                type="button"
                                                onClick={() => setIsToolsExpanded((prev) => !prev)}
                                                className="w-full flex items-center justify-between px-3.5 py-2 bg-slate-950/80 hover:bg-slate-950 text-left transition-colors cursor-pointer text-xs font-mono font-bold text-amber-400"
                                            >
                                                <div className="flex items-center gap-2">
                                                    <Wrench className="w-3.5 h-3.5 text-amber-400" />
                                                    <span>Tool Execution Pipeline</span>
                                                </div>
                                                <div className="flex items-center gap-1 text-[11px] text-slate-400">
                                                    <span>{isToolsExpanded ? 'Collapse' : 'Expand'}</span>
                                                    {isToolsExpanded ? <ChevronDown className="w-3.5 h-3.5" /> : <ChevronRight className="w-3.5 h-3.5" />}
                                                </div>
                                            </button>
                                            {isToolsExpanded && (
                                                <div className="p-3.5 text-[11px] font-mono whitespace-pre-wrap select-text max-h-60 overflow-y-auto leading-relaxed border-t border-slate-800">
                                                    {(inspectorPrompt as any).execution_results || (inspectorPrompt as any).tool_calls_summary}
                                                </div>
                                            )}
                                        </div>
                                    )}
                                </div>
                            </div>
                        )}
                    </div>
                </div>

                {/* 3. AI Verification Action Bar */}
                <div className="border-t border-slate-200 dark:border-[#40285e] p-3 sm:px-6 bg-slate-50/80 dark:bg-[#0c0617]/90 backdrop-blur-md shrink-0 flex flex-col gap-2">
                    <div className="flex items-center justify-between gap-3 flex-wrap">
                        <div className="flex items-center gap-2">
                            <span className="flex h-2 w-2 rounded-full bg-purple-500 animate-pulse" />
                            <span className="text-xs font-bold text-slate-800 dark:text-slate-200">
                                AI Task Verification
                            </span>
                            <span className="text-[11px] text-slate-500 dark:text-slate-400 hidden sm:inline">
                                Synthesize structured 5-Chapter prompt to audit agent completion
                            </span>
                        </div>

                        <div className="flex items-center gap-2">
                            <button
                                type="button"
                                onClick={handleGenerateVerificationPrompt}
                                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-purple-600 hover:bg-purple-700 text-white text-xs font-semibold shadow-xs transition-colors cursor-pointer"
                            >
                                <Sparkles className="w-3.5 h-3.5" />
                                <span>Generate AI Verification Prompt</span>
                            </button>

                            <button
                                type="button"
                                onClick={handleCopyVerificationPrompt}
                                className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-slate-200 dark:bg-[#25173e] hover:bg-slate-300 dark:hover:bg-[#341f57] text-slate-800 dark:text-slate-200 text-xs font-semibold transition-colors cursor-pointer"
                            >
                                {copiedVerification ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                                <span>{copiedVerification ? 'Copied Prompt!' : 'Copy Verification Prompt'}</span>
                            </button>

                            {generatedPrompt && (
                                <button
                                    type="button"
                                    onClick={() => setShowVerificationDrawer((prev) => !prev)}
                                    className="px-2.5 py-1.5 text-xs text-purple-600 dark:text-purple-400 hover:underline cursor-pointer"
                                >
                                    {showVerificationDrawer ? 'Hide Preview' : 'Show Preview'}
                                </button>
                            )}
                        </div>
                    </div>

                    {/* Verification Drawer Preview */}
                    {showVerificationDrawer && generatedPrompt && (
                        <div className="rounded-xl border border-purple-500/30 bg-purple-950/20 p-3 max-h-48 overflow-y-auto text-xs font-mono text-purple-200 whitespace-pre-wrap select-text leading-relaxed">
                            {generatedPrompt}
                        </div>
                    )}
                </div>
            </div>

            {/* 4. Full-Screen Image Lightbox with Zoom Controls */}
            {lightboxImage && (
                <div
                    className="fixed inset-0 z-[400] flex flex-col items-center justify-center bg-black/95 backdrop-blur-xl p-4 animate-in fade-in duration-150"
                    onClick={() => {
                        setLightboxImage(null);
                        setLightboxZoom(1);
                    }}
                >
                    {/* Lightbox Controls Bar */}
                    <div
                        className="absolute top-4 right-4 flex items-center gap-2 bg-slate-900/80 border border-slate-700 rounded-lg px-3 py-1.5 z-10"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <button
                            type="button"
                            onClick={() => setLightboxZoom((z) => Math.max(0.5, z - 0.25))}
                            className="p-1 text-slate-300 hover:text-white transition-colors cursor-pointer"
                            title="Zoom Out"
                        >
                            <ZoomOut className="w-4 h-4" />
                        </button>
                        <span className="text-[11px] font-mono text-slate-300 px-1">
                            {Math.round(lightboxZoom * 100)}%
                        </span>
                        <button
                            type="button"
                            onClick={() => setLightboxZoom((z) => Math.min(3, z + 0.25))}
                            className="p-1 text-slate-300 hover:text-white transition-colors cursor-pointer"
                            title="Zoom In"
                        >
                            <ZoomIn className="w-4 h-4" />
                        </button>
                        <button
                            type="button"
                            onClick={() => {
                                setLightboxImage(null);
                                setLightboxZoom(1);
                            }}
                            className="p-1 ml-1 text-slate-400 hover:text-rose-400 transition-colors cursor-pointer"
                            title="Close Lightbox (Escape)"
                        >
                            <X className="w-4 h-4" />
                        </button>
                    </div>

                    {/* Lightbox Image */}
                    <div
                        className="overflow-auto max-h-[85vh] max-w-[90vw] flex items-center justify-center"
                        onClick={(e) => e.stopPropagation()}
                    >
                        <img
                            src={lightboxImage}
                            alt="Zoomed View"
                            style={{ transform: `scale(${lightboxZoom})`, transition: 'transform 0.15s ease-out' }}
                            className="max-h-[80vh] max-w-[85vw] object-contain rounded-xl shadow-2xl"
                        />
                    </div>
                </div>
            )}
        </div>
    );
}
