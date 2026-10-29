/**
 * Queued-turn visibility: Default vs default-copy alias, merge after reload,
 * and repo path resolution for enqueue.
 *
 * Run: node scripts/run-frontend-tests.mjs queuedTurns
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

import {
    isDefaultInstanceAlias,
    isInstanceCompatible,
    mergeQueuedTurns,
    mergeQueuedTree,
    applySelectedTurnOverlay,
    queuedPromptCount,
} from '../usePromptTreeData';
import { resolveRepoPathForDispatch } from '../usePromptTreePromptActions';
import type { AgmConversationNode, AgmProjectTreeNode } from '../types';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    try {
        fn();
        passed += 1;
        console.log(`  ok  ${description}`);
    } catch (e: unknown) {
        failed += 1;
        console.error(`  FAIL  ${description}`);
        console.error(e);
    }
}

function assert(cond: boolean, msg: string): void {
    if (!cond) {
        throw new Error(msg);
    }
}

function conv(id: string, extra?: Partial<AgmConversationNode>): AgmConversationNode {
    return {
        seq_id: 1,
        seq_code: 'C001',
        gitmap_seq_code: 'GM:abcd',
        conversation_id: id,
        short_id: 'abcd',
        title: 'Work',
        status: 'IDLE',
        is_running: false,
        step_count: 1,
        instance_id: 'default',
        prompt_preview_200w: 'hello',
        prompt_word_count: 1,
        last_modified: '1',
        prompts: [],
        ...extra,
    };
}

test('default-copy is not the Default instance', () => {
    assert(isDefaultInstanceAlias('default') === true, 'default is default');
    assert(isDefaultInstanceAlias('__default__') === true, '__default__ is default');
    assert(isDefaultInstanceAlias('default-copy-5597') === false, 'copy is not default');
    assert(isDefaultInstanceAlias('v2-3127') === false, 'named instance is not default');
    assert(isInstanceCompatible('default-copy-5597', 'default') === false, 'copy convs stay off Default');
    assert(isInstanceCompatible('default-copy-5597', 'default-copy-5597') === true, 'copy matches itself');
});

test('mergeQueuedTurns keeps optimistic queued turns the reload omitted', () => {
    const previous = conv('c1', {
        is_queued: true,
        prompts: [
            {
                turn_number: 1,
                step_index: 0,
                prompt_text: 'done',
                preview_200w: 'done',
                status: 'COMPLETED',
                is_running: false,
                is_queued: false,
                timestamp: 't',
                created_at: 1,
                media_paths: [],
                media_thumbnails: [],
            },
            {
                turn_number: 2,
                step_index: 1,
                prompt_text: 'just enqueued',
                preview_200w: 'just enqueued',
                status: 'QUEUED',
                is_running: false,
                is_queued: true,
                timestamp: 't2',
                created_at: 2,
                media_paths: [],
                media_thumbnails: [],
            },
        ],
    });
    const incoming = conv('c1', {
        prompts: [previous.prompts![0]],
    });
    const merged = mergeQueuedTurns(incoming, previous);
    assert(merged.prompts?.length === 2, `expected 2 turns, got ${merged.prompts?.length}`);
    assert(merged.is_queued === true, 'conversation stays queued');
    assert(merged.prompts?.[1].prompt_text === 'just enqueued', 'queued text kept');
});

test('mergeQueuedTree preserves queued turns across projects', () => {
    const previous: AgmProjectTreeNode[] = [
        {
            seq_id: 1,
            seq_code: 'P001',
            gitmap_seq_code: 'GM:p',
            project_id: 'p1',
            repo_name: 'work',
            repo_path: '/work',
            instance_id: 'default',
            instance_name: 'Default',
            is_running: false,
            conversations: [
                conv('c1', {
                    is_queued: true,
                    prompts: [
                        {
                            turn_number: 1,
                            step_index: 0,
                            prompt_text: 'queued-now',
                            preview_200w: 'queued-now',
                            status: 'QUEUED',
                            is_running: false,
                            is_queued: true,
                            timestamp: 't',
                            created_at: 1,
                            media_paths: [],
                            media_thumbnails: [],
                        },
                    ],
                }),
            ],
        },
    ];
    const incoming: AgmProjectTreeNode[] = [
        {
            ...previous[0],
            conversations: [conv('c1')],
        },
    ];
    const merged = mergeQueuedTree(incoming, previous);
    assert(merged[0].conversations[0].prompts?.length === 1, 'queued turn survived tree merge');
});

test('mergeQueuedTurns keeps each identical rerun click as its own queued turn', () => {
    const queued = (n: number) => ({
        turn_number: n,
        step_index: n - 1,
        prompt_text: 'same rerun',
        preview_200w: 'same rerun',
        status: 'QUEUED',
        is_running: false,
        is_queued: true,
        timestamp: `t${n}`,
        created_at: n,
        media_paths: [] as string[],
        media_thumbnails: [] as string[],
    });
    const previous = conv('c1', {
        is_queued: true,
        prompts: [queued(1), queued(2), queued(3)],
    });
    const incoming = conv('c1', { prompts: [queued(1)] });
    const merged = mergeQueuedTurns(incoming, previous);
    assert(merged.prompts?.length === 3, `expected 3 queued turns, got ${merged.prompts?.length}`);
});

test('inspecting a completed turn still reports later queued turns', () => {
    const queuedConv = conv('c1', {
        is_queued: false,
        status: 'COMPLETED',
        selected_turn_number: 11,
        prompts: [
            {
                turn_number: 11,
                step_index: 10,
                prompt_text: 'older completed',
                preview_200w: 'older completed',
                status: 'COMPLETED',
                is_running: false,
                is_queued: false,
                timestamp: 't11',
                created_at: 11,
                media_paths: [],
                media_thumbnails: [],
            },
            {
                turn_number: 12,
                step_index: 11,
                prompt_text: 'just enqueued',
                preview_200w: 'just enqueued',
                status: 'QUEUED',
                is_running: false,
                is_queued: true,
                timestamp: 't12',
                created_at: 12,
                media_paths: [],
                media_thumbnails: [],
            },
            {
                turn_number: 13,
                step_index: 12,
                prompt_text: 'second enqueue',
                preview_200w: 'second enqueue',
                status: 'QUEUED',
                is_running: false,
                is_queued: true,
                timestamp: 't13',
                created_at: 13,
                media_paths: [],
                media_thumbnails: [],
            },
        ],
    });
    const overlay = applySelectedTurnOverlay(queuedConv, 11);
    assert(overlay.is_queued === true, 'conversation stays queued while viewing turn 11');
    assert(overlay.status === 'QUEUED', `expected QUEUED status, got ${overlay.status}`);
    assert(queuedPromptCount(overlay) === 2, `expected 2 queued turns, got ${queuedPromptCount(overlay)}`);
});

test('queuedPromptCount is 0 for idle conversations with no queued turns', () => {
    const idle = conv('c1', { is_queued: false, status: 'IDLE' });
    assert(queuedPromptCount(idle) === 0, 'idle has no queued count');
});

test('resolveRepoPathForDispatch walks the tree when project path is empty', () => {
    const c = conv('c1');
    const tree = [
        {
            repo_path: '/real/workspace',
            conversations: [{ conversation_id: 'c1' }],
        },
    ];
    const path = resolveRepoPathForDispatch(null, null, c, tree);
    assert(path === '/real/workspace', `expected workspace path, got ${path}`);
});

console.log(`queuedTurns: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} queuedTurns test(s) failed`);
}
