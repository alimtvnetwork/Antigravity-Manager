import { Loader2 } from 'lucide-react';
import { useSupabaseSync } from './SupabaseSync/useSupabaseSync';
import { HeaderSection } from './SupabaseSync/HeaderSection';
import { GlobalSyncSection } from './SupabaseSync/GlobalSyncSection';
import { EndpointsSection } from './SupabaseSync/EndpointsSection';
import { LeasesSection } from './SupabaseSync/LeasesSection';
import { TelegramSection } from './SupabaseSync/TelegramSection';
import { SyncModalsA } from './SupabaseSync/SyncModalsA';
import { SyncModalsB } from './SupabaseSync/SyncModalsB';

export default function SupabaseSyncSettings() {
    const api = useSupabaseSync();
    const { isLoading, config } = api;

    if (isLoading) {
        return (
            <div className="flex items-center justify-center p-12 text-gray-400">
                <Loader2 className="w-6 h-6 animate-spin mr-2" />
                Loading Supabase synchronization configuration...
            </div>
        );
    }

    if (!config) return null;

    return (
        <div className="space-y-4">
            <HeaderSection {...api} />
            <GlobalSyncSection {...api} />
            <EndpointsSection {...api} />
            <LeasesSection {...api} />
            <TelegramSection {...api} />
            <SyncModalsA {...api} />
            <SyncModalsB {...api} />
        </div>
    );
}
