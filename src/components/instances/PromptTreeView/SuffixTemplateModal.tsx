import { useState, useEffect } from 'react';
import {
    X,
    Plus,
    Trash2,
    Edit3,
    Check,
    Copy,
    Bookmark,
    Lock,
    Eye,
    Sparkles,
    ShieldCheck,
    Rocket,
    AlertTriangle,
    Terminal,
    GitPullRequest,
} from 'lucide-react';
import { cn } from '../../../utils/cn';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';
import { invoke } from '@tauri-apps/api/core';
import { isTauri } from '../../../utils/env';
import {
    type SuffixTemplate,
    BUILTIN_SUFFIX_TEMPLATES,
    getCustomSuffixTemplates,
    saveCustomSuffixTemplate,
    deleteCustomSuffixTemplate,
    getDefaultSuffixTemplateId,
    setDefaultSuffixTemplateId,
} from './verificationTemplates';

export interface SuffixTemplateModalProps {
    isOpen: boolean;
    onClose: () => void;
    onSelectTemplate?: (templateId: string) => void;
    activeTemplateId?: string;
}

const CATEGORY_OPTIONS: Array<{ value: SuffixTemplate['category']; label: string }> = [
    { value: 'custom', label: 'Custom' },
    { value: 'verification', label: 'Verification & Audit' },
    { value: 'release', label: 'Release Engineering' },
    { value: 'quality', label: 'Code Quality' },
    { value: 'robustness', label: 'Robustness & Edge Cases' },
    { value: 'security', label: 'Security & Secrets' },
    { value: 'governance', label: 'Governance & PR Review' },
    { value: 'testing', label: 'Testing & Coverage' },
    { value: 'architecture', label: 'Architecture & DRY' },
];

function getCategoryIcon(cat: SuffixTemplate['category']) {
    switch (cat) {
        case 'verification':
            return <Check className="w-3.5 h-3.5 text-emerald-500" />;
        case 'release':
            return <Rocket className="w-3.5 h-3.5 text-blue-500" />;
        case 'quality':
            return <ShieldCheck className="w-3.5 h-3.5 text-purple-500" />;
        case 'robustness':
            return <AlertTriangle className="w-3.5 h-3.5 text-amber-500" />;
        case 'security':
            return <Lock className="w-3.5 h-3.5 text-rose-500" />;
        case 'governance':
            return <GitPullRequest className="w-3.5 h-3.5 text-indigo-500" />;
        case 'testing':
            return <Terminal className="w-3.5 h-3.5 text-cyan-500" />;
        case 'architecture':
            return <Sparkles className="w-3.5 h-3.5 text-fuchsia-500" />;
        default:
            return <Bookmark className="w-3.5 h-3.5 text-slate-400" />;
    }
}

