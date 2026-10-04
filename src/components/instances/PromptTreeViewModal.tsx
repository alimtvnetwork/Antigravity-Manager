import { useState, useEffect, useMemo, useCallback } from 'react';
import {
    X,
    Folder,
    Layers,
    MessageSquare,
    Download,
    Upload,
    Maximize2,
    Copy,
    Check,
    FileText,
    Image as ImageIcon,
    RefreshCw,
    Search,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    Sparkles,
    RotateCw,
    RotateCcw,
    ListPlus,
    Eye,
    Code,
    Edit3,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../utils/cn';

export interface AgmConversationNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    conversation_id: string;
    short_id: string;
    title: string;
    status: string;
    is_running: boolean;
    step_count: number;
    instance_id: string;
    prompt_preview_200w: string;
    prompt_word_count: number;
    last_modified: string;
}

export interface AgmProjectTreeNode {
    seq_id: number;
    seq_code: string;
    gitmap_seq_code: string;
    project_id: string;
    repo_name: string;
    repo_path: string;
    instance_id: string;
    instance_seq_num?: number;
    instance_name: string;
    bound_email?: string;
    is_running: boolean;
    conversations: AgmConversationNode[];
}

export interface PromptTurnNode {
    turnNumber: number;
    title: string;
    text: string;
    wordCount: number;
}

interface PromptTreeViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
}

type ViewMode = 'preview' | 'raw' | 'edit';

// Helper to count words
function countWords(str: string): number {
    const trimmed = str.trim();
    if (!trimmed) return 0;
    return trimmed.split(/\s+/).length;
}

// Helper to truncate text at word limit
function getTruncatedText(text: string, maxWords: number): { displayText: string; isTruncated: boolean; totalWords: number } {
    const trimmed = text.trim();
    if (!trimmed) return { displayText: '', isTruncated: false, totalWords: 0 };
    const words = trimmed.split(/\s+/);
    if (words.length <= maxWords) {
        return { displayText: text, isTruncated: false, totalWords: words.length };
    }
    const truncated = words.slice(0, maxWords).join(' ');
    return { displayText: truncated, isTruncated: true, totalWords: words.length };
}

// Helper to derive prompt turns for Layer 3
function getPromptTurns(conv: AgmConversationNode): PromptTurnNode[] {
    const rawText = conv.prompt_preview_200w || '';
    const wordCount = conv.prompt_word_count || countWords(rawText);
    const steps = Math.min(Math.max(conv.step_count || 1, 1), 10);

    if (steps > 1) {
        const turns: PromptTurnNode[] = [];
        for (let i = 1; i <= steps; i++) {
            const isLatest = i === steps;
            turns.push({
                turnNumber: i,
                title: isLatest ? `Turn #${i} (Latest)` : `Turn #${i}`,
                text: isLatest ? rawText : `[Turn #${i}] Checkpoint prompt in conversation ${conv.short_id || conv.conversation_id}`,
                wordCount: isLatest ? wordCount : Math.max(15, Math.round(wordCount / steps)),
            });
        }
        return turns;
    }

    return [
        {
            turnNumber: 1,
            title: 'Turn #1',
            text: rawText || 'No prompt content recorded for this conversation.',
            wordCount,
        },
    ];
}

// Helper to check if a conversation is untitled or empty
function isUntitledOrEmpty(conv: AgmConversationNode): boolean {
    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled conversation') ||
        title === 'new conversation' ||
        title === 'conversation' ||
        title === (conv.short_id || '').toLowerCase();
    const isEmptyPrompt = !conv.prompt_preview_200w || conv.prompt_preview_200w.trim().length === 0;
    return isUntitled || isEmptyPrompt;
}

