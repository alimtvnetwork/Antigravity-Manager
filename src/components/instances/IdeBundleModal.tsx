import { useState, useRef } from 'react';
import {
    X,
    FolderArchive,
    FileDown,
    FileUp,
    RotateCw,
    CheckCircle2,
    Check,
    Cpu,
} from 'lucide-react';
import { invoke } from '@tauri-apps/api/core';
import { cn } from '../../utils/cn';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

export interface IdeBundleModalProps {
    isOpen: boolean;
    onClose: () => void;
    instances: Array<{ id: string; name: string; is_default?: boolean }>;
    onSuccess?: () => void;
}

interface BundlePreview {
    agm_bundle_version: string;
    exported_at: string;
    source_environment: {
        instance_id: string;
        instance_name: string;
        agm_version: string;
        platform: string;
        arch: string;
    };
    metadata: {
        bundle_checksum: string;
        item_counts: Record<string, number>;
    };
}

interface ImportResult {
    instance_id: string;
    instance_name: string;
    settings_applied: number;
    projects_restored: number;
    prompts_queued: number;
}

export function IdeBundleModal({
    isOpen,
    onClose,
    instances,
    onSuccess,
}: IdeBundleModalProps) {
    const [activeTab, setActiveTab] = useState<'export' | 'import'>('export');

    // Export State
    const [exportInstanceId, setExportInstanceId] = useState<string>(
        instances[0]?.id || 'default'
    );
    const [includeSettings, setIncludeSettings] = useState(true);
    const [includeProjects, setIncludeProjects] = useState(true);
    const [includePrompts, setIncludePrompts] = useState(true);
    const [isExporting, setIsExporting] = useState(false);
    const [exportedChecksum, setExportedChecksum] = useState<string | null>(null);

    // Import State
    const [importMode, setImportMode] = useState<'new' | 'merge'>('new');
    const [newInstanceName, setNewInstanceName] = useState('');
    const [targetMergeId, setTargetMergeId] = useState<string>(
        instances[0]?.id || 'default'
    );
    const [rawBundleContent, setRawBundleContent] = useState<string | null>(null);
    const [bundlePreview, setBundlePreview] = useState<BundlePreview | null>(null);
    const [fileName, setFileName] = useState<string | null>(null);
    const [isImporting, setIsImporting] = useState(false);
    const fileInputRef = useRef<HTMLInputElement>(null);

    if (!isOpen) return null;

    const handleFileSelected = (e: React.ChangeEvent<HTMLInputElement>) => {
        const file = e.target.files?.[0];
        if (!file) return;

        setFileName(file.name);
        const reader = new FileReader();
        reader.onload = (event) => {
            try {
                const text = event.target?.result as string;
                const parsed = JSON.parse(text) as BundlePreview;
                if (!parsed.agm_bundle_version) {
                    showToast('Invalid bundle: Missing agm_bundle_version', 'error');
                    return;
                }
                setRawBundleContent(text);
                setBundlePreview(parsed);
                if (parsed.source_environment?.instance_name) {
                    setNewInstanceName(`${parsed.source_environment.instance_name}-Imported`);
                }
            } catch (err) {
                useErrorStore.getState().trackWarning(err, {
                    source: 'IdeBundleModal.handleFileSelected',
                    triggerAction: 'parse_bundle_json',
                });
                showToast('Failed to parse profile bundle JSON file', 'error');
            }
        };
        reader.readAsText(file);
    };

    const handleExecuteExport = async () => {
        setIsExporting(true);
        setExportedChecksum(null);
        try {
            const bundle = await invoke<any>('export_ide_bundle', {
                instanceId: exportInstanceId,
                outputPath: null,
            });

            // Filter out components if deselected
            const exportData = { ...bundle };
            if (!includeSettings) exportData.settings = { editor_settings: {}, keybindings: [], snippets: {} };
            if (!includeProjects) exportData.projects = { workspace_uris: [], pinned_project_sequences: [], running_projects: [] };
            if (!includePrompts) exportData.prompts = { active_prompts: [], suffix_templates: [], resend_as_templates: [], global_system_prompt: null };

            const jsonStr = JSON.stringify(exportData, null, 2);
            const blob = new Blob([jsonStr], { type: 'application/json' });
            const url = URL.createObjectURL(blob);
            const a = document.createElement('a');
            a.href = url;
            a.download = `${exportInstanceId}-profile.agmide`;
            document.body.appendChild(a);
            a.click();
            document.body.removeChild(a);
            URL.revokeObjectURL(url);

            setExportedChecksum(bundle.metadata?.bundle_checksum || 'sha256-verified');
            showToast('IDE profile bundle exported successfully!', 'success');
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'IdeBundleModal.handleExecuteExport',
                triggerAction: 'export_ide_bundle',
            });
            showToast(`Export failed: ${String(err)}`, 'error');
        } finally {
            setIsExporting(false);
        }
    };

    const handleExecuteImport = async () => {
        if (!rawBundleContent) {
            showToast('Please select an .agmide or .json bundle file first', 'error');
            return;
        }

        setIsImporting(true);
        try {
            const result = await invoke<ImportResult>('import_ide_bundle', {
                bundlePath: rawBundleContent,
                targetInstanceId: importMode === 'merge' ? targetMergeId : null,
                newInstanceName: importMode === 'new' ? (newInstanceName.trim() || 'Imported-Profile') : null,
                force: false,
            });

            showToast(
                `Profile imported into '${result.instance_name}' (${result.settings_applied} settings, ${result.projects_restored} projects, ${result.prompts_queued} prompts)`,
                'success'
            );
            onSuccess?.();
            onClose();
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'IdeBundleModal.handleExecuteImport',
                triggerAction: 'import_ide_bundle',
            });
            showToast(`Import failed: ${String(err)}`, 'error');
        } finally {
            setIsImporting(false);
        }
    };

    return (
        <div
            className="fixed inset-0 bg-black/60 backdrop-blur-xs flex items-center justify-center z-[99999] p-4"
            onClick={onClose}
        >
            <div
                className="bg-white dark:bg-[#171025] rounded-2xl w-full max-w-xl max-h-[90vh] shadow-2xl border border-gray-200 dark:border-[#40285e] flex flex-col overflow-hidden text-xs text-gray-900 dark:text-gray-100 animate-in fade-in zoom-in-95"
                onClick={(e) => e.stopPropagation()}
            >
                {/* Modal Header */}
                <div className="flex items-center justify-between px-5 py-3.5 border-b border-gray-100 dark:border-[#40285e] bg-gray-50/70 dark:bg-[#091e30]">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2 rounded-xl bg-blue-50 dark:bg-blue-900/30 text-blue-600 dark:text-cyan-400">
                            <FolderArchive className="w-5 h-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-gray-900 dark:text-slate-100 flex items-center gap-2">
                                <span>IDE Profile Bundle (.agmide)</span>
                                {(isExporting || isImporting) && (
                                    <RotateCw className="w-3.5 h-3.5 animate-spin text-blue-500" />
                                )}
                            </h2>
                            <p className="text-[11px] text-gray-500 dark:text-gray-400">
                                Seamlessly export and import settings, workspaces, sequences, and prompts
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={onClose}
                        className="p-1.5 rounded-lg hover:bg-gray-200 dark:hover:bg-[#40285e] text-gray-400 hover:text-gray-600 dark:hover:text-gray-200 transition-colors cursor-pointer"
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                {/* Tab Switcher */}
                <div className="flex border-b border-gray-100 dark:border-[#40285e] bg-slate-50 dark:bg-[#130d20] px-5 pt-2">
                    <button
                        type="button"
                        onClick={() => setActiveTab('export')}
                        className={cn(
                            'flex items-center gap-2 px-4 py-2 text-xs font-semibold border-b-2 transition-colors cursor-pointer',
                            activeTab === 'export'
                                ? 'border-blue-600 text-blue-600 dark:border-cyan-400 dark:text-cyan-300'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        )}
                    >
                        <FileDown className="w-4 h-4" />
                        <span>Export Profile Bundle</span>
                    </button>
                    <button
                        type="button"
                        onClick={() => setActiveTab('import')}
                        className={cn(
                            'flex items-center gap-2 px-4 py-2 text-xs font-semibold border-b-2 transition-colors cursor-pointer',
                            activeTab === 'import'
                                ? 'border-blue-600 text-blue-600 dark:border-cyan-400 dark:text-cyan-300'
                                : 'border-transparent text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200'
                        )}
                    >
                        <FileUp className="w-4 h-4" />
                        <span>Import Profile Bundle</span>
                    </button>
                </div>

                {/* Modal Body */}
                <div className="flex-1 overflow-y-auto p-5 space-y-4">
                    {activeTab === 'export' ? (
                        <div className="space-y-4">
                            {/* Source Selection */}
                            <div>
                                <label className="block text-[11px] font-bold text-gray-700 dark:text-gray-300 uppercase tracking-wider mb-1.5">
                                    Source Instance Profile
                                </label>
                                <select
                                    value={exportInstanceId}
                                    onChange={(e) => setExportInstanceId(e.target.value)}
                                    className="w-full px-3 py-2 rounded-lg bg-white dark:bg-[#1f1633] border border-gray-300 dark:border-[#40285e] text-xs font-medium text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                >
                                    {instances.map((inst) => (
                                        <option key={inst.id} value={inst.id}>
                                            {inst.name} ({inst.id}){inst.is_default ? ' [Default]' : ''}
                                        </option>
                                    ))}
                                </select>
                            </div>

                            {/* Components Checklist */}
                            <div className="space-y-2.5 bg-slate-50 dark:bg-[#1a122c] p-3.5 rounded-xl border border-gray-200/80 dark:border-[#40285e]">
                                <span className="text-[11px] font-bold text-gray-600 dark:text-gray-400 uppercase tracking-wider block mb-1">
                                    Bundle Components
                                </span>
                                <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                                    <input
                                        type="checkbox"
                                        checked={includeSettings}
                                        onChange={(e) => setIncludeSettings(e.target.checked)}
                                        className="rounded border-gray-300 text-blue-600 focus:ring-0"
                                    />
                                    <span>Editor Settings, Themes, Keybindings & Snippets</span>
                                </label>
                                <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                                    <input
                                        type="checkbox"
                                        checked={includeProjects}
                                        onChange={(e) => setIncludeProjects(e.target.checked)}
                                        className="rounded border-gray-300 text-blue-600 focus:ring-0"
                                    />
                                    <span>Workspaces, Running Projects & Pinned Sequences</span>
                                </label>
                                <label className="flex items-center gap-2.5 cursor-pointer text-xs">
                                    <input
                                        type="checkbox"
                                        checked={includePrompts}
                                        onChange={(e) => setIncludePrompts(e.target.checked)}
                                        className="rounded border-gray-300 text-blue-600 focus:ring-0"
                                    />
                                    <span>Active Prompts (FIFO Queued) & Suffix Templates</span>
                                </label>
                            </div>

                            {/* Checksum Badge */}
                            {exportedChecksum && (
                                <div className="p-3 bg-emerald-50 dark:bg-emerald-950/40 border border-emerald-200 dark:border-emerald-800 rounded-xl flex items-center gap-2 text-emerald-800 dark:text-emerald-300 text-[11px]">
                                    <CheckCircle2 className="w-4 h-4 text-emerald-600 shrink-0" />
                                    <span className="font-mono truncate">SHA-256: {exportedChecksum}</span>
                                </div>
                            )}
                        </div>
                    ) : (
                        <div className="space-y-4">
                            {/* File Upload Zone */}
                            <input
                                ref={fileInputRef}
                                type="file"
                                accept=".agmide,.json"
                                className="hidden"
                                onChange={handleFileSelected}
                            />
                            <div
                                onClick={() => fileInputRef.current?.click()}
                                className="border-2 border-dashed border-gray-300 dark:border-[#40285e] hover:border-blue-500 dark:hover:border-cyan-400 p-6 rounded-xl text-center cursor-pointer transition-colors bg-slate-50/50 dark:bg-[#1a122c]/50"
                            >
                                <FileUp className="w-8 h-8 text-blue-500 dark:text-cyan-400 mx-auto mb-2" />
                                <div className="text-xs font-semibold text-gray-800 dark:text-gray-200">
                                    {fileName ? fileName : 'Click to select or drag & drop .agmide bundle'}
                                </div>
                                <p className="text-[11px] text-gray-500 dark:text-gray-400 mt-1">
                                    Compatible with AGM Bundle v1.0 and standard IDE JSON configurations
                                </p>
                            </div>

                            {/* Bundle Inspection Preview */}
                            {bundlePreview && (
                                <div className="p-3.5 bg-slate-100/80 dark:bg-[#1b142d] rounded-xl border border-gray-200 dark:border-[#40285e] space-y-2 text-[11px]">
                                    <div className="flex items-center justify-between font-semibold text-gray-800 dark:text-gray-200">
                                        <span className="flex items-center gap-1.5">
                                            <Cpu className="w-3.5 h-3.5 text-purple-400" />
                                            Source: {bundlePreview.source_environment?.instance_name} ({bundlePreview.source_environment?.platform} {bundlePreview.source_environment?.arch})
                                        </span>
                                        <span className="text-gray-500">v{bundlePreview.agm_bundle_version}</span>
                                    </div>
                                    <div className="grid grid-cols-2 gap-2 text-gray-600 dark:text-gray-400 pt-1 border-t border-gray-200 dark:border-[#40285e]/60">
                                        <div>Workspaces: {bundlePreview.metadata?.item_counts?.workspaces ?? 0}</div>
                                        <div>Pinned Projects: {bundlePreview.metadata?.item_counts?.pinned_projects ?? 0}</div>
                                        <div>Active Prompts: {bundlePreview.metadata?.item_counts?.active_prompts ?? 0}</div>
                                        <div>Settings Keys: {bundlePreview.metadata?.item_counts?.settings_keys ?? 0}</div>
                                    </div>
                                </div>
                            )}

                            {/* Destination Option */}
                            <div className="space-y-2.5">
                                <span className="text-[11px] font-bold text-gray-600 dark:text-gray-400 uppercase tracking-wider block">
                                    Destination Action
                                </span>
                                <div className="flex items-center gap-4">
                                    <label className="flex items-center gap-2 cursor-pointer text-xs font-medium">
                                        <input
                                            type="radio"
                                            name="importMode"
                                            checked={importMode === 'new'}
                                            onChange={() => setImportMode('new')}
                                            className="text-blue-600 focus:ring-0"
                                        />
                                        <span>Create New Profile</span>
                                    </label>
                                    <label className="flex items-center gap-2 cursor-pointer text-xs font-medium">
                                        <input
                                            type="radio"
                                            name="importMode"
                                            checked={importMode === 'merge'}
                                            onChange={() => setImportMode('merge')}
                                            className="text-blue-600 focus:ring-0"
                                        />
                                        <span>Merge into Existing Profile</span>
                                    </label>
                                </div>

                                {importMode === 'new' ? (
                                    <input
                                        type="text"
                                        placeholder="Enter new profile name..."
                                        value={newInstanceName}
                                        onChange={(e) => setNewInstanceName(e.target.value)}
                                        className="w-full px-3 py-2 rounded-lg bg-white dark:bg-[#1f1633] border border-gray-300 dark:border-[#40285e] text-xs font-medium text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                    />
                                ) : (
                                    <select
                                        value={targetMergeId}
                                        onChange={(e) => setTargetMergeId(e.target.value)}
                                        className="w-full px-3 py-2 rounded-lg bg-white dark:bg-[#1f1633] border border-gray-300 dark:border-[#40285e] text-xs font-medium text-gray-800 dark:text-gray-200 focus:outline-none focus:ring-1 focus:ring-blue-500"
                                    >
                                        {instances.map((inst) => (
                                            <option key={inst.id} value={inst.id}>
                                                {inst.name} ({inst.id})
                                            </option>
                                        ))}
                                    </select>
                                )}
                            </div>
                        </div>
                    )}
                </div>

                {/* Modal Footer */}
                <div className="flex items-center justify-end gap-2.5 px-5 py-3.5 border-t border-gray-100 dark:border-[#40285e] bg-gray-50/70 dark:bg-[#091e30]">
                    <button
                        type="button"
                        onClick={onClose}
                        className="px-4 py-1.5 rounded-lg border border-gray-300 dark:border-[#40285e] hover:bg-gray-100 dark:hover:bg-[#1f1633] text-gray-700 dark:text-gray-300 font-medium transition-colors cursor-pointer"
                    >
                        Cancel
                    </button>
                    {activeTab === 'export' ? (
                        <button
                            type="button"
                            onClick={handleExecuteExport}
                            disabled={isExporting}
                            className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold transition-all shadow-xs cursor-pointer disabled:opacity-50"
                        >
                            {isExporting ? (
                                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <FileDown className="w-3.5 h-3.5" />
                            )}
                            <span>Export & Download (.agmide)</span>
                        </button>
                    ) : (
                        <button
                            type="button"
                            onClick={handleExecuteImport}
                            disabled={isImporting || !rawBundleContent}
                            className="flex items-center gap-1.5 px-4 py-1.5 rounded-lg bg-blue-600 hover:bg-blue-500 text-white font-semibold transition-all shadow-xs cursor-pointer disabled:opacity-50"
                        >
                            {isImporting ? (
                                <RotateCw className="w-3.5 h-3.5 animate-spin" />
                            ) : (
                                <Check className="w-3.5 h-3.5" />
                            )}
                            <span>Execute Import</span>
                        </button>
                    )}
                </div>
            </div>
        </div>
    );
}
