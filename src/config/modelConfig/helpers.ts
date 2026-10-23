import { MODEL_CONFIG } from './data';
import type { ModelConfig } from './types';

/**
 * Get a list of all model IDs
 */
export const getAllModelIds = (): string[] => Object.keys(MODEL_CONFIG);

/**
 * Get configuration based on model ID
 */
export const getModelConfig = (modelId: string): ModelConfig | undefined => {
    return MODEL_CONFIG[modelId.toLowerCase()];
};

/**
 * Dynamically calculate group sorting weight (the smaller the number, the higher the priority, the higher it is)
 * rule:
 * 1. The Gemini series is always at the top (Gemini 5 -> 50, Gemini 4 -> 60, Gemini 3 -> 70, Gemini 2.5 -> 75, other Gemini -> 90)
 * 2. Claude series ranks second (weight 200)
 * 3. OpenAI series ranks third (weight 300)
 * 4. Other / Dynamic (weight 500)
 */
export function getGroupPriority(group: string): number {
    const s = group.trim().toLowerCase();
    if (s.startsWith('gemini')) {
        const match = group.match(/gemini\s*(\d+(?:\.\d+)?)/i);
        const ver = match ? parseFloat(match[1]) : 0;
        // The higher the numerical code of the Gemini group, the higher the priority (5 -> 50, 4 -> 60, 3 -> 70, 2.5 -> 75)
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
 * Generalize the derivation of the group to which any unknown model or future model belongs
 * The format follows: model manufacturer-3.9/4/5-pro/flash/other-xx
 */
export function inferModelGroup(modelIdOrName: string): string {
    const s = modelIdOrName.trim().toLowerCase();

    if (s.startsWith('gemini') || s.includes('gemini')) {
        const [maj, min] = extractModelVersion(s);
        if (maj >= 3) {
            return `Gemini ${maj}`; // Automatically generate dynamic high-version groups such as Gemini 3, Gemini 4, Gemini 5, etc.!
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
 * Group sort authority priority (preserves backwards compatibility exports)
 */
export const CANONICAL_GROUP_ORDER = ['Gemini 3', 'Gemini 2.5', 'Gemini', 'Claude', 'OpenAI', 'Open Source', 'Other', 'Dynamic'];

/**
 * Extract numeric codename version from model name or ID (for descending order)
 * For example:
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

    // Special Match OpenAI
    if (s.includes('gpt-4o')) return [4, 5];
    if (s.includes('gpt-4')) return [4, 0];
    if (s.includes('gpt-3.5')) return [3, 5];

    // Matches like 4-6, 4.6, 3.8, 3-8, 2.5, 3.5, 3.7
    const match = s.match(/(?:gemini|claude|opus|sonnet|haiku|gpt)[-_ ]*(\d+)[._-](\d+)/i)
        || s.match(/(\d+)[._-](\d+)/);

    if (match) {
        return [parseInt(match[1], 10), parseInt(match[2], 10)];
    }

    // Matches a single major version such as gemini-3, claude-4
    const singleMatch = s.match(/(?:gemini|claude|gpt)[-_ ]*(\d+)/i) || s.match(/(\d+)/);
    if (singleMatch) {
        return [parseInt(singleMatch[1], 10), 0];
    }

    return [0, 0];
}

/**
 * Gear segmentation priority (from high to low within the same version)
 */
export function getModelTierPriority(id: string): number {
    const s = id.toLowerCase();
    let score = 0;

    // 1. Flagship type weight
    if (s.includes('opus')) score += 50;
    else if (s.includes('pro')) score += 40;
    else if (s.includes('sonnet')) score += 30;
    else if (s.includes('flash')) score += 20;
    else if (s.includes('haiku')) score += 10;
    else if (s.includes('image')) score += 5;

    // 2. Gear weight: high/agent > thinking > medium > (base) > low > lite
    if (s.includes('high') || s.includes('agent')) score += 9;
    else if (s.includes('thinking')) score += 7;
    else if (s.includes('medium')) score += 5;
    else if (s.includes('tiered')) score += 3;
    else if (s.includes('low')) score += 1;
    else if (s.includes('lite')) score -= 5;

    return score;
}

/**
 * Authoritative model descending comparison function:
 * 1. Group priority: Gemini series models will be displayed first, followed by Claude models, followed by Other/Dynamic
 * 2. Descending order of numerical codes: from high to low (3.8 > 3.7 > 3.5 > 3.1, 4.6 > 4.5)
 * 3. Descending order of gears within the same version: high > medium > low
 */
export function compareModelsDesc<T extends { id: string; name?: string; group?: string }>(a: T, b: T): number {
    const grpA = a.group || inferModelGroup(a.id);
    const grpB = b.group || inferModelGroup(b.id);

    const prioA = getGroupPriority(grpA);
    const prioB = getGroupPriority(grpB);

    // 1. Group priority: Display the Gemini series models first (groups with higher numerical codes are given priority, such as Gemini 5 > Gemini 4 > Gemini 3), then display the Claude model, followed by Other/Dynamic
    if (prioA !== prioB) {
        return prioA - prioB;
    }

    // 2. Extract the numerical code versions in the group and sort them in descending order from high to low (for example: 3.9 > 3.8 > 3.7 > 3.5 > 3.1)
    const [majA, minA] = extractModelVersion(a.id);
    const [majB, minB] = extractModelVersion(b.id);

    if (majA !== majB) {
        return majB - majA; // Descending order: 5 > 4 > 3 > 2
    }
    if (minA !== minB) {
        return minB - minA; // Descending order: 3.9 > 3.8 > 3.7 > 3.5 > 3.1
    }

    // 3. When the versions are the same, sort by performance level in descending order (Pro > Flash > Lite, High > Med > Low)
    const tierA = getModelTierPriority(a.id);
    const tierB = getModelTierPriority(b.id);
    if (tierA !== tierB) {
        return tierB - tierA;
    }

    // 4. Stable sorting
    return a.id.localeCompare(b.id);
}

/**
 * Sort the model list (unified descending order/latest first)
 * @param models model list
 * @returns sorted model list
 */
export function sortModels<T extends { id: string; name?: string; group?: string }>(models: T[]): T[] {
    return [...models].sort(compareModelsDesc);
}
