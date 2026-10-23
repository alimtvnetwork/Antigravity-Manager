import type { ThinkingBudgetConfig } from "../../../types/config";

export interface ThinkingBudgetProps {
    config?: ThinkingBudgetConfig;
    onChange: (config: ThinkingBudgetConfig) => void;
    onSave?: () => Promise<void> | void;
    thinkingStoreEnabled?: boolean;
    onThinkingStoreChange?: (enabled: boolean) => void;
    thinkingMaxMemoryTurns?: number;
    onThinkingMaxMemoryTurnsChange?: (turns: number) => void;
    thinkingRetentionDays?: number;
    onThinkingRetentionDaysChange?: (days: number) => void;
}

interface ConcurrencyGuidePreset {
    key: string;
    icon: string;
    titleKey: string;
    titleDefault: string;
    descKey: string;
    descDefault: string;
}

export const CONCURRENCY_GUIDE_PRESETS: ConcurrencyGuidePreset[] = [
    {
        key: "1g",
        icon: "🖥️",
        titleKey: "proxy.config.thinking_budget.guide_preset_1g_title",
        titleDefault: "1GB memory lightweight server:",
        descKey: "proxy.config.thinking_budget.guide_preset_1g_desc",
        descDefault: "It is recommended to fill in <1>100 ~ 200 rounds</1>. Hundreds of concurrent sessions consume only about 50MB of memory, making it extremely burst-resistant.",
    },
    {
        key: "team",
        icon: "👥",
        titleKey: "proxy.config.thinking_budget.guide_preset_team_title",
        titleDefault: "Individual ~ Self-use team of 10 people:",
        descKey: "proxy.config.thinking_budget.guide_preset_team_desc",
        descDefault: "It is recommended to fill in <1>600 ~ 1000 rounds</1>. Thousands of rounds of historical conversations are resident in physical memory and processed in 0ms.",
    },
    {
        key: "enterprise",
        icon: "🏢",
        titleKey: "proxy.config.thinking_budget.guide_preset_enterprise_title",
        titleDefault: "100 people enterprise-level concurrency (2G-4G):",
        descKey: "proxy.config.thinking_budget.guide_preset_enterprise_desc",
        descDefault: "It is recommended to fill in <1>300 ~ 600 rounds</1>. 95%+ requests hit RAM for ultimate performance and absolute robustness.",
    },
    {
        key: "relay",
        icon: "🌐",
        titleKey: "proxy.config.thinking_budget.guide_preset_relay_title",
        titleDefault: "1K+ users public transit station (4G-8G):",
        descKey: "proxy.config.thinking_budget.guide_preset_relay_desc",
        descDefault: "It is recommended to fill in <1>150~300 rounds</1>. Memory priority is given to the long connection pool, and long conversation cold history is entrusted to SQLite.",
    },
    {
        key: "cluster",
        icon: "🚀",
        titleKey: "proxy.config.thinking_budget.guide_preset_cluster_title",
        titleDefault: "1W+ ~ 10W+ massive concurrent cluster:",
        descKey: "proxy.config.thinking_budget.guide_preset_cluster_desc",
        descDefault: "It is recommended to fill in <1>50~100 rounds</1>. A single machine can carry tens of thousands of concurrent sessions without pressure, and WAL can provide high-speed indexing and lock-free second-level response.",
    },
];

export const DEFAULT_CONFIG: ThinkingBudgetConfig = {
    control_source: "gateway",
    flash_mode: "custom",
    flash_low: 1024,
    flash_medium: 4096,
    flash_high: 16384,
    flash_tiered: -1,

    pro_mode: "custom",
    pro_low: 1001,
    pro_high: 10001,

    claude_mode: "custom",
    claude_budget: 16384,
    claude_low: 1024,
    claude_medium: 4096,
    claude_high: 16384,

    mode: "custom",
    custom_value: 24576,
    custom_low: 1024,
    custom_medium: 4096,
    custom_high: 16384,
    custom_tiered: -1,
};

export type BudgetFieldKey =
    | "flash_low"
    | "flash_medium"
    | "flash_high"
    | "flash_tiered"
    | "pro_low"
    | "pro_high"
    | "claude_budget"
    | "claude_low"
    | "claude_medium"
    | "claude_high";

export const BUDGET_DEFAULTS: Record<BudgetFieldKey, number> = {
    flash_low: 1024,
    flash_medium: 4096,
    flash_high: 16384,
    flash_tiered: -1,
    pro_low: 1001,
    pro_high: 10001,
    claude_budget: 16384,
    claude_low: 1024,
    claude_medium: 4096,
    claude_high: 16384,
};

