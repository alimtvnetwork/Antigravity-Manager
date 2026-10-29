/**
 * Unit tests for Prompt Tree Copy body resolution.
 *
 * Copy must prefer the untruncated full prompt over a 200-word preview,
 * while still honoring a user-edited body that is not the truncated preview.
 *
 * Run: npm test -- promptCopy
 */

if (typeof (globalThis as { window?: unknown }).window === 'undefined') {
    const store = new Map<string, string>();
    const mockStorage = {
        getItem: (k: string) => store.get(k) ?? null,
        setItem: (k: string, v: string) => {
            store.set(k, String(v));
        },
        removeItem: (k: string) => {
            store.delete(k);
        },
        clear: () => {
            store.clear();
        },
    };
    (globalThis as { window?: unknown }).window = {
        localStorage: mockStorage,
        location: { pathname: '/prompts', hash: '', search: '', href: 'http://localhost/prompts' },
        __TAURI_INTERNALS__: { invoke: async () => 'ok' },
    };
    (globalThis as { localStorage?: unknown }).localStorage = mockStorage;
}

import { resolvePromptBody } from '../usePromptTreePromptActions';
import type { AgmConversationNode } from '../types';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    try {
        fn();
        passed++;
    } catch (e: unknown) {
        failed++;
        const msg = e instanceof Error ? e.message : String(e);
        console.error(`  FAIL: ${description} — ${msg}`);
    }
}

function assertEqual<T>(actual: T, expected: T): void {
    if (actual !== expected) {
        throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
    }
}

function conv(partial: Partial<AgmConversationNode>): AgmConversationNode {
    return {
        conversation_id: 'c1',
        title: 'GitMap User Change Command',
        ...partial,
    } as AgmConversationNode;
}

test('resolvePromptBody prefers full_prompt_text over truncated preview preferred text', () => {
    const full = 'instead of @[.agents/skills/execute-parent-task-with-n-steps-v7]\n\nWrite as\n\n@[execute-parent-task-with-n-steps-v7](file;.agents/skills/execute-parent-task-with-n-steps-v7)';
    const preview = 'instead of @[.agents/skills/execute-parent-task-with-n-steps-v7]';
    assertEqual(
        resolvePromptBody(conv({ full_prompt_text: full, prompt_preview_200w: preview }), preview),
        full
    );
});

test('resolvePromptBody uses a user-edited body that is not the truncated preview', () => {
    const full = 'original full prompt text that is longer than the preview';
    const preview = 'original full prompt';
    const edited = 'user edited the prompt in the inspector';
    assertEqual(
        resolvePromptBody(conv({ full_prompt_text: full, prompt_preview_200w: preview }), edited),
        edited
    );
});

test('resolvePromptBody falls back to preview when full text is missing', () => {
    const preview = 'short preview only';
    assertEqual(
        resolvePromptBody(conv({ prompt_preview_200w: preview }), ''),
        preview
    );
});

console.log(`\npromptCopy tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} promptCopy test(s) failed`);
}
