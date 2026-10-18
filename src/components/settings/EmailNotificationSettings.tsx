import { Link } from 'react-router-dom';
import { MessageSquare, ArrowRight } from 'lucide-react';
import { useEmailNotifications } from './EmailNotifications/useEmailNotifications';
import { MailboxesSection } from './EmailNotifications/MailboxesSection';
import { RecipientsSection } from './EmailNotifications/RecipientsSection';
import { WatcherSection } from './EmailNotifications/WatcherSection';
import { RemoteCommandSection } from './EmailNotifications/RemoteCommandSection';
import { SshGitmapSection } from './EmailNotifications/SshGitmapSection';
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
            {/* Telegram Remote Control Standalone Navigation Redirect */}
            <div className="rounded-2xl border border-sky-500/20 bg-sky-500/5 dark:bg-sky-950/20 p-4.5 flex flex-col sm:flex-row items-start sm:items-center justify-between gap-3.5 shadow-2xs">
                <div className="flex items-center gap-3">
                    <div className="w-10 h-10 rounded-xl bg-sky-500/10 dark:bg-sky-500/20 border border-sky-500/25 text-sky-600 dark:text-sky-400 flex items-center justify-center shrink-0">
                        <MessageSquare className="w-5 h-5" />
                    </div>
                    <div>
                        <h3 className="text-sm font-semibold text-slate-900 dark:text-slate-100 flex items-center gap-2">
                            <span>Telegram Remote Control</span>
                            <span className="px-2 py-0.5 text-[10px] font-medium rounded-full bg-sky-500/15 text-sky-600 dark:text-sky-300 border border-sky-500/30">
                                Standalone View
                            </span>
                        </h3>
                        <p className="text-xs text-slate-500 dark:text-slate-400 mt-0.5">
                            Telegram bot configuration, chat ID detection, live listener status, and remote commands have moved to dedicated navigation.
                        </p>
                    </div>
                </div>
                <Link
                    to="/telegram"
                    className="px-3.5 py-1.5 rounded-lg text-xs font-semibold bg-sky-600 hover:bg-sky-500 text-white shadow-xs flex items-center gap-1.5 transition-colors shrink-0"
                >
                    <span>Open Telegram Control</span>
                    <ArrowRight className="w-3.5 h-3.5" />
                </Link>
            </div>
            <SshGitmapSection />
            <AccountModal {...api} />
            <EmailDialogs {...api} />
        </div>
    );
}
