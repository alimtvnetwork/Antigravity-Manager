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
        titleDefault: "1GB 内存轻量服务器:",
        descKey: "proxy.config.thinking_budget.guide_preset_1g_desc",
        descDefault: "推荐填写 <1>100 ~ 200 轮</1>。几百个并发会话仅消耗约 50MB 内存，极端抗爆。",
    },
    {
        key: "team",
        icon: "👥",
        titleKey: "proxy.config.thinking_budget.guide_preset_team_title",
        titleDefault: "个人 ~ 10 人自用团队:",
        descKey: "proxy.config.thinking_budget.guide_preset_team_desc",
        descDefault: "推荐填写 <1>600 ~ 1000 轮</1>。数千轮历史对话常驻物理内存 0ms 闪电直出。",
    },
    {
        key: "enterprise",
        icon: "🏢",
        titleKey: "proxy.config.thinking_budget.guide_preset_enterprise_title",
        titleDefault: "100 人企业级并发 (2G-4G):",
        descKey: "proxy.config.thinking_budget.guide_preset_enterprise_desc",
        descDefault: "推荐填写 <1>300 ~ 600 轮</1>。95%+ 请求命中 RAM，兼具极致性能与绝对稳健。",
    },
    {
        key: "relay",
        icon: "🌐",
        titleKey: "proxy.config.thinking_budget.guide_preset_relay_title",
        titleDefault: "1K+ 用户公共中转站 (4G-8G):",
        descKey: "proxy.config.thinking_budget.guide_preset_relay_desc",
        descDefault: "推荐填写 <1>150 ~ 300 轮</1>。内存优先倾斜给长连接池，长对话冷历史托付 SQLite。",
    },
    {
        key: "cluster",
        icon: "🚀",
        titleKey: "proxy.config.thinking_budget.guide_preset_cluster_title",
        titleDefault: "1W+ ~ 10W+ 海量并发集群:",
        descKey: "proxy.config.thinking_budget.guide_preset_cluster_desc",
        descDefault: "推荐填写 <1>50 ~ 100 轮</1>。单机无压承载数万并发会话，WAL 高速索引并发无锁秒级响应。",
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

