import { useState, useMemo } from 'react';
import {
    Search,
    Copy,
    Check,
    MessageSquare,
    ShieldCheck,
    ChevronDown,
    ChevronUp,
    Sparkles,
    History,
} from 'lucide-react';
import { copyToClipboard } from '../../utils/clipboard';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

export interface TelegramCommandItem {
    command: string;
    aliases?: string[];
    description: string;
    usage: string;
    example: string;
    category: 'seeking' | 'prompting' | 'quota' | 'export' | 'cluster';
    features?: string[];
}

const TELEGRAM_COMMAND_REFERENCE: TelegramCommandItem[] = [
    {
        command: '/seek',
        aliases: ['/conversations', '/convs', '/turns'],
        description:
            'Search running and historical conversations across all instances by title, sequence, prompt text, or CID8. Renders ≥200-word expandable blockquote previews with inline rerun, verification, and copy buttons.',
        usage: '/seek [query]',
        example: '/seek auth token refresh',
        category: 'seeking',
        features: [
            'Filters by sequence ([AGM:P001 | GM:#1]), CID8 prefix, or query text in prompt body',
            'Enforces FIFO ordering invariant (ORDER BY created_at ASC, id ASC)',
            'Expandable quotes with ≥200-word preview (<blockquote expandable><pre><code class="language-markdown">...</code></pre></blockquote>)',
            'Inline 🔁 Rerun Turn button (send:t:<cid8>:<turn>)',
            'Inline 🛡️ Verify Turn button (send:v:<cid8>:<turn>) with CDP screenshot capture',
            'Inline 📋 Copy Turn button with Telegram 7.8+ copy_text payload',
        ],
    },
    {
        command: '/conversations',
        aliases: ['/seek', '/convs'],
        description:
            'Canonical alias for /seek to inspect active and archived conversation turns across instances.',
        usage: '/conversations [query]',
        example: '/conversations build pipeline',
        category: 'seeking',
        features: [
            'Identical functionality and formatting as /seek',
            'Supports short CID8 prefixes (e.g., /conversations 3f8a12bc)',
            'Strictly bounded 60-byte callback payloads (RCA 15 invariant)',
        ],
    },
    {
        command: '/turns',
        aliases: ['/seek', '/convs'],
        description:
            'Search conversation turns directly with full turn indices and step status badges.',
        usage: '/turns [query]',
        example: '/turns #1',
        category: 'seeking',
        features: [
            'Displays turn status (Turn 1/5) with green/amber/zinc status badges',
            'Instant rerun and verification triggers for targeted turn index',
        ],
    },
    {
        command: '/prompt',
        aliases: ['/p'],
        description: 'Inject prompt directly into a local or remote target instance via CDP.',
        usage: '/prompt [machine:]<instance> <text>',
        example: '/prompt default Run unit tests and report coverage',
        category: 'prompting',
        features: [
            'Supports colon notation (W3:default)',
            'Automatic active instance targeting if omitted',
            'Captures CDP screenshot proof upon submission',
        ],
    },
    {
        command: '/target',
        aliases: ['/lock'],
        description: 'Lock default target machine and instance for subsequent chat messages.',
        usage: '/target <machine> <instance>',
        example: '/target local default',
        category: 'prompting',
        features: ['Pins destination context for all future un-prefixed messages in Telegram session'],
    },
    {
        command: '/send',
        aliases: [],
        description: 'Interactive machine -> instance -> conversation picker wizard with inline keyboards.',
        usage: '/send',
        example: '/send',
        category: 'prompting',
        features: ['Step-by-step navigation through instances and conversations without typing IDs'],
    },
    {
        command: '/balance',
        aliases: ['/credits', '/quota'],
        description: 'Multi-machine and local per-instance 4h rolling and weekly quota credit report.',
        usage: '/balance [--csv]',
        example: '/balance',
        category: 'quota',
        features: [
            'Calculates 5h rolling vs 7-day weekly quota exhaustion percentages',
            'Supports --csv flag to receive live account table as a downloadable document',
        ],
    },
    {
        command: '/export',
        aliases: [],
        description: 'Export system datasets (accounts, instances, balance, prompts) in CSV or JSON format.',
        usage: '/export <dataset> <csv|json>',
        example: '/export prompts json',
        category: 'export',
        features: [
            'Datasets: accounts, instances, balance, nodes, prompts',
            'Delivers downloadable file attachment directly in Telegram chat',
        ],
    },
    {
        command: '/nodes',
        aliases: ['/ssh nodes'],
        description: 'List configured SSH cluster fleet nodes, health status, and GitMap cluster topology.',
        usage: '/nodes',
        example: '/nodes',
        category: 'cluster',
        features: ['Inspect multi-node status, GitMap clusters, and remote execution nodes'],
    },
];

