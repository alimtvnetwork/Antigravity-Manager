import { useState, useRef, useEffect } from 'react';
import { cn } from '../../utils/cn';
import { useAccountStore } from '../../stores/useAccountStore';
import { useErrorStore } from '../../stores/error-store';
import type { InstanceTableProps } from './table/instanceTableTypes';
import { buildRowCellData, ProfileCell, QuotaCell, StatusCell, PathCell } from './table/InstanceRowCells';
import { ActionsCell, MoreMenuPortal } from './table/InstanceRowActions';
import { resolveDataDir } from './table/instanceTableUtils';

// Re-exported for backward compatibility (Instances page imports from here).
export type { InstanceActionType } from './table/instanceTableTypes';

export default function InstanceTable({
    instances,
    activeInstanceId,
    actionState,
    onLaunch,
    onStop,
    onRestart,
    onSwitch,
    onFastForward,
    onAudit,
    onSync,
    onSettings,
    onClone,
    onDelete,
    onOpenPromptTree,
    onSetActive,
    onSetDefault,
}: InstanceTableProps) {
    const { accounts, currentAccount } = useAccountStore();
    const [revealedEmails, setRevealedEmails] = useState<Record<string, boolean>>({});
    const [copiedPathId, setCopiedPathId] = useState<string | null>(null);
    const [openMoreId, setOpenMoreId] = useState<string | null>(null);
    const [menuPos, setMenuPos] = useState<{ top: number; left: number } | null>(null);
    const moreMenuRef = useRef<HTMLDivElement>(null);

    useEffect(() => {
        const handleClickOutside = (e: MouseEvent) => {
            if (moreMenuRef.current && !moreMenuRef.current.contains(e.target as Node)) {
                setOpenMoreId(null);
            }
        };
        const handleScroll = () => {
            setOpenMoreId(null);
        };
        document.addEventListener('mousedown', handleClickOutside);
        window.addEventListener('scroll', handleScroll, true);
        return () => {
            document.removeEventListener('mousedown', handleClickOutside);
            window.removeEventListener('scroll', handleScroll, true);
        };
    }, []);

    const toggleEmail = (id: string) => {
        setRevealedEmails((prev) => ({ ...prev, [id]: !prev[id] }));
    };

    const handleCopyPath = async (id: string, path: string) => {
        try {
            await navigator.clipboard.writeText(path);
            setCopiedPathId(id);
            setTimeout(() => setCopiedPathId(null), 1500);
        } catch (err) {
            // Tracked in the error module; copy feedback simply won't show.
            useErrorStore.getState().trackWarning(err, {
                source: 'InstanceTable.copyPath',
                triggerAction: 'copy_path',
            });
        }
    };

    const handleToggleMore = (instId: string, anchor: HTMLElement) => {
        if (openMoreId === instId) {
            setOpenMoreId(null);
        } else {
            const rect = anchor.getBoundingClientRect();
            setMenuPos({ top: rect.bottom + 4, left: rect.right });
            setOpenMoreId(instId);
        }
    };

    const moreMenuInstance = openMoreId ? instances.find((i) => i.config.id === openMoreId) : undefined;

    return (
        <div className="w-full overflow-hidden rounded-[5px] border border-slate-200/90 dark:border-slate-800/90 bg-white dark:bg-[#0c2438] shadow-xs">
            <div className="overflow-x-auto scrollbar-thin scrollbar-thumb-slate-300 dark:scrollbar-thumb-[#15334d]">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-200/90 dark:border-slate-800/90 bg-slate-50/80 dark:bg-[#071a27] text-[11px] font-semibold uppercase tracking-wider text-slate-500 dark:text-slate-400">
                            <th className="px-2 py-1.5 min-w-[150px] max-w-[190px]">Profile & Account</th>
                            <th className="px-2 py-1.5 min-w-[160px] max-w-[200px]">Model & Weekly Quota</th>
                            <th className="px-2 py-1.5 min-w-[90px] max-w-[110px]">Status & PID</th>
                            <th className="px-2 py-1.5 min-w-[110px] max-w-[130px]">File / Data Path</th>
                            <th className="px-2 py-1.5 text-right w-[130px]">Actions</th>
                        </tr>
                    </thead>
                    <tbody className="font-medium">
                        {instances.map((inst, idx) => {
                            const data = buildRowCellData(
                                inst,
                                idx,
                                activeInstanceId,
                                accounts,
                                currentAccount,
                                revealedEmails,
                                actionState
                            );
                            return (
                                <tr
                                    key={inst.config.id}
                                    className={cn(
                                        'transition-colors duration-150 border-b border-slate-200/90 dark:border-slate-800/90 last:border-b-0',
                                        data.isBusy && 'pointer-events-none select-none opacity-60',
                                        data.isActive
                                            ? 'bg-sky-50/40 dark:bg-[#0a2338] hover:bg-sky-50/70 dark:hover:bg-[#0d2c46]'
                                            : 'hover:bg-slate-50/80 dark:hover:bg-[#0d253a]/70'
                                    )}
                                >
                                    <ProfileCell
                                        inst={inst}
                                        data={data}
                                        onToggleEmail={() => toggleEmail(inst.config.id)}
                                        onSetActive={() => onSetActive(inst.config.id)}
                                        onSetDefault={onSetDefault ? () => onSetDefault(inst.config.id) : undefined}
                                    />
                                    <QuotaCell data={data} />
                                    <StatusCell inst={inst} data={data} />
                                    <PathCell
                                        inst={inst}
                                        isCopied={copiedPathId === inst.config.id}
                                        onCopy={() => handleCopyPath(inst.config.id, resolveDataDir(inst))}
                                    />
                                    <ActionsCell
                                        inst={inst}
                                        currentAction={data.currentAction}
                                        isBusy={data.isBusy}
                                        isMoreOpen={openMoreId === inst.config.id}
                                        onLaunch={() => onLaunch(inst.config.id)}
                                        onStop={() => onStop(inst.config.id)}
                                        onRestart={() => onRestart(inst.config.id)}
                                        onSwitch={() => onSwitch(inst.config.id)}
                                        onFastForward={() => onFastForward(inst.config.id)}
                                        onToggleMore={(anchor) => handleToggleMore(inst.config.id, anchor)}
                                    />
                                </tr>
                            );
                        })}
                    </tbody>
                </table>
            </div>

            {openMoreId && menuPos && moreMenuInstance && (
                <MoreMenuPortal
                    inst={moreMenuInstance}
                    menuPos={menuPos}
                    menuRef={moreMenuRef}
                    onAudit={onAudit}
                    onSync={onSync}
                    onClose={() => setOpenMoreId(null)}
                    onOpenPromptTree={onOpenPromptTree}
                    onRestart={onRestart}
                    onSettings={onSettings}
                    onClone={onClone}
                    onDelete={onDelete}
                />
            )}
        </div>
    );
}
