import type React from 'react';

/**
 * Model configuration interface
 */
export interface ModelConfig {
    /** Full display name of the model (as fallback or default presentation) */
    label: string;
    /** Short label for the model (for lists / cards) */
    shortLabel: string;
    /** Key name of the protected model */
    protectedKey: string;
    /** Model icon component */
    Icon: React.ComponentType<any>;
    /** i18n key name (for dynamic names) */
    i18nKey: string;
    /** Description i18n key (for detailed notes) */
    i18nDescKey: string;
    /** Series / category family */
    group: string;
    /** Optional tags (for filtering) */
    tags?: string[];
    /** Display priority (lower values appear first) */
    priority?: number;
    /** Capability tags: what the model excels at */
    capabilities?: Array<'chat' | 'reasoning' | 'code' | 'image' | 'vision' | 'agent' | 'fast' | 'long-context'>;
    /** Speed rating: how fast the model responds */
    speed?: 'nano' | 'fast' | 'balanced' | 'powerful';
}

/**
 * Model configuration mapping
 * The key is the model ID and the value is the model configuration
 */
