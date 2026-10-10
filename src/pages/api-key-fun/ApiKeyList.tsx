import React from 'react';
import type { Dispatch, SetStateAction, MouseEvent } from 'react';
import { useTranslation } from 'react-i18next';
import {
    Settings,
    HelpCircle,
    Copy,
    Pencil,
    Trash2,
    RefreshCw,
    CheckSquare,
    MinusSquare,
    Square
} from 'lucide-react';
import type { ManagedApiKey, OpenCodeProfileInfo } from './types';
import { DEFAULT_ENDPOINT, formatDate } from './utils';

export interface ApiKeyListProps {
    managedKeys: ManagedApiKey[];
    apiKey: string;
    baseUrl: string;
    editingId: string | null;
    setEditingId: Dispatch<SetStateAction<string | null>>;
    editNameValue: string;
    setEditNameValue: Dispatch<SetStateAction<string>>;
    syncingKey: string | null;
    querying: boolean;
    handleSelectKey: (item: ManagedApiKey) => void;
    handleCopy: (text: string) => Promise<void>;
    handleDeleteKey: (id: string, e: MouseEvent) => void;
    startRename: (item: ManagedApiKey, e: MouseEvent) => void;
    saveRename: (id: string) => void;
    getModelsForKey: (targetKey: string, targetUrl?: string) => string[] | undefined;
    profileInfo: (key: string, url: string, modelIds?: string[]) => OpenCodeProfileInfo;
    handleToggleOpenCodeProfile: (targetKey: string, targetUrl: string, explicitModels?: string[]) => Promise<void>;
}

