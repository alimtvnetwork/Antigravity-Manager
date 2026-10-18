/**
 * Standalone Image Pipeline & JSON Envelope v2.0 Specifications
 *
 * Implements envelope schema validation, virtual asset reference extraction (agm-asset://),
 * and materialization to normalized local file:/// URIs for offline portability.
 */

export const STANDALONE_SCHEMA_VERSION = '2.0';
export const STANDALONE_EXPORT_KIND = 'agm/prompt-tree-section-export';

export interface StandaloneScope {
    target_type: 'full_tree' | 'project' | 'conversation' | 'prompt_subtree' | string;
    project_id?: string;
    project_name?: string;
    conversation_id?: string;
    prompt_ids?: string[];
}

export interface StandaloneStats {
    path?: string;
    project_count: number;
    conversation_count: number;
    prompt_count: number;
    image_count: number;
    total_image_bytes: number;
}

export interface StandalonePromptRecord {
    id: string;
    project_id: string;
    instance_id: string;
    repo_path: string;
    prompt_content: string;
    model?: string;
    session_id?: string;
    status?: string;
    created_at?: number;
    updated_at?: number;
    image_payload?: string | null;
    media_paths?: string[];
}

export interface StandaloneProjectRecord {
    id: string;
    instance_id: string;
    repo_name: string;
    repo_path: string;
    workspace_storage_path?: string;
    is_running?: boolean;
    last_detected_at?: number;
}

export interface StandaloneExportData {
    projects: StandaloneProjectRecord[];
    active_prompts: StandalonePromptRecord[];
    project_sequences?: unknown[];
    conversation_sequences?: unknown[];
}

export interface StandaloneExportEnvelope {
    schema_version: string;
    kind: string;
    exported_at: string;
    agm_version: string;
    scope: StandaloneScope;
    stats: StandaloneStats;
    data: StandaloneExportData;
    embedded_images: Record<string, string>; // image_key (e.g. img_9a1f2b3c) -> data:image/png;base64,...
}

export interface StandaloneValidationResult {
    isValid: boolean;
    error?: string;
    imageCount: number;
    unresolvedReferences: string[];
}

/**
 * Extracts all virtual asset keys referenced via `agm-asset://<key>` from text.
 */
export function extractAssetKeys(text: string): string[] {
    if (!text) return [];
    const regex = /agm-asset:\/\/([a-zA-Z0-9_-]+)/g;
    const keys: string[] = [];
    let match: RegExpExecArray | null;
    while ((match = regex.exec(text)) !== null) {
        if (match[1] && !keys.includes(match[1])) {
            keys.push(match[1]);
        }
    }
    return keys;
}

/**
 * Inspects a candidate payload to determine if it complies with the Standalone v2.0 envelope schema.
 */
export function validateStandaloneEnvelope(candidate: unknown): StandaloneValidationResult {
    if (!candidate || typeof candidate !== 'object') {
        return { isValid: false, error: 'Envelope payload must be a non-null object', imageCount: 0, unresolvedReferences: [] };
    }

    const env = candidate as Partial<StandaloneExportEnvelope>;

    if (env.schema_version !== STANDALONE_SCHEMA_VERSION) {
        return {
            isValid: false,
            error: `Invalid schema_version "${env.schema_version}". Expected "${STANDALONE_SCHEMA_VERSION}".`,
            imageCount: 0,
            unresolvedReferences: [],
        };
    }

    if (env.kind !== STANDALONE_EXPORT_KIND) {
        return {
            isValid: false,
            error: `Invalid kind "${env.kind}". Expected "${STANDALONE_EXPORT_KIND}".`,
            imageCount: 0,
            unresolvedReferences: [],
        };
    }

    if (!env.embedded_images || typeof env.embedded_images !== 'object' || Array.isArray(env.embedded_images)) {
        return {
            isValid: false,
            error: 'Missing or invalid "embedded_images" map',
            imageCount: 0,
            unresolvedReferences: [],
        };
    }

    if (!env.data || typeof env.data !== 'object' || !Array.isArray(env.data.active_prompts)) {
        return {
            isValid: false,
            error: 'Missing or invalid "data.active_prompts" array in envelope',
            imageCount: 0,
            unresolvedReferences: [],
        };
    }

    // Validate embedded image values are valid data URIs
    for (const [key, dataUri] of Object.entries(env.embedded_images)) {
        if (typeof dataUri !== 'string' || !dataUri.startsWith('data:image/')) {
            return {
                isValid: false,
                error: `Image entry "${key}" must be a valid Base64 data URI starting with "data:image/"`,
                imageCount: Object.keys(env.embedded_images).length,
                unresolvedReferences: [],
            };
        }
    }

    // Check for unresolved agm-asset:// references
    const availableKeys = new Set(Object.keys(env.embedded_images));
    const unresolved: string[] = [];

    for (const prompt of env.data.active_prompts) {
        const referencedInContent = extractAssetKeys(prompt.prompt_content || '');
        const referencedInPayload = prompt.image_payload ? extractAssetKeys(prompt.image_payload) : [];
        const referencedInMedia = prompt.media_paths ? prompt.media_paths.flatMap((p) => extractAssetKeys(p)) : [];

        const allReferenced = [...new Set([...referencedInContent, ...referencedInPayload, ...referencedInMedia])];
        for (const ref of allReferenced) {
            if (!availableKeys.has(ref) && !unresolved.includes(ref)) {
                unresolved.push(ref);
            }
        }
    }

    if (unresolved.length > 0) {
        return {
            isValid: false,
            error: `Found ${unresolved.length} unresolved agm-asset:// reference(s): ${unresolved.join(', ')}`,
            imageCount: availableKeys.size,
            unresolvedReferences: unresolved,
        };
    }

    return {
        isValid: true,
        imageCount: availableKeys.size,
        unresolvedReferences: [],
    };
}

