import { useState, useEffect, useRef } from 'react';
import { useAccountStore } from '../../../stores/useAccountStore';
import { useTranslation } from 'react-i18next';
import { listen } from '@tauri-apps/api/event';
import { open } from '@tauri-apps/plugin-dialog';
import { request as invoke } from '../../../utils/request';
import { isTauri } from '../../../utils/env';
import { copyToClipboard } from '../../../utils/clipboard';
import { useErrorStore } from '../../../stores/error-store';

export type AddAccountStatus = 'idle' | 'loading' | 'success' | 'error';
export type AddAccountTab = 'oauth' | 'token' | 'import';

export interface AddAccountDialogApi {
    isOpen: boolean;
    setIsOpen: (open: boolean) => void;
    activeTab: AddAccountTab;
    setActiveTab: (tab: AddAccountTab) => void;
    refreshToken: string;
    setRefreshToken: (v: string) => void;
    oauthUrl: string;
    oauthUrlCopied: boolean;
    manualCode: string;
    setManualCode: (v: string) => void;
    status: AddAccountStatus;
    message: string;
    resetState: () => void;
    handleSubmit: () => Promise<void>;
    handleOAuth: () => void;
    handleCompleteOAuth: () => void;
    handleCopyUrl: () => Promise<void>;
    handleManualSubmit: () => Promise<void>;
    handleImportDb: () => void;
    handleImportV1: () => void;
    handleImportCustomDb: () => Promise<void>;
    handleCancel: () => Promise<void>;
}

interface TokenCandidate {
    refresh_token?: unknown;
}

