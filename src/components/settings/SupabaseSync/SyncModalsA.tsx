import { SupabaseSyncApi } from './useSupabaseSync';
import ModalDialog from '../common/ModalDialog';

export function SyncModalsA(props: SupabaseSyncApi) {
    const { config, isAddModalOpen, setIsAddModalOpen, isMigrateModalOpen, setIsMigrateModalOpen, sourceEpId, setSourceEpId, targetEpId, setTargetEpId, isMigrating, migrationResult, leases, formEndpoint, setFormEndpoint, normalizeSupabaseUrl, handleMigrateSubmit, text, handleAddEndpointSubmit } = props;

    return (
        <>
                {/* Cross-DB Migration Modal */}
                <ModalDialog
                    isOpen={isMigrateModalOpen}
                    onClose={() => setIsMigrateModalOpen(false)}
                    title="Cross-Database Selective Data Migration"
                >
                    <div className="space-y-4 text-sm">
                        <p className="text-xs text-gray-400">
                            Migrate recent, critical state (online nodes, active profiles, unexpired leases, and last 50 commands) from a source or damaged Supabase endpoint to a new target endpoint.
                        </p>

                        <div>
                            <label className="block text-xs font-medium text-gray-300 mb-1">
                                Source Endpoint (Damaged / Current)
                            </label>
                            <select
                                value={sourceEpId}
                                onChange={(e) => setSourceEpId(e.target.value)}
                                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-amber-500 text-xs"
                            >
                                {config?.endpoints.map((ep) => (
                                    <option key={ep.id} value={ep.id}>
                                        {ep.name} ({ep.role} - {ep.url})
                                    </option>
                                ))}
                            </select>
                        </div>

                        <div>
                            <label className="block text-xs font-medium text-gray-300 mb-1">
                                Target Endpoint (New Supabase Project)
                            </label>
                            <select
                                value={targetEpId}
                                onChange={(e) => setTargetEpId(e.target.value)}
                                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-amber-500 text-xs"
                            >
                                {config?.endpoints.map((ep) => (
                                    <option key={ep.id} value={ep.id}>
                                        {ep.name} ({ep.role} - {ep.url})
                                    </option>
                                ))}
                            </select>
                        </div>

                        {migrationResult && (
                            <div
                                className={`p-3 rounded-lg border text-xs ${
                                    migrationResult.is_success
                                        ? 'bg-emerald-500/10 border-emerald-500/30 text-emerald-300'
                                        : 'bg-red-500/10 border-red-500/30 text-red-300'
                                }`}
                            >
                                <p className="font-semibold">{migrationResult.message}</p>
                                {migrationResult.is_success && (
                                    <div className="mt-1 font-mono text-[11px] text-gray-300 flex gap-3">
                                        <span>Nodes: {migrationResult.nodes_migrated}</span>
                                        <span>Profiles: {migrationResult.profiles_migrated}</span>
                                        <span>Leases: {migrationResult.leases_migrated}</span>
                                        <span>Commands: {migrationResult.commands_migrated}</span>
                                    </div>
                                )}
                            </div>
                        )}

                        <div className="flex justify-end gap-2 pt-2 border-t border-slate-800">
                            <button
                                onClick={() => setIsMigrateModalOpen(false)}
                                className="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 text-xs hover:bg-slate-700 transition"
                            >
                                Close
                            </button>
                            <button
                                onClick={handleMigrateSubmit}
                                disabled={isMigrating}
                                className="px-4 py-2 rounded-lg bg-amber-600 hover:bg-amber-500 text-white text-xs font-medium transition flex items-center gap-1.5"
                            >
                                {isMigrating ? (
                                    <Loader2 className="w-3.5 h-3.5 animate-spin" />
                                ) : (
                                    <ArrowRightLeft className="w-3.5 h-3.5" />
                                )}
                                Start Migration
                            </button>
                        </div>
                    </div>
                </ModalDialog>

                {/* Add Endpoint Modal */}
                <ModalDialog
                    isOpen={isAddModalOpen}
                    onClose={() => setIsAddModalOpen(false)}
                    title="Add Supabase Endpoint"
                >
                    <div className="space-y-4 text-sm">
                        <div>
                            <label className="block text-xs font-medium text-gray-300 mb-1">
                                Endpoint Name
                            </label>
                            <input
                                type="text"
                                placeholder="e.g. Primary Supabase Cluster"
                                value={formEndpoint.name}
                                onChange={(e) => setFormEndpoint({ ...formEndpoint, name: e.target.value })}
                                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs"
                            />
                        </div>

                        <div>
                            <label className="block text-xs font-medium text-gray-300 mb-1">
                                Supabase Project URL
                            </label>
                            <input
                                type="text"
                                placeholder="https://xyzcompany.supabase.co"
                                value={formEndpoint.url}
                                onChange={(e) => setFormEndpoint({ ...formEndpoint, url: e.target.value })}
                                onBlur={() =>
                                    setFormEndpoint((prev) => ({
                                        ...prev,
                                        url: prev.url ? normalizeSupabaseUrl(prev.url) : '',
                                    }))
                                }
                                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs font-mono"
                            />
                            <p className="text-[10px] text-gray-500 mt-1">
                                Enter the base Project URL (e.g. https://xyz.supabase.co). Redundant /rest/v1 paths and trailing slashes are automatically sanitized.
                            </p>
                        </div>

                        <div>
                            <label className="block text-xs font-medium text-gray-300 mb-1">
                                Supabase Service Role / Anon API Key
                            </label>
                            <input
                                type="password"
                                placeholder="eyJh... (kept encrypted via multi-pass Base64)"
                                value={formEndpoint.api_key}
                                onChange={(e) => setFormEndpoint({ ...formEndpoint, api_key: e.target.value })}
                                className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs font-mono"
                            />
                        </div>

                        <div className="grid grid-cols-2 gap-3">
                            <div>
                                <label className="block text-xs font-medium text-gray-300 mb-1">
                                    Database Role
                                </label>
                                <select
                                    value={formEndpoint.role}
                                    onChange={(e) =>
                                        setFormEndpoint({
                                            ...formEndpoint,
                                            role: e.target.value as 'root' | 'secondary',
                                        })
                                    }
                                    className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs"
                                >
                                    <option value="root">Root DB (Nodes & Locks)</option>
                                    <option value="secondary">Secondary DB (Prompts & Logs)</option>
                                </select>
                            </div>
                            <div>
                                <label className="block text-xs font-medium text-gray-300 mb-1">
                                    Prune Threshold (MB)
                                </label>
                                <input
                                    type="number"
                                    value={formEndpoint.prune_threshold_mb}
                                    onChange={(e) =>
                                        setFormEndpoint({
                                            ...formEndpoint,
                                            prune_threshold_mb: Number(e.target.value),
                                        })
                                    }
                                    className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs"
                                />
                            </div>
                        </div>

                        <div className="flex justify-end gap-2 pt-3 border-t border-slate-800">
                            <button
                                onClick={() => setIsAddModalOpen(false)}
                                className="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 text-xs hover:bg-slate-700 transition"
                            >
                                Cancel
                            </button>
                            <button
                                onClick={handleAddEndpointSubmit}
                                className="px-4 py-2 rounded-lg bg-emerald-600 text-white text-xs hover:bg-emerald-500 font-medium transition"
                            >
                                Save Endpoint
                            </button>
                        </div>
                    </div>
                </ModalDialog>

        </>
    );
}