/**
 * Derives the appropriate file extension from an image MIME type.
 */
export function mimeToExtension(mimeType: string): string {
    const clean = (mimeType || '').toLowerCase().trim();
    if (clean.includes('png')) return 'png';
    if (clean.includes('jpeg') || clean.includes('jpg')) return 'jpg';
    if (clean.includes('webp')) return 'webp';
    if (clean.includes('gif')) return 'gif';
    if (clean.includes('svg')) return 'svg';
    if (clean.includes('bmp')) return 'bmp';
    return 'png';
}

/**
 * Normalizes an attachments filesystem path into a valid file:/// URI.
 */
export function normalizeToFileUri(destPath: string): string {
    const forwardSlashed = destPath.replace(/\\/g, '/');
    if (forwardSlashed.startsWith('file:///')) {
        return forwardSlashed;
    }
    if (forwardSlashed.startsWith('/')) {
        return `file://${forwardSlashed}`;
    }
    // Windows path e.g. C:/... -> file:///C:/...
    return `file:///${forwardSlashed}`;
}

/**
 * Materializes virtual `agm-asset://<key>` references within markdown or payload text
 * to concrete local `file:///` URIs based on the provided materialized map.
 */
export function materializeAssetUris(text: string, materializedMap: Record<string, string>): string {
    if (!text) return '';
    let result = text;
    for (const [key, localPathOrUri] of Object.entries(materializedMap)) {
        const assetMarker = `agm-asset://${key}`;
        const targetUri = normalizeToFileUri(localPathOrUri);
        result = result.split(assetMarker).join(targetUri);
    }
    return result;
}

/**
 * Simulates or executes envelope materialization against a specified attachments directory.
 * Maps every entry in `embedded_images` to its target file path and rewrites all prompt content,
 * payloads, and media paths.
 */
export function materializeEnvelope(
    envelope: StandaloneExportEnvelope,
    attachmentsDirectory: string
): {
    envelope: StandaloneExportEnvelope;
    materializedMap: Record<string, string>;
    materializedCount: number;
} {
    const normDir = attachmentsDirectory.replace(/\\/g, '/').replace(/\/+$/, '');
    const materializedMap: Record<string, string> = {};

    // 1. Map each image key to destination local file URI
    for (const [key, dataUri] of Object.entries(envelope.embedded_images || {})) {
        const mimeMatch = dataUri.match(/^data:([^;]+);base64,/);
        const mime = mimeMatch ? mimeMatch[1] : 'image/png';
        const ext = mimeToExtension(mime);
        const destPath = `${normDir}/${key}.${ext}`;
        materializedMap[key] = destPath;
    }

    // 2. Clone envelope and rewrite references across prompts
    const clonedEnvelope: StandaloneExportEnvelope = JSON.parse(JSON.stringify(envelope));

    for (const prompt of clonedEnvelope.data.active_prompts) {
        prompt.prompt_content = materializeAssetUris(prompt.prompt_content, materializedMap);
        if (prompt.image_payload) {
            prompt.image_payload = materializeAssetUris(prompt.image_payload, materializedMap);
        }
        if (prompt.media_paths) {
            prompt.media_paths = prompt.media_paths.map((p) => materializeAssetUris(p, materializedMap));
        }
    }

    return {
        envelope: clonedEnvelope,
        materializedMap,
        materializedCount: Object.keys(materializedMap).length,
    };
}
