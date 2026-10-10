import { useEffect, useMemo, useState } from 'react';
import { request } from '../../utils/request';
import { useErrorStore } from '../../stores/error-store';
import { isSchedulerAction, isSwitchAction } from './audit-helpers';
import type {
    TaskRecord,
    TaskDetail,
    TaskHistoryPage,
    TaskPayload,
    AuditFilterType,
} from '../../types/audit';

export function useAuditData() {
    const [page, setPage] = useState(0);
    const [pageSize, setPageSize] = useState<number>(() => {
        const saved = localStorage.getItem('audit_page_size');
        return saved === '200' ? 200 : 100;
    });
    const [filter, setFilter] = useState<AuditFilterType>('all');
    const [data, setData] = useState<TaskHistoryPage | null>(null);
    const [error, setError] = useState('');
    const [revealed, setRevealed] = useState<Record<string, boolean>>({});
    const [openId, setOpenId] = useState('');
    const [detail, setDetail] = useState<TaskDetail | null>(null);
    const [detailError, setDetailError] = useState('');
    const [copied, setCopied] = useState(false);

    const handlePageSizeChange = (newSize: number) => {
        setPageSize(newSize);
        localStorage.setItem('audit_page_size', String(newSize));
        setPage(0);
    };

    useEffect(() => {
        let alive = true;
        const fetchHistory = async () => {
            try {
                if (pageSize === 200) {
                    const [p1, p2] = await Promise.all([
                        request<TaskHistoryPage>('list_task_history', { offset: page * 200, limit: 100 }),
                        request<TaskHistoryPage>('list_task_history', { offset: page * 200 + 100, limit: 100 }).catch(() => null),
                    ]);
                    if (alive) {
                        const items = p2 ? [...p1.items, ...p2.items] : p1.items;
                        setData({
                            total: p1.total,
                            offset: page * 200,
                            limit: 200,
                            items,
                            splits: p1.splits,
                        });
                        setError('');
                    }
                } else {
                    const res = await request<TaskHistoryPage>('list_task_history', { offset: page * 100, limit: 100 });
                    if (alive) {
                        setData(res);
                        setError('');
                    }
                }
            } catch (err: unknown) {
                if (alive) setError(err instanceof Error ? err.message : String(err));
            }
        };
        fetchHistory();
        return () => {
            alive = false;
        };
    }, [page, pageSize]);

    const loadDetail = (id: string) => {
        setOpenId(id);
        setDetail(null);
        setDetailError('');
        setCopied(false);
        request<TaskDetail>('get_task_history_detail', { id })
            .then((result) => setDetail(result))
            .catch((err: unknown) => setDetailError(err instanceof Error ? err.message : String(err)));
    };

    const allItems = useMemo(() => data?.items || [], [data?.items]);

    const { switchItems, schedulerItems } = useMemo(() => {
        const sw: TaskRecord[] = [];
        const sc: TaskRecord[] = [];
        for (const item of allItems) {
            if (isSwitchAction(item)) sw.push(item);
            if (isSchedulerAction(item)) sc.push(item);
        }
        return { switchItems: sw, schedulerItems: sc };
    }, [allItems]);

    const filterCounts = useMemo(() => ({
        all: allItems.length,
        switch: switchItems.length,
        scheduler: schedulerItems.length,
    }), [allItems.length, switchItems.length, schedulerItems.length]);

    const displayItems = useMemo(() => {
        if (filter === 'switch') return switchItems;
        if (filter === 'scheduler') return schedulerItems;
        return allItems;
    }, [filter, switchItems, schedulerItems, allItems]);

    const total = data?.total ?? 0;
    const pageCount = Math.max(1, Math.ceil(total / pageSize));

    let payload: TaskPayload | null = null;
    if (detail?.payload_json) {
        try {
            payload = JSON.parse(detail.payload_json) as TaskPayload;
        } catch (e) {
            useErrorStore.getState().trackWarning(e, {
                source: 'Audit.parsePayload',
                triggerAction: 'JSON.parse',
            });
            payload = null;
        }
    }
    const detailRevealed = detail ? Boolean(revealed[detail.id]) : false;

    return {
        page, setPage,
        pageSize, handlePageSizeChange,
        filter, setFilter,
        data, error,
        revealed, setRevealed,
        openId, setOpenId,
        detail, setDetail, detailError,
        copied, setCopied,
        loadDetail,
        allItems, displayItems, filterCounts,
        total, pageCount,
        payload, detailRevealed,
    };
}
