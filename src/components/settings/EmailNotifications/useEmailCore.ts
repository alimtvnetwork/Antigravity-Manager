import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';

export function useEmailCore() {
    const [accounts, setAccounts] = useState<EmailAccount[]>([]);
    const [recipients, setRecipients] = useState<NotifyRecipient[]>([]);
    const [settings, setSettings] = useState<ISettings>({
        id: 'global',
        is_enabled: false,
        polling_interval_minutes: 3,
        inbox_check_interval_minutes: 1,
        baseline_polling_interval_minutes: 5,
        active_awaiting_interval_seconds: 10,
        notify_on_quota_drop: true,
        quota_drop_threshold_percent: 25,
        notify_on_workspace_switch: true,
        notify_on_idle_workspace: true,
        notify_on_system_update: true,
        allow_remote_prompt_execution: true,
        allow_remote_cli_execution: true,
        allow_remote_instance_rotation: true,
        local_machine_name: '',
        local_machine_ip: '',
        updated_at: 0,
    });
    const [isSaving, setIsSaving] = useState(false);
    const [isPinging, setIsPinging] = useState(false);

    const HELP_COMMAND_PAYLOAD = `================================================================================
ANTIGRAVITY-MANAGER EMAIL COMMAND MANUAL & SYNTAX GUIDE
================================================================================
1. EMAIL SUBJECT FORMAT & TARGET ROUTING:
   sub: [worker-name|ip] | [ins-{instance}] | <command> [ | proj-{project name} ]

   Wildcard / Broadcast Target:
   * | <command> (dispatches to all matching worker nodes/instances)

   Examples:
   • * | help
   • * | prompt | proj-Antigravity-Manager
   • * | ps | Get-Process
   • * | cmd | dir /b
   • * | gitmap | status
   • * | agm | status
   • * | status

2. AVAILABLE INBOUND COMMANDS:
   • help
     Returns this comprehensive command manual & syntax guide.
   • prompt: Injects prompt into workspace.
     Email Subject: * | prompt | proj-<ProjectName>
     Email Body format:
       prompt-name: <Optional Prompt Name>
       prompt instruction:
       <Your Multi-line AI Instructions Here>
   • powershell / ps: Executes Windows PowerShell commands/scripts from body.
     Email Subject: * | ps | <Short Command Description>
   • cmd: Runs Command Prompt script from email body.
     Email Subject: * | cmd | <Short Command Description>
   • gitmap: Executes GitMap autonomous CLI commands (e.g. status, scan, sync, macro).
     Email Subject: * | gitmap | <Arguments>
   • agm status: Returns node status, active account, and credits remaining.
   • agm ff / agm smart-switch: Rotates immediately to the freshest available account.
   • agm switch | <email>: Switches the active account profile to the specified email.
   • agm accounts / acc: Returns registered accounts and active status.
   • agm doctor / check: Runs system health diagnostics and reports anomalies.
   • agm instances / ls: Lists active sandbox profiles and running process IDs.
   • agm proxy [status|test]: Checks proxy socket status or runs loopback test.
   • agm clean / purge: Safely prunes build caches and test artifacts.
   • agm sync: Synchronizes local accounts, instances, and DB vaults.
   • agy prompts ls: Lists backed-up workspace prompts.
   • gitmap prompts ls: Lists GitMap automated prompts.
   • update: Checks for and applies latest Antigravity Manager updates.

3. TWO-PHASE AUTOMATED RECEIPTS:
   • Phase 1 ACK: Immediate acknowledgement email with IN_PROGRESS badge.
   • Phase 2 RESULT: Final completion receipt with exit code and stdout/stderr logs.
================================================================================`;

    // Developer Task Quick Dispatch state (Defaults to 'help' with comprehensive manual)
    const [developerTaskType, setDeveloperTaskType] = useState<string>('help');
    const [developerTargetRecipient, setDeveloperTargetRecipient] = useState<string>('');
    const [developerTaskPayload, setDeveloperTaskPayload] = useState<string>(HELP_COMMAND_PAYLOAD);
    const [developerCustomSubject, setDeveloperCustomSubject] = useState<string>('* | help');
    const [isDispatchingTask, setIsDispatchingTask] = useState<boolean>(false);

    // Telegram Bot Integration State
    const [telegramConfig, setTelegramConfig] = useState<TelegramConfig | null>(null);
    const [telegramStatus, setTelegramStatus] = useState<TelegramWatcherStatus | null>(null);
    const [isTestingTelegram, setIsTestingTelegram] = useState<boolean>(false);
    const [isDetectingChatId, setIsDetectingChatId] = useState<boolean>(false);
    const [isSavingTelegram, setIsSavingTelegram] = useState<boolean>(false);
    const [isSendingTelegramPing, setIsSendingTelegramPing] = useState<boolean>(false);
    const [telegramBotUsername, setTelegramBotUsername] = useState<string | null>(null);
    const [showBotToken, setShowBotToken] = useState<boolean>(false);

    // Account modal state
    const [isAccountModalOpen, setIsAccountModalOpen] = useState(false);
    const [editingAccount, setEditingAccount] = useState<EmailAccountInput>({
        alias: '',
        email: '',
        password: '',
        smtp_host: 'smtp.gmail.com',
        smtp_port: 587,
        imap_host: 'imap.gmail.com',
        imap_port: 993,
        encryption_type: 'TLS',
        is_default: false,
        is_active: true,
    });

    // Recipient state
    const [newRecipientEmail, setNewRecipientEmail] = useState('');
    const [newRecipientGroup, setNewRecipientGroup] = useState('default');

    // Import/Export state
    const [isImportModalOpen, setIsImportModalOpen] = useState(false);
    const [importFormat, setImportFormat] = useState<'json' | 'yaml' | 'csv' | 'xlsx'>('json');
    const [importPayload, setImportPayload] = useState('');
    const [testingAccountId, setTestingAccountId] = useState<string | null>(null);
    const fileInputRef = React.useRef<HTMLInputElement>(null);
    const [isActionsOpen, setIsActionsOpen] = useState(false);
    const [isSampleTemplatesOpen, setIsSampleTemplatesOpen] = useState(false);
    const actionsDropdownRef = React.useRef<HTMLDivElement>(null);

    // Export preview modal state (all accounts or single account)
    const [exportModalState, setExportModalState] = useState<{
        isOpen: boolean;
        singleAccount?: Partial<EmailAccount> | null;
        allAccounts?: EmailAccount[];
        initialFormat?: 'json' | 'yaml' | 'csv';
    }>({ isOpen: false });

    // Single account modal quick import/export state
    const [isModalQuickImportOpen, setIsModalQuickImportOpen] = useState(false);
    const [modalQuickImportText, setModalQuickImportText] = useState('');
    const [showAiJsonSyntax, setShowAiJsonSyntax] = useState(false);
    const singleAccountFileInputRef = React.useRef<HTMLInputElement>(null);

    useEffect(() => {
        const handleClickOutside = (event: MouseEvent) => {
            if (actionsDropdownRef.current && !actionsDropdownRef.current.contains(event.target as Node)) {
                setIsActionsOpen(false);
            }
        };
        document.addEventListener('mousedown', handleClickOutside);
        return () => document.removeEventListener('mousedown', handleClickOutside);
    }, []);

    const loadAll = async () => {
        try {
            const [accs, recs, sets, tgConf, tgStat] = await Promise.all([
                listEmailAccounts(),
                listNotifyRecipients(),
                getEmailSettings(),
                telegramService.getConfig().catch(() => null),
                telegramService.getStatus().catch(() => null),
            ]);
            setAccounts(accs);
            setRecipients(recs);
            setSettings({
                ...sets,
                notify_on_system_update: sets.notify_on_system_update ?? true,
            });
            if (tgConf) {
                setTelegramConfig({
                    ...tgConf,
                    notify_on_system_update: tgConf.notify_on_system_update ?? true,
                });
            }
            if (tgStat) setTelegramStatus(tgStat);
        } catch (e: any) {
            console.error('Failed to load email settings:', e);
            // Tracked in the error module; user already notified via error toast.
            useErrorStore.getState().trackWarning(e, {
              source: 'EmailNotificationSettings.loadAll',
              triggerAction: 'load_email_settings',
            });
            showToast('Failed to load email configurations', 'error');
        }
    };

    useEffect(() => {
        loadAll();
    }, []);


    const [isTestingDirect, setIsTestingDirect] = useState(false);
    const [testResult, setTestResult] = useState<{ success: boolean; message: string } | null>(null);

    return {
        accounts,
        setAccounts,
        recipients,
        setRecipients,
        settings,
        setSettings,
        isSaving,
        setIsSaving,
        isPinging,
        setIsPinging,
        HELP_COMMAND_PAYLOAD,
        developerTaskType,
        setDeveloperTaskType,
        developerTargetRecipient,
        setDeveloperTargetRecipient,
        developerTaskPayload,
        setDeveloperTaskPayload,
        developerCustomSubject,
        setDeveloperCustomSubject,
        isDispatchingTask,
        setIsDispatchingTask,
        telegramConfig,
        setTelegramConfig,
        telegramStatus,
        setTelegramStatus,
        isTestingTelegram,
        setIsTestingTelegram,
        isDetectingChatId,
        setIsDetectingChatId,
        isSavingTelegram,
        setIsSavingTelegram,
        isSendingTelegramPing,
        setIsSendingTelegramPing,
        telegramBotUsername,
        setTelegramBotUsername,
        showBotToken,
        setShowBotToken,
        isAccountModalOpen,
        setIsAccountModalOpen,
        editingAccount,
        setEditingAccount,
        newRecipientEmail,
        setNewRecipientEmail,
        newRecipientGroup,
        setNewRecipientGroup,
        isImportModalOpen,
        setIsImportModalOpen,
        importFormat,
        setImportFormat,
        importPayload,
        setImportPayload,
        testingAccountId,
        setTestingAccountId,
        fileInputRef,
        isActionsOpen,
        setIsActionsOpen,
        isSampleTemplatesOpen,
        setIsSampleTemplatesOpen,
        actionsDropdownRef,
        exportModalState,
        setExportModalState,
        isModalQuickImportOpen,
        setIsModalQuickImportOpen,
        modalQuickImportText,
        setModalQuickImportText,
        showAiJsonSyntax,
        setShowAiJsonSyntax,
        singleAccountFileInputRef,
        handleClickOutside,
        loadAll,
        isTestingDirect,
        setIsTestingDirect,
        testResult,
        setTestResult,
    };
}

export type EmailNotificationsState = ReturnType<typeof useEmailCore>;
