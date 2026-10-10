import { useState, useCallback } from 'react';
import { useTranslation } from 'react-i18next';
import { request as invoke } from '../../utils/request';
import { showToast } from '../../components/common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import type { AppConfig } from '../../types/config';
import type { CredentialEditing, ProxyConfigUpdater } from './types';
import { API_KEY_MIN_LENGTH, ADMIN_PASSWORD_MIN_LENGTH } from './constants';

interface UseCredentialEditingArgs {
    appConfig: AppConfig | null;
    updateProxyConfig: ProxyConfigUpdater;
}

export function useCredentialEditing({ appConfig, updateProxyConfig }: UseCredentialEditingArgs): CredentialEditing {
    const { t } = useTranslation();

    const [isEditingApiKey, setIsEditingApiKey] = useState(false);
    const [tempApiKey, setTempApiKey] = useState('');

    const [isEditingAdminPassword, setIsEditingAdminPassword] = useState(false);
    const [tempAdminPassword, setTempAdminPassword] = useState('');

    const [isRegenerateKeyConfirmOpen, setIsRegenerateKeyConfirmOpen] = useState(false);

    // API Key editing functions
    const validateApiKey = (key: string): boolean => {
        // Must start with 'sk-' and be at least 10 characters long
        return key.startsWith('sk-') && key.length >= API_KEY_MIN_LENGTH;
    };

    const handleEditApiKey = useCallback(() => {
        setTempApiKey(appConfig?.proxy.api_key || '');
        setIsEditingApiKey(true);
    }, [appConfig]);

    const handleSaveApiKey = useCallback(() => {
        if (!validateApiKey(tempApiKey)) {
            showToast(t('proxy.config.api_key_invalid'), 'error');
            return;
        }
        updateProxyConfig({ api_key: tempApiKey });
        setIsEditingApiKey(false);
        showToast(t('proxy.config.api_key_updated'), 'success');
    }, [tempApiKey, updateProxyConfig, t]);

    const handleCancelEditApiKey = useCallback(() => {
        setTempApiKey('');
        setIsEditingApiKey(false);
    }, []);

    const handleGenerateApiKey = useCallback(() => {
        setIsRegenerateKeyConfirmOpen(true);
    }, []);

    const executeGenerateApiKey = useCallback(async () => {
        setIsRegenerateKeyConfirmOpen(false);
        try {
            const newKey = await invoke<string>('generate_api_key');
            updateProxyConfig({ api_key: newKey });
            showToast(t('common.success'), 'success');
        } catch (error: unknown) {
            useErrorStore.getState().trackWarning(error, {
                source: 'useCredentialEditing.executeGenerateApiKey',
                triggerAction: 'generate_api_key',
            });
            showToast(t('proxy.dialog.operate_failed', { error: String(error) }), 'error');
        }
    }, [updateProxyConfig, t]);

    // Admin Password editing functions
    const handleEditAdminPassword = useCallback(() => {
        setTempAdminPassword(appConfig?.proxy.admin_password || '');
        setIsEditingAdminPassword(true);
    }, [appConfig]);

    const handleSaveAdminPassword = useCallback(() => {
        // Validation: can be empty (meaning fallback to api_key) or at least 4 chars
        if (tempAdminPassword && tempAdminPassword.length < ADMIN_PASSWORD_MIN_LENGTH) {
            showToast(t('proxy.config.admin_password_short', { defaultValue: 'Password is too short (min 4 chars)' }), 'error');
            return;
        }
        updateProxyConfig({ admin_password: tempAdminPassword || undefined });
        setIsEditingAdminPassword(false);
        showToast(t('proxy.config.admin_password_updated', { defaultValue: 'Web UI password updated' }), 'success');
    }, [tempAdminPassword, updateProxyConfig, t]);

    const handleCancelEditAdminPassword = useCallback(() => {
        setTempAdminPassword('');
        setIsEditingAdminPassword(false);
    }, []);

    return {
        isEditingApiKey,
        tempApiKey,
        setTempApiKey,
        handleEditApiKey,
        handleSaveApiKey,
        handleCancelEditApiKey,
        isEditingAdminPassword,
        tempAdminPassword,
        setTempAdminPassword,
        handleEditAdminPassword,
        handleSaveAdminPassword,
        handleCancelEditAdminPassword,
        handleGenerateApiKey,
        executeGenerateApiKey,
        isRegenerateKeyConfirmOpen,
        setIsRegenerateKeyConfirmOpen,
    };
}
