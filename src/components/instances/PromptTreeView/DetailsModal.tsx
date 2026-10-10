import { X, Copy, Check, FileText } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmConversationNode, AgmProjectTreeNode } from './types';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';

export interface DetailsModalProps {
    isOpen: boolean;
    onClose: () => void;
    selectedConversation: AgmConversationNode | null;
    selectedProject: AgmProjectTreeNode | null;
    copiedField: string | null;
    setCopiedField: (v: string | null) => void;
}

export function DetailsModal(props: DetailsModalProps) {
    const {
        isOpen, onClose,
        selectedConversation, selectedProject,
        copiedField, setCopiedField,
    } = props;
    if (!isOpen) return null;

    const handleCopyField = async (field: string, value: string) => {
        try {
            await copyToClipboard(value);
            setCopiedField(field);
            setTimeout(() => setCopiedField(null), 2000);
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'DetailsModal.handleCopyField',
                triggerAction: 'copy_field',
            });
        }
    };

    return (
    <div className="fixed inset-0 z-[330] flex items-center justify-center bg-black/75 backdrop-blur-xs p-4 animate-in fade-in duration-200">
        <div className="w-full max-w-lg overflow-hidden rounded-xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl flex flex-col">
            {/* Details Modal Header */}
            <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#15334d] px-5 py-3.5 bg-slate-50 dark:bg-[#071a27] shrink-0">
                <div className="flex items-center gap-2.5">
                    <FileText className="h-4 w-4 text-blue-500" />
                    <h3 className="text-sm font-bold text-slate-900 dark:text-white">
                        Conversation & Project Details
                    </h3>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    className="rounded-[5px] p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#15334d] dark:hover:text-slate-200 transition-colors cursor-pointer"
                    title="Close Details"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>

            {/* Details Modal Body */}
            <div className="p-5 space-y-4 overflow-y-auto max-h-[70vh] text-xs">
                {/* Conversation Section */}
                <div className="space-y-2">
                    <div className="text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                        Conversation Info
                    </div>
                    <div className="bg-slate-50 dark:bg-[#071a27] rounded-lg p-3 border border-slate-200/80 dark:border-[#15334d]/80 space-y-2">
                        <div>
                            <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Title</span>
                            <span className="font-semibold text-slate-800 dark:text-slate-200">
                                {selectedConversation?.title || selectedConversation?.short_id || 'Untitled'}
                            </span>
                        </div>
                        <div>
                            <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Conversation ID</span>
                            <div className="flex items-center justify-between gap-2 mt-0.5">
                                <span className="font-mono text-[11px] text-slate-700 dark:text-slate-300 break-all">
                                    {selectedConversation?.conversation_id || 'N/A'}
                                </span>
                                {selectedConversation?.conversation_id && (
                                    <button
                                        type="button"
                                        onClick={() => {
                                            handleCopyField('conv_id', selectedConversation.conversation_id);
                                        }}
                                        className="px-2 py-0.5 rounded-[4px] border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#15334d] shrink-0 cursor-pointer"
                                    >
                                        {copiedField === 'conv_id' ? 'Copied!' : 'Copy'}
                                    </button>
                                )}
                            </div>
                        </div>
                        <div className="grid grid-cols-2 gap-2 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/50">
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Status</span>
                                <span className={cn('font-semibold', selectedConversation?.is_running ? 'text-emerald-500' : 'text-slate-600 dark:text-slate-400')}>
                                    {selectedConversation?.is_running ? 'RUNNING' : 'QUEUED / IDLE'}
                                </span>
                            </div>
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Step Count</span>
                                <span className="font-mono text-slate-800 dark:text-slate-200">
                                    {selectedConversation?.step_count || 1}
                                </span>
                            </div>
                        </div>
                        <div className="pt-1 border-t border-slate-200/50 dark:border-[#15334d]/50">
                            <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Last Modified</span>
                            <span className="font-mono text-[11px] text-slate-700 dark:text-slate-300">
                                {selectedConversation?.last_modified || 'N/A'}
                            </span>
                        </div>
                    </div>
                </div>

                {/* Project Section */}
                {selectedProject && (
                    <div className="space-y-2">
                        <div className="text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                            Project & Instance Info
                        </div>
                        <div className="bg-slate-50 dark:bg-[#071a27] rounded-lg p-3 border border-slate-200/80 dark:border-[#15334d]/80 space-y-2">
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Repository Name</span>
                                <span className="font-semibold text-slate-800 dark:text-slate-200">
                                    {selectedProject.repo_name}
                                </span>
                            </div>
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Repository Path</span>
                                <div className="flex items-center justify-between gap-2 mt-0.5">
                                    <span className="font-mono text-[11px] text-slate-700 dark:text-slate-300 break-all">
                                        {selectedProject.repo_path || 'N/A'}
                                    </span>
                                    {selectedProject.repo_path && (
                                        <button
                                            type="button"
                                            onClick={() => {
                                                handleCopyField('repo_path', selectedProject.repo_path);
                                            }}
                                            className="px-2 py-0.5 rounded-[4px] border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#15334d] shrink-0 cursor-pointer"
                                        >
                                            {copiedField === 'repo_path' ? 'Copied!' : 'Copy'}
                                        </button>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-2 gap-2 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/50">
                                <div>
                                    <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Instance ID</span>
                                    <span className="font-mono text-slate-800 dark:text-slate-200">
                                        {selectedProject.instance_id || instanceId || 'default'}
                                    </span>
                                </div>
                                <div>
                                    <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Project ID</span>
                                    <span className="font-mono text-slate-800 dark:text-slate-200 truncate block">
                                        {selectedProject.project_id}
                                    </span>
                                </div>
                            </div>
                        </div>
                    </div>
                )}
            </div>

            {/* Details Modal Footer */}
            <div className="flex items-center justify-end border-t border-slate-200 dark:border-[#15334d] px-5 py-3 bg-slate-50 dark:bg-[#071a27] shrink-0">
                <button
                    type="button"
                    onClick={onClose}
                    className="px-4 py-1.5 rounded-[5px] bg-slate-200 dark:bg-[#15334d] text-slate-700 dark:text-slate-200 hover:bg-slate-300 dark:hover:bg-[#1c4466] text-xs font-semibold transition-colors cursor-pointer"
                >
                    Close
                </button>
            </div>
        </div>
    </div>
    );
}
