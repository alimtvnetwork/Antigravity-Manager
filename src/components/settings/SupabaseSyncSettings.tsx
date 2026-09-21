import { useState, useEffect } from 'react';
import {
    Database,
    Server,
    RefreshCw,
    Plus,
    Trash2,
    CheckCircle2,
    AlertCircle,
    Loader2,
    Copy,
    Download,
    Upload,
    Sparkles,
    Lock,
    Cpu,
} from 'lucide-react';
import {
    supabaseService,
    SupabaseConfig,
    SupabaseEndpoint,
    LocalNodeInfo,
} from '../../services/supabaseService';
import ModalDialog from '../common/ModalDialog';
import { showToast } from '../common/ToastContainer';

export default function SupabaseSyncSettings() {
    const [config, setConfig] = useState<SupabaseConfig | null>(null);
    const [nodeInfo, setNodeInfo] = useState<LocalNodeInfo | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isSaving, setIsSaving] = useState(false);
    const [testingEndpointId, setTestingEndpointId] = useState<string | null>(null);
    const [testResults, setTestResults] = useState<Record<string, { isSuccess: boolean; msg: string }>>({});

    // Modals
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [isSchemaModalOpen, setIsSchemaModalOpen] = useState(false);
    const [schemaRole, setSchemaRole] = useState<'root' | 'secondary'>('root');
    const [schemaSql, setSchemaSql] = useState('');
    const [isExportModalOpen, setIsExportModalOpen] = useState(false);
    const [exportContent, setExportContent] = useState('');
    const [exportFormat, setExportFormat] = useState<'json' | 'yaml'>('json');
    const [exportRounds, setExportRounds] = useState(4);
    const [isImportModalOpen, setIsImportModalOpen] = useState(false);
    const [importText, setImportText] = useState('');
    const [isAiPromptModalOpen, setIsAiPromptModalOpen] = useState(false);

    // Form state for adding/editing endpoint
    const [formEndpoint, setFormEndpoint] = useState<Partial<SupabaseEndpoint>>({
        name: '',
        url: '',
        api_key: '',
        role: 'secondary',
        is_enabled: true,
        prune_threshold_mb: 200,
        priority: 1,
    });

    useEffect(() => {
        loadData();
    }, []);

    const loadData = async () => {
        setIsLoading(true);
        try {
            const [loadedConfig, loadedNode] = await Promise.all([
                supabaseService.getConfig(),
                supabaseService.getLocalNodeInfo(),
            ]);
            setConfig(loadedConfig);
            setNodeInfo(loadedNode);
        } catch (e) {
            console.error('Failed to load Supabase settings:', e);
        } finally {
            setIsLoading(false);
        }
    };

    const handleSaveConfig = async (newConfig: SupabaseConfig) => {
        setIsSaving(true);
        try {
            await supabaseService.saveConfig(newConfig);
            setConfig(newConfig);
            showToast('Supabase settings saved successfully', 'success');
        } catch (e) {
            showToast(`Failed to save: ${String(e)}`, 'error');
        } finally {
            setIsSaving(false);
        }
    };

    const handleTestEndpoint = async (endpoint: SupabaseEndpoint) => {
        setTestingEndpointId(endpoint.id);
        const isOk = await supabaseService.testEndpoint(endpoint);
        setTestResults((prev) => ({
            ...prev,
            [endpoint.id]: {
                isSuccess: isOk,
                msg: isOk ? 'Connected successfully' : 'Connection failed',
            },
        }));
        setTestingEndpointId(null);
    };

    const handleOpenSchemaModal = async (role: 'root' | 'secondary') => {
        setSchemaRole(role);
        try {
            const sql = await supabaseService.getSchemaSql(role);
            setSchemaSql(sql);
            setIsSchemaModalOpen(true);
        } catch (e) {
            showToast('Failed to load schema SQL', 'error');
        }
    };

    const handleOpenExport = async (format: 'json' | 'yaml') => {
        setExportFormat(format);
        try {
            const text = await supabaseService.exportConfig(format, exportRounds);
            setExportContent(text);
            setIsExportModalOpen(true);
        } catch (e) {
            showToast('Failed to generate export', 'error');
        }
    };

    const handleImportSubmit = async () => {
        if (!importText.trim()) return;
        try {
            const updated = await supabaseService.importConfig(importText);
            setConfig(updated);
            setIsImportModalOpen(false);
            setImportText('');
            showToast('Configuration imported successfully', 'success');
        } catch (e) {
            showToast(`Import failed: ${String(e)}`, 'error');
        }
    };

    const handleAddEndpointSubmit = () => {
        if (!config || !formEndpoint.name || !formEndpoint.url || !formEndpoint.api_key) {
            showToast('Please fill all required endpoint fields', 'error');
            return;
        }

        const newEp: SupabaseEndpoint = {
            id: `ep_${Date.now()}`,
            name: formEndpoint.name,
            url: formEndpoint.url,
            api_key: formEndpoint.api_key,
            role: formEndpoint.role as 'root' | 'secondary',
            is_enabled: formEndpoint.is_enabled ?? true,
            prune_threshold_mb: formEndpoint.prune_threshold_mb ?? (formEndpoint.role === 'root' ? 400 : 200),
            priority: formEndpoint.priority ?? 1,
        };

        const updatedConfig = {
            ...config,
            endpoints: [...config.endpoints, newEp],
        };

        handleSaveConfig(updatedConfig);
        setIsAddModalOpen(false);
        setFormEndpoint({
            name: '',
            url: '',
            api_key: '',
            role: 'secondary',
            is_enabled: true,
            prune_threshold_mb: 200,
            priority: 1,
        });
    };

    const handleDeleteEndpoint = (id: string) => {
        if (!config) return;
        const updated = {
            ...config,
            endpoints: config.endpoints.filter((ep) => ep.id !== id),
        };
        handleSaveConfig(updated);
    };

    const aiInstructionTemplate = `You are a DevOps Assistant for Antigravity Manager.
Convert my Supabase project credentials into the following strict JSON format for AGM cross-machine synchronization:

\`\`\`json
{
  "version": "1.0.0",
  "node_alias": "Node-${nodeInfo?.node_alias || 'Primary'}",
  "is_sync_enabled": true,
  "auto_prune_root_mb": 400,
  "auto_prune_secondary_mb": 200,
  "heartbeat_interval_secs": 30,
  "endpoints": [
    {
      "id": "ep_root_01",
      "name": "Supabase Root DB",
      "url": "<YOUR_SUPABASE_PROJECT_URL>",
      "api_key": "<YOUR_SUPABASE_SERVICE_ROLE_OR_ANON_KEY>",
      "role": "root",
      "is_enabled": true,
      "prune_threshold_mb": 400,
      "priority": 1
    },
    {
      "id": "ep_sec_01",
      "name": "Supabase Secondary Command DB",
      "url": "<YOUR_SECONDARY_SUPABASE_URL>",
      "api_key": "<YOUR_SECONDARY_SUPABASE_KEY>",
      "role": "secondary",
      "is_enabled": true,
      "prune_threshold_mb": 200,
      "priority": 1
    }
  ]
}
\`\`\`

Here are my Supabase details:
- Root Project URL: [Paste URL here]
- Root API Key: [Paste key here]
- Secondary Project URL (optional): [Paste secondary URL here]
- Secondary API Key (optional): [Paste secondary key here]`;

    if (isLoading) {
        return (
            <div className="flex items-center justify-center p-12 text-gray-400">
                <Loader2 className="w-6 h-6 animate-spin mr-2" />
                Loading Supabase synchronization configuration...
            </div>
        );
    }

    if (!config) return null;

    return (
        <div className="space-y-6 text-sm">
            {/* Header & Local Node Banner */}
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

                <div className="flex items-center gap-2">
                    <button
                        onClick={() => setIsAiPromptModalOpen(true)}
                        className="px-3 py-1.5 rounded-lg bg-indigo-600/20 hover:bg-indigo-600/30 text-indigo-300 border border-indigo-500/30 flex items-center gap-1.5 transition text-xs font-medium"
                    >
                        <Sparkles className="w-3.5 h-3.5" />
                        AI Prompt Template
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

            {/* Global Sync and Free-Tier Limits Card */}
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

            {/* Endpoints Management */}
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
                    <div className="p-8 rounded-xl bg-slate-900/40 border border-dashed border-slate-800 text-center text-gray-400">
                        <Database className="w-8 h-8 mx-auto mb-2 text-slate-600" />
                        <p className="font-medium">No Supabase endpoints configured</p>
                        <p className="text-xs text-gray-500 mt-1">
                            Add a Root database endpoint to begin synchronizing nodes and preventing account rotation collisions.
                        </p>
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
                                                <span className="text-[10px] text-emerald-400 bg-emerald-500/10 px-1.5 py-0.5 rounded">
                                                    Active
                                                </span>
                                            ) : (
                                                <span className="text-[10px] text-gray-500 bg-gray-500/10 px-1.5 py-0.5 rounded">
                                                    Disabled
                                                </span>
                                            )}
                                        </div>
                                        <div className="text-xs font-mono text-gray-400 truncate max-w-md">
                                            {ep.url}
                                        </div>
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
                            className="w-full px-3 py-2 bg-slate-950 border border-slate-800 rounded-lg text-white focus:outline-none focus:border-emerald-500 text-xs font-mono"
                        />
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

            {/* Schema SQL Modal */}
            <ModalDialog
                isOpen={isSchemaModalOpen}
                onClose={() => setIsSchemaModalOpen(false)}
                title={`PostgreSQL Schema Migration (${schemaRole.toUpperCase()} DB)`}
            >
                <div className="space-y-3">
                    <p className="text-xs text-gray-400">
                        Copy and execute this idempotent DDL script in your Supabase Project SQL Editor to provision required tables and stored functions.
                    </p>
                    <pre className="p-3 bg-slate-950 border border-slate-800 rounded-lg text-[11px] font-mono text-gray-300 overflow-x-auto max-h-72 select-all">
                        {schemaSql}
                    </pre>
                    <div className="flex justify-end gap-2">
                        <button
                            onClick={() => {
                                navigator.clipboard.writeText(schemaSql);
                                showToast('SQL copied to clipboard', 'success');
                            }}
                            className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            Copy SQL
                        </button>
                    </div>
                </div>
            </ModalDialog>

            {/* Export Modal with Iterative Base64 Rounds */}
            <ModalDialog
                isOpen={isExportModalOpen}
                onClose={() => setIsExportModalOpen(false)}
                title={`Export Supabase Configuration (${exportFormat.toUpperCase()})`}
            >
                <div className="space-y-4">
                    <div className="flex items-center justify-between p-3 rounded-lg bg-slate-950 border border-slate-800">
                        <div className="flex items-center gap-2">
                            <Lock className="w-4 h-4 text-emerald-400" />
                            <span className="text-xs text-gray-300 font-medium">
                                Multi-Pass Base64 Encryption Rounds:
                            </span>
                        </div>
                        <select
                            value={exportRounds}
                            onChange={(e) => {
                                const r = Number(e.target.value);
                                setExportRounds(r);
                                handleOpenExport(exportFormat);
                            }}
                            className="px-2 py-1 bg-slate-900 border border-slate-700 rounded text-xs text-white"
                        >
                            <option value={1}>1 Round (Standard)</option>
                            <option value={2}>2 Rounds</option>
                            <option value={3}>3 Rounds</option>
                            <option value={4}>4 Rounds (Recommended: 4$)</option>
                            <option value={6}>6 Rounds</option>
                        </select>
                    </div>

                    <textarea
                        readOnly
                        rows={10}
                        value={exportContent}
                        className="w-full p-3 bg-slate-950 border border-slate-800 rounded-lg text-xs font-mono text-gray-300 focus:outline-none select-all"
                    />

                    <div className="flex justify-end gap-2">
                        <button
                            onClick={() => {
                                navigator.clipboard.writeText(exportContent);
                                showToast('Export configuration copied', 'success');
                            }}
                            className="px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            Copy to Clipboard
                        </button>
                    </div>
                </div>
            </ModalDialog>

            {/* Import Modal */}
            <ModalDialog
                isOpen={isImportModalOpen}
                onClose={() => setIsImportModalOpen(false)}
                title="Import Supabase Configuration"
            >
                <div className="space-y-3">
                    <p className="text-xs text-gray-400">
                        Paste exported JSON/YAML configuration. Multi-pass Base64 keys (e.g. <code>4$payload</code>) are automatically decrypted.
                    </p>
                    <textarea
                        rows={10}
                        placeholder="Paste JSON / YAML config bundle here..."
                        value={importText}
                        onChange={(e) => setImportText(e.target.value)}
                        className="w-full p-3 bg-slate-950 border border-slate-800 rounded-lg text-xs font-mono text-white focus:outline-none focus:border-emerald-500"
                    />
                    <div className="flex justify-end gap-2">
                        <button
                            onClick={() => setIsImportModalOpen(false)}
                            className="px-4 py-2 rounded-lg bg-slate-800 text-gray-300 text-xs hover:bg-slate-700 transition"
                        >
                            Cancel
                        </button>
                        <button
                            onClick={handleImportSubmit}
                            className="px-4 py-2 rounded-lg bg-emerald-600 text-white text-xs hover:bg-emerald-500 font-medium transition"
                        >
                            Import & Decrypt
                        </button>
                    </div>
                </div>
            </ModalDialog>

            {/* AI Prompt Template Modal */}
            <ModalDialog
                isOpen={isAiPromptModalOpen}
                onClose={() => setIsAiPromptModalOpen(false)}
                title="AI Prompt Instruction Generator"
            >
                <div className="space-y-3">
                    <p className="text-xs text-gray-400">
                        Copy this instruction prompt and send it to ChatGPT, Claude, or Gemini alongside your Supabase project keys to instantly generate an AGM-compatible configuration bundle.
                    </p>
                    <pre className="p-3 bg-slate-950 border border-slate-800 rounded-lg text-[11px] font-mono text-gray-300 overflow-x-auto max-h-72 select-all">
                        {aiInstructionTemplate}
                    </pre>
                    <div className="flex justify-end gap-2">
                        <button
                            onClick={() => {
                                navigator.clipboard.writeText(aiInstructionTemplate);
                                showToast('AI Prompt copied to clipboard', 'success');
                            }}
                            className="px-3 py-1.5 rounded-lg bg-indigo-600 hover:bg-indigo-500 text-white text-xs flex items-center gap-1.5 transition font-medium"
                        >
                            <Copy className="w-3.5 h-3.5" />
                            Copy AI Prompt
                        </button>
                    </div>
                </div>
            </ModalDialog>
        </div>
    );
}
