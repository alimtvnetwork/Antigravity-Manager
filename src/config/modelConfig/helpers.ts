import { MODEL_CONFIG } from './data';
import type { ModelConfig } from './types';

/**
 * 获取所有模型 ID 列表
 */
export const getAllModelIds = (): string[] => Object.keys(MODEL_CONFIG);

/**
 * 根据模型 ID 获取配置
 */
export const getModelConfig = (modelId: string): ModelConfig | undefined => {
    return MODEL_CONFIG[modelId.toLowerCase()];
};

/**
 * 动态计算组排序权重 (数字越小，优先级越高，越靠前)
 * 规则：
 * 1. Gemini 系列永远置顶 (Gemini 5 -> 50, Gemini 4 -> 60, Gemini 3 -> 70, Gemini 2.5 -> 75, 其他 Gemini -> 90)
 * 2. Claude 系列排第二 (权重 200)
 * 3. OpenAI 系列排第三 (权重 300)
 * 4. Other / Dynamic 兜底 (权重 500)
 */
export function getGroupPriority(group: string): number {
    const s = group.trim().toLowerCase();
    if (s.startsWith('gemini')) {
        const match = group.match(/gemini\s*(\d+(?:\.\d+)?)/i);
        const ver = match ? parseFloat(match[1]) : 0;
        // 数字代号越高的 Gemini 组，优先级越高 (5 -> 50, 4 -> 60, 3 -> 70, 2.5 -> 75)
        return Math.max(10, 100 - ver * 10);
    }
    if (s.includes('claude')) {
        return 200;
    }
    if (s.includes('openai') || s.includes('gpt')) {
        return 300;
    }
    if (s.includes('open source') || s.includes('opensource')) {
        return 310;
    }
    return 500;
}

/**
 * 泛化推导任意未知模型或未来模型的所属分组
 * 格式遵循：模型厂商-3.9/4/5-pro/flash/其他-xx
 */
export function inferModelGroup(modelIdOrName: string): string {
    const s = modelIdOrName.trim().toLowerCase();

    if (s.startsWith('gemini') || s.includes('gemini')) {
        const [maj, min] = extractModelVersion(s);
        if (maj >= 3) {
            return `Gemini ${maj}`; // 自动生成 Gemini 3, Gemini 4, Gemini 5 等动态高版本分组！
        } else if (maj === 2) {
            return min >= 5 ? 'Gemini 2.5' : 'Gemini 2';
        }
        return 'Gemini';
    }

    if (s.includes('claude') || s.includes('sonnet') || s.includes('opus') || s.includes('haiku')) {
        return 'Claude';
    }

    if (s.includes('gpt') || s.includes('openai') || s.includes('o1') || s.includes('o3') || s.includes('o4')) {
        return 'OpenAI';
    }

    return 'Other';
}

/**
 * 组排序权威优先级 (保留向后兼容导出)
 */
export const CANONICAL_GROUP_ORDER = ['Gemini 3', 'Gemini 2.5', 'Gemini', 'Claude', 'OpenAI', 'Open Source', 'Other', 'Dynamic'];

/**
 * 从模型名称或 ID 中提取数字代号版本 (用于降序排列)
 * 例如:
 * "gemini-3.8-flash" -> [3, 8]
 * "gemini-3.7-flash-high" -> [3, 7]
 * "gemini-3.1-pro" -> [3, 1]
 * "gemini-3-flash" -> [3, 0]
 * "claude-opus-4-6" -> [4, 6]
 * "claude-sonnet-4-5" -> [4, 5]
 * "claude-haiku-4" -> [4, 0]
 * "gpt-4o" -> [4, 5]
 * "gpt-4" -> [4, 0]
 * "gpt-3.5-turbo" -> [3, 5]
 */
