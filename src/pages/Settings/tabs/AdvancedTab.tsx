import { SettingsPageApi } from '../useSettingsPage';
import { ShieldCheck, Bug, Terminal } from 'lucide-react';
import { request as invoke } from '../utils/request';
import { showToast } from '../components/common/ToastContainer';
import { isTauri } from '../utils/env';
import versionData from '../../version.json';

import { AdvancedMaintenanceSection } from './AdvancedMaintenanceSection';
export function AdvancedTab(props: SettingsPageApi) {
    const { t, enable, disable, isEnabled, setIsBackupModalOpen, formData, setFormData, setIsClearLogsOpen, dataDirPath, isMigratingDataDir, handleOpenDataDir, handleSelectDataDir, handleSelectExportPath, handleSelectAntigravityPath, handleSelectAntigravityIdePath, handleSelectDebugLogDir, handleDetectAntigravityPath, command, path, handleSelectAntigravityCliPath, handleDetectAntigravityCliPath, handleOpenClearCacheDialog } = props;

    return (
                        <>
                            <div className="space-y-4">
                                {/* Default export path */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">{t('settings.advanced.export_path')}</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={formData.default_export_path || t('settings.advanced.export_path_placeholder')}
                                            readOnly
                                        />
                                        {formData.default_export_path && (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
                                                onClick={() => setFormData({ ...formData, default_export_path: undefined })}
                                            >
                                                {t('common.clear')}
                                            </button>
                                        )}
                                        {isTauri() ? (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 hover:text-gray-900 dark:hover:text-base-content transition-colors"
                                                onClick={handleSelectExportPath}
                                            >
                                                {t('settings.advanced.select_btn')}
                                            </button>
                                        ) : (
                                            <span className="self-center text-xs text-gray-400 dark:text-gray-500 italic px-2">
                                                {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">{t('settings.advanced.default_export_path_desc')}</p>
                                </div>

                                {/* Unified Backup & Vault Card */}
                                <div className="p-4 bg-purple-50/50 dark:bg-purple-950/20 rounded-xl border border-purple-200/60 dark:border-purple-800/40 flex items-center justify-between gap-4">
                                    <div className="space-y-0.5">
                                        <div className="text-sm font-semibold text-purple-900 dark:text-purple-300 flex items-center gap-1.5">
                                            <ShieldCheck className="w-4 h-4 text-purple-600 dark:text-purple-400" />
                                            <span>Unified Backup & Encrypted Vault</span>
                                        </div>
                                        <p className="text-xs text-purple-700 dark:text-purple-300/80">
                                            Export entire environment, accounts, and configurations with AES-256-GCM encryption.
                                        </p>
                                    </div>
                                    <button
                                        type="button"
                                        onClick={() => setIsBackupModalOpen(true)}
                                        className="px-3.5 py-2 text-xs font-semibold text-white bg-purple-600 hover:bg-purple-500 rounded-lg shadow-xs transition-colors shrink-0 cursor-pointer"
                                    >
                                        Manage Backups
                                    </button>
                                </div>

                                {/* Data directory */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">{t('settings.advanced.data_dir')}</label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={dataDirPath}
                                            readOnly
                                        />
                                        {isTauri() ? (
                                            <>
                                                <button
                                                    className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 hover:text-gray-900 dark:hover:text-base-content transition-colors"
                                                    onClick={handleSelectDataDir}
                                                    disabled={isMigratingDataDir}
                                                >
                                                    {t('settings.advanced.select_btn')}
                                                </button>
                                                <button
                                                    className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 hover:text-gray-900 dark:hover:text-base-content transition-colors"
                                                    onClick={handleOpenDataDir}
                                                >
                                                    {t('settings.advanced.open_btn')}
                                                </button>
                                            </>
                                        ) : (
                                            <span className="self-center text-xs text-gray-400 dark:text-gray-500 italic px-2">
                                                {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">{t('settings.advanced.data_dir_desc')}</p>
                                </div>

                                {/* Antigravity executable path */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                        {t('settings.advanced.antigravity_path')}
                                    </label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={formData.antigravity_executable || ''}
                                            placeholder={t('settings.advanced.antigravity_path_placeholder')}
                                            onChange={(e) => setFormData({ ...formData, antigravity_executable: e.target.value })}
                                        />
                                        {formData.antigravity_executable && (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
                                                onClick={() => setFormData({ ...formData, antigravity_executable: undefined })}
                                            >
                                                {t('common.clear')}
                                            </button>
                                        )}
                                        <button
                                            className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                            onClick={handleDetectAntigravityPath}
                                        >
                                            {t('settings.advanced.detect_btn')}
                                        </button>
                                        {isTauri() ? (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                                onClick={handleSelectAntigravityPath}
                                            >
                                                {t('settings.advanced.select_btn')}
                                            </button>
                                        ) : (
                                            <span className="self-center text-xs text-gray-400 dark:text-gray-500 italic px-2">
                                                {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                                        {t('settings.advanced.antigravity_path_desc')}
                                    </p>
                                </div>

                                {/* Antigravity CLI (agy) executable path */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                        {t('settings.advanced.antigravity_cli_path', 'Antigravity CLI (agy) Path')}
                                    </label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={formData.antigravity_cli_executable || ''}
                                            placeholder={t('settings.advanced.antigravity_cli_path_placeholder', 'Not set (will auto-detect)')}
                                            onChange={(e) => setFormData({ ...formData, antigravity_cli_executable: e.target.value })}
                                        />
                                        {formData.antigravity_cli_executable && (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
                                                onClick={() => setFormData({ ...formData, antigravity_cli_executable: undefined })}
                                            >
                                                {t('common.clear')}
                                            </button>
                                        )}
                                        <button
                                            className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                            onClick={handleDetectAntigravityCliPath}
                                        >
                                            {t('settings.advanced.detect_btn')}
                                        </button>
                                        {isTauri() ? (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                                onClick={handleSelectAntigravityCliPath}
                                            >
                                                {t('settings.advanced.select_btn')}
                                            </button>
                                        ) : (
                                            <span className="self-center text-xs text-gray-400 dark:text-gray-500 italic px-2">
                                                {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                                        {t('settings.advanced.antigravity_cli_path_desc', 'Set the executable path for the command line client (agy) to bypass account restrictions.')}
                                    </p>

                                    {/* Patch account eligibility button */}
                                    <div className={`mt-3 flex items-center gap-4 p-3 rounded-lg border ${formData.antigravity_cli_executable ? 'bg-blue-50 dark:bg-blue-950/20 border-blue-100 dark:border-blue-900/30' : 'bg-gray-50 dark:bg-gray-800 border-gray-200 dark:border-gray-700'}`}>
                                        <div className="flex-1">
                                            <h4 className={`text-sm font-semibold ${formData.antigravity_cli_executable ? 'text-blue-900 dark:text-blue-200' : 'text-gray-500 dark:text-gray-400'}`}>
                                                {t('settings.advanced.patch_eligibility_title', 'Bypass Account Eligibility Check')}
                                            </h4>
                                            <p className={`text-xs mt-0.5 ${formData.antigravity_cli_executable ? 'text-blue-700 dark:text-blue-300/80' : 'text-gray-400 dark:text-gray-500'}`}>
                                                {t('settings.advanced.patch_eligibility_desc', 'Newer agy client versions enforce account checks. This operation dynamically patches it to skip checks.')}
                                                {!formData.antigravity_cli_executable && " (Requires path to be set or detected above)"}
                                            </p>
                                        </div>
                                        <button
                                            className={`px-4 py-2 rounded-lg transition-colors font-medium text-sm shadow-sm ${formData.antigravity_cli_executable ? 'bg-blue-600 hover:bg-blue-700 active:bg-blue-800 text-white' : 'bg-gray-200 dark:bg-gray-700 text-gray-400 dark:text-gray-500 cursor-not-allowed'}`}
                                            disabled={!formData.antigravity_cli_executable}
                                            onClick={async () => {
                                                if (!formData.antigravity_cli_executable) return;
                                                try {
                                                    const res = await invoke<string>('patch_agy_binary', { filePath: formData.antigravity_cli_executable });
                                                    showToast(res, 'success');
                                                } catch (err) {
                                                    showToast(String(err), 'error');
                                                }
                                            }}
                                        >
                                            {t('settings.advanced.patch_btn', 'Bypass Check')}
                                        </button>
                                    </div>
                                </div>

                                {/* Antigravity IDE executable path */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                        {t('settings.advanced.antigravity_ide_path', 'Antigravity IDE Path')}
                                    </label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={formData.antigravity_ide_executable || ''}
                                            placeholder={t('settings.advanced.antigravity_ide_path_placeholder', 'D:\\Antigravity\\Antigravity.exe')}
                                            onChange={(e) => setFormData({ ...formData, antigravity_ide_executable: e.target.value })}
                                        />
                                        {formData.antigravity_ide_executable && (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-red-600 dark:text-red-400 rounded-lg hover:bg-red-50 dark:hover:bg-red-900/10 transition-colors"
                                                onClick={() => setFormData({ ...formData, antigravity_ide_executable: undefined })}
                                            >
                                                {t('common.clear')}
                                            </button>
                                        )}
                                        {isTauri() ? (
                                            <button
                                                className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-50 dark:hover:bg-base-200 transition-colors"
                                                onClick={handleSelectAntigravityIdePath}
                                            >
                                                {t('settings.advanced.select_btn')}
                                            </button>
                                        ) : (
                                            <span className="self-center text-xs text-gray-400 dark:text-gray-500 italic px-2">
                                                {t('settings.web_mode_limitation', '(Not supported in Web mode)')}
                                            </span>
                                        )}
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                                        {t('settings.advanced.antigravity_ide_path_desc', 'Specify the executable path for Antigravity IDE (code editor). Once set, account switching will strictly protect processes at this path from being terminated.')}
                                    </p>
                                </div>

                                {/* Antigravity startup arguments */}
                                <div>
                                    <label className="block text-sm font-medium text-gray-900 dark:text-base-content mb-1">
                                        {t('settings.advanced.antigravity_args')}
                                    </label>
                                    <div className="flex gap-2">
                                        <input
                                            type="text"
                                            className="flex-1 px-4 py-4 border border-gray-200 dark:border-base-300 rounded-lg bg-gray-50 dark:bg-base-200 text-gray-900 dark:text-base-content font-medium"
                                            value={formData.antigravity_args ? formData.antigravity_args.join(' ') : ''}
                                            placeholder={t('settings.advanced.antigravity_args_placeholder')}
                                            onChange={(e) => {
                                                const args = e.target.value.trim() === '' ? [] : e.target.value.split(' ').map(arg => arg.trim()).filter(arg => arg !== '');
                                                setFormData({ ...formData, antigravity_args: args });
                                            }}
                                        />
                                        <button
                                            className="px-4 py-2 border border-gray-200 dark:border-base-300 text-gray-700 dark:text-gray-300 rounded-lg hover:bg-gray-100 dark:hover:bg-base-200 transition-colors"
                                            onClick={async () => {
                                                try {
                                                    const args = await invoke<string[]>('get_antigravity_args');
                                                    setFormData({ ...formData, antigravity_args: args });
                                                    showToast(t('settings.advanced.antigravity_args_detected'), 'success');
                                                } catch (error) {
                                                    showToast(`${t('settings.advanced.antigravity_args_detect_error')}: ${error}`, 'error');
                                                }
                                            }}
                                        >
                                            {t('settings.advanced.detect_args_btn')}
                                        </button>
                                    </div>
                                    <p className="text-sm text-gray-500 dark:text-gray-400 mt-2">
                                        {t('settings.advanced.antigravity_args_desc')}
                                    </p>
                                </div>

                                <AdvancedMaintenanceSection {...props} />


                            </div>
                        </>
    );
}
