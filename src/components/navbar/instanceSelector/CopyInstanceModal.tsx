import { Copy, X, Check, Layers, Sliders, FolderSync } from 'lucide-react';
import { cn } from '../../../utils/cn';
import type { InstanceSelectorApi } from './useInstanceSelector';

function CloneScopeCard({
    selected,
    onSelect,
    icon,
    title,
    badge,
    badgeClass,
    desc,
}: {
    selected: boolean;
    onSelect: () => void;
    icon: React.ReactNode;
    title: string;
    badge: string;
    badgeClass: string;
    desc: string;
}) {
    return (
        <div
            onClick={onSelect}
            className={cn(
                'p-3.5 rounded-xl border-2 transition-all cursor-pointer flex items-start gap-3',
                selected
                    ? 'border-blue-500 dark:border-cyan-400 bg-blue-50/80 dark:bg-[#0c283f] ring-2 ring-blue-500/20 dark:ring-cyan-400/25 shadow-xs dark:shadow-[0_0_15px_rgba(6,182,212,0.12)]'
                    : 'border-gray-200 dark:border-[#14344d] bg-gray-50/50 dark:bg-[#061724] hover:border-gray-300 dark:hover:border-[#1e4a6d] hover:bg-gray-100/60 dark:hover:bg-[#092033]'
            )}
        >
            <div className="mt-0.5 shrink-0">
                <div
                    className={cn(
                        'w-4 h-4 rounded-full border-2 flex items-center justify-center transition-colors',
                        selected
                            ? 'border-blue-600 dark:border-cyan-400 bg-blue-600 dark:bg-cyan-500'
                            : 'border-gray-400 dark:border-slate-500 bg-transparent'
                    )}
                >
                    {selected && <div className="w-1.5 h-1.5 rounded-full bg-white dark:bg-[#081a29]" />}
                </div>
            </div>
            <div className="min-w-0 flex-1">
                <div className="font-bold text-xs text-gray-900 dark:text-white flex items-center justify-between gap-1.5">
                    <span className="flex items-center gap-1.5">
                        {icon}
                        <span>{title}</span>
                    </span>
                    <span className={`px-2 py-0.5 rounded-full text-[9px] font-bold uppercase tracking-wider border ${badgeClass}`}>
                        {badge}
                    </span>
                </div>
                <p className="text-[11px] text-gray-600 dark:text-slate-300 leading-relaxed mt-1">{desc}</p>
            </div>
        </div>
    );
}

