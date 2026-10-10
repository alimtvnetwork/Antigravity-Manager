import { useNavigate } from 'react-router-dom';
import { Server, RotateCw, CloudOff, ExternalLink } from 'lucide-react';
import { cn } from '../../../utils/cn';

const cardClass =
    'rounded-xl bg-slate-900/50 dark:bg-[#07131e]/70 backdrop-blur-md border border-slate-700/40 dark:border-slate-800/60 shadow-lg';

export function FleetDisabledState() {
    const navigate = useNavigate();
    return (
        <div className={`${cardClass} p-5`}>
            <div className="flex flex-col sm:flex-row items-start sm:items-center justify-between gap-4">
                <div className="flex items-start gap-3">
                    <div className="p-2.5 rounded-lg bg-amber-500/10 text-amber-400 border border-amber-500/20 shrink-0 mt-0.5 sm:mt-0">
                        <CloudOff className="w-5 h-5" />
                    </div>
                    <div>
                        <h4 className="text-sm font-semibold text-slate-200">
                            Multi-Machine Fleet Sync is Disabled
                        </h4>
                        <p className="text-xs text-slate-400 mt-0.5 leading-relaxed">
                            Fleet monitoring and cross-machine instance leases are inactive. Enable Supabase Sync in configuration to coordinate instances and track remote fleet nodes across your network.
                        </p>
                    </div>
                </div>
                <button
                    type="button"
                    onClick={() => navigate('/supabase')}
                    className="shrink-0 px-3 py-1.5 rounded-lg text-xs font-medium bg-blue-600/20 text-blue-400 border border-blue-500/30 hover:bg-blue-600/30 hover:text-blue-300 transition-colors flex items-center gap-1.5 cursor-pointer"
                >
                    <span>Supabase Settings</span>
                    <ExternalLink className="w-3.5 h-3.5" />
                </button>
            </div>
        </div>
    );
}

export function FleetLoadingState() {
    return (
        <div className={`${cardClass} p-6 space-y-4`}>
            <div className="flex items-center justify-between">
                <div className="flex items-center gap-3">
                    <div className="w-8 h-8 rounded-lg bg-slate-800/60 animate-pulse" />
                    <div className="space-y-1.5">
                        <div className="w-36 h-4 rounded bg-slate-800/60 animate-pulse" />
                        <div className="w-56 h-3 rounded bg-slate-800/40 animate-pulse" />
                    </div>
                </div>
                <div className="w-24 h-7 rounded-full bg-slate-800/60 animate-pulse" />
            </div>
            <div className="space-y-2">
                {[1, 2, 3].map((i) => (
                    <div key={i} className="w-full h-10 rounded-lg bg-slate-800/40 animate-pulse" />
                ))}
            </div>
        </div>
    );
}

interface FleetEmptyStateProps {
    isRefreshing: boolean;
    onRefresh: () => void;
}

export function FleetEmptyState({ isRefreshing, onRefresh }: FleetEmptyStateProps) {
    return (
        <div className={`${cardClass} p-8 text-center`}>
            <div className="w-12 h-12 rounded-xl bg-slate-800/60 border border-slate-700/60 flex items-center justify-center mx-auto text-slate-400 mb-3 shadow-inner">
                <Server className="w-6 h-6 text-slate-400" />
            </div>
            <h4 className="text-sm font-semibold text-slate-200">
                No Remote Fleet Machines Detected
            </h4>
            <p className="text-xs text-slate-400 mt-1 max-w-md mx-auto leading-relaxed">
                This local machine is currently the sole active node registered in the Supabase cluster. Once other worker machines or fleet nodes join with the shared Supabase credentials, their status, instances, and active prompts will appear here automatically.
            </p>
            <div className="mt-4 flex items-center justify-center gap-2">
                <button
                    type="button"
                    onClick={onRefresh}
                    disabled={isRefreshing}
                    className="px-3 py-1.5 rounded-lg text-xs font-medium bg-slate-800/80 hover:bg-slate-700/80 text-slate-300 border border-slate-700/60 transition-colors flex items-center gap-1.5 cursor-pointer disabled:opacity-50"
                >
                    <RotateCw className={cn('w-3.5 h-3.5 text-cyan-400', isRefreshing && 'animate-spin')} />
                    <span>Check Again</span>
                </button>
            </div>
        </div>
    );
}