export function SuffixTemplateModal({
    isOpen,
    onClose,
    onSelectTemplate,
    activeTemplateId = 'is-it-done',
}: SuffixTemplateModalProps) {
    const [customTemplates, setCustomTemplates] = useState<SuffixTemplate[]>([]);
    const [selectedTemplate, setSelectedTemplate] = useState<SuffixTemplate | null>(null);
    const [isEditing, setIsEditing] = useState(false);
    const [isCreating, setIsCreating] = useState(false);
    const [defaultTemplateId, setDefaultId] = useState<string>('is-it-done');
    const [searchFilter, setSearchFilter] = useState('');
    const [activeTab, setActiveTab] = useState<'editor' | 'preview'>('editor');
    const [copiedId, setCopiedId] = useState<string | null>(null);

    // Form fields for editing/creating
    const [formId, setFormId] = useState('');
    const [formName, setFormName] = useState('');
    const [formDescription, setFormDescription] = useState('');
    const [formCategory, setFormCategory] = useState<SuffixTemplate['category']>('custom');
    const [formTemplate, setFormTemplate] = useState('');
    const [formError, setFormError] = useState<string | null>(null);

    // Load templates on modal open
    const loadTemplates = async () => {
        try {
            let list: SuffixTemplate[] = [];
            let defId = 'is-it-done';

            if (isTauri()) {
                try {
                    const dbTemplates = await invoke<SuffixTemplate[]>('list_prompt_suffix_templates');
                    if (Array.isArray(dbTemplates) && dbTemplates.length > 0) {
                        const customOnly = dbTemplates.filter(
                            (t) => !BUILTIN_SUFFIX_TEMPLATES.some((b) => b.id === t.id)
                        );
                        list = customOnly;
                        const def = dbTemplates.find((t) => t.is_default);
                        if (def) defId = def.id;
                    }
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'SuffixTemplateModal.loadTemplates.ipc',
                        triggerAction: 'list_prompt_suffix_templates',
                    });
                }
            }

            if (list.length === 0) {
                list = getCustomSuffixTemplates();
                defId = getDefaultSuffixTemplateId();
            }

            setCustomTemplates(list);
            setDefaultId(defId);

            // Find initially selected template
            const all = [...BUILTIN_SUFFIX_TEMPLATES, ...list];
            const found = all.find((t) => t.id === activeTemplateId) || all[0] || null;
            setSelectedTemplate(found);
            setIsEditing(false);
            setIsCreating(false);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SuffixTemplateModal.loadTemplates',
                triggerAction: 'load_templates',
            });
        }
    };

    useEffect(() => {
        if (!isOpen) return;
        loadTemplates();
    }, [isOpen, activeTemplateId]);

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

    const allTemplates = [...BUILTIN_SUFFIX_TEMPLATES, ...customTemplates];
    const filteredTemplates = allTemplates.filter((t) => {
        if (!searchFilter.trim()) return true;
        const q = searchFilter.toLowerCase();
        return (
            t.name.toLowerCase().includes(q) ||
            t.id.toLowerCase().includes(q) ||
            t.description.toLowerCase().includes(q) ||
            t.category.toLowerCase().includes(q)
        );
    });

    const isBuiltin = selectedTemplate ? BUILTIN_SUFFIX_TEMPLATES.some((b) => b.id === selectedTemplate.id) : false;

    const startCreateNew = () => {
        setIsCreating(true);
        setIsEditing(true);
        setFormId('');
        setFormName('');
        setFormDescription('');
        setFormCategory('custom');
        setFormTemplate('');
        setFormError(null);
        setActiveTab('editor');
    };

    const startClone = (source: SuffixTemplate) => {
        setIsCreating(true);
        setIsEditing(true);
        setFormId(`${source.id}-custom`);
        setFormName(`${source.name} (Custom)`);
        setFormDescription(`Custom variant based on ${source.name}`);
        setFormCategory(source.category);
        setFormTemplate(source.template);
        setFormError(null);
        setActiveTab('editor');
    };

    const startEdit = (t: SuffixTemplate) => {
        setIsCreating(false);
        setIsEditing(true);
        setFormId(t.id);
        setFormName(t.name);
        setFormDescription(t.description);
        setFormCategory(t.category);
        setFormTemplate(t.template);
        setFormError(null);
        setActiveTab('editor');
    };

    const handleSave = async () => {
        const cleanId = formId.trim().toLowerCase().replace(/[^a-z0-9_-]/g, '-');
        if (!cleanId) {
            setFormError('Template ID is required (alphanumeric, dashes, underscores)');
            return;
        }
        if (!formName.trim()) {
            setFormError('Template Name is required');
            return;
        }
        if (!formTemplate.trim()) {
            setFormError('Template Body cannot be empty');
            return;
        }

        // Check if ID collides with built-in
        if (isCreating && BUILTIN_SUFFIX_TEMPLATES.some((b) => b.id === cleanId)) {
            setFormError(`'${cleanId}' is a protected built-in preset ID. Choose another ID.`);
            return;
        }

        try {
            const templateObj: SuffixTemplate = {
                id: cleanId,
                name: formName.trim(),
                description: formDescription.trim(),
                category: formCategory,
                template: formTemplate.trim(),
            };

            if (isTauri()) {
                try {
                    await invoke('save_prompt_suffix_template', {
                        input: {
                            id: cleanId,
                            name: formName.trim(),
                            description: formDescription.trim(),
                            category: formCategory,
                            template: formTemplate.trim(),
                            is_default: false,
                        },
                    });
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'SuffixTemplateModal.handleSave.ipc',
                        triggerAction: 'save_prompt_suffix_template',
                    });
                }
            }

            saveCustomSuffixTemplate(templateObj);
            await loadTemplates();
            setSelectedTemplate(templateObj);
            setIsEditing(false);
            setIsCreating(false);
            setFormError(null);
        } catch (err) {
            useErrorStore.getState().captureException(
                err instanceof Error ? err : new Error(String(err)),
                {
                    source: 'SuffixTemplateModal.handleSave',
                    triggerAction: 'save_template',
                }
            );
        }
    };

    const handleDelete = async (id: string) => {
        if (!confirm(`Are you sure you want to delete custom template '${id}'?`)) return;
        try {
            if (isTauri()) {
                try {
                    await invoke('delete_prompt_suffix_template', { id });
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'SuffixTemplateModal.handleDelete.ipc',
                        triggerAction: 'delete_prompt_suffix_template',
                    });
                }
            }
            deleteCustomSuffixTemplate(id);
            await loadTemplates();
            if (selectedTemplate?.id === id) {
                setSelectedTemplate(BUILTIN_SUFFIX_TEMPLATES[0]);
            }
            setIsEditing(false);
            setIsCreating(false);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SuffixTemplateModal.handleDelete',
                triggerAction: 'delete_template',
            });
        }
    };

    const handleSetDefault = async (id: string) => {
        try {
            if (isTauri()) {
                try {
                    await invoke('set_default_prompt_suffix_template', { id });
                } catch (ipcErr) {
                    useErrorStore.getState().trackWarning(ipcErr, {
                        source: 'SuffixTemplateModal.handleSetDefault.ipc',
                        triggerAction: 'set_default_prompt_suffix_template',
                    });
                }
            }
            setDefaultSuffixTemplateId(id);
            setDefaultId(id);
        } catch (err) {
            useErrorStore.getState().trackWarning(err, {
                source: 'SuffixTemplateModal.handleSetDefault',
                triggerAction: 'set_default_template',
            });
        }
    };

    const handleCopyTemplateText = async (text: string, id: string) => {
        await copyToClipboard(text);
        setCopiedId(id);
        setTimeout(() => setCopiedId(null), 2000);
    };

    return (
        <div className="fixed inset-0 z-[330] flex items-center justify-center bg-black/80 backdrop-blur-md p-4 animate-in fade-in duration-200">
            <div className="flex h-[88vh] w-full max-w-5xl flex-col overflow-hidden rounded-2xl border border-slate-200 dark:border-[#40285e] bg-white dark:bg-[#171025] shadow-2xl">
                {/* Header */}
                <div className="flex items-center justify-between border-b border-slate-200 dark:border-[#40285e] px-6 py-3.5 bg-slate-50 dark:bg-[#100a1c] shrink-0">
                    <div className="flex items-center gap-2.5">
                        <div className="flex h-9 w-9 items-center justify-center rounded-xl bg-purple-500/15 text-purple-600 dark:text-purple-400 border border-purple-500/30">
                            <Sparkles className="h-5 w-5" />
                        </div>
                        <div>
                            <h2 className="text-sm font-bold text-slate-900 dark:text-white flex items-center gap-2">
                                <span>Suffix Template Library</span>
                                <span className="text-[10px] font-mono px-2 py-0.5 rounded-full bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20">
                                    {allTemplates.length} Presets & Templates
                                </span>
                            </h2>
                            <p className="text-[11px] text-slate-500 dark:text-slate-400">
                                Manage built-in audit presets and custom AI verification checklists for prompt injection
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
                            <span>New Template</span>
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
                    {/* Left Pane: Template Master List */}
                    <div className="w-80 flex flex-col bg-slate-50/50 dark:bg-[#120b1e]/60 shrink-0">
                        {/* Search Bar */}
                        <div className="p-3 border-b border-slate-200 dark:border-[#40285e]/80">
                            <input
                                type="text"
                                value={searchFilter}
                                onChange={(e) => setSearchFilter(e.target.value)}
                                placeholder="Search presets & templates..."
                                className="w-full px-3 py-1.5 text-xs rounded-lg bg-white dark:bg-[#1b122c] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 placeholder-slate-400 focus:outline-none focus:ring-1 focus:ring-purple-500"
                            />
                        </div>

                        {/* List */}
                        <div className="flex-1 overflow-y-auto p-2 space-y-1">
                            {filteredTemplates.map((t) => {
                                const active = selectedTemplate?.id === t.id;
                                const isDef = defaultTemplateId === t.id;
                                const builtin = BUILTIN_SUFFIX_TEMPLATES.some((b) => b.id === t.id);

                                return (
                                    <div
                                        key={t.id}
                                        onClick={() => {
                                            setSelectedTemplate(t);
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
                                            <div className="mt-0.5 shrink-0">{getCategoryIcon(t.category)}</div>
                                            <div className="min-w-0">
                                                <div className="flex items-center gap-1.5 flex-wrap">
                                                    <span className="text-xs font-semibold truncate">{t.name}</span>
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
                                                <div className="text-[10px] text-slate-400 dark:text-slate-500 line-clamp-1 mt-0.5">
                                                    {t.description || t.id}
                                                </div>
                                            </div>
                                        </div>

                                        <div className="shrink-0 ml-1 opacity-0 group-hover:opacity-100 transition-opacity">
                                            <span className="text-[9px] font-mono text-slate-400">#{t.id}</span>
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
                                        <span>{isCreating ? 'Create Suffix Template' : `Edit Template '${formId}'`}</span>
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
                                            Save Template
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
                                            Template ID (slug):
                                        </label>
                                        <input
                                            type="text"
                                            disabled={!isCreating}
                                            value={formId}
                                            onChange={(e) => setFormId(e.target.value)}
                                            placeholder="e.g. my-e2e-gate"
                                            className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 disabled:opacity-60 focus:outline-none focus:ring-1 focus:ring-purple-500"
                                        />
                                    </div>

                                    <div>
                                        <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                            Category:
                                        </label>
                                        <select
                                            value={formCategory}
                                            onChange={(e) => setFormCategory(e.target.value as SuffixTemplate['category'])}
                                            className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500 cursor-pointer"
                                        >
                                            {CATEGORY_OPTIONS.map((opt) => (
                                                <option key={opt.value} value={opt.value}>
                                                    {opt.label}
                                                </option>
                                            ))}
                                        </select>
                                    </div>
                                </div>

                                <div>
                                    <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                        Display Name:
                                    </label>
                                    <input
                                        type="text"
                                        value={formName}
                                        onChange={(e) => setFormName(e.target.value)}
                                        placeholder="e.g. E2E Quota Switch Gate"
                                        className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500"
                                    />
                                </div>

                                <div>
                                    <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                        Description:
                                    </label>
                                    <input
                                        type="text"
                                        value={formDescription}
                                        onChange={(e) => setFormDescription(e.target.value)}
                                        placeholder="Brief summary of what this verification checklist checks"
                                        className="w-full px-3 py-1.5 text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500"
                                    />
                                </div>

                                <div className="flex-1 flex flex-col min-h-[220px]">
                                    <label className="text-[11px] font-mono text-slate-500 dark:text-slate-400 block mb-1">
                                        Template Body (Markdown / Checklists):
                                    </label>
                                    <textarea
                                        value={formTemplate}
                                        onChange={(e) => setFormTemplate(e.target.value)}
                                        placeholder="Write markdown instructions or checklist items..."
                                        className="flex-1 w-full p-3 font-mono text-xs rounded-lg bg-slate-50 dark:bg-[#120a1f] border border-slate-200 dark:border-[#40285e] text-slate-800 dark:text-slate-200 focus:outline-none focus:ring-1 focus:ring-purple-500 resize-none leading-relaxed"
                                    />
                                </div>
                            </div>
                        ) : selectedTemplate ? (
                            /* View / Preview Mode */
                            <div className="flex-1 flex flex-col min-h-0">
                                {/* Details Header */}
                                <div className="p-6 border-b border-slate-200 dark:border-[#40285e]/80 space-y-3 bg-slate-50/40 dark:bg-[#100a1c]/40 shrink-0">
                                    <div className="flex items-start justify-between gap-4">
                                        <div>
                                            <div className="flex items-center gap-2">
                                                <h3 className="text-base font-bold text-slate-900 dark:text-white">
                                                    {selectedTemplate.name}
                                                </h3>
                                                <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-purple-500/10 text-purple-600 dark:text-purple-300 border border-purple-500/20">
                                                    {selectedTemplate.category}
                                                </span>
                                                {defaultTemplateId === selectedTemplate.id && (
                                                    <span className="px-2 py-0.5 rounded-full text-[10px] font-mono font-bold bg-emerald-500/15 text-emerald-700 dark:text-emerald-300 border border-emerald-500/30">
                                                        DEFAULT
                                                    </span>
                                                )}
                                            </div>
                                            <p className="text-xs text-slate-500 dark:text-slate-400 mt-1">
                                                {selectedTemplate.description}
                                            </p>
                                        </div>

                                        {/* Actions Capsule */}
                                        <div className="flex items-center gap-2 shrink-0">
                                            {onSelectTemplate && (
                                                <button
                                                    type="button"
                                                    onClick={() => {
                                                        onSelectTemplate(selectedTemplate.id);
                                                        onClose();
                                                    }}
                                                    className="flex items-center gap-1.5 px-3 py-1.5 rounded-lg bg-emerald-600 hover:bg-emerald-700 text-white text-xs font-semibold shadow-xs transition-colors cursor-pointer"
                                                >
                                                    <Check className="w-3.5 h-3.5" />
                                                    <span>Apply to Suffix</span>
                                                </button>
                                            )}

                                            {defaultTemplateId !== selectedTemplate.id && (
                                                <button
                                                    type="button"
                                                    onClick={() => handleSetDefault(selectedTemplate.id)}
                                                    className="px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] border border-slate-200 dark:border-[#40285e] rounded-lg transition-colors cursor-pointer"
                                                    title="Set as global default template"
                                                >
                                                    Set Default
                                                </button>
                                            )}

                                            {isBuiltin ? (
                                                <button
                                                    type="button"
                                                    onClick={() => startClone(selectedTemplate)}
                                                    className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-purple-700 dark:text-purple-300 hover:bg-purple-50 dark:hover:bg-purple-950/40 border border-purple-200 dark:border-purple-500/30 rounded-lg transition-colors cursor-pointer"
                                                    title="Clone built-in preset to custom template"
                                                >
                                                    <Copy className="w-3 h-3" />
                                                    <span>Clone</span>
                                                </button>
                                            ) : (
                                                <>
                                                    <button
                                                        type="button"
                                                        onClick={() => startEdit(selectedTemplate)}
                                                        className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-slate-700 dark:text-slate-300 hover:bg-slate-100 dark:hover:bg-[#40285e] border border-slate-200 dark:border-[#40285e] rounded-lg transition-colors cursor-pointer"
                                                    >
                                                        <Edit3 className="w-3 h-3" />
                                                        <span>Edit</span>
                                                    </button>
                                                    <button
                                                        type="button"
                                                        onClick={() => handleDelete(selectedTemplate.id)}
                                                        className="flex items-center gap-1.5 px-2.5 py-1.5 text-xs text-rose-600 dark:text-rose-400 hover:bg-rose-50 dark:hover:bg-rose-950/30 border border-rose-200 dark:border-rose-500/30 rounded-lg transition-colors cursor-pointer"
                                                    >
                                                        <Trash2 className="w-3 h-3" />
                                                        <span>Delete</span>
                                                    </button>
                                                </>
                                            )}
                                        </div>
                                    </div>

                                    {/* Tabs */}
                                    <div className="flex items-center gap-2 pt-1">
                                        <div className="inline-flex rounded-lg bg-slate-100 dark:bg-[#120a1f] p-0.5 border border-slate-200/80 dark:border-purple-500/20 text-xs">
                                            <button
                                                type="button"
                                                onClick={() => setActiveTab('editor')}
                                                className={cn(
                                                    'px-3 py-1 rounded-md font-medium transition-colors cursor-pointer',
                                                    activeTab === 'editor'
                                                        ? 'bg-white dark:bg-[#2c1a4d] text-purple-600 dark:text-purple-300 shadow-2xs font-semibold'
                                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                                                )}
                                            >
                                                Template Body
                                            </button>
                                            <button
                                                type="button"
                                                onClick={() => setActiveTab('preview')}
                                                className={cn(
                                                    'px-3 py-1 rounded-md font-medium transition-colors cursor-pointer flex items-center gap-1.5',
                                                    activeTab === 'preview'
                                                        ? 'bg-white dark:bg-[#2c1a4d] text-purple-600 dark:text-purple-300 shadow-2xs font-semibold'
                                                        : 'text-slate-600 dark:text-slate-400 hover:text-slate-900 dark:hover:text-slate-200'
                                                )}
                                            >
                                                <Eye className="w-3 h-3" />
                                                <span>Rendered Preview</span>
                                            </button>
                                        </div>

                                        <button
                                            type="button"
                                            onClick={() => handleCopyTemplateText(selectedTemplate.template, selectedTemplate.id)}
                                            className="flex items-center gap-1.5 px-2.5 py-1 text-xs text-slate-500 hover:text-slate-800 dark:hover:text-slate-200 cursor-pointer ml-auto"
                                        >
                                            {copiedId === selectedTemplate.id ? (
                                                <Check className="w-3 h-3 text-emerald-500" />
                                            ) : (
                                                <Copy className="w-3 h-3" />
                                            )}
                                            <span>{copiedId === selectedTemplate.id ? 'Copied' : 'Copy Template Text'}</span>
                                        </button>
                                    </div>
                                </div>

                                {/* Body Container */}
                                <div className="flex-1 overflow-y-auto p-6">
                                    {activeTab === 'editor' ? (
                                        <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50 dark:bg-[#100a1c] p-4 text-xs font-mono text-slate-800 dark:text-slate-200 whitespace-pre-wrap leading-relaxed shadow-inner">
                                            {selectedTemplate.template}
                                        </div>
                                    ) : (
                                        <div className="rounded-xl border border-slate-200 dark:border-[#40285e] bg-slate-50/50 dark:bg-[#100a1c]/50 p-5 text-xs text-slate-900 dark:text-slate-100 space-y-2 leading-relaxed">
                                            <div className="font-bold text-sm text-purple-700 dark:text-purple-300 pb-2 border-b border-slate-200 dark:border-[#40285e]/80">
                                                {selectedTemplate.name}
                                            </div>
                                            <div className="whitespace-pre-wrap font-sans text-xs text-slate-700 dark:text-slate-300">
                                                {selectedTemplate.template}
                                            </div>
                                        </div>
                                    )}
                                </div>
                            </div>
                        ) : (
                            <div className="flex-1 flex items-center justify-center text-slate-400 text-xs">
                                Select a template from the left pane
                            </div>
                        )}
                    </div>
                </div>
            </div>
        </div>
    );
}
