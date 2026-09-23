import React, { useEffect, useState, useRef } from 'react';
import { X, Sparkles, Loader2, CheckCircle, RotateCcw } from 'lucide-react';
import { request as invoke } from '../utils/request';
import { useTranslation } from 'react-i18next';
import { relaunch as tauriRelaunch } from '@tauri-apps/plugin-process';
import { useUpdateStore } from '../stores/use-update-store';

interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url: string;
  source?: string;
}

type UpdateState = 'checking' | 'downloading' | 'ready' | 'error' | 'none' | 'manual';

interface UpdateNotificationProps {
  onClose: () => void;
}

export const UpdateNotification: React.FC<UpdateNotificationProps> = ({ onClose }) => {
  const { t } = useTranslation();
  const storeInfo = useUpdateStore((state) => state.updateInfo);
  const checkForUpdates = useUpdateStore((state) => state.checkForUpdates);
  const [updateInfo, setUpdateInfo] = useState<UpdateInfo | null>(storeInfo);
  const [isVisible, setIsVisible] = useState(false);
  const [isClosing, setIsClosing] = useState(false);
  const [updateState, setUpdateState] = useState<UpdateState>(storeInfo?.has_update ? 'manual' : 'checking');
  const [downloadProgress, setDownloadProgress] = useState(0);
  const [isInstalling, setIsInstalling] = useState(false);
  const downloadStarted = useRef(false);

  useEffect(() => {
    if (storeInfo?.has_update) {
      setUpdateInfo(storeInfo);
      setUpdateState('manual');
      setTimeout(() => setIsVisible(true), 50);
    } else {
      checkAndDownload();
    }
  }, [storeInfo]);

  const checkAndDownload = async () => {
    try {
      const info = await checkForUpdates(true);
      if (!info || !info.has_update) {
        onClose();
        return;
      }

      setUpdateInfo(info);
      setUpdateState('manual');
      setTimeout(() => setIsVisible(true), 100);
    } catch (error) {
      const errorMsg = error instanceof Error ? error.message : String(error);
      console.error('Update check failed:', errorMsg);
      onClose();
    }
  };

  const handleRunInstaller = async () => {
    setIsInstalling(true);
    try {
      await invoke('run_installer_update');
      setUpdateState('ready');
    } catch (error) {
      console.error('Installer update failed:', error);
      setUpdateState('error');
    } finally {
      setIsInstalling(false);
    }
  };

  const handleRestart = async () => {
    try {
      await tauriRelaunch();
    } catch (error) {
      console.error('Relaunch failed:', error);
    }
  };

  const handleClose = () => {
    setIsClosing(true);
    setIsVisible(false);
    setTimeout(onClose, 400);
  };

  if (updateState === 'none') {
    return null;
  }

  return (
    <div
      className={`
        fixed bottom-6 right-6 z-[100]
        transition-all duration-400 ease-[cubic-bezier(0.34,1.56,0.64,1)]
        ${isVisible && !isClosing ? 'translate-y-0 opacity-100 scale-100' : 'translate-y-4 opacity-0 scale-95'}
      `}
    >
      <div className="
        relative overflow-hidden
        w-64 p-2.5
        rounded-xl
        border border-white/30 dark:border-white/10
        shadow-[0_8px_30px_rgb(0,0,0,0.12)] dark:shadow-[0_8px_30px_rgb(0,0,0,0.3)]
        backdrop-blur-xl
        bg-white/90 dark:bg-slate-900/90
        group
      ">
        <div className="absolute -top-8 -right-8 w-20 h-20 bg-blue-500/15 rounded-full blur-xl pointer-events-none group-hover:bg-blue-500/25 transition-colors duration-500" />
        <div className="absolute -bottom-8 -left-8 w-20 h-20 bg-purple-500/15 rounded-full blur-xl pointer-events-none group-hover:bg-purple-500/25 transition-colors duration-500" />

        <div className="relative z-10">
          <div className="flex items-start justify-between mb-1.5">
            <div className="flex items-center gap-1.5">
              <div className="p-1 rounded-md bg-gradient-to-br from-blue-500 to-purple-600 shadow-xs">
                {updateState === 'ready' ? (
                  <CheckCircle className="w-3 h-3 text-white" />
                ) : (
                  <Sparkles className="w-3 h-3 text-white" />
                )}
              </div>
              <div>
                <h3 className="font-semibold text-xs text-gray-800 dark:text-white leading-tight">
                  {updateState === 'ready'
                    ? t('update_notification.ready')
                    : t('update_notification.title')}
                </h3>
                {updateInfo && (
                  <p className="text-[10px] font-medium text-blue-600 dark:text-blue-400">
                    v{updateInfo.latest_version}
                  </p>
                )}
              </div>
            </div>

            {(updateState === 'error' || updateState === 'ready' || updateState === 'manual') && (
              <button
                onClick={handleClose}
                className="
                  p-0.5 rounded-full
                  text-gray-400 hover:text-gray-600 dark:text-gray-500 dark:hover:text-gray-300
                  hover:bg-black/5 dark:hover:bg-white/10
                  transition-all duration-200
                "
                aria-label={t('common.cancel')}
              >
                <X className="w-3 h-3" />
              </button>
            )}
          </div>

          {/* Status message */}
          <div className="mb-2">
            <p className="text-[11px] text-gray-600 dark:text-gray-300 leading-snug">
              {updateState === 'downloading' && t('update_notification.downloading')}
              {updateState === 'ready' && t('update_notification.restart_prompt')}
              {updateState === 'error' && `${t('update_notification.toast.failed')}`}
              {updateState === 'manual' && (
                isInstalling
                  ? t('update_notification.installing_desc', 'Installing update in background...')
                  : t('update_notification.installer_available', 'A newer version is available. Upgrade now?')
              )}
            </p>
          </div>

          {/* Progress bar during download */}
          {updateState === 'downloading' && (
            <div className="mb-2">
              <div className="w-full bg-gray-200 dark:bg-gray-700 rounded-full h-1.5">
                <div
                  className="bg-gradient-to-r from-blue-500 to-purple-600 h-1.5 rounded-full transition-all duration-300"
                  style={{ width: `${downloadProgress}%` }}
                />
              </div>
              <div className="flex items-center justify-between mt-1">
                <p className="text-[10px] text-gray-500">{downloadProgress}%</p>
                <Loader2 className="w-3 h-3 animate-spin text-blue-500" />
              </div>
            </div>
          )}

          {/* Restart button when ready */}
          {updateState === 'ready' && (
            <div className="flex gap-1.5">
              <button
                onClick={handleRestart}
                className="
                  flex-1 group/btn
                  relative overflow-hidden
                  bg-gradient-to-r from-green-600 to-emerald-600 hover:from-green-500 hover:to-emerald-500
                  text-white font-medium text-xs
                  py-1 px-2.5 rounded-lg
                  shadow-xs shadow-green-500/20
                  transition-all duration-300
                  flex items-center justify-center gap-1.5
                  active:scale-[0.98] cursor-pointer
                "
              >
                <RotateCcw className="w-3 h-3" />
                <span>{t('update_notification.btn_restart')}</span>
              </button>
              <button
                onClick={handleClose}
                className="
                  px-2 py-1 rounded-lg
                  text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200
                  hover:bg-black/5 dark:hover:bg-white/10
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
            <div className="flex flex-col gap-1">
              <div className="flex gap-1.5">
                <button
                  onClick={handleRunInstaller}
                  disabled={isInstalling}
                  className="
                    flex-1 group/btn
                    relative overflow-hidden
                    bg-gradient-to-r from-blue-600 to-purple-600 hover:from-blue-500 hover:to-purple-500
                    disabled:from-blue-400 disabled:to-purple-400 disabled:cursor-not-allowed
                    text-white font-medium
                    py-1 px-2.5 rounded-lg
                    shadow-xs shadow-blue-500/20
                    transition-all duration-300
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
                  onClick={handleClose}
                  disabled={isInstalling}
                  className="
                    px-2 py-1 rounded-lg
                    text-gray-500 hover:text-gray-700 dark:text-gray-400 dark:hover:text-gray-200
                    hover:bg-black/5 dark:hover:bg-white/10
                    transition-all duration-200
                    text-xs font-medium cursor-pointer
                  "
                >
                  {t('update_notification.btn_later')}
                </button>
              </div>
              <button
                type="button"
                onClick={async () => {
                  if (updateInfo) {
                    try {
                      const { openUrl } = await import('@tauri-apps/plugin-opener');
                      await openUrl(updateInfo.download_url);
                    } catch {
                      window.open(updateInfo.download_url, '_blank');
                    }
                  }
                }}
                disabled={isInstalling}
                className="text-[10px] text-gray-400 hover:text-blue-500 dark:text-gray-500 dark:hover:text-blue-400 text-center transition-colors py-0.5 cursor-pointer"
              >
                {t('update_notification.btn_view_github', 'View on GitHub')}
              </button>
            </div>
          )}

          {/* Error state — retry button */}
          {updateState === 'error' && (
            <button
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
                py-1.5 px-3 rounded-lg
                shadow-xs shadow-blue-500/25
                transition-all duration-300
                flex items-center justify-center gap-1.5 text-xs
                active:scale-[0.98]
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