export default function TelegramSettings() {
    const [searchQuery, setSearchQuery] = useState('');
    const [selectedCategory, setSelectedCategory] = useState<string>('all');
    const [copiedIndex, setCopiedIndex] = useState<number | null>(null);
    const [expandedCommand, setExpandedCommand] = useState<string | null>('/seek');

    const handleCopy = async (text: string, index: number) => {
        try {
            const ok = await copyToClipboard(text);
            if (ok) {
                setCopiedIndex(index);
                showToast(`Copied "${text}" to clipboard`, 'success', 2000);
                setTimeout(() => setCopiedIndex(null), 2000);
            }
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'TelegramSettings.handleCopy',
                triggerAction: 'copy_command_syntax',
            });
        }
    };

    const filteredCommands = useMemo(() => {
        return TELEGRAM_COMMAND_REFERENCE.filter((cmd) => {
            const matchesCat = selectedCategory === 'all' || cmd.category === selectedCategory;
            const q = searchQuery.toLowerCase().trim();
            if (!q) return matchesCat;
            const matchesQuery =
                cmd.command.toLowerCase().includes(q) ||
                (cmd.aliases && cmd.aliases.some((a) => a.toLowerCase().includes(q))) ||
                cmd.description.toLowerCase().includes(q) ||
                cmd.usage.toLowerCase().includes(q) ||
                (cmd.features && cmd.features.some((f) => f.toLowerCase().includes(q)));
            return matchesCat && matchesQuery;
        });
    }, [searchQuery, selectedCategory]);

    return (
        <div className="space-y-6 max-w-5xl mx-auto p-4 sm:p-6 text-gray-800 dark:text-gray-200">
            {/* Header & Overview Card */}
            <div className="bg-white dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-2xl p-6 shadow-sm">
                <div className="flex flex-col md:flex-row md:items-center justify-between gap-4">
                    <div className="flex items-center gap-3">
                        <div className="p-3 bg-sky-50 dark:bg-sky-950/40 text-sky-600 dark:text-sky-400 rounded-xl border border-sky-100 dark:border-sky-800/60">
                            <MessageSquare className="w-6 h-6" />
                        </div>
                        <div>
                            <h2 className="text-xl font-bold flex items-center gap-2">
                                Telegram Remote Command Reference
                                <span className="text-xs px-2.5 py-0.5 rounded-full bg-sky-100 dark:bg-sky-900/60 text-sky-700 dark:text-sky-300 font-semibold">
                                    Telegram API 7.8+
                                </span>
                            </h2>
                            <p className="text-sm text-gray-500 dark:text-gray-400 mt-0.5">
                                Unified command interface for conversation seeking, prompt injection, verification rerun, and quota management.
                            </p>
                        </div>
                    </div>
                </div>

                {/* Architecture Invariants Highlights */}
                <div className="grid grid-cols-1 md:grid-cols-3 gap-3 mt-6 pt-6 border-t border-gray-100 dark:border-base-300/80">
                    <div className="bg-gray-50/70 dark:bg-base-300/40 rounded-xl p-3.5 border border-gray-200/60 dark:border-base-300">
                        <div className="flex items-center gap-2 text-xs font-semibold text-sky-600 dark:text-sky-400 mb-1">
                            <History className="w-4 h-4" />
                            <span>≥200-Word Expandable Previews</span>
                        </div>
                        <p className="text-xs text-gray-600 dark:text-gray-400 leading-relaxed">
                            Telegram HTML blockquotes with word-count floor preserving markdown code blocks while saving chat vertical space.
                        </p>
                    </div>

                    <div className="bg-gray-50/70 dark:bg-base-300/40 rounded-xl p-3.5 border border-gray-200/60 dark:border-base-300">
                        <div className="flex items-center gap-2 text-xs font-semibold text-purple-600 dark:text-purple-400 mb-1">
                            <ShieldCheck className="w-4 h-4" />
                            <span>Strict 60-Byte Callback Cap</span>
                        </div>
                        <p className="text-xs text-gray-600 dark:text-gray-400 leading-relaxed">
                            Enforces RCA 15 invariant. Callback payloads (<code className="text-purple-700 dark:text-purple-300">send:t:</code>, <code className="text-purple-700 dark:text-purple-300">send:v:</code>) strictly bound to ~17 bytes.
                        </p>
                    </div>

                    <div className="bg-gray-50/70 dark:bg-base-300/40 rounded-xl p-3.5 border border-gray-200/60 dark:border-base-300">
                        <div className="flex items-center gap-2 text-xs font-semibold text-emerald-600 dark:text-emerald-400 mb-1">
                            <Sparkles className="w-4 h-4" />
                            <span>CDP Rerun & Proof Capture</span>
                        </div>
                        <p className="text-xs text-gray-600 dark:text-gray-400 leading-relaxed">
                            Dispatches verified prompts directly to Antigravity webview via Chrome DevTools Protocol and returns screenshot proof.
                        </p>
                    </div>
                </div>
            </div>

            {/* Filter and Search Bar */}
            <div className="flex flex-col sm:flex-row gap-3 items-center justify-between">
                <div className="relative w-full sm:w-80">
                    <Search className="w-4 h-4 absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" />
                    <input
                        type="text"
                        value={searchQuery}
                        onChange={(e) => setSearchQuery(e.target.value)}
                        placeholder="Search commands, aliases, syntax..."
                        className="w-full pl-9 pr-4 py-2 text-sm rounded-xl border border-gray-200 dark:border-base-300 bg-white dark:bg-base-200 focus:outline-none focus:ring-2 focus:ring-sky-500/20 focus:border-sky-500"
                    />
                </div>

                {/* Category Pills */}
                <div className="flex items-center gap-1.5 overflow-x-auto w-full sm:w-auto pb-1 sm:pb-0">
                    {[
                        { id: 'all', label: 'All' },
                        { id: 'seeking', label: 'Seeking & Rerun' },
                        { id: 'prompting', label: 'Prompting' },
                        { id: 'quota', label: 'Quota & Balance' },
                        { id: 'export', label: 'Export' },
                        { id: 'cluster', label: 'Cluster' },
                    ].map((cat) => (
                        <button
                            key={cat.id}
                            onClick={() => setSelectedCategory(cat.id)}
                            className={`px-3 py-1.5 rounded-lg text-xs font-medium transition-colors cursor-pointer whitespace-nowrap ${
                                selectedCategory === cat.id
                                    ? 'bg-sky-500 text-white shadow-xs'
                                    : 'bg-gray-100 dark:bg-base-200 text-gray-600 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200'
                            }`}
                        >
                            {cat.label}
                        </button>
                    ))}
                </div>
            </div>

            {/* Command Reference List */}
            <div className="space-y-3">
                {filteredCommands.length === 0 ? (
                    <div className="bg-white dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-xl p-8 text-center text-gray-500 dark:text-gray-400">
                        No Telegram commands match your search query.
                    </div>
                ) : (
                    filteredCommands.map((cmd, idx) => {
                        const isExpanded = expandedCommand === cmd.command;
                        return (
                            <div
                                key={cmd.command}
                                className="bg-white dark:bg-base-200 border border-gray-200 dark:border-base-300 rounded-xl overflow-hidden shadow-xs transition-all"
                            >
                                <div
                                    className="p-4 flex items-center justify-between cursor-pointer hover:bg-gray-50/50 dark:hover:bg-base-300/30 transition-colors"
                                    onClick={() => setExpandedCommand(isExpanded ? null : cmd.command)}
                                >
                                    <div className="flex items-center gap-3">
                                        <code className="text-base font-bold text-sky-600 dark:text-sky-400 bg-sky-50 dark:bg-sky-950/40 px-2.5 py-1 rounded-lg border border-sky-100 dark:border-sky-800/50">
                                            {cmd.command}
                                        </code>
                                        {cmd.aliases && cmd.aliases.length > 0 && (
                                            <div className="flex items-center gap-1">
                                                {cmd.aliases.map((alias) => (
                                                    <span
                                                        key={alias}
                                                        className="text-xs px-2 py-0.5 rounded-md bg-gray-100 dark:bg-base-300 text-gray-600 dark:text-gray-400 font-mono"
                                                    >
                                                        {alias}
                                                    </span>
                                                ))}
                                            </div>
                                        )}
                                        <span className="hidden sm:inline text-xs text-gray-400 dark:text-gray-500">•</span>
                                        <span className="text-xs text-gray-600 dark:text-gray-400 line-clamp-1 max-w-md">
                                            {cmd.description}
                                        </span>
                                    </div>

                                    <div className="flex items-center gap-2">
                                        <button
                                            type="button"
                                            onClick={(e) => {
                                                e.stopPropagation();
                                                handleCopy(cmd.example, idx);
                                            }}
                                            className="p-1.5 text-gray-400 hover:text-sky-600 dark:hover:text-sky-400 rounded-lg hover:bg-gray-100 dark:hover:bg-base-300 transition-colors"
                                            title="Copy example command"
                                        >
                                            {copiedIndex === idx ? (
                                                <Check className="w-4 h-4 text-emerald-500" />
                                            ) : (
                                                <Copy className="w-4 h-4" />
                                            )}
                                        </button>
                                        {isExpanded ? (
                                            <ChevronUp className="w-4 h-4 text-gray-400" />
                                        ) : (
                                            <ChevronDown className="w-4 h-4 text-gray-400" />
                                        )}
                                    </div>
                                </div>

                                {isExpanded && (
                                    <div className="p-4 pt-0 border-t border-gray-100 dark:border-base-300/60 bg-gray-50/40 dark:bg-base-300/20 space-y-3">
                                        <p className="text-sm text-gray-700 dark:text-gray-300 leading-relaxed mt-3">
                                            {cmd.description}
                                        </p>

                                        <div className="grid grid-cols-1 md:grid-cols-2 gap-3 text-xs">
                                            <div className="bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-300 rounded-lg p-2.5">
                                                <span className="text-gray-400 font-medium block mb-1">Syntax Usage:</span>
                                                <code className="text-purple-600 dark:text-purple-400 font-mono font-semibold">
                                                    {cmd.usage}
                                                </code>
                                            </div>

                                            <div className="bg-white dark:bg-base-200 border border-gray-200/80 dark:border-base-300 rounded-lg p-2.5 flex items-center justify-between">
                                                <div>
                                                    <span className="text-gray-400 font-medium block mb-1">Ready-to-Use Example:</span>
                                                    <code className="text-emerald-600 dark:text-emerald-400 font-mono font-semibold">
                                                        {cmd.example}
                                                    </code>
                                                </div>
                                                <button
                                                    onClick={() => handleCopy(cmd.example, idx)}
                                                    className="px-2 py-1 text-xs rounded bg-gray-100 dark:bg-base-300 hover:bg-gray-200 dark:hover:bg-base-100 text-gray-700 dark:text-gray-300 transition-colors flex items-center gap-1 cursor-pointer"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                    <span>Copy</span>
                                                </button>
                                            </div>
                                        </div>

                                        {cmd.features && cmd.features.length > 0 && (
                                            <div className="mt-2">
                                                <span className="text-xs font-semibold text-gray-500 dark:text-gray-400 uppercase tracking-wider block mb-1.5">
                                                    Protocol Specifications & Invariants:
                                                </span>
                                                <ul className="space-y-1">
                                                    {cmd.features.map((feat, fIdx) => (
                                                        <li
                                                            key={fIdx}
                                                            className="text-xs text-gray-600 dark:text-gray-400 flex items-start gap-2"
                                                        >
                                                            <span className="text-sky-500 mt-0.5">•</span>
                                                            <span>{feat}</span>
                                                        </li>
                                                    ))}
                                                </ul>
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
    );
}
