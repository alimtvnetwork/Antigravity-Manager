import type { TFunction } from 'i18next';
import { BrainCircuit, Sparkles } from 'lucide-react';
import HelpTooltip from '../../components/common/HelpTooltip';
import AdvancedThinking from '../../components/settings/AdvancedThinking';
import DebouncedSlider from '../../components/common/DebouncedSlider';
import type { AppConfig } from '../../types/config';
import type { ExperimentalConfigUpdater, ProxyConfigUpdater } from './types';
import { DEFAULT_EXPERIMENTAL_THRESHOLDS, DEFAULT_THINKING_RETENTION_DAYS } from './constants';
import { CollapsibleCard } from './CollapsibleCard';

interface AdvancedSettingsSectionProps {
    config: AppConfig;
    t: TFunction;
    updateProxyConfig: ProxyConfigUpdater;
    updateExperimentalConfig: ExperimentalConfigUpdater;
}

const COMPRESSION_SLIDER_MIN = 0.1;
const COMPRESSION_SLIDER_MAX = 1;
const COMPRESSION_SLIDER_STEP = 0.05;

export function AdvancedSettingsSection({
    config,
    t,
    updateProxyConfig,
    updateExperimentalConfig,
}: AdvancedSettingsSectionProps) {
    const compressionLevel = config.proxy.experimental?.compression_level
        || (config.proxy.experimental?.enable_usage_scaling ? 'high' : 'disabled');

    return (
        <>
            {/* Advanced Thinking & Global Config */}
            <CollapsibleCard
                title={t('settings.advanced_thinking.title', { defaultValue: 'Advanced Thinking & Global Config' })}
                icon={<BrainCircuit size={18} className="text-pink-500" />}
            >
                <AdvancedThinking
                    config={config.proxy}
                    onChange={(newProxyConfig) => updateProxyConfig(newProxyConfig)}
                />
            </CollapsibleCard>

            {/* 实验性设置 */}
            <CollapsibleCard
                title={t('proxy.config.experimental.title')}
                icon={<Sparkles size={18} className="text-purple-500" />}
            >
                <div className="space-y-4">
                    <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                        <div className="space-y-1">
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                    {t('proxy.config.experimental.compression_level_label', { defaultValue: '智能上下文压缩等级' })}
                                </span>
                                <HelpTooltip text={t('proxy.config.experimental.compression_level_tooltip', { defaultValue: '选择您希望启用的压缩等级。静态降噪与口语提纯不需要达到 30k 即可常驻生效。' })} />
                                <span className="px-1.5 py-0.5 rounded bg-indigo-100 dark:bg-indigo-900/30 text-[10px] text-indigo-600 dark:text-indigo-400 font-bold border border-indigo-200 dark:border-indigo-800">
                                    All Protocols
                                </span>
                            </div>
                            <p className="text-[10px] text-gray-500 dark:text-gray-400 max-w-lg">
                                {t('proxy.config.experimental.compression_level_desc', { defaultValue: '选择不同的压缩方案：Low 仅终端日志降噪；Medium 在此基础上增加口语净化；High 额外开启大上下文分阶段防御重置。' })}
                            </p>
                        </div>
                        <select
                            className="select select-sm select-bordered w-48 text-xs font-normal focus:outline-none dark:bg-base-300 dark:text-base-content border-gray-200 dark:border-base-400"
                            value={compressionLevel}
                            onChange={(e) => {
                                const val = e.target.value;
                                updateExperimentalConfig({
                                    compression_level: val,
                                    enable_usage_scaling: val === 'high'
                                });
                            }}
                        >
                            <option value="disabled" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.level_disabled', { defaultValue: '关闭 (Disabled)' })}</option>
                            <option value="low" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.level_low', { defaultValue: '低度 (Low - 日志降噪)' })}</option>
                            <option value="medium" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.level_medium', { defaultValue: '中度 (Medium - 日志+口语)' })}</option>
                            <option value="high" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.level_high', { defaultValue: '高度 (High - 动态防暴)' })}</option>
                        </select>
                    </div>

                    {compressionLevel === 'high' && (
                        <>
                            {/* L1 Threshold */}
                            <div className="flex flex-col gap-2 p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                                <div className="flex items-center justify-between w-full">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                            {t('proxy.config.experimental.context_compression_threshold_l1')}
                                        </span>
                                        <HelpTooltip text={t('proxy.config.experimental.context_compression_threshold_l1_tooltip')} />
                                    </div>
                                </div>
                                <DebouncedSlider
                                    min={COMPRESSION_SLIDER_MIN}
                                    max={COMPRESSION_SLIDER_MAX}
                                    step={COMPRESSION_SLIDER_STEP}
                                    className="range range-purple range-xs"
                                    value={config.proxy.experimental?.context_compression_threshold_l1 || DEFAULT_EXPERIMENTAL_THRESHOLDS.l1}
                                    onChange={(val) => updateExperimentalConfig({ context_compression_threshold_l1: val })}
                                />
                            </div>

                            {/* L2 Threshold */}
                            <div className="flex flex-col gap-2 p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                                <div className="flex items-center justify-between w-full">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                            {t('proxy.config.experimental.context_compression_threshold_l2')}
                                        </span>
                                        <HelpTooltip text={t('proxy.config.experimental.context_compression_threshold_l2_tooltip')} />
                                    </div>
                                </div>
                                <DebouncedSlider
                                    min={COMPRESSION_SLIDER_MIN}
                                    max={COMPRESSION_SLIDER_MAX}
                                    step={COMPRESSION_SLIDER_STEP}
                                    className="range range-purple range-xs"
                                    value={config.proxy.experimental?.context_compression_threshold_l2 || DEFAULT_EXPERIMENTAL_THRESHOLDS.l2}
                                    onChange={(val) => updateExperimentalConfig({ context_compression_threshold_l2: val })}
                                />
                            </div>

                            {/* L3 Threshold */}
                            <div className="flex flex-col gap-2 p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                                <div className="flex items-center justify-between w-full">
                                    <div className="flex items-center gap-2">
                                        <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                            {t('proxy.config.experimental.context_compression_threshold_l3')}
                                        </span>
                                        <HelpTooltip text={t('proxy.config.experimental.context_compression_threshold_l3_tooltip')} />
                                    </div>
                                </div>
                                <DebouncedSlider
                                    min={COMPRESSION_SLIDER_MIN}
                                    max={COMPRESSION_SLIDER_MAX}
                                    step={COMPRESSION_SLIDER_STEP}
                                    className="range range-purple range-xs"
                                    value={config.proxy.experimental?.context_compression_threshold_l3 || DEFAULT_EXPERIMENTAL_THRESHOLDS.l3}
                                    onChange={(val) => updateExperimentalConfig({ context_compression_threshold_l3: val })}
                                />
                            </div>
                        </>
                    )}

                    <div className="flex items-center justify-between p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                        <div className="space-y-1">
                            <div className="flex items-center gap-2">
                                <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                    {t('proxy.config.experimental.payload_storage_mode_label', { defaultValue: 'Payload Storage Mode' })}
                                </span>
                                <HelpTooltip text={t('proxy.config.experimental.payload_storage_mode_tooltip', { defaultValue: 'Concise mode writes only thinking blocks, usage, session IDs, and concise messages to SQLite; Full mode persists raw payloads. API keys are always masked.' })} />
                            </div>
                            <p className="text-[10px] text-gray-500 dark:text-gray-400 max-w-lg">
                                {t('proxy.config.experimental.payload_storage_mode_desc', { defaultValue: 'Default concise storage prevents database bloat from tool payloads and images. Switch to full mode when raw payload debugging is required.' })}
                            </p>
                        </div>
                        <select
                            className="select select-sm select-bordered w-48 text-xs font-normal focus:outline-none dark:bg-base-300 dark:text-base-content border-gray-200 dark:border-base-400"
                            value={config.proxy.experimental?.payload_storage_mode || 'simple'}
                            onChange={(e) => updateExperimentalConfig({ payload_storage_mode: e.target.value as 'simple' | 'full' })}
                        >
                            <option value="simple" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.payload_mode_simple', { defaultValue: 'Concise (Default)' })}</option>
                            <option value="full" className="text-xs dark:bg-base-300">{t('proxy.config.experimental.payload_mode_full', { defaultValue: 'Full Raw' })}</option>
                        </select>
                    </div>

                    <div className="grid grid-cols-1 md:grid-cols-2 gap-3">
                        <div className="flex flex-col gap-1 p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                            <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                {t('proxy.config.experimental.log_retention_days_label', { defaultValue: 'Request Log Retention (Days)' })}
                            </span>
                            <input
                                type="number"
                                min={1}
                                max={3650}
                                className="input input-sm input-bordered w-full text-xs"
                                value={config.proxy.experimental?.log_retention_days ?? 30}
                                onChange={(e) => updateExperimentalConfig({ log_retention_days: Math.max(1, parseInt(e.target.value) || 30) })}
                            />
                        </div>
                        <div className="flex flex-col gap-1 p-4 bg-gray-50 dark:bg-base-200 rounded-xl border border-gray-100 dark:border-base-300">
                            <span className="text-sm font-bold text-gray-900 dark:text-base-content">
                                {t('proxy.config.experimental.thinking_retention_days_label', { defaultValue: 'Thinking Block Retention (Sliding Window Days)' })}
                            </span>
                            <p className="text-[10px] text-gray-500 dark:text-gray-400">
                                {t('proxy.config.experimental.thinking_retention_days_desc', { defaultValue: 'Default 15-day sliding window. As long as the client session ID is active, each request refreshes the expiration window.' })}
                            </p>
                            <input
                                type="number"
                                min={1}
                                max={3650}
                                className="input input-sm input-bordered w-full text-xs"
                                value={config.proxy.experimental?.thinking_retention_days ?? DEFAULT_THINKING_RETENTION_DAYS}
                                onChange={(e) => updateExperimentalConfig({ thinking_retention_days: Math.max(1, parseInt(e.target.value) || DEFAULT_THINKING_RETENTION_DAYS) })}
                            />
                        </div>
                    </div>
                </div>
            </CollapsibleCard>
        </>
    );
}
