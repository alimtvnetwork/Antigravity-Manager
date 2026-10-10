import type { AutoProfileSwitcherConfig } from "../../types/config";

export interface AutoSwitcherSettingsProps {
    config?: AutoProfileSwitcherConfig;
    onChange: (config: AutoProfileSwitcherConfig) => void;
}

const DEFAULT_CONFIG: AutoProfileSwitcherConfig = {
    is_enabled: true,
    check_interval_seconds: 300,
    low_quota_threshold_percent: 15.0,
    target_model: 'gemini-3.8-flash-high',
    has_auto_resume: true,
    cooldown_seconds: 180,
    auto_resume_recent_prompts: true,
    auto_focus_window: false,
    auto_reopen_on_switch: true,
    watchdog_interval_seconds: 120,
    prompt_recency_threshold_seconds: 3600,
    caution_interval_seconds: 60,
    critical_interval_seconds: 40,
    critical_threshold_percent: 12.0,
    account_cooldown_minutes: 60,
    ultra_tier_multiplier: 4.0,
    pro_tier_multiplier: 2.0,
    free_tier_multiplier: 1.0,
};
