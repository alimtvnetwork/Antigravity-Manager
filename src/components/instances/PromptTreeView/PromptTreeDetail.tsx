import { useState } from 'react';
import type { AgmConversationNode, AgmProjectTreeNode, ViewMode } from './types';
import { DetailHeader, type DetailHeaderProps } from './DetailHeader';
import { RichMarkdownRenderer } from './markdown';

export interface PromptTreeDetailProps {
    selectedConversation: AgmConversationNode;
    selectedProject: AgmProjectTreeNode | null;
    activePromptText: string;
    headerProps: Omit<DetailHeaderProps, 'selectedConversation' | 'activePromptText'>;
    viewMode?: ViewMode;
    onOpenInspector?: (conv: AgmConversationNode, repoPath: string) => void;
}

export function PromptTreeDetail({
    selectedConversation,
    selectedProject,
    activePromptText,
    headerProps,
    viewMode = 'preview',
    onOpenInspector,
}: PromptTreeDetailProps) {
    const [showFullText, setShowFullText] = useState(false);
    const displayedText = activePromptText || selectedConversation.full_prompt_text || selectedConversation.prompt_preview_200w || '';

    return (
        <div className="flex flex-col h-full space-y-3 font-poppins">
            {/* Header with Ubuntu title and Poppins actions */}
            <DetailHeader
                selectedConversation={selectedConversation}
                activePromptText={activePromptText}
                {...headerProps}
            />

            {/* Prompt preview & text inspector body with Poppins typography */}
            <div className="flex-1 overflow-y-auto space-y-3 font-poppins text-xs text-slate-800 dark:text-slate-200">
                {viewMode === 'preview' ? (
                    <div
                        onDoubleClick={() => onOpenInspector?.(selectedConversation, selectedProject?.repo_path || '')}
                        className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#090611] p-4 text-xs font-poppins leading-relaxed shadow-inner"
                    >
                        <RichMarkdownRenderer
                            content={displayedText}
                            showAllWords={showFullText}
                            onToggleExpand={() => setShowFullText((prev) => !prev)}
                        />
                    </div>
                ) : (
                    <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-900 text-slate-100 p-4 font-mono text-xs whitespace-pre-wrap select-text">
                        {displayedText}
                    </div>
                )}
            </div>
        </div>
    );
}

export default PromptTreeDetail;
