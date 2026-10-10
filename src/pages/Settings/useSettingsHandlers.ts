import { SettingsState, normalizeDataDirDisplay } from './useSettingsState';
import { request as invoke } from '../../utils/request';
import { open } from '@tauri-apps/plugin-dialog';
import { showToast } from '../../components/common/ToastContainer';
import { isTauri } from '../../utils/env';
import { exit } from '@tauri-apps/plugin-process';

export function useSettingsHandlers(s: SettingsState) {
    const { t, saveConfig, formData, setFormData, setIsClearLogsOpen, dataDirPath, setDataDirPath, pendingDataDir, setPendingDataDir, setIsMigrateDataDirOpen, isMigratingDataDir, setIsMigratingDataDir, setIsClearCacheOpen, setCachePaths, setIsClearingCache, setIsCheckingUpdate, setUpdateInfo, setIsInstallerUpdating, setIsBrewUpgrading, setIsBrewConfirmOpen, setIsBrewSuccessOpen } = s;

    const handleSave = async () => {
        try {
            // Validation: prompt if upstream proxy is enabled without an address
            const proxyEnabled = formData.proxy?.upstream_proxy?.enabled;
            const proxyUrl = formData.proxy?.upstream_proxy?.url?.trim();
            if (proxyEnabled && !proxyUrl) {
                showToast(t('proxy.config.upstream_proxy.validation_error'), 'error');
                return;
            }

            await saveConfig(formData);
            showToast(t('common.saved'), 'success');

            // If proxy configuration changed, prompt user that restart may be required
            if (proxyEnabled && proxyUrl) {
                showToast(t('proxy.config.upstream_proxy.restart_hint'), 'info');
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const confirmClearLogs = async () => {
        try {
            await invoke('clear_log_cache');
            showToast(t('settings.advanced.logs_cleared'), 'success');
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
        setIsClearLogsOpen(false);
    };

    const handleOpenDataDir = async () => {
        try {
            await invoke('open_data_folder');
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleSelectDataDir = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t('settings.advanced.data_dir_select'),
            });
            if (!selected || typeof selected !== 'string') {
                return;
            }
            if (selected === dataDirPath) {
                return;
            }
            setPendingDataDir(selected);
            setIsMigrateDataDirOpen(true);
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const confirmMigrateDataDir = async () => {
        if (!pendingDataDir || isMigratingDataDir) {
            return;
        }
        setIsMigratingDataDir(true);
        try {
            const newPath = await invoke<string>('set_data_dir', { path: pendingDataDir });
            setDataDirPath(normalizeDataDirDisplay(newPath));
            setIsMigrateDataDirOpen(false);
            setPendingDataDir('');
            showToast(t('settings.advanced.data_dir_migrated'), 'success');
            showToast(t('settings.advanced.data_dir_restart_hint'), 'info');
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        } finally {
            setIsMigratingDataDir(false);
        }
    };

    const handleSelectExportPath = async () => {
        try {
            // @ts-ignore
            const selected = await open({
                directory: true,
                multiple: false,
                title: t('settings.advanced.export_path'),
            });
            if (selected && typeof selected === 'string') {
                setFormData({ ...formData, default_export_path: selected });
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleSelectAntigravityPath = async () => {
        try {
            const selected = await open({
                directory: false,
                multiple: false,
                title: t('settings.advanced.antigravity_path_select'),
            });
            if (selected && typeof selected === 'string') {
                setFormData({ ...formData, antigravity_executable: selected });
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleSelectAntigravityIdePath = async () => {
        try {
            const selected = await open({
                directory: false,
                multiple: false,
                title: t('settings.advanced.antigravity_ide_path_select', 'Select Antigravity IDE Executable'),
            });
            if (selected && typeof selected === 'string') {
                setFormData({ ...formData, antigravity_ide_executable: selected });
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleSelectDebugLogDir = async () => {
        try {
            const selected = await open({
                directory: true,
                multiple: false,
                title: t('settings.advanced.debug_log_dir_select'),
            });
            if (selected && typeof selected === 'string') {
                setFormData({
                    ...formData,
                    proxy: {
                        ...formData.proxy,
                        debug_logging: {
                            enabled: formData.proxy?.debug_logging?.enabled ?? false,
                            output_dir: selected,
                        },
                    },
                });
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleDetectAntigravityPath = async () => {
        try {
            const command = isTauri() ? 'get_antigravity_path' : 'get_antigravity_path'; // Backend unified
            const path = await invoke<string>(command, { bypassConfig: true });
            setFormData({ ...formData, antigravity_executable: path });
            showToast(t('settings.advanced.antigravity_path_detected'), 'success');
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleSelectAntigravityCliPath = async () => {
        try {
            const selected = await open({
                directory: false,
                multiple: false,
                title: t('settings.advanced.antigravity_cli_path_select', 'Select Antigravity CLI (agy) Executable'),
            });
            if (selected && typeof selected === 'string') {
                setFormData({ ...formData, antigravity_cli_executable: selected });
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleDetectAntigravityCliPath = async () => {
        try {
            const path = await invoke<string>('get_antigravity_cli_path', { bypassConfig: true });
            setFormData({ ...formData, antigravity_cli_executable: path });
            showToast(t('settings.advanced.antigravity_cli_path_detected', 'Detected CLI path updated'), 'success');
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        }
    };

    const handleCheckUpdate = async () => {
        setIsCheckingUpdate(true);
        setUpdateInfo(null);
        try {
            const result = await invoke<{
                has_update: boolean;
                latest_version: string;
                current_version: string;
                download_url: string;
                source?: string;
                channel?: 'stable' | 'beta';
            }>('check_update_via_script').catch(() => invoke<{
                has_update: boolean;
                latest_version: string;
                current_version: string;
                download_url: string;
                source?: string;
                channel?: 'stable' | 'beta';
            }>('check_for_updates'));

            setUpdateInfo({
                hasUpdate: result.has_update,
                latestVersion: result.latest_version,
                currentVersion: result.current_version,
                downloadUrl: result.download_url,
                source: result.source,
                channel: result.channel,
            });

            if (result.has_update) {
                const sourceMsg = result.source && result.source !== 'GitHub API' ? ` (via ${result.source})` : '';
                showToast(t('settings.about.new_version_available', { version: result.latest_version }) + sourceMsg, 'info');
            } else {
                showToast(t('settings.about.latest_version'), 'success');
            }
        } catch (error) {
            showToast(`${t('settings.about.update_check_failed')}: ${error}`, 'error');
        } finally {
            setIsCheckingUpdate(false);
        }
    };

    const saveUpdateSettingsHelper = async (overrides: {
        auto_check?: boolean;
        check_interval_hours?: number;
        notify_on_update?: boolean;
        notify_via_email?: boolean;
        notify_via_telegram?: boolean;
    }) => {
        const payload = {
            auto_check: overrides.auto_check !== undefined ? overrides.auto_check : (formData.auto_check_update ?? true),
            last_check_time: 0,
            check_interval_hours: overrides.check_interval_hours !== undefined ? overrides.check_interval_hours : (formData.update_check_interval ?? 24),
            notify_on_update: overrides.notify_on_update !== undefined ? overrides.notify_on_update : (formData.notify_on_update ?? true),
            notify_via_email: overrides.notify_via_email !== undefined ? overrides.notify_via_email : (formData.notify_via_email ?? true),
            notify_via_telegram: overrides.notify_via_telegram !== undefined ? overrides.notify_via_telegram : (formData.notify_via_telegram ?? true),
            last_known_version: '',
        };
        await invoke('save_update_settings', { settings: payload });
    };

    const handleRunInstallerUpdate = async () => {
        setIsInstallerUpdating(true);
        try {
            await invoke<string>('run_installer_update');
            showToast(t('settings.about.installer_success', 'Delegated updater started! Closing application so the update can complete cleanly...'), 'success');
            setTimeout(async () => {
                try {
                    await exit(0);
                } catch {
                    try {
                        window.close();
                    } catch {
                        // ignore
                    }
                }
            }, 1200);
        } catch (error) {
            showToast(`${t('settings.about.installer_failed', 'Installer update failed')}: ${error}`, 'error');
        } finally {
            setIsInstallerUpdating(false);
        }
    };

    const handleBrewUpgrade = async () => {
        setIsBrewConfirmOpen(false);
        setIsBrewUpgrading(true);
        try {
            await invoke<string>('brew_upgrade_cask');
            setUpdateInfo(null);
            setIsBrewSuccessOpen(true);
        } catch (error) {
            const errKey = String(error);
            const errMsg = t(`settings.about.brew_error_${errKey}`, t('settings.about.brew_upgrade_failed'));
            showToast(errMsg, 'error');
        } finally {
            setIsBrewUpgrading(false);
        }
    };

    // Handle opening cache clear dialog
    const handleOpenClearCacheDialog = async () => {
        try {
            const paths = await invoke<string[]>('get_antigravity_cache_paths');
            setCachePaths(paths);
            setIsClearCacheOpen(true);
        } catch (error) {
            // If no cache paths found, still allow opening the dialog
            setCachePaths([]);
            setIsClearCacheOpen(true);
        }
    };

    // Handle clearing Antigravity cache
    const confirmClearAntigravityCache = async () => {
        setIsClearingCache(true);
        try {
            const result = await invoke<{
                cleared_paths: string[];
                total_size_freed: number;
                errors: string[];
            }>('clear_antigravity_cache');

            const sizeMB = (result.total_size_freed / 1024 / 1024).toFixed(2);

            if (result.cleared_paths.length > 0) {
                showToast(t('settings.advanced.cache_cleared_success', { size: sizeMB }), 'success');
            } else if (result.errors.length > 0) {
                showToast(`${t('common.error')}: ${result.errors[0]}`, 'error');
            } else {
                showToast(t('settings.advanced.cache_not_found'), 'info');
            }
        } catch (error) {
            showToast(`${t('common.error')}: ${error}`, 'error');
        } finally {
            setIsClearingCache(false);
            setIsClearCacheOpen(false);
        }
    };

    return {
        handleSave,
        confirmClearLogs,
        handleOpenDataDir,
        handleSelectDataDir,
        confirmMigrateDataDir,
        handleSelectExportPath,
        handleSelectAntigravityPath,
        handleSelectAntigravityIdePath,
        handleSelectDebugLogDir,
        handleDetectAntigravityPath,
        handleSelectAntigravityCliPath,
        handleDetectAntigravityCliPath,
        handleCheckUpdate,
        saveUpdateSettingsHelper,
        handleRunInstallerUpdate,
        handleBrewUpgrade,
        handleOpenClearCacheDialog,
        confirmClearAntigravityCache,
    };
}

export type SettingsHandlers = ReturnType<typeof useSettingsHandlers>;
