import { useState, useEffect, type ReactNode } from 'react';
import {
    User,
    Bot,
    Terminal,
    Wrench,
    ChevronDown,
    ChevronRight,
    Repeat,
    ImagePlus,
    LocateFixed,
    RefreshCw,
    Clock,
    AlertTriangle,
    type LucideIcon,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { useErrorStore } from '../../../stores/error-store';
import { classifyPromptTier } from './tierClassification';
import { PromptOriginTier } from './types';
import type { UpNextDisplayItem } from './useRunningAndUpNext';

/**
 * NowRunningUpNext — docked summary strip for the prompt tree view (spec 154 §1).
 *
 * Renders EXACTLY what the data hook returns (single source of truth): no
 * re-sorting, no TTL logic, no ghost filtration here. Running prompt at
 * position 0, then the first four queued prompts FIFO, with a `+N more`
 * accordion for overflow. Self-hides when empty.
 */

export interface NowRunningUpNextProps {
    items: UpNextDisplayItem[];
    isLoading: boolean;
    lastSyncAt: number | null;
    /** True when the last poll failed; last-good items render with a stale marker. */
    hasError: boolean;
    /** Full queued count from the backend ("Up next (4 of 11)"). */
    upNextTotal: number;
    onSelectPrompt: (promptId: string) => void;
    onRerun: (promptId: string) => void;
    onResendWithImage: (promptId: string) => void;
    onRefresh: () => void;
}

interface TierBadge {
    label: string;
    badgeStyle: string;
    Icon: LucideIcon;
    iconClass: string;
}

/** EXACT 4-tier taxonomy (AGENTS.md house invariant — do not re-derive). */
function tierBadgeFor(item: UpNextDisplayItem): TierBadge {
    const tier = (item.origin_tier || '').toUpperCase();
    switch (tier) {
        case PromptOriginTier.SubagentInstruction:
        case 'SUBAGENT_INSTRUCTION':
            return {
                label: 'Subagent',
                badgeStyle: 'text-purple-900 dark:text-amber-200 bg-purple-500/20 border-purple-400/40',
                Icon: Bot,
                iconClass: 'text-purple-500',
            };
        case PromptOriginTier.SystemMessage:
        case 'SYSTEM_MESSAGE':
            return {
                label: 'System',
                badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
                Icon: Terminal,
                iconClass: 'text-slate-500',
            };
        case PromptOriginTier.ToolOutput:
        case 'TOOL_OUTPUT':
            return {
                label: 'Tool',
                badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
                Icon: Wrench,
                iconClass: 'text-amber-500',
            };
        case PromptOriginTier.UserPrompt:
        case 'USER_PROMPT':
            return {
                label: 'User',
                badgeStyle: 'bg-sky-500/15 text-sky-700 dark:text-cyan-300 border-sky-400/30',
                Icon: User,
                iconClass: 'text-sky-500',
            };
        default: {
            // Fallback: classify from title text (same taxonomy, never invented).
            const classified = classifyPromptTier(item.title || '', item.title || '');
            const map: Record<string, TierBadge> = {
                [PromptOriginTier.SubagentInstruction]: {
                    label: 'Subagent',
                    badgeStyle: 'text-purple-900 dark:text-amber-200 bg-purple-500/20 border-purple-400/40',
                    Icon: Bot,
                    iconClass: 'text-purple-500',
                },
                [PromptOriginTier.SystemMessage]: {
                    label: 'System',
                    badgeStyle: 'bg-zinc-500/15 text-zinc-700 dark:text-zinc-300 border-zinc-400/30',
                    Icon: Terminal,
                    iconClass: 'text-slate-500',
                },
                [PromptOriginTier.ToolOutput]: {
                    label: 'Tool',
                    badgeStyle: 'bg-amber-500/15 text-amber-700 dark:text-amber-300 border-amber-400/30',
                    Icon: Wrench,
                    iconClass: 'text-amber-500',
                },
            };
            return (
                map[classified.tier as string] ?? {
                    label: 'User',
                    badgeStyle: 'bg-sky-500/15 text-sky-700 dark:text-cyan-300 border-sky-400/30',
                    Icon: User,
                    iconClass: 'text-sky-500',
                }
            );
        }
    }
}

function formatElapsed(ms: number): string {
    const secs = Math.max(0, Math.floor(ms / 1000));
    const m = Math.floor(secs / 60);
    const s = secs % 60;
    if (m === 0) return `${s}s`;
    const h = Math.floor(m / 60);
    if (h === 0) return `${m}m ${s < 10 ? '0' : ''}${s}s`;
    return `${h}h ${m % 60}m`;
}

function formatSyncAge(lastSyncAt: number | null): string {
    if (!lastSyncAt) return 'never synced';
    const ageS = Math.max(0, Math.floor((Date.now() - lastSyncAt) / 1000));
    if (ageS < 5) return 'just now';
    if (ageS < 60) return `${ageS}s ago`;
    return `${Math.floor(ageS / 60)}m ago`;
}

/** Render a title, converting raw `<truncated …>` markers into the dark-glass omission banner. */
function renderTitle(title: string | null): ReactNode {
    const text = (title || '').trim() || '(untitled prompt)';
    const truncMatch = text.match(/<truncated\s+([^>]+)>/i);
    if (!truncMatch) {
        return <span className="truncate">{text}</span>;
    }
    const before = text.slice(0, truncMatch.index).trim();
    const omitted = truncMatch[1].trim();
    return (
        <span className="min-w-0">
            {before && <span className="truncate">{before} </span>}
            <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md bg-slate-900/80 dark:bg-black/60 border border-slate-600/40 text-slate-200 text-[9px] font-mono">
                <span>⚡</span>
                <span>[Omitted {omitted} — expand in tree]</span>
            </span>
        </span>
    );
}

function RowActions({
    item,
    onSelectPrompt,
    onRerun,
    onResendWithImage,
}: {
    item: UpNextDisplayItem;
    onSelectPrompt: (promptId: string) => void;
    onRerun: (promptId: string) => void;
    onResendWithImage: (promptId: string) => void;
}) {
    const btn =
        'flex items-center justify-center p-1.5 text-slate-500 dark:text-slate-300 hover:bg-slate-200/70 dark:hover:bg-[#40285e]/70 transition-colors cursor-pointer';
    return (
        <div
            className="flex items-center rounded-full bg-slate-100/90 dark:bg-[#120a1f]/90 backdrop-blur-md border border-slate-200/80 dark:border-purple-500/30 divide-x divide-slate-200/70 dark:divide-purple-500/20 shadow-2xs shrink-0"
            role="group"
            aria-label={`Actions for prompt ${item.id}`}
        >
            <button
                type="button"
                onClick={() => onSelectPrompt(item.id)}
                className={cn(btn, 'rounded-l-full pl-2')}
                title="Select in tree"
                aria-label={`Select prompt ${item.id} in tree`}
            >
                <LocateFixed className="w-3.5 h-3.5" />
            </button>
            <button
                type="button"
                onClick={() => onRerun(item.id)}
                className={btn}
                title="Rerun this prompt"
                aria-label={`Rerun prompt ${item.id}`}
            >
                <Repeat className="w-3.5 h-3.5 text-sky-500" />
            </button>
            <button
                type="button"
                onClick={() => onResendWithImage(item.id)}
                className={cn(btn, 'rounded-r-full pr-2')}
                title="Resend with image"
                aria-label={`Resend prompt ${item.id} with image`}
            >
                <ImagePlus className="w-3.5 h-3.5 text-emerald-500" />
            </button>
        </div>
    );
}

function PanelRow({
    item,
    onSelectPrompt,
    onRerun,
    onResendWithImage,
}: {
    item: UpNextDisplayItem;
    onSelectPrompt: (promptId: string) => void;
    onRerun: (promptId: string) => void;
    onResendWithImage: (promptId: string) => void;
}) {
    const badge = tierBadgeFor(item);
    const BadgeIcon = badge.Icon;
    const isRunning = item.isRunningRow;
    // Backend timestamps are Unix seconds; normalize to ms for elapsed math.
    const elapsedBaseRaw = item.started_at ?? item.created_at ?? null;
    const elapsedBase =
        typeof elapsedBaseRaw === 'number' && Number.isFinite(elapsedBaseRaw) && elapsedBaseRaw > 0
            ? elapsedBaseRaw < 1e11
                ? elapsedBaseRaw * 1000
                : elapsedBaseRaw
            : null;
    const [nowTick, setNowTick] = useState(() => Date.now());

    // Elapsed ticker for the running row only.
    useEffect(() => {
        if (!isRunning || !elapsedBase) return;
        const t = setInterval(() => setNowTick(Date.now()), 1000);
        return () => clearInterval(t);
    }, [isRunning, elapsedBase]);

    return (
        <div
            className={cn(
                'flex items-center gap-2 px-2.5 py-1.5 rounded-lg border transition-colors',
                isRunning
                    ? 'bg-emerald-500/[0.07] border-emerald-500/25 dark:bg-emerald-500/10'
                    : 'bg-white/60 dark:bg-[#120a1f]/60 border-slate-200/70 dark:border-[#40285e]/40 hover:border-slate-300 dark:hover:border-[#40285e]'
            )}
            {...(isRunning ? { 'aria-live': 'polite' as const } : {})}
        >
            {/* Queue position / running marker */}
            <span className="w-6 shrink-0 text-center text-[10px] font-mono font-bold text-slate-400 dark:text-slate-500">
                {isRunning ? '●' : `#${item.queuePosition ?? '–'}`}
            </span>

            {/* 4-tier origin badge */}
            <span
                className={cn(
                    'inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[9px] font-mono font-bold border shrink-0',
                    badge.badgeStyle
                )}
                title={badge.label}
            >
                <BadgeIcon className={cn('w-2.5 h-2.5', badge.iconClass)} />
                <span className="hidden sm:inline">{badge.label}</span>
                <span className="sr-only">{badge.label}</span>
            </span>

            {/* Status chip */}
            {isRunning ? (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[9px] font-mono font-bold bg-amber-500/15 text-amber-700 dark:text-amber-300 border border-amber-500/30 animate-pulse shrink-0">
                    <span className="w-1.5 h-1.5 rounded-full bg-amber-500 animate-pulse" />
                    RUNNING
                </span>
            ) : (
                <span className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[9px] font-mono font-bold bg-amber-500/10 text-amber-700 dark:text-amber-300/90 border border-amber-500/20 shrink-0">
                    <Clock className="w-2.5 h-2.5" />
                    QUEUED
                </span>
            )}

            {/* Stale badge */}
            {item.isStale && (
                <span
                    className="inline-flex items-center gap-1 px-1.5 py-0.5 rounded-md text-[9px] font-mono font-bold bg-slate-500/15 text-slate-600 dark:text-slate-300 border border-slate-400/30 shrink-0"
                    title="No fresh heartbeat within TTL"
                >
                    <AlertTriangle className="w-2.5 h-2.5" />
                    STALE
                </span>
            )}

            {/* Title + Id */}
            <button
                type="button"
                onClick={() => onSelectPrompt(item.id)}
                className="flex items-center gap-1.5 min-w-0 flex-1 text-left cursor-pointer group"
                title={`${item.title || '(untitled prompt)'}\nId: ${item.id}`}
            >
                {item.is_subagent && <span className="text-purple-400 shrink-0 font-mono text-[10px]">↳</span>}
                <span className="text-[10px] font-mono text-slate-400 dark:text-slate-500 shrink-0">
                    #{item.id.length > 8 ? item.id.slice(0, 8) : item.id}
                </span>
                <span className="text-[11px] font-medium text-slate-700 dark:text-slate-200 truncate group-hover:text-purple-600 dark:group-hover:text-purple-300 transition-colors">
                    {renderTitle(item.title ?? null)}
                </span>
            </button>

            {/* Elapsed (running row) */}
            {isRunning && elapsedBase && (
                <span className="text-[10px] font-mono text-slate-500 dark:text-slate-400 shrink-0">
                    {formatElapsed(nowTick - elapsedBase)}
                </span>
            )}

            {/* Image marker */}
            {item.has_images && (
                <span
                    className="text-[9px] font-mono text-sky-600 dark:text-sky-400 shrink-0"
                    title="Prompt carries images"
                >
                    🖼
                </span>
            )}

            <RowActions
                item={item}
                onSelectPrompt={onSelectPrompt}
                onRerun={onRerun}
                onResendWithImage={onResendWithImage}
            />
        </div>
    );
}

export function NowRunningUpNext({
    items,
    isLoading,
    lastSyncAt,
    hasError,
    upNextTotal,
    onSelectPrompt,
    onRerun,
    onResendWithImage,
    onRefresh,
}: NowRunningUpNextProps) {
    const [isCollapsed, setIsCollapsed] = useState(false);
    const [isOverflowOpen, setIsOverflowOpen] = useState(false);

    // Self-hide when empty (spec §1.1, §1.5): no empty chrome.
    if (!isLoading && items.length === 0 && !hasError) {
        return null;
    }

    const runningItem = items.find((i) => i.isRunningRow) ?? null;
    const queuedItems = items.filter((i) => !i.isRunningRow);
    const visibleQueued = isOverflowOpen ? queuedItems : queuedItems.slice(0, 4);
    const overflowCount = upNextTotal > 4 ? upNextTotal - 4 : 0;
    const headerCount = upNextTotal > 4 ? `Up next (4 of ${upNextTotal})` : `Up next (${upNextTotal})`;

    const handleRefreshClick = () => {
        try {
            const res = onRefresh() as unknown;
            if (res && typeof (res as Promise<void>).catch === 'function') {
                (res as Promise<void>).catch((err: unknown) => {
                    useErrorStore.getState().trackWarning(err, {
                        source: 'NowRunningUpNext.onRefresh',
                        triggerAction: 'manual_refresh',
                    });
                });
            }
        } catch (err: unknown) {
            useErrorStore.getState().trackWarning(err, {
                source: 'NowRunningUpNext.onRefresh',
                triggerAction: 'manual_refresh',
            });
        }
    };

    return (
        <section
            role="region"
            aria-label="Now running and up next"
            className="shrink-0 border-b border-slate-200 dark:border-[#40285e] bg-slate-50/80 dark:bg-[#0e0818]/80"
        >
            {/* Strip header */}
            <button
                type="button"
                onClick={() => setIsCollapsed((v) => !v)}
                className="w-full flex items-center justify-between px-3 py-1.5 cursor-pointer hover:bg-slate-100/70 dark:hover:bg-[#171025]/70 transition-colors"
                aria-expanded={!isCollapsed}
                aria-label={isCollapsed ? 'Expand now running and up next panel' : 'Collapse now running and up next panel'}
            >
                <span className="flex items-center gap-2 min-w-0">
                    {isCollapsed ? (
                        <ChevronRight className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    ) : (
                        <ChevronDown className="w-3.5 h-3.5 text-slate-400 shrink-0" />
                    )}
                    <span className="text-[10px] font-mono font-bold uppercase tracking-wider text-slate-500 dark:text-slate-400 truncate">
                        {runningItem ? 'Now running' : 'No prompt running'}
                        <span className="mx-1.5 text-slate-300 dark:text-slate-600">·</span>
                        {queuedItems.length > 0 || upNextTotal > 0 ? headerCount : 'Queue empty'}
                    </span>
                </span>
                <span className="flex items-center gap-1.5 shrink-0">
                    {hasError && (
                        <span className="inline-flex items-center gap-1 text-[9px] font-mono font-bold text-amber-700 dark:text-amber-300">
                            <AlertTriangle className="w-3 h-3" />
                            stale
                        </span>
                    )}
                    <span className="text-[9px] font-mono text-slate-400 dark:text-slate-500">
                        synced {formatSyncAge(lastSyncAt)}
                    </span>
                    <span
                        role="button"
                        tabIndex={0}
                        onClick={(e) => {
                            e.stopPropagation();
                            handleRefreshClick();
                        }}
                        onKeyDown={(e) => {
                            if (e.key === 'Enter' || e.key === ' ') {
                                e.preventDefault();
                                e.stopPropagation();
                                handleRefreshClick();
                            }
                        }}
                        className="p-1 rounded-md text-slate-400 hover:text-purple-500 hover:bg-slate-200/60 dark:hover:bg-[#40285e]/60 transition-colors cursor-pointer"
                        title="Refresh now"
                        aria-label="Refresh now running and up next"
                    >
                        <RefreshCw className="w-3 h-3" />
                    </span>
                </span>
            </button>

            {!isCollapsed && (
                <div className="px-2 pb-2 space-y-1">
                    {isLoading && items.length === 0 ? (
                        // Skeleton rows while loading (spec §1.5).
                        <div className="space-y-1" aria-label="Loading running and queued prompts">
                            {[0, 1, 2].map((n) => (
                                <div
                                    key={n}
                                    className="h-9 rounded-lg bg-slate-200/70 dark:bg-[#211631]/70 animate-pulse"
                                />
                            ))}
                        </div>
                    ) : (
                        <>
                            {runningItem && (
                                <PanelRow
                                    item={runningItem}
                                    onSelectPrompt={onSelectPrompt}
                                    onRerun={onRerun}
                                    onResendWithImage={onResendWithImage}
                                />
                            )}
                            {visibleQueued.map((item) => (
                                <PanelRow
                                    key={item.id}
                                    item={item}
                                    onSelectPrompt={onSelectPrompt}
                                    onRerun={onRerun}
                                    onResendWithImage={onResendWithImage}
                                />
                            ))}
                            {overflowCount > 0 && !isOverflowOpen && (
                                <button
                                    type="button"
                                    onClick={() => setIsOverflowOpen(true)}
                                    className="w-full flex items-center justify-center gap-1.5 py-1.5 rounded-lg text-[11px] font-semibold text-purple-600 dark:text-purple-300 hover:bg-purple-500/10 border border-dashed border-purple-500/30 transition-colors cursor-pointer"
                                    aria-label={`Show ${overflowCount} more queued prompts`}
                                >
                                    <ChevronDown className="w-3.5 h-3.5" />
                                    <span>+{overflowCount} more in queue</span>
                                </button>
                            )}
                            {isOverflowOpen && overflowCount > 0 && (
                                <button
                                    type="button"
                                    onClick={() => setIsOverflowOpen(false)}
                                    className="w-full flex items-center justify-center gap-1.5 py-1 rounded-lg text-[11px] font-medium text-slate-500 dark:text-slate-400 hover:bg-slate-200/60 dark:hover:bg-[#40285e]/40 transition-colors cursor-pointer"
                                    aria-label="Collapse queued prompts"
                                >
                                    <ChevronRight className="w-3.5 h-3.5 rotate-[-90deg]" />
                                    <span>Show less</span>
                                </button>
                            )}
                            {hasError && items.length > 0 && (
                                <div className="flex items-center justify-between px-2 py-1 rounded-lg bg-amber-500/10 border border-amber-500/25 text-[10px] text-amber-800 dark:text-amber-200">
                                    <span className="flex items-center gap-1.5">
                                        <AlertTriangle className="w-3 h-3" />
                                        Showing last-good data — refresh failed.
                                    </span>
                                    <button
                                        type="button"
                                        onClick={handleRefreshClick}
                                        className="flex items-center gap-1 px-2 py-0.5 rounded-md bg-amber-500/20 hover:bg-amber-500/30 font-semibold transition-colors cursor-pointer"
                                    >
                                        <RefreshCw className="w-3 h-3" />
                                        Retry
                                    </button>
                                </div>
                            )}
                        </>
                    )}
                </div>
            )}
        </section>
    );
}
