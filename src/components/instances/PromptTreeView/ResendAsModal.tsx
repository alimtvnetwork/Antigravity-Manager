import { useState, useEffect } from 'react';
import {
    X,
    Plus,
    Trash2,
    Edit3,
    Check,
    Bookmark,
    Eye,
    Send,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { useErrorStore } from '../../../stores/error-store';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../../utils/env';
import {
    type SuffixTemplate,
    BUILTIN_SUFFIX_TEMPLATES,
    getCustomSuffixTemplates,
} from './verificationTemplates';

export interface ResendAsPreset {
    id: string;
    name: string;
    title_pattern: string;
    target_mode: 'new' | 'current';
    suffix_id?: string | null;
    is_default?: boolean;
    created_at?: string;
    updated_at?: string;
}

export interface ResendAsModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSelectPreset?: (preset: ResendAsPreset) => void;
    activePresetId?: string;
}

export const BUILTIN_RESEND_AS_PRESETS: ReadonlyArray<ResendAsPreset> = [
    {
        id: 'verify',
        name: 'Verify Task',
        title_pattern: '[Verify] {title}',
        target_mode: 'new',
        suffix_id: 'is-it-done',
        is_default: true,
    },
    {
        id: 'fix',
        name: 'Fix Defects',
        title_pattern: '[Fix] {title}',
        target_mode: 'new',
        suffix_id: 'edge-cases-and-tests',
        is_default: false,
    },
    {
        id: 'rerun',
        name: 'Rerun Execution',
        title_pattern: '[Rerun] {title}',
        target_mode: 'current',
        suffix_id: 'rerun-nudge',
        is_default: false,
    },
    {
        id: 'audit',
        name: 'Code Audit',
        title_pattern: '[Audit] {title}',
        target_mode: 'new',
        suffix_id: 'security-audit',
        is_default: false,
    },
    {
        id: 'followup',
        name: 'Follow-up Query',
        title_pattern: '[Follow-up] {title}',
        target_mode: 'new',
        suffix_id: null,
        is_default: false,
    },
];

const STORAGE_KEY_PRESETS = 'agm_prompt_resend_as_presets';
const STORAGE_KEY_DEFAULT = 'agm_prompt_default_resend_as_id';

export function getStoredPresets(): ResendAsPreset[] {
    try {
        if (typeof window === 'undefined' || !window.localStorage) {
            return [];
        }
        const raw = window.localStorage.getItem(STORAGE_KEY_PRESETS);
        if (!raw) return [];
        const parsed = JSON.parse(raw);
        return Array.isArray(parsed) ? parsed : [];
    } catch {
        return [];
    }
}

export function saveStoredPreset(preset: ResendAsPreset): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        const current = getStoredPresets();
        const filtered = current.filter((p) => p.id !== preset.id);
        filtered.push(preset);
        window.localStorage.setItem(STORAGE_KEY_PRESETS, JSON.stringify(filtered));
    } catch {
        // Fallback ignored
    }
}

export function deleteStoredPreset(id: string): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        const current = getStoredPresets();
        const filtered = current.filter((p) => p.id !== id);
        window.localStorage.setItem(STORAGE_KEY_PRESETS, JSON.stringify(filtered));
    } catch {
        // Fallback ignored
    }
}

export function getStoredDefaultPresetId(): string {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return 'verify';
        return window.localStorage.getItem(STORAGE_KEY_DEFAULT) || 'verify';
    } catch {
        return 'verify';
    }
}

export function setStoredDefaultPresetId(id: string): void {
    try {
        if (typeof window === 'undefined' || !window.localStorage) return;
        window.localStorage.setItem(STORAGE_KEY_DEFAULT, id);
    } catch {
        // Fallback ignored
    }
}

