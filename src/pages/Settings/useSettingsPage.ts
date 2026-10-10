import { useSettingsState, SettingsState } from './useSettingsState';
import { useSettingsHandlers, SettingsHandlers } from './useSettingsHandlers';

export type SettingsPageApi = SettingsState & SettingsHandlers;

export function useSettingsPage(): SettingsPageApi {
    const state = useSettingsState();
    const handlers = useSettingsHandlers(state);
    return { ...state, ...handlers };
}
