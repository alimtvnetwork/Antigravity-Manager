import { SupabaseSyncApi } from './useSupabaseSync';
import { Database } from 'lucide-react';

export function GlobalSyncSection(props: SupabaseSyncApi) {
    const { config, handleSaveConfig } = props;

    if (!config) return null;

    return (
            <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 space-y-4">
                <div className="flex items-center justify-between">
                    <div>
                        <h4 className="font-semibold text-white flex items-center gap-2">
                            <Database className="w-4 h-4 text-emerald-400" />
                            Cross-Machine State Synchronization
                        </h4>
                        <p className="text-xs text-gray-400 mt-0.5">
                            Synchronize instance states, distributed account locks, and prompt queues across multiple machines via Supabase.
                        </p>
                    </div>
                    <label className="relative inline-flex items-center cursor-pointer">
                        <input
                            type="checkbox"
                            checked={config.is_sync_enabled}
                            onChange={(e) =>
                                handleSaveConfig({ ...config, is_sync_enabled: e.target.checked })
                            }
                            className="sr-only peer"
                        />
                        <div className="w-11 h-6 bg-slate-700 peer-focus:outline-none rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-emerald-600"></div>
                    </label>
                </div>

                <div className="grid grid-cols-1 md:grid-cols-2 gap-4 pt-2 border-t border-slate-800/80">
                    <div className="p-3 rounded-lg bg-slate-950/40 border border-slate-800/80">
                        <div className="flex items-center justify-between mb-1">
                            <span className="text-xs font-medium text-gray-300">Root DB Pruning Threshold</span>
                            <span className="text-xs font-mono text-emerald-400">{config.auto_prune_root_mb} MB</span>
                        </div>
                        <input
                            type="range"
                            min="100"
                            max="450"
                            step="25"
                            value={config.auto_prune_root_mb}
                            onChange={(e) =>
                                handleSaveConfig({ ...config, auto_prune_root_mb: Number(e.target.value) })
                            }
                            className="w-full accent-emerald-500 h-1.5 bg-slate-800 rounded-lg cursor-pointer"
                        />
                        <div className="flex justify-between text-[11px] text-gray-500 mt-1">
                            <span>100 MB</span>
                            <span>Max Free-Tier Safe: 450 MB (Ceiling 500 MB)</span>
                        </div>
                    </div>

                    <div className="p-3 rounded-lg bg-slate-950/40 border border-slate-800/80">
                        <div className="flex items-center justify-between mb-1">
                            <span className="text-xs font-medium text-gray-300">Secondary DB Pruning Threshold</span>
                            <span className="text-xs font-mono text-emerald-400">{config.auto_prune_secondary_mb} MB</span>
                        </div>
                        <input
                            type="range"
                            min="50"
                            max="350"
                            step="25"
                            value={config.auto_prune_secondary_mb}
                            onChange={(e) =>
                                handleSaveConfig({ ...config, auto_prune_secondary_mb: Number(e.target.value) })
                            }
                            className="w-full accent-emerald-500 h-1.5 bg-slate-800 rounded-lg cursor-pointer"
                        />
                        <div className="flex justify-between text-[11px] text-gray-500 mt-1">
                            <span>50 MB</span>
                            <span>Cascade / Prune Limit: 350 MB</span>
                        </div>
                    </div>
                </div>
            </div>

    );
}
