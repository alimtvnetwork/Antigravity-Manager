/**
 * Model configuration — facade
 * The actual implementation is split into ./modelConfig/ submodules, and re-export is unified here to keep the import path unchanged.
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

// ── Model classification and protection keys (implemented in src/utils/modelCategory.ts, only re-export here)───

export {
    categorizeModel,
    getModelProtectionKey,
    getModelDisplayName,
    findQuotaModel,
    findImageQuotaModel,
    ensurePinnedImageSelector,
    DEFAULT_IMAGE_PIN_SELECTOR,
    DEFAULT_LATEST_MODEL_LABEL,
    DEFAULT_LATEST_MODEL_ID,
    resolveQuotaModels,
    type ModelCategory,
    type QuotaModelSelection,
} from '../utils/modelCategory';

