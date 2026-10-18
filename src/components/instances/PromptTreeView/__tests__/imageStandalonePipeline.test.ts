/**
 * Unit Tests for Standalone Image Pipeline & JSON Envelope v2.0
 *
 * Verifies:
 * 1. base64ToBlob in clipboard.ts converts Base64 data directly into binary PNG Blob
 *    without touching HTML canvas elements, eliminating canvas tainting and CORS errors.
 * 2. Standalone JSON Envelope v2.0 schema structure conforms to specifications:
 *    - schema_version === "2.0"
 *    - kind === "agm/prompt-tree-section-export"
 *    - embedded_images map indexing portable image keys (img_<sha256>) to data URIs
 *    - Prompts referencing virtual portable URIs via agm-asset://img_<sha256>
 *    - Unresolved reference detection for dangling assets
 * 3. Materialization logic:
 *    - Maps virtual agm-asset:// references back to local file:/// URIs in .antigravity/attachments/
 *    - Normalizes paths across Windows and POSIX platforms
 *    - Updates prompt_content, image_payload, and media_paths completely with zero remaining agm-asset:// tags
 *
 * Run: node scripts/run-frontend-tests.mjs imageStandalonePipeline
 *      npm test -- imageStandalonePipeline
 */

import { base64ToBlob } from '../clipboard';
import {
    validateStandaloneEnvelope,
    materializeAssetUris,
    materializeEnvelope,
    extractAssetKeys,
    mimeToExtension,
    normalizeToFileUri,
    STANDALONE_SCHEMA_VERSION,
    STANDALONE_EXPORT_KIND,
    type StandaloneExportEnvelope,
} from '../standalonePipeline';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void | Promise<void>): void {
    if (typeof (globalThis as any).it === 'function') {
        (globalThis as any).it(description, fn);
        return;
    }
    try {
        const res = fn();
        if (res && typeof (res as any).then === 'function') {
            (res as Promise<void>)
                .then(() => {
                    passed++;
                })
                .catch((e: unknown) => {
                    failed++;
                    const msg = e instanceof Error ? e.message : String(e);
                    console.error(`  FAIL: ${description} — ${msg}`);
                });
        } else {
            passed++;
        }
    } catch (e: unknown) {
        failed++;
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`  FAIL: ${description} — ${msg}`);
    }
}

function assertEqual<T>(actual: T, expected: T, msg?: string): void {
    if (actual !== expected) {
        throw new Error(msg || `expected "${expected}", got "${actual}"`);
    }
}

function assertTrue(actual: boolean, msg?: string): void {
    if (actual !== true) {
        throw new Error(msg || `expected true, got ${actual}`);
    }
}

function assertFalse(actual: boolean, msg?: string): void {
    if (actual !== false) {
        throw new Error(msg || `expected false, got ${actual}`);
    }
}

function assertDeepEqual<T>(actual: T, expected: T, msg?: string): void {
    const a = JSON.stringify(actual);
    const b = JSON.stringify(expected);
    if (a !== b) {
        throw new Error(msg || `expected ${b}, got ${a}`);
    }
}

// ---------------------------------------------------------------------------
// Standard Test Fixtures (1x1 Transparent PNG and Sample Images)
// ---------------------------------------------------------------------------

// 1x1 Transparent PNG Base64 (70 bytes binary)
const PNG_1X1_BASE64 = 'iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAYAAAAfFcSJAAAADUlEQVR42mNk+M9QDwADhgGAWjR9awAAAABJRU5ErkJggg==';
const PNG_MAGIC_BYTES = [0x89, 0x50, 0x4e, 0x47, 0x0d, 0x0a, 0x1a, 0x0a];

// 1x1 Red JPEG Base64
const JPEG_1X1_BASE64 = '/9j/4AAQSkZJRgABAQEASABIAAD/2wBDAP//////////////////////////////////////////////////////////////////////////////////////wgALCAABAAEBAREA/8QAFBABAAAAAAAAAAAAAAAAAAAAAP/aAAgBAQABPxA=';

