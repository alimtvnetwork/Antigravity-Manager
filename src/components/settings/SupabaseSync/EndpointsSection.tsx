import { SupabaseSyncApi } from './useSupabaseSync';
import { Server, Plus, Database, Loader2, Sparkles, CheckCircle2, AlertCircle, ShieldCheck, RefreshCw, Trash2 } from 'lucide-react';

export function EndpointsSection(props: SupabaseSyncApi) {
    const { config, testingEndpointId, testResults, verifyingEndpointId, tableVerification, isAutoDiscovering, setIsAddModalOpen, handleTestEndpoint, handleCheckTables, handleAutoDiscover, handleOpenSchemaModal, handleDeleteEndpoint } = props;

    if (!config) return null;

    return (
            <div className="space-y-3">
                <div className="flex items-center justify-between">
                    <div>
                        <h4 className="font-semibold text-white flex items-center gap-2">
                            <Server className="w-4 h-4 text-emerald-400" />
                            Supabase Database Endpoints ({config.endpoints.length})
                        </h4>
                        <p className="text-xs text-gray-400">
                            Root DB tracks machine heartbeats & account locks. Secondary DBs queue commands & telemetry.
                        </p>
                    </div>
                    <div className="flex items-center gap-2">
                        <button
                            onClick={() => handleOpenSchemaModal('root')}
                            className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs font-medium transition"
                        >
                            Root Schema SQL
                        </button>
                        <button
                            onClick={() => handleOpenSchemaModal('secondary')}
                            className="px-3 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs font-medium transition"
                        >
                            Secondary Schema SQL
                        </button>
                        <button
                            onClick={() => setIsAddModalOpen(true)}
                            className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs font-medium flex items-center gap-1 transition"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            Add Endpoint
                        </button>
                    </div>
                </div>

                {config.endpoints.length === 0 ? (
                    <div className="p-8 rounded-xl bg-slate-900/40 border border-dashed border-slate-800 text-center text-gray-400 flex flex-col items-center">
                        <Database className="w-8 h-8 mx-auto mb-2 text-slate-600" />
                        <p className="font-medium">No Supabase endpoints configured</p>
                        <p className="text-xs text-gray-500 mt-1 max-w-md">
                            Add a Root database endpoint or auto-discover from local repo-secrets to begin synchronizing nodes and preventing account rotation collisions.
                        </p>
                        <button
                            type="button"
                            onClick={handleAutoDiscover}
                            disabled={isAutoDiscovering}
                            className="mt-4 px-3.5 py-1.5 rounded-[5px] bg-teal-600 hover:bg-teal-500 text-white font-semibold text-xs flex items-center gap-1.5 transition-colors cursor-pointer disabled:opacity-50 shadow-xs"
                            title="Auto-discover Supabase credentials from local repo-secrets and vault"
                        >
                            {isAutoDiscovering ? (
                                <Loader2 className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Sparkles className="w-3.5 h-3.5 text-teal-200" />
                            )}
                            Auto-Discover from Repo Secrets
                        </button>
                    </div>
                ) : (
                    <div className="space-y-2">
                        {config.endpoints.map((ep) => {
                            const isTesting = testingEndpointId === ep.id;
                            const result = testResults[ep.id];
                            return (
                                <div
                                    key={ep.id}
                                    className="p-3.5 rounded-xl bg-slate-900/60 border border-slate-800 flex flex-col md:flex-row md:items-center justify-between gap-3"
                                >
                                    <div className="space-y-1">
                                        <div className="flex items-center gap-2">
                                            <span className="font-medium text-white">{ep.name}</span>
                                            <span
                                                className={`px-2 py-0.5 text-[10px] font-bold uppercase rounded border ${
                                                    ep.role === 'root'
                                                        ? 'bg-purple-500/20 text-purple-300 border-purple-500/30'
                                                        : 'bg-blue-500/20 text-blue-300 border-blue-500/30'
                                                }`}
                                            >
                                                {ep.role} DB
                                            </span>
                                            {ep.is_enabled ? (
                                                result && !result.isSuccess ? (
                                                    <span className="text-[10px] text-red-400 bg-red-500/10 px-1.5 py-0.5 rounded">
                                                        Unreachable
                                                    </span>
                                                ) : (
                                                    <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded">
                                                        Active
                                                    </span>
                                                )
                                            ) : (
                                                <span className="text-[10px] text-gray-500 bg-gray-500/10 px-1.5 py-0.5 rounded">
                                                    Disabled
                                                </span>
                                            )}
                                        </div>
                                        <div className="text-xs font-mono text-gray-400 truncate max-w-md">
                                            {ep.url}
                                        </div>
                                        {tableVerification[ep.id] && (
                                            <div className="flex flex-wrap items-center gap-1 pt-1">
                                                {tableVerification[ep.id].verified_tables.map((t) => (
                                                    <span
                                                        key={t}
                                                        className="px-1.5 py-0.5 text-[10px] bg-emerald-500/10 text-emerald-400 rounded border border-emerald-500/20 font-mono"
                                                    >
                                                        ✓ {t}
                                                    </span>
                                                ))}
                                                {tableVerification[ep.id].missing_tables.map((t) => (
                                                    <span
                                                        key={t}
                                                        className="px-1.5 py-0.5 text-[10px] bg-red-500/10 text-red-400 rounded border border-red-500/20 font-mono"
                                                    >
                                                        ✗ {t}
                                                    </span>
                                                ))}
                                            </div>
                                        )}
                                    </div>

                                    <div className="flex items-center gap-2">
                                        {result && (
                                            <span
                                                className={`text-xs flex items-center gap-1 ${
                                                    result.isSuccess ? 'text-emerald-400' : 'text-red-400'
                                                }`}
                                            >
                                                {result.isSuccess ? (
                                                    <CheckCircle2 className="w-3.5 h-3.5" />
                                                ) : (
                                                    <AlertCircle className="w-3.5 h-3.5" />
                                                )}
                                                {result.msg}
                                            </span>
                                        )}
                                        <button
                                            onClick={() => handleCheckTables(ep)}
                                            disabled={verifyingEndpointId === ep.id}
                                            className="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1 transition"
                                            title="Verify if required tables exist on Supabase"
                                        >
                                            {verifyingEndpointId === ep.id ? (
                                                <Loader2 className="w-3 h-3 animate-spin" />
                                            ) : (
                                                <ShieldCheck className="w-3 h-3 text-cyan-400" />
                                            )}
                                            Check Tables
                                        </button>
                                        <button
                                            onClick={() => handleTestEndpoint(ep)}
                                            disabled={isTesting}
                                            className="px-2.5 py-1.5 rounded-lg bg-slate-800 hover:bg-slate-700 text-gray-200 border border-slate-700 text-xs flex items-center gap-1 transition"
                                        >
                                            {isTesting ? (
                                                <Loader2 className="w-3 h-3 animate-spin" />
                                            ) : (
                                                <RefreshCw className="w-3 h-3" />
                                            )}
                                            Test
                                        </button>
                                        <button
                                            onClick={() => handleDeleteEndpoint(ep.id)}
                                            className="p-1.5 rounded-lg hover:bg-red-500/20 text-gray-400 hover:text-red-400 transition"
                                        >
                                            <Trash2 className="w-4 h-4" />
                                        </button>
                                    </div>
                                </div>
                            );
                        })}
                    </div>
                )}
            </div>

    );
}
