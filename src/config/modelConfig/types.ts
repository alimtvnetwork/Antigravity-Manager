import type React from 'react';

/**
 * 模型配置接口
 */
export interface ModelConfig {
    /** 模型完整显示名称 (作为回退或默认展示) */
    label: string;
    /** 模型简短标签 (用于列表/卡片) */
    shortLabel: string;
    /** 保护模型的键名 */
    protectedKey: string;
    /** 模型图标组件 */
    Icon: React.ComponentType<any>;
    /** 国际化键名 (用于动态名称) */
    i18nKey: string;
    /** 描述信息键名 (用于详细说明) */
    i18nDescKey: string;
    /** 所属系列/分组 */
    group: string;
    /** 选填标签 (用于筛选) */
    tags?: string[];
    /** 展示优先级 (数字越小越靠前) */
    priority?: number;
    /** 能力标签: 模型擅长什么 */
    capabilities?: Array<'chat' | 'reasoning' | 'code' | 'image' | 'vision' | 'agent' | 'fast' | 'long-context'>;
    /** 速度档位: 响应有多快 */
    speed?: 'nano' | 'fast' | 'balanced' | 'powerful';
}

/**
 * 模型配置映射
 * 键为模型 ID，值为模型配置
 */
