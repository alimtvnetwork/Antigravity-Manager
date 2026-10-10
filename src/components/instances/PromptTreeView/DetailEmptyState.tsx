import { Folder, MessageSquare, RotateCw } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { AgmProjectTreeNode } from './types';

export interface DetailEmptyStateProps {
    selectedProject: AgmProjectTreeNode | null;
    isResending: boolean;
    onResend: () => void;
}

export function DetailEmptyState(props: DetailEmptyStateProps) {
    const { selectedProject, isResending, onResend } = props;
    return (
        <div className="flex-1 flex flex-col items-center justify-center text-slate-400 text-xs p-6 space-y-3 text-center">
            {selectedProject ? (
                <>
                    <Folder className="w-8 h-8 text-amber-500/70" />
                    <span className="font-semibold text-slate-700 dark:text-slate-200 text-sm">
                        {selectedProject.repo_name}
                    </span>
                    <span className="text-slate-400 max-w-sm">
                        Select a conversation from the hierarchy, or press <kbd className="px-1.5 py-0.5 bg-slate-100 dark:bg-slate-800 border border-slate-300 dark:border-slate-700 text-slate-700 dark:text-slate-200 rounded text-[10px] font-mono font-bold">N</kbd> to dispatch the latest prompt directly.
                    </span>
                    <button
                        type="button"
                        onClick={onResend}
                        disabled={isResending}
                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-emerald-50 dark:bg-[#0c2438] text-emerald-700 dark:text-emerald-300 border border-emerald-300/60 dark:border-[#15334d] hover:bg-emerald-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer disabled:opacity-50 mt-1"
                        title="Send latest prompt from this project (Hotkey: N)"
                    >
                        <RotateCw className={cn('w-3.5 h-3.5 text-emerald-500', isResending && 'animate-spin')} />
                        <span>Send Now</span>
                        <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[9px] font-mono font-bold">N</kbd>
                    </button>
                </>
            ) : (
                <>
                    <MessageSquare className="w-8 h-8 text-slate-300 dark:text-slate-600" />
                    <span>Select a conversation from the left hierarchy to inspect prompt history</span>
                </>
            )}
        </div>
    );
}
