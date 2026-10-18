import { X, FileText, Terminal } from 'lucide-react';
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
    instanceId: string;
    instanceExeName?: string;
    instanceNameDisplay?: string;
    instanceSeqNum?: number;
    executablePath?: string;
    instancePid?: number;
}

export function DetailsModal(props: DetailsModalProps) {
    const {
        isOpen, onClose,
        selectedConversation, selectedProject,
        copiedField, setCopiedField,
        instanceId,
        instanceExeName, instanceNameDisplay, instanceSeqNum,
        executablePath, instancePid,
    } = props;
    if (!isOpen) return null;

    const resolvedExe = instanceExeName || selectedConversation?.instance_exe_name || selectedProject?.instance_exe_name || 'Antigravity IDE';
    const resolvedName = instanceNameDisplay || selectedConversation?.instance_name || selectedProject?.instance_name || instanceId || 'default';
    const resolvedSeqNum = instanceSeqNum ?? selectedConversation?.instance_seq_num ?? selectedProject?.instance_seq_num;
    const resolvedPath = executablePath || selectedProject?.repo_path || '';
    const isLive = Boolean(instancePid && instancePid > 0);

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
        <div className="w-full max-w-lg overflow-hidden rounded-xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl flex flex-col">
            {/* Details Modal Header */}
            <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#40285e] px-5 py-3.5 bg-slate-50 dark:bg-[#090611] shrink-0">
                <div className="flex items-center gap-2.5">
                    <FileText className="h-4 w-4 text-blue-500" />
                    <h3 className="text-sm font-bold text-slate-900 dark:text-white">
                        Conversation & Project Details
                    </h3>
                </div>
                <button
                    type="button"
                    onClick={onClose}
                    className="rounded-md p-1 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#40285e] dark:hover:text-slate-200 transition-colors cursor-pointer"
                    title="Close Details"
                >
                    <X className="h-4 w-4" />
                </button>
            </div>

            {/* Details Modal Body */}
            <div className="p-5 space-y-4 overflow-y-auto max-h-[70vh] text-xs">
                {/* Runtime Environment Telemetry Section */}
                <div className="space-y-2">
                    <div className="text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                        Runtime Environment
                    </div>
                    <div className="bg-slate-50 dark:bg-[#090611] rounded-lg p-3 border border-slate-200/80 dark:border-[#40285e]/80 space-y-2">
                        <div className="grid grid-cols-2 gap-2">
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Target IDE</span>
                                <span className="font-semibold text-slate-800 dark:text-slate-200 flex items-center gap-1.5 mt-0.5">
                                    <Terminal className="w-3.5 h-3.5 text-purple-400 shrink-0" />
                                    {resolvedExe}
                                </span>
                            </div>
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Instance Name</span>
                                <span className="font-semibold text-slate-800 dark:text-slate-200 block mt-0.5 truncate">
                                    {resolvedName}{resolvedSeqNum ? ` (#${resolvedSeqNum})` : ''}
                                </span>
                            </div>
                        </div>
                        <div className="pt-2 border-t border-slate-200/50 dark:border-[#40285e]/50">
                            <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Executable Path</span>
                            <div className="flex items-center justify-between gap-2 mt-0.5">
                                <span className="font-mono text-[11px] text-slate-700 dark:text-slate-300 break-all select-all">
                                    {resolvedPath || resolvedExe || 'N/A'}
                                </span>
                                {resolvedPath && (
                                    <button
                                        type="button"
                                        onClick={() => handleCopyField('exe_path', resolvedPath)}
                                        className="px-2 py-0.5 rounded-md border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] shrink-0 cursor-pointer"
                                    >
                                        {copiedField === 'exe_path' ? 'Copied!' : 'Copy'}
                                    </button>
                                )}
                            </div>
                        </div>
                        <div className="grid grid-cols-2 gap-2 pt-2 border-t border-slate-200/50 dark:border-[#40285e]/50">
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Process PID</span>
                                <span className="font-mono font-bold text-slate-800 dark:text-slate-200">
                                    {instancePid ? `PID: ${instancePid}` : 'Not Running / Detached'}
                                </span>
                            </div>
                            <div>
                                <span className="text-slate-500 dark:text-slate-400 block text-[10px] uppercase font-mono">Liveness Status</span>
                                <span className={cn('font-semibold text-xs', isLive ? 'text-emerald-500' : 'text-slate-500')}>
                                    {isLive ? 'Active (Live)' : 'Idle / Detached'}
                                </span>
                            </div>
                        </div>
                    </div>
                </div>

                {/* Conversation Section */}
                <div className="space-y-2">
                    <div className="text-[11px] font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                        Conversation Info
                    </div>
                    <div className="bg-slate-50 dark:bg-[#090611] rounded-lg p-3 border border-slate-200/80 dark:border-[#40285e]/80 space-y-2">
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
                                        className="px-2 py-0.5 rounded-md border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] shrink-0 cursor-pointer"
                                    >
                                        {copiedField === 'conv_id' ? 'Copied!' : 'Copy'}
                                    </button>
                                )}
                            </div>
                        </div>
                        <div className="grid grid-cols-2 gap-2 pt-1 border-t border-slate-200/50 dark:border-[#40285e]/50">
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
                        <div className="pt-1 border-t border-slate-200/50 dark:border-[#40285e]/50">
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
                        <div className="bg-slate-50 dark:bg-[#090611] rounded-lg p-3 border border-slate-200/80 dark:border-[#40285e]/80 space-y-2">
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
                                            className="px-2 py-0.5 rounded-md border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] text-[10px] font-medium text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] shrink-0 cursor-pointer"
                                        >
                                            {copiedField === 'repo_path' ? 'Copied!' : 'Copy'}
                                        </button>
                                    )}
                                </div>
                            </div>
                            <div className="grid grid-cols-2 gap-2 pt-1 border-t border-slate-200/50 dark:border-[#40285e]/50">
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
            <div className="flex items-center justify-end border-t border-slate-200 dark:border-[#40285e] px-5 py-3 bg-slate-50 dark:bg-[#090611] shrink-0">
                <button
                    type="button"
                    onClick={onClose}
                    className="px-4 py-1.5 rounded-md bg-slate-200 dark:bg-[#40285e] text-slate-700 dark:text-slate-200 hover:bg-slate-300 dark:hover:bg-[#1c4466] text-xs font-semibold transition-colors cursor-pointer"
                >
                    Close
                </button>
            </div>
        </div>
    </div>
    );
}