// Inline Markdown parser (Headings, bold, italic, code blocks, inline code, lists, images, blockquotes)
function parseInlineMarkdown(text: string): React.ReactNode[] {
    const nodes: React.ReactNode[] = [];
    const tokenRegex = /(!\[(.*?)\]\((.*?)\)|\[(.*?)\]\((.*?)\)|`([^`]+)`|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*]+)\*|_([^_]+)_)/g;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = tokenRegex.exec(text)) !== null) {
        if (match.index > lastIndex) {
            nodes.push(text.substring(lastIndex, match.index));
        }

        const [full, , imgAlt, imgSrc, linkText, linkUrl, inlineCode, boldStar, boldUnder, delText, italicStar, italicUnder] = match;

        if (imgSrc !== undefined) {
            nodes.push(
                <img
                    key={`img-${match.index}`}
                    src={imgSrc}
                    alt={imgAlt || ''}
                    className="inline-block max-h-48 max-w-full my-1 rounded-[5px] border border-slate-200 dark:border-[#15334d]"
                    onError={(e) => {
                        (e.target as HTMLElement).style.display = 'none';
                    }}
                />
            );
        } else if (linkUrl !== undefined) {
            nodes.push(
                <a
                    key={`link-${match.index}`}
                    href={linkUrl}
                    target="_blank"
                    rel="noopener noreferrer"
                    className="text-blue-500 dark:text-cyan-400 hover:underline"
                >
                    {linkText || linkUrl}
                </a>
            );
        } else if (inlineCode !== undefined) {
            nodes.push(
                <code
                    key={`code-${match.index}`}
                    className="px-1.5 py-0.5 rounded-[4px] bg-slate-100 dark:bg-[#071a27] text-pink-600 dark:text-pink-400 font-mono text-[11px] border border-slate-200 dark:border-[#15334d]"
                >
                    {inlineCode}
                </code>
            );
        } else if (boldStar !== undefined || boldUnder !== undefined) {
            nodes.push(
                <strong key={`bold-${match.index}`} className="font-bold text-slate-900 dark:text-white">
                    {boldStar || boldUnder}
                </strong>
            );
        } else if (delText !== undefined) {
            nodes.push(
                <del key={`del-${match.index}`} className="line-through opacity-70">
                    {delText}
                </del>
            );
        } else if (italicStar !== undefined || italicUnder !== undefined) {
            nodes.push(
                <em key={`italic-${match.index}`} className="italic">
                    {italicStar || italicUnder}
                </em>
            );
        } else {
            nodes.push(full);
        }

        lastIndex = tokenRegex.lastIndex;
    }

    if (lastIndex < text.length) {
        nodes.push(text.substring(lastIndex));
    }

    return nodes.length > 0 ? nodes : [text];
}

// Rich Markdown renderer component
function RichMarkdownRenderer({ content }: { content: string }) {
    const [copiedBlockIndex, setCopiedBlockIndex] = useState<number | null>(null);

    const handleCopyCode = (code: string, index: number) => {
        navigator.clipboard.writeText(code);
        setCopiedBlockIndex(index);
        setTimeout(() => setCopiedBlockIndex(null), 2000);
    };

    if (!content.trim()) {
        return <div className="text-xs text-slate-400 italic">No content to preview.</div>;
    }

    const lines = content.split('\n');
    const elements: React.ReactNode[] = [];
    let inCodeBlock = false;
    let codeLanguage = '';
    let codeBuffer: string[] = [];
    let listBuffer: { type: 'ul' | 'ol'; items: string[] } | null = null;
    let blockquoteBuffer: string[] = [];

    const flushList = (keyPrefix: number) => {
        if (!listBuffer) return;
        const currentList = listBuffer;
        listBuffer = null;
        if (currentList.type === 'ul') {
            elements.push(
                <ul key={`ul-${keyPrefix}`} className="list-disc ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item)}</li>
                    ))}
                </ul>
            );
        } else {
            elements.push(
                <ol key={`ol-${keyPrefix}`} className="list-decimal ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item)}</li>
                    ))}
                </ol>
            );
        }
    };

    const flushBlockquote = (keyPrefix: number) => {
        if (blockquoteBuffer.length === 0) return;
        const text = blockquoteBuffer.join('\n');
        blockquoteBuffer = [];
        elements.push(
            <blockquote
                key={`quote-${keyPrefix}`}
                className="border-l-3 border-blue-500/80 bg-blue-50/30 dark:bg-blue-950/20 px-3 py-1.5 my-2 rounded-r-[5px] text-xs italic text-slate-700 dark:text-slate-300"
            >
                {parseInlineMarkdown(text)}
            </blockquote>
        );
    };

    for (let i = 0; i < lines.length; i++) {
        const line = lines[i];
        const trimmed = line.trim();

        // Check code blocks ```
        if (trimmed.startsWith('```')) {
            if (inCodeBlock) {
                // End code block
                const fullCode = codeBuffer.join('\n');
                const blockIdx = i;
                const lang = codeLanguage;
                elements.push(
                    <div
                        key={`codeblock-${blockIdx}`}
                        className="my-3 overflow-hidden rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-slate-900 text-slate-100"
                    >
                        <div className="flex items-center justify-between border-b border-slate-800 bg-slate-950/70 px-3 py-1.5 text-[11px] font-mono text-slate-400">
                            <span>{lang || 'code'}</span>
                            <button
                                type="button"
                                onClick={() => handleCopyCode(fullCode, blockIdx)}
                                className="flex items-center gap-1 rounded-[5px] px-2 py-0.5 hover:bg-slate-800 text-slate-300 transition-colors cursor-pointer"
                                title="Copy code"
                            >
                                {copiedBlockIndex === blockIdx ? (
                                    <>
                                        <Check className="w-3 h-3 text-emerald-400" />
                                        <span className="text-[10px] text-emerald-400">Copied</span>
                                    </>
                                ) : (
                                    <>
                                        <Copy className="w-3 h-3" />
                                        <span className="text-[10px]">Copy</span>
                                    </>
                                )}
                            </button>
                        </div>
                        <pre className="p-3 font-mono text-xs overflow-x-auto leading-relaxed select-text">
                            {fullCode}
                        </pre>
                    </div>
                );
                codeBuffer = [];
                codeLanguage = '';
                inCodeBlock = false;
            } else {
                flushList(i);
                flushBlockquote(i);
                inCodeBlock = true;
                codeLanguage = trimmed.slice(3).trim();
                codeBuffer = [];
            }
            continue;
        }

        if (inCodeBlock) {
            codeBuffer.push(line);
            continue;
        }

        // Check blockquote >
        if (trimmed.startsWith('>')) {
            flushList(i);
            blockquoteBuffer.push(trimmed.slice(1).trim());
            continue;
        } else {
            flushBlockquote(i);
        }

        // Check bullet lists
        const ulMatch = line.match(/^(\s*)[-*+]\s+(.*)$/);
        if (ulMatch) {
            if (listBuffer && listBuffer.type !== 'ul') {
                flushList(i);
            }
            if (!listBuffer) {
                listBuffer = { type: 'ul', items: [] };
            }
            listBuffer.items.push(ulMatch[2]);
            continue;
        }

        // Check numbered lists
        const olMatch = line.match(/^(\s*)\d+\.\s+(.*)$/);
        if (olMatch) {
            if (listBuffer && listBuffer.type !== 'ol') {
                flushList(i);
            }
            if (!listBuffer) {
                listBuffer = { type: 'ol', items: [] };
            }
            listBuffer.items.push(olMatch[2]);
            continue;
        }

        flushList(i);

        if (!trimmed) {
            continue;
        }

        // Horizontal rules
        if (/^(---|\*\*\*|___)$/.test(trimmed)) {
            elements.push(<hr key={`hr-${i}`} className="my-3 border-slate-200 dark:border-[#15334d]" />);
            continue;
        }

        // Headings
        if (trimmed.startsWith('# ')) {
            elements.push(
                <h1 key={`h1-${i}`} className="text-base font-bold mt-3 mb-1.5 text-slate-900 dark:text-white pb-1 border-b border-slate-200 dark:border-[#15334d]">
                    {parseInlineMarkdown(trimmed.slice(2))}
                </h1>
            );
            continue;
        }
        if (trimmed.startsWith('## ')) {
            elements.push(
                <h2 key={`h2-${i}`} className="text-sm font-bold mt-2.5 mb-1 text-slate-900 dark:text-white pb-0.5 border-b border-slate-200/60 dark:border-[#15334d]/60">
                    {parseInlineMarkdown(trimmed.slice(3))}
                </h2>
            );
            continue;
        }
        if (trimmed.startsWith('### ')) {
            elements.push(
                <h3 key={`h3-${i}`} className="text-xs font-bold mt-2 mb-1 text-slate-900 dark:text-white">
                    {parseInlineMarkdown(trimmed.slice(4))}
                </h3>
            );
            continue;
        }
        if (trimmed.startsWith('#### ')) {
            elements.push(
                <h4 key={`h4-${i}`} className="text-xs font-semibold mt-1.5 mb-0.5 text-slate-800 dark:text-slate-200">
                    {parseInlineMarkdown(trimmed.slice(5))}
                </h4>
            );
            continue;
        }

        // Standalone image line: ![alt](url)
        const singleImgMatch = trimmed.match(/^!\[(.*?)\]\((.*?)\)$/);
        if (singleImgMatch) {
            elements.push(
                <div key={`imgline-${i}`} className="my-2">
                    <img
                        src={singleImgMatch[2]}
                        alt={singleImgMatch[1] || ''}
                        className="max-h-60 max-w-full rounded-[5px] border border-slate-200 dark:border-[#15334d] object-contain bg-slate-100 dark:bg-[#071a27]"
                        onError={(e) => {
                            (e.target as HTMLElement).style.display = 'none';
                        }}
                    />
                    {singleImgMatch[1] && <div className="text-[10px] text-slate-400 mt-0.5">{singleImgMatch[1]}</div>}
                </div>
            );
            continue;
        }

        // Standard paragraph
        elements.push(
            <p key={`p-${i}`} className="text-xs text-slate-800 dark:text-slate-200 my-1 leading-relaxed">
                {parseInlineMarkdown(line)}
            </p>
        );
    }

    flushList(lines.length);
    flushBlockquote(lines.length);

    if (inCodeBlock && codeBuffer.length > 0) {
        elements.push(
            <pre key="unclosed-code" className="my-2 p-3 font-mono text-xs rounded-[5px] bg-slate-900 text-slate-100 overflow-x-auto leading-relaxed select-text">
                {codeBuffer.join('\n')}
            </pre>
        );
    }

    return <div className="space-y-1">{elements}</div>;
}

