/**
 * 模型配置 — facade
 * 实际实现拆分至 ./modelConfig/ 子模块，此处统一 re-export 以保持导入路径不变。
 */
export type { ModelConfig } from './modelConfig/types';
export { MODEL_CONFIG } from './modelConfig/data';
export {
    getAllModelIds,
    getModelConfig,
    getGroupPriority,
    inferModelGroup,
    CANONICAL_GROUP_ORDER,
    extractModelVersion,
    getModelTierPriority,
    compareModelsDesc,
    sortModels,
} from './modelConfig/helpers';

// ── 模型分类与保护键（实现在 src/utils/modelCategory.ts，此处只 re-export）───

export {
    categorizeModel,
    getModelProtectionKey,
    getModelDisplayName,
    findQuotaModel,
    findImageQuotaModel,
    ensurePinnedImageSelector,
    DEFAULT_IMAGE_PIN_SELECTOR,
    resolveQuotaModels,
    type ModelCategory,
    type QuotaModelSelection,
} from '../utils/modelCategory';

