import { create } from 'zustand';
import { request as invoke } from '../utils/request';

export interface SkippedVersion {
  version: string;
  reason: string;
}

export interface SourceError {
  source: string;
  message: string;
}

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url: string;
  source?: string;
  proxy_url?: string;
  channel?: 'stable' | 'beta';
  updater_json_url?: string;
  /** True when latest_version was verified to have a platform installer asset (plan 86). */
  asset_verified?: boolean;
  asset_url?: string;
  /** Verified installable versions, newest first (plan 86). */
  candidates?: string[];
  /** Newer versions skipped with reasons (plan 86). */
  skipped_versions?: SkippedVersion[];
  source_errors?: SourceError[];
}

interface UpdateStoreState {
  updateInfo: UpdateInfo | null;
  isChecking: boolean;
  isInstalling: boolean;
  showNotification: boolean;
  setShowNotification: (show: boolean) => void;
  checkForUpdates: (force?: boolean) => Promise<UpdateInfo | null>;
  installUpdate: () => Promise<string | null>;
}

export const useUpdateStore = create<UpdateStoreState>((set, get) => ({
  updateInfo: null,
  isChecking: false,
  isInstalling: false,
  showNotification: false,
  setShowNotification: (show: boolean) => set({ showNotification: show }),
  checkForUpdates: async (_force = false) => {
    if (get().isChecking) return get().updateInfo;
    set({ isChecking: true });
    try {
      const info = await invoke<UpdateInfo>('check_for_updates').catch(() =>
        invoke<UpdateInfo>('check_update_via_script')
      );
      const hasValidUpdate = Boolean(
        info &&
          info.has_update &&
          info.latest_version &&
          info.latest_version !== info.current_version
      );
      const normalized = info ? { ...info, has_update: hasValidUpdate } : null;
      set({ updateInfo: normalized, isChecking: false });
      if (hasValidUpdate) {
        set({ showNotification: true });
      }
      return normalized;
    } catch (e) {
      console.warn('[useUpdateStore] update check notice:', e);
      set({ isChecking: false });
      return null;
    }
  },
  installUpdate: async () => {
    if (get().isInstalling) return null;
    set({ isInstalling: true, showNotification: true });
    try {
      const res = await invoke<string>('run_installer_update');
      set({ isInstalling: false });
      if (res) {
        setTimeout(async () => {
          try {
            const { exit } = await import('@tauri-apps/plugin-process');
            await exit(0);
          } catch {
            try {
              window.close();
            } catch {
              // ignore
            }
          }
        }, 500);
      }
      return res;
    } catch (e) {
      console.error('[useUpdateStore] installer execution notice:', e);
      set({ isInstalling: false });
      // Fallback: try opening GitHub download URL
      const info = get().updateInfo;
      if (info?.download_url) {
        try {
          const { openUrl } = await import('@tauri-apps/plugin-opener');
          await openUrl(info.download_url);
        } catch {
          window.open(info.download_url, '_blank', 'noopener,noreferrer');
        }
      }
      return null;
    }
  },
}));
