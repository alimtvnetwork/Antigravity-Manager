import { create } from 'zustand';
import { request as invoke } from '../utils/request';

export interface UpdateInfo {
  has_update: boolean;
  latest_version: string;
  current_version: string;
  download_url: string;
  source?: string;
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
      const info = await invoke<UpdateInfo>('check_update_via_script').catch(() =>
        invoke<UpdateInfo>('check_for_updates')
      );
      set({ updateInfo: info, isChecking: false });
      if (info && info.has_update) {
        set({ showNotification: true });
      }
      return info;
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