export function interpolateTitlePattern(
    pattern: string,
    currentTitle: string,
    dateStr?: string,
    seq?: number
): string {
    const baseTitle = (currentTitle || '').trim();
    const now = new Date();
    const effectiveDate =
        dateStr ||
        `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;
    const hours = String(now.getHours()).padStart(2, '0');
    const minutes = String(now.getMinutes()).padStart(2, '0');
    const effectiveTime = `${hours}:${minutes}`;
    const effectiveSeq = seq !== undefined ? String(seq).padStart(3, '0') : '001';

    let res = pattern
        .replace(/{title}/g, baseTitle)
        .replace(/{date}/g, effectiveDate)
        .replace(/{time}/g, effectiveTime)
        .replace(/{seq}/g, effectiveSeq);

    if (!pattern.includes('{title}') && baseTitle) {
        res = `${res} - ${baseTitle}`;
    }
    return res.trim();
}

export function ResendAsModal({
    isOpen,
    onClose,
    onSelectPreset,
    activePresetId = 'verify',
}: ResendAsModalProps) {
    const [customPresets, setCustomPresets] = useState<ResendAsPreset[]>([]);
    const [selectedPreset, setSelectedPreset] = useState<ResendAsPreset | null>(null);
    const [availableSuffixTemplates, setAvailableSuffixTemplates] = useState<SuffixTemplate[]>([]);
    const [defaultPresetId, setDefaultPresetId] = useState<string>('verify');
    const [searchFilter, setSearchFilter] = useState('');
    const [isEditing, setIsEditing] = useState(false);
    const [isCreating, setIsCreating] = useState(false);

    // Form fields
    const [formId, setFormId] = useState('');
    const [formName, setFormName] = useState('');
    const [formPattern, setFormPattern] = useState('');
    const [formTargetMode, setFormTargetMode] = useState<'new' | 'current'>('new');
    const [formSuffixId, setFormSuffixId] = useState<string>('none');
    const [formError, setFormError] = useState<string | null>(null);

    // Live preview test input
    const [previewSampleTitle, setPreviewSampleTitle] = useState('Implement OAuth Token Auto-Refresh');

    const loadPresets = async () => {
        try {
            let list: ResendAsPreset[] = [];
            let defId = 'verify';

            if (isTauri()) {
                try {
                    const dbPresets = await invoke<ResendAsPreset[]>('list_prompt_resend_as_templates');
                    if (Array.isArray(dbPresets) && dbPresets.length > 0) {
                        const customOnly = dbPresets.filter(
                            (p) => !BUILTIN_RESEND_AS_PRESETS.some((b) => b.id === p.id)
                        );
                        list = customOnly;
                        const def = dbPresets.find((p) => p.is_default);
                        if (def) defId = def.id;
                    }
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'ResendAsModal.loadPresets.ipc',
                        triggerAction: 'list_prompt_resend_as_templates',
                    });
                }
            }

            if (list.length === 0) {
                list = getStoredPresets();
                defId = getStoredDefaultPresetId();
            }

            setCustomPresets(list);
            setDefaultPresetId(defId);

            const all = [...BUILTIN_RESEND_AS_PRESETS, ...list];
            const found = all.find((p) => p.id === activePresetId) || all[0] || null;
            setSelectedPreset(found);
            setIsEditing(false);
            setIsCreating(false);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ResendAsModal.loadPresets',
                triggerAction: 'load_presets',
            });
        }
    };

    const loadSuffixTemplates = async () => {
        try {
            let templates: SuffixTemplate[] = [];
            if (isTauri()) {
                try {
                    const dbT = await invoke<SuffixTemplate[]>('list_prompt_suffix_templates');
                    if (Array.isArray(dbT) && dbT.length > 0) {
                        templates = dbT;
                    }
                } catch {
                    // Fall back to local
                }
            }
            if (templates.length === 0) {
                templates = [...BUILTIN_SUFFIX_TEMPLATES, ...getCustomSuffixTemplates()];
            }
            setAvailableSuffixTemplates(templates);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ResendAsModal.loadSuffixTemplates',
                triggerAction: 'load_suffix_templates',
            });
        }
    };

    useEffect(() => {
        if (!isOpen) return;
        loadPresets();
        loadSuffixTemplates();
    }, [isOpen, activePresetId]);

    // Handle ESC key
    useEffect(() => {
        if (!isOpen) return;
        const handleKeyDown = (e: KeyboardEvent) => {
            if (e.key === 'Escape') {
                if (isEditing || isCreating) {
                    setIsEditing(false);
                    setIsCreating(false);
                } else {
                    onClose();
                }
            }
        };
        window.addEventListener('keydown', handleKeyDown);
        return () => window.removeEventListener('keydown', handleKeyDown);
    }, [isOpen, isEditing, isCreating, onClose]);

    if (!isOpen) return null;

    const allPresets = [...BUILTIN_RESEND_AS_PRESETS, ...customPresets];
    const filteredPresets = allPresets.filter((p) => {
        if (!searchFilter.trim()) return true;
        const q = searchFilter.toLowerCase();
        return (
            p.name.toLowerCase().includes(q) ||
            p.id.toLowerCase().includes(q) ||
            p.title_pattern.toLowerCase().includes(q)
        );
    });

    const isBuiltin = selectedPreset
        ? BUILTIN_RESEND_AS_PRESETS.some((b) => b.id === selectedPreset.id)
        : false;

    const startCreateNew = () => {
        setIsCreating(true);
        setIsEditing(true);
        setFormId('');
        setFormName('');
        setFormPattern('[{date}] {title}');
        setFormTargetMode('new');
        setFormSuffixId('none');
        setFormError(null);
    };

    const startEdit = (p: ResendAsPreset) => {
        setIsCreating(false);
        setIsEditing(true);
        setFormId(p.id);
        setFormName(p.name);
        setFormPattern(p.title_pattern);
        setFormTargetMode(p.target_mode);
        setFormSuffixId(p.suffix_id || 'none');
        setFormError(null);
    };

    const insertToken = (token: string) => {
        setFormPattern((prev) => `${prev} ${token}`.trim());
    };

    const handleSave = async () => {
        const cleanId = formId.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '-');
        if (!cleanId) {
            setFormError('Preset ID is required (alphanumeric, dashes, underscores)');
            return;
        }
        if (!formName.trim()) {
            setFormError('Preset Name is required');
            return;
        }
        if (!formPattern.trim()) {
            setFormError('Title Pattern cannot be empty');
            return;
        }

        if (isCreating && BUILTIN_RESEND_AS_PRESETS.some((b) => b.id === cleanId)) {
            setFormError(`'${cleanId}' is a protected built-in preset ID. Choose another ID.`);
            return;
        }

        const suffixVal = formSuffixId === 'none' ? null : formSuffixId;
        const presetObj: ResendAsPreset = {
            id: cleanId,
            name: formName.trim(),
            title_pattern: formPattern.trim(),
            target_mode: formTargetMode,
            suffix_id: suffixVal,
            is_default: false,
        };

        if (isTauri()) {
            try {
                await invoke('save_prompt_resend_as_template', {
                    input: {
                        id: cleanId,
                        name: formName.trim(),
                        title_pattern: formPattern.trim(),
                        target_mode: formTargetMode,
                        suffix_id: suffixVal,
                        is_default: false,
                    },
                });
            } catch (ipcErr) {
                useErrorStore.getState().trackWarning(ipcErr, {
                    source: 'ResendAsModal.handleSave.ipc',
                    triggerAction: 'save_prompt_resend_as_template',
                });
            }
        }

        saveStoredPreset(presetObj);
        await loadPresets();
        setSelectedPreset(presetObj);
        setIsEditing(false);
        setIsCreating(false);
        setFormError(null);
    };

    const handleDelete = async (id: string) => {
        if (!confirm(`Are you sure you want to delete preset '${id}'?`)) return;
        try {
            if (isTauri()) {
                try {
                    await invoke('delete_prompt_resend_as_template', { id });
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'ResendAsModal.handleDelete.ipc',
                        triggerAction: 'delete_prompt_resend_as_template',
                    });
                }
            }
            deleteStoredPreset(id);
            await loadPresets();
            if (selectedPreset?.id === id) {
                setSelectedPreset(BUILTIN_RESEND_AS_PRESETS[0]);
            }
            setIsEditing(false);
            setIsCreating(false);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ResendAsModal.handleDelete',
                triggerAction: 'delete_preset',
            });
        }
    };

    const handleSetDefault = async (id: string) => {
        try {
            if (isTauri()) {
                try {
                    await invoke('set_default_prompt_resend_as_template', { id });
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'ResendAsModal.handleSetDefault.ipc',
                        triggerAction: 'set_default_prompt_resend_as_template',
                    });
                }
            }
            setStoredDefaultPresetId(id);
            setDefaultPresetId(id);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'ResendAsModal.handleSetDefault',
                triggerAction: 'set_default_preset',
            });
        }
    };

    const currentLivePreview = isEditing
        ? interpolateTitlePattern(formPattern, previewSampleTitle)
        : selectedPreset
        ? interpolateTitlePattern(selectedPreset.title_pattern, previewSampleTitle)
        : '';

    return (
        <div className="fixed inset-0 z-[330] flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
            <div className="flex h-[88vh] w-full max-w-5xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl">
                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#40285e] px-6 py-3.5 bg-slate-50 dark:bg-[#100a1c] shrink-0">
                    <div className="flex items-center gap-2.5">
                        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-purple-500/15 text-purple-600 dark:text-purple-400 border border-purple-500/30">
                            <Send className="h-5 w-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                                <span>&quot;Resend As&quot; Title Presets</span>
                                <span className="text-[10px] font-mono px-2 py-0.5 rounded-md bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20">
                                    {allPresets.length} Presets
                                </span>
                            </h2>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                Configure conversation title patterns, execution targets, and bound verification suffixes
                            </p>
                        </div>
                    </div>

                    <div className="flex items-center gap-2">
                        <button
                            type="button"
                            onClick={startCreateNew}
                            className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-purple-600 hover:bg-purple-700 text-white text-xs font-semibold shadow-xs transition-colors cursor-pointer"
                        >
                            <Plus className="w-3.5 h-3.5" />
                            <span>New Preset</span>
                        </button>
                        <button
                            type="button"
                            onClick={onClose}
                            className="flex items-center justify-center h-8 w-8 rounded-lg text-slate-400 hover:text-slate-600 dark:hover:text-slate-200 hover:bg-slate-100 dark:hover:bg-[#40285e] transition-colors cursor-pointer"
                            title="Close modal (Escape)"
                        >
                            <X className="w-4 h-4" />
                        </button>
                    </div>
                </div>

                {/* Content Area */}
                <div className="flex flex-1 min-h-0 divide-x divide-slate-200 dark:divide-[#40285e]">
                    {/* Left Pane: Preset Master List */}
                    <div className="w-80 flex flex-col bg-slate-50/50 dark:bg-[#120b1e]/60 shrink-0">
                        {/* Search Bar */}
                        <div className="p-3 border-b border-slate-200 dark:border-[#40285e]/80">
                            <input
                                type="text"
                                value={searchFilter}
                                onChange={(e) => setSearchFilter(e.target.value)}
                                placeholder="Search title presets..."
                                className="w-full px-3 py-1.5 text-xs rounded-lg bg-white dark:bg-[#1b122c] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-purple-500"
                            />
                        </div>

                        {/* List */}
                        <div className="flex-1 overflow-y-auto p-2 space-y-1">
                            {filteredPresets.map((p) => {
                                const active = selectedPreset?.id === p.id;
                                const isDef = defaultPresetId === p.id;
                                const builtin = BUILTIN_RESEND_AS_PRESETS.some((b) => b.id === p.id);

                                return (
                                    <div
                                        key={p.id}
                                        onClick={() => {
                                            setSelectedPreset(p);
                                            setIsEditing(false);
                                            setIsCreating(false);
                                        }}
                                        className={cn(
                                            'group flex items-start justify-between p-2.5 rounded-xl cursor-pointer transition-all border text-left',
                                            active
                                                ? 'bg-purple-500/10 dark:bg-purple-500/15 border-purple-500/40 text-slate-900 dark:text-white shadow-2xs'
                                                : 'border-transparent hover:bg-slate-100 dark:hover:bg-[#1d1430] text-slate-700 dark:text-slate-300'
                                        )}
                                    >
                                        <div className="flex items-start gap-2.5 min-w-0">
                                            <div className="mt-0.5 shrink-0 text-slate-400">
                                                <Bookmark className="w-3.5 h-3.5" />
                                            </div>
                                            <div className="min-w-0">
                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    <span className="text-xs font-semibold truncate">{p.name}</span>
                                                    {isDef && (
                                                        <span className="px-1.5 py-0.2 rounded text-[9px] font-mono font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                                                            DEFAULT
                                                        </span>
                                                    )}
                                                    {builtin && (
                                                        <span className="text-[9px] text-slate-400 dark:text-slate-500 font-mono">
                                                            built-in
                                                        </span>
                                                    )}
                                                </div>
                                                <div className="text-[10px] text-purple-600 dark:text-purple-400 font-mono truncate mt-0.5">
                                                    {p.title_pattern}
                                                </div>
                                                <div className="text-[9px] text-slate-400 dark:text-slate-500 flex items-center gap-2 mt-1">
                                                    <span>Target: {p.target_mode}</span>
                                                    {p.suffix_id && <span>Suffix: {p.suffix_id}</span>}
                                                </div>
                                            </div>
                                        </div>

                                        <div className="shrink-0 ml-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <span className="text-[9px] font-mono text-slate-400">#{p.id}</span>
                                        </div>
                                    </div>
                                );
                            })}
                        </div>
                    </div>

                    {/* Right Pane: Detail & Editor */}
                    <div className="flex-1 flex flex-col min-w-0 bg-white dark:bg-[#171025]">
                        {isEditing ? (
                            /* Editor Mode */
                            <div className="flex-1 flex flex-col min-h-0 p-6 space-y-4 overflow-y-auto">
                                <div className="flex items-center justify-between pb-2 border-b border-slate-200 dark:border-[#40285e]/80">
                                    <h3 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                                        <Edit3 className="w-4 h-4 text-purple-500" />
                                        <span>
                                            {isCreating ? 'Create "Resend As" Preset' : `Edit Preset '${formId}'`}
                                        </span>
                                    </h3>
                                    <div className="flex items-center gap-2">
                                        <button
                                            type="button"
                                            onClick={() => {
                                                setIsEditing(false);
                                                setIsCreating(false);
                                            }}
                                            className="px-3 py-1.5 text-xs text-slate-600 dark:text-slate-400 hover:bg-slate-100 dark:hover:bg-[#40285e] rounded-lg transition-colors cursor-pointer"
                                        >
                                            Cancel
                                        </button>
                                        <button
                                            type="button"
                                            onClick={handleSave}
                                            className="px-3.5 py-1.5 text-xs font-semibold text-white bg-purple-600 hover:bg-purple-700 rounded-lg shadow-xs transition-colors cursor-pointer"
                                        >
                                            Save Preset
                                        </button>
                                    </div>
                                </div>

                                {formError && (
                                    <div className="p-2.5 text-xs text-rose-700 dark:text-rose-300 bg-rose-500/10 border border-rose-500/30 rounded-lg">
                                        {formError}
                                    </div>
                                )}

                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                            Preset ID (slug):
                                        </label>
                                        <input
                                            type="text"
                                            disabled={!isCreating}
                                            value={formId}
                                            onChange={(e) => setFormId(e.target.value)}
                                            placeholder="e.g. audit-security"
                                            className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 disabled:opacity-60 focus:outline-none focus:ring-1 focus:ring-purple-500 font-mono"
                                        />
                                    </div>

                                    <div>
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                            Display Name:
                                        </label>
                                        <input
                                            type="text"
                                            value={formName}
                                            onChange={(e) => setFormName(e.target.value)}
                                            placeholder="e.g. Security Audit Run"
                                            className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500"
                                        />
                                    </div>
                                </div>

                                <div>
                                    <div className="flex items-center justify-between mb-1">
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400">
                                            Title Pattern:
                                        </label>
                                        <div className="flex items-center gap-1.5">
                                            <span className="text-[10px] text-slate-400">Insert token:</span>
                                            <button
                                                type="button"
                                                onClick={() => insertToken('{title}')}
                                                className="px-1.5 py-0.5 rounded bg-purple-500/10 hover:bg-purple-500/20 text-purple-600 dark:text-purple-300 font-mono text-[10px] border border-purple-500/20 cursor-pointer"
                                            >
                                                &#123;title&#125;
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => insertToken('{date}')}
                                                className="px-1.5 py-0.5 rounded bg-blue-500/10 hover:bg-blue-500/20 text-blue-600 dark:text-blue-300 font-mono text-[10px] border border-blue-500/20 cursor-pointer"
                                            >
                                                &#123;date&#125;
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => insertToken('{time}')}
                                                className="px-1.5 py-0.5 rounded bg-amber-500/10 hover:bg-amber-500/20 text-amber-600 dark:text-amber-300 font-mono text-[10px] border border-amber-500/20 cursor-pointer"
                                            >
                                                &#123;time&#125;
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => insertToken('{seq}')}
                                                className="px-1.5 py-0.5 rounded bg-emerald-500/10 hover:bg-emerald-500/20 text-emerald-600 dark:text-emerald-300 font-mono text-[10px] border border-emerald-500/20 cursor-pointer"
                                            >
                                                &#123;seq&#125;
                                            </button>
                                        </div>
                                    </div>
                                    <input
                                        type="text"
                                        value={formPattern}
                                        onChange={(e) => setFormPattern(e.target.value)}
                                        placeholder="e.g. [Audit:{date}] {title}"
                                        className="w-full px-3 py-2 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500 font-mono"
                                    />
                                </div>

                                {/* Live Preview Box */}
                                <div className="rounded-xl border border-purple-500/30 bg-purple-500/5 p-4 space-y-2">
                                    <div className="flex items-center justify-between text-[11px] font-mono text-purple-600 dark:text-purple-300">
                                        <span className="flex items-center gap-1.5">
                                            <Eye className="w-3.5 h-3.5" />
                                            <span>Live Title Preview</span>
                                        </span>
                                        <span className="text-[10px] opacity-75">Sample Title</span>
                                    </div>
                                    <div className="text-sm font-semibold text-slate-900 dark:text-white font-mono break-all bg-white dark:bg-[#120a1f] p-3 rounded-lg border border-purple-500/20">
                                        {currentLivePreview}
                                    </div>
                                </div>

                                <div className="grid grid-cols-2 gap-4">
                                    <div>
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                            Target Conversation Mode:
                                        </label>
                                        <div className="flex rounded-lg bg-slate-100 dark:bg-[#120a1f] p-1 border border-slate-200 dark:border-[#40285e]">
                                            <button
                                                type="button"
                                                onClick={() => setFormTargetMode('new')}
                                                className={cn(
                                                    'flex-1 py-1.5 text-xs rounded-md font-medium transition-colors cursor-pointer',
                                                    formTargetMode === 'new'
                                                        ? 'bg-white dark:bg-[#2c1a4d] text-purple-600 dark:text-purple-300 font-semibold shadow-2xs'
                                                        : 'text-slate-600 dark:text-slate-400'
                                                )}
                                            >
                                                New Thread
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => setFormTargetMode('current')}
                                                className={cn(
                                                    'flex-1 py-1.5 text-xs rounded-md font-medium transition-colors cursor-pointer',
                                                    formTargetMode === 'current'
                                                        ? 'bg-white dark:bg-[#2c1a4d] text-purple-600 dark:text-purple-300 font-semibold shadow-2xs'
                                                        : 'text-slate-600 dark:text-slate-400'
                                                )}
                                            >
                                                Current Thread
                                            </button>
                                        </div>
                                    </div>

                                    <div>
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                            Bound Suffix Template:
                                        </label>
                                        <select
                                            value={formSuffixId}
                                            onChange={(e) => setFormSuffixId(e.target.value)}
                                            className="w-full px-3 py-2 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500 cursor-pointer"
                                        >
                                            <option value="none">None (No bound template)</option>
                                            {availableSuffixTemplates.map((t) => (
                                                <option key={t.id} value={t.id}>
                                                    {t.name} ({t.category})
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                </div>
                            </div>
                        ) : selectedPreset ? (
                            /* Detail / Preview Mode */
                            <div className="flex-1 flex flex-col min-h-0">
                                <div className="p-6 border-b border-slate-200 dark:border-[#40285e]/80 space-y-3 bg-slate-50/40 dark:bg-[#100a1c]/40 shrink-0">
                                    <div className="flex items-start justify-between gap-4">
                                        <div>
                                            <div className="flex items-center gap-2">
                                                <h3 className="text-base font-bold text-slate-900 dark:text-white">
                                                    {selectedPreset.name}
                                                </h3>
                                                <span className="px-2 py-0.5 rounded-md text-[10px] font-mono font-bold bg-blue-500/10 text-blue-600 dark:text-blue-300 border border-blue-500/20">
                                                    target: {selectedPreset.target_mode}
                                                </span>
                                                {defaultPresetId === selectedPreset.id && (
                                                    <span className="px-2 py-0.5 rounded-md text-[10px] font-mono font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                                                        DEFAULT
                                                    </span>
                                                )}
                                            </div>
                                            <div className="text-xs font-mono text-purple-600 dark:text-purple-400 mt-1">
                                                Pattern: {selectedPreset.title_pattern}
                                            </div>
                                        </div>

                                        {/* Actions */}
                                        <div className="flex items-center gap-2 shrink-0">
                                            {onSelectPreset && (
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        onSelectPreset(selectedPreset);
                                                        onClose();
                                                    }}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold shadow-xs transition-colors cursor-pointer"
                                                >
                                                    <Check className="w-3.5 h-3.5" />
                                                    <span>Select Preset</span>
                                                </button>
                                            )}

                                            {defaultPresetId !== selectedPreset.id && (
                                                <button
                                                    type="button"
                                                    onClick={() => handleSetDefault(selectedPreset.id)}
                                                    className="px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] border border-slate-200 dark:border-[#40285e] rounded-lg transition-colors cursor-pointer"
                                                >
                                                    Set Default
                                                </button>
                                            )}

                                            {!isBuiltin && (
                                                <>
                                                    <button
                                                        type="button"
                                                        onClick={() => startEdit(selectedPreset)}
                                                        className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] border border-slate-200 dark:border-[#40285e] rounded-lg transition-colors cursor-pointer"
                                                    >
                                                        <Edit3 className="w-3 h-3" />
                                                        <span>Edit</span>
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onClick={() => handleDelete(selectedPreset.id)}
                                                        className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 border border-rose-200 dark:border-rose-500/30 rounded-lg transition-colors cursor-pointer"
                                                    >
                                                        <Trash2 className="w-3 h-3" />
                                                        <span>Delete</span>
                                                    </button>
                                                </>
                                            )}
                                        </div>
                                    </div>
                                </div>

                                {/* Body / Live Preview Inspector */}
                                <div className="flex-1 overflow-y-auto p-6 space-y-5">
                                    <div className="space-y-2">
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block">
                                            Test against sample conversation title:
                                        </label>
                                        <input
                                            type="text"
                                            value={previewSampleTitle}
                                            onChange={(e) => setPreviewSampleTitle(e.target.value)}
                                            className="w-full px-3 py-2 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500"
                                        />
                                    </div>

                                    <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/50 dark:bg-[#100a1c]/50 p-5 space-y-4">
                                        <div className="text-xs font-bold text-slate-700 dark:text-slate-300">
                                            Interpolated Output:
                                        </div>
                                        <div className="p-4 rounded-xl bg-white dark:bg-[#171025] border border-purple-500/30 font-mono text-sm text-purple-700 dark:text-purple-300 font-semibold shadow-xs break-all">
                                            {currentLivePreview}
                                        </div>
                                    </div>

                                    <div className="grid grid-cols-2 gap-4 text-xs">
                                        <div className="p-4 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/30 dark:bg-[#120a1f]/30 space-y-1">
                                            <div className="font-semibold text-slate-800 dark:text-slate-200">
                                                Target Thread Mode:
                                            </div>
                                            <div className="text-slate-500 dark:text-slate-400 capitalize">
                                                {selectedPreset.target_mode === 'new'
                                                    ? 'Create a new conversation thread'
                                                    : 'Continue inside current conversation'}
                                            </div>
                                        </div>

                                        <div className="p-4 rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/30 dark:bg-[#120a1f]/30 space-y-1">
                                            <div className="font-semibold text-slate-800 dark:text-slate-200">
                                                Bound Verification Suffix:
                                            </div>
                                            <div className="text-slate-500 dark:text-slate-400">
                                                {selectedPreset.suffix_id || 'None (no suffix attached)'}
                                            </div>
                                        </div>
                                    </div>
                                </div>
                            </div>
                        ) : (
                            <div className="flex-1 flex items-center justify-center text-slate-400 text-xs">
                                Select a preset from the left pane
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
