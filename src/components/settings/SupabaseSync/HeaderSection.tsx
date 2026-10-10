import { SupabaseSyncApi } from './useSupabaseSync';

export function HeaderSection(props: SupabaseSyncApi) {
    const { config, nodeInfo, isSaving, isAutoDiscovering, setIsImportModalOpen, setIsAiPromptModalOpen, setIsBackupModalOpen, handleAutoDiscover, handleOpenMigrateModal, handleOpenExport, text } = props;

    return (
            <div className="p-4 rounded-xl bg-slate-900/60 border border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-4">
                <div className="flex items-center gap-3">
                    <div className="p-2.5 rounded-lg bg-emerald-500/10 text-emerald-400 border border-emerald-500/20">
                        <Cpu className="w-6 h-6" />
                    </div>
                    <div>
                        <div className="flex items-center gap-2">
                            <span className="font-semibold text-white text-base">
                                {config.node_alias}
                            </span>
                            <span className="px-2 py-0.5 text-xs font-mono rounded bg-emerald-500/20 text-emerald-300 border border-emerald-500/30">
                                LOCAL NODE
                            </span>
                            {isSaving && (
                                <span className="flex items-center gap-1 text-[11px] text-amber-400 font-mono bg-amber-500/10 px-2 py-0.5 rounded border border-amber-500/20">
                                    <Loader2 className="w-3 h-3 animate-spin" /> Syncing...
                                </span>
                            )}
                        </div>
                        <div className="text-xs text-gray-400 flex items-center gap-4 mt-0.5">
                            <span>ID: <code className="text-gray-300">{nodeInfo?.node_id.slice(0, 12)}...</code></span>
                            <span>IP: <code className="text-gray-300">{nodeInfo?.ip_address}</code></span>
                            <span>Uptime: <code className="text-gray-300">{Math.round((nodeInfo?.uptime_seconds ?? 0) / 60)}m</code></span>
                        </div>
                    </div>
                </div>

                <div className="flex items-center gap-2 flex-wrap">
                    <button
                        onClick={handleAutoDiscover}
                        disabled={isAutoDiscovering}
                        className="px-3 py-1.5 rounded-[5px] bg-teal-600/20 hover:bg-teal-600/30 text-teal-300 border border-teal-500/30 flex items-center gap-1.5 transition text-xs font-medium cursor-pointer disabled:opacity-50"
                        title="Auto-discover Supabase credentials from local repo-secrets and vault"
                    >
                        {isAutoDiscovering ? (
                            <Loader2 className="w-3.5 h-3.5 animate-spin" />
                        ) : (
                            <Sparkles className="w-3.5 h-3.5 text-teal-400" />
                        )}
                        Auto-Discover Secrets
                    </button>
                    <button
                        onClick={handleOpenMigrateModal}
                        className="px-3 py-1.5 rounded-[5px] bg-amber-600/20 hover:bg-amber-600/30 text-amber-300 border border-amber-500/30 flex items-center gap-1.5 transition text-xs font-medium"
                        title="Migrate recent state from damaged/old Supabase to another"
                    >
                        <ArrowRightLeft className="w-3.5 h-3.5" />
                        Migrate to New DB
                    </button>
                    <button
                        onClick={() => setIsAiPromptModalOpen(true)}
                        className="px-3 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 flex items-center gap-1.5 transition text-xs font-medium"
                    >
                        <Sparkles className="w-3.5 h-3.5" />
                        AI Prompt Template
                    </button>
                    <button
                        onClick={() => setIsBackupModalOpen(true)}
                        className="px-3 py-1.5 rounded-lg bg-emerald-600/20 hover:bg-emerald-600/30 text-emerald-300 border border-emerald-500/30 flex items-center gap-1.5 transition text-xs font-medium cursor-pointer"
                        title="Encrypted Full System Backup & Restore"
                    >
                        <ShieldCheck className="w-3.5 h-3.5 text-emerald-400" />
                        Backup / Restore
                    </button>
                    <button
                        onClick={() => handleOpenExport('json')}
                        className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 flex items-center gap-1.5 transition text-xs"
                    >
                        <Download className="w-3.5 h-3.5" />
                        Export
                    </button>
                    <button
                        onClick={() => setIsImportModalOpen(true)}
                        className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 flex items-center gap-1.5 transition text-xs"
                    >
                        <Upload className="w-3.5 h-3.5" />
                        Import
                    </button>
                </div>
            </div>

    );
}
