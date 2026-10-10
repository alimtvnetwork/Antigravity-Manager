import { useSupabaseSyncState, SupabaseSyncState } from './useSupabaseSyncState';
import { useSupabaseSyncHandlers } from './useSupabaseSyncHandlers';

export type SupabaseSyncApi = SupabaseSyncState & ReturnType<typeof useSupabaseSyncHandlers>;

export function useSupabaseSync(): SupabaseSyncApi {
    const state = useSupabaseSyncState();
    const handlers = useSupabaseSyncHandlers(state);
    return { ...state, ...handlers };
}
