import { createPortal } from 'react-dom';
import { Plus, Loader2, Info } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { isTauri } from '../../utils/env';
import { useAddAccountDialog } from './addAccount/useAddAccountDialog';
import type { AddAccountTab } from './addAccount/useAddAccountDialog';
import { StatusAlert, OAuthTabPanel, TokenTabPanel, ImportTabPanel } from './addAccount/AddAccountTabs';

interface AddAccountDialogProps {
    onAdd: (email: string, refreshToken: string) => Promise<void>;
    showText?: boolean;
    triggerClassName?: string;
}

const TABS: AddAccountTab[] = ['oauth', 'token', 'import'];

function AddAccountDialog({ onAdd, showText = true, triggerClassName }: AddAccountDialogProps) {
    const { t } = useTranslation();
    const api = useAddAccountDialog(onAdd);
    const { isOpen, setIsOpen, activeTab, setActiveTab, status, message } = api;

    return (
        <>
            <button
                className={
                    triggerClassName ||
                    'px-2.5 lg:px-4 py-2 bg-white dark:bg-base-100 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors flex items-center gap-2 shadow-sm border border-gray-200/50 dark:border-base-300'
                }
                onClick={() => setIsOpen(true)}
                title={!showText ? t('accounts.add_account') : undefined}
            >
                <Plus className="w-3.5 h-3.5" />
                {showText && <span className="hidden lg:inline">{t('accounts.add_account')}</span>}
            </button>

            {isOpen &&
                createPortal(
                    <div
                        className="fixed inset-0 z-[99999] flex items-center justify-center bg-black/50 backdrop-blur-sm"
                        style={{ position: 'fixed', top: 0, left: 0, right: 0, bottom: 0 }}
                    >
                        {/* Draggable Top Region */}
                        <div data-tauri-drag-region className="fixed top-0 left-0 right-0 h-8 z-[1]" />

                        {/* Click outside to close */}
                        <div className="absolute inset-0 z-[0]" onClick={() => setIsOpen(false)} />

                        <div className="bg-white dark:bg-base-100 text-gray-900 dark:text-base-content rounded-2xl shadow-2xl w-full max-w-lg p-6 relative z-[10] m-4 max-h-[90vh] overflow-y-auto">
                            <h3 className="font-bold text-lg mb-4">{t('accounts.add.title')}</h3>

                            {/* Tab Navigation */}
                            <div className="bg-gray-100 dark:bg-base-200 p-1 rounded-xl mb-6 grid grid-cols-3 gap-1">
                                {TABS.map((tab) => (
                                    <button
                                        key={tab}
                                        className={`py-2 px-1.5 sm:px-3 rounded-lg text-xs sm:text-sm font-medium transition-all duration-200 ${
                                            activeTab === tab
                                                ? 'bg-white dark:bg-base-100 shadow-sm text-blue-600 dark:text-blue-400'
                                                : 'text-gray-500 dark:text-gray-400 hover:text-gray-900 dark:hover:text-gray-200 hover:bg-gray-200/50 dark:hover:bg-base-300'
                                        } `}
                                        onClick={() => setActiveTab(tab)}
                                    >
                                        {t(`accounts.add.tabs.${tab}`)}
                                    </button>
                                ))}
                            </div>

                            {/* Web Mode Hint */}
                            {!isTauri() && (
                                <div className="alert alert-info mb-4 text-xs py-2 flex items-center gap-2 bg-blue-50 dark:bg-blue-900/10 text-blue-600 dark:text-blue-400 border-blue-100 dark:border-blue-800">
                                    <Info className="w-4 h-4" />
                                    <span>{t('accounts.add.oauth.web_hint', 'Google login will open in a new window')}</span>
                                </div>
                            )}

                            {/* Status alert section */}
                            <StatusAlert status={status} message={message} />

                            <div className="min-h-[200px]">
                                {activeTab === 'oauth' && <OAuthTabPanel api={api} />}
                                {activeTab === 'token' && <TokenTabPanel api={api} />}
                                {activeTab === 'import' && <ImportTabPanel api={api} />}
                            </div>

                            <div className="flex gap-3 w-full mt-6">
                                <button
                                    className="flex-1 px-4 py-2.5 bg-gray-100 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl hover:bg-gray-200 dark:hover:bg-base-300 transition-colors focus:outline-none focus:ring-2 focus:ring-200 dark:focus:ring-base-300"
                                    onClick={api.handleCancel}
                                    disabled={status === 'success'}
                                >
                                    {t('accounts.add.btn_cancel')}
                                </button>
                                {activeTab === 'token' && (
                                    <button
                                        className="flex-1 px-4 py-2.5 text-white font-medium rounded-xl shadow-md transition-all focus:outline-none focus:ring-2 focus:ring-offset-2 bg-blue-500 hover:bg-blue-600 focus:ring-blue-500 shadow-blue-100 dark:shadow-blue-900/30 flex justify-center items-center gap-2"
                                        onClick={api.handleSubmit}
                                        disabled={status === 'loading' || status === 'success'}
                                    >
                                        {status === 'loading' ? <Loader2 className="w-4 h-4 animate-spin" /> : null}
                                        {t('accounts.add.btn_confirm')}
                                    </button>
                                )}
                            </div>
                        </div>
                    </div>,
                    document.body
                )}
        </>
    );
}

export default AddAccountDialog;
