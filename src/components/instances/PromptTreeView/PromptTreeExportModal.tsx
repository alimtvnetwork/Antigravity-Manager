import { useState, useEffect, useMemo } from 'react';
import { createPortal } from 'react-dom';
import {
    X,
    FileJson,
    Database,
    FileText,
    FileCode,
    Image as ImageIcon,
    Folder,
    MessageSquare,
    Layers,
    Download,
    Check,
    Loader2,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';

export interface ExportScopeParams {
    target_type: 'full_tree' | 'project' | 'conversation' | 'prompt_subtree';
    project_id?: string;
    project_name?: string;
    conversation_id?: string;
    prompt_ids?: string[];
}

export interface PromptTreeExportModalProps {
    isOpen: boolean;
    onClose: () => void;
    preselectedScope?: ExportScopeParams;
    initialFormat?: 'json' | 'sqlite' | 'yaml' | 'markdown';
    treeData: AgmProjectTreeNode[];
    selectedProject?: AgmProjectTreeNode | null;
    selectedConversation?: AgmConversationNode | null;
    onExecuteExport: (params: {
        scope: ExportScopeParams;
        format: 'json' | 'sqlite' | 'yaml' | 'markdown';
        embed_images: boolean;
        export_mode?: 'standalone' | 'lean';
    }) => Promise<void>;
}

export function PromptTreeExportModal({
    isOpen,
    onClose,
    preselectedScope,
    initialFormat = 'json',
    treeData,
    selectedProject,
    selectedConversation,
    onExecuteExport,
}: PromptTreeExportModalProps) {
    const [targetType, setTargetType] = useState<ExportScopeParams['target_type']>('full_tree');
    const [selectedProjId, setSelectedProjId] = useState<string>('');
    const [selectedConvId, setSelectedConvId] = useState<string>('');
    const [format, setFormat] = useState<'json' | 'sqlite' | 'yaml' | 'markdown'>('json');
    const [embedImages, setEmbedImages] = useState<boolean>(true);
    const [isExporting, setIsExporting] = useState<boolean>(false);
    const [exportSuccess, setExportSuccess] = useState<boolean>(false);

    useEffect(() => {
        if (!isOpen) return;
        setExportSuccess(false);
        setIsExporting(false);
        setFormat(initialFormat || 'json');

        if (preselectedScope) {
            setTargetType(preselectedScope.target_type);
            if (preselectedScope.project_id) {
                setSelectedProjId(preselectedScope.project_id);
            }
            if (preselectedScope.conversation_id) {
                setSelectedConvId(preselectedScope.conversation_id);
            }
        } else if (selectedConversation) {
            setTargetType('conversation');
            setSelectedConvId(selectedConversation.conversation_id);
            if (selectedProject) {
                setSelectedProjId(selectedProject.project_id);
            }
        } else if (selectedProject) {
            setTargetType('project');
            setSelectedProjId(selectedProject.project_id);
        } else {
            setTargetType('full_tree');
            if (treeData.length > 0) {
                setSelectedProjId(treeData[0].project_id);
            }
        }
    }, [isOpen, preselectedScope, initialFormat, selectedProject, selectedConversation, treeData]);

    const activeProject = useMemo(() => {
        return treeData.find((p) => p.project_id === selectedProjId) || selectedProject || treeData[0];
    }, [treeData, selectedProjId, selectedProject]);

    const activeConversation = useMemo(() => {
        if (!activeProject) return undefined;
        return activeProject.conversations?.find((c) => c.conversation_id === selectedConvId) || selectedConversation;
    }, [activeProject, selectedConvId, selectedConversation]);

    // Live statistics calculation
    const liveStats = useMemo(() => {
        if (targetType === 'full_tree') {
            const projectCount = treeData.length;
            const conversationCount = treeData.reduce((sum, p) => sum + (p.conversations?.length || 0), 0);
            const promptCount = treeData.reduce(
                (sum, p) =>
                    sum +
                    (p.conversations || []).reduce(
                        (cSum, c) => cSum + (c.step_count > 0 ? c.step_count : 1),
                        0
                    ),
                0
            );
            return { projectCount, conversationCount, promptCount };
        }
        if (targetType === 'project') {
            const convs = activeProject?.conversations || [];
            const promptCount = convs.reduce((sum, c) => sum + (c.step_count > 0 ? c.step_count : 1), 0);
            return { projectCount: 1, conversationCount: convs.length, promptCount };
        }
        if (targetType === 'conversation') {
            const promptCount = activeConversation ? (activeConversation.step_count > 0 ? activeConversation.step_count : 1) : 0;
            return { projectCount: 1, conversationCount: 1, promptCount };
        }
        return { projectCount: 1, conversationCount: 1, promptCount: 1 };
    }, [targetType, treeData, activeProject, activeConversation]);

    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                e.preventDefault();
                e.stopPropagation();
                onClose();
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, onClose]);

    if (!isOpen) return null;

    const handleExport = async () => {
        setIsExporting(true);
        try {
            const scope: ExportScopeParams = {
                target_type: targetType,
                project_id: targetType === 'project' || targetType === 'conversation' ? activeProject?.project_id : undefined,
                project_name: targetType === 'project' || targetType === 'conversation' ? activeProject?.repo_name : undefined,
                conversation_id: targetType === 'conversation' ? activeConversation?.conversation_id : undefined,
                prompt_ids: preselectedScope?.prompt_ids,
            };

            await onExecuteExport({
                scope,
                format,
                embed_images: embedImages,
                export_mode: embedImages ? 'standalone' : 'lean',
            });

            setExportSuccess(true);
            setTimeout(() => {
                onClose();
            }, 1200);
        } catch {
            // Error captured by coreActions
        } finally {
            setIsExporting(false);
        }
    };

    return createPortal(
        <div
            className="fixed inset-0 z-[100050] flex items-center justify-center p-4 bg-black/75 backdrop-blur-md overflow-y-auto animate-in fade-in duration-150"
            onClick={onClose}
        >
            <div
                className="relative w-full max-w-lg rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#120c1d] shadow-2xl p-5 space-y-4 text-slate-800 dark:text-slate-100 animate-in zoom-in-95 duration-150 max-h-[90vh] overflow-y-auto"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-100 dark:border-[#2b1b42] pb-3">
                    <div className="flex items-center gap-2">
                        <div className="p-1.5 rounded-lg bg-blue-500/10 dark:bg-blue-500/20 text-blue-600 dark:text-cyan-400">
                            <Download className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold tracking-tight">Export Prompt Tree Section</h2>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                Export scoped prompts, project hierarchy, and multimodal image assets.
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1 rounded-md text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#251b3a] transition-colors cursor-pointer"
                        title="Close (Escape)"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Scope Selection */}
                <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                        Export Scope
                    </label>
                    <div className="grid grid-cols-3 gap-1.5 p-1 rounded-xl bg-slate-100/80 dark:bg-[#1a1228] border border-slate-200/60 dark:border-[#382352]">
                        <button
                            type="button"
                            onClick={() => setTargetType('full_tree')}
                            className={cn(
                                "flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-lg transition-all cursor-pointer",
                                targetType === 'full_tree'
                                    ? "bg-blue-600 text-white shadow-xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                            )}
                        >
                            <Layers className="w-3.5 h-3.5 shrink-0" />
                            <span>Full Tree</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setTargetType('project')}
                            className={cn(
                                "flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-lg transition-all cursor-pointer",
                                targetType === 'project'
                                    ? "bg-blue-600 text-white shadow-xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                            )}
                        >
                            <Folder className="w-3.5 h-3.5 shrink-0" />
                            <span>Project</span>
                        </button>
                        <button
                            type="button"
                            onClick={() => setTargetType('conversation')}
                            className={cn(
                                "flex items-center justify-center gap-1.5 py-1.5 text-xs font-medium rounded-lg transition-all cursor-pointer",
                                targetType === 'conversation'
                                    ? "bg-blue-600 text-white shadow-xs font-semibold"
                                    : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                            )}
                        >
                            <MessageSquare className="w-3.5 h-3.5 shrink-0" />
                            <span>Conversation</span>
                        </button>
                    </div>

                    {/* Contextual dropdowns if scoped to project or conversation */}
                    {targetType === 'project' && (
                        <div className="pt-1">
                            <select
                                value={selectedProjId}
                                onChange={(e) => setSelectedProjId(e.target.value)}
                                className="w-full text-xs font-medium rounded-lg border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#1a1228] px-3 py-1.5 text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                            >
                                {treeData.map((p) => (
                                    <option key={p.project_id} value={p.project_id}>
                                        Project: {p.repo_name} ({p.conversations?.length || 0} convs)
                                    </option>
                                ))}
                            </select>
                        </div>
                    )}

                    {targetType === 'conversation' && (
                        <div className="pt-1 space-y-1.5">
                            <select
                                value={selectedProjId}
                                onChange={(e) => {
                                    setSelectedProjId(e.target.value);
                                    const proj = treeData.find((p) => p.project_id === e.target.value);
                                    if (proj && proj.conversations && proj.conversations.length > 0) {
                                        setSelectedConvId(proj.conversations[0].conversation_id);
                                    }
                                }}
                                className="w-full text-xs font-medium rounded-lg border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#1a1228] px-3 py-1.5 text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                            >
                                {treeData.map((p) => (
                                    <option key={p.project_id} value={p.project_id}>
                                        {p.repo_name}
                                    </option>
                                ))}
                            </select>
                            <select
                                value={selectedConvId}
                                onChange={(e) => setSelectedConvId(e.target.value)}
                                className="w-full text-xs font-medium rounded-lg border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#1a1228] px-3 py-1.5 text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                            >
                                {(activeProject?.conversations || []).map((c) => (
                                    <option key={c.conversation_id} value={c.conversation_id}>
                                        Conv: {c.title || c.short_id || c.conversation_id.slice(0, 8)} ({c.step_count || 1} steps)
                                    </option>
                                ))}
                            </select>
                        </div>
                    )}
                </div>

                {/* Format Cards */}
                <div className="space-y-1.5">
                    <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                        Export Format
                    </label>
                    <div className="grid grid-cols-2 sm:grid-cols-4 gap-2">
                        {/* JSON */}
                        <div
                            onClick={() => setFormat('json')}
                            className={cn(
                                "flex flex-col items-center justify-center p-3 rounded-xl border text-center cursor-pointer transition-all",
                                format === 'json'
                                    ? "border-amber-500 bg-amber-500/10 text-amber-900 dark:text-amber-200 ring-2 ring-amber-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <FileJson className="w-6 h-6 text-amber-500 mb-1" />
                            <span className="text-xs font-bold">JSON (.json)</span>
                            <span className="text-[10px] text-slate-400 mt-0.5">Envelope v2.0</span>
                        </div>

                        {/* SQLite DB */}
                        <div
                            onClick={() => setFormat('sqlite')}
                            className={cn(
                                "flex flex-col items-center justify-center p-3 rounded-xl border text-center cursor-pointer transition-all",
                                format === 'sqlite'
                                    ? "border-indigo-500 bg-indigo-500/10 text-indigo-900 dark:text-indigo-200 ring-2 ring-indigo-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <Database className="w-6 h-6 text-indigo-500 mb-1" />
                            <span className="text-xs font-bold">SQLite (.db)</span>
                            <span className="text-[10px] text-slate-400 mt-0.5">Portable DB</span>
                        </div>

                        {/* YAML */}
                        <div
                            onClick={() => setFormat('yaml')}
                            className={cn(
                                "flex flex-col items-center justify-center p-3 rounded-xl border text-center cursor-pointer transition-all",
                                format === 'yaml'
                                    ? "border-emerald-500 bg-emerald-500/10 text-emerald-900 dark:text-emerald-200 ring-2 ring-emerald-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <FileText className="w-6 h-6 text-emerald-500 mb-1" />
                            <span className="text-xs font-bold">YAML (.yaml)</span>
                            <span className="text-[10px] text-slate-400 mt-0.5">YAML Structure</span>
                        </div>

                        {/* Markdown (.md) */}
                        <div
                            onClick={() => setFormat('markdown')}
                            className={cn(
                                "flex flex-col items-center justify-center p-3 rounded-xl border text-center cursor-pointer transition-all",
                                format === 'markdown'
                                    ? "border-sky-500 bg-sky-500/10 text-sky-900 dark:text-sky-200 ring-2 ring-sky-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <FileCode className="w-6 h-6 text-sky-500 mb-1" />
                            <span className="text-xs font-bold">Markdown (.md)</span>
                            <span className="text-[10px] text-slate-400 mt-0.5">AI Ready Doc</span>
                        </div>
                    </div>
                </div>

                {/* Export Mode: Standalone vs Lean */}
                <div className="space-y-1.5">
                    <div className="flex items-center justify-between">
                        <label className="text-xs font-semibold text-slate-600 dark:text-slate-300">
                            JSON & Asset Export Mode
                        </label>
                        <span className="text-[10px] font-mono text-slate-400">
                            {embedImages ? 'Standalone (Embedded Base64)' : 'Lean (Relative Paths)'}
                        </span>
                    </div>
                    <div className="grid grid-cols-2 gap-2">
                        <div
                            onClick={() => setEmbedImages(true)}
                            className={cn(
                                "flex flex-col p-2.5 rounded-xl border cursor-pointer transition-all",
                                embedImages
                                    ? "border-purple-500 bg-purple-500/10 text-purple-900 dark:text-purple-200 ring-2 ring-purple-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <div className="flex items-center gap-1.5 font-bold text-xs">
                                <ImageIcon className="w-3.5 h-3.5 text-purple-500 shrink-0" />
                                <span>Standalone JSON</span>
                            </div>
                            <span className="text-[10px] text-slate-500 dark:text-slate-400 mt-1 leading-tight">
                                Embeds Base64 images & portable {'${workDir}'} paths directly into JSON. 100% portable across machines without external image dependencies.
                            </span>
                            <div className="mt-1.5 flex items-center gap-1 text-[9px] font-mono text-purple-600 dark:text-purple-400 font-semibold">
                                <span>● Full Portability</span>
                            </div>
                        </div>
                        <div
                            onClick={() => setEmbedImages(false)}
                            className={cn(
                                "flex flex-col p-2.5 rounded-xl border cursor-pointer transition-all",
                                !embedImages
                                    ? "border-blue-500 bg-blue-500/10 text-blue-900 dark:text-blue-200 ring-2 ring-blue-500/30"
                                    : "border-slate-200 dark:border-[#382352] hover:bg-slate-50 dark:hover:bg-[#1a1228]"
                            )}
                        >
                            <div className="flex items-center gap-1.5 font-bold text-xs">
                                <FileJson className="w-3.5 h-3.5 text-blue-500 shrink-0" />
                                <span>Lean JSON</span>
                            </div>
                            <span className="text-[10px] text-slate-500 dark:text-slate-400 mt-1 leading-tight">
                                Preserves relative file paths only. Minimal payload size with zero image data URI bloat; references local disk images.
                            </span>
                            <div className="mt-1.5 flex items-center gap-1 text-[9px] font-mono text-blue-600 dark:text-blue-400 font-semibold">
                                <span>● Zero Payload Bloat</span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Live Stats Preview */}
                <div className="flex items-center justify-between px-3 py-2 rounded-lg bg-slate-100/60 dark:bg-[#171025] border border-slate-200/50 dark:border-[#382352]/50 text-[11px] text-slate-500 dark:text-slate-400 font-mono">
                    <span>Projects: <strong className="text-slate-800 dark:text-slate-200">{liveStats.projectCount}</strong></span>
                    <span>Convs: <strong className="text-slate-800 dark:text-slate-200">{liveStats.conversationCount}</strong></span>
                    <span>Prompts: <strong className="text-slate-800 dark:text-slate-200">{liveStats.promptCount}</strong></span>
                    <span>Mode: <strong className={embedImages ? "text-purple-600 dark:text-purple-400" : "text-blue-600 dark:text-blue-400"}>{embedImages ? 'Standalone JSON (Base64)' : 'Lean JSON (Relative)'}</strong></span>
                </div>

                {/* Footer Buttons */}
                <div className="flex items-center justify-end gap-2 pt-2 border-t border-slate-100 dark:border-[#2b1b42]">
                    <button
                        type="button"
                        onClick={onClose}
                        disabled={isExporting}
                        className="px-4 py-1.5 text-xs font-semibold rounded-md border border-slate-200 dark:border-[#40285e] hover:bg-slate-100 dark:hover:bg-[#251b3a] text-slate-700 dark:text-slate-300 transition-colors cursor-pointer"
                    >
                        Cancel
                    </button>
                    <button
                        type="button"
                        onClick={handleExport}
                        disabled={isExporting || exportSuccess}
                        className={cn(
                            "flex items-center gap-1.5 px-5 py-1.5 text-xs font-bold rounded-md text-white shadow-md transition-all cursor-pointer",
                            exportSuccess
                                ? "bg-emerald-600 hover:bg-emerald-700"
                                : "bg-blue-600 hover:bg-blue-700 active:scale-95 disabled:opacity-50"
                        )}
                    >
                        {isExporting ? (
                            <>
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                <span>Exporting...</span>
                            </>
                        ) : exportSuccess ? (
                            <>
                                <Check className="w-3.5 h-3.5" />
                                <span>Exported!</span>
                            </>
                        ) : (
                            <>
                                <Download className="w-3.5 h-3.5" />
                                <span>Export Section</span>
                            </>
                        )}
                    </button>
                </div>
            </div>
        </div>,
        document.body
    );
}
