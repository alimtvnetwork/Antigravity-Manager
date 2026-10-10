import { useState, useEffect } from 'react';
import type { SupabaseConfig, SupabaseEndpoint, LocalNodeInfo, TableVerificationResult, DataMigrationSummary, WorkspaceLease } from '../../services/supabaseService';
import type { TelegramConfig, TelegramWatcherStatus } from '../../services/telegramService';

export function useSupabaseSyncState() {
    const [config, setConfig] = useState<SupabaseConfig | null>(null);
    const [nodeInfo, setNodeInfo] = useState<LocalNodeInfo | null>(null);
    const [isLoading, setIsLoading] = useState(true);
    const [isSaving, setIsSaving] = useState(false);
    const [testingEndpointId, setTestingEndpointId] = useState<string | null>(null);
    const [testResults, setTestResults] = useState<Record<string, { isSuccess: boolean; msg: string }>>({});
    const [verifyingEndpointId, setVerifyingEndpointId] = useState<string | null>(null);
    const [tableVerification, setTableVerification] = useState<Record<string, TableVerificationResult>>({});
    const [isAutoDiscovering, setIsAutoDiscovering] = useState(false);

    // Modals
    const [isAddModalOpen, setIsAddModalOpen] = useState(false);
    const [isSchemaModalOpen, setIsSchemaModalOpen] = useState(false);
    const [schemaRole, setSchemaRole] = useState<'root' | 'secondary'>('root');
    const [schemaSql, setSchemaSql] = useState('');
    const [isExportModalOpen, setIsExportModalOpen] = useState(false);
    const [exportContent, setExportContent] = useState('');
    const [exportFormat, setExportFormat] = useState<'json' | 'yaml'>('json');
    const [exportRounds, setExportRounds] = useState(4);
    const [isImportModalOpen, setIsImportModalOpen] = useState(false);
    const [importText, setImportText] = useState('');
    const [isAiPromptModalOpen, setIsAiPromptModalOpen] = useState(false);

    // Cross-DB Migration Modal
    const [isMigrateModalOpen, setIsMigrateModalOpen] = useState(false);
    const [sourceEpId, setSourceEpId] = useState('');
    const [targetEpId, setTargetEpId] = useState('');
    const [isMigrating, setIsMigrating] = useState(false);
    const [migrationResult, setMigrationResult] = useState<DataMigrationSummary | null>(null);

    // Telegram Bot Integration State
    const [telegramConfig, setTelegramConfig] = useState<TelegramConfig | null>(null);
    const [telegramStatus, setTelegramStatus] = useState<TelegramWatcherStatus | null>(null);
    const [isTestingTelegram, setIsTestingTelegram] = useState(false);
    const [telegramBotUsername, setTelegramBotUsername] = useState<string | null>(null);
    const [isSavingTelegram, setIsSavingTelegram] = useState(false);
    const [isSendingPing, setIsSendingPing] = useState(false);
    const [isTgGuideOpen, setIsTgGuideOpen] = useState(false);
    const [isBackupModalOpen, setIsBackupModalOpen] = useState(false);

    // Workspace Leases State
    const [leases, setLeases] = useState<WorkspaceLease[]>([]);
    const [isLoadingLeases, setIsLoadingLeases] = useState(false);
    const [revealedLeaseEmails, setRevealedLeaseEmails] = useState<Record<string, boolean>>({});

    // Form state for adding/editing endpoint
    const [formEndpoint, setFormEndpoint] = useState<Partial<SupabaseEndpoint>>({
        name: '',
        url: '',
        api_key: '',
        role: 'secondary',
        is_enabled: true,
        prune_threshold_mb: 200,
        priority: 1,
    });

    useEffect(() => {
        loadData();
    }, []);


    return {
        config,
        setConfig,
        nodeInfo,
        setNodeInfo,
        isLoading,
        setIsLoading,
        isSaving,
        setIsSaving,
        testingEndpointId,
        setTestingEndpointId,
        testResults,
        setTestResults,
        verifyingEndpointId,
        setVerifyingEndpointId,
        tableVerification,
        setTableVerification,
        isAutoDiscovering,
        setIsAutoDiscovering,
        isAddModalOpen,
        setIsAddModalOpen,
        isSchemaModalOpen,
        setIsSchemaModalOpen,
        schemaRole,
        setSchemaRole,
        schemaSql,
        setSchemaSql,
        isExportModalOpen,
        setIsExportModalOpen,
        exportContent,
        setExportContent,
        exportFormat,
        setExportFormat,
        exportRounds,
        setExportRounds,
        isImportModalOpen,
        setIsImportModalOpen,
        importText,
        setImportText,
        isAiPromptModalOpen,
        setIsAiPromptModalOpen,
        isMigrateModalOpen,
        setIsMigrateModalOpen,
        sourceEpId,
        setSourceEpId,
        targetEpId,
        setTargetEpId,
        isMigrating,
        setIsMigrating,
        migrationResult,
        setMigrationResult,
        telegramConfig,
        setTelegramConfig,
        telegramStatus,
        setTelegramStatus,
        isTestingTelegram,
        setIsTestingTelegram,
        telegramBotUsername,
        setTelegramBotUsername,
        isSavingTelegram,
        setIsSavingTelegram,
        isSendingPing,
        setIsSendingPing,
        isTgGuideOpen,
        setIsTgGuideOpen,
        isBackupModalOpen,
        setIsBackupModalOpen,
        leases,
        setLeases,
        isLoadingLeases,
        setIsLoadingLeases,
        revealedLeaseEmails,
        setRevealedLeaseEmails,
        formEndpoint,
        setFormEndpoint,
    };
}

export type SupabaseSyncState = ReturnType<typeof useSupabaseSyncState>;