export default function PromptTreeViewModal({
    isOpen,
    onClose,
    instanceId,
    instanceName,
}: PromptTreeViewModalProps) {
    const [treeData, setTreeData] = useState<AgmProjectTreeNode[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [error, setError] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({});
    const [expandedConversations, setExpandedConversations] = useState<Record<string, boolean>>({});
    const [expandedUntitledGroups, setExpandedUntitledGroups] = useState<Record<string, boolean>>({});

    const [selectedProject, setSelectedProject] = useState<AgmProjectTreeNode | null>(null);
    const [selectedConversation, setSelectedConversation] = useState<AgmConversationNode | null>(null);
    const [selectedTurnNumber, setSelectedTurnNumber] = useState<number>(1);
    const [activePromptText, setActivePromptText] = useState<string>('');
    const [editedPromptText, setEditedPromptText] = useState<string>('');
    const [viewMode, setViewMode] = useState<ViewMode>('preview');
    const [showAllWords, setShowAllWords] = useState<boolean>(false);

    const [isCopied, setIsCopied] = useState(false);
    const [isResending, setIsResending] = useState(false);
    const [isEnqueueing, setIsEnqueueing] = useState(false);
    const [actionMsg, setActionMsg] = useState<string | null>(null);

    // Full screen prompt detail inspector
    const [inspectorPrompt, setInspectorPrompt] = useState<{
        title: string;
        text: string;
        convId: string;
        projectPath: string;
        images: string[];
        files: string[];
    } | null>(null);

    // Select conversation and initialize turn #1 / latest
    const selectConversation = useCallback((conv: AgmConversationNode, project: AgmProjectTreeNode, turnNumber = 1) => {
        setSelectedProject(project);
        setSelectedConversation(conv);
        setSelectedTurnNumber(turnNumber);
        const turns = getPromptTurns(conv);
        const activeTurn = turns.find((t) => t.turnNumber === turnNumber) || turns[turns.length - 1] || turns[0];
        const text = activeTurn?.text || conv.prompt_preview_200w || '';
        setActivePromptText(text);
        setEditedPromptText(text);
        setShowAllWords(false);
    }, []);

    // Select specific prompt turn
    const selectPromptTurn = useCallback((turn: PromptTurnNode, conv: AgmConversationNode, project: AgmProjectTreeNode) => {
        setSelectedProject(project);
        setSelectedConversation(conv);
        setSelectedTurnNumber(turn.turnNumber);
        setActivePromptText(turn.text);
        setEditedPromptText(turn.text);
        setShowAllWords(false);
    }, []);

    const loadTree = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 300,
                onlyRunning: false,
            });
            const relevant = instanceId
                ? data.filter(
                      (p) =>
                          !p.instance_id ||
                          p.instance_id === instanceId ||
                          (p.instance_id === 'default' && instanceId === 'default')
                  )
                : data;
            const finalData = relevant.length > 0 ? relevant : data;
            setTreeData(finalData);

            // Auto-expand projects
            const initialExpanded: Record<string, boolean> = {};
            finalData.forEach((p) => {
                initialExpanded[p.project_id] = true;
            });
            setExpandedProjects(initialExpanded);

            if (finalData.length > 0) {
                const firstProj = finalData[0];
                setSelectedProject(firstProj);
                if (firstProj.conversations.length > 0) {
                    const firstConv = firstProj.conversations[0];
                    selectConversation(firstConv, firstProj);
                    setExpandedConversations({ [firstConv.conversation_id]: true });
                }
            }
        } catch (err: any) {
            setError(err?.toString() || 'Failed to load project and conversation tree');
        } finally {
            setIsLoading(false);
        }
    };

    useEffect(() => {
        if (isOpen) {
            loadTree();
        }
    }, [isOpen, instanceId]);

    // Handle Backup Prompts
    const handleBackup = async () => {
        try {
            setActionMsg('Backing up prompts...');
            const backedUp = await invoke<any[]>('list_backed_up_prompts');
            const jsonStr = JSON.stringify(backedUp, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `prompts-backup-${instanceId || 'all'}-${Date.now()}.json`;
            a.click();
            URL.revokeObjectURL(url);
            setActionMsg('Prompts backed up successfully!');
            setTimeout(() => setActionMsg(null), 3000);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to backup prompts');
        }
    };

    // Handle Restore Prompts
    const handleRestore = async () => {
        try {
            setActionMsg('Restoring prompts with 7s channel stabilization...');
            await invoke('resume_recent_project_prompts', {
                instanceId: instanceId || 'default',
                maxAgeSeconds: 3600,
            });
            setActionMsg('Prompts restoration dispatched!');
            setTimeout(() => {
                setActionMsg(null);
                loadTree();
            }, 3000);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to restore prompts');
        }
    };

    // Handle Resend Prompt Action (Immediate injection / writes .antigravity_resume_task.json)
    const handleResendPrompt = async () => {
        if (!selectedConversation) return;
        try {
            setIsResending(true);
            setActionMsg('Resending & injecting prompt to running instance...');
            const promptContent = editedPromptText.trim() || activePromptText || selectedConversation.prompt_preview_200w || '';
            const repoPath = selectedProject?.repo_path || '';

            // 1. Write .antigravity_resume_task.json to project directory
            if (repoPath) {
                const taskPath = `${repoPath.replace(/[\\/]+$/, '')}/.antigravity_resume_task.json`;
                const payload = {
                    prompt_id: selectedConversation.conversation_id || `prompt-${Date.now()}`,
                    project_id: selectedProject?.project_id || '',
                    instance_id: selectedProject?.instance_id || instanceId || 'default',
                    repo_path: repoPath,
                    prompt_content: promptContent,
                    model: 'gemini-2.5-pro',
                    auto_boot: true,
                    status: 'dispatched',
                    resumed_at: Math.floor(Date.now() / 1000),
                };
                try {
                    await invoke('save_text_file', {
                        path: taskPath,
                        content: JSON.stringify(payload, null, 2),
                    });
                } catch (fsErr) {
                    console.warn('save_text_file resume error', fsErr);
                }
            }

            // 2. Trigger auto resume command
            try {
                await invoke('resume_recent_project_prompts', {
                    instanceId: selectedProject?.instance_id || instanceId || 'default',
                    maxAgeSeconds: 3600,
                });
            } catch {
                // non-fatal fallback
            }

            setActionMsg('Prompt resent & injected via .antigravity_resume_task.json!');
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to resend prompt');
        } finally {
            setIsResending(false);
        }
    };

    // Handle Enqueue Prompt Action (Calls FIFO scheduler queue / writes queue task)
    const handleEnqueuePrompt = async () => {
        if (!selectedConversation) return;
        try {
            setIsEnqueueing(true);
            setActionMsg('Enqueueing prompt into FIFO scheduler queue...');
            const promptContent = editedPromptText.trim() || activePromptText || selectedConversation.prompt_preview_200w || '';
            const repoPath = selectedProject?.repo_path || '';

            let enqueued = false;
            try {
                await invoke('enqueue_prompt', {
                    conversationId: selectedConversation.conversation_id,
                    projectId: selectedProject?.project_id,
                    instanceId: selectedProject?.instance_id || instanceId || 'default',
                    repoPath,
                    promptContent,
                });
                enqueued = true;
            } catch {
                // Fallback: write .antigravity_resume_task.json with queued status
                if (repoPath) {
                    const taskPath = `${repoPath.replace(/[\\/]+$/, '')}/.antigravity_resume_task.json`;
                    const payload = {
                        prompt_id: selectedConversation.conversation_id || `prompt-${Date.now()}`,
                        project_id: selectedProject?.project_id || '',
                        instance_id: selectedProject?.instance_id || instanceId || 'default',
                        repo_path: repoPath,
                        prompt_content: promptContent,
                        model: 'gemini-2.5-pro',
                        auto_boot: false,
                        status: 'queued',
                        queued_at: Math.floor(Date.now() / 1000),
                    };
                    try {
                        await invoke('save_text_file', {
                            path: taskPath,
                            content: JSON.stringify(payload, null, 2),
                        });
                        enqueued = true;
                    } catch (fsErr) {
                        console.warn('save_text_file queue error', fsErr);
                    }
                }
            }

            setActionMsg(enqueued ? 'Prompt enqueued into FIFO scheduler queue!' : 'Prompt recorded for queue scheduler!');
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to enqueue prompt');
        } finally {
            setIsEnqueueing(false);
        }
    };

    // Extract images and file paths from text
    const extractAssets = (text: string) => {
        const images: string[] = [];
        const files: string[] = [];

        const imgRegex = /!\[.*?\]\((https?:\/\/.*?|data:image\/.*?;base64,.*?)\)/g;
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
    };

    // Open Full-Screen Inspector
    const openInspector = (conv: AgmConversationNode, repoPath: string) => {
        const text = activePromptText || conv.prompt_preview_200w || 'No prompt content preview available for this conversation.';
        const { images, files } = extractAssets(text);
        setInspectorPrompt({
            title: conv.title || conv.short_id,
            text,
            convId: conv.conversation_id,
            projectPath: repoPath,
            images,
            files,
        });
    };

    // Search filter across projects and conversations
    const filteredProjects = useMemo(() => {
        if (!searchQuery.trim()) return treeData;
        const q = searchQuery.toLowerCase();
        return treeData.filter(
            (p) =>
                p.repo_name.toLowerCase().includes(q) ||
                p.repo_path.toLowerCase().includes(q) ||
                p.conversations.some(
                    (c) =>
                        c.title.toLowerCase().includes(q) ||
                        c.conversation_id.toLowerCase().includes(q) ||
                        c.prompt_preview_200w.toLowerCase().includes(q)
                )
        );
    }, [treeData, searchQuery]);

    // Active prompt truncation for Preview mode
    const { displayText: previewDisplayText, isTruncated, totalWords } = useMemo(() => {
        return getTruncatedText(activePromptText, 500);
    }, [activePromptText]);

    const displayedMarkdown = showAllWords || !isTruncated ? activePromptText : previewDisplayText;
    const activeWordCount = totalWords || countWords(activePromptText);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-[200] flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 sm:p-6 animate-in fade-in duration-200">
            <div className="flex h-[88vh] w-full max-w-6xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl mt-12 sm:mt-14">
                {/* Modal Header */}
                <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#15334d] px-6 py-4 bg-slate-50/80 dark:bg-[#071a27] shrink-0">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-[5px] bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20">
                            <Layers className="h-5 w-5" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <h2 className="text-base font-bold text-slate-900 dark:text-white">
                                    Project & Prompt Tree View
                                </h2>
                                <span className="rounded-[5px] bg-blue-100 px-2.5 py-0.5 text-[11px] font-semibold text-blue-800 dark:bg-blue-950/60 dark:text-cyan-300 border border-blue-300/40">
                                    {instanceName || instanceId || 'All Instances'}
                                </span>
                            </div>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                                Inspect 3-layer project, conversation & prompt hierarchies, preview markdown, and dispatch tasks
                            </p>
                        </div>
                    </div>

                    {/* Toolbar Capsule */}
                    <div className="flex items-center gap-2">
                        <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                            <button
                                type="button"
                                onClick={handleBackup}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-[5px] transition-colors cursor-pointer"
                                title="Backup Prompts to JSON"
                            >
                                <Download className="w-3.5 h-3.5 text-indigo-500" />
                                <span>Backup</span>
                            </button>
                            <button
                                type="button"
                                onClick={handleRestore}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-[5px] transition-colors cursor-pointer"
                                title="Restore Running Prompts (7s stabilization delay)"
                            >
                                <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Restore</span>
                            </button>
                            <button
                                type="button"
                                onClick={loadTree}
                                disabled={isLoading}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-[5px] transition-colors cursor-pointer"
                                title="Refresh Tree"
                            >
                                <RefreshCw className={cn('w-3.5 h-3.5 text-blue-500', isLoading && 'animate-spin')} />
                                <span>Refresh</span>
                            </button>
                        </div>

                        <button
                            type="button"
                            onClick={onClose}
                            className="rounded-[5px] p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#15334d] dark:hover:text-slate-200 transition-colors cursor-pointer border border-transparent hover:border-slate-200 dark:hover:border-[#15334d]"
                            title="Close"
                        >
                            <X className="h-5 w-5" />
                        </button>
                    </div>
                </div>

                {/* Notification Banner */}
                {actionMsg && (
                    <div className="bg-emerald-500/10 border-b border-emerald-500/20 px-6 py-2 text-xs font-medium text-emerald-700 dark:text-emerald-300 flex items-center gap-2 shrink-0">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                        {actionMsg}
                    </div>
                )}
                {error && (
                    <div className="bg-rose-500/10 border-b border-rose-500/20 px-6 py-2 text-xs font-medium text-rose-700 dark:text-rose-300 shrink-0">
                        {error}
                    </div>
                )}

                {/* Main Split View */}
                <div className="flex flex-1 overflow-hidden">
                    {/* Left Panel: 3-Layer Project -> Conversation -> Prompt Tree */}
                    <div className="w-80 sm:w-96 flex flex-col border-r border-slate-200 dark:border-[#15334d] bg-slate-50/50 dark:bg-[#071a27]/60">
                        {/* Search Input */}
                        <div className="p-3 border-b border-slate-200 dark:border-[#15334d] shrink-0">
                            <div className="relative">
                                <Search className="absolute left-3 top-2.5 h-3.5 w-3.5 text-slate-400" />
                                <input
                                    type="text"
                                    placeholder="Search projects, convos, or prompts..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className="w-full rounded-[5px] bg-white dark:bg-[#0c2438] pl-9 pr-3 py-1.5 text-xs text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                                />
                            </div>
                        </div>

                        {/* Tree Items List */}
                        <div className="flex-1 overflow-y-auto p-3 space-y-1">
                            {filteredProjects.length === 0 ? (
                                <div className="py-12 text-center text-xs text-slate-400">
                                    No projects or conversations found.
                                </div>
                            ) : (
                                filteredProjects.map((project) => {
                                    const isProjectExpanded = Boolean(expandedProjects[project.project_id]);
                                    const isProjectSelected = selectedProject?.project_id === project.project_id;

                                    // Calculate total prompt count for Layer 1
                                    const totalProjectPrompts = project.conversations.reduce(
                                        (sum, c) => sum + (c.step_count > 0 ? c.step_count : 1),
                                        0
                                    );

                                    // Partition into normal conversations vs untitled/empty conversations
                                    const normalConversations: AgmConversationNode[] = [];
                                    const untitledConversations: AgmConversationNode[] = [];

                                    project.conversations.forEach((conv) => {
                                        if (isUntitledOrEmpty(conv)) {
                                            untitledConversations.push(conv);
                                        } else {
                                            normalConversations.push(conv);
                                        }
                                    });

                                    const isUntitledGroupExpanded = Boolean(expandedUntitledGroups[project.project_id]);

                                    return (
                                        <div key={project.project_id} className="space-y-0.5">
                                            {/* Layer 1: Project Row */}
                                            <div
                                                onClick={() => {
                                                    setSelectedProject(project);
                                                    setExpandedProjects((prev) => ({
                                                        ...prev,
                                                        [project.project_id]: !isProjectExpanded,
                                                    }));
                                                }}
                                                className={cn(
                                                    'group flex items-center justify-between rounded-[5px] px-2.5 py-1.5 text-xs font-semibold cursor-pointer transition-colors',
                                                    isProjectSelected
                                                        ? 'bg-blue-100/70 text-blue-900 dark:bg-blue-950/60 dark:text-cyan-300'
                                                        : 'text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#0c2438]'
                                                )}
                                                title={`Project: ${project.repo_name} (${totalProjectPrompts} prompts total)`}
                                            >
                                                <div className="flex items-center gap-1.5 min-w-0">
                                                    {isProjectExpanded ? (
                                                        <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                    ) : (
                                                        <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                    )}
                                                    <Folder className="h-4 w-4 text-amber-500 shrink-0" />
                                                    <span className="truncate">{project.repo_name}</span>
                                                </div>
                                                <span
                                                    className="rounded-[5px] bg-slate-200 dark:bg-[#15334d] px-2 py-0.5 text-[10px] font-mono text-slate-600 dark:text-slate-300 shrink-0"
                                                    title={`${totalProjectPrompts} total prompt(s)`}
                                                >
                                                    {totalProjectPrompts} prompts
                                                </span>
                                            </div>

                                            {/* Layer 2: Conversations List */}
                                            {isProjectExpanded && (
                                                <div className="ml-4 pl-2 border-l border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                                                    {/* Normal Conversations */}
                                                    {normalConversations.map((conv) => {
                                                        const isConvSelected = selectedConversation?.conversation_id === conv.conversation_id;
                                                        const isConvExpanded = Boolean(expandedConversations[conv.conversation_id]);
                                                        const promptTurns = getPromptTurns(conv);

                                                        return (
                                                            <div key={conv.conversation_id} className="space-y-0.5">
                                                                {/* Conversation Row */}
                                                                <div
                                                                    onClick={() => {
                                                                        selectConversation(conv, project, 1);
                                                                        setExpandedConversations((prev) => ({
                                                                            ...prev,
                                                                            [conv.conversation_id]: !isConvExpanded,
                                                                        }));
                                                                    }}
                                                                    onDoubleClick={() => openInspector(conv, project.repo_path)}
                                                                    className={cn(
                                                                        'group flex items-center justify-between rounded-[5px] px-2 py-1.5 text-xs cursor-pointer transition-colors',
                                                                        isConvSelected
                                                                            ? 'bg-blue-600 text-white font-medium shadow-2xs'
                                                                            : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#0c2438]'
                                                                    )}
                                                                    title="Click to view prompts; double-click for Full inspector"
                                                                >
                                                                    <div className="flex items-center gap-1.5 min-w-0">
                                                                        {isConvExpanded ? (
                                                                            <ChevronDown className={cn('h-3 w-3 shrink-0', isConvSelected ? 'text-white' : 'text-slate-400')} />
                                                                        ) : (
                                                                            <ChevronRight className={cn('h-3 w-3 shrink-0', isConvSelected ? 'text-white' : 'text-slate-400')} />
                                                                        )}
                                                                        <MessageSquare className={cn('h-3.5 w-3.5 shrink-0', isConvSelected ? 'text-white' : 'text-blue-500 opacity-70')} />
                                                                        <span className="truncate text-[11px]">
                                                                            {conv.title || conv.short_id}
                                                                        </span>
                                                                    </div>
                                                                    <div className="flex items-center gap-1.5 shrink-0">
                                                                        <span
                                                                            className={cn(
                                                                                'text-[9px] font-mono px-1 rounded-[3px]',
                                                                                isConvSelected
                                                                                    ? 'bg-blue-700/80 text-white'
                                                                                    : 'bg-slate-200 dark:bg-[#15334d] text-slate-500 dark:text-slate-400'
                                                                            )}
                                                                        >
                                                                            {conv.step_count || 1} stp
                                                                        </span>
                                                                        {conv.is_running && (
                                                                            <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-ping" title="Running" />
                                                                        )}
                                                                    </div>
                                                                </div>

                                                                {/* Layer 3: Prompts Inside Conversation */}
                                                                {isConvExpanded && (
                                                                    <div className="ml-5 pl-2 border-l border-slate-200 dark:border-[#15334d]/50 space-y-0.5 py-0.5">
                                                                        {promptTurns.map((turn) => {
                                                                            const isTurnSelected =
                                                                                selectedConversation?.conversation_id === conv.conversation_id &&
                                                                                selectedTurnNumber === turn.turnNumber;

                                                                            return (
                                                                                <div
                                                                                    key={turn.turnNumber}
                                                                                    onClick={(e) => {
                                                                                        e.stopPropagation();
                                                                                        selectPromptTurn(turn, conv, project);
                                                                                    }}
                                                                                    className={cn(
                                                                                        'group flex items-center justify-between rounded-[5px] px-2 py-1 text-[11px] cursor-pointer transition-colors',
                                                                                        isTurnSelected
                                                                                            ? 'bg-cyan-600 text-white font-semibold shadow-2xs'
                                                                                            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-[#0c2438]'
                                                                                    )}
                                                                                    title={`Click to load ${turn.title} (${turn.wordCount} words)`}
                                                                                >
                                                                                    <div className="flex items-center gap-1.5 min-w-0">
                                                                                        <Sparkles className="w-3 h-3 text-amber-400 shrink-0" />
                                                                                        <span className="truncate">{turn.title}</span>
                                                                                    </div>
                                                                                    <span
                                                                                        className={cn(
                                                                                            'text-[10px] font-mono shrink-0 px-1 rounded-[3px]',
                                                                                            isTurnSelected
                                                                                                ? 'text-cyan-100'
                                                                                                : 'text-slate-400 dark:text-slate-500'
                                                                                        )}
                                                                                    >
                                                                                        {turn.wordCount}w
                                                                                    </span>
                                                                                </div>
                                                                            );
                                                                        })}
                                                                    </div>
                                                                )}
                                                            </div>
                                                        );
                                                    })}

                                                    {/* Group Untitled & Empty Conversations Under Collapsible Node */}
                                                    {untitledConversations.length > 0 && (
                                                        <div className="space-y-0.5 mt-1 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/40">
                                                            <div
                                                                onClick={() => {
                                                                    setExpandedUntitledGroups((prev) => ({
                                                                        ...prev,
                                                                        [project.project_id]: !isUntitledGroupExpanded,
                                                                    }));
                                                                }}
                                                                className="group flex items-center justify-between rounded-[5px] px-2 py-1 text-xs font-medium text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438] cursor-pointer transition-colors"
                                                                title="Toggle Untitled Conversations"
                                                            >
                                                                <div className="flex items-center gap-1.5 min-w-0">
                                                                    {isUntitledGroupExpanded ? (
                                                                        <ChevronDown className="h-3 w-3 text-slate-400 shrink-0" />
                                                                    ) : (
                                                                        <ChevronRight className="h-3 w-3 text-slate-400 shrink-0" />
                                                                    )}
                                                                    <Folder className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                                    <span className="truncate">
                                                                        Untitled Conversations ({untitledConversations.length})
                                                                    </span>
                                                                </div>
                                                                <span className="text-[10px] font-mono text-slate-400">
                                                                    {untitledConversations.length}
                                                                </span>
                                                            </div>

                                                            {/* Untitled Conversations Sublist */}
                                                            {isUntitledGroupExpanded && (
                                                                <div className="ml-4 pl-2 border-l border-dashed border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                                                                    {untitledConversations.map((conv) => {
                                                                        const isConvSelected = selectedConversation?.conversation_id === conv.conversation_id;
                                                                        const isConvExpanded = Boolean(expandedConversations[conv.conversation_id]);
                                                                        const promptTurns = getPromptTurns(conv);

                                                                        return (
                                                                            <div key={conv.conversation_id} className="space-y-0.5">
                                                                                <div
                                                                                    onClick={() => {
                                                                                        selectConversation(conv, project, 1);
                                                                                        setExpandedConversations((prev) => ({
                                                                                            ...prev,
                                                                                            [conv.conversation_id]: !isConvExpanded,
                                                                                        }));
                                                                                    }}
                                                                                    onDoubleClick={() => openInspector(conv, project.repo_path)}
                                                                                    className={cn(
                                                                                        'group flex items-center justify-between rounded-[5px] px-2 py-1 text-xs cursor-pointer transition-colors',
                                                                                        isConvSelected
                                                                                            ? 'bg-blue-600 text-white font-medium shadow-2xs'
                                                                                            : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#0c2438]'
                                                                                    )}
                                                                                    title="Click to view prompts; double click for Full inspector"
                                                                                >
                                                                                    <div className="flex items-center gap-1.5 min-w-0">
                                                                                        {isConvExpanded ? (
                                                                                            <ChevronDown className={cn('h-3 w-3 shrink-0', isConvSelected ? 'text-white' : 'text-slate-400')} />
                                                                                        ) : (
                                                                                            <ChevronRight className={cn('h-3 w-3 shrink-0', isConvSelected ? 'text-white' : 'text-slate-400')} />
                                                                                        )}
                                                                                        <MessageSquare className={cn('h-3.5 w-3.5 shrink-0', isConvSelected ? 'text-white' : 'text-slate-400')} />
                                                                                        <span className="truncate text-[11px]">
                                                                                            {conv.title || conv.short_id || conv.conversation_id.slice(0, 8)}
                                                                                        </span>
                                                                                    </div>
                                                                                    <span className="text-[9px] font-mono opacity-70">
                                                                                        {conv.step_count || 1} stp
                                                                                    </span>
                                                                                </div>

                                                                                {/* Layer 3 Prompts for Untitled Conversation */}
                                                                                {isConvExpanded && (
                                                                                    <div className="ml-5 pl-2 border-l border-slate-200 dark:border-[#15334d]/50 space-y-0.5 py-0.5">
                                                                                        {promptTurns.map((turn) => {
                                                                                            const isTurnSelected =
                                                                                                selectedConversation?.conversation_id === conv.conversation_id &&
                                                                                                selectedTurnNumber === turn.turnNumber;

                                                                                            return (
                                                                                                <div
                                                                                                    key={turn.turnNumber}
                                                                                                    onClick={(e) => {
                                                                                                        e.stopPropagation();
                                                                                                        selectPromptTurn(turn, conv, project);
                                                                                                    }}
                                                                                                    className={cn(
                                                                                                        'group flex items-center justify-between rounded-[5px] px-2 py-1 text-[11px] cursor-pointer transition-colors',
                                                                                                        isTurnSelected
                                                                                                            ? 'bg-cyan-600 text-white font-semibold shadow-2xs'
                                                                                                            : 'text-slate-600 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-[#0c2438]'
                                                                                                    )}
                                                                                                    title={`Click to load ${turn.title} (${turn.wordCount} words)`}
                                                                                                >
                                                                                                    <div className="flex items-center gap-1.5 min-w-0">
                                                                                                        <Sparkles className="w-3 h-3 text-amber-400 shrink-0" />
                                                                                                        <span className="truncate">{turn.title}</span>
                                                                                                    </div>
                                                                                                    <span
                                                                                                        className={cn(
                                                                                                            'text-[10px] font-mono shrink-0 px-1 rounded-[3px]',
                                                                                                            isTurnSelected
                                                                                                                ? 'text-cyan-100'
                                                                                                                : 'text-slate-400 dark:text-slate-500'
                                                                                                        )}
                                                                                                    >
                                                                                                        {turn.wordCount}w
                                                                                                    </span>
                                                                                                </div>
                                                                                            );
                                                                                        })}
                                                                                    </div>
                                                                                )}
                                                                            </div>
                                                                        );
                                                                    })}
                                                                </div>
                                                            )}
                                                        </div>
                                                    )}
                                                </div>
                                            )}
                                        </div>
                                    );
                                })
                            )}
                        </div>
                    </div>

                    {/* Right Panel: Prompt Details, Markdown Modes & Actions */}
                    <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-[#0c2438]">
                        {selectedConversation ? (
                            <div className="flex-1 flex flex-col p-6 overflow-y-auto space-y-5">
                                {/* Prompt Content Header */}
                                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-4 border-b border-slate-200 dark:border-[#15334d] shrink-0">
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <h3 className="text-base font-bold text-slate-900 dark:text-white">
                                                {selectedConversation.title || selectedConversation.short_id}
                                            </h3>
                                            <span
                                                className={cn(
                                                    'px-2 py-0.5 rounded-[5px] text-[10px] font-bold border',
                                                    selectedConversation.is_running
                                                        ? 'bg-emerald-50 text-emerald-700 border-emerald-300 dark:bg-emerald-950/60 dark:text-emerald-300'
                                                        : 'bg-slate-100 text-slate-600 border-slate-200 dark:bg-[#071a27] dark:text-slate-400 dark:border-[#15334d]'
                                                )}
                                            >
                                                {selectedConversation.is_running ? 'RUNNING' : 'QUEUED / IDLE'}
                                            </span>
                                            <span className="rounded-[5px] bg-blue-50 dark:bg-[#071a27] text-blue-700 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] px-2 py-0.5 text-[10px] font-mono">
                                                Turn #{selectedTurnNumber}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-3 text-xs text-slate-500 dark:text-slate-400 mt-1 font-mono">
                                            <span>Conv ID: {selectedConversation.conversation_id}</span>
                                            <span>·</span>
                                            <span>Modified: {selectedConversation.last_modified}</span>
                                        </div>
                                    </div>

                                    {/* Action Buttons: Resend, Enqueue, Full */}
                                    <div className="flex items-center gap-2">
                                        {/* Resend Button */}
                                        <button
                                            type="button"
                                            onClick={handleResendPrompt}
                                            disabled={isResending}
                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-emerald-50 dark:bg-[#0c2438] text-emerald-700 dark:text-emerald-300 border border-emerald-300/60 dark:border-[#15334d] hover:bg-emerald-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer disabled:opacity-50"
                                            title="Immediately inject prompt to running instance (writes .antigravity_resume_task.json)"
                                        >
                                            <RotateCw className={cn('w-3.5 h-3.5 text-emerald-500', isResending && 'animate-spin')} />
                                            <span>Resend</span>
                                        </button>

                                        {/* Enqueue Button */}
                                        <button
                                            type="button"
                                            onClick={handleEnqueuePrompt}
                                            disabled={isEnqueueing}
                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-purple-50 dark:bg-[#0c2438] text-purple-700 dark:text-purple-300 border border-purple-300/60 dark:border-[#15334d] hover:bg-purple-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer disabled:opacity-50"
                                            title="Enqueue prompt into FIFO scheduler queue"
                                        >
                                            <ListPlus className="w-3.5 h-3.5 text-purple-500" />
                                            <span>Enqueue</span>
                                        </button>

                                        {/* Full Button */}
                                        <button
                                            type="button"
                                            onClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-blue-50 dark:bg-[#0c2438] text-blue-700 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] text-xs font-semibold hover:bg-blue-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                            title="Full-Screen Inspector"
                                        >
                                            <Maximize2 className="h-3.5 w-3.5" />
                                            <span>Full</span>
                                        </button>
                                    </div>
                                </div>

                                {/* View Mode Tabs & Word Count */}
                                <div>
                                    <div className="flex items-center justify-between mb-3">
                                        <div className="flex items-center gap-2">
                                            <label className="text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider">
                                                Prompt Instruction ({activeWordCount} words)
                                            </label>
                                        </div>

                                        {/* View Mode Toggle Tabs */}
                                        <div className="flex items-center rounded-[5px] bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d] p-0.5">
                                            <button
                                                type="button"
                                                onClick={() => setViewMode('preview')}
                                                className={cn(
                                                    'flex items-center gap-1 px-3 py-1 text-xs font-medium rounded-[5px] transition-colors cursor-pointer',
                                                    viewMode === 'preview'
                                                        ? 'bg-white dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 font-semibold shadow-2xs border border-slate-200/60 dark:border-[#15334d]'
                                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                                )}
                                                title="Rich Markdown Preview"
                                            >
                                                <Eye className="w-3.5 h-3.5" />
                                                <span>Preview</span>
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => setViewMode('raw')}
                                                className={cn(
                                                    'flex items-center gap-1 px-3 py-1 text-xs font-medium rounded-[5px] transition-colors cursor-pointer',
                                                    viewMode === 'raw'
                                                        ? 'bg-white dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 font-semibold shadow-2xs border border-slate-200/60 dark:border-[#15334d]'
                                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                                )}
                                                title="Plain Monospace Raw View"
                                            >
                                                <Code className="w-3.5 h-3.5" />
                                                <span>Raw</span>
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => setViewMode('edit')}
                                                className={cn(
                                                    'flex items-center gap-1 px-3 py-1 text-xs font-medium rounded-[5px] transition-colors cursor-pointer',
                                                    viewMode === 'edit'
                                                        ? 'bg-white dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 font-semibold shadow-2xs border border-slate-200/60 dark:border-[#15334d]'
                                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-white'
                                                )}
                                                title="Editable Textarea Mode"
                                            >
                                                <Edit3 className="w-3.5 h-3.5" />
                                                <span>Edit</span>
                                            </button>
                                        </div>
                                    </div>

                                    {/* Tab 1: Preview Mode (Rich Markdown) */}
                                    {viewMode === 'preview' && (
                                        <div className="space-y-3">
                                            <div
                                                onDoubleClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                                className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-800 dark:text-slate-200 leading-relaxed max-h-[460px] overflow-y-auto cursor-pointer hover:border-blue-400 dark:hover:border-blue-500/50 transition-colors shadow-inner"
                                            >
                                                <RichMarkdownRenderer content={displayedMarkdown} />
                                            </div>

                                            {/* Truncation & Copy Controls */}
                                            <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                                                <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                                                    {isTruncated && !showAllWords
                                                        ? `Truncated at 500 words (${totalWords} total)`
                                                        : `Showing all ${activeWordCount} words`}
                                                </div>

                                                <div className="flex items-center gap-2">
                                                    {isTruncated && (
                                                        <button
                                                            type="button"
                                                            onClick={() => setShowAllWords(!showAllWords)}
                                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-blue-50 dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] hover:bg-blue-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                            title={showAllWords ? 'Collapse preview to 500 words' : 'Expand full prompt'}
                                                        >
                                                            {showAllWords ? (
                                                                <>
                                                                    <ChevronUp className="w-3.5 h-3.5" />
                                                                    <span>Show Less (500w)</span>
                                                                </>
                                                            ) : (
                                                                <>
                                                                    <ChevronDown className="w-3.5 h-3.5" />
                                                                    <span>Show All ({totalWords}w)</span>
                                                                </>
                                                            )}
                                                        </button>
                                                    )}

                                                    <button
                                                        type="button"
                                                        onClick={() => {
                                                            navigator.clipboard.writeText(activePromptText);
                                                            setIsCopied(true);
                                                            setTimeout(() => setIsCopied(false), 2000);
                                                        }}
                                                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                        title="Copy full prompt text to clipboard"
                                                    >
                                                        {isCopied ? (
                                                            <>
                                                                <Check className="w-3.5 h-3.5 text-emerald-500" />
                                                                <span>Copied Full Prompt!</span>
                                                            </>
                                                        ) : (
                                                            <>
                                                                <Copy className="w-3.5 h-3.5" />
                                                                <span>Copy Full Prompt</span>
                                                            </>
                                                        )}
                                                    </button>
                                                </div>
                                            </div>
                                        </div>
                                    )}

                                    {/* Tab 2: Raw Mode (Monospace Clean Text) */}
                                    {viewMode === 'raw' && (
                                        <div className="space-y-3">
                                            <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 font-mono text-xs text-slate-800 dark:text-slate-200 leading-relaxed whitespace-pre-wrap break-words max-h-[460px] overflow-y-auto select-text shadow-inner">
                                                {activePromptText || 'No prompt content recorded.'}
                                            </div>
                                            <div className="flex items-center justify-end pt-1">
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        navigator.clipboard.writeText(activePromptText);
                                                        setIsCopied(true);
                                                        setTimeout(() => setIsCopied(false), 2000);
                                                    }}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                    title="Copy raw text to clipboard"
                                                >
                                                    {isCopied ? (
                                                        <>
                                                            <Check className="w-3.5 h-3.5 text-emerald-500" />
                                                            <span>Copied Raw!</span>
                                                        </>
                                                    ) : (
                                                        <>
                                                            <Copy className="w-3.5 h-3.5" />
                                                            <span>Copy Raw</span>
                                                        </>
                                                    )}
                                                </button>
                                            </div>
                                        </div>
                                    )}

                                    {/* Tab 3: Edit Mode (Editable Textarea) */}
                                    {viewMode === 'edit' && (
                                        <div className="space-y-3">
                                            <div className="flex items-center justify-between text-xs text-slate-500 dark:text-slate-400 font-mono">
                                                <span>Modify prompt instruction directly:</span>
                                                <span>
                                                    {editedPromptText.length} chars · {countWords(editedPromptText)} words
                                                </span>
                                            </div>
                                            <textarea
                                                value={editedPromptText}
                                                onChange={(e) => setEditedPromptText(e.target.value)}
                                                className="w-full h-80 rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-4 font-mono text-xs text-slate-900 dark:text-slate-100 leading-relaxed focus:outline-none focus:ring-2 focus:ring-blue-500/40 resize-y"
                                                placeholder="Type or modify prompt instruction here..."
                                            />
                                            <div className="flex items-center justify-between pt-1">
                                                <button
                                                    type="button"
                                                    onClick={() => setEditedPromptText(activePromptText)}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer"
                                                    title="Reset to original text"
                                                >
                                                    <RotateCcw className="w-3.5 h-3.5 text-slate-400" />
                                                    <span>Reset</span>
                                                </button>
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        setActivePromptText(editedPromptText);
                                                        setActionMsg('Prompt changes saved to current session!');
                                                        setTimeout(() => setActionMsg(null), 2500);
                                                    }}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-emerald-50 dark:bg-[#0c2438] text-emerald-700 dark:text-emerald-300 border border-emerald-300/60 dark:border-[#15334d] hover:bg-emerald-100 dark:hover:bg-[#15334d] text-xs font-semibold transition-colors cursor-pointer"
                                                    title="Save prompt changes"
                                                >
                                                    <Check className="w-3.5 h-3.5 text-emerald-500" />
                                                    <span>Save Changes</span>
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        ) : (
                            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 text-slate-400">
                                <Layers className="h-12 w-12 stroke-1 mb-3 opacity-40" />
                                <p className="text-sm font-medium">Select a project, conversation, or prompt turn from the left tree</p>
                                <p className="text-xs mt-1">
                                    Expand conversation nodes to access specific prompt turns, preview rich Markdown, or resend instructions.
                                </p>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Full-Screen Prompt Inspector Modal */}
            {inspectorPrompt && (
                <div className="fixed inset-0 z-[220] flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
                    <div className="flex h-[92vh] w-full max-w-5xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl">
                        {/* Inspector Header */}
                        <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#15334d] px-6 py-4 bg-slate-50 dark:bg-[#071a27] shrink-0">
                            <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-[5px] bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/20">
                                    <FileText className="h-5 w-5" />
                                </div>
                                <div>
                                    <h3 className="text-base font-bold text-slate-900 dark:text-white">
                                        {inspectorPrompt.title}
                                    </h3>
                                    <div className="flex items-center gap-2 text-xs text-slate-500 dark:text-slate-400 font-mono mt-0.5">
                                        <span>Conv: {inspectorPrompt.convId}</span>
                                        <span>·</span>
                                        <span className="truncate max-w-md">{inspectorPrompt.projectPath}</span>
                                    </div>
                                </div>
                            </div>

                            <div className="flex items-center gap-2">
                                <button
                                    type="button"
                                    onClick={() => {
                                        navigator.clipboard.writeText(inspectorPrompt.text);
                                        setIsCopied(true);
                                        setTimeout(() => setIsCopied(false), 2000);
                                    }}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] bg-slate-100 hover:bg-slate-200 dark:bg-[#15334d] dark:hover:bg-[#1c4566] text-xs font-semibold text-slate-700 dark:text-slate-200 transition-colors cursor-pointer border border-slate-200 dark:border-[#15334d]"
                                    title="Copy full text"
                                >
                                    {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                                    <span>{isCopied ? 'Copied!' : 'Copy Text'}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setInspectorPrompt(null)}
                                    className="rounded-[5px] p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#15334d] dark:hover:text-slate-200 transition-colors cursor-pointer border border-transparent hover:border-slate-200 dark:hover:border-[#15334d]"
                                    title="Close Inspector"
                                >
                                    <X className="h-5 w-5" />
                                </button>
                            </div>
                        </div>

                        {/* Inspector Body */}
                        <div className="flex-1 overflow-y-auto p-6 space-y-6">
                            {/* Images Gallery */}
                            {inspectorPrompt.images.length > 0 && (
                                <div className="space-y-2">
                                    <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                                        <ImageIcon className="h-4 w-4 text-blue-500" />
                                        <span>Embedded Images ({inspectorPrompt.images.length})</span>
                                    </div>
                                    <div className="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-3">
                                        {inspectorPrompt.images.map((img, i) => (
                                            <div
                                                key={i}
                                                className="overflow-hidden rounded-[5px] border border-slate-200 dark:border-[#15334d] bg-slate-100 dark:bg-[#071a27] p-2"
                                            >
                                                <img
                                                    src={img}
                                                    alt={`Prompt Image ${i + 1}`}
                                                    className="w-full h-44 object-contain rounded-[4px]"
                                                    onError={(e) => {
                                                        (e.target as HTMLElement).style.display = 'none';
                                                    }}
                                                />
                                                <div className="mt-1.5 truncate text-[10px] font-mono text-slate-500 dark:text-slate-400">
                                                    {img.substring(0, 60)}...
                                                </div>
                                            </div>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Referenced Files */}
                            {inspectorPrompt.files.length > 0 && (
                                <div className="space-y-2">
                                    <div className="flex items-center gap-2 text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                                        <Folder className="h-4 w-4 text-amber-500" />
                                        <span>Referenced Files ({inspectorPrompt.files.length})</span>
                                    </div>
                                    <div className="flex flex-wrap gap-2">
                                        {inspectorPrompt.files.map((file, i) => (
                                            <span
                                                key={i}
                                                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-[5px] bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d] text-xs font-mono text-slate-700 dark:text-slate-300"
                                            >
                                                <FileText className="w-3 h-3 text-slate-400" />
                                                <span className="truncate max-w-xs">{file}</span>
                                            </span>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Full Prompt Text (Rendered Markdown) */}
                            <div className="space-y-2">
                                <label className="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                                    Full Prompt Instruction
                                </label>
                                <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-900 dark:text-slate-100 leading-relaxed max-h-[600px] overflow-y-auto select-text shadow-inner">
                                    <RichMarkdownRenderer content={inspectorPrompt.text} />
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
