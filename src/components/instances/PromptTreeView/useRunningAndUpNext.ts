import { useCallback, useEffect, useRef, useState } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { useErrorStore } from '../../../stores/error-store';

/**
 * useRunningAndUpNext — data hook for the "Now Running + Up Next" panel strip.
 *
 * Consumes the `get_running_and_up_next` IPC (spec 154 §6.1, implemented by
 * subtask 01). Polls every 3s while the prompt tree view is mounted and also
 * refreshes immediately on backend dispatch/resume events for immediacy.
 *
 * Staleness (spec §6.3, no flicker): a row flips to `isStale` only after
 * RUNNING_UP_NEXT_TTL_MS (30s) without being seen in a poll — never on a
 * single missed poll. Stale rows are retained (with badge) until
 * RUNNING_UP_NEXT_DROP_MS, then dropped. They are never silently dropped.
 *
 * Ordering: running item first (index 0), queued items FIFO after it. The
 * strip renders verbatim — no re-sorting here (single source of truth).
 */

/** TTL after which an unseen row flips to stale (spec §6.3, A11). */
export const RUNNING_UP_NEXT_TTL_MS = 30_000;
/** After this long unseen, a row is dropped from the panel. */
export const RUNNING_UP_NEXT_DROP_MS = 90_000;
/** Poll cadence while the tree view is mounted (spec §6.4). */
export const RUNNING_UP_NEXT_POLL_MS = 3_000;
/** Backend events that trigger an immediate refresh (immediacy). */
const REFRESH_EVENTS = ['prompt://dispatched', 'prompt://resumed'] as const;

export interface RunningQueueItem {
    id: string;
    instance_id: string;
    instance_name?: string | null;
    title?: string | null;
    status: string;
    origin_tier?: string | null;
    created_at?: number | null;
    started_at?: number | null;
    has_images?: boolean | null;
    is_subagent?: boolean | null;
    parent_id?: string | null;
}

export interface UpNextDisplayItem extends RunningQueueItem {
    /** Client-side TTL staleness (spec §6.3). */
    isStale: boolean;
    /** Last time this row was seen in a successful poll (ms epoch). */
    lastSeenAtMs: number;
    /** 1-based FIFO position for queued rows; null for the running row. */
    queuePosition: number | null;
    /** True for the running row (index 0). */
    isRunningRow: boolean;
}

interface RunningAndUpNextResponse {
    now_running?: RunningQueueItem | null;
    up_next?: RunningQueueItem[] | null;
    up_next_total?: number | null;
    instance_id?: string | null;
    as_of_ms?: number | null;
}

export interface UseRunningAndUpNextResult {
    /** Ordered: running row first, then queued rows FIFO. */
    items: UpNextDisplayItem[];
    runningItem: UpNextDisplayItem | null;
    queuedItems: UpNextDisplayItem[];
    /** Full queued count from the backend ("Up next (4 of 11)"). */
    upNextTotal: number;
    isLoading: boolean;
    /** True when the last poll failed; last-good items are kept with stale marking. */
    hasError: boolean;
    lastSyncAt: number | null;
    onRefresh: () => Promise<void>;
}

function toDisplayItem(
    raw: RunningQueueItem,
    now: number,
    isRunningRow: boolean,
    queuePosition: number | null
): UpNextDisplayItem {
    return {
        id: raw.id,
        instance_id: raw.instance_id ?? '',
        instance_name: raw.instance_name ?? null,
        title: raw.title ?? null,
        status: raw.status ?? '',
        origin_tier: raw.origin_tier ?? null,
        created_at: raw.created_at ?? null,
        started_at: raw.started_at ?? null,
        has_images: raw.has_images ?? false,
        is_subagent: raw.is_subagent ?? false,
        parent_id: raw.parent_id ?? null,
        isStale: false,
        lastSeenAtMs: now,
        queuePosition,
        isRunningRow,
    };
}

