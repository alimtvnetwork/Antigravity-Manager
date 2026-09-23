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
  showNotification: boolean;
  setShowNotification: (show: boolean) => void;
  checkForUpdates: (force?: boolean) => Promise<UpdateInfo | null>;
}

export const useUpdateStore = create<UpdateStoreState>((set, get) => ({
  updateInfo: null,
  isChecking: false,
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
}));