export const ApiKeyList: React.FC<ApiKeyListProps> = ({
    managedKeys,
    apiKey,
    baseUrl,
    editingId,
    setEditingId,
    editNameValue,
    setEditNameValue,
    syncingKey,
    querying,
    handleSelectKey,
    handleCopy,
    handleDeleteKey,
    startRename,
    saveRename,
    getModelsForKey,
    profileInfo,
    handleToggleOpenCodeProfile,
}) => {
    const { t } = useTranslation();

    return (
        <div className="xl:col-span-4 lg:col-span-4 bg-white dark:bg-base-100 rounded-xl p-5 shadow-sm border border-gray-100 dark:border-base-200 sticky top-5">
            <h2 className="text-lg font-bold text-gray-900 dark:text-white mb-1 flex items-center gap-2">
                <Settings size={18} className="text-blue-500" />
                {t('apiKeyFun.keyManager.title', { defaultValue: 'Key Management' })}
            </h2>
            <p className="text-xs text-gray-400 mb-4">
                {t('apiKeyFun.keyManager.desc', { defaultValue: 'Save frequently used keys, click to quickly switch and query balance.' })}
            </p>

            <div className="space-y-2 max-h-[500px] overflow-y-auto pr-1">
                {managedKeys.map(item => {
                    const isActive = apiKey === item.key;
                    const isEditing = editingId === item.id;

                    return (
                        <div
                            key={item.id}
                            onClick={() => !isEditing && handleSelectKey(item)}
                            className={`p-4 rounded-2xl border text-left transition-all relative flex flex-col justify-between cursor-pointer group ${
                                isActive
                                    ? 'border-blue-500 bg-blue-50/40 dark:bg-blue-500/10 shadow-sm shadow-blue-500/10'
                                    : 'border-slate-200 dark:border-white/5 hover:border-slate-300 dark:hover:border-white/10 bg-white dark:bg-white/[0.02]'
                            }`}
                        >
                            <div className="flex items-center justify-between w-full">
                                <div className="flex flex-col gap-1.5 w-full min-w-0 pr-2">
                                    {isEditing ? (
                                        <input
                                            type="text"
                                            className="input input-sm input-bordered w-full max-w-[200px]"
                                            value={editNameValue}
                                            onChange={e => setEditNameValue(e.target.value)}
                                            onBlur={() => saveRename(item.id)}
                                            onKeyDown={e => {
                                                if (e.key === 'Enter') {
                                                    saveRename(item.id);
                                                } else if (e.key === 'Escape') {
                                                    e.preventDefault();
                                                    setEditingId(null);
                                                }
                                            }}
                                            autoFocus
                                            onClick={e => e.stopPropagation()}
                                        />
                                    ) : (
                                        <button type="button" className="text-left font-bold text-[13px] text-slate-800 dark:text-gray-200 truncate">
                                            {item.name}
                                        </button>
                                    )}

                                    <div className="flex items-center gap-2 text-[11px] text-slate-500 dark:text-gray-400 font-medium">
                                        <span>
                                            {t('apiKeyFun.keyManager.lastRemainingLabel', { defaultValue: 'Last Balance' })} {item.lastRemaining ? item.lastRemaining : '--'}
                                        </span>
                                        {item.lastStatus && (
                                            <span className={`w-1.5 h-1.5 rounded-full ${item.lastStatus === 'ok' ? 'bg-green-500 shadow-[0_0_8px_rgba(34,197,94,0.6)]' : 'bg-red-500 shadow-[0_0_8px_rgba(239,68,68,0.6)]'}`} />
                                        )}
                                    </div>

                                    <span className="text-[10px] text-slate-400 dark:text-gray-500 font-normal">
                                        {t('apiKeyFun.keyManager.addedAt', { defaultValue: 'Added at' })} {formatDate(item.createdAt)}
                                    </span>
                                </div>

                                {/* Hover Actions */}
                                <div className="flex flex-row items-center gap-0.5 opacity-0 group-hover:opacity-100 transition-opacity shrink-0">
                                    <button
                                        onClick={e => { e.stopPropagation(); handleCopy(item.key); }}
                                        className="p-1.5 text-slate-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-white/10 rounded-lg transition-colors"
                                        title="Copy Key"
                                    >
                                        <Copy size={15} />
                                    </button>
                                    {!isEditing && (
                                        <button
                                            onClick={e => startRename(item, e)}
                                            className="p-1.5 text-slate-400 hover:text-blue-500 hover:bg-blue-50 dark:hover:bg-white/10 rounded-lg transition-colors"
                                            title="Edit Name"
                                        >
                                            <Pencil size={14} />
                                        </button>
                                    )}
                                    <button
                                        onClick={e => handleDeleteKey(item.id, e)}
                                        className="p-1.5 text-slate-400 hover:text-red-500 hover:bg-red-50 dark:hover:bg-red-500/10 rounded-lg transition-colors"
                                        title="Delete"
                                    >
                                        <Trash2 size={15} />
                                    </button>
                                </div>
                            </div>

                            {/* OpenCode Profile Status & Activation */}
                            {(() => {
                                const isActiveKey = item.key.trim() === apiKey.trim();
                                const effectiveUrl = (isActiveKey && baseUrl.trim()) ? baseUrl.trim() : (item.baseUrl || DEFAULT_ENDPOINT);
                                const currentModels = getModelsForKey(item.key, effectiveUrl);
                                const keyInfo = profileInfo(item.key, effectiveUrl, currentModels);
                                const isThisKeySyncing = syncingKey === item.key.trim();

                                return (
                                    <div
                                        onClick={e => e.stopPropagation()}
                                        className="flex items-center justify-between pt-2 mt-2 border-t border-slate-100 dark:border-white/5 text-[11px]"
                                    >
                                        <div
                                            role="checkbox"
                                            aria-checked={keyInfo.status === 'synced' ? true : keyInfo.status === 'partial' ? 'mixed' : false}
                                            aria-label={`OpenCode ${keyInfo.suffix}`}
                                            aria-busy={isThisKeySyncing}
                                            aria-disabled={Boolean(syncingKey) || (isActiveKey && querying)}
                                            tabIndex={0}
                                            className="flex items-center gap-1.5 min-w-0 cursor-pointer select-none rounded focus:outline-none focus:ring-1 focus:ring-teal-500"
                                            onClick={() => handleToggleOpenCodeProfile(item.key, effectiveUrl, currentModels)}
                                            onKeyDown={(e) => {
                                                if (e.key === 'Enter' || e.key === ' ') {
                                                    e.preventDefault();
                                                    handleToggleOpenCodeProfile(item.key, effectiveUrl, currentModels);
                                                }
                                            }}
                                            title={
                                                keyInfo.status === 'synced'
                                                    ? t('apiKeyFun.opencode.clickToRemove', { defaultValue: 'OpenCode profile is active and synced. Click to deactivate/remove' })
                                                    : keyInfo.status === 'partial'
                                                        ? t('apiKeyFun.opencode.clickToUpdate', { defaultValue: 'Settings or models differ. Click to update' })
                                                        : t('apiKeyFun.opencode.clickToActivate', { defaultValue: 'Click to activate separate profile in OpenCode' })
                                            }
                                        >
                                            {isThisKeySyncing ? (
                                                <RefreshCw size={13} className="animate-spin text-teal-500 shrink-0" />
                                            ) : keyInfo.status === 'synced' ? (
                                                <CheckSquare size={14} className="text-emerald-500 shrink-0" />
                                            ) : keyInfo.status === 'partial' ? (
                                                <MinusSquare size={14} className="text-amber-500 shrink-0" />
                                            ) : (
                                                <Square size={14} className="text-slate-400 dark:text-gray-600 shrink-0" />
                                            )}
                                            <span className="font-mono text-slate-600 dark:text-gray-300 truncate">
                                                OpenCode: <span className="font-semibold text-slate-800 dark:text-gray-100">{keyInfo.suffix}</span>
                                            </span>
                                        </div>

                                        <div className="flex items-center gap-1 shrink-0">
                                            {keyInfo.status === 'synced' && (
                                                <span className="badge badge-xs badge-success text-[9px] font-medium gap-1 py-1 px-1.5">
                                                    {t('apiKeyFun.opencode.active', { defaultValue: 'Active' })}
                                                </span>
                                            )}
                                            {keyInfo.status === 'partial' && (
                                                <span className="badge badge-xs badge-warning text-[9px] font-medium gap-1 py-1 px-1.5">
                                                    {t('apiKeyFun.opencode.modified', { defaultValue: 'Modified' })}
                                                </span>
                                            )}
                                            {keyInfo.status === 'not_present' && (
                                                <span className="text-[10px] text-slate-400 dark:text-gray-500">
                                                    {t('apiKeyFun.opencode.inactive', { defaultValue: 'Inactive' })}
                                                </span>
                                            )}
                                        </div>
                                    </div>
                                );
                            })()}
                        </div>
                    );
                })}

                {managedKeys.length === 0 && (
                    <div className="text-center py-8 text-gray-400 text-xs flex flex-col items-center gap-2">
                        <HelpCircle size={28} className="opacity-20" />
                        <span>{t('apiKeyFun.keyManager.empty', { defaultValue: 'No saved keys.' })}</span>
                    </div>
                )}
            </div>
        </div>
    );
};
