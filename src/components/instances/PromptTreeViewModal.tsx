import { useState, useEffect, useMemo, useCallback, useRef } from 'react';
import { createPortal } from 'react-dom';
import {
    X,
    Folder,
    Layers,
    MessageSquare,
    Download,
    Upload,
    Maximize2,
    Minimize2,
    Pin,
    Copy,
    Check,
    FileText,
    Image as ImageIcon,
    RefreshCw,
    Search,
    ChevronRight,
    ChevronDown,
    ChevronUp,
    RotateCw,
    RotateCcw,
    ListPlus,
    Eye,
    Code,
    Edit3,
    Clock,
    Archive,
    ArchiveRestore,
    ExternalLink,
    Bot,
    User,
    Terminal,
    Wrench,
    Sparkles,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { useInstanceStore } from '../../stores/useInstanceStore';
import {
    type SyncInterval,
    getPromptTreeSyncInterval,
    setPromptTreeSyncInterval,
    getArchivedProjectsForInstance,
    setArchivedProjectsForInstance,
    focusOrLaunchInstance,
    focusInstanceWorkspace,
    sendPromptNow,
} from '../../services/instanceService';
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
    instance_seq_num?: number;
    instance_name?: string;
    instance_exe_name?: string;
    prompt_preview_200w: string;
    prompt_tail_snippet?: string;
    prompt_word_count: number;
    last_modified: string;
    byte_size?: number;
    repeat_count?: number;
    repeat_badge?: string;
    sub_runs?: AgmConversationNode[];
    prompt_category?: string;
    is_queued?: boolean;
    full_prompt_text?: string;
    latest_step_summary?: string;
    latest_response?: string;
    execution_results?: string;
    tool_calls_summary?: string;
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
    instance_exe_name?: string;
    bound_email?: string;
    is_running: boolean;
    conversations: AgmConversationNode[];
    running_count?: number;
    queued_count?: number;
    byte_size?: number;
    repeat_count?: number;
    repeat_badge?: string;
}

interface PromptTreeViewModalProps {
    isOpen: boolean;
    onClose: () => void;
    instanceId: string;
    instanceName: string;
    sequenceNumber?: number;
    executablePath?: string;
    initialSelectedProjectId?: string;
}

type ViewMode = 'preview' | 'raw' | 'edit';

// Helper to format clean, compact sequence badge without heavy bracket clutter
export function formatDualBadge(agmCode: string | undefined, defaultAgm: string, gmCode: string | undefined, defaultGm: string): string {
    const rawAgm = (agmCode || defaultAgm || '').replace(/^AGM:/i, '').replace(/[\[\]]/g, '').trim();
    const rawGm = (gmCode || defaultGm || '').replace(/^GM:/i, '').replace(/[\[\]]/g, '').trim();
    if (rawAgm && rawGm && rawAgm !== rawGm) {
        return `${rawAgm} · ${rawGm}`;
    }
    return rawGm || rawAgm || defaultAgm;
}

// Helper to count words
function countWords(str: string): number {
    const trimmed = str.trim();
    if (!trimmed) return 0;
    return trimmed.split(/\s+/).length;
}


// Helper to truncate text at word limit while strictly preserving original line breaks and newlines
function getTruncatedText(text: string, maxWords: number): { displayText: string; isTruncated: boolean; totalWords: number } {
    const trimmed = text.trim();
    if (!trimmed) return { displayText: '', isTruncated: false, totalWords: 0 };
    const allWords = trimmed.split(/\s+/).filter(Boolean);
    const totalWords = allWords.length;
    if (totalWords <= maxWords) {
        return { displayText: text, isTruncated: false, totalWords };
    }

    const lines = text.split('\n');
    const selectedLines: string[] = [];
    let wordsCollected = 0;

    for (const line of lines) {
        const lineWords = line.trim().split(/\s+/).filter(Boolean);
        if (lineWords.length === 0) {
            if (selectedLines.length > 0 && selectedLines[selectedLines.length - 1] !== '') {
                selectedLines.push('');
            }
            continue;
        }

        if (wordsCollected + lineWords.length <= maxWords) {
            selectedLines.push(line);
            wordsCollected += lineWords.length;
        } else {
            const remaining = maxWords - wordsCollected;
            if (remaining > 0) {
                selectedLines.push(lineWords.slice(0, remaining).join(' ') + ' ...');
            } else if (selectedLines.length > 0) {
                selectedLines[selectedLines.length - 1] = selectedLines[selectedLines.length - 1] + ' ...';
            }
            break;
        }
    }

    return {
        displayText: selectedLines.join('\n').trim(),
        isTruncated: true,
        totalWords,
    };
}