export function useAddAccountDialog(onAdd: (email: string, refreshToken: string) => Promise<void>): AddAccountDialogApi {
    const { t } = useTranslation();
    const fetchAccounts = useAccountStore((state) => state.fetchAccounts);
    const [isOpen, setIsOpen] = useState(false);
    const [activeTab, setActiveTab] = useState<AddAccountTab>(isTauri() ? 'oauth' : 'token');
    const [refreshToken, setRefreshToken] = useState('');
    const [oauthUrl, setOauthUrl] = useState('');
    const [oauthUrlCopied, setOauthUrlCopied] = useState(false);
    const [manualCode, setManualCode] = useState('');

    const [status, setStatus] = useState<AddAccountStatus>('idle');
    const [message, setMessage] = useState('');

    const { startOAuthLogin, completeOAuthLogin, cancelOAuthLogin, importFromDb, importV1Accounts, importFromCustomDb } =
        useAccountStore();

    const oauthUrlRef = useRef(oauthUrl);
    const statusRef = useRef(status);
    const activeTabRef = useRef(activeTab);
    const isOpenRef = useRef(isOpen);

    useEffect(() => {
        oauthUrlRef.current = oauthUrl;
        statusRef.current = status;
        activeTabRef.current = activeTab;
        isOpenRef.current = isOpen;
    }, [oauthUrl, status, activeTab, isOpen]);

    const resetState = () => {
        setStatus('idle');
        setMessage('');
        setRefreshToken('');
        setOauthUrl('');
        setOauthUrlCopied(false);
    };

    // Reset state when dialog opens or tab changes
    useEffect(() => {
        if (isOpen) {
            resetState();
        }
    }, [isOpen, activeTab]);

    // Listen for OAuth URL
    useEffect(() => {
        if (!isTauri()) return;
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen('oauth-url-generated', (event) => {
                setOauthUrl(event.payload as string);
            });
        };

        setupListener();

        return () => {
            if (unlisten) unlisten();
        };
    }, []);

    // Listen for OAuth callback completion (user may open the URL manually without clicking Start)
    useEffect(() => {
        if (!isTauri()) return;
        let unlisten: (() => void) | undefined;

        const setupListener = async () => {
            unlisten = await listen('oauth-callback-received', async () => {
                if (!isOpenRef.current) return;
                if (activeTabRef.current !== 'oauth') return;
                if (statusRef.current === 'loading' || statusRef.current === 'success') return;
                if (!oauthUrlRef.current) return;

                setStatus('loading');
                setMessage(`${t('accounts.add.tabs.oauth')}...`);

                try {
                    await completeOAuthLogin();
                    setStatus('success');
                    setMessage(`${t('accounts.add.tabs.oauth')} ${t('common.success')}!`);
                    setTimeout(() => {
                        setIsOpen(false);
                        resetState();
                    }, 1500);
                } catch (error) {
                    setStatus('error');
                    const errorMsg = String(error);
                    if (errorMsg.includes('Refresh Token') || errorMsg.includes('refresh_token')) {
                        setMessage(errorMsg);
                    } else if (errorMsg.includes('Tauri') || errorMsg.toLowerCase().includes('environment')) {
                        setMessage(t('common.environment_error', { error: errorMsg }));
                    } else {
                        setMessage(`${t('accounts.add.tabs.oauth')} ${t('common.error')}: ${errorMsg}`);
                    }
                }
            });
        };

        setupListener();

        return () => {
            if (unlisten) unlisten();
        };
    }, [completeOAuthLogin, t]);

    // Pre-generate OAuth URL when dialog opens on OAuth tab (so URL is shown BEFORE "Start OAuth")
    useEffect(() => {
        if (!isOpen) return;
        if (activeTab !== 'oauth') return;
        if (oauthUrl) return;

        invoke<{ url?: string } | string>('prepare_oauth_url')
            .then((res) => {
                const url = typeof res === 'string' ? res : res?.url;
                if (url && url.length > 0) setOauthUrl(url);
            })
            .catch((e) => {
                // Tracked in the error module; OAuth tab simply won't offer the prepared URL.
                useErrorStore.getState().trackWarning(e, {
                    source: 'AddAccountDialog.prepareOAuthUrl',
                    triggerAction: 'prepare_oauth_url',
                });
            });
    }, [isOpen, activeTab, oauthUrl]);

    // If user navigates away from OAuth tab, cancel prepared flow to release the port.
    useEffect(() => {
        if (!isOpen) return;
        if (activeTab === 'oauth') return;
        if (!oauthUrl) return;

        cancelOAuthLogin().catch(() => {
            // Best-effort cancellation; port release is handled server-side on timeout.
        });
        setOauthUrl('');
        setOauthUrlCopied(false);
    }, [isOpen, activeTab, oauthUrl, cancelOAuthLogin]);

    const handleAction = async (
        actionName: string,
        actionFn: () => Promise<unknown>,
        options?: { clearOauthUrl?: boolean }
    ) => {
        setStatus('loading');
        setMessage(`${actionName}...`);
        if (options?.clearOauthUrl !== false) {
            setOauthUrl('');
        }
        try {
            await actionFn();
            setStatus('success');
            setMessage(`${actionName} ${t('common.success')}!`);
            setTimeout(() => {
                setIsOpen(false);
                resetState();
            }, 1500);
        } catch (error) {
            setStatus('error');
            const errorMsg = String(error);
            if (errorMsg.includes('Refresh Token') || errorMsg.includes('refresh_token')) {
                setMessage(errorMsg);
            } else if (errorMsg.includes('Tauri') || errorMsg.toLowerCase().includes('environment')) {
                setMessage(t('common.environment_error', { error: errorMsg }));
            } else {
                setMessage(`${actionName} ${t('common.error')}: ${errorMsg}`);
            }
        }
    };

    const handleSubmit = async () => {
        if (!refreshToken) {
            setStatus('error');
            setMessage(t('accounts.add.token.error_token'));
            return;
        }

        setStatus('loading');

        let tokens: string[] = [];
        const input = refreshToken.trim();

        try {
            if (input.startsWith('[') && input.endsWith(']')) {
                const parsed = JSON.parse(input) as unknown;
                if (Array.isArray(parsed)) {
                    tokens = (parsed as TokenCandidate[])
                        .map((item) => item.refresh_token)
                        .filter((tok): tok is string => typeof tok === 'string' && tok.startsWith('1//'));
                }
            }
        } catch {
            // JSON parse failed, fall back to regex extraction below.
        }

        if (tokens.length === 0) {
            const regex = /1\/\/[a-zA-Z0-9_\-]+/g;
            const matches = input.match(regex);
            if (matches) {
                tokens = matches;
            }
        }

        tokens = [...new Set(tokens)];

        if (tokens.length === 0) {
            setStatus('error');
            setMessage(t('accounts.add.token.error_token'));
            return;
        }

        let successCount = 0;
        let failCount = 0;

        for (let i = 0; i < tokens.length; i++) {
            const currentToken = tokens[i];
            setMessage(t('accounts.add.token.batch_progress', { current: i + 1, total: tokens.length }));

            try {
                await onAdd('', currentToken);
                successCount++;
            } catch (error) {
                // Tracked in the error module; per-token failure counted and reported in the result summary.
                useErrorStore.getState().trackWarning(error, {
                    source: 'AddAccountDialog.importTokens',
                    triggerAction: 'add_token',
                    context: { tokenIndex: i + 1 },
                });
                failCount++;
            }
            await new Promise((r) => setTimeout(r, 100));
        }

        if (successCount === tokens.length) {
            setStatus('success');
            setMessage(t('accounts.add.token.batch_success', { count: successCount }));
            setTimeout(() => {
                setIsOpen(false);
                resetState();
            }, 1500);
        } else if (successCount > 0) {
            setStatus('success');
            setMessage(t('accounts.add.token.batch_partial', { success: successCount, fail: failCount }));
        } else {
            setStatus('error');
            setMessage(t('accounts.add.token.batch_fail'));
        }
    };

    const handleOAuthWeb = async () => {
        try {
            setStatus('loading');
            setMessage(t('accounts.add.oauth.btn_start') + '...');

            const res = await invoke<{ url?: string } | string>('prepare_oauth_url');
            const url = typeof res === 'string' ? res : res.url;

            if (!url) {
                throw new Error(t('accounts.add.oauth.error_no_url', 'Failed to retrieve OAuth URL'));
            }

            setOauthUrl(url);

            const popup = window.open(url, '_blank');

            if (!popup) {
                setStatus('error');
                setMessage(t('accounts.add.oauth.popup_blocked', 'Popup was blocked by the browser'));
                return;
            }

            const handleMessage = async (event: MessageEvent) => {
                if (event.origin && event.origin !== window.location.origin) {
                    return;
                }
                if (event.data?.type === 'oauth-success') {
                    popup.close();
                    window.removeEventListener('message', handleMessage);
                    await fetchAccounts();
                    setStatus('success');
                    setMessage(t('accounts.add.oauth_success') || t('common.success'));
                    setTimeout(() => {
                        setIsOpen(false);
                        resetState();
                    }, 1500);
                }
            };

            window.addEventListener('message', handleMessage);

            const timer = setInterval(() => {
                if (popup.closed) {
                    clearInterval(timer);
                    window.removeEventListener('message', handleMessage);
                    if (statusRef.current === 'loading') {
                        setStatus('idle');
                        setMessage('');
                    }
                }
            }, 1000);
        } catch (error) {
            // Tracked in the error module; dialog already shows the error status to the user.
            useErrorStore.getState().trackWarning(error, {
                source: 'AddAccountDialog.handleOAuthWeb',
                triggerAction: 'oauth_web',
            });
            setStatus('error');
            setMessage(`${t('common.error')}: ${error}`);
        }
    };

    const handleOAuth = () => {
        if (!isTauri()) {
            handleOAuthWeb();
            return;
        }
        handleAction(t('accounts.add.tabs.oauth'), startOAuthLogin, { clearOauthUrl: false });
    };

    const handleCompleteOAuth = () => {
        handleAction(t('accounts.add.tabs.oauth'), completeOAuthLogin, { clearOauthUrl: false });
    };

    const handleCopyUrl = async () => {
        if (oauthUrl) {
            const success = await copyToClipboard(oauthUrl);
            if (success) {
                setOauthUrlCopied(true);
                window.setTimeout(() => setOauthUrlCopied(false), 1500);
            }
        }
    };

    const handleManualSubmit = async () => {
        if (!manualCode.trim()) return;

        setStatus('loading');
        setMessage(t('accounts.add.oauth.manual_submitting', 'Submitting authorization code...'));

        try {
            await invoke('submit_oauth_code', { code: manualCode.trim(), state: null });
            setStatus('success');
            setMessage(t('accounts.add.oauth.manual_submitted', 'Authorization code submitted. Processing in background...'));
            setManualCode('');

            if (!isTauri()) {
                setTimeout(async () => {
                    await fetchAccounts();
                    setIsOpen(false);
                    resetState();
                }, 2000);
            }
        } catch (error) {
            const errStr = String(error);
            if (errStr.includes('No active OAuth flow')) {
                setMessage(t('accounts.add.oauth.error_no_flow'));
            } else {
                setMessage(`${t('common.error')}: ${errStr}`);
            }
            setStatus('error');
        }
    };

    const handleImportDb = () => {
        handleAction(t('accounts.add.import.btn_db'), async () => {
            const accounts = await importFromDb();
            if (Array.isArray(accounts) && accounts.length > 0) {
                setMessage(t('accounts.add.token.batch_success', { count: accounts.length }));
            }
            return accounts;
        });
    };

    const handleImportV1 = () => {
        handleAction(t('accounts.add.import.btn_v1'), importV1Accounts);
    };

    const handleImportCustomDb = async () => {
        try {
            if (!isTauri()) {
                alert(t('common.tauri_api_not_loaded') || 'Storage import only works in desktop app.');
                return;
            }
            const selected = await open({
                multiple: false,
                filters: [
                    { name: 'VSCode DB', extensions: ['vscdb'] },
                    { name: 'All Files', extensions: ['*'] },
                ],
            });

            if (selected && typeof selected === 'string') {
                handleAction(t('accounts.add.import.btn_custom_db') || 'Import Custom DB', () =>
                    importFromCustomDb(selected)
                );
            }
        } catch (err) {
            // Tracked in the error module; file-picker dialog failed to open, user can retry.
            useErrorStore.getState().trackWarning(err, {
                source: 'AddAccountDialog.openDialog',
                triggerAction: 'open_file_dialog',
            });
        }
    };

    const handleCancel = async () => {
        if (status === 'loading' && activeTab === 'oauth') {
            await cancelOAuthLogin();
        }
        setIsOpen(false);
    };

    return {
        isOpen,
        setIsOpen,
        activeTab,
        setActiveTab,
        refreshToken,
        setRefreshToken,
        oauthUrl,
        oauthUrlCopied,
        manualCode,
        setManualCode,
        status,
        message,
        resetState,
        handleSubmit,
        handleOAuth,
        handleCompleteOAuth,
        handleCopyUrl,
        handleManualSubmit,
        handleImportDb,
        handleImportV1,
        handleImportCustomDb,
        handleCancel,
    };
}
