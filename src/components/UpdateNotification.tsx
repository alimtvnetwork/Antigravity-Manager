import React, { useEffect, useState, useRef } from 'react';
import { X, Sparkles, Loader2, CheckCircle, RotateCcw } from 'lucide-react';
import { useTranslation } from 'react-i18next';
import { relaunch as tauriRelaunch, exit as tauriExit } from '@tauri-apps/plugin-process';
import { useUpdateStore, type UpdateInfo } from '../stores/use-update-store';
import { useErrorStore } from '../stores/error-store';

type UpdateState = 'checking' | 'downloading' | 'ready' | 'error' | 'none' | 'manual' | 'skipped';

interface UpdateNotificationProps {
  onClose: () => void;
}

export const UpdateNotification: React.FC<UpdateNotificationProps> = ({ onClose }) => {
  const { t } = useTranslation();
  const storeInfo = useUpdateStore((state) => state.updateInfo);
  const checkForUpdates = useUpdateStore((state) => state.checkForUpdates);
  const installUpdate = useUpdateStore((state) => state.installUpdate);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(storeInfo);
  const [isVisible, setIsVisible] = useState(false);
  const [isClosing, setIsClosing] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState>(storeInfo?.has_update ? 'manual' : 'checking');
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [isInstalling, setIsInstalling] = useState(false);
  const [showSkippedDetails, setShowSkippedDetails] = useState(false);
  const downloadStarted = useRef(false);

  useEffect(() => {
    if (storeInfo?.has_update) {
      setUpdateInfo(storeInfo);
      setUpdateState('manual');
      setTimeout(() => setIsVisible(true), 50);
    } else if (storeInfo?.skipped_versions && storeInfo.skipped_versions.length > 0) {
      // Newer versions exist but none is installable: tell the user (plan 86).
      setUpdateInfo(storeInfo);
      setUpdateState('skipped');
      setTimeout(() => setIsVisible(true), 50);
    } else {
      checkAndDownload();
    }
  }, [storeInfo]);

  const checkAndDownload = async () => {
    try {
      const info = await checkForUpdates(true);
      if (!info) {
        onClose();
        return;
      }
      if (!info.has_update) {
        if (info.skipped_versions && info.skipped_versions.length > 0) {
          setUpdateInfo(info);
          setUpdateState('skipped');
          setTimeout(() => setIsVisible(true), 100);
          return;
        }
        onClose();
        return;
      }

      setUpdateInfo(info);
      setUpdateState('manual');
      setTimeout(() => setIsVisible(true), 100);

    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error('Update check failed:', errorMsg);
      // Tracked in the error module; dialog closes as the visible signal, no duplicate modal.
      useErrorStore.getState().trackWarning(error, {
        source: 'UpdateNotification.checkForUpdates',
        triggerAction: 'check_for_updates',
      });
      onClose();
    }
  };

  const handleRunInstaller = async () => {
    setIsInstalling(true);
    try {
      const res = await installUpdate();
      if (res) {
        setUpdateState('ready');
        setTimeout(async () => {
          try {
            await tauriExit(0);
          } catch {
            try {
              window.close();
            } catch {
              // ignore
            }
          }
        }, 500);
      } else {
        setUpdateState('manual');
      }
    } catch (error) {
      console.error('Installer update failed:', error);
      // Tracked in the error module; dialog already shows the error state to the user.
      useErrorStore.getState().trackWarning(error, {
        source: 'UpdateNotification.handleRunInstaller',
        triggerAction: 'install_update',
      });
      setUpdateState('error');
    } finally {
      setIsInstalling(false);
    }
  };

  const handleRestart = async () => {
    try {
      await tauriExit(0);
    } catch {
      try {
        await tauriRelaunch();
      } catch (error) {
        console.error('Relaunch failed:', error);
        // Tracked in the error module; both relaunch paths failed, user must relaunch manually.
        useErrorStore.getState().trackWarning(error, {
          source: 'UpdateNotification.handleRestart',
          triggerAction: 'relaunch_app',
        });
      }
    }
  };

  const handleClose = () => {
    setIsClosing(true);
    setIsVisible(false);
    setTimeout(onClose, 400);
  };

  if (updateState === 'none' || (updateState === 'checking' && !updateInfo?.has_update)) {
    return null;
  }

  return (
    <div
      className={`
        fixed bottom-6 left-6 z-[100]
        transition-all duration-400 ease-[cubic-bezier(0.34,1.56,0.64,1)]
        ${isVisible && !isClosing ? 'translate-y-0 opacity-100 scale-100' : 'translate-y-4 opacity-0 scale-95'}
      `}
    >
      <div
        data-ui-name="update-notification"
        data-ui-id="update-notification-card"
        className="
        relative overflow-hidden
        w-80 p-3.5
        rounded-2xl
        border border-slate-300 dark:border-purple-500/30
        shadow-[0_12px_40px_rgba(0,0,0,0.18)] dark:shadow-[0_12px_40px_rgba(124,58,237,0.18)]
        backdrop-blur-xl
        bg-white/95 dark:bg-[#120a1f]/95
        group
      ">
        <div className="absolute -top-8 -right-8 w-24 h-24 bg-purple-500/20 dark:bg-[#1af18d]/15 rounded-full blur-2xl pointer-events-none transition-colors duration-500" />
        <div className="absolute -bottom-8 -left-8 w-24 h-24 bg-[#1af18d]/15 dark:bg-purple-600/15 rounded-full blur-2xl pointer-events-none transition-colors duration-500" />

        <div className="relative z-10">
          <div className="flex items-start justify-between mb-2">
            <div className="flex items-center gap-2">
              <div className="p-1.5 rounded-xl bg-gradient-to-br from-emerald-400 to-[#1af18d] shadow-[0_0_10px_rgba(26,241,141,0.35)] shrink-0">
                {updateState === 'ready' ? (
                  <CheckCircle className="w-3.5 h-3.5 text-slate-950" />
                ) : (
                  <Sparkles className="w-3.5 h-3.5 text-slate-950" />
                )}
              </div>
              <div>
                <h3 className="font-semibold text-xs text-gray-900 dark:text-white leading-tight">
                  {updateState === 'ready'
                    ? t('update_notification.ready')
                    : updateState === 'manual'
                      ? t('update_notification.available_title', 'Update Available')
                      : updateState === 'skipped'
                        ? t('update_notification.no_installable_title', 'No installable update')
                        : updateState === 'error'
                          ? t('update_notification.toast.failed')
                          : updateState === 'downloading'
                            ? t('update_notification.title')
                            : t('update_notification.checking_title', 'Checking for updates...')}
                </h3>
                {updateInfo && (
                  <div className="flex items-center gap-1.5 mt-1">
                    <span className="font-mono text-[10px] font-bold text-emerald-600 dark:text-[#1af18d] bg-emerald-500/10 dark:bg-[#1af18d]/10 px-2 py-0.5 rounded-full border border-emerald-500/20 dark:border-[#1af18d]/30 shadow-2xs">
                      v{updateInfo.latest_version}
                    </span>
                    {updateInfo.channel === 'beta' && (
                      <span className="text-[9px] font-bold uppercase tracking-wider px-2 py-0.5 rounded-full bg-amber-500/15 text-amber-600 dark:text-amber-400 border border-amber-500/30">
                        Beta
                      </span>
                    )}
                  </div>
                )}
              </div>
            </div>

            {(updateState === 'error' || updateState === 'ready' || updateState === 'manual' || updateState === 'skipped') && (
              <button
                type="button"
                data-ui-name="update-close-btn"
                data-ui-id="update-notification-close"
                onClick={handleClose}
                className="
                  p-1 rounded-full
                  text-gray-400 hover:text-gray-600 dark:text-gray-400 dark:hover:text-white
                  hover:bg-black/5 dark:hover:bg-white/10
                  transition-all duration-200 cursor-pointer
                "
                aria-label={t('common.cancel')}
              >
                <X className="w-3.5 h-3.5" />
              </button>
            )}
          </div>

          {/* Status message */}
          <div className="mb-2.5">
            <p className="text-[11px] text-gray-600 dark:text-gray-300 leading-snug">
              {updateState === 'downloading' && t('update_notification.downloading')}
              {updateState === 'ready' && t('update_notification.restart_prompt')}
              {updateState === 'error' && `${t('update_notification.toast.failed')}`}
              {updateState === 'manual' && (
                isInstalling
                  ? t('update_notification.installing_desc', 'Installing update in background...')
                  : t('update_notification.installer_available', 'A newer version is available. Upgrade now?')
              )}
              {updateState === 'skipped' &&
                t(
                  'update_notification.no_installable_desc',
                  'A newer version was released, but it has no installer for this platform yet.'
                )}
            </p>
          </div>

          {/* Skipped versions notice (plan 86): versions that were not offered, with reasons */}
          {updateInfo?.skipped_versions && updateInfo.skipped_versions.length > 0 && (
            <div className="mb-2.5 rounded-xl border border-amber-500/30 bg-amber-500/10 px-2.5 py-2">
              <div className="flex items-center justify-between gap-2">
                <p className="text-[11px] font-medium text-amber-700 dark:text-amber-300 leading-snug">
                  {t('update_notification.skipped_notice', {
                    defaultValue: 'v{{version}} skipped: {{reason}}',
                    version: updateInfo.skipped_versions[0].version,
                    reason: updateInfo.skipped_versions[0].reason,
                  })}
                </p>
                <button
                  type="button"
                  data-ui-name="update-skipped-details-btn"
                  data-ui-id="update-notification-skipped-details"
                  onClick={() => setShowSkippedDetails((v) => !v)}
                  className="shrink-0 text-[10px] font-semibold text-amber-700 dark:text-amber-300 underline underline-offset-2 cursor-pointer"
                >
                  {showSkippedDetails
                    ? t('common.hide', 'Hide')
                    : t('common.details', 'Details')}
                </button>
              </div>
              {showSkippedDetails && (
                <div className="mt-1.5 space-y-1">
                  {updateInfo.skipped_versions.map((s) => (
                    <p key={s.version} className="text-[10px] text-amber-700/90 dark:text-amber-200/80 leading-snug">
                      v{s.version}: {s.reason}
                    </p>
                  ))}
                  {updateInfo.candidates && updateInfo.candidates.length > 0 && (
                    <p className="text-[10px] text-gray-600 dark:text-gray-300 leading-snug pt-0.5">
                      {t('update_notification.verified_candidates', 'Verified installable:')}{' '}
                      {updateInfo.candidates.map((v) => `v${v}`).join(', ')}
                    </p>
                  )}
                </div>
              )}
            </div>
          )}

          {/* Progress bar during download */}
          {updateState === 'downloading' && (
            <div className="mb-2">
              <div className="w-full bg-gray-200 dark:bg-gray-700/80 rounded-full h-1.5 overflow-hidden">
                <div
                  className="bg-gradient-to-r from-emerald-500 to-[#1af18d] h-1.5 rounded-full transition-all duration-300 shadow-[0_0_8px_rgba(26,241,141,0.5)]"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
              <div className="flex items-center justify-between mt-1">
                <p className="text-[10px] text-gray-500 font-mono">{downloadProgress}%</p>
                <Loader2 className="w-3 h-3 animate-spin text-emerald-500" />
              </div>
            </div>
          )}

          {/* Restart button when ready */}
          {updateState === 'ready' && (
            <div className="flex gap-1.5">
              <button
                type="button"
                data-ui-name="update-restart-btn"
                data-ui-id="update-notification-restart"
                onClick={handleRestart}
                className="
                  flex-1
                  bg-gradient-to-r from-emerald-500 to-teal-400 hover:from-emerald-400 hover:to-teal-300
                  text-slate-950 font-semibold text-xs
                  py-1.5 px-3 rounded-full
                  shadow-[0_0_12px_rgba(26,241,141,0.25)]
                  transition-all duration-200
                  flex items-center justify-center gap-1.5
                  active:scale-[0.98] cursor-pointer
                "
              >
                <RotateCcw className="w-3 h-3" />
                <span>{t('update_notification.btn_restart')}</span>
              </button>
              <button
                type="button"
                data-ui-name="update-later-btn"
                data-ui-id="update-notification-later"
                onClick={handleClose}
                className="
                  px-3 py-1.5 rounded-full
                  border border-slate-300 dark:border-slate-700/60
                  bg-slate-100 hover:bg-slate-200 dark:bg-slate-800/60 dark:hover:bg-slate-700/60
                  text-slate-600 hover:text-slate-800 dark:text-slate-300 dark:hover:text-white
                  transition-all duration-200
                  text-xs font-medium cursor-pointer
                "
              >
                {t('update_notification.btn_later')}
              </button>
            </div>
          )}

          {/* Installer / Manual download button */}
          {updateState === 'manual' && (
            <div className="flex flex-col gap-1.5">
              <div className="flex gap-1.5">
                <button
                  type="button"
                  data-ui-name="update-install-btn"
                  data-ui-id="update-notification-install"
                  onClick={handleRunInstaller}
                  disabled={isInstalling}
                  className="
                    flex-1
                    bg-gradient-to-r from-emerald-500 to-teal-400 hover:from-emerald-400 hover:to-teal-300
                    disabled:from-emerald-500/40 disabled:to-teal-400/40 disabled:cursor-not-allowed
                    text-slate-950 font-semibold
                    py-1.5 px-3 rounded-full
                    shadow-[0_0_12px_rgba(26,241,141,0.25)]
                    transition-all duration-200
                    flex items-center justify-center gap-1.5
                    active:scale-[0.98] cursor-pointer text-xs
                  "
                >
                  {isInstalling ? (
                    <>
                      <Loader2 className="w-3 h-3 animate-spin" />
                      <span>{t('update_notification.installing', 'Installing...')}</span>
                    </>
                  ) : (
                    <>
                      <Sparkles className="w-3 h-3" />
                      <span>{t('update_notification.btn_install_now', 'Install Now')}</span>
                    </>
                  )}
                </button>
                <button
                  type="button"
                  data-ui-name="update-later-btn"
                  data-ui-id="update-notification-later"
                  onClick={handleClose}
                  disabled={isInstalling}
                  className="
                    px-3 py-1.5 rounded-full
                    bg-slate-100 hover:bg-slate-200 dark:bg-slate-800/60 dark:hover:bg-slate-700/60
                    border border-slate-300 dark:border-slate-700/60
                    text-slate-700 hover:text-slate-900 dark:text-slate-300 dark:hover:text-white
                    transition-all duration-200
                    text-xs font-medium cursor-pointer disabled:opacity-50
                  "
                >
                  {t('update_notification.btn_later')}
                </button>
              </div>
              <button
                type="button"
                data-ui-name="update-github-link"
                data-ui-id="update-notification-github"
                onClick={async () => {
                  if (updateInfo) {
                    try {
                      const { openUrl } = await import('@tauri-apps/plugin-opener');
                      await openUrl(updateInfo.download_url);
                    } catch {
                      window.open(updateInfo.download_url, '_blank', 'noopener,noreferrer');
                    }
                  }
                }}
                disabled={isInstalling}
                className="text-[10px] font-medium text-emerald-600 hover:text-emerald-500 dark:text-[#1af18d]/90 dark:hover:text-[#1af18d] text-center transition-colors py-0.5 cursor-pointer"
              >
                {t('update_notification.btn_view_github', 'View on GitHub')}
              </button>
            </div>
          )}

          {/* Skipped state — no Install action (plan 86): nothing installable to offer */}
          {updateState === 'skipped' && (
            <button
              type="button"
              data-ui-name="update-skipped-later-btn"
              data-ui-id="update-notification-skipped-later"
              onClick={handleClose}
              className="
                w-full px-3 py-1.5 rounded-full
                border border-slate-300 dark:border-slate-700/60
                bg-slate-100 hover:bg-slate-200 dark:bg-slate-800/60 dark:hover:bg-slate-700/60
                text-slate-600 hover:text-slate-800 dark:text-slate-300 dark:hover:text-white
                transition-all duration-200
                text-xs font-medium cursor-pointer
              "
            >
              {t('update_notification.btn_later')}
            </button>
          )}

          {/* Error state — retry button */}
          {updateState === 'error' && (
            <button
              type="button"
              data-ui-name="update-retry-btn"
              data-ui-id="update-notification-retry"
              onClick={() => {
                downloadStarted.current = false;
                setUpdateState('checking');
                setDownloadProgress(0);
                checkAndDownload();
              }}
              className="
                w-full
                bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500
                text-white font-medium
                py-1.5 px-3 rounded-full
                shadow-[0_0_12px_rgba(124,58,237,0.25)]
                transition-all duration-200
                flex items-center justify-center gap-1.5 text-xs
                active:scale-[0.98] cursor-pointer
              "
            >
              <RotateCcw className="w-3.5 h-3.5" />
              <span>{t('common.retry')}</span>
            </button>
          )}
        </div>
      </div>
    </div>
  );
};