export function extractModelVersion(idOrName: string): [number, number] {
    const s = idOrName.toLowerCase();

    // 特殊匹配 OpenAI
    if (s.includes('gpt-4o')) return [4, 5];
    if (s.includes('gpt-4')) return [4, 0];
    if (s.includes('gpt-3.5')) return [3, 5];

    // 匹配如 4-6, 4.6, 3.8, 3-8, 2.5, 3.5, 3.7
    const match = s.match(/(?:gemini|claude|opus|sonnet|haiku|gpt)[-_ ]*(\d+)[._-](\d+)/i)
        || s.match(/(\d+)[._-](\d+)/);

    if (match) {
        return [parseInt(match[1], 10), parseInt(match[2], 10)];
    }

    // 匹配单一主版本如 gemini-3, claude-4
    const singleMatch = s.match(/(?:gemini|claude|gpt)[-_ ]*(\d+)/i) || s.match(/(\d+)/);
    if (singleMatch) {
        return [parseInt(singleMatch[1], 10), 0];
    }

    return [0, 0];
}

/**
 * 档位细分优先级 (同版本内从高往低排)
 */
export function getModelTierPriority(id: string): number {
    const s = id.toLowerCase();
    let score = 0;

    // 1. 旗舰类型权重
    if (s.includes('opus')) score += 50;
    else if (s.includes('pro')) score += 40;
    else if (s.includes('sonnet')) score += 30;
    else if (s.includes('flash')) score += 20;
    else if (s.includes('haiku')) score += 10;
    else if (s.includes('image')) score += 5;

    // 2. 档位权重: high/agent > thinking > medium > (base) > low > lite
    if (s.includes('high') || s.includes('agent')) score += 9;
    else if (s.includes('thinking')) score += 7;
    else if (s.includes('medium')) score += 5;
    else if (s.includes('tiered')) score += 3;
    else if (s.includes('low')) score += 1;
    else if (s.includes('lite')) score -= 5;

    return score;
}

/**
 * 权威模型降序比较函数：
 * 1. 组优先：优先展示 Gemini 系列模型，再展示 Claude 模型，后接 Other/Dynamic
 * 2. 数字代号降序：从高到低倒序排（3.8 > 3.7 > 3.5 > 3.1, 4.6 > 4.5）
 * 3. 同版本内档位降序：high > medium > low
 */
export function compareModelsDesc<T extends { id: string; name?: string; group?: string }>(a: T, b: T): number {
    const grpA = a.group || inferModelGroup(a.id);
    const grpB = b.group || inferModelGroup(b.id);

    const prioA = getGroupPriority(grpA);
    const prioB = getGroupPriority(grpB);

    // 1. 组优先：优先展示 Gemini 系列模型 (数字代号高组优先，如 Gemini 5 > Gemini 4 > Gemini 3)，再展示 Claude 模型，后接 Other/Dynamic
    if (prioA !== prioB) {
        return prioA - prioB;
    }

    // 2. 组内提取数字代号版本，从高往低降序排序 (例如: 3.9 > 3.8 > 3.7 > 3.5 > 3.1)
    const [majA, minA] = extractModelVersion(a.id);
    const [majB, minB] = extractModelVersion(b.id);

    if (majA !== majB) {
        return majB - majA; // 降序：5 > 4 > 3 > 2
    }
    if (minA !== minB) {
        return minB - minA; // 降序：3.9 > 3.8 > 3.7 > 3.5 > 3.1
    }

    // 3. 版本相同时，按性能档位高低降序 (Pro > Flash > Lite, High > Med > Low)
    const tierA = getModelTierPriority(a.id);
    const tierB = getModelTierPriority(b.id);
    if (tierA !== tierB) {
        return tierB - tierA;
    }

    // 4. 兜底稳定排序
    return a.id.localeCompare(b.id);
}

/**
 * 对模型列表进行排序 (统一倒序/最新优先)
 * @param models 模型列表
 * @returns 排序后的模型列表
 */
export function sortModels<T extends { id: string; name?: string; group?: string }>(models: T[]): T[] {
    return [...models].sort(compareModelsDesc);
}