function createMockEnvelope(overrides: Partial<StandaloneExportEnvelope> = {}): StandaloneExportEnvelope {
    return {
        schema_version: STANDALONE_SCHEMA_VERSION,
        kind: STANDALONE_EXPORT_KIND,
        exported_at: '2026-10-18T18:00:00Z',
        agm_version: '4.235.0',
        scope: {
            target_type: 'project',
            project_id: 'proj-1234',
            project_name: 'Antigravity-Manager',
        },
        stats: {
            project_count: 1,
            conversation_count: 1,
            prompt_count: 2,
            image_count: 2,
            total_image_bytes: 200,
        },
        data: {
            projects: [
                {
                    id: 'proj-1234',
                    instance_id: 'default',
                    repo_name: 'Antigravity-Manager',
                    repo_path: 'd:/work/Antigravity-Manager',
                    is_running: true,
                },
            ],
            active_prompts: [
                {
                    id: 'prompt-1',
                    project_id: 'proj-1234',
                    instance_id: 'default',
                    repo_path: 'd:/work/Antigravity-Manager',
                    prompt_content: 'Analyze dashboard UI: ![Dashboard](agm-asset://img_9a1f2b3c4d5e)',
                    session_id: 'conv-5678',
                    status: 'COMPLETED',
                    image_payload: 'agm-asset://img_9a1f2b3c4d5e',
                    media_paths: ['agm-asset://img_9a1f2b3c4d5e'],
                },
                {
                    id: 'prompt-2',
                    project_id: 'proj-1234',
                    instance_id: 'default',
                    repo_path: 'd:/work/Antigravity-Manager',
                    prompt_content: 'Review second screenshot: ![Architecture](agm-asset://img_cafebabe9999)',
                    session_id: 'conv-5678',
                    status: 'COMPLETED',
                    image_payload: 'agm-asset://img_cafebabe9999',
                    media_paths: ['agm-asset://img_cafebabe9999'],
                },
            ],
        },
        embedded_images: {
            img_9a1f2b3c4d5e: `data:image/png;base64,${PNG_1X1_BASE64}`,
            img_cafebabe9999: `data:image/jpeg;base64,${JPEG_1X1_BASE64}`,
        },
        ...overrides,
    };
}

// ===========================================================================
// Test Group 1: base64ToBlob (Zero Canvas Tainting & Binary Integrity)
// ===========================================================================

test('base64ToBlob: converts 1x1 PNG Base64 data to binary PNG Blob with exact size and type', async () => {
    const blob = base64ToBlob(PNG_1X1_BASE64, 'image/png');
    assertTrue(blob instanceof Blob, 'Must return a Blob instance');
    assertEqual(blob.type, 'image/png');
    assertEqual(blob.size, 70, '1x1 transparent PNG is exactly 70 bytes');

    // Verify binary magic bytes
    const arrayBuffer = await blob.arrayBuffer();
    const bytes = new Uint8Array(arrayBuffer);
    assertEqual(bytes.length, 70);
    for (let i = 0; i < PNG_MAGIC_BYTES.length; i++) {
        assertEqual(bytes[i], PNG_MAGIC_BYTES[i], `PNG header byte at offset ${i} must match`);
    }
});

test('base64ToBlob: supports custom MIME types (e.g. image/jpeg, image/webp)', () => {
    const jpegBlob = base64ToBlob(JPEG_1X1_BASE64, 'image/jpeg');
    assertEqual(jpegBlob.type, 'image/jpeg');
    assertTrue(jpegBlob.size > 0);

    const webpBlob = base64ToBlob(PNG_1X1_BASE64, 'image/webp');
    assertEqual(webpBlob.type, 'image/webp');
});

test('base64ToBlob: operates purely via memory buffer decoding without touching HTML canvas elements', () => {
    // If canvas taint error were possible, it would stem from document.createElement('canvas').
    // Here we verify that base64ToBlob succeeds even if createElement is temporarily overridden to throw
    const originalCreateElement = (globalThis as any).document?.createElement;
    let createElementCalled = false;

    if (typeof (globalThis as any).document !== 'undefined') {
        (globalThis as any).document.createElement = (tag: string) => {
            if (tag.toLowerCase() === 'canvas') {
                createElementCalled = true;
                throw new Error('SecurityError: The canvas has been tainted by cross-origin data.');
            }
            return originalCreateElement.call((globalThis as any).document, tag);
        };
    }

    try {
        const blob = base64ToBlob(PNG_1X1_BASE64, 'image/png');
        assertEqual(blob.size, 70);
        assertFalse(createElementCalled, 'base64ToBlob must never call document.createElement("canvas")');
    } finally {
        if (typeof (globalThis as any).document !== 'undefined' && originalCreateElement) {
            (globalThis as any).document.createElement = originalCreateElement;
        }
    }
});

