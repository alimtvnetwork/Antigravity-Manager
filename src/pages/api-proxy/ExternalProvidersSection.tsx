import type { TFunction } from 'i18next';
import type { SelectOption } from '../../components/common/GroupedSelect';
import type { AppConfig, CircuitBreakerConfig } from '../../types/config';
import type {
    CloudflaredControls,
    ExperimentalConfigUpdater,
    FixedAccountControls,
    PresetManager,
    ProxyConfigUpdater,
    ProxyStatus,
    SaveConfigFn,
    SchedulingConfigUpdater,
} from './types';
import type { ZaiDispatcher } from './useZaiDispatcher';
import { ThinkingSection } from './ThinkingSection';
import { ModelMappingCard } from './ModelMappingCard';
import { ZaiDispatcherSection } from './ZaiDispatcherSection';
import { McpSystemSection } from './McpSystemSection';
import { AccountSchedulingSection } from './AccountSchedulingSection';
import { AdvancedSettingsSection } from './AdvancedSettingsSection';
import { CloudflaredSection } from './CloudflaredSection';

interface ExternalProvidersSectionProps {
    config: AppConfig;
    t: TFunction;
    status: ProxyStatus;
    copied: string | null;
    saveConfig: SaveConfigFn;
    updateProxyConfig: ProxyConfigUpdater;
    updateSchedulingConfig: SchedulingConfigUpdater;
    updateExperimentalConfig: ExperimentalConfigUpdater;
    updateCircuitBreakerConfig: (config: CircuitBreakerConfig) => void;
    handleSaveProxySettings: () => Promise<void>;
    handleMappingUpdate: (type: 'custom', key: string, value: string) => Promise<void>;
    handleRemoveCustomMapping: (key: string) => Promise<void>;
    handleResetMapping: () => void;
    customMappingOptions: SelectOption[];
    presetManager: PresetManager;
    cloudflared: CloudflaredControls;
    fixedAccount: FixedAccountControls;
    zai: ZaiDispatcher;
    onClearSessionBindings: () => void;
    onClearRateLimits: () => void;
}

/**
 * External Providers Integration — rendered under the Service Settings tab.
 * Composes the thinking/model-router/z.ai/MCP/scheduling/advanced/cloudflared cards.
 */
export function ExternalProvidersSection({
    config,
    t,
    status,
    copied,
    saveConfig,
    updateProxyConfig,
    updateSchedulingConfig,
    updateExperimentalConfig,
    updateCircuitBreakerConfig,
    handleSaveProxySettings,
    handleMappingUpdate,
    handleRemoveCustomMapping,
    handleResetMapping,
    customMappingOptions,
    presetManager,
    cloudflared,
    fixedAccount,
    zai,
    onClearSessionBindings,
    onClearRateLimits,
}: ExternalProvidersSectionProps) {
    return (
        <div className="space-y-4">
            {/* Thinking & Reasoning Settings */}
            <ThinkingSection
                config={config}
                t={t}
                updateProxyConfig={updateProxyConfig}
                updateExperimentalConfig={updateExperimentalConfig}
                onSave={handleSaveProxySettings}
            />

            {/* Model Router Center */}
            <ModelMappingCard
                config={config}
                t={t}
                updateProxyConfig={updateProxyConfig}
                customMappingOptions={customMappingOptions}
                presetManager={presetManager}
                onMappingUpdate={handleMappingUpdate}
                onRemoveCustomMapping={handleRemoveCustomMapping}
                onResetMapping={handleResetMapping}
            />

            {/* z.ai (GLM) Dispatcher */}
            <ZaiDispatcherSection
                config={config}
                t={t}
                zai={zai}
            />

            {/* MCP System */}
            <McpSystemSection
                config={config}
                t={t}
                status={status}
                zai={zai}
            />

            {/* Account Scheduling & Rotation */}
            <AccountSchedulingSection
                config={config}
                t={t}
                status={status}
                updateSchedulingConfig={updateSchedulingConfig}
                updateCircuitBreakerConfig={updateCircuitBreakerConfig}
                fixedAccount={fixedAccount}
                onClearSessionBindings={onClearSessionBindings}
                onClearRateLimits={onClearRateLimits}
            />

            {/* Advanced Thinking & Global Config + Experimental */}
            <AdvancedSettingsSection
                config={config}
                t={t}
                updateProxyConfig={updateProxyConfig}
                updateExperimentalConfig={updateExperimentalConfig}
            />

            {/* 公网访问 (Cloudflared) - 仅在桌面端显示 */}
            <CloudflaredSection
                config={config}
                t={t}
                saveConfig={saveConfig}
                copied={copied}
                cloudflared={cloudflared}
            />
        </div>
    );
}
