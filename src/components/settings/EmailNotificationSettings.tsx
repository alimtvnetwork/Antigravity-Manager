import { useEmailNotifications } from './EmailNotifications/useEmailNotifications';
import { MailboxesSection } from './EmailNotifications/MailboxesSection';
import { RecipientsSection } from './EmailNotifications/RecipientsSection';
import { WatcherSection } from './EmailNotifications/WatcherSection';
import { RemoteCommandSection } from './EmailNotifications/RemoteCommandSection';
import { TelegramSection } from './EmailNotifications/TelegramSection';
import { AccountModal } from './EmailNotifications/AccountModal';
import { EmailDialogs } from './EmailNotifications/EmailDialogs';

export default function EmailNotificationSettings() {
    const api = useEmailNotifications();

    return (
        <div className="space-y-4">
            <MailboxesSection {...api} />
            <RecipientsSection {...api} />
            <WatcherSection {...api} />
            <RemoteCommandSection {...api} />
            <TelegramSection {...api} />
            <AccountModal {...api} />
            <EmailDialogs {...api} />
        </div>
    );
}
