import { useState, useEffect, useMemo } from 'react';
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
    ChevronDown
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

interface PromptTreeViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
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
    const [selectedProject, setSelectedProject] = useState<AgmProjectTreeNode | null>(null);
    const [selectedConversation, setSelectedConversation] = useState<AgmConversationNode | null>(null);

    // Full screen prompt detail inspector
    const [inspectorPrompt, setInspectorPrompt] = useState<{
        title: string;
        text: string;
        convId: string;
        projectPath: string;
        images: string[];
        files: string[];
    } | null>(null);
    const [isCopied, setIsCopied] = useState(false);
    const [actionMsg, setActionMsg] = useState<string | null>(null);

    const loadTree = async () => {
        setIsLoading(true);
        setError(null);
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                maxWords: 300,
                onlyRunning: false,
            });
            // Filter to current instance if instanceId provided, or show all
            const relevant = instanceId
                ? data.filter((p) => !p.instance_id || p.instance_id === instanceId || p.instance_id === 'default' && instanceId === 'default')
                : data;
            setTreeData(relevant.length > 0 ? relevant : data);

            // Auto-expand projects
            const initialExpanded: Record<string, boolean> = {};
            data.forEach((p) => {
                initialExpanded[p.project_id] = true;
            });
            setExpandedProjects(initialExpanded);

            if (data.length > 0) {
                setSelectedProject(data[0]);
                if (data[0].conversations.length > 0) {
                    setSelectedConversation(data[0].conversations[0]);
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

    // Extract images and file paths from text
    const extractAssets = (text: string) => {
        const images: string[] = [];
        const files: string[] = [];

        // Match base64 images or image urls
        const imgRegex = /!\[.*?\]\((https?:\/\/.*?|data:image\/.*?;base64,.*?)\)/g;
        let match;
        while ((match = imgRegex.exec(text)) !== null) {
            images.push(match[1]);
        }

        // Match file paths (e.g. file:///... or [name](file://...))
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
        const text = conv.prompt_preview_200w || 'No prompt content preview available for this conversation.';
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

    const filteredProjects = useMemo(() => {
        if (!searchQuery.trim()) return treeData;
        const q = searchQuery.toLowerCase();
        return treeData.filter(
            (p) =>
                p.repo_name.toLowerCase().includes(q) ||
                p.repo_path.toLowerCase().includes(q) ||
                p.conversations.some((c) => c.title.toLowerCase().includes(q) || c.conversation_id.toLowerCase().includes(q))
        );
    }, [treeData, searchQuery]);

    if (!isOpen) return null;

    return (
        <div className="fixed inset-0 z-50 flex items-center justify-center bg-black/60 backdrop-blur-xs p-4 sm:p-6 animate-in fade-in duration-200">
            <div className="flex h-[90vh] w-full max-w-6xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl">
                {/* Modal Header */}
                <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#15334d] px-6 py-4 bg-slate-50/80 dark:bg-[#071a27]">
                    <div className="flex items-center gap-3">
                        <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20">
                            <Layers className="h-5 w-5" />
                        </div>
                        <div>
                            <div className="flex items-center gap-2">
                                <h2 className="text-base font-bold text-slate-900 dark:text-white">
                                    Project & Prompt Tree View
                                </h2>
                                <span className="rounded-full bg-blue-100 px-2.5 py-0.5 text-[11px] font-semibold text-blue-800 dark:bg-blue-950/60 dark:text-cyan-300 border border-blue-300/40">
                                    {instanceName || instanceId || 'All Instances'}
                                </span>
                            </div>
                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                                Inspect conversation histories, active queues, and full-screen prompt details with asset preview
                            </p>
                        </div>
                    </div>

                    {/* Toolbar Capsule */}
                    <div className="flex items-center gap-2">
                        <div className="flex items-center rounded-full bg-slate-100 dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs">
                            <button
                                type="button"
                                onClick={handleBackup}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-l-full transition-colors cursor-pointer"
                                title="Backup Prompts to JSON"
                            >
                                <Download className="w-3.5 h-3.5 text-indigo-500" />
                                <span>Backup</span>
                            </button>
                            <button
                                type="button"
                                onClick={handleRestore}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                title="Restore Running Prompts (7s stabilization delay)"
                            >
                                <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Restore</span>
                            </button>
                            <button
                                type="button"
                                onClick={loadTree}
                                disabled={isLoading}
                                className="flex items-center gap-1.5 px-3 py-1.5 text-xs font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200 dark:hover:bg-[#15334d] rounded-r-full transition-colors cursor-pointer"
                                title="Refresh Tree"
                            >
                                <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
                                <span>Refresh</span>
                            </button>
                        </div>

                        <button
                            type="button"
                            onClick={onClose}
                            className="rounded-xl p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#15334d] dark:hover:text-slate-200 transition-colors cursor-pointer"
                            title="Close"
                        >
                            <X className="h-5 w-5" />
                        </button>
                    </div>
                </div>

                {/* Notification Banner */}
                {actionMsg && (
                    <div className="bg-emerald-500/10 border-b border-emerald-500/20 px-6 py-2 text-xs font-medium text-emerald-700 dark:text-emerald-300 flex items-center gap-2">
                        <span className="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse" />
                        {actionMsg}
                    </div>
                )}
                {error && (
                    <div className="bg-rose-500/10 border-b border-rose-500/20 px-6 py-2 text-xs font-medium text-rose-700 dark:text-rose-300">
                        {error}
                    </div>
                )}

                {/* Main Split View */}
                <div className="flex flex-1 overflow-hidden">
                    {/* Left Panel: Projects & Conversation Tree */}
                    <div className="w-80 sm:w-96 flex flex-col border-r border-slate-200 dark:border-[#15334d] bg-slate-50/50 dark:bg-[#071a27]/60">
                        {/* Search Input */}
                        <div className="p-3 border-b border-slate-200 dark:border-[#15334d]">
                            <div className="relative">
                                <Search className="absolute left-3 top-2.5 h-3.5 w-3.5 text-slate-400" />
                                <input
                                    type="text"
                                    placeholder="Search projects or conversations..."
                                    value={searchQuery}
                                    onChange={(e) => setSearchQuery(e.target.value)}
                                    className="w-full rounded-xl bg-white dark:bg-[#0c2438] pl-9 pr-3 py-1.5 text-xs text-slate-800 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] focus:outline-none focus:ring-2 focus:ring-blue-500/30"
                                />
                            </div>
                        </div>

                        {/* Tree Items */}
                        <div className="flex-1 overflow-y-auto p-3 space-y-1">
                            {filteredProjects.length === 0 ? (
                                <div className="py-12 text-center text-xs text-slate-400">
                                    No projects or conversations found.
                                </div>
                            ) : (
                                filteredProjects.map((project) => {
                                    const isExpanded = Boolean(expandedProjects[project.project_id]);
                                    const isSelected = selectedProject?.project_id === project.project_id;

                                    return (
                                        <div key={project.project_id} className="space-y-0.5">
                                            {/* Project Row */}
                                            <div
                                                onClick={() => {
                                                    setSelectedProject(project);
                                                    setExpandedProjects((prev) => ({
                                                        ...prev,
                                                        [project.project_id]: !isExpanded,
                                                    }));
                                                }}
                                                className={cn(
                                                    "group flex items-center justify-between rounded-xl px-2.5 py-1.5 text-xs font-semibold cursor-pointer transition-colors",
                                                    isSelected
                                                        ? "bg-blue-100/70 text-blue-900 dark:bg-blue-950/60 dark:text-cyan-300"
                                                        : "text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#0c2438]"
                                                )}
                                            >
                                                <div className="flex items-center gap-1.5 min-w-0">
                                                    {isExpanded ? (
                                                        <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                    ) : (
                                                        <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                    )}
                                                    <Folder className="h-4 w-4 text-amber-500 shrink-0" />
                                                    <span className="truncate">{project.repo_name}</span>
                                                </div>
                                                <span className="rounded-full bg-slate-200 dark:bg-[#15334d] px-2 py-0.2 text-[10px] font-mono text-slate-600 dark:text-slate-300 shrink-0">
                                                    {project.conversations.length}
                                                </span>
                                            </div>

                                            {/* Conversation Children */}
                                            {isExpanded && (
                                                <div className="ml-5 pl-2 border-l border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                                                    {project.conversations.map((conv) => {
                                                        const isConvSelected = selectedConversation?.conversation_id === conv.conversation_id;

                                                        return (
                                                            <div
                                                                key={conv.conversation_id}
                                                                onClick={() => {
                                                                    setSelectedProject(project);
                                                                    setSelectedConversation(conv);
                                                                }}
                                                                onDoubleClick={() => openInspector(conv, project.repo_path)}
                                                                className={cn(
                                                                    "group flex items-center justify-between rounded-lg px-2.5 py-1.5 text-xs cursor-pointer transition-colors",
                                                                    isConvSelected
                                                                        ? "bg-blue-600 text-white shadow-2xs font-semibold"
                                                                        : "text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#0c2438]"
                                                                )}
                                                                title="Single click to view; double-click for full-screen inspector"
                                                            >
                                                                <div className="flex items-center gap-2 min-w-0">
                                                                    <MessageSquare className="h-3.5 w-3.5 shrink-0 opacity-70" />
                                                                    <span className="truncate text-[11px]">
                                                                        {conv.title || conv.short_id}
                                                                    </span>
                                                                </div>
                                                                {conv.is_running && (
                                                                    <span className="h-1.5 w-1.5 rounded-full bg-emerald-400 animate-ping shrink-0" />
                                                                )}
                                                            </div>
                                                        );
                                                    })}
                                                </div>
                                            )}
                                        </div>
                                    );
                                })
                            )}
                        </div>
                    </div>

                    {/* Right Panel: Prompt Details & Queued Items */}
                    <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-[#0c2438]">
                        {selectedConversation ? (
                            <div className="flex-1 flex flex-col p-6 overflow-y-auto space-y-5">
                                {/* Conversation Header */}
                                <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 pb-4 border-b border-slate-200 dark:border-[#15334d]">
                                    <div>
                                        <div className="flex items-center gap-2">
                                            <h3 className="text-base font-bold text-slate-900 dark:text-white">
                                                {selectedConversation.title || selectedConversation.short_id}
                                            </h3>
                                            <span className={cn(
                                                "px-2 py-0.5 rounded-full text-[10px] font-bold border",
                                                selectedConversation.is_running
                                                    ? "bg-emerald-50 text-emerald-700 border-emerald-300 dark:bg-emerald-950/60 dark:text-emerald-300"
                                                    : "bg-slate-100 text-slate-600 border-slate-200 dark:bg-[#071a27] dark:text-slate-400 dark:border-[#15334d]"
                                            )}>
                                                {selectedConversation.is_running ? 'RUNNING' : 'QUEUED / IDLE'}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-3 text-xs text-slate-500 dark:text-slate-400 mt-1 font-mono">
                                            <span>Conv ID: {selectedConversation.conversation_id}</span>
                                            <span>·</span>
                                            <span>Modified: {selectedConversation.last_modified}</span>
                                        </div>
                                    </div>

                                    {/* Double Click Full Screen Button */}
                                    <button
                                        type="button"
                                        onClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                        className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-blue-50 text-blue-700 dark:bg-blue-950/60 dark:text-cyan-300 border border-blue-200 dark:border-blue-800 text-xs font-semibold hover:bg-blue-100 transition-colors cursor-pointer"
                                        title="Double-click row or click here to open full-screen inspector"
                                    >
                                        <Maximize2 className="h-3.5 w-3.5" />
                                        <span>Full-Screen Inspector</span>
                                    </button>
                                </div>

                                {/* Active / Queued Prompt Preview Box */}
                                <div>
                                    <div className="flex items-center justify-between mb-2">
                                        <label className="text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider">
                                            Prompt Content & Instruction ({selectedConversation.prompt_word_count || 0} words)
                                        </label>
                                        <span className="text-[11px] text-slate-400 italic">
                                            Double-click to expand with images & syntax highlighting
                                        </span>
                                    </div>
                                    <div
                                        onDoubleClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                        className="rounded-2xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-4 text-xs font-mono text-slate-800 dark:text-slate-200 leading-relaxed whitespace-pre-wrap break-words max-h-96 overflow-y-auto cursor-pointer hover:border-blue-400 transition-colors shadow-inner"
                                    >
                                        {selectedConversation.prompt_preview_200w || 'No prompt preview recorded.'}
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="flex-1 flex flex-col items-center justify-center text-center p-8 text-slate-400">
                                <Layers className="h-12 w-12 stroke-1 mb-3 opacity-40" />
                                <p className="text-sm font-medium">Select a conversation from the left tree</p>
                                <p className="text-xs mt-1">Double-click any conversation row to view full prompts, images, and attachments.</p>
                            </div>
                        )}
                    </div>
                </div>
            </div>

            {/* Full-Screen Prompt Inspector Modal */}
            {inspectorPrompt && (
                <div className="fixed inset-0 z-60 flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
                    <div className="flex h-[95vh] w-full max-w-5xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#15334d] bg-white dark:bg-[#0c2438] shadow-2xl">
                        {/* Inspector Header */}
                        <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#15334d] px-6 py-4 bg-slate-50 dark:bg-[#071a27]">
                            <div className="flex items-center gap-3">
                                <div className="flex h-10 w-10 items-center justify-center rounded-xl bg-purple-500/10 text-purple-600 dark:text-purple-400 border border-purple-500/20">
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
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-xl bg-slate-100 hover:bg-slate-200 dark:bg-[#15334d] dark:hover:bg-[#1c4566] text-xs font-semibold text-slate-700 dark:text-slate-200 transition-colors cursor-pointer"
                                >
                                    {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5" />}
                                    <span>{isCopied ? 'Copied!' : 'Copy Text'}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setInspectorPrompt(null)}
                                    className="rounded-xl p-2 text-slate-400 hover:bg-slate-100 hover:text-slate-700 dark:hover:bg-[#15334d] dark:hover:text-slate-200 transition-colors cursor-pointer"
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
                                                className="overflow-hidden rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-100 dark:bg-[#071a27] p-2"
                                            >
                                                <img
                                                    src={img}
                                                    alt={`Prompt Image ${i + 1}`}
                                                    className="w-full h-44 object-contain rounded-lg"
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
                                                className="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg bg-slate-100 dark:bg-[#071a27] border border-slate-200 dark:border-[#15334d] text-xs font-mono text-slate-700 dark:text-slate-300"
                                            >
                                                <FileText className="w-3 h-3 text-slate-400" />
                                                <span className="truncate max-w-xs">{file}</span>
                                            </span>
                                        ))}
                                    </div>
                                </div>
                            )}

                            {/* Full Prompt Text */}
                            <div className="space-y-2">
                                <label className="text-xs font-bold uppercase tracking-wider text-slate-700 dark:text-slate-300">
                                    Full Prompt Instruction
                                </label>
                                <div className="rounded-2xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 font-mono text-xs text-slate-900 dark:text-slate-100 leading-relaxed whitespace-pre-wrap break-words select-text">
                                    {inspectorPrompt.text}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );
}