export function CopyInstanceModal({ api }: { api: InstanceSelectorApi }) {
    const {
        t,
        isCopyOpen,
        setIsCopyOpen,
        copyInstanceName,
        setCopyInstanceName,
        cloneMode,
        setCloneMode,
        copyProjects,
        setCopyProjects,
        handleCopy,
    } = api;

    if (!isCopyOpen) return null;

    return (
        <div
            className="fixed inset-0 bg-black/50 backdrop-blur-sm flex items-center justify-center z-[99999] p-4"
            onClick={() => setIsCopyOpen(false)}
        >
            <div
                className="bg-white dark:bg-[#081a29] rounded-2xl p-6 w-full max-w-md shadow-2xl border border-gray-200 dark:border-[#153a54] ring-1 ring-black/5 dark:ring-white/5 transition-all"
                onClick={(e) => e.stopPropagation()}
            >
                <div className="flex items-center justify-between pb-3.5 border-b border-gray-100 dark:border-[#153a54] mb-4">
                    <div className="flex items-center gap-2.5">
                        <div className="p-2.5 rounded-xl bg-blue-50 dark:bg-[#0c283f] text-blue-600 dark:text-cyan-400 border border-blue-200/60 dark:border-cyan-500/30 shadow-xs">
                            <Copy className="w-5 h-5" />
                        </div>
                        <div>
                            <h3 className="font-bold text-sm text-gray-900 dark:text-white tracking-tight">
                                {t('instances.copy_modal_title', 'Duplicate / Clone Profile')}
                            </h3>
                            <p className="text-[11px] text-gray-500 dark:text-slate-400 mt-0.5">
                                {t('instances.copy_modal_subtitle', 'Create an isolated duplicate of this profile environment')}
                            </p>
                        </div>
                    </div>
                    <button
                        type="button"
                        onClick={() => setIsCopyOpen(false)}
                        className="p-1.5 rounded-lg text-gray-400 hover:text-gray-600 dark:text-slate-400 dark:hover:text-white hover:bg-gray-100 dark:hover:bg-[#122e47] transition-colors cursor-pointer"
                        title={t('common.close', 'Close')}
                    >
                        <X className="w-4 h-4" />
                    </button>
                </div>

                <label className="block text-xs font-semibold text-gray-700 dark:text-slate-200 mb-1.5">
                    {t('instances.copy_name_label', 'New Profile Name')}
                </label>
                <input
                    type="text"
                    placeholder={t('instances.copy_placeholder', 'New profile name')}
                    value={copyInstanceName}
                    onChange={(e) => setCopyInstanceName(e.target.value)}
                    onKeyDown={(e) => e.key === 'Enter' && handleCopy()}
                    className="w-full px-4 py-2.5 bg-gray-50 dark:bg-[#051320] border border-gray-200 dark:border-[#173d5c] text-gray-900 dark:text-white rounded-xl mb-4 text-xs font-medium focus:outline-none focus:ring-2 focus:ring-blue-500/40 dark:focus:ring-cyan-500/40 focus:border-blue-500 dark:focus:border-cyan-400 transition-all shadow-inner placeholder-gray-400 dark:placeholder-slate-500"
                    autoFocus
                />

                {/* Scope Selection Cards */}
                <div className="mb-4 space-y-2">
                    <span className="block text-[11px] font-bold text-gray-500 dark:text-cyan-400/90 uppercase tracking-wider mb-2">
                        {t('instances.clone_mode_label', 'Duplication Scope / Clone Type')}
                    </span>
                    <div className="grid grid-cols-1 gap-2.5">
                        <CloneScopeCard
                            selected={cloneMode === 'full'}
                            onSelect={() => setCloneMode('full')}
                            icon={<Layers className="w-3.5 h-3.5 text-blue-500 dark:text-cyan-400 shrink-0" />}
                            title={t('instances.clone_mode_full', 'IDE Copy (Full Environment & Sessions)')}
                            badge="Full"
                            badgeClass="bg-cyan-100 dark:bg-cyan-950/80 text-cyan-800 dark:text-cyan-300 border-cyan-300/50 dark:border-cyan-500/30"
                            desc={t('instances.clone_mode_full_desc', 'Clones complete isolated environment, sessions, extensions, cache, and state.')}
                        />
                        <CloneScopeCard
                            selected={cloneMode === 'profile'}
                            onSelect={() => setCloneMode('profile')}
                            icon={<Sliders className="w-3.5 h-3.5 text-indigo-500 dark:text-blue-400 shrink-0" />}
                            title={t('instances.clone_mode_profile', 'Profile Copy (Preferences & Snippets)')}
                            badge="Preferences"
                            badgeClass="bg-slate-100 dark:bg-slate-800 text-slate-700 dark:text-slate-300 border-slate-200 dark:border-slate-700"
                            desc={t('instances.clone_mode_profile_desc', 'Copies only User preferences, keybindings, and snippets without bulky runtime session state.')}
                        />
                    </div>
                </div>

                {/* Copy Workspace Projects & Folders Option */}
                <div
                    onClick={() => setCopyProjects(!copyProjects)}
                    className={cn(
                        'mb-5 p-3.5 rounded-xl border transition-all flex items-center justify-between gap-3 cursor-pointer',
                        copyProjects
                            ? 'border-blue-500/70 dark:border-cyan-400/70 bg-blue-50/60 dark:bg-[#0c283f] ring-1 ring-blue-500/20 dark:ring-cyan-400/20 shadow-xs dark:shadow-[0_0_12px_rgba(6,182,212,0.1)]'
                            : 'border-gray-200 dark:border-[#14344d] bg-gray-50/50 dark:bg-[#061724] hover:border-gray-300 dark:hover:border-[#1e4a6d]'
                    )}
                >
                    <div className="min-w-0 pr-2">
                        <label
                            htmlFor="inst-selector-copy-projects"
                            onClick={(e) => e.stopPropagation()}
                            className="font-bold text-xs text-gray-900 dark:text-white cursor-pointer flex items-center gap-1.5"
                        >
                            <FolderSync className="w-3.5 h-3.5 text-emerald-500 dark:text-emerald-400 shrink-0" />
                            <span>{t('instances.copy_projects_label', 'Copy Workspace Projects & Folders')}</span>
                        </label>
                        <p className="text-[11px] text-gray-600 dark:text-slate-300 mt-1 leading-relaxed">
                            {t('instances.copy_projects_desc', 'Duplicate opened workspaces, project states, and recent folder paths into the new profile.')}
                        </p>
                    </div>
                    <div
                        className={cn(
                            'w-5 h-5 rounded-md border flex items-center justify-center transition-all duration-200 shrink-0 shadow-xs',
                            copyProjects
                                ? 'border-blue-600 dark:border-cyan-400 bg-blue-600 dark:bg-cyan-500 text-white'
                                : 'border-gray-300 dark:border-slate-600 bg-white dark:bg-[#040e16]'
                        )}
                    >
                        {copyProjects && <Check className="w-3.5 h-3.5 stroke-[3]" />}
                    </div>
                </div>

                <div className="flex justify-end items-center gap-2.5 pt-3 border-t border-gray-100 dark:border-[#153a54]">
                    <button
                        type="button"
                        onClick={() => setIsCopyOpen(false)}
                        className="px-4 py-2.5 rounded-xl text-xs font-semibold text-gray-600 dark:text-slate-300 border border-gray-200 dark:border-[#153a54] hover:bg-gray-100 dark:hover:bg-[#102a40] transition-all duration-200 active:scale-95 cursor-pointer"
                    >
                        {t('common.cancel', 'Cancel')}
                    </button>
                    <button
                        type="button"
                        onClick={handleCopy}
                        disabled={!copyInstanceName.trim()}
                        className="px-5 py-2.5 bg-gradient-to-r from-blue-600 via-indigo-600 to-cyan-500 hover:from-blue-500 hover:via-indigo-500 hover:to-cyan-400 text-white font-bold text-xs rounded-xl shadow-lg shadow-blue-500/25 dark:shadow-cyan-500/20 hover:shadow-cyan-500/40 transition-all duration-200 active:scale-95 disabled:opacity-40 disabled:cursor-not-allowed cursor-pointer flex items-center gap-2"
                    >
                        <Copy className="w-3.5 h-3.5" />
                        <span>{t('instances.duplicate', 'Duplicate')}</span>
                    </button>
                </div>
            </div>
        </div>
    );
}
