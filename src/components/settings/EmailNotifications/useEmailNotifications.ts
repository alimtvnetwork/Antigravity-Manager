import { useEmailCore, EmailNotificationsState } from './useEmailCore';
import { useTelegramHandlers } from './useTelegramHandlers';
import { useAccountHandlers } from './useAccountHandlers';
import { useEmailUtilities } from './useEmailUtilities';

export type EmailNotificationsApi = EmailNotificationsState &
    ReturnType<typeof useTelegramHandlers> &
    ReturnType<typeof useAccountHandlers> &
    ReturnType<typeof useEmailUtilities>;

export function useEmailNotifications(): EmailNotificationsApi {
    const core = useEmailCore();
    const telegram = useTelegramHandlers(core);
    const accounts = useAccountHandlers(core);
    const utilities = useEmailUtilities(core);
    return { ...core, ...telegram, ...accounts, ...utilities };
}