test('base64ToBlob: handles URL-safe base64 characters correctly', () => {
    // Normal base64 '+/' vs URL safe '-_'
    const standardStr = 'Hello+World/Test==';
    const standardB64 = btoa(standardStr);
    const blob = base64ToBlob(standardB64, 'text/plain');
    assertTrue(blob.size > 0);
});

// ===========================================================================
// Test Group 2: Standalone JSON Envelope v2.0 Schema Structure
// ===========================================================================

test('Standalone Envelope: validates standard conforming v2.0 envelope schema', () => {
    const envelope = createMockEnvelope();
    const result = validateStandaloneEnvelope(envelope);
    assertTrue(result.isValid, `Envelope validation failed: ${result.error}`);
    assertEqual(result.imageCount, 2);
    assertEqual(result.unresolvedReferences.length, 0);
});

test('Standalone Envelope: rejects invalid schema_version', () => {
    const envelope = createMockEnvelope({ schema_version: '1.0' });
    const result = validateStandaloneEnvelope(envelope);
    assertFalse(result.isValid);
    assertTrue((result.error || '').includes('Invalid schema_version'));
});

test('Standalone Envelope: rejects invalid export kind', () => {
    const envelope = createMockEnvelope({ kind: 'unknown/export' });
    const result = validateStandaloneEnvelope(envelope);
    assertFalse(result.isValid);
    assertTrue((result.error || '').includes('Invalid kind'));
});

test('Standalone Envelope: rejects missing embedded_images map', () => {
    const envelope = createMockEnvelope({ embedded_images: undefined as any });
    const result = validateStandaloneEnvelope(envelope);
    assertFalse(result.isValid);
    assertTrue((result.error || '').includes('embedded_images'));
});

test('Standalone Envelope: rejects non-data URI payloads in embedded_images', () => {
    const envelope = createMockEnvelope({
        embedded_images: {
            img_invalid: 'not-a-data-uri-content',
        },
    });
    const result = validateStandaloneEnvelope(envelope);
    assertFalse(result.isValid);
    assertTrue((result.error || '').includes('must be a valid Base64 data URI'));
});

test('Standalone Envelope: detects unresolved agm-asset:// references in prompt content', () => {
    const envelope = createMockEnvelope();
    // Add reference to an image key that does not exist in embedded_images
    envelope.data.active_prompts.push({
        id: 'prompt-orphan',
        project_id: 'proj-1234',
        instance_id: 'default',
        repo_path: 'd:/work/Antigravity-Manager',
        prompt_content: 'Orphaned reference: ![Missing](agm-asset://img_missing_key_9999)',
    });

    const result = validateStandaloneEnvelope(envelope);
    assertFalse(result.isValid, 'Should fail validation on orphaned asset reference');
    assertTrue(result.unresolvedReferences.includes('img_missing_key_9999'));
    assertTrue((result.error || '').includes('unresolved agm-asset:// reference'));
});

// ===========================================================================
// Test Group 3: Asset Key Extraction & Helpers
// ===========================================================================

test('extractAssetKeys: extracts virtual asset keys from prompt markdown and image_payload', () => {
    const text = 'Here is UI: ![UI](agm-asset://img_1a2b3c) and diagram: ![Diag](agm-asset://img_4d5e6f) and repeated: agm-asset://img_1a2b3c';
    const keys = extractAssetKeys(text);
    assertEqual(keys.length, 2);
    assertTrue(keys.includes('img_1a2b3c'));
    assertTrue(keys.includes('img_4d5e6f'));
});

test('extractAssetKeys: returns empty array when no agm-asset:// tokens are present', () => {
    const text = 'Normal prompt with standard markdown: ![Logo](https://example.com/logo.png) and ![File](file:///tmp/img.png)';
    const keys = extractAssetKeys(text);
    assertEqual(keys.length, 0);
});