export function useRunningAndUpNext(
    instanceId: string | null | undefined,
    enabled: boolean
): UseRunningAndUpNextResult {
    const [items, setItems] = useState<UpNextDisplayItem[]>([]);
    const [upNextTotal, setUpNextTotal] = useState(0);
    const [isLoading, setIsLoading] = useState(true);
    const [hasError, setHasError] = useState(false);
    const [lastSyncAt, setLastSyncAt] = useState<number | null>(null);
    const instanceIdRef = useRef(instanceId);
    instanceIdRef.current = instanceId;
    const pollInFlightRef = useRef(false);

    const poll = useCallback(async (isManual: boolean): Promise<void> => {
        if (pollInFlightRef.current) return;
        pollInFlightRef.current = true;
        const targetId = instanceIdRef.current || null;
        try {
            const res = await invoke<RunningAndUpNextResponse>('get_running_and_up_next', {
                instanceId: targetId,
            });
            const now = Date.now();
            const asOf = typeof res?.as_of_ms === 'number' && Number.isFinite(res.as_of_ms) ? res.as_of_ms : now;

            const fresh: UpNextDisplayItem[] = [];
            let queuePos = 0;
            const pushRaw = (raw: RunningQueueItem | null | undefined, isRunningRow: boolean) => {
                if (!raw || typeof raw.id !== 'string' || raw.id.length === 0) return;
                if (!isRunningRow) queuePos += 1;
                fresh.push(toDisplayItem(raw, now, isRunningRow, isRunningRow ? null : queuePos));
            };
            pushRaw(res?.now_running ?? null, true);
            for (const q of res?.up_next ?? []) pushRaw(q, false);

            const freshIds = new Set(fresh.map((i) => i.id));
            setItems((prev) => {
                const retained: UpNextDisplayItem[] = [];
                for (const old of prev) {
                    if (freshIds.has(old.id)) continue;
                    const ageMs = now - old.lastSeenAtMs;
                    // Drop only after the drop TTL — never silently on a single missed poll.
                    if (ageMs > RUNNING_UP_NEXT_DROP_MS) continue;
                    retained.push({ ...old, isStale: ageMs > RUNNING_UP_NEXT_TTL_MS });
                }
                return [...fresh, ...retained];
            });
            setUpNextTotal(
                typeof res?.up_next_total === 'number' && Number.isFinite(res.up_next_total)
                    ? Math.max(0, Math.floor(res.up_next_total))
                    : fresh.length
            );
            setLastSyncAt(asOf);
            setHasError(false);
            setIsLoading(false);
        } catch (err: unknown) {
            const meta = {
                source: 'useRunningAndUpNext.poll',
                triggerAction: 'get_running_and_up_next',
                targetId: targetId ?? undefined,
            };
            if (isManual) {
                // User explicitly asked for fresh data: surface it.
                useErrorStore.getState().captureError(err, meta);
            } else {
                // Background poll: benign — last-good items stay visible with stale marking.
                useErrorStore.getState().trackWarning(err, meta);
            }
            setHasError(true);
            setIsLoading(false);
        } finally {
            pollInFlightRef.current = false;
        }
    }, []);

    const onRefresh = useCallback(async (): Promise<void> => {
        await poll(true);
    }, [poll]);

    // Poll every 3s while enabled; immediate poll on enable.
    useEffect(() => {
        if (!enabled) return;
        setIsLoading(true);
        void poll(false);
        const timer = setInterval(() => {
            void poll(false);
        }, RUNNING_UP_NEXT_POLL_MS);
        return () => clearInterval(timer);
    }, [enabled, instanceId, poll]);

    // Staleness re-evaluation tick: flip isStale flags even between polls.
    useEffect(() => {
        if (!enabled) return;
        const timer = setInterval(() => {
            const now = Date.now();
            setItems((prev) => {
                let changed = false;
                const next = prev.map((item) => {
                    const shouldBeStale = now - item.lastSeenAtMs > RUNNING_UP_NEXT_TTL_MS;
                    if (shouldBeStale !== item.isStale) {
                        changed = true;
                        return { ...item, isStale: shouldBeStale };
                    }
                    return item;
                });
                return changed ? next : prev;
            });
        }, 10_000);
        return () => clearInterval(timer);
    }, [enabled]);

    // Backend dispatch/resume events → immediate refresh (no waiting for the next poll).
    useEffect(() => {
        if (!enabled) return;
        let unlistenFns: Array<() => void> = [];
        let cancelled = false;
        void import('@tauri-apps/api/event')
            .then(({ listen }) => {
                if (cancelled) return;
                return Promise.all(
                    REFRESH_EVENTS.map((ev) =>
                        listen(ev, () => {
                            void poll(false);
                        })
                    )
                ).then((fns) => {
                    if (cancelled) {
                        fns.forEach((fn) => fn());
                        return;
                    }
                    unlistenFns = fns;
                });
            })
            .catch((err: unknown) => {
                useErrorStore.getState().trackWarning(err, {
                    source: 'useRunningAndUpNext.eventListen',
                    triggerAction: 'listen_prompt_events',
                });
            });
        return () => {
            cancelled = true;
            unlistenFns.forEach((fn) => {
                try {
                    fn();
                } catch {
                    // Best-effort unlisten; never throw from cleanup.
                }
            });
        };
    }, [enabled, poll]);

    const runningItem = items.find((i) => i.isRunningRow) ?? null;
    const queuedItems = items.filter((i) => !i.isRunningRow);

    return {
        items,
        runningItem,
        queuedItems,
        upNextTotal,
        isLoading,
        hasError,
        lastSyncAt,
        onRefresh,
    };
}
