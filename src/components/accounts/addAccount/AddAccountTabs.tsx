import { Globe, Copy, Check, CheckCircle2, Link2, Database, FileClock, Loader2, XCircle } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import type { AddAccountDialogApi } from './useAddAccountDialog';

export function StatusAlert({ status, message }: { status: AddAccountDialogApi['status']; message: string }) {
    if (status === 'idle' || !message) return null;

    const styles = {
        loading: 'alert-info',
        success: 'alert-success',
        error: 'alert-error',
    };

    const icons = {
        loading: <Loader2 className="w-5 h-5 animate-spin" />,
        success: <CheckCircle2 className="w-5 h-5" />,
        error: <XCircle className="w-5 h-5" />,
    };

    return (
        <div className={`alert ${styles[status]} mb-4 text-sm py-2 shadow-sm`}>
            {icons[status]}
            <span>{message}</span>
        </div>
    );
}

export function OAuthTabPanel({ api }: { api: AddAccountDialogApi }) {
    const { t } = useTranslation();
    const { status, oauthUrl, oauthUrlCopied, manualCode, setManualCode } = api;
    const isBusy = status === 'loading' || status === 'success';

    return (
        <div className="space-y-6 py-4">
            <div className="text-center space-y-3">
                <div className="bg-blue-50 dark:bg-blue-900/20 p-6 rounded-full w-20 h-20 mx-auto flex items-center justify-center">
                    <Globe className="w-10 h-10 text-blue-500" />
                </div>
                <div className="space-y-1">
                    <h4 className="font-medium text-gray-900 dark:text-gray-100">{t('accounts.add.oauth.recommend')}</h4>
                    <p className="text-sm text-gray-500 dark:text-gray-400 max-w-xs mx-auto">
                        {t('accounts.add.oauth.desc')}
                    </p>
                </div>
            </div>
            <div className="space-y-3">
                <button
                    className="w-full px-4 py-3 bg-blue-600 hover:bg-blue-700 text-white font-medium rounded-xl shadow-lg shadow-blue-500/20 transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed"
                    onClick={api.handleOAuth}
                    disabled={isBusy}
                >
                    {status === 'loading' ? t('accounts.add.oauth.btn_waiting') : t('accounts.add.oauth.btn_start')}
                </button>

                {oauthUrl && (
                    <div className="space-y-2">
                        <div className="text-[11px] text-gray-500 dark:text-gray-400 text-left">
                            {t('accounts.add.oauth.link_label')}
                        </div>
                        <button
                            type="button"
                            className="w-full px-4 py-2 bg-white dark:bg-base-100 text-gray-600 dark:text-gray-300 text-sm font-medium rounded-xl border border-dashed border-gray-300 dark:border-gray-600 hover:bg-gray-50 dark:hover:bg-base-200 transition-all flex items-center gap-2"
                            onClick={api.handleCopyUrl}
                            title={t('accounts.add.oauth.link_click_to_copy')}
                        >
                            {oauthUrlCopied ? (
                                <Check className="w-3.5 h-3.5 text-emerald-600" />
                            ) : (
                                <Copy className="w-3.5 h-3.5" />
                            )}
                            <code className="text-[11px] font-mono truncate flex-1 text-left">{oauthUrl}</code>
                            <span className="text-[11px] whitespace-nowrap">
                                {oauthUrlCopied ? t('accounts.add.oauth.copied') : t('accounts.add.oauth.copy_link')}
                            </span>
                        </button>

                        <button
                            type="button"
                            className="w-full px-4 py-2 bg-white dark:bg-base-100 text-gray-700 dark:text-gray-300 text-sm font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-gray-50 dark:hover:bg-base-200 transition-all flex items-center justify-center gap-2 disabled:opacity-70 disabled:cursor-not-allowed"
                            onClick={api.handleCompleteOAuth}
                            disabled={isBusy}
                        >
                            <CheckCircle2 className="w-4 h-4" />
                            {t('accounts.add.oauth.btn_finish')}
                        </button>
                    </div>
                )}

                {/* Manual Code Entry - Always enabled to rescue stuck flows */}
                <div className="pt-4 mt-2 border-t border-gray-100 dark:border-base-200">
                    <div className="text-[11px] font-medium text-gray-400 dark:text-gray-500 mb-2 uppercase tracking-wider">
                        {t('accounts.add.oauth.manual_hint')}
                    </div>
                    <div className="relative group/manual flex gap-2">
                        <div className="relative flex-1">
                            <input
                                type="text"
                                className="w-full text-xs py-2 px-3 bg-white dark:bg-base-100 border border-gray-200 dark:border-base-300 rounded-xl focus:ring-2 focus:ring-blue-500/20 focus:border-blue-500 outline-none transition-all placeholder:text-gray-300 dark:placeholder:text-gray-600"
                                placeholder={t('accounts.add.oauth.manual_placeholder')}
                                value={manualCode}
                                onChange={(e) => setManualCode(e.target.value)}
                            />
                        </div>
                        <button
                            className="px-4 py-2 bg-neutral text-white dark:bg-white dark:text-neutral text-xs font-semibold rounded-xl hover:opacity-90 active:scale-95 transition-all disabled:opacity-50 disabled:scale-100 flex items-center gap-1.5"
                            onClick={api.handleManualSubmit}
                            disabled={!manualCode.trim()}
                        >
                            <Link2 className="w-3.5 h-3.5" />
                            {t('common.submit')}
                        </button>
                    </div>
                </div>
            </div>
        </div>
    );
}