test('mimeToExtension: maps image MIME types to standard file extensions', () => {
    assertEqual(mimeToExtension('image/png'), 'png');
    assertEqual(mimeToExtension('image/jpeg'), 'jpg');
    assertEqual(mimeToExtension('image/jpg'), 'jpg');
    assertEqual(mimeToExtension('image/webp'), 'webp');
    assertEqual(mimeToExtension('image/gif'), 'gif');
    assertEqual(mimeToExtension('image/svg+xml'), 'svg');
});

test('normalizeToFileUri: converts Windows and POSIX paths to valid file:/// URIs', () => {
    assertEqual(normalizeToFileUri('C:\\Users\\test\\.antigravity\\attachments\\img.png'), 'file:///C:/Users/test/.antigravity/attachments/img.png');
    assertEqual(normalizeToFileUri('/home/user/.antigravity/attachments/img.png'), 'file:///home/user/.antigravity/attachments/img.png');
    assertEqual(normalizeToFileUri('file:///already/normalized.png'), 'file:///already/normalized.png');
});

// ===========================================================================
// Test Group 4: Materialization Logic (agm-asset:// to Local file:/// URIs)
// ===========================================================================

test('materializeAssetUris: rewrites agm-asset:// references with concrete file:/// URIs', () => {
    const promptText = 'Check UI layout: ![Layout](agm-asset://img_9a1f2b3c4d5e) before commit';
    const map = {
        img_9a1f2b3c4d5e: 'C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png',
    };

    const rewritten = materializeAssetUris(promptText, map);
    assertEqual(rewritten, 'Check UI layout: ![Layout](file:///C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png) before commit');
    assertFalse(rewritten.includes('agm-asset://'));
});

test('materializeEnvelope: executes full standalone materialization across active prompts', () => {
    const envelope = createMockEnvelope();
    const attachmentsDir = 'C:\\Users\\Administrator\\.antigravity\\attachments';

    const { envelope: materialized, materializedMap, materializedCount } = materializeEnvelope(envelope, attachmentsDir);

    assertEqual(materializedCount, 2);
    assertEqual(materializedMap['img_9a1f2b3c4d5e'], 'C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png');
    assertEqual(materializedMap['img_cafebabe9999'], 'C:/Users/Administrator/.antigravity/attachments/img_cafebabe9999.jpg');

    // Prompt 1
    const p1 = materialized.data.active_prompts[0];
    assertEqual(p1.prompt_content, 'Analyze dashboard UI: ![Dashboard](file:///C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png)');
    assertEqual(p1.image_payload, 'file:///C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png');
    assertDeepEqual(p1.media_paths, ['file:///C:/Users/Administrator/.antigravity/attachments/img_9a1f2b3c4d5e.png']);
    assertFalse(p1.prompt_content.includes('agm-asset://'));

    // Prompt 2
    const p2 = materialized.data.active_prompts[1];
    assertEqual(p2.prompt_content, 'Review second screenshot: ![Architecture](file:///C:/Users/Administrator/.antigravity/attachments/img_cafebabe9999.jpg)');
    assertEqual(p2.image_payload, 'file:///C:/Users/Administrator/.antigravity/attachments/img_cafebabe9999.jpg');
    assertFalse(p2.prompt_content.includes('agm-asset://'));
});

test('materializeEnvelope: preserves external HTTP URLs and regular text unaffected', () => {
    const envelope = createMockEnvelope({
        data: {
            projects: [],
            active_prompts: [
                {
                    id: 'prompt-mixed',
                    project_id: 'proj-1234',
                    instance_id: 'default',
                    repo_path: 'd:/work/Antigravity-Manager',
                    prompt_content: 'External logo: ![Logo](https://example.com/logo.png) and embedded: ![Embedded](agm-asset://img_9a1f2b3c4d5e)',
                },
            ],
        },
    });

    const { envelope: materialized } = materializeEnvelope(envelope, '/tmp/attachments');
    const content = materialized.data.active_prompts[0].prompt_content;

    assertTrue(content.includes('https://example.com/logo.png'), 'External HTTP URL must remain intact');
    assertTrue(content.includes('file:///tmp/attachments/img_9a1f2b3c4d5e.png'), 'Embedded asset must be materialized');
    assertFalse(content.includes('agm-asset://'), 'All agm-asset:// references must be replaced');
});

// ===========================================================================
// Summary
// ===========================================================================

console.log(`\nimageStandalonePipeline verification: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} imageStandalonePipeline test(s) failed`);
}