// Pre-formatter to ensure inline markdown headings and paragraph line gaps have explicit spacing
function formatPromptForMarkdown(text: string): string {
    if (!text) return '';
    let formatted = text.replace(/\r\n/g, '\n').replace(/\r/g, '\n');

    // Elevate inline <truncated ...> markers to standalone block callouts
    formatted = formatted.replace(/(<truncated\s+\d+\s+(?:bytes|lines)>)/gi, '\n\n$1\n\n');

    // Separate inline markdown headings attached to paragraph text:
    // e.g. "# High Priority Instruction Hi there." -> "# High Priority Instruction\n\nHi there."
    formatted = formatted.replace(
        /^(#{1,4}\s+[A-Za-z0-9_\-\s]{2,40}?)([\.\:\!\?])\s+([A-Z])/gm,
        '$1$2\n\n$3'
    );
    formatted = formatted.replace(
        /^(#{1,4}\s+High Priority Instruction|#{1,4}\s+Instruction|#{1,4}\s+Overview|#{1,4}\s+Notice|#{1,4}\s+Task|#{1,4}\s+Plan)\s+([A-Z])/gm,
        '$1\n\n$2'
    );

    return formatted;
}

export interface TruncatedContextCalloutProps {
    omittedBytes?: number;
    omittedLines?: number;
    fullText?: string;
    onExpandFull?: () => void;
}

export function TruncatedContextCallout({
    omittedBytes,
    omittedLines,
    onExpandFull,
}: TruncatedContextCalloutProps) {
    const formattedSize = omittedBytes 
        ? omittedBytes >= 1024 * 1024 
            ? `${(omittedBytes / (1024 * 1024)).toFixed(1)} MB` 
            : `${(omittedBytes / 1024).toFixed(1)} KB`
        : omittedLines 
            ? `${omittedLines} lines`
            : 'transcript context';

    return (
        <div
            onClick={onExpandFull}
            className="my-3 flex items-center justify-between px-3.5 py-2 rounded-xl bg-amber-500/10 border border-amber-500/30 text-amber-800 dark:text-amber-200 text-xs shadow-xs backdrop-blur-xs cursor-pointer hover:bg-amber-500/20 transition-all"
            title="Click to inspect/expand full un-truncated context"
        >
            <div className="flex items-center gap-2 font-mono font-medium">
                <span className="text-amber-500 text-sm">⚡</span>
                <span>Omitted {formattedSize} transcript context · Click to expand</span>
            </div>
            {onExpandFull && (
                <button
                    type="button"
                    onClick={(e) => {
                        e.stopPropagation();
                        onExpandFull();
                    }}
                    className="flex items-center gap-1 px-2.5 py-1 rounded-full bg-amber-500 text-white dark:text-slate-900 font-bold hover:bg-amber-600 transition-colors text-[10px] cursor-pointer"
                >
                    <span>Expand Full</span>
                    <ChevronDown className="w-3 h-3" />
                </button>
            )}
        </div>
    );
}

// Helper to standardize and format clean sequence codes (#P001, C001)
export function formatCleanSeqCode(code: string | undefined | null, prefix: 'P' | 'C' = 'C'): string {
    if (!code) return prefix === 'P' ? '#P001' : 'C001';
    const stripped = code.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
    if (!stripped) return prefix === 'P' ? '#P001' : 'C001';
    if (prefix === 'P') {
        const withP = stripped.startsWith('P') || stripped.startsWith('#P') ? stripped : `P${stripped}`;
        return withP.startsWith('#') ? withP : `#${withP}`;
    }
    const withoutHash = stripped.replace(/^#/, '');
    return withoutHash.startsWith('C') ? withoutHash : `C${withoutHash}`;
}

export function formatSeqBadge(raw: string | undefined | null, fallback: string): string {
    if (!raw) return fallback;
    const clean = raw.replace(/^(AGM:|GM:)/i, '').replace(/[\[\]]/g, '').trim();
    if (!clean) return fallback;
    if (fallback.startsWith('#P') || fallback === '#P001') {
        const withP = clean.startsWith('P') || clean.startsWith('#P') ? clean : `P${clean}`;
        return withP.startsWith('#') ? withP : `#${withP}`;
    }
    if (fallback.startsWith('C') || fallback === 'C001') {
        const withoutHash = clean.replace(/^#/, '');
        return withoutHash.startsWith('C') ? withoutHash : `C${withoutHash}`;
    }
    return clean.startsWith('#') || clean.startsWith('P') || clean.startsWith('C') ? clean : `#${clean}`;
}

// Helper to check if a conversation has zero prompt content and untitled title (true ghost node)
export function isGhostConversation(conv: AgmConversationNode): boolean {
    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();
    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;
    return isUntitled && isEmptyPrompt;
}

// Helper to check if a conversation is stale or empty (exempt if actively running with content)
export function isStaleOrEmptyConversation(conv: AgmConversationNode): boolean {
    if (isGhostConversation(conv)) {
        return true;
    }
    const isEmptyPrompt =
        conv.prompt_word_count === 0 ||
        !conv.prompt_preview_200w ||
        conv.prompt_preview_200w.trim().length === 0;
    // 0-word empty untitled conversations must NEVER be treated as running or active
    if (isEmptyPrompt) {
        return true;
    }
    if (Boolean(conv.is_running)) {
        return false;
    }
    const title = (conv.title || '').trim().toLowerCase();
    const isUntitled =
        !title ||
        title === 'untitled' ||
        title.startsWith('untitled') ||
        title === 'new conversation' ||
        title.startsWith('new conversation') ||
        title === 'conversation' ||
        title.startsWith('conversation-') ||
        title.startsWith('chat-') ||
        title === 'chat' ||
        title === (conv.short_id || '').toLowerCase();
    return isUntitled;
}

export const isUntitledOrEmpty = isStaleOrEmptyConversation;


// Helper to strip markdown image syntax and data URIs for "Copy Text"
function stripImagesFromPrompt(text: string): string {
    return text
        .replace(/!\[.*?\]\((?:https?:\/\/.*?|data:image\/.*?;base64,.*?|[^\s)]+)\)/g, '')
        .trim();
}

// Helper for rich clipboard media copying with ClipboardItem
export async function copyPromptWithRichImages(markdownText: string): Promise<boolean> {
    try {
        if (!navigator.clipboard || typeof ClipboardItem === 'undefined') {
            await navigator.clipboard.writeText(markdownText);
            return true;
        }

        // Convert markdown images to HTML <img> tags
        let htmlContent = markdownText
            .replace(/\n\n/g, '</p><p>')
            .replace(/\n/g, '<br/>')
            .replace(/!\[(.*?)\]\((.*?)\)/g, '<img src="$2" alt="$1" style="max-width:100%; border-radius:8px; margin:8px 0;" />');

        htmlContent = `<!DOCTYPE html><html><body><p>${htmlContent}</p></body></html>`;

        const htmlBlob = new Blob([htmlContent], { type: 'text/html' });
        const textBlob = new Blob([markdownText], { type: 'text/plain' });

        const item = new ClipboardItem({
            'text/html': htmlBlob,
            'text/plain': textBlob,
        });

        await navigator.clipboard.write([item]);
        return true;
    } catch {
        // Fallback to standard text copy if rich clipboard write fails
        try {
            await navigator.clipboard.writeText(markdownText);
            return true;
        } catch {
            return false;
        }
    }
}

export type PromptTier = 'USER_PROMPT' | 'SUBAGENT_INSTRUCTION' | 'SYSTEM_MESSAGE' | 'TOOL_OUTPUT';

export interface TierClassificationResult {
    tier: PromptTier;
    category: PromptTier;
    label: string;
    roleBadge: string;
    badgeStyle: string;
    iconName: 'user' | 'bot' | 'terminal' | 'wrench';
    isSubagent: boolean;
    isNonPrompt: boolean;
    confidence: number;
    matchedPattern?: string;
    subagentRole?: string;
    description: string;
}

export type PromptCategory = PromptTier;
export type PromptCategoryResult = TierClassificationResult;

export function classifyPromptTier(
    text?: string,
    title?: string,
    metadata?: Record<string, any>
): TierClassificationResult {
    const raw = (text || '').trim();
    const cleanTitle = (title || '').trim().toLowerCase();

    // 1. Tool Output Classification (Highest Specificity)
    const isToolOutput =
        raw.startsWith('[Tool Result]') ||
        raw.startsWith('{"tool_call_id":') ||
        raw.startsWith('Tool returned:') ||
        raw.includes('<tool_response>') ||
        raw.includes('<tool_calls>') ||
        (raw.startsWith('```') && (cleanTitle.includes('output') || cleanTitle.includes('result')));

    if (isToolOutput) {
        return {
            tier: 'TOOL_OUTPUT',
            category: 'TOOL_OUTPUT',
            label: 'Tool Output',
            roleBadge: 'TOOL',
            badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
            iconName: 'wrench',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.98,
            matchedPattern: 'tool_output_marker',
            description: 'Diagnostic tool execution results and execution steps',
        };
    }

    // 2. System Message Classification
    const isSystemMessage =
        raw.startsWith('<SYSTEM_MESSAGE>') ||
        raw.includes('<conversation_transcript>') ||
        raw.includes('<artifacts>') ||
        cleanTitle.startsWith('system:');

    if (isSystemMessage && !raw.includes('invoked by a caller agent') && !raw.includes('You are')) {
        return {
            tier: 'SYSTEM_MESSAGE',
            category: 'SYSTEM_MESSAGE',
            label: 'System Message',
            roleBadge: 'SYSTEM',
            badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
            iconName: 'terminal',
            isSubagent: false,
            isNonPrompt: true,
            confidence: 0.95,
            matchedPattern: 'system_message_tag',
            description: 'System initialization directives and environment preambles',
        };
    }

    // 3. AI Subagent Instruction Classification
    const subagentRoleMatch =
        raw.match(/Role:\s*([A-Za-z0-9_\-\s]{3,30})/i) ||
        raw.match(/You are (?:the )?([A-Za-z0-9_\-\s]{3,30}) for Task/i) ||
        raw.match(/You are an? ([A-Za-z0-9_\-\s]{3,30}) agent/i);

    const isSubagent =
        raw.includes('invoked by a caller agent') ||
        raw.includes('<subagent_reminder>') ||
        raw.includes('send_message to communicate all results') ||
        raw.includes('Execute enhanced Read Memory protocol') ||
        raw.includes('You are pair programming with a USER') ||
        raw.includes('Project Structure Memory Analysis') ||
        raw.includes('Safe Removal Architect') ||
        raw.includes('Spec Author') ||
        raw.includes('Codebase Researcher') ||
        raw.includes('Database Debugger') ||
        raw.includes('QA Tester') ||
        raw.includes('Read & Understand') ||
        /Role:\s*(?:Codebase Researcher|Database Debugger|QA Tester|Subagent|Worker|Architect)/i.test(raw) ||
        cleanTitle.includes('subagent') ||
        cleanTitle.includes('worker-') ||
        cleanTitle.includes('worker ') ||
        cleanTitle.includes('author-') ||
        cleanTitle.includes('researcher') ||
        cleanTitle.includes('debugger') ||
        cleanTitle.includes('memory') ||
        cleanTitle.includes('analysis') ||
        cleanTitle.includes('spec') ||
        cleanTitle.includes('tester') ||
        cleanTitle.includes('qa') ||
        metadata?.is_subagent === true ||
        metadata?.prompt_category === 'subagent';

    if (isSubagent) {
        let detectedRole = 'Subagent';
        if (subagentRoleMatch) {
            detectedRole = subagentRoleMatch[1].trim();
        } else if (cleanTitle.includes('memory') || raw.includes('Memory Analysis')) {
            detectedRole = 'Memory Analysis';
        } else if (cleanTitle.includes('spec') || raw.includes('Spec Author')) {
            detectedRole = 'Spec Author';
        } else if (cleanTitle.includes('research') || raw.includes('Researcher')) {
            detectedRole = 'Researcher';
        } else if (cleanTitle.includes('worker')) {
            detectedRole = 'Worker';
        } else if (cleanTitle.includes('debug') || raw.includes('Debugger')) {
            detectedRole = 'Debugger';
        } else if (cleanTitle.includes('tester') || cleanTitle.includes('qa') || raw.includes('QA Tester')) {
            detectedRole = 'QA Tester';
        } else if (cleanTitle.includes('architect') || raw.includes('Architect')) {
            detectedRole = 'Architect';
        } else if (cleanTitle.includes('analysis') || raw.includes('Analysis')) {
            detectedRole = 'Analysis';
        }

        return {
            tier: 'SUBAGENT_INSTRUCTION',
            category: 'SUBAGENT_INSTRUCTION',
            label: 'AI Subagent Instruction',
            roleBadge: detectedRole,
            badgeStyle: 'bg-purple-500/15 text-purple-700 dark:text-purple-300 border-purple-400/30',
            iconName: 'bot',
            isSubagent: true,
            isNonPrompt: false,
            confidence: 0.96,
            matchedPattern: 'subagent_signature',
            subagentRole: detectedRole,
            description: 'Autonomous worker prompt or subagent execution directive',
        };
    }

    // 4. Default: USER_PROMPT (Direct Human Request)
    return {
        tier: 'USER_PROMPT',
        category: 'USER_PROMPT',
        label: 'User Prompt',
        roleBadge: 'USER',
        badgeStyle: 'bg-sky-500/15 text-sky-700 dark:text-cyan-300 border-sky-400/30',
        iconName: 'user',
        isSubagent: false,
        isNonPrompt: false,
        confidence: 0.90,
        description: 'Direct human directive submitted via chat, CLI, or task request',
    };
}

export function detectPromptCategory(
    text?: string,
    title?: string,
    conv?: AgmConversationNode
): TierClassificationResult {
    return classifyPromptTier(text, title, conv as any);
}

export interface HierarchicalConversationNode {
    primaryNode: AgmConversationNode;
    classification: TierClassificationResult;
    subagents: HierarchicalConversationNode[];
    isExpanded: boolean;
}

export function assembleConversationHierarchy(
    conversations: AgmConversationNode[]
): HierarchicalConversationNode[] {
    // Sort chronologically ascending (oldest first) so that a user prompt precedes the subagents it spawned
    const chronological = [...conversations].sort((a, b) => {
        const aTime = new Date(a.last_modified).getTime() || 0;
        const bTime = new Date(b.last_modified).getTime() || 0;
        return aTime - bTime;
    });

    const rootNodes: HierarchicalConversationNode[] = [];
    let currentRoot: HierarchicalConversationNode | null = null;

    for (const conv of chronological) {
        const classification = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);

        const node: HierarchicalConversationNode = {
            primaryNode: conv,
            classification,
            subagents: [],
            isExpanded: true,
        };

        if (classification.tier === 'SUBAGENT_INSTRUCTION') {
            if (currentRoot) {
                // Attach as child sub-point to active root prompt
                currentRoot.subagents.push(node);
            } else {
                // If no root has occurred yet, treat as standalone root
                rootNodes.push(node);
            }
        } else {
            // New User Prompt, System Message or Tool Output becomes new root
            rootNodes.push(node);
            if (classification.tier === 'USER_PROMPT') {
                currentRoot = node;
            }
        }
    }

    // Reverse rootNodes so the newest user prompt appears at top,
    // and sort running roots to the top
    return rootNodes.reverse().sort((a, b) => {
        const aRunning = Boolean(a.primaryNode.is_running) || a.subagents.some((s) => Boolean(s.primaryNode.is_running));
        const bRunning = Boolean(b.primaryNode.is_running) || b.subagents.some((s) => Boolean(s.primaryNode.is_running));
        if (aRunning !== bRunning) return aRunning ? -1 : 1;
        return 0;
    });
}

export function normalizePromptForGrouping(conv: AgmConversationNode): string {
    const text = conv.prompt_preview_200w || conv.title || conv.short_id || conv.conversation_id;
    return text.trim().toLowerCase().replace(/\s+/g, ' ');
}

// Inline Markdown parser (Headings, bold, italic, code blocks, inline code, lists, images, blockquotes, truncated callouts)
function parseInlineMarkdown(text: string, onToggleExpand?: () => void): React.ReactNode[] {
    const rawNodes: React.ReactNode[] = [];
    const tokenRegex = /(!\[(.*?)\]\((.*?)\)|\[(.*?)\]\((.*?)\)|<truncated\s+(\d+)\s+(bytes|lines)>|`([^`]+)`|\*\*([^*]+)\*\*|__([^_]+)__|~~([^~]+)~~|\*([^*]+)\*|_([^_]+)_)/gi;
    let lastIndex = 0;
    let match: RegExpExecArray | null;

    while ((match = tokenRegex.exec(text)) !== null) {
        if (match.index > lastIndex) {
            rawNodes.push(text.substring(lastIndex, match.index));
        }

        const [full, , imgAlt, imgSrc, linkText, linkUrl, truncCount, truncUnit, inlineCode, boldStar, boldUnder, delText, italicStar, italicUnder] = match;

        if (truncCount !== undefined) {
            const count = parseInt(truncCount, 10);
            const unit = truncUnit.toLowerCase();
            const formattedSize = unit === 'bytes'
                ? (count >= 1024 * 1024 ? `${(count / (1024 * 1024)).toFixed(1)} MB` : `${(count / 1024).toFixed(1)} KB`)
                : `${count} lines`;
            rawNodes.push(
                <span
                    key={`trunc-${match.index}`}
                    onClick={onToggleExpand}
                    className="inline-flex items-center gap-1.5 px-2.5 py-0.5 my-0.5 rounded-full text-[10px] font-mono font-medium bg-amber-500/15 text-amber-800 dark:text-amber-200 border border-amber-500/30 shadow-2xs cursor-pointer hover:bg-amber-500/25 transition-all"
                    title={`Omitted ${truncCount} ${truncUnit} from prompt transcript context - Click to inspect/expand`}
                >
                    <span className="text-amber-500 font-bold">⚡</span>
                    <span>Omitted {formattedSize} transcript context · Click to expand</span>
                </span>
            );
        } else if (imgSrc !== undefined) {
            rawNodes.push(
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
            rawNodes.push(
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
            rawNodes.push(
                <code
                    key={`code-${match.index}`}
                    className="px-1.5 py-0.5 rounded-[4px] bg-slate-100 dark:bg-[#071a27] text-cyan-700 dark:text-cyan-300 font-mono text-[11px] border border-slate-200 dark:border-cyan-900/40"
                >
                    {inlineCode}
                </code>
            );
        } else if (boldStar !== undefined || boldUnder !== undefined) {
            rawNodes.push(
                <strong key={`bold-${match.index}`} className="font-bold text-slate-900 dark:text-white">
                    {boldStar || boldUnder}
                </strong>
            );
        } else if (delText !== undefined) {
            rawNodes.push(
                <del key={`del-${match.index}`} className="line-through opacity-70">
                    {delText}
                </del>
            );
        } else if (italicStar !== undefined || italicUnder !== undefined) {
            rawNodes.push(
                <em key={`italic-${match.index}`} className="italic">
                    {italicStar || italicUnder}
                </em>
            );
        } else {
            rawNodes.push(full);
        }

        lastIndex = tokenRegex.lastIndex;
    }

    if (lastIndex < text.length) {
        rawNodes.push(text.substring(lastIndex));
    }

    // Convert \n in text nodes to explicit <br className="my-1.5 block select-none" /> elements
    const nodes: React.ReactNode[] = [];
    rawNodes.forEach((node, nodeIdx) => {
        if (typeof node === 'string') {
            if (node.includes('\n')) {
                const parts = node.split('\n');
                parts.forEach((part, pIdx) => {
                    if (part) nodes.push(part);
                    if (pIdx < parts.length - 1) {
                        nodes.push(<br key={`br-${nodeIdx}-${pIdx}`} className="my-1.5 block select-none" />);
                    }
                });
            } else {
                nodes.push(node);
            }
        } else {
            nodes.push(node);
        }
    });

    return nodes.length > 0 ? nodes : [text];
}

interface RichMarkdownRendererProps {
    content: string;
    showAllWords?: boolean;
    onToggleExpand?: () => void;
    isTruncated?: boolean;
}

// Rich Markdown renderer component
function RichMarkdownRenderer({ content, showAllWords, onToggleExpand, isTruncated }: RichMarkdownRendererProps) {
    const [copiedBlockIndex, setCopiedBlockIndex] = useState<number | null>(null);

    const handleCopyCode = (code: string, index: number) => {
        navigator.clipboard.writeText(code);
        setCopiedBlockIndex(index);
        setTimeout(() => setCopiedBlockIndex(null), 2000);
    };

    if (!content.trim()) {
        return <div className="text-xs text-slate-400 italic">No content to preview.</div>;
    }

    const formattedContent = formatPromptForMarkdown(content);
    const lines = formattedContent.split('\n');
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
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand)}</li>
                    ))}
                </ul>
            );
        } else {
            elements.push(
                <ol key={`ol-${keyPrefix}`} className="list-decimal ml-5 my-2 space-y-1 text-xs text-slate-700 dark:text-slate-300">
                    {currentList.items.map((item, idx) => (
                        <li key={idx}>{parseInlineMarkdown(item, onToggleExpand)}</li>
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
                {parseInlineMarkdown(text, onToggleExpand)}
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
            elements.push(<br key={`br-${i}`} className="my-1.5 block select-none" />);
            continue;
        }

        // Truncated transcript omission banner
        const truncLineMatch = trimmed.match(/^<truncated\s+(\d+)\s+(bytes|lines)>/i);
        if (truncLineMatch) {
            const count = parseInt(truncLineMatch[1], 10);
            const unit = truncLineMatch[2].toLowerCase();
            elements.push(
                <TruncatedContextCallout
                    key={`trunc-block-${i}`}
                    omittedBytes={unit === 'bytes' ? count : undefined}
                    omittedLines={unit === 'lines' ? count : undefined}
                    onExpandFull={onToggleExpand}
                />
            );
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
                    {parseInlineMarkdown(trimmed.slice(2), onToggleExpand)}
                </h1>
            );
            continue;
        }
        if (trimmed.startsWith('## ')) {
            elements.push(
                <h2 key={`h2-${i}`} className="text-sm font-bold mt-2.5 mb-1 text-slate-900 dark:text-white pb-0.5 border-b border-slate-200/60 dark:border-[#15334d]/60">
                    {parseInlineMarkdown(trimmed.slice(3), onToggleExpand)}
                </h2>
            );
            continue;
        }
        if (trimmed.startsWith('### ')) {
            elements.push(
                <h3 key={`h3-${i}`} className="text-xs font-bold mt-2 mb-1 text-slate-900 dark:text-white">
                    {parseInlineMarkdown(trimmed.slice(4), onToggleExpand)}
                </h3>
            );
            continue;
        }
        if (trimmed.startsWith('#### ')) {
            elements.push(
                <h4 key={`h4-${i}`} className="text-xs font-semibold mt-1.5 mb-0.5 text-slate-800 dark:text-slate-200">
                    {parseInlineMarkdown(trimmed.slice(5), onToggleExpand)}
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

        // Standard paragraph with explicit <br /> line gap
        const isTrailingEllipsis = trimmed.endsWith('...') || trimmed.endsWith('…');
        const isEndOfExpandedText = Boolean(showAllWords && isTruncated && i === lines.length - 1);

        if ((isTrailingEllipsis || isEndOfExpandedText) && onToggleExpand) {
            const cleanLine = isTrailingEllipsis ? line.replace(/(\.{3}|…)\s*$/, '') : line;
            elements.push(
                <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
                    <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words inline">
                        {parseInlineMarkdown(cleanLine, onToggleExpand)}
                    </p>
                    <span
                        onClick={(e) => {
                            e.stopPropagation();
                            onToggleExpand();
                        }}
                        title={showAllWords ? 'Click to collapse preview to 120 words' : 'Click to expand full prompt text'}
                        className="cursor-pointer font-bold text-cyan-600 dark:text-cyan-400 hover:underline px-1.5 py-0.5 rounded bg-cyan-500/10 hover:bg-cyan-500/20 transition-colors ml-1.5 inline-block select-none"
                    >
                        {showAllWords ? 'Collapse full text' : 'Expand full text'}
                    </span>
                    <br className="my-1.5 block select-none" />
                </div>
            );
        } else {
            elements.push(
                <div key={`p-wrap-${i}`} className="my-1.5 leading-relaxed">
                    <p className="text-xs text-slate-800 dark:text-slate-200 whitespace-pre-wrap break-words">
                        {parseInlineMarkdown(line, onToggleExpand)}
                    </p>
                    <br className="my-1.5 block select-none" />
                </div>
            );
        }
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
    sequenceNumber,
    executablePath,
    initialSelectedProjectId,
}: PromptTreeViewModalProps) {
    const { instances } = useInstanceStore();
    const [treeData, setTreeData] = useState<AgmProjectTreeNode[]>([]);
    const [isLoading, setIsLoading] = useState(false);
    const [isAutoSyncing, setIsAutoSyncing] = useState(false);
    const [syncInterval, setSyncInterval] = useState<SyncInterval>(() => getPromptTreeSyncInterval());
    const [error, setError] = useState<string | null>(null);
    const [searchQuery, setSearchQuery] = useState('');
    const [activeFilter, setActiveFilter] = useState<'all' | 'running' | 'queued' | 'latest_conv' | 'latest_prompt' | 'pinned' | 'archived'>('all');
    const [categoryFilter, setCategoryFilter] = useState<'all' | 'user' | 'subagent' | 'system'>('all');
    const restoreFileInputRef = useRef<HTMLInputElement>(null);
    const [isFullscreen, setIsFullscreen] = useState(false);

    // Pinned projects persisted in localStorage
    const [pinnedProjectIds, setPinnedProjectIds] = useState<string[]>(() => {
        try {
            const stored = localStorage.getItem(`agm_pinned_projects_${instanceId || 'default'}`);
            return stored ? JSON.parse(stored) : [];
        } catch {
            return [];
        }
    });

    // Archived projects persisted in localStorage
    const [archivedProjectIds, setArchivedProjectIds] = useState<string[]>(() => {
        return getArchivedProjectsForInstance(instanceId);
    });

    const [isArchivedProjectsExpanded, setIsArchivedProjectsExpanded] = useState(false);

    const [refreshingProjectId, setRefreshingProjectId] = useState<string | null>(null);
    const [confirmationSuffix, setConfirmationSuffix] = useState<string>('None (Send as is)');
    const [isCopiedText, setIsCopiedText] = useState(false);
    const [isCopiedRaw, setIsCopiedRaw] = useState(false);

    const [selectedProjectScope, setSelectedProjectScope] = useState<string>('all');
    const [isDetailsModalOpen, setIsDetailsModalOpen] = useState(false);
    const [isFocusing, setIsFocusing] = useState(false);
    const [copiedField, setCopiedField] = useState<string | null>(null);

    const [expandedProjects, setExpandedProjects] = useState<Record<string, boolean>>({});
    const [expandedStaleGroups, setExpandedStaleGroups] = useState<Record<string, boolean>>({});

    const [selectedProject, setSelectedProject] = useState<AgmProjectTreeNode | null>(null);
    const [selectedConversation, setSelectedConversation] = useState<AgmConversationNode | null>(null);
    const [activePromptText, setActivePromptText] = useState<string>('');
    const [editedPromptText, setEditedPromptText] = useState<string>('');
    const [viewMode, setViewMode] = useState<ViewMode>('preview');
    const [showAllWords, setShowAllWords] = useState<boolean>(false);
    const [previewTab, setPreviewTab] = useState<'instruction' | 'results'>('instruction');
    const [isCopiedResults, setIsCopiedResults] = useState(false);

    const [isCopied, setIsCopied] = useState(false);
    const [isResending, setIsResending] = useState(false);
    const [isEnqueueing, setIsEnqueueing] = useState(false);
    const [isExportMenuOpen, setIsExportMenuOpen] = useState(false);
    const [actionMsg, setActionMsg] = useState<string | null>(null);

    // Live refs to prevent closure staleness in background auto-sync timer
    const selectedConversationRef = useRef(selectedConversation);
    selectedConversationRef.current = selectedConversation;
    const selectedProjectRef = useRef(selectedProject);
    selectedProjectRef.current = selectedProject;
    const activePromptTextRef = useRef(activePromptText);
    activePromptTextRef.current = activePromptText;

    const currentInstance = useMemo(() => {
        const targetId = selectedProject?.instance_id || instanceId || 'default';
        return instances.find((i) => i.config.id === targetId);
    }, [instances, selectedProject?.instance_id, instanceId]);

    const instancePid = currentInstance?.pid;

    const [elapsedSeconds, setElapsedSeconds] = useState(0);
    useEffect(() => {
        if (!selectedConversation?.is_running) {
            setElapsedSeconds(0);
            return;
        }
        let startTime = Date.now();
        if (selectedConversation.last_modified) {
            const raw = selectedConversation.last_modified;
            if (typeof raw === 'number') {
                startTime = raw < 1e11 ? raw * 1000 : raw;
            } else {
                const trimmed = String(raw).trim();
                if (/^\d+$/.test(trimmed)) {
                    const num = Number(trimmed);
                    startTime = num < 1e11 ? num * 1000 : num;
                } else {
                    const parsed = Date.parse(trimmed);
                    if (Number.isFinite(parsed) && parsed > 0) {
                        startTime = parsed;
                    }
                }
            }
        }
        const updateElapsed = () => {
            const now = Date.now();
            const diff = Math.max(0, Math.floor((now - startTime) / 1000));
            setElapsedSeconds(Number.isFinite(diff) ? diff : 0);
        };
        updateElapsed();
        const timer = setInterval(updateElapsed, 1000);
        return () => clearInterval(timer);
    }, [selectedConversation?.conversation_id, selectedConversation?.is_running, selectedConversation?.last_modified]);

    const formatDuration = (secs: number) => {
        if (!Number.isFinite(secs) || isNaN(secs) || secs < 0) return '0s';
        const m = Math.floor(secs / 60);
        const s = secs % 60;
        if (m === 0) return `${s}s`;
        return `${m}m ${s < 10 ? '0' : ''}${s}s`;
    };

    const togglePinProject = (projectId: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setPinnedProjectIds((prev) => {
            const next = prev.includes(projectId) ? prev.filter((id) => id !== projectId) : [...prev, projectId];
            try {
                localStorage.setItem(`agm_pinned_projects_${instanceId || 'default'}`, JSON.stringify(next));
            } catch {}
            return next;
        });
    };

    const toggleArchiveProject = (projectId: string, e: React.MouseEvent) => {
        e.stopPropagation();
        const isCurrentlyArchived = archivedProjectIds.includes(projectId);
        let nextArchived: string[];
        if (isCurrentlyArchived) {
            nextArchived = archivedProjectIds.filter((id) => id !== projectId);
        } else {
            nextArchived = [...archivedProjectIds, projectId];
            // Automatically unpin the project when archived
            if (pinnedProjectIds.includes(projectId)) {
                const nextPinned = pinnedProjectIds.filter((id) => id !== projectId);
                setPinnedProjectIds(nextPinned);
                try {
                    localStorage.setItem(`agm_pinned_projects_${instanceId || 'default'}`, JSON.stringify(nextPinned));
                } catch {}
            }
        }
        setArchivedProjectIds(nextArchived);
        setArchivedProjectsForInstance(instanceId, nextArchived);
    };

    const handleRefreshSingleProject = async (projectId: string, e: React.MouseEvent) => {
        e.stopPropagation();
        setRefreshingProjectId(projectId);
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                instanceId: instanceId || undefined,
                maxWords: 2000,
                onlyRunning: false,
                force: true,
            });
            const isTargetDefault = instanceId === 'default' || instanceId === '__default__';
            const relevant = instanceId
                ? data
                      .filter((p) => {
                          if (p.instance_id === instanceId) return true;
                          if (isTargetDefault && (!p.instance_id || p.instance_id === 'default' || p.instance_id === '__default__')) {
                              return true;
                          }
                          return false;
                      })
                      .map((p) => ({
                          ...p,
                          conversations: (p.conversations || []).filter((c) => {
                              if (c.instance_id === instanceId) return true;
                              if (isTargetDefault && (!c.instance_id || c.instance_id === 'default' || c.instance_id === '__default__')) {
                                  return true;
                              }
                              return false;
                          }),
                      }))
                : data;
            const finalData = relevant;
            setTreeData(finalData);

            const updatedProject = finalData.find((p) => p.project_id === projectId);
            if (updatedProject && selectedProject?.project_id === projectId) {
                setSelectedProject(updatedProject);
            }
            setActionMsg(`Project ${updatedProject?.repo_name || ''} refreshed!`);
            setTimeout(() => setActionMsg(null), 2500);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to refresh project');
        } finally {
            setRefreshingProjectId(null);
        }
    };

    // Full screen prompt detail inspector
    const [inspectorPrompt, setInspectorPrompt] = useState<{
        title: string;
        text: string;
        convId: string;
        projectPath: string;
        images: string[];
        files: string[];
    } | null>(null);

    // Select conversation directly (2-tier leaf)
    const selectConversation = useCallback((conv: AgmConversationNode, project: AgmProjectTreeNode) => {
        setSelectedProject(project);
        setSelectedConversation(conv);
        const text = conv.full_prompt_text || conv.prompt_preview_200w || '';
        setActivePromptText(text);
        setEditedPromptText(text);
        setShowAllWords(false);
    }, []);

    // Auto-selection algorithm on modal open
    const performAutoSelection = useCallback(
        (data: AgmProjectTreeNode[], currentArchivedIds: string[], currentPinnedIds: string[]) => {
            if (data.length === 0) {
                setSelectedProject(null);
                setSelectedConversation(null);
                return;
            }

            // Direct deep-link focus if initialSelectedProjectId is provided
            if (initialSelectedProjectId) {
                const target = data.find((p) => p.project_id === initialSelectedProjectId);
                if (target) {
                    setExpandedProjects({ [target.project_id]: true });
                    setSelectedProject(target);
                    if (target.conversations && target.conversations.length > 0) {
                        const runningConv = target.conversations.find(
                            (c) => Boolean(c.is_running) && !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                        );
                        if (runningConv) {
                            selectConversation(runningConv, target);
                            if (isStaleOrEmptyConversation(runningConv)) {
                                setExpandedStaleGroups({ [target.project_id]: true });
                            } else {
                                setExpandedStaleGroups({});
                            }
                            return;
                        }
                        const candidateConvs = target.conversations.filter(
                            (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                        );
                        if (candidateConvs.length > 0) {
                            const sortedConvs = [...candidateConvs].sort((a, b) => {
                                const aTime = new Date(a.last_modified).getTime() || 0;
                                const bTime = new Date(b.last_modified).getTime() || 0;
                                return bTime - aTime;
                            });
                            const winningConv = sortedConvs[0];
                            selectConversation(winningConv, target);
                            if (isStaleOrEmptyConversation(winningConv)) {
                                setExpandedStaleGroups({ [target.project_id]: true });
                            } else {
                                setExpandedStaleGroups({});
                            }
                        } else {
                            setSelectedConversation(null);
                        }
                    } else {
                        setSelectedConversation(null);
                    }
                    return;
                }
            }

            // Target non-archived projects in prioritized order
            const nonArchived = data.filter((p) => !currentArchivedIds.includes(p.project_id));
            const targetPool = nonArchived.length > 0 ? nonArchived : data;

            const prioritized = [...targetPool].sort((a, b) => {
                const aPinned = currentPinnedIds.includes(a.project_id);
                const bPinned = currentPinnedIds.includes(b.project_id);
                if (aPinned !== bPinned) return aPinned ? -1 : 1;

                const aRunning = Boolean(a.is_running) || a.conversations.some((c) => Boolean(c.is_running));
                const bRunning = Boolean(b.is_running) || b.conversations.some((c) => Boolean(c.is_running));
                if (aRunning !== bRunning) return aRunning ? -1 : 1;

                const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
                const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
                if (bLatest !== aLatest) return bLatest - aLatest;

                return a.repo_name.localeCompare(b.repo_name);
            });

            // 1. Look for actively running conversation across non-archived projects
            interface RunningCand {
                conv: AgmConversationNode;
                proj: AgmProjectTreeNode;
                timestamp: number;
            }
            const runningCandidates: RunningCand[] = [];
            for (const proj of prioritized) {
                for (const conv of proj.conversations) {
                    if (
                        Boolean(conv.is_running) &&
                        !isGhostConversation(conv) &&
                        !(conv.prompt_word_count === 0 && (!conv.prompt_preview_200w || !conv.prompt_preview_200w.trim()))
                    ) {
                        runningCandidates.push({
                            conv,
                            proj,
                            timestamp: new Date(conv.last_modified).getTime() || 0,
                        });
                    }
                }
            }

            if (runningCandidates.length > 0) {
                // Pick newest running conversation
                runningCandidates.sort((a, b) => b.timestamp - a.timestamp);
                const winning = runningCandidates[0];
                setExpandedProjects({ [winning.proj.project_id]: true });
                selectConversation(winning.conv, winning.proj);
                if (isStaleOrEmptyConversation(winning.conv)) {
                    setExpandedStaleGroups({ [winning.proj.project_id]: true });
                } else {
                    setExpandedStaleGroups({});
                }
                return;
            }

            // 2. If no conversation is running, check if any non-archived project has Boolean(p.is_running)
            const runningProj = prioritized.find((p) => Boolean(p.is_running) && p.conversations.length > 0);
            if (runningProj) {
                const candidateConvs = runningProj.conversations.filter(
                    (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                );
                if (candidateConvs.length > 0) {
                    const sortedConvs = [...candidateConvs].sort((a, b) => {
                        const aTime = new Date(a.last_modified).getTime() || 0;
                        const bTime = new Date(b.last_modified).getTime() || 0;
                        return bTime - aTime;
                    });
                    const winningConv = sortedConvs[0];
                    setExpandedProjects({ [runningProj.project_id]: true });
                    selectConversation(winningConv, runningProj);
                    if (isStaleOrEmptyConversation(winningConv)) {
                        setExpandedStaleGroups({ [runningProj.project_id]: true });
                    } else {
                        setExpandedStaleGroups({});
                    }
                    return;
                }
            }

            // 3. Fallback: Auto-expand the first non-archived project in prioritized order and select its newest conversation
            const firstProj = prioritized[0];
            if (firstProj) {
                setExpandedProjects({ [firstProj.project_id]: true });
                const candidateConvs = firstProj.conversations.filter(
                    (c) => !isGhostConversation(c) && !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
                );
                if (candidateConvs.length > 0) {
                    const sortedConvs = [...candidateConvs].sort((a, b) => {
                        const aTime = new Date(a.last_modified).getTime() || 0;
                        const bTime = new Date(b.last_modified).getTime() || 0;
                        return bTime - aTime;
                    });
                    const winningConv = sortedConvs[0];
                    selectConversation(winningConv, firstProj);
                    if (isStaleOrEmptyConversation(winningConv)) {
                        setExpandedStaleGroups({ [firstProj.project_id]: true });
                    } else {
                        setExpandedStaleGroups({});
                    }
                } else {
                    setSelectedProject(firstProj);
                    setSelectedConversation(null);
                }
            }
        },
        [selectConversation, initialSelectedProjectId]
    );

    const loadTree = async (
        isInitialLoad: boolean = true,
        isForce: boolean = true,
        overrideArchivedIds?: string[],
        overridePinnedIds?: string[]
    ) => {
        if (isInitialLoad) {
            setIsLoading(true);
        } else {
            setIsAutoSyncing(true);
        }
        setError(null);
        try {
            const data = await invoke<AgmProjectTreeNode[]>('get_project_conversation_tree', {
                instanceId: instanceId || undefined,
                maxWords: 2000,
                onlyRunning: false,
                force: isForce,
            });
            const isTargetDefault = instanceId === 'default' || instanceId === '__default__';
            const relevant = instanceId
                ? data
                      .filter((p) => {
                          if (p.instance_id === instanceId) return true;
                          if (isTargetDefault && (!p.instance_id || p.instance_id === 'default' || p.instance_id === '__default__')) {
                              return true;
                          }
                          return false;
                      })
                      .map((p) => ({
                          ...p,
                          conversations: (p.conversations || []).filter((c) => {
                              if (c.instance_id === instanceId) return true;
                              if (isTargetDefault && (!c.instance_id || c.instance_id === 'default' || c.instance_id === '__default__')) {
                                  return true;
                              }
                              return false;
                          }),
                      }))
                : data;
            const finalData = relevant;
            setTreeData(finalData);

            if (isInitialLoad) {
                const targetArchived = overrideArchivedIds || archivedProjectIds;
                const targetPinned = overridePinnedIds || pinnedProjectIds;
                performAutoSelection(finalData, targetArchived, targetPinned);
            } else {
                // Background sync: match existing selected conversation in place
                const currentConv = selectedConversationRef.current;
                const currentProj = selectedProjectRef.current;
                const currentActiveText = activePromptTextRef.current;

                if (currentConv && currentProj) {
                    const updatedProj = finalData.find((p) => p.project_id === currentProj.project_id);
                    if (updatedProj) {
                        setSelectedProject(updatedProj);
                        const updatedConv = updatedProj.conversations.find(
                            (c) => c.conversation_id === currentConv.conversation_id
                        );
                        if (updatedConv) {
                            setSelectedConversation(updatedConv);
                            const freshText = updatedConv.full_prompt_text || updatedConv.prompt_preview_200w || '';
                            setActivePromptText(freshText);
                            // Do not clobber user's dirty textarea edits
                            setEditedPromptText((prev) => (prev === currentActiveText ? freshText : prev));
                        }
                    }
                }
            }
        } catch (err: any) {
            if (isInitialLoad) {
                setError(err?.toString() || 'Failed to load project and conversation tree');
            } else {
                console.warn('Background auto-sync failed:', err);
            }
        } finally {
            if (isInitialLoad) {
                setIsLoading(false);
            } else {
                setIsAutoSyncing(false);
            }
        }
    };

    // Modal open effect
    useEffect(() => {
        if (isOpen) {
            const latestArchived = getArchivedProjectsForInstance(instanceId);
            const latestPinned = (() => {
                try {
                    const stored = localStorage.getItem(`agm_pinned_projects_${instanceId || 'default'}`);
                    return stored ? JSON.parse(stored) : [];
                } catch {
                    return [];
                }
            })();
            setArchivedProjectIds(latestArchived);
            setPinnedProjectIds(latestPinned);
            loadTree(true, true, latestArchived, latestPinned);
        }
    }, [isOpen, instanceId, initialSelectedProjectId]);

    // Configurable auto-sync interval timer (minimum 15s floor)
    useEffect(() => {
        if (!isOpen || syncInterval === 'off') return;

        let intervalMs = 30000;
        if (syncInterval === '15s') intervalMs = 15000;
        else if (syncInterval === '30s') intervalMs = 30000;
        else if (syncInterval === '1m') intervalMs = 60000;
        else if (syncInterval === '2m') intervalMs = 120000;

        const safeIntervalMs = Math.max(intervalMs, 15000);

        const timer = setInterval(() => {
            loadTree(false, true);
        }, safeIntervalMs);

        return () => clearInterval(timer);
    }, [isOpen, syncInterval, instanceId]);


    // Handle Backup Prompts
    const handleBackup = async () => {
        try {
            setActionMsg('Backing up prompts...');
            let backedUpDb: any[] = [];
            try {
                backedUpDb = await invoke<any[]>('list_backed_up_prompts');
            } catch (_) {}

            const fullBackupPayload = {
                version: '1.0',
                exportedAt: new Date().toISOString(),
                instanceId: instanceId || 'all',
                instanceName: instanceName || 'All Instances',
                projects: treeData,
                activePrompts: backedUpDb,
                totalProjects: treeData.length,
                totalConversations: treeData.reduce((sum, p) => sum + p.conversations.length, 0),
            };

            const jsonStr = JSON.stringify(fullBackupPayload, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `prompts-backup-${instanceId || 'all'}-${Date.now()}.json`;
            a.click();
            URL.revokeObjectURL(url);
            setActionMsg(`Backed up ${fullBackupPayload.totalProjects} project(s) and ${fullBackupPayload.totalConversations} conversation(s)!`);
            setTimeout(() => setActionMsg(null), 3000);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to backup prompts');
        }
    };

    // Handle File-based Restore
    const handleFileRestore = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;
        const reader = new FileReader();
        reader.onload = async (event) => {
            try {
                setActionMsg('Importing and persisting prompts backup into SQLite...');
                const content = event.target?.result as string;
                JSON.parse(content);
                const restoredCount = await invoke<number>('restore_prompts_backup', { backupJson: content });
                setActionMsg(`Successfully restored and persisted ${restoredCount} prompts in strict FIFO order!`);
                await loadTree(true, true);
                setTimeout(() => {
                    setActionMsg(null);
                }, 3500);
            } catch (err: any) {
                setError(`Restore failed: ${err?.message || err || 'Invalid backup format'}`);
            }
        };
        reader.readAsText(file);
        e.target.value = '';
    };

    // Handle Restore Prompts
    const handleRestore = async () => {
        if (restoreFileInputRef.current) {
            restoreFileInputRef.current.click();
        } else {
            try {
                setActionMsg('Restoring prompts with 7s channel stabilization...');
                await invoke('resume_recent_project_prompts', {
                    instanceId: instanceId || 'default',
                    maxAgeSeconds: 3600,
                });
                setActionMsg('Prompts restoration dispatched!');
                setTimeout(() => {
                    setActionMsg(null);
                    loadTree(true, true);
                }, 3000);
            } catch (err: any) {
                setError(err?.toString() || 'Failed to restore prompts');
            }
        }
    };

    // Handle Copy with Images (HTML rich clipboard formatting with image tags)
    const handleCopyWithImagesRich = async () => {
        try {
            let htmlContent = activePromptText
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
    };

    // Handle Save Extracted Images to files
    const handleSaveImages = () => {
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
    };

    // Handle Resend / Send Now Prompt Action (Immediate injection / writes .antigravity_resume_task.json)
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

        if (!conv && !proj) return;

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

            // 2. Copy prompt content to clipboard so user can paste immediately (manual fallback)
            try {
                await navigator.clipboard.writeText(promptContent);
            } catch {}

            setActionMsg("Prompt dispatched to IDE & copied to clipboard!");
            setTimeout(() => setActionMsg(null), 3000);
            // 4. Focus IDE instance window if workspace is known (avoids secondary launch hazard)
            try {
                if (repoPath) {
                    const repoName = repoPath.split(/[/\\]/).filter(Boolean).pop() || repoPath;
                    await focusInstanceWorkspace(targetInstId, repoPath, repoName);
                }
            } catch (focusErr) {
                console.warn('focusInstanceWorkspace error', focusErr);
            }

            if (dispatchOk) {
                setActionMsg("Prompt Dispatched & Focused IDE (via Hotkey 'N' / Send Now)!");
            } else {
                setError(`Send failed: ${dispatchErr?.toString() || 'dispatch rejected'} — prompt copied to clipboard; paste it into the IDE manually.`);
            }
            setTimeout(() => setActionMsg(null), 3500);
        } catch (err: any) {
            setError(err?.toString() || 'Failed to dispatch prompt');
        } finally {
            setIsResending(false);
        }
    }, [selectedConversation, selectedProject, treeData, archivedProjectIds, editedPromptText, activePromptText, confirmationSuffix, instanceId, selectConversation]);

    const handleExport = useCallback((format: 'md' | 'json') => {
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
    }, [selectedConversation, selectedProject, activePromptText]);

    // Modal-level hotkey listener: 'N' or 'n' key triggers Send Now / Resend prompt
    useEffect(() => {
        if (!isOpen) return;

        const handleKeyDown = (e: KeyboardEvent) => {
            // Ignore if modifier keys are pressed
            if (e.ctrlKey || e.altKey || e.metaKey) return;

            // Check if user is typing inside an input, textarea, select or contentEditable element
            const target = e.target as HTMLElement | null;
            if (target) {
                const tagName = target.tagName?.toLowerCase();
                if (
                    tagName === 'input' ||
                    tagName === 'textarea' ||
                    tagName === 'select' ||
                    target.isContentEditable ||
                    target.getAttribute('contenteditable') === 'true'
                ) {
                    return;
                }
            }

            if (e.key === 'n' || e.key === 'N') {
                e.preventDefault();
                handleResendPrompt();
            }
        };

        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, handleResendPrompt]);

    // Handle Enqueue Prompt Action (Calls FIFO scheduler queue / writes queue task)
    const handleEnqueuePrompt = async () => {
        if (!selectedConversation) return;
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
        } catch (err: any) {
            setError(err?.toString() || 'Failed to enqueue prompt');
        } finally {
            setIsEnqueueing(false);
        }
    };

    const handleFocusIde = async () => {
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
        } catch (err: any) {
            setError(err?.toString() || 'Failed to focus IDE');
        } finally {
            setIsFocusing(false);
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
        setShowAllWords(true);
        setInspectorPrompt({
            title: conv.title || conv.short_id,
            text,
            convId: conv.conversation_id,
            projectPath: repoPath,
            images,
            files,
        });
    };

    const archivedCount = useMemo(() => {
        return treeData.filter((p) => archivedProjectIds.includes(p.project_id)).length;
    }, [treeData, archivedProjectIds]);

    // Search filter & multi-tier sorting across projects, partitioned into active & archived
    const { activeProjects, archivedProjects } = useMemo(() => {
        let list = [...treeData];

        // 1. Search filter
        if (searchQuery.trim()) {
            const q = searchQuery.toLowerCase();
            list = list.filter(
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
        }

        // 1.5 Project Scope Filter
        if (selectedProjectScope !== 'all') {
            list = list.filter((p) => p.project_id === selectedProjectScope);
        }

        // 2. Filter Pills
        if (activeFilter === 'running') {
            list = list.filter((p) => Boolean(p.is_running) || p.conversations.some((c) => Boolean(c.is_running)));
        } else if (activeFilter === 'queued') {
            list = list.filter((p) => p.conversations.some((c) => Boolean(c.is_queued) || c.status.toLowerCase().includes('queue')));
        } else if (activeFilter === 'pinned') {
            list = list.filter((p) => pinnedProjectIds.includes(p.project_id));
        } else if (activeFilter === 'latest_conv') {
            list = list.filter((p) => p.conversations.length > 0);
        } else if (activeFilter === 'latest_prompt') {
            list = list.filter((p) => p.conversations.some((c) => c.step_count > 0 || Boolean(c.prompt_preview_200w)));
        } else if (activeFilter === 'archived') {
            list = list.filter((p) => archivedProjectIds.includes(p.project_id));
        }

        // 2.5 Category Filter (User vs Subagents vs System/Tools)
        if (categoryFilter === 'user') {
            list = list.filter((p) =>
                p.conversations.some(
                    (c) => classifyPromptTier(c.prompt_preview_200w, c.title, c).tier === 'USER_PROMPT'
                )
            );
        } else if (categoryFilter === 'subagent') {
            list = list.filter((p) =>
                p.conversations.some(
                    (c) => classifyPromptTier(c.prompt_preview_200w, c.title, c).tier === 'SUBAGENT_INSTRUCTION'
                )
            );
        } else if (categoryFilter === 'system') {
            list = list.filter((p) =>
                p.conversations.some((c) => {
                    const t = classifyPromptTier(c.prompt_preview_200w, c.title, c).tier;
                    return t === 'SYSTEM_MESSAGE' || t === 'TOOL_OUTPUT';
                })
            );
        }

        // 3. Multi-Tier Prioritization:
        // Pinned first -> Actively running projects -> Recent activity (last_modified) -> Alphabetical
        const sortFn = (a: AgmProjectTreeNode, b: AgmProjectTreeNode) => {
            const aPinned = pinnedProjectIds.includes(a.project_id);
            const bPinned = pinnedProjectIds.includes(b.project_id);
            if (aPinned !== bPinned) return aPinned ? -1 : 1;

            const aRunning = Boolean(a.is_running) || a.conversations.some((c) => Boolean(c.is_running));
            const bRunning = Boolean(b.is_running) || b.conversations.some((c) => Boolean(c.is_running));
            if (aRunning !== bRunning) return aRunning ? -1 : 1;

            const aLatest = Math.max(0, ...a.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            const bLatest = Math.max(0, ...b.conversations.map((c) => new Date(c.last_modified).getTime() || 0));
            if (bLatest !== aLatest) return bLatest - aLatest;

            return a.repo_name.localeCompare(b.repo_name);
        };

        if (activeFilter === 'archived') {
            return {
                activeProjects: [],
                archivedProjects: [...list].sort(sortFn),
            };
        }

        const activeList = list.filter((p) => !archivedProjectIds.includes(p.project_id)).sort(sortFn);
        const archivedList = list.filter((p) => archivedProjectIds.includes(p.project_id)).sort(sortFn);

        return {
            activeProjects: activeList,
            archivedProjects: archivedList,
        };
    }, [treeData, searchQuery, activeFilter, categoryFilter, pinnedProjectIds, archivedProjectIds, selectedProjectScope]);

    // Sort conversations inside each project: Running at top -> Descending by last_modified
    const sortConversations = useCallback(
        (convs: AgmConversationNode[]) => {
            let sorted = [...convs];
            if (activeFilter === 'running') {
                sorted = sorted.filter((c) => Boolean(c.is_running));
            }
            return sorted.sort((a, b) => {
                const aRunning = Boolean(a.is_running);
                const bRunning = Boolean(b.is_running);
                if (aRunning !== bRunning) {
                    return aRunning ? -1 : 1;
                }
                const aTime = new Date(a.last_modified).getTime() || 0;
                const bTime = new Date(b.last_modified).getTime() || 0;
                return bTime - aTime;
            });
        },
        [activeFilter]
    );

    // Active prompt truncation for Preview mode (default preview word threshold: 120 words)
    const { displayText: previewDisplayText, isTruncated, totalWords } = useMemo(() => {
        return getTruncatedText(activePromptText, 120);
    }, [activePromptText]);

    const displayedMarkdown = showAllWords || !isTruncated ? activePromptText : previewDisplayText;
    const activeWordCount = totalWords || countWords(activePromptText);
    const activeByteCount = useMemo(() => {
        return new TextEncoder().encode(activePromptText).length;
    }, [activePromptText]);

    const hasImages = useMemo(() => {
        const text = activePromptText || selectedConversation?.prompt_preview_200w || '';
        return /!\[.*?\]\((?:https?:\/\/.*?|data:image\/.*?;base64,.*?|[^\s)]+)\)/.test(text);
    }, [activePromptText, selectedConversation]);

    const formatByteSize = (bytes: number): string => {
        if (bytes < 1024) return `${bytes} B`;
        return `${(bytes / 1024).toFixed(1)} KB`;
    };

    const instanceSeqNum = selectedConversation?.instance_seq_num
        ?? selectedProject?.instance_seq_num
        ?? 1;

    const instanceExeName = selectedConversation?.instance_exe_name
        ?? selectedProject?.instance_exe_name
        ?? 'Antigravity.exe';

    const instanceNameDisplay = selectedConversation?.instance_name
        ?? selectedProject?.instance_name
        ?? instanceName
        ?? 'Default';

    const renderConversationNode = (conv: AgmConversationNode, project: AgmProjectTreeNode) => {
        const isConvSelected = selectedConversation?.conversation_id === conv.conversation_id;
        const isRunning = Boolean(conv.is_running) && !isGhostConversation(conv) && !(conv.prompt_word_count === 0 && (!conv.prompt_preview_200w || !conv.prompt_preview_200w.trim()));
        const isQueued = Boolean(conv.is_queued) || conv.status.toLowerCase().includes('queue');
        const promptCategory = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
        const IconComponent =
            promptCategory.tier === 'SUBAGENT_INSTRUCTION' ? Bot :
            promptCategory.tier === 'SYSTEM_MESSAGE' ? Terminal :
            promptCategory.tier === 'TOOL_OUTPUT' ? Wrench : User;

        return (
            <div
                key={conv.conversation_id}
                onClick={() => selectConversation(conv, project)}
                onDoubleClick={() => openInspector(conv, project.repo_path)}
                className={cn(
                    'group flex items-center justify-between rounded-[5px] px-2 py-1.5 text-xs cursor-pointer transition-colors',
                    isConvSelected
                        ? 'bg-blue-600 text-white font-medium shadow-2xs'
                        : 'text-slate-600 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#0c2438]'
                )}
                title={`${conv.title || 'Conversation'} · ${conv.step_count || 1} steps · Click to view`}
            >
                <div className="flex items-center gap-1.5 min-w-0">
                    <span title={promptCategory.roleBadge || promptCategory.tier} className="shrink-0 flex items-center">
                        <IconComponent
                            className={cn('h-3.5 w-3.5 shrink-0', isConvSelected ? 'text-white' : (promptCategory.tier === 'SUBAGENT_INSTRUCTION' ? 'text-purple-500' : promptCategory.tier === 'SYSTEM_MESSAGE' ? 'text-zinc-500' : promptCategory.tier === 'TOOL_OUTPUT' ? 'text-amber-500' : 'text-sky-500'))}
                        />
                    </span>
                    <span
                        className={cn(
                            'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
                            isConvSelected
                                ? 'bg-blue-700/80 text-white'
                                : 'bg-slate-200/90 dark:bg-[#15334d] text-slate-600 dark:text-cyan-300'
                        )}
                        title={conv.short_id ? `GitMap SHA: ${conv.short_id}` : (conv.gitmap_seq_code ? `GitMap: ${conv.gitmap_seq_code}` : undefined)}
                    >
                        {formatSeqBadge(conv.seq_code, 'C001')}
                    </span>
                    {(conv.repeat_badge || (conv.repeat_count && conv.repeat_count > 1)) && (
                        <span
                            className={cn(
                                "px-1.5 py-0.2 rounded-full text-[8.5px] font-mono font-bold shrink-0 border",
                                isConvSelected
                                    ? "bg-amber-300 text-amber-950 border-amber-200"
                                    : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30"
                            )}
                            title={`${conv.repeat_count || 2} duplicate/repeated prompt runs`}
                        >
                            {conv.repeat_badge || `x${conv.repeat_count}`}
                        </span>
                    )}
                    <span className="truncate text-[11px]">
                        {conv.title || conv.short_id || conv.conversation_id.slice(0, 8)}
                    </span>
                </div>
                <div className="flex items-center gap-1.5 shrink-0">
                    {isRunning && (
                        <span className={cn(
                            "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border animate-pulse",
                            isConvSelected
                                ? "bg-emerald-400 text-emerald-950 border-emerald-300"
                                : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
                        )}>
                            <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                            RUNNING
                        </span>
                    )}
                    {isQueued && !isRunning && (
                        <span className={cn(
                            "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border",
                            isConvSelected
                                ? "bg-amber-300 text-amber-950 border-amber-200"
                                : "bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/40"
                        )}>
                            <Clock className="w-2.5 h-2.5 text-amber-500" />
                            <span>QUEUED</span>
                        </span>
                    )}
                </div>
            </div>
        );
    };

    const renderConversationListWithGrouping = (convList: AgmConversationNode[], project: AgmProjectTreeNode) => {
        if (categoryFilter === 'subagent') {
            return convList.map((conv) => renderConversationNode(conv, project));
        }

        const hierarchy = assembleConversationHierarchy(convList);

        return hierarchy.map((root) => {
            return (
                <div key={`root-${root.primaryNode.conversation_id}`} className="space-y-0.5">
                    {renderConversationNode(root.primaryNode, project)}

                    {/* Rendering Child Sub-Points (Indented with Branch Glyphs) */}
                    {root.subagents.length > 0 && (
                        <div className="pl-5 space-y-0.5 border-l-2 border-purple-300/40 dark:border-purple-800/40 ml-3.5 my-0.5">
                            {root.subagents.map((subNode, subIdx) => {
                                const isSubSelected = selectedConversation?.conversation_id === subNode.primaryNode.conversation_id;
                                const isSubRunning = Boolean(subNode.primaryNode.is_running) &&
                                    !isGhostConversation(subNode.primaryNode) &&
                                    !(subNode.primaryNode.prompt_word_count === 0 && (!subNode.primaryNode.prompt_preview_200w || !subNode.primaryNode.prompt_preview_200w.trim()));
                                const isLast = subIdx === root.subagents.length - 1;
                                return (
                                    <div
                                        key={subNode.primaryNode.conversation_id}
                                        onClick={() => selectConversation(subNode.primaryNode, project)}
                                        onDoubleClick={() => openInspector(subNode.primaryNode, project.repo_path)}
                                        className={cn(
                                            "relative flex items-center justify-between rounded-[5px] px-2 py-1 text-xs cursor-pointer transition-colors group",
                                            isSubSelected
                                                ? "bg-purple-600 text-white font-medium shadow-2xs"
                                                : "text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438]"
                                        )}
                                        title={`AI Subagent Task · ${subNode.primaryNode.title || 'Subagent'} · ${subNode.primaryNode.step_count || 1} steps · Click to view`}
                                    >
                                        <div className="flex items-center gap-1.5 min-w-0">
                                            {/* Tree Branch Connector Glyph: ↳ or └── */}
                                            <span className="text-purple-400 dark:text-purple-400 font-mono text-[11px] select-none shrink-0">
                                                {isLast ? '└──' : '↳'}
                                            </span>
                                            <span title={subNode.classification.roleBadge || 'Subagent'} className="shrink-0 flex items-center">
                                                <Bot
                                                    className={cn("w-3.5 h-3.5 shrink-0", isSubSelected ? "text-white" : "text-purple-500")}
                                                />
                                            </span>
                                            <span
                                                className={cn(
                                                    'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
                                                    isSubSelected
                                                        ? 'bg-purple-700 text-white'
                                                        : 'bg-slate-200/90 dark:bg-[#15334d] text-slate-600 dark:text-cyan-300'
                                                )}
                                                title={subNode.primaryNode.short_id ? `GitMap SHA: ${subNode.primaryNode.short_id}` : (subNode.primaryNode.gitmap_seq_code ? `GitMap: ${subNode.primaryNode.gitmap_seq_code}` : undefined)}
                                            >
                                                {formatSeqBadge(subNode.primaryNode.seq_code, 'C001')}
                                            </span>
                                            <span className="truncate text-[11px]">
                                                {subNode.primaryNode.title || 'Subagent Task'}
                                            </span>
                                        </div>
                                        <div className="flex items-center gap-1.5 shrink-0">
                                            {isSubRunning && (
                                                <span className={cn(
                                                    "flex items-center gap-1 px-1.5 py-0.2 rounded-full text-[8.5px] font-bold font-mono border animate-pulse",
                                                    isSubSelected
                                                        ? "bg-emerald-400 text-emerald-950 border-emerald-300"
                                                        : "bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border-emerald-500/40"
                                                )}>
                                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                                                    RUNNING
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    )}
                </div>
            );
        });
    };

    const renderProjectNode = (project: AgmProjectTreeNode) => {
        const isProjectExpanded = Boolean(expandedProjects[project.project_id]);
        const isProjectSelected = selectedProject?.project_id === project.project_id;
        const isPinned = pinnedProjectIds.includes(project.project_id);
        const isArchived = archivedProjectIds.includes(project.project_id);

        const filteredConvs = project.conversations.filter((c) => {
            if (activeFilter === 'archived') return true;
            return !isGhostConversation(c);
        });

        const totalProjectPrompts = filteredConvs.reduce(
            (sum, c) => sum + (c.step_count > 0 ? c.step_count : 1),
            0
        );

        const sortedConvs = sortConversations(project.conversations);
        const activeConversations: AgmConversationNode[] = [];
        const staleConversations: AgmConversationNode[] = [];

        sortedConvs.forEach((conv) => {
            if (isGhostConversation(conv) && activeFilter !== 'archived') {
                return;
            }
            const cat = classifyPromptTier(conv.prompt_preview_200w, conv.title, conv);
            if (categoryFilter === 'all') {
                if (cat.isNonPrompt && !searchQuery.trim()) return;
            } else if (categoryFilter === 'user') {
                if (cat.tier !== 'USER_PROMPT') return;
            } else if (categoryFilter === 'subagent') {
                if (cat.tier !== 'SUBAGENT_INSTRUCTION') return;
            } else if (categoryFilter === 'system') {
                if (cat.tier !== 'SYSTEM_MESSAGE' && cat.tier !== 'TOOL_OUTPUT') return;
            }
            if (activeFilter === 'running' && !conv.is_running) {
                return;
            }
            if (activeFilter === 'queued' && !conv.is_queued && !conv.status.toLowerCase().includes('queue')) {
                return;
            }
            if (isStaleOrEmptyConversation(conv)) {
                staleConversations.push(conv);
            } else {
                activeConversations.push(conv);
            }
        });

        const isStaleGroupExpanded = Boolean(expandedStaleGroups[project.project_id]);
        const runningConversations = project.conversations.filter(
            (c) => Boolean(c.is_running) &&
                   !isGhostConversation(c) &&
                   !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
        );
        const runningCount = runningConversations.length;
        const queuedConversations = project.conversations.filter((c) => (Boolean(c.is_queued) || c.status.toLowerCase().includes('queue')) && !c.is_running);
        const queuedCount = queuedConversations.length;

        return (
            <div key={project.project_id} className="space-y-0.5">
                {/* Layer 1: Project Row */}
                <div
                    onClick={() => {
                        setSelectedProject(project);
                        setShowAllWords(false);
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
                    title={`Project: ${project.repo_name} · ${totalProjectPrompts} prompts total`}
                >
                    <div className="flex items-center gap-1.5 min-w-0">
                        {isProjectExpanded ? (
                            <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                        ) : (
                            <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                        )}
                        <Folder className="h-4 w-4 text-amber-500 shrink-0" />
                        <span
                            className={cn(
                                'text-[9px] font-mono px-1 py-0.2 rounded-[3px] shrink-0 font-medium whitespace-nowrap',
                                isProjectSelected
                                    ? 'bg-blue-600/20 text-blue-700 dark:text-cyan-300 border border-blue-500/30'
                                    : 'bg-slate-200 dark:bg-[#15334d] text-slate-600 dark:text-cyan-400'
                            )}
                            title={project.gitmap_seq_code || (project.seq_id ? `Project #${project.seq_id}` : undefined)}
                        >
                            {formatSeqBadge(project.seq_code, '#P001')}
                        </span>
                        <span className="truncate">{project.repo_name}</span>
                    </div>
                    <div className="flex items-center gap-1 shrink-0">
                        {runningCount > 0 && (
                            <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-emerald-500/10 border border-emerald-500/30">
                                <span className="relative flex h-2 w-2">
                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                    <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d] shadow-[0_0_6px_rgba(26,241,141,0.9)] animate-pulse"></span>
                                </span>
                                <span className="text-[9px] font-bold font-mono text-emerald-700 dark:text-[#1af18d]">
                                    {runningCount} RUNNING
                                </span>
                            </div>
                        )}
                        {queuedCount > 0 && (
                            <div className="flex items-center gap-1.5 px-2 py-0.5 rounded-full bg-amber-500/10 border border-amber-500/30">
                                <Clock className="w-2.5 h-2.5 text-amber-500 shrink-0" />
                                <span className="text-[9px] font-bold font-mono text-amber-700 dark:text-amber-300">
                                    {queuedCount} QUEUED
                                </span>
                            </div>
                        )}
                        {/* Project Action Button Cluster - Hover Only */}
                        <div className={cn(
                            "flex items-center gap-0.5 transition-opacity duration-150 shrink-0",
                            isPinned || isArchived || refreshingProjectId === project.project_id
                                ? "opacity-100"
                                : "opacity-0 group-hover:opacity-100"
                        )}>
                            <button
                                type="button"
                                onClick={(e) => handleRefreshSingleProject(project.project_id, e)}
                                disabled={refreshingProjectId === project.project_id}
                                className="p-1 rounded-[5px] text-slate-400 hover:text-blue-500 hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                title="Refresh this project"
                            >
                                <RotateCw className={cn("w-3 h-3", refreshingProjectId === project.project_id && "animate-spin text-blue-500")} />
                            </button>
                            <button
                                type="button"
                                onClick={(e) => togglePinProject(project.project_id, e)}
                                className={cn(
                                    "p-1 rounded-[5px] transition-colors cursor-pointer",
                                    isPinned
                                        ? "text-amber-500 bg-amber-50 dark:bg-amber-950/40"
                                        : "text-slate-400 hover:text-amber-500 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                                )}
                                title={isPinned ? "Unpin project" : "Pin project to top"}
                            >
                                <Pin className={cn("w-3 h-3", isPinned && "fill-current")} />
                            </button>
                            <button
                                type="button"
                                onClick={(e) => toggleArchiveProject(project.project_id, e)}
                                className={cn(
                                    "p-1 rounded-[5px] transition-colors cursor-pointer",
                                    isArchived
                                        ? "text-rose-500 bg-rose-50 dark:bg-rose-950/40"
                                        : "text-slate-400 hover:text-rose-500 hover:bg-slate-200 dark:hover:bg-[#15334d]"
                                )}
                                title={isArchived ? "Unarchive project" : "Archive / Less Favorite (Thumbs Down)"}
                            >
                                {isArchived ? (
                                    <ArchiveRestore className="w-3 h-3 text-rose-500" />
                                ) : (
                                    <Archive className="w-3 h-3" />
                                )}
                            </button>
                        </div>
                    </div>
                </div>

                {/* Layer 2: Conversations List */}
                {isProjectExpanded && (
                    <div className="ml-4 pl-2 border-l border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                        {/* Active Conversations */}
                        {renderConversationListWithGrouping(activeConversations, project)}

                        {/* Collapsible Stale / Empty Prompts Node */}
                        {staleConversations.length > 0 && (
                            <div className="space-y-0.5 mt-1 pt-1 border-t border-slate-200/50 dark:border-[#15334d]/40">
                                <div
                                    onClick={() => {
                                        setExpandedStaleGroups((prev) => ({
                                            ...prev,
                                            [project.project_id]: !isStaleGroupExpanded,
                                        }));
                                    }}
                                    className="group flex items-center justify-between rounded-[5px] px-2 py-1 text-xs font-medium text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438] cursor-pointer transition-colors"
                                    title="Toggle Archived / Stale Prompts"
                                >
                                    <div className="flex items-center gap-1.5 min-w-0">
                                        {isStaleGroupExpanded ? (
                                            <ChevronDown className="h-3 w-3 text-slate-400 shrink-0" />
                                        ) : (
                                            <ChevronRight className="h-3 w-3 text-slate-400 shrink-0" />
                                        )}
                                        <Folder className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                        <span className="truncate">
                                            Archived / Stale Prompts ({staleConversations.length})
                                        </span>
                                    </div>
                                    <span className="text-[10px] font-mono text-slate-400">
                                        {staleConversations.length}
                                    </span>
                                </div>

                                {isStaleGroupExpanded && (
                                    <div className="ml-4 pl-2 border-l border-dashed border-slate-200 dark:border-[#15334d]/60 space-y-0.5 py-0.5">
                                        {renderConversationListWithGrouping(staleConversations, project)}
                                    </div>
                                )}
                            </div>
                        )}
                    </div>
                )}
            </div>
        );
    };


    if (!isOpen) return null;

    const modalContent = (
        <div
            className={cn(
                "fixed inset-0 z-[300] flex items-center justify-center bg-black/60 backdrop-blur-xs animate-in fade-in duration-200",
                isFullscreen ? "p-0" : "p-4 sm:p-6"
            )}
        >
            <div
                className={cn(
                    "flex flex-col overflow-hidden bg-white dark:bg-[#0c2438] shadow-2xl transition-all",
                    isFullscreen
                        ? "h-full w-full rounded-none border-0"
                        : "h-[88vh] w-full max-w-6xl rounded-2xl border border-slate-200 dark:border-[#15334d]"
                )}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between border-b border-slate-200/80 dark:border-[#15334d] px-4 py-2.5 bg-slate-50/80 dark:bg-[#071a27] shrink-0">
                    <div className="flex items-center gap-2.5 min-w-0">
                        <div className="flex h-8 w-8 items-center justify-center rounded-[5px] bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20 shrink-0">
                            <Layers className="h-4 w-4" />
                        </div>
                        <div className="min-w-0">
                            <div className="flex items-center gap-2 flex-wrap min-w-0">
                                <h2 className="text-sm font-bold text-slate-900 dark:text-white shrink-0">
                                    Project & Prompt Tree View
                                </h2>
                                {sequenceNumber != null && sequenceNumber > 0 && (
                                    <span className="shrink-0 rounded-[5px] bg-slate-100 dark:bg-[#0c2438] px-1.5 py-0.5 text-[10px] font-bold text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#15334d]">
                                        #{sequenceNumber}
                                    </span>
                                )}
                                <span className="font-semibold text-slate-900 dark:text-white text-xs truncate max-w-[140px]" title={instanceName}>
                                    {instanceName}
                                </span>
                                {executablePath && (() => {
                                    const trailing = executablePath.replace(/^.*[\\/]/, '');
                                    return (
                                        <span
                                            className="hidden sm:inline-block shrink-0 rounded-[5px] bg-slate-100 dark:bg-[#0c2438] px-1.5 py-0.5 text-[10px] font-mono text-slate-500 dark:text-slate-400 border border-slate-200 dark:border-[#15334d] max-w-[150px] truncate"
                                            title={executablePath}
                                        >
                                            ...\{trailing}
                                        </span>
                                    );
                                })()}
                            </div>
                        </div>
                    </div>

                    {/* Top Right Header Controls: Two Independent Segmented Dark-Glass Capsules */}
                    <div className="flex items-center gap-1.5 shrink-0">
                        {/* Capsule 1: Data Operations Capsule */}
                        <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
                            {/* Backup Button */}
                            <button
                                type="button"
                                onClick={handleBackup}
                                className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Backup Prompts to JSON"
                            >
                                <Download className="w-3.5 h-3.5 text-indigo-500" />
                                <span>Backup</span>
                            </button>

                            {/* Restore Button */}
                            <button
                                type="button"
                                onClick={handleRestore}
                                className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Restore Prompts from JSON backup file"
                            >
                                <Upload className="w-3.5 h-3.5 text-emerald-500" />
                                <span>Restore</span>
                            </button>

                            {/* Refresh Button */}
                            <button
                                type="button"
                                onClick={() => loadTree(true, true)}
                                disabled={isLoading}
                                className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-none transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Refresh Tree"
                            >
                                <RefreshCw className={cn("w-3.5 h-3.5 text-blue-500", isLoading && "animate-spin")} />
                                <span>Refresh</span>
                            </button>

                            {/* Sync Interval Selector with Transparent Inline Styling */}
                            <div className="flex items-center gap-1 pl-2 pr-1.5 py-0.5 text-[11px] font-semibold text-slate-700 dark:text-slate-200 rounded-r-full">
                                <Clock className={cn("w-3 h-3 text-cyan-500 shrink-0", isAutoSyncing && "animate-spin")} />
                                <span className="text-[10px] text-slate-500 dark:text-slate-400 select-none">Sync:</span>
                                <select
                                    value={syncInterval}
                                    onChange={(e) => {
                                        const val = e.target.value as SyncInterval;
                                        setSyncInterval(val);
                                        setPromptTreeSyncInterval(val);
                                    }}
                                    className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[11px] font-semibold focus:ring-0 focus:outline-none cursor-pointer pr-1 py-0"
                                    title="Auto-sync interval timer"
                                >
                                    <option value="15s" className="bg-white dark:bg-[#0c2438]">15s</option>
                                    <option value="30s" className="bg-white dark:bg-[#0c2438]">30s</option>
                                    <option value="1m" className="bg-white dark:bg-[#0c2438]">1m</option>
                                    <option value="2m" className="bg-white dark:bg-[#0c2438]">2m</option>
                                    <option value="off" className="bg-white dark:bg-[#0c2438]">Off</option>
                                </select>
                            </div>
                        </div>

                        {/* Capsule 2: Window Controls Capsule */}
                        <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
                            {/* Fullscreen Toggle */}
                            <button
                                type="button"
                                onClick={() => setIsFullscreen(!isFullscreen)}
                                className="flex items-center gap-1 px-2.5 py-1 text-[11px] font-semibold text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d]/90 rounded-l-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title={isFullscreen ? "Exit Full Screen" : "Full Screen Mode"}
                            >
                                {isFullscreen ? (
                                    <>
                                        <Minimize2 className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Exit</span>
                                    </>
                                ) : (
                                    <>
                                        <Maximize2 className="w-3.5 h-3.5 text-blue-500" />
                                        <span>Full</span>
                                    </>
                                )}
                            </button>

                            {/* Close Button with Rose Dark-Glass Hover */}
                            <button
                                type="button"
                                onClick={onClose}
                                className="flex items-center px-2 py-1 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 dark:hover:bg-rose-500/20 rounded-r-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                title="Close Modal"
                            >
                                <X className="w-3.5 h-3.5" />
                            </button>
                        </div>
                    </div>

                    <input
                        type="file"
                        ref={restoreFileInputRef}
                        accept=".json"
                        onChange={handleFileRestore}
                        style={{ display: 'none' }}
                    />
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
                    {/* Left Panel: 2-Tier Project -> Conversation Tree */}
                    <div className="w-80 sm:w-96 flex flex-col border-r border-slate-200 dark:border-[#15334d] bg-slate-50/50 dark:bg-[#071a27]/60">
                        {/* Search Input & Filter Controls */}
                        <div className="p-3 border-b border-slate-200 dark:border-[#15334d] shrink-0 space-y-2">
                            {/* Project Scope Dropdown */}
                            <div className="flex items-center gap-1.5 bg-white dark:bg-[#0c2438] px-2 py-1 rounded-[5px] border border-slate-200 dark:border-[#15334d]">
                                <Folder className="h-3.5 w-3.5 text-blue-500 shrink-0" />
                                <select
                                    value={selectedProjectScope}
                                    onChange={(e) => setSelectedProjectScope(e.target.value)}
                                    className="w-full text-xs font-medium bg-transparent text-slate-800 dark:text-slate-200 focus:outline-none cursor-pointer"
                                    title="Filter tree by project scope"
                                >
                                    <option value="all" className="dark:bg-[#0c2438]">All Projects ({treeData.length})</option>
                                    {treeData.map((p) => (
                                        <option key={p.project_id} value={p.project_id} className="dark:bg-[#0c2438]">
                                            {p.repo_name} {p.is_running ? '●' : ''} ({p.conversations.length} convs)
                                        </option>
                                    ))}
                                </select>
                            </div>

                            {/* Search Input */}
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

                            {/* Row 1: 4-Tier Category Filter Segmented Capsule */}
                            <div className="flex items-center justify-between gap-1">
                                <div className="w-full inline-flex items-center rounded-full border border-slate-200 dark:border-[#15334d] bg-white/80 dark:bg-[#0c2438]/80 backdrop-blur-xs p-0.5 shadow-2xs divide-x divide-slate-200 dark:divide-[#15334d]">
                                    <button
                                        type="button"
                                        onClick={() => setCategoryFilter('all')}
                                        className={cn(
                                            "flex-1 py-1 text-[10px] font-medium rounded-l-full transition-colors text-center cursor-pointer",
                                            categoryFilter === 'all'
                                                ? "bg-blue-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="All prompt types"
                                    >
                                        All
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => setCategoryFilter('user')}
                                        className={cn(
                                            "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                            categoryFilter === 'user'
                                                ? "bg-sky-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="User Prompts submitted by humans"
                                    >
                                        <User className="w-2.5 h-2.5 shrink-0" />
                                        <span>User</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => setCategoryFilter('subagent')}
                                        className={cn(
                                            "flex-1 py-1 text-[10px] font-medium transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                            categoryFilter === 'subagent'
                                                ? "bg-purple-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="Automated subagent and background task instructions"
                                    >
                                        <Bot className="w-2.5 h-2.5 shrink-0" />
                                        <span>Subagent</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => setCategoryFilter('system')}
                                        className={cn(
                                            "flex-1 py-1 text-[10px] font-medium rounded-r-full transition-colors flex items-center justify-center gap-1 cursor-pointer",
                                            categoryFilter === 'system'
                                                ? "bg-slate-700 dark:bg-slate-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="System messages & tool outputs"
                                    >
                                        <Terminal className="w-2.5 h-2.5 shrink-0" />
                                        <span>System</span>
                                    </button>
                                </div>
                            </div>

                            {/* Row 2: Status Filter Segmented Capsule & Archive Scope */}
                            <div className="flex items-center justify-between gap-1 pt-0.5">
                                <div className="inline-flex items-center rounded-full border border-slate-200 dark:border-[#15334d] bg-white/80 dark:bg-[#0c2438]/80 backdrop-blur-xs p-0.5 shadow-2xs">
                                    <button
                                        type="button"
                                        onClick={() => setActiveFilter(activeFilter === 'running' ? 'all' : 'running')}
                                        className={cn(
                                            "px-2 py-0.5 text-[10px] font-medium rounded-full transition-colors whitespace-nowrap cursor-pointer flex items-center gap-1",
                                            activeFilter === 'running'
                                                ? "bg-emerald-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="Active running prompts"
                                    >
                                        <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'running' ? "bg-white" : "bg-emerald-500 animate-pulse")} />
                                        <span>Running</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => setActiveFilter(activeFilter === 'queued' ? 'all' : 'queued')}
                                        className={cn(
                                            "px-2 py-0.5 text-[10px] font-medium rounded-full transition-colors whitespace-nowrap cursor-pointer flex items-center gap-1",
                                            activeFilter === 'queued'
                                                ? "bg-amber-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                        title="Queued prompts"
                                    >
                                        <span className={cn("w-1.5 h-1.5 rounded-full", activeFilter === 'queued' ? "bg-white" : "bg-amber-500")} />
                                        <span>Queued</span>
                                    </button>
                                    <button
                                        type="button"
                                        onClick={() => setActiveFilter(activeFilter === 'pinned' ? 'all' : 'pinned')}
                                        className={cn(
                                            "px-2 py-0.5 text-[10px] font-medium rounded-full transition-colors whitespace-nowrap cursor-pointer flex items-center gap-0.5",
                                            activeFilter === 'pinned'
                                                ? "bg-blue-600 text-white shadow-2xs font-semibold"
                                                : "text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200"
                                        )}
                                    >
                                        <Pin className="w-2.5 h-2.5 shrink-0" />
                                        <span>Pinned</span>
                                    </button>
                                </div>

                                {/* Scope / State Dropdown (All vs Archived) */}
                                <select
                                    value={activeFilter === 'archived' ? 'archived' : 'all'}
                                    onChange={(e) => {
                                        const val = e.target.value;
                                        if (val === 'archived') setActiveFilter('archived');
                                        else setActiveFilter('all');
                                    }}
                                    className="text-[10px] font-medium bg-white dark:bg-[#0c2438] text-slate-700 dark:text-slate-300 border border-slate-200 dark:border-[#15334d] rounded-full px-2 py-0.5 focus:outline-none cursor-pointer"
                                >
                                    <option value="all" className="dark:bg-[#0c2438]">All ({treeData.length})</option>
                                    <option value="archived" className="dark:bg-[#0c2438]">Archived ({archivedCount})</option>
                                </select>
                            </div>
                        </div>

                        {/* Tree Items List */}
                        <div className="flex-1 overflow-y-auto p-3 space-y-1">
                            {activeFilter === 'archived' ? (
                                archivedProjects.length === 0 ? (
                                    <div className="py-12 text-center text-xs text-slate-400">
                                        No archived projects found.
                                    </div>
                                ) : (
                                    archivedProjects.map((project) => renderProjectNode(project))
                                )
                            ) : (
                                <>
                                    {activeProjects.length === 0 && archivedProjects.length === 0 ? (
                                        treeData.length === 0 ? (
                                            <div className="py-16 text-center space-y-3 px-6">
                                                <div className="w-12 h-12 rounded-full bg-slate-100 dark:bg-[#0c2438] flex items-center justify-center mx-auto text-slate-400">
                                                    <Folder className="w-6 h-6 text-slate-400" />
                                                </div>
                                                <div className="text-sm font-semibold text-slate-700 dark:text-slate-300">
                                                    No Projects Found in this Profile
                                                </div>
                                                <p className="text-xs text-slate-500 dark:text-slate-400 max-w-sm mx-auto">
                                                    No workspace storage or conversation records were found for instance &quot;{instanceName || instanceId}&quot;. Launch this instance and open a workspace to start logging prompts.
                                                </p>
                                            </div>
                                        ) : searchQuery.trim() ? (
                                            <div className="py-12 text-center text-xs text-slate-400 space-y-2">
                                                <div>No conversations match &quot;{searchQuery}&quot;</div>
                                                <button
                                                    onClick={() => setSearchQuery('')}
                                                    className="text-blue-500 hover:underline text-[11px] cursor-pointer"
                                                >
                                                    Clear search query
                                                </button>
                                            </div>
                                        ) : (
                                            <div className="py-12 text-center text-xs text-slate-400">
                                                No projects or conversations found.
                                            </div>
                                        )
                                    ) : (
                                        <>
                                            {activeProjects.map((project) => renderProjectNode(project))}

                                            {/* Collapsed at bottom: Archived Projects */}
                                            {archivedProjects.length > 0 && (
                                                <div className="space-y-0.5 mt-2 pt-2 border-t border-slate-200 dark:border-[#15334d]">
                                                    <div
                                                        onClick={() => setIsArchivedProjectsExpanded(!isArchivedProjectsExpanded)}
                                                        className="group flex items-center justify-between rounded-[5px] px-2.5 py-1.5 text-xs font-semibold text-slate-500 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#0c2438] cursor-pointer transition-colors"
                                                        title="Toggle Archived Projects"
                                                    >
                                                        <div className="flex items-center gap-1.5 min-w-0">
                                                            {isArchivedProjectsExpanded ? (
                                                                <ChevronDown className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                            ) : (
                                                                <ChevronRight className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                            )}
                                                            <Archive className="h-3.5 w-3.5 text-slate-400 shrink-0" />
                                                            <span className="truncate">Archived Projects ({archivedProjects.length})</span>
                                                        </div>
                                                        <span className="text-[10px] font-mono text-slate-400">
                                                            {archivedProjects.length}
                                                        </span>
                                                    </div>

                                                    {isArchivedProjectsExpanded && (
                                                        <div className="space-y-0.5 pl-1">
                                                            {archivedProjects.map((project) => renderProjectNode(project))}
                                                        </div>
                                                    )}
                                                </div>
                                            )}
                                        </>
                                    )}
                                </>
                            )}
                        </div>
                    </div>

                    {/* Right Panel: Prompt Details, Markdown Modes & Actions */}
                    <div className="flex-1 flex flex-col overflow-hidden bg-white dark:bg-[#0c2438]">
                        {selectedConversation ? (
                            <div className="flex-1 flex flex-col px-5 py-3.5 overflow-y-auto space-y-3">
                                {/* Prompt Content Header */}
                                <div className="space-y-2 pb-2.5 border-b border-slate-200 dark:border-[#15334d] shrink-0">
                                    {/* Row 1: Identity, Badges & Actions */}
                                    <div className="flex items-center justify-between gap-2.5 flex-wrap">
                                        {/* Left: Indicator 1 (Sequence + Tier), Indicator 2 (Status Capsule), Title, Instance Context */}
                                        <div className="flex items-center gap-2 flex-wrap min-w-0">
                                            {/* Indicator 1: Clean Sequence Code with compact Origin Tier Icon */}
                                            {(() => {
                                                const rawSeq = selectedConversation.seq_code || 'C001';
                                                const seqDisplay = formatSeqBadge(rawSeq, 'C001');
                                                const tierInfo = classifyPromptTier(activePromptText, selectedConversation.title);
                                                return (
                                                    <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-[5px] text-[10px] font-mono font-bold bg-blue-500/10 text-blue-600 dark:text-cyan-400 border border-blue-500/20" title={`${tierInfo.tier}`}>
                                                        {tierInfo.tier === 'USER_PROMPT' && <User className="w-2.5 h-2.5 text-sky-500 shrink-0" />}
                                                        {tierInfo.tier === 'SUBAGENT_INSTRUCTION' && <Bot className="w-2.5 h-2.5 text-purple-500 shrink-0" />}
                                                        {tierInfo.tier === 'SYSTEM_MESSAGE' && <Terminal className="w-2.5 h-2.5 text-slate-500 shrink-0" />}
                                                        {tierInfo.tier === 'TOOL_OUTPUT' && <Wrench className="w-2.5 h-2.5 text-amber-500 shrink-0" />}
                                                        <span>{seqDisplay}</span>
                                                    </span>
                                                );
                                            })()}

                                            {/* Indicator 2: Unified Status Capsule */}
                                            {Boolean(selectedConversation.is_running) && !isGhostConversation(selectedConversation) ? (
                                                <span className="inline-flex items-center gap-1.5 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-emerald-500/15 text-emerald-700 dark:text-[#1af18d] border border-emerald-500/40 shadow-2xs animate-pulse">
                                                    <span className="w-1.5 h-1.5 rounded-full bg-[#1af18d] animate-pulse" />
                                                    <span>RUNNING</span>
                                                    {instancePid ? <span className="opacity-80">PID: {instancePid}</span> : null}
                                                    <span className="border-l border-emerald-400/40 pl-1">{formatDuration(elapsedSeconds)}</span>
                                                </span>
                                            ) : Boolean(selectedConversation.is_queued) ? (
                                                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[9.5px] font-bold font-mono bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 shadow-2xs">
                                                    <Clock className="w-2.5 h-2.5 text-amber-500" />
                                                    <span>QUEUED</span>
                                                </span>
                                            ) : (
                                                <span className="inline-flex items-center px-1.5 py-0.5 rounded-full text-[9px] font-mono text-slate-500 dark:text-slate-400 bg-slate-100 dark:bg-slate-800/60 border border-slate-200 dark:border-slate-700">
                                                    IDLE
                                                </span>
                                            )}

                                            {/* Title */}
                                            <h3 className="text-xs font-bold text-slate-900 dark:text-white truncate max-w-[200px] md:max-w-xs" title={selectedConversation.title || selectedConversation.short_id}>
                                                {selectedConversation.title || selectedConversation.short_id}
                                            </h3>

                                            {/* Subtle Instance Context (Breadcrumb style) */}
                                            <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 truncate" title={`Instance #${instanceSeqNum} · ${instanceExeName} (${instanceNameDisplay})`}>
                                                #{formatSeqBadge(selectedConversation.seq_code, `C${String(instanceSeqNum).padStart(3, '0')}`).replace(/^#/, '')} · {instanceNameDisplay}
                                            </span>
                                        </div>

                                        {/* Right: The 2 Canonical Segmented Dark-Glass Action Capsules */}
                                        <div className="flex items-center gap-1.5 overflow-x-auto no-scrollbar shrink-0">
                                            {/* Capsule 1: Content & Export */}
                                            <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs shrink-0">
                                                {/* Copy Text Button */}
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        const cleanText = stripImagesFromPrompt(activePromptText);
                                                        navigator.clipboard.writeText(cleanText);
                                                        setIsCopiedText(true);
                                                        setTimeout(() => setIsCopiedText(false), 2000);
                                                    }}
                                                    className="flex items-center gap-1 px-2 py-0.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                                    title="Copy clean prompt text (excluding embedded images)"
                                                >
                                                    {isCopiedText ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-slate-500 dark:text-slate-400" />}
                                                    <span>{isCopiedText ? 'Copied!' : 'Copy'}</span>
                                                </button>

                                                {/* Copy With Images */}
                                                <button
                                                    type="button"
                                                    onClick={handleCopyWithImagesRich}
                                                    className={cn(
                                                        "flex items-center gap-1 px-2 py-0.5 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                                        !hasImages && "rounded-r-none"
                                                    )}
                                                    title="Copy prompt with rich embedded HTML images for document pasting"
                                                >
                                                    {isCopiedRaw ? <Check className="w-3 h-3 text-emerald-500" /> : <Copy className="w-3 h-3 text-indigo-500" />}
                                                    <span>{isCopiedRaw ? 'Copied + Imgs!' : '+ Imgs'}</span>
                                                </button>

                                                {/* Save Images (conditionally rendered only when images exist) */}
                                                {hasImages && (
                                                    <button
                                                        type="button"
                                                        onClick={handleSaveImages}
                                                        className="flex items-center gap-1 px-2 py-0.5 rounded-none text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                                        title="Extract and save embedded images"
                                                    >
                                                        <ImageIcon className="w-3 h-3 text-blue-500" />
                                                        <span>Save Imgs</span>
                                                    </button>
                                                )}

                                                {/* Export Dropdown */}
                                                <div className="relative inline-block rounded-r-full">
                                                    <button
                                                        type="button"
                                                        onClick={() => setIsExportMenuOpen(!isExportMenuOpen)}
                                                        className="flex items-center gap-1 px-2 py-0.5 rounded-r-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                                        title="Export Prompt to Markdown or JSON"
                                                    >
                                                        <Download className="w-3 h-3 text-amber-500" />
                                                        <span>Export</span>
                                                        <ChevronDown className="w-2.5 h-2.5 text-slate-400" />
                                                    </button>
                                                    {isExportMenuOpen && (
                                                        <div
                                                            className="absolute right-0 top-full mt-1.5 z-50 w-36 rounded-xl bg-white dark:bg-[#0c2438] border border-slate-200 dark:border-[#15334d] shadow-xl py-1 animate-in fade-in zoom-in-95 duration-100"
                                                            onMouseLeave={() => setIsExportMenuOpen(false)}
                                                        >
                                                            <button
                                                                type="button"
                                                                onClick={() => {
                                                                    handleExport('md');
                                                                    setIsExportMenuOpen(false);
                                                                }}
                                                                className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
                                                            >
                                                                <FileText className="w-3.5 h-3.5 text-cyan-500" />
                                                                <span>Export as .md</span>
                                                            </button>
                                                            <button
                                                                type="button"
                                                                onClick={() => {
                                                                    handleExport('json');
                                                                    setIsExportMenuOpen(false);
                                                                }}
                                                                className="w-full flex items-center gap-2 px-3 py-1.5 text-xs text-slate-700 dark:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#15334d] text-left cursor-pointer transition-colors"
                                                            >
                                                                <Code className="w-3.5 h-3.5 text-amber-500" />
                                                                <span>Export as .json</span>
                                                            </button>
                                                        </div>
                                                    )}
                                                </div>
                                            </div>

                                            {/* Capsule 2: Execution & Workflow */}
                                            <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs shrink-0">
                                                {/* Confirmation Suffix Dropdown */}
                                                <div className="flex items-center px-1.5 py-0.5 rounded-l-full">
                                                    <select
                                                        value={confirmationSuffix}
                                                        onChange={(e) => setConfirmationSuffix(e.target.value)}
                                                        className="bg-transparent border-0 text-slate-700 dark:text-slate-200 text-[10.5px] font-semibold focus:ring-0 focus:outline-none cursor-pointer max-w-[85px] truncate pr-1 py-0"
                                                        title="Confirmation suffix appended on Resend"
                                                    >
                                                        <option value="None (Send as is)" className="bg-white dark:bg-[#0c2438]">Suffix: None</option>
                                                        <option value="Is it done?" className="bg-white dark:bg-[#0c2438]">Done?</option>
                                                        <option value="Is it released?" className="bg-white dark:bg-[#0c2438]">Released?</option>
                                                        <option value="Are you sure about it?" className="bg-white dark:bg-[#0c2438]">Sure?</option>
                                                        <option value="Double check all edge cases" className="bg-white dark:bg-[#0c2438]">Edge cases</option>
                                                        <option value="Verify build and tests" className="bg-white dark:bg-[#0c2438]">Verify tests</option>
                                                    </select>
                                                </div>

                                                {/* Focus IDE Button */}
                                                <button
                                                    type="button"
                                                    onClick={handleFocusIde}
                                                    disabled={isFocusing}
                                                    className="flex items-center gap-1 px-2 py-0.5 text-sky-700 dark:text-sky-300 hover:bg-sky-100/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                                                    title="Focus Antigravity IDE window or launch instance"
                                                >
                                                    <ExternalLink className={cn('w-3 h-3 text-sky-500', isFocusing && 'animate-spin')} />
                                                    <span>Focus IDE</span>
                                                </button>

                                                {/* Send Now Button (Hotkey: N) */}
                                                <button
                                                    type="button"
                                                    onClick={handleResendPrompt}
                                                    disabled={isResending}
                                                    className="flex items-center gap-1 px-2 py-0.5 text-emerald-700 dark:text-emerald-300 hover:bg-emerald-100/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                                                    title="Immediately inject prompt to running instance (Hotkey: N)"
                                                >
                                                    <RotateCw className={cn('w-3 h-3 text-emerald-500', isResending && 'animate-spin')} />
                                                    <span>Send</span>
                                                    <kbd className="ml-0.5 px-1 py-0.2 bg-emerald-500/20 text-emerald-700 dark:text-emerald-300 rounded text-[8.5px] font-mono font-bold">N</kbd>
                                                </button>

                                                {/* Enqueue Button */}
                                                <button
                                                    type="button"
                                                    onClick={handleEnqueuePrompt}
                                                    disabled={isEnqueueing}
                                                    className="flex items-center gap-1 px-2 py-0.5 text-purple-700 dark:text-purple-300 hover:bg-purple-100/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer disabled:opacity-50 rounded-none"
                                                    title="Enqueue prompt into FIFO scheduler queue"
                                                >
                                                    <ListPlus className="w-3 h-3 text-purple-500" />
                                                    <span>Queue</span>
                                                </button>

                                                {/* Full Inspector */}
                                                <button
                                                    type="button"
                                                    onClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                                    className="flex items-center gap-1 px-2 py-0.5 text-blue-700 dark:text-cyan-300 hover:bg-blue-100/80 dark:hover:bg-[#15334d] text-[11px] font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer rounded-r-full"
                                                    title="Full-Screen Inspector"
                                                >
                                                    <Maximize2 className="h-3 w-3 text-blue-500" />
                                                    <span>Full</span>
                                                </button>
                                            </div>
                                        </div>
                                    </div>
                                </div>

                                {/* Row 2: Unified Navigation Tabs & View Controls */}
                                <div className="flex items-center justify-between gap-3 flex-wrap pb-1 border-b border-slate-200/60 dark:border-[#15334d]/60">
                                    {/* Primary Tabs Capsule: Prompt Instruction vs AI Results */}
                                    <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
                                        <button
                                            type="button"
                                            onClick={() => setPreviewTab('instruction')}
                                            className={cn(
                                                "flex items-center gap-1.5 px-3 py-1 rounded-l-full text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                                previewTab === 'instruction'
                                                    ? "bg-blue-600 text-white shadow-2xs"
                                                    : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                                            )}
                                            title="View original prompt instruction"
                                        >
                                            <MessageSquare className="w-3.5 h-3.5" />
                                            <span>Prompt Instruction</span>
                                        </button>

                                        <button
                                            type="button"
                                            onClick={() => setPreviewTab('results')}
                                            className={cn(
                                                "flex items-center gap-1.5 px-3 py-1 rounded-r-full text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer",
                                                previewTab === 'results'
                                                    ? "bg-blue-600 text-white shadow-2xs"
                                                    : "text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white"
                                            )}
                                            title="View AI execution outputs, responses and tools"
                                        >
                                            {selectedConversation.is_running ? (
                                                <span className="relative flex h-2 w-2">
                                                    <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                                    <span className="relative inline-flex rounded-full h-2 w-2 bg-[#1af18d]"></span>
                                                </span>
                                            ) : (
                                                <Terminal className="w-3.5 h-3.5 text-cyan-500" />
                                            )}
                                            <span>AI Results & Outputs</span>
                                            {(selectedConversation.execution_results || selectedConversation.latest_response) && (
                                                <span className="w-1.5 h-1.5 rounded-full bg-cyan-400" />
                                            )}
                                        </button>
                                    </div>

                                    {/* Right Controls: View Mode Capsule (Preview/Raw/Edit) + Expand Toggle + Details */}
                                    <div className="flex items-center gap-2 flex-wrap">
                                        {previewTab === 'instruction' && (
                                            <>
                                                <span className="text-xs font-semibold text-slate-500 dark:text-slate-400 font-mono hidden sm:inline">
                                                    {activeWordCount} words · {formatByteSize(activeByteCount)}
                                                </span>
                                                {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                                                    <button
                                                        type="button"
                                                        onClick={() => setShowAllWords(!showAllWords)}
                                                        className="flex items-center gap-1 px-2.5 py-0.5 rounded-full text-[10px] font-semibold bg-cyan-500/10 text-cyan-700 dark:text-cyan-300 border border-cyan-500/20 hover:bg-cyan-500/20 transition-all cursor-pointer"
                                                        title={showAllWords ? 'Collapse full prompt text' : 'Expand full prompt text'}
                                                    >
                                                        {showAllWords ? (
                                                            <>
                                                                <ChevronUp className="w-3 h-3" />
                                                                <span>Collapse</span>
                                                            </>
                                                        ) : (
                                                            <>
                                                                <ChevronDown className="w-3 h-3" />
                                                                <span>Expand Full</span>
                                                            </>
                                                        )}
                                                    </button>
                                                )}

                                                {/* View Mode Toggle Capsule */}
                                                <div className="inline-flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200/80 dark:border-[#15334d]/90 p-0.5 divide-x divide-slate-200/70 dark:divide-[#15334d]/80 shadow-2xs">
                                                    <button
                                                        type="button"
                                                        onClick={() => setViewMode('preview')}
                                                        className={cn(
                                                            'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-l-full transition-colors cursor-pointer',
                                                            viewMode === 'preview'
                                                                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#15334d]/60'
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
                                                            'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-none transition-colors cursor-pointer',
                                                            viewMode === 'raw'
                                                                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#15334d]/60'
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
                                                            'flex items-center gap-1.5 px-3 py-1 text-xs font-medium rounded-r-full transition-colors cursor-pointer',
                                                            viewMode === 'edit'
                                                                ? 'bg-blue-600 text-white font-semibold shadow-2xs'
                                                                : 'text-slate-600 dark:text-slate-300 hover:text-slate-900 dark:hover:text-white hover:bg-slate-200/60 dark:hover:bg-[#15334d]/60'
                                                        )}
                                                        title="Editable Textarea Mode"
                                                    >
                                                        <Edit3 className="w-3.5 h-3.5" />
                                                        <span>Edit</span>
                                                    </button>
                                                </div>
                                            </>
                                        )}

                                        <button
                                            type="button"
                                            onClick={() => setIsDetailsModalOpen(true)}
                                            className="inline-flex items-center gap-1 px-2.5 py-1 rounded-full text-xs font-medium bg-slate-100 dark:bg-[#071a27] text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                            title="View full conversation & project details"
                                        >
                                            <FileText className="w-3 h-3 text-slate-400" />
                                            <span>Details</span>
                                        </button>
                                    </div>
                                </div>

                                {/* Tab 1: Prompt Instruction View */}
                                {previewTab === 'instruction' && (
                                    <div className="space-y-3">

                                            {/* Preview Mode (Rich Markdown) */}
                                            {viewMode === 'preview' && (
                                                <div className="space-y-3">

                                            <div
                                                onDoubleClick={() => openInspector(selectedConversation, selectedProject?.repo_path || '')}
                                                className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-800 dark:text-slate-200 leading-relaxed max-h-[460px] overflow-y-auto cursor-pointer hover:border-blue-400 dark:hover:border-blue-500/50 transition-colors shadow-inner"
                                            >
                                                <RichMarkdownRenderer
                                                    content={displayedMarkdown}
                                                    showAllWords={showAllWords}
                                                    onToggleExpand={() => setShowAllWords(!showAllWords)}
                                                    isTruncated={isTruncated}
                                                />
                                                {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                                                    <div className="mt-3 pt-2 border-t border-slate-200/50 dark:border-[#15334d]/50 flex items-center">
                                                        <span
                                                            onClick={(e) => {
                                                                e.stopPropagation();
                                                                setShowAllWords(!showAllWords);
                                                            }}
                                                            title="Click to expand or collapse full prompt text"
                                                            className="cursor-pointer text-xs font-semibold text-cyan-600 dark:text-cyan-400 hover:underline flex items-center gap-1 select-none"
                                                        >
                                                            {showAllWords ? 'Collapse full text' : 'Expand full text'}
                                                        </span>
                                                    </div>
                                                )}
                                            </div>

                                            {/* Truncation & Copy Controls */}
                                            <div className="flex flex-wrap items-center justify-between gap-2 pt-1">
                                                <div className="text-xs text-slate-500 dark:text-slate-400 font-mono">
                                                    {isTruncated && !showAllWords
                                                        ? `${totalWords} words · ${formatByteSize(activeByteCount)}`
                                                        : `${activeWordCount} words · ${formatByteSize(activeByteCount)}`}
                                                </div>

                                                <div className="flex items-center gap-2">
                                                    {(isTruncated || activePromptText.trim().endsWith('...') || activePromptText.trim().endsWith('…') || showAllWords) && (
                                                        <button
                                                            type="button"
                                                            onClick={() => setShowAllWords(!showAllWords)}
                                                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-[5px] text-xs font-semibold bg-blue-50 dark:bg-[#0c2438] text-blue-600 dark:text-cyan-300 border border-blue-200 dark:border-[#15334d] hover:bg-blue-100 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                            title={showAllWords ? 'Collapse preview to 120 words' : 'Expand full prompt'}
                                                        >
                                                            {showAllWords ? (
                                                                <>
                                                                    <ChevronUp className="w-3.5 h-3.5" />
                                                                    <span>Show Less</span>
                                                                </>
                                                            ) : (
                                                                <>
                                                                    <ChevronDown className="w-3.5 h-3.5" />
                                                                    <span>Show All ({totalWords || activeWordCount} words)</span>
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

                                    {/* Tab 2: Raw Mode (Monospace Clean Text with Explicit <br /> Line Gaps) */}
                                    {viewMode === 'raw' && (
                                        <div className="space-y-3">
                                            <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 font-mono text-xs text-slate-800 dark:text-slate-200 leading-relaxed whitespace-pre-wrap break-words max-h-[460px] overflow-y-auto select-text shadow-inner">
                                                {activePromptText ? (
                                                    formatPromptForMarkdown(activePromptText).split('\n').map((line, idx, arr) => (
                                                        <span key={`raw-line-${idx}`} className="block">
                                                            {line || <br className="my-1.5 block select-none" />}
                                                            {idx < arr.length - 1 && <br className="my-1.5 block select-none" />}
                                                        </span>
                                                    ))
                                                ) : (
                                                    'No prompt content recorded.'
                                                )}
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
                            )}

                            {/* Tab 2: AI Results & Outputs View */}
                            {previewTab === 'results' && (
                                <div className="space-y-3">
                                    {/* In-Flight Live Telemetry Status Banner if running */}
                                    {selectedConversation.is_running && (
                                        <div className="rounded-xl border border-emerald-500/40 bg-emerald-500/10 p-3.5 text-emerald-800 dark:text-emerald-200 shadow-xs space-y-2">
                                            <div className="flex items-center justify-between">
                                                <div className="flex items-center gap-2">
                                                    <span className="relative flex h-2.5 w-2.5">
                                                        <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                                        <span className="relative inline-flex rounded-full h-2.5 w-2.5 bg-[#1af18d]"></span>
                                                    </span>
                                                    <span className="font-bold text-xs uppercase tracking-wider font-mono">
                                                        Live Execution in Progress
                                                    </span>
                                                </div>
                                                <div className="flex items-center gap-2 text-[10px] font-mono">
                                                    {instancePid && (
                                                        <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                                                            PID: {instancePid}
                                                        </span>
                                                    )}
                                                    <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                                                        Elapsed: {formatDuration(elapsedSeconds)}
                                                    </span>
                                                    <span className="px-1.5 py-0.5 rounded bg-black/10 dark:bg-white/10">
                                                        {selectedConversation.step_count || 1} steps
                                                    </span>
                                                </div>
                                            </div>
                                            {selectedConversation.latest_step_summary && (
                                                <div className="flex items-center gap-2 text-xs bg-black/5 dark:bg-black/20 p-2 rounded-lg font-mono">
                                                    <Terminal className="w-3.5 h-3.5 text-cyan-400 shrink-0" />
                                                    <span className="truncate">{selectedConversation.latest_step_summary}</span>
                                                </div>
                                            )}
                                            {selectedConversation.tool_calls_summary && (
                                                <div className="flex items-center gap-2 text-xs text-amber-700 dark:text-amber-300">
                                                    <Wrench className="w-3.5 h-3.5 shrink-0" />
                                                    <span className="truncate">{selectedConversation.tool_calls_summary}</span>
                                                </div>
                                            )}
                                        </div>
                                    )}

                                    {/* Display captured AI Response or Execution Results */}
                                    {(selectedConversation.execution_results || selectedConversation.latest_response) ? (
                                        <div className="space-y-3">
                                            <div className="flex items-center justify-between">
                                                <span className="text-xs font-bold text-slate-700 dark:text-slate-300 uppercase tracking-wider">
                                                    Latest AI Response & Execution Output
                                                </span>
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        const parts = [];
                                                        if (selectedConversation.latest_response) {
                                                            parts.push(selectedConversation.latest_response);
                                                        }
                                                        if (selectedConversation.execution_results) {
                                                            parts.push(`\n\n--- Execution Details ---\n${selectedConversation.execution_results}`);
                                                        }
                                                        const resText = parts.join('\n') || '';
                                                        navigator.clipboard.writeText(resText);
                                                        setIsCopiedResults(true);
                                                        setTimeout(() => setIsCopiedResults(false), 2000);
                                                    }}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-full text-xs font-semibold bg-slate-100 dark:bg-[#0c2438] text-slate-700 dark:text-slate-200 border border-slate-200 dark:border-[#15334d] hover:bg-slate-200 dark:hover:bg-[#15334d] transition-colors cursor-pointer"
                                                    title="Copy AI execution results to clipboard"
                                                >
                                                    {isCopiedResults ? (
                                                        <>
                                                            <Check className="w-3.5 h-3.5 text-emerald-500" />
                                                            <span>Copied AI Results!</span>
                                                        </>
                                                    ) : (
                                                        <>
                                                            <Copy className="w-3.5 h-3.5" />
                                                            <span>Copy AI Results</span>
                                                        </>
                                                    )}
                                                </button>
                                            </div>

                                            {/* 1. Assistant Text Response */}
                                            {selectedConversation.latest_response && (
                                                <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-50 dark:bg-[#071a27] p-5 text-xs text-slate-800 dark:text-slate-200 leading-relaxed max-h-[460px] overflow-y-auto shadow-inner">
                                                    <div className="flex items-center gap-1.5 mb-2 pb-1.5 border-b border-slate-200/60 dark:border-[#15334d]/60 text-emerald-600 dark:text-emerald-400 font-bold text-xs">
                                                        <Sparkles className="w-3.5 h-3.5" />
                                                        <span>AI Assistant Response</span>
                                                    </div>
                                                    <RichMarkdownRenderer
                                                        content={selectedConversation.latest_response}
                                                        showAllWords={true}
                                                        onToggleExpand={() => {}}
                                                        isTruncated={false}
                                                    />
                                                </div>
                                            )}

                                            {/* 2. Tool & Terminal Execution Details */}
                                            {selectedConversation.execution_results && (
                                                <div className="rounded-xl border border-slate-200 dark:border-[#15334d] bg-slate-900 text-slate-100 p-4 font-mono text-[11px] leading-relaxed max-h-[300px] overflow-y-auto select-text shadow-inner">
                                                    <div className="flex items-center gap-1.5 mb-2 pb-1 border-b border-slate-800 text-cyan-400 font-semibold text-[10px]">
                                                        <Terminal className="w-3 h-3" />
                                                        <span>Execution & Tool Details</span>
                                                    </div>
                                                    <pre className="whitespace-pre-wrap break-words">{selectedConversation.execution_results}</pre>
                                                </div>
                                            )}
                                        </div>
                                    ) : selectedConversation.is_running ? (
                                        <div className="rounded-xl border border-emerald-500/30 bg-emerald-500/5 p-8 flex flex-col items-center justify-center text-center space-y-3">
                                            <div className="relative flex h-10 w-10 items-center justify-center">
                                                <span className="animate-ping absolute inline-flex h-full w-full rounded-full bg-emerald-400 opacity-75"></span>
                                                <div className="relative flex h-8 w-8 items-center justify-center rounded-full bg-emerald-500 text-white shadow-md">
                                                    <Sparkles className="w-4 h-4 animate-spin" />
                                                </div>
                                            </div>
                                            <div className="space-y-1">
                                                <div className="text-xs font-bold text-slate-800 dark:text-slate-100 uppercase tracking-wider font-mono">
                                                    In-Flight Antigravity Execution Active
                                                </div>
                                                <p className="text-[11px] text-slate-500 dark:text-slate-400 max-w-md">
                                                    {selectedConversation.latest_step_summary || 'Agent is actively thinking, evaluating code, or executing tools in the main Antigravity window.'}
                                                </p>
                                            </div>
                                            <div className="flex items-center gap-2 pt-1 font-mono text-[10px] text-emerald-700 dark:text-[#1af18d] bg-emerald-500/10 px-3 py-1 rounded-full border border-emerald-500/20">
                                                <span>Elapsed: {formatDuration(elapsedSeconds)}</span>
                                                <span>·</span>
                                                <span>Step {selectedConversation.step_count || 1}</span>
                                            </div>
                                            <button
                                                type="button"
                                                onClick={handleFocusIde}
                                                disabled={isFocusing}
                                                className="flex items-center gap-1.5 px-3.5 py-1.5 rounded-full text-xs font-semibold bg-emerald-600 hover:bg-emerald-700 text-white transition-all duration-150 cursor-pointer shadow-xs mt-2"
                                                title="Open and focus the Antigravity main window"
                                            >
                                                <ExternalLink className="w-3.5 h-3.5" />
                                                <span>Open in Antigravity Window</span>
                                            </button>
                                        </div>
                                    ) : (
                                        <div className="rounded-xl border border-dashed border-slate-200 dark:border-[#15334d] p-8 flex flex-col items-center justify-center text-center space-y-2 text-slate-400">
                                            <Terminal className="w-8 h-8 text-slate-400/80 mb-1" />
                                            <span className="font-semibold text-slate-600 dark:text-slate-300 text-xs">
                                                No execution results recorded yet
                                            </span>
                                            <p className="text-[11px] max-w-sm text-slate-500">
                                                This prompt has not produced output in the transcript log, or is currently waiting in queue.
                                            </p>
                                            <div className="flex items-center gap-2 pt-2">
                                                <button
                                                    type="button"
                                                    onClick={handleFocusIde}
                                                    disabled={isFocusing}
                                                    className="flex items-center gap-1 px-3 py-1.5 rounded-full text-xs font-semibold bg-blue-600 text-white hover:bg-blue-700 transition-colors cursor-pointer"
                                                >
                                                    <ExternalLink className="w-3.5 h-3.5" />
                                                    <span>Open IDE Window</span>
                                                </button>
                                            </div>
                                        </div>
                                    )}
                                </div>
                            )}
                        </div>
                        ) : (
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
                                            onClick={handleResendPrompt}
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
                        )}
                    </div>
                </div>
            </div>

            {/* Full-Screen Prompt Inspector Modal */}
            {inspectorPrompt && (
                <div className="fixed inset-0 z-[320] flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
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

                            {/* Inspector Actions Capsule */}
                            <div className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#0c2438]/90 backdrop-blur-md border border-slate-200 dark:border-[#15334d] p-0.5 divide-x divide-slate-200 dark:divide-[#15334d] shadow-2xs shrink-0">
                                <button
                                    type="button"
                                    onClick={() => {
                                        const cleanText = stripImagesFromPrompt(inspectorPrompt.text);
                                        navigator.clipboard.writeText(cleanText);
                                        setIsCopied(true);
                                        setTimeout(() => setIsCopied(false), 2000);
                                    }}
                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-l-full text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                    title="Copy clean prompt text"
                                >
                                    {isCopied ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-slate-500 dark:text-slate-400" />}
                                    <span>{isCopied ? 'Copied!' : 'Copy Text'}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={handleCopyWithImagesRich}
                                    className="flex items-center gap-1.5 px-3 py-1.5 text-slate-700 dark:text-slate-200 hover:bg-slate-200/80 dark:hover:bg-[#15334d] text-xs font-semibold transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                    title="Copy rich text with images"
                                >
                                    {isCopiedRaw ? <Check className="w-3.5 h-3.5 text-emerald-500" /> : <Copy className="w-3.5 h-3.5 text-indigo-500" />}
                                    <span>{isCopiedRaw ? 'Copied + Imgs!' : '+ Imgs'}</span>
                                </button>
                                <button
                                    type="button"
                                    onClick={() => setInspectorPrompt(null)}
                                    className="flex items-center px-2.5 py-1.5 text-slate-400 hover:text-rose-600 dark:hover:text-rose-400 hover:bg-rose-500/10 dark:hover:bg-rose-500/20 rounded-r-full transition-all duration-150 ease-out active:scale-[0.98] cursor-pointer"
                                    title="Close Inspector"
                                >
                                    <X className="h-4 w-4" />
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
                                    <RichMarkdownRenderer
                                        content={inspectorPrompt.text}
                                        showAllWords={showAllWords}
                                        onToggleExpand={() => setShowAllWords(!showAllWords)}
                                        isTruncated={inspectorPrompt.text.trim().endsWith('...') || inspectorPrompt.text.trim().endsWith('…')}
                                    />
                                    {(inspectorPrompt.text.trim().endsWith('...') || inspectorPrompt.text.trim().endsWith('…') || showAllWords) && (
                                        <div className="mt-3 pt-2 border-t border-slate-200/50 dark:border-[#15334d]/50 flex items-center">
                                            <span
                                                onClick={(e) => {
                                                    e.stopPropagation();
                                                    setShowAllWords(!showAllWords);
                                                }}
                                                title="Click to expand or collapse full prompt text"
                                                className="cursor-pointer text-xs font-semibold text-cyan-600 dark:text-cyan-400 hover:underline flex items-center gap-1 select-none"
                                            >
                                                {showAllWords ? 'Collapse full text' : 'Expand full text'}
                                            </span>
                                        </div>
                                    )}
                                </div>
                            </div>
                        </div>
                    </div>
                </div>
            )}

            {/* Conversation & Project Details Modal */}
            {isDetailsModalOpen && (
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
                                onClick={() => setIsDetailsModalOpen(false)}
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
                                                        navigator.clipboard.writeText(selectedConversation.conversation_id);
                                                        setCopiedField('conv_id');
                                                        setTimeout(() => setCopiedField(null), 2000);
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
                                                            navigator.clipboard.writeText(selectedProject.repo_path);
                                                            setCopiedField('repo_path');
                                                            setTimeout(() => setCopiedField(null), 2000);
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
                                onClick={() => setIsDetailsModalOpen(false)}
                                className="px-4 py-1.5 rounded-[5px] bg-slate-200 dark:bg-[#15334d] text-slate-700 dark:text-slate-200 hover:bg-slate-300 dark:hover:bg-[#1c4466] text-xs font-semibold transition-colors cursor-pointer"
                            >
                                Close
                            </button>
                        </div>
                    </div>
                </div>
            )}
        </div>
    );

    return typeof document !== 'undefined' ? createPortal(modalContent, document.body) : modalContent;
}