export function TokenTabPanel({ api }: { api: AddAccountDialogApi }) {
    const { t } = useTranslation();
    const { status, refreshToken, setRefreshToken } = api;
    const isBusy = status === 'loading' || status === 'success';

    return (
        <div className="space-y-4 py-2">
            <div className="bg-gray-50 dark:bg-base-200 p-4 rounded-lg border border-gray-200 dark:border-base-300">
                <div className="flex justify-between items-center mb-2">
                    <span className="text-sm font-medium text-gray-500 dark:text-gray-400">
                        {t('accounts.add.token.label')}
                    </span>
                </div>
                <textarea
                    className="textarea textarea-bordered w-full h-32 font-mono text-xs leading-relaxed focus:outline-none focus:border-blue-500 transition-colors bg-white dark:bg-base-100 text-gray-900 dark:text-base-content border-gray-300 dark:border-base-300 placeholder:text-gray-400"
                    placeholder={t('accounts.add.token.placeholder')}
                    value={refreshToken}
                    onChange={(e) => setRefreshToken(e.target.value)}
                    disabled={isBusy}
                />
                <p className="text-[10px] text-gray-400 mt-2">{t('accounts.add.token.hint')}</p>
            </div>
        </div>
    );
}

export function ImportTabPanel({ api }: { api: AddAccountDialogApi }) {
    const { t } = useTranslation();
    const { status } = api;
    const isBusy = status === 'loading' || status === 'success';

    return (
        <div className="space-y-6 py-2">
            <div className="space-y-2">
                <h4 className="font-semibold flex items-center gap-2 text-gray-800 dark:text-gray-200">
                    <Database className="w-4 h-4 text-gray-600 dark:text-gray-400" />
                    {t('accounts.add.import.scheme_a')}
                </h4>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t('accounts.add.import.scheme_a_desc')}</p>
                <button
                    className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-blue-50 dark:hover:bg-blue-900/20 hover:border-blue-200 dark:hover:border-blue-800 hover:text-blue-600 dark:hover:text-blue-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed mb-2 shadow-sm"
                    onClick={api.handleImportDb}
                    disabled={isBusy}
                >
                    <CheckCircle2 className="w-4 h-4 opacity-0 group-hover:opacity-100 transition-opacity" />
                    {t('accounts.add.import.btn_db')}
                </button>
                <button
                    className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-indigo-50 dark:hover:bg-indigo-900/20 hover:border-indigo-200 dark:hover:border-indigo-800 hover:text-indigo-600 dark:hover:text-indigo-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
                    onClick={api.handleImportCustomDb}
                    disabled={isBusy}
                >
                    <Database className="w-4 h-4" />
                    {t('accounts.add.import.btn_custom_db') || 'Custom DB (state.vscdb)'}
                </button>
            </div>

            <div className="divider text-xs text-gray-300 dark:text-gray-600">{t('accounts.add.import.or')}</div>

            <div className="space-y-2">
                <h4 className="font-semibold flex items-center gap-2 text-gray-800 dark:text-gray-200">
                    <FileClock className="w-4 h-4 text-gray-600 dark:text-gray-400" />
                    {t('accounts.add.import.scheme_b')}
                </h4>
                <p className="text-xs text-gray-500 dark:text-gray-400">{t('accounts.add.import.scheme_b_desc')}</p>
                <button
                    className="w-full px-4 py-3 bg-gray-50 dark:bg-base-200 text-gray-700 dark:text-gray-300 font-medium rounded-xl border border-gray-200 dark:border-base-300 hover:bg-emerald-50 dark:hover:bg-emerald-900/20 hover:border-emerald-200 dark:hover:border-emerald-800 hover:text-emerald-600 dark:hover:text-emerald-400 transition-all flex items-center justify-center gap-2 disabled:opacity-50 disabled:cursor-not-allowed shadow-sm"
                    onClick={api.handleImportV1}
                    disabled={isBusy}
                >
                    <FileClock className="w-4 h-4" />
                    {t('accounts.add.import.btn_v1')}
                </button>
            </div>
        </div>
    );
}
