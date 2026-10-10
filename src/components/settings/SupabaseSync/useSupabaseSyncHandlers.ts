import { SupabaseSyncState } from './useSupabaseSyncState';
import { invoke } from '@tauri-apps/api/core';
import { showToast } from '../common/ToastContainer';
import { useErrorStore } from '../../stores/error-store';
import { supabaseService } from '../../services/supabaseService';
import { telegramService } from '../../services/telegramService';

export function useSupabaseSyncHandlers(s: SupabaseSyncState) {
    const { config, setConfig, nodeInfo, setNodeInfo, isLoading, setIsLoading, setIsSaving, setTestingEndpointId, setTestResults, setVerifyingEndpointId, setTableVerification, setIsAutoDiscovering, setIsAddModalOpen, setIsSchemaModalOpen, setSchemaRole, setSchemaSql, setIsExportModalOpen, setExportContent, setExportFormat, exportRounds, setIsImportModalOpen, importText, setImportText, setIsMigrateModalOpen, sourceEpId, setSourceEpId, targetEpId, setTargetEpId, setIsMigrating, setMigrationResult, telegramConfig, setTelegramConfig, setTelegramStatus, setIsTestingTelegram, setTelegramBotUsername, setIsSavingTelegram, setIsSendingPing, leases, setLeases, setIsLoadingLeases, formEndpoint, setFormEndpoint } = s;

    const fetchLeases = async () => {
        setIsLoadingLeases(true);
        try {
            const loadedLeases = await supabaseService.listActiveLeases();
            setLeases(loadedLeases || []);
        } catch (e) {
            console.warn('Failed to load active workspace leases:', e);
            // Tracked in the error module; empty lease list shown, retried on next load.
            useErrorStore.getState().trackWarning(e, {
              source: 'SupabaseSyncSettings.loadLeases',
              triggerAction: 'load_active_leases',
            });
            setLeases([]);
        } finally {
            setIsLoadingLeases(false);
        }
    };

    const loadData = async () => {
        setIsLoading(true);
        try {
            const [loadedConfig, loadedNode, loadedTgConfig, loadedTgStatus, loadedLeases] = await Promise.all([
                supabaseService.getConfig(),
                supabaseService.getLocalNodeInfo(),
                telegramService.getConfig().catch(() => null),
                telegramService.getStatus().catch(() => null),
                supabaseService.listActiveLeases().catch(() => []),
            ]);
            setConfig(loadedConfig);
            setNodeInfo(loadedNode);
            if (loadedTgConfig) setTelegramConfig(loadedTgConfig);
            if (loadedTgStatus) setTelegramStatus(loadedTgStatus);
            if (loadedLeases) setLeases(loadedLeases);
        } catch (e) {
            console.error('Failed to load Supabase / Telegram settings:', e);
            // Tracked in the error module; settings form keeps defaults, user can retry.
            useErrorStore.getState().trackWarning(e, {
              source: 'SupabaseSyncSettings.loadAll',
              triggerAction: 'load_supabase_settings',
            });
        } finally {
            setIsLoading(false);
        }
    };

    const handleSaveConfig = async (newConfig: SupabaseConfig) => {
        setIsSaving(true);
        try {
            await supabaseService.saveConfig(newConfig);
            setConfig(newConfig);
            showToast('Supabase settings saved successfully', 'success');
        } catch (e) {
            showToast(`Failed to save: ${String(e)}`, 'error');
        } finally {
            setIsSaving(false);
        }
    };

    const normalizeSupabaseUrl = (raw: string): string => {
        let trimmed = raw.trim();
        while (trimmed.endsWith('/')) {
            trimmed = trimmed.slice(0, -1);
        }
        if (trimmed.endsWith('/rest/v1')) {
            trimmed = trimmed.slice(0, -8);
            while (trimmed.endsWith('/')) {
                trimmed = trimmed.slice(0, -1);
            }
        }
        return trimmed;
    };

    const handleTestEndpoint = async (endpoint: SupabaseEndpoint) => {
        setTestingEndpointId(endpoint.id);
        const res = await supabaseService.testEndpoint(endpoint);
        setTestResults((prev) => ({
            ...prev,
            [endpoint.id]: {
                isSuccess: res.is_success,
                msg: res.message,
            },
        }));
        if (res.is_success) {
            showToast(res.message || 'Connected successfully', 'success');
        } else {
            showToast(`${endpoint.name}: ${res.message || 'Connection failed'}`, 'error');
        }
        setTestingEndpointId(null);
    };

    const handleCheckTables = async (endpoint: SupabaseEndpoint) => {
        setVerifyingEndpointId(endpoint.id);
        try {
            const res = await supabaseService.checkEndpointTables(endpoint);
            setTableVerification((prev) => ({
                ...prev,
                [endpoint.id]: res,
            }));
            if (res.missing_tables.length === 0) {
                showToast(`All ${res.verified_tables.length} tables verified on ${endpoint.name}`, 'success');
            } else {
                showToast(`Missing tables on ${endpoint.name}: ${res.missing_tables.join(', ')}`, 'warning');
            }
        } catch (e) {
            showToast(`Table verification failed: ${String(e)}`, 'error');
        } finally {
            setVerifyingEndpointId(null);
        }
    };

    const handleAutoDiscover = async () => {
        setIsAutoDiscovering(true);
        try {
            const updatedConfig = await invoke<SupabaseConfig>('auto_discover_supabase_credentials');
            setConfig(updatedConfig);
            showToast(
                `Auto-discovered ${updatedConfig.endpoints.length} Supabase endpoint(s) from repo-secrets!`,
                'success'
            );
            fetchLeases();
        } catch (e) {
            showToast(`Auto-discovery failed: ${String(e)}`, 'error');
        } finally {
            setIsAutoDiscovering(false);
        }
    };

    const handleOpenMigrateModal = () => {
        if (!config || config.endpoints.length < 2) {
            showToast('You need at least 2 endpoints to perform cross-database migration', 'warning');
            return;
        }
        setSourceEpId(config.endpoints[0].id);
        setTargetEpId(config.endpoints[1].id);
        setMigrationResult(null);
        setIsMigrateModalOpen(true);
    };

    const handleMigrateSubmit = async () => {
        if (!sourceEpId || !targetEpId) {
            showToast('Please select source and target endpoints', 'warning');
            return;
        }
        if (sourceEpId === targetEpId) {
            showToast('Source and target endpoints cannot be the same', 'warning');
            return;
        }
        setIsMigrating(true);
        try {
            const res = await supabaseService.migrateData(sourceEpId, targetEpId);
            setMigrationResult(res);
            if (res.is_success) {
                showToast(res.message, 'success');
            } else {
                showToast(`Migration error: ${res.message}`, 'error');
            }
        } catch (e) {
            showToast(`Data migration failed: ${String(e)}`, 'error');
        } finally {
            setIsMigrating(false);
        }
    };

    const handleSaveTelegram = async () => {
        if (!telegramConfig) return;
        setIsSavingTelegram(true);
        try {
            await telegramService.saveConfig(telegramConfig);
            showToast('Telegram bot settings saved', 'success');
            const st = await telegramService.getStatus();
            setTelegramStatus(st);
        } catch (e) {
            showToast(`Failed to save Telegram settings: ${String(e)}`, 'error');
        } finally {
            setIsSavingTelegram(false);
        }
    };

    const handleTestTelegram = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim()) {
            showToast('Please provide a Telegram Bot Token', 'warning');
            return;
        }
        setIsTestingTelegram(true);
        try {
            const username = await telegramService.testBot(telegramConfig.bot_token);
            setTelegramBotUsername(username);
            if (!telegramConfig.allowed_chat_id) {
                try {
                    const detected = await telegramService.detectChatId(telegramConfig.bot_token);
                    const updated = {
                        ...telegramConfig,
                        allowed_chat_id: detected.chat_id,
                        is_enabled: true,
                    };
                    setTelegramConfig(updated);
                    await telegramService.saveConfig(updated);
                    showToast(`Connected to @${username} & auto-detected Chat ID ${detected.chat_id} (${detected.chat_label})`, 'success');
                    return;
                } catch {
                    // Chat ID not yet available in getUpdates
                }
            }
            showToast(`Connected to Telegram bot: @${username}`, 'success');
        } catch (e) {
            setTelegramBotUsername(null);
            showToast(`Telegram connection failed: ${String(e)}`, 'error');
        } finally {
            setIsTestingTelegram(false);
        }
    };

    const handleDetectTelegramChatId = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim()) {
            showToast('Please paste your Telegram Bot Token first', 'warning');
            return;
        }
        setIsTestingTelegram(true);
        try {
            const detected = await telegramService.detectChatId(telegramConfig.bot_token);
            setTelegramBotUsername(detected.bot_username);
            const updated = {
                ...telegramConfig,
                allowed_chat_id: detected.chat_id,
                is_enabled: true,
            };
            setTelegramConfig(updated);
            await telegramService.saveConfig(updated);
            showToast(`Auto-detected Chat ID: ${detected.chat_id} (${detected.chat_label})`, 'success');
        } catch (e) {
            showToast(`Auto-detect Chat ID: ${String(e)}`, 'warning');
        } finally {
            setIsTestingTelegram(false);
        }
    };

    const handleSendTelegramPing = async () => {
        if (!telegramConfig || !telegramConfig.bot_token.trim() || !telegramConfig.allowed_chat_id) {
            showToast('Please specify Bot Token and Allowed Chat ID first', 'warning');
            return;
        }
        setIsSendingPing(true);
        try {
            await telegramService.sendTestMessage(telegramConfig.bot_token, telegramConfig.allowed_chat_id);
            showToast('Test ping sent to your Telegram chat!', 'success');
        } catch (e) {
            showToast(`Failed to send test ping: ${String(e)}`, 'error');
        } finally {
            setIsSendingPing(false);
        }
    };

    const handleOpenSchemaModal = async (role: 'root' | 'secondary') => {
        setSchemaRole(role);
        try {
            const sql = await supabaseService.getSchemaSql(role);
            setSchemaSql(sql);
            setIsSchemaModalOpen(true);
        } catch (e) {
            showToast('Failed to load schema SQL', 'error');
        }
    };

    const handleOpenExport = async (format: 'json' | 'yaml') => {
        setExportFormat(format);
        try {
            const text = await supabaseService.exportConfig(format, exportRounds);
            setExportContent(text);
            setIsExportModalOpen(true);
        } catch (e) {
            showToast('Failed to generate export', 'error');
        }
    };

    const handleImportSubmit = async () => {
        if (!importText.trim()) return;
        try {
            const updated = await supabaseService.importConfig(importText);
            setConfig(updated);
            setIsImportModalOpen(false);
            setImportText('');
            showToast('Configuration imported successfully', 'success');
        } catch (e) {
            showToast(`Import failed: ${String(e)}`, 'error');
        }
    };

    const handleAddEndpointSubmit = () => {
        if (!config || !formEndpoint.name || !formEndpoint.url || !formEndpoint.api_key) {
            showToast('Please fill all required endpoint fields', 'error');
            return;
        }

        const cleanUrl = normalizeSupabaseUrl(formEndpoint.url);
        const newEp: SupabaseEndpoint = {
            id: `ep_${Date.now()}`,
            name: formEndpoint.name.trim(),
            url: cleanUrl,
            api_key: formEndpoint.api_key.trim(),
            role: formEndpoint.role as 'root' | 'secondary',
            is_enabled: formEndpoint.is_enabled ?? true,
            prune_threshold_mb: formEndpoint.prune_threshold_mb ?? (formEndpoint.role === 'root' ? 400 : 200),
            priority: formEndpoint.priority ?? 1,
        };

        const updatedConfig = {
            ...config,
            endpoints: [...config.endpoints, newEp],
        };

        handleSaveConfig(updatedConfig);
        setIsAddModalOpen(false);
        setFormEndpoint({
            name: '',
            url: '',
            api_key: '',
            role: 'secondary',
            is_enabled: true,
            prune_threshold_mb: 200,
            priority: 1,
        });
    };

    const handleDeleteEndpoint = (id: string) => {
        if (!config) return;
        const updated = {
            ...config,
            endpoints: config.endpoints.filter((ep) => ep.id !== id),
        };
        handleSaveConfig(updated);
    };

    const aiInstructionTemplate = `You are a DevOps Assistant for Antigravity Manager.
Convert my Supabase project credentials into the following strict JSON format for AGM cross-machine synchronization:

\`\`\`json
{
  "version": "1.0.0",
  "node_alias": "Node-${nodeInfo?.node_alias || 'Primary'}",
  "is_sync_enabled": true,
  "auto_prune_root_mb": 400,
  "auto_prune_secondary_mb": 200,
  "heartbeat_interval_secs": 30,
  "endpoints": [
    {
      "id": "ep_root_01",
      "name": "Supabase Root DB",
      "url": "<YOUR_SUPABASE_PROJECT_URL>",
      "api_key": "<YOUR_SUPABASE_SERVICE_ROLE_OR_ANON_KEY>",
      "role": "root",
      "is_enabled": true,
      "prune_threshold_mb": 400,
      "priority": 1
    },
    {
      "id": "ep_sec_01",
      "name": "Supabase Secondary Command DB",
      "url": "<YOUR_SECONDARY_SUPABASE_URL>",
      "api_key": "<YOUR_SECONDARY_SUPABASE_KEY>",
      "role": "secondary",
      "is_enabled": true,
      "prune_threshold_mb": 200,
      "priority": 1
    }
  ]
}
\`\`\`

Here are my Supabase details:
- Root Project URL: [Paste URL here]
- Root API Key: [Paste key here]
- Secondary Project URL (optional): [Paste secondary URL here]
- Secondary API Key (optional): [Paste secondary key here]`;



    return {
        fetchLeases,
        loadedLeases,
        loadData,
        handleSaveConfig,
        normalizeSupabaseUrl,
        handleTestEndpoint,
        res,
        handleCheckTables,
        handleAutoDiscover,
        updatedConfig,
        handleOpenMigrateModal,
        handleMigrateSubmit,
        handleSaveTelegram,
        st,
        handleTestTelegram,
        username,
        detected,
        updated,
        handleDetectTelegramChatId,
        handleSendTelegramPing,
        handleOpenSchemaModal,
        sql,
        handleOpenExport,
        text,
        handleImportSubmit,
        handleAddEndpointSubmit,
        cleanUrl,
        handleDeleteEndpoint,
        aiInstructionTemplate,
    };
}
