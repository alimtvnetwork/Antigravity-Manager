import { Server, RotateCw, Eye, EyeOff } from 'lucide-react';
import { cn } from '../../utils/cn';
import { useFleetMachines } from './fleet/useFleetMachines';
import { FleetMachineRow } from './fleet/FleetMachineRow';
import { FleetDisabledState, FleetLoadingState, FleetEmptyState } from './fleet/FleetTableStates';

// Re-exported for backward compatibility (tests + external consumers import from here).
export { maskEmailAddress, formatRelativeHeartbeat } from './fleet/fleetFormatUtils';
export type { FleetInstanceSummary, FleetMachineInfo } from './fleet/fleetTypes';

export function FleetMachinesTable() {
    const {
        machines,
        isLoading,
        isRefreshing,
        isSyncEnabled,
        showAllEmails,
        unmaskedRows,
        copiedIp,
        countdown,
        setShowAllEmails,
        handleManualRefresh,
        handleCopyIp,
        toggleRowMask,
    } = useFleetMachines();

    if (!isLoading && !isSyncEnabled) {
        return <FleetDisabledState />;
    }

    if (isLoading) {
        return <FleetLoadingState />;
    }

    if (machines.length === 0) {
        return <FleetEmptyState isRefreshing={isRefreshing} onRefresh={handleManualRefresh} />;
    }

    return (
        <div className="rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 shadow-lg overflow-hidden">
            {/* Table Header toolbar */}
            <div className="px-4 py-3.5 border-b border-slate-700/40 dark:border-slate-800/60 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3 bg-slate-900/30">
                <div className="flex items-center gap-2.5">
                    <div className="p-1.5 rounded-lg bg-blue-500/10 text-blue-400 border border-blue-500/20">
                        <Server className="w-4 h-4" />
                    </div>
                    <div>
                        <div className="flex items-center gap-2">
                            <h3 className="font-semibold text-sm text-slate-100">
                                Other Machines / Fleet Nodes
                            </h3>
                            <span className="px-2 py-0.5 rounded-full text-[11px] font-bold bg-blue-500/20 text-blue-300 border border-blue-500/30">
                                {machines.length}
                            </span>
                        </div>
                        <p className="text-[11px] text-slate-400">
                            Real-time multi-machine cluster state synced via Supabase
                        </p>
                    </div>
                </div>

                {/* Segmented Pill Capsule Toolbar */}
                <div className="flex items-center rounded-full bg-slate-800/60 dark:bg-slate-900/80 border border-slate-700/60 p-0.5 text-xs divide-x divide-slate-700/60 shadow-xs self-stretch sm:self-auto justify-end">
                    {/* Countdown / Poll status pill */}
                    <div className="px-2.5 py-1 text-slate-400 flex items-center gap-1.5 font-mono text-[11px]">
                        <span className="inline-block w-1.5 h-1.5 rounded-full bg-blue-400 animate-pulse" />
                        <span>{countdown}s</span>
                    </div>

                    {/* Reveal / Mask all emails toggle */}
                    <button
                        type="button"
                        onClick={() => setShowAllEmails((prev) => !prev)}
                        className="px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors flex items-center gap-1.5 cursor-pointer"
                        title={showAllEmails ? 'Mask all account emails' : 'Reveal all account emails'}
                    >
                        {showAllEmails ? (
                            <EyeOff className="w-3.5 h-3.5 text-blue-400" />
                        ) : (
                            <Eye className="w-3.5 h-3.5 text-slate-400" />
                        )}
                        <span>{showAllEmails ? 'Mask' : 'Reveal'}</span>
                    </button>

                    {/* Manual refresh button */}
                    <button
                        type="button"
                        onClick={handleManualRefresh}
                        disabled={isRefreshing}
                        className="px-2.5 py-1 text-slate-300 hover:text-white hover:bg-slate-700/50 transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                        title="Refresh Fleet Status Now"
                    >
                        <RotateCw className={cn('w-3.5 h-3.5 text-cyan-400', isRefreshing && 'animate-spin')} />
                        <span>Refresh</span>
                    </button>
                </div>
            </div>

            {/* Table */}
            <div className="overflow-x-auto">
                <table className="w-full text-left text-xs">
                    <thead>
                        <tr className="border-b border-slate-700/40 dark:border-slate-800/60 bg-slate-950/40 text-[11px] font-semibold uppercase tracking-wider text-slate-400">
                            <th className="px-3.5 py-2.5 min-w-[170px]">Machine & Status</th>
                            <th className="px-3.5 py-2.5 min-w-[140px]">IP Address</th>
                            <th className="px-3.5 py-2.5 min-w-[180px]">Running Instance(s)</th>
                            <th className="px-3.5 py-2.5 min-w-[190px]">Bound Account Email(s)</th>
                            <th className="px-3.5 py-2.5 min-w-[130px]">Running Prompts</th>
                            <th className="px-3.5 py-2.5 text-right min-w-[110px]">Last Seen</th>
                        </tr>
                    </thead>
                    <tbody className="divide-y divide-slate-800/40 font-medium">
                        {machines.map((machine) => (
                            <FleetMachineRow
                                key={machine.node_id}
                                machine={machine}
                                isRowUnmasked={showAllEmails || Boolean(unmaskedRows[machine.node_id])}
                                copiedIp={copiedIp}
                                onCopyIp={handleCopyIp}
                                onToggleMask={toggleRowMask}
                            />
                        ))}
                    </tbody>
                </table>
            </div>
        </div>
    );
}

export default FleetMachinesTable;
