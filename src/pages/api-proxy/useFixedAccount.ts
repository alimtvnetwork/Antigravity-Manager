import { useState, useEffect, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import { listAccounts } from '../../services/accountService';
import type { AvailableAccount, FixedAccountControls } from './types';

export function useFixedAccount(): FixedAccountControls {
    const { t } = useTranslation();

    // [FIX #820] Fixed account mode states
    const [preferredAccountId, setPreferredAccountId] = useState<string | null>(null);
    const [availableAccounts, setAvailableAccounts] = useState<AvailableAccount[]>([]);

    // [FIX #820] Load available accounts for fixed account mode
    const loadAccounts = useCallback(async () => {
        try {
            const accounts = await listAccounts();
            setAvailableAccounts(accounts.map(a => ({ id: a.id, email: a.email })));
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useFixedAccount.loadAccounts',
                triggerAction: 'listAccounts',
            });
        }
    }, []);

    // [FIX #820] Load current preferred account
    const loadPreferredAccount = useCallback(async () => {
        try {
            const prefId = await invoke<string | null>('get_preferred_account');
            setPreferredAccountId(prefId);
        } catch (error) {
            // Service not running, ignore; tracked in error history
            useErrorStore.getState().trackWarning(error, {
                source: 'useFixedAccount.loadPreferredAccount',
                triggerAction: 'get_preferred_account',
            });
        }
    }, []);

    // [FIX #820] Set preferred account
    const handleSetPreferredAccount = useCallback(async (accountId: string | null) => {
        try {
            const wasEnabled = preferredAccountId !== null;
            await invoke('set_preferred_account', { accountId });
            setPreferredAccountId(accountId);

            // Determine appropriate message
            let message: string;
            if (accountId === null) {
                message = t('proxy.config.scheduling.round_robin_set', { defaultValue: 'Round-robin mode enabled' });
            } else if (wasEnabled) {
                // Changed account while already in fixed mode
                const account = availableAccounts.find(a => a.id === accountId);
                message = t('proxy.config.scheduling.account_changed', {
                    defaultValue: `Switched to ${account?.email || accountId}`,
                    email: account?.email || accountId
                });
            } else {
                // Just enabled fixed mode
                message = t('proxy.config.scheduling.fixed_account_set', { defaultValue: 'Fixed account mode enabled' });
            }

            showToast(message, 'success');
        } catch (error) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useFixedAccount.handleSetPreferredAccount',
                triggerAction: 'set_preferred_account',
            });
            showToast(String(error), 'error');
        }
    }, [preferredAccountId, availableAccounts, t]);

    useEffect(() => {
        loadAccounts();
        loadPreferredAccount();
    }, [loadAccounts, loadPreferredAccount]);

    return {
        preferredAccountId,
        availableAccounts,
        handleSetPreferredAccount,
    };
}
