/**
 * Standalone verification test suite for Instance Running Prompts and Task Liveness.
 *
 * Verifies:
 * 1. isNodeOwnedByInstance correctly associates projects to default & cloned instances.
 * 2. Sorting logic in InstanceCardProjects places running projects at the top.
 * 3. hasActiveTask logic returns true only when an active conversation turn is genuine
 *    (not ghost, not empty) and the instance process is alive.
 * 4. Prompt status identification marks running conversations and filters out ghosts.
 *
 * Run: node scripts/run-frontend-tests.mjs instanceRunningPrompts
 */

import {
    isNodeOwnedByInstance,
    isDefaultOwned,
} from '../instancePageUtils';
import {
    isGhostConversation,
    isStaleOrEmptyConversation,
    isUntitledOrEmpty,
} from '../../../components/instances/PromptTreeView/conversationGuards';
import type {
    AgmProjectTreeNode,
    AgmConversationNode,
} from '../../../components/instances/PromptTreeView/types';

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

// ---------------------------------------------------------------------------
// Mock Factory Helpers
// ---------------------------------------------------------------------------

function makeConversation(overrides: Partial<AgmConversationNode> = {}): AgmConversationNode {
    return {
        seq_id: 1,
        seq_code: 'C001',
        gitmap_seq_code: 'GM:c1a2b3c4',
        conversation_id: 'conv-uuid-1234',
        short_id: 'c1a2b3c4',
        title: 'Fix issue in auth module',
        status: 'RUNNING',
        is_running: true,
        step_count: 5,
        instance_id: 'default',
        prompt_preview_200w: 'Please fix the token refresh bug in auth module',
        prompt_word_count: 8,
        last_modified: '2026-10-11T12:00:00Z',
        ...overrides,
    };
}

function makeProject(overrides: Partial<AgmProjectTreeNode> = {}): AgmProjectTreeNode {
    return {
        seq_id: 1,
        seq_code: 'P001',
        gitmap_seq_code: 'GM:proj1',
        project_id: 'proj-1',
        repo_name: 'Antigravity-Manager',
        repo_path: '/repos/Antigravity-Manager',
        instance_id: 'default',
        instance_name: 'default',
        is_running: true,
        conversations: [],
        ...overrides,
    };
}

// ---------------------------------------------------------------------------
// Pure InstanceCardProjects Sorting Logic (mirrors InstanceCardProjects.tsx)
// ---------------------------------------------------------------------------

function sortInstanceProjects(
    instanceProjects: AgmProjectTreeNode[],
    instIsRunning: boolean
): AgmProjectTreeNode[] {
    return [...instanceProjects].sort((a, b) => {
        const isARunning = Boolean(instIsRunning) && Boolean(a.is_running);
        const isBRunning = Boolean(instIsRunning) && Boolean(b.is_running);
        if (isARunning && !isBRunning) return -1;
        if (isBRunning && !isARunning) return 1;
        const aLatest = Math.max(0, ...(a.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        const bLatest = Math.max(0, ...(b.conversations || []).map((c) => new Date(c.last_modified).getTime() || 0));
        if (bLatest !== aLatest) return bLatest - aLatest;
        return a.repo_name.localeCompare(b.repo_name);
    });
}

// ---------------------------------------------------------------------------
// Pure hasActiveTask Logic (mirrors instanceCardData.ts & useInstancePage.ts)
// ---------------------------------------------------------------------------

function checkHasActiveTask(
    inst: { is_running?: boolean; config: { id: string; is_default?: boolean; seq_num?: number } },
    runningTreeNodes: AgmProjectTreeNode[]
): boolean {
    return (
        Boolean(inst.is_running) &&
        runningTreeNodes.some((node) => {
            const isInstanceMatch = isNodeOwnedByInstance(node, inst.config);
            const hasGenuineRunningConv = (node.conversations || []).some(
                (c) =>
                    Boolean(c.is_running) &&
                    !isGhostConversation(c) &&
                    !(c.prompt_word_count === 0 && (!c.prompt_preview_200w || !c.prompt_preview_200w.trim()))
            );
            return isInstanceMatch && Boolean(node.is_running) && hasGenuineRunningConv;
        })
    );
}

// ===========================================================================
// Test Group 1: isDefaultOwned & isNodeOwnedByInstance
// ===========================================================================

test('isDefaultOwned: matches default and __default__ aliases', () => {
    assertTrue(isDefaultOwned('default', 'default'));
    assertTrue(isDefaultOwned('__default__', 'default'));
    assertTrue(isDefaultOwned('default', 'custom-default-id'));
    assertTrue(isDefaultOwned('__default__', 'custom-default-id'));
    assertTrue(isDefaultOwned('custom-default-id', 'custom-default-id'));
});

test('isDefaultOwned: rejects non-matching or empty values', () => {
    assertFalse(isDefaultOwned('', 'default'));
    assertFalse(isDefaultOwned(null, 'default'));
    assertFalse(isDefaultOwned(undefined, 'default'));
    assertFalse(isDefaultOwned('default-copy-8159', 'default'));
    assertFalse(isDefaultOwned('other-instance', 'default'));
});

test('isNodeOwnedByInstance: default instance matches "default" and "__default__" node instance_id', () => {
    const defaultCfg = { id: 'default', is_default: true, seq_num: 1 };
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: 'default' }), defaultCfg));
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: '__default__' }), defaultCfg));
});

test('isNodeOwnedByInstance: default instance with custom ID matches both alias and ID', () => {
    const customDefaultCfg = { id: 'custom-default-uuid', is_default: true, seq_num: 1 };
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: 'default' }), customDefaultCfg));
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: '__default__' }), customDefaultCfg));
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: 'custom-default-uuid' }), customDefaultCfg));
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: 'cloned-inst' }), customDefaultCfg));
});

test('isNodeOwnedByInstance: cloned instance rejects default aliases', () => {
    const cloneCfg = { id: 'default-copy-8159', is_default: false, seq_num: 2 };
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: 'default' }), cloneCfg));
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: '__default__' }), cloneCfg));
});

test('isNodeOwnedByInstance: cloned instance matches exact ID', () => {
    const cloneCfg = { id: 'default-copy-8159', is_default: false, seq_num: 2 };
    assertTrue(isNodeOwnedByInstance(makeProject({ instance_id: 'default-copy-8159' }), cloneCfg));
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: 'default-copy-9999' }), cloneCfg));
});

test('isNodeOwnedByInstance: cloned instance matches by seq_num fallback', () => {
    const cloneCfg = { id: 'default-copy-8159', is_default: false, seq_num: 2 };
    const projectWithSeq = makeProject({
        instance_id: 'unmatched-id',
        instance_seq_num: 2,
    });
    assertTrue(isNodeOwnedByInstance(projectWithSeq, cloneCfg));

    const projectWithWrongSeq = makeProject({
        instance_id: 'unmatched-id',
        instance_seq_num: 3,
    });
    assertFalse(isNodeOwnedByInstance(projectWithWrongSeq, cloneCfg));
});

test('isNodeOwnedByInstance: rejects node with empty or whitespace instance_id', () => {
    const defaultCfg = { id: 'default', is_default: true };
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: '' }), defaultCfg));
    assertFalse(isNodeOwnedByInstance(makeProject({ instance_id: '   ' }), defaultCfg));
});

// ===========================================================================
// Test Group 2: Sorting Logic in InstanceCardProjects
// ===========================================================================

test('sortInstanceProjects: running projects sort above idle projects when instance is running', () => {
    const idleProj = makeProject({
        project_id: 'idle',
        repo_name: 'alpha-repo',
        is_running: false,
        conversations: [makeConversation({ last_modified: '2026-10-11T12:00:00Z' })],
    });
    const runProj = makeProject({
        project_id: 'running',
        repo_name: 'zeta-repo',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-10T12:00:00Z' })],
    });

    const sorted = sortInstanceProjects([idleProj, runProj], true);
    assertEqual(sorted[0].project_id, 'running');
    assertEqual(sorted[1].project_id, 'idle');
});

test('sortInstanceProjects: two running projects sort by most recent conversation timestamp', () => {
    const runOlder = makeProject({
        project_id: 'older',
        repo_name: 'repo-a',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-10T10:00:00Z' })],
    });
    const runNewer = makeProject({
        project_id: 'newer',
        repo_name: 'repo-b',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-11T15:30:00Z' })],
    });

    const sorted = sortInstanceProjects([runOlder, runNewer], true);
    assertEqual(sorted[0].project_id, 'newer');
    assertEqual(sorted[1].project_id, 'older');
});

test('sortInstanceProjects: ties in timestamp break by repo_name alphabetically', () => {
    const projB = makeProject({
        project_id: 'b',
        repo_name: 'beta-tool',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-11T12:00:00Z' })],
    });
    const projA = makeProject({
        project_id: 'a',
        repo_name: 'alpha-tool',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-11T12:00:00Z' })],
    });

    const sorted = sortInstanceProjects([projB, projA], true);
    assertEqual(sorted[0].repo_name, 'alpha-tool');
    assertEqual(sorted[1].repo_name, 'beta-tool');
});

test('sortInstanceProjects: dead-process gating suppresses running elevation when instance is stopped', () => {
    const runningNode = makeProject({
        project_id: 'run-node',
        repo_name: 'zeta-repo',
        is_running: true,
        conversations: [makeConversation({ last_modified: '2026-10-10T10:00:00Z' })],
    });
    const idleNode = makeProject({
        project_id: 'idle-node',
        repo_name: 'alpha-repo',
        is_running: false,
        conversations: [makeConversation({ last_modified: '2026-10-11T12:00:00Z' })],
    });

    // When instIsRunning is false, neither project is treated as active running
    const sorted = sortInstanceProjects([runningNode, idleNode], false);
    // idleNode has more recent timestamp (Oct 11 vs Oct 10), so it comes first
    assertEqual(sorted[0].project_id, 'idle-node');
    assertEqual(sorted[1].project_id, 'run-node');
});

test('sortInstanceProjects: projects with no conversations fallback to timestamp 0', () => {
    const noConvs = makeProject({
        project_id: 'empty',
        repo_name: 'empty-repo',
        is_running: false,
        conversations: [],
    });
    const withConvs = makeProject({
        project_id: 'with-conv',
        repo_name: 'active-repo',
        is_running: false,
        conversations: [makeConversation({ last_modified: '2026-10-01T00:00:00Z' })],
    });

    const sorted = sortInstanceProjects([noConvs, withConvs], false);
    assertEqual(sorted[0].project_id, 'with-conv');
    assertEqual(sorted[1].project_id, 'empty');
});

// ===========================================================================
// Test Group 3: hasActiveTask Logic
// ===========================================================================

test('hasActiveTask: returns true when live instance has matching running node and genuine conversation', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const node = makeProject({
        instance_id: 'default',
        is_running: true,
        conversations: [
            makeConversation({
                is_running: true,
                title: 'Refactor prompt engine',
                prompt_word_count: 10,
                prompt_preview_200w: 'Refactor prompt engine to eliminate deadlocks',
            }),
        ],
    });

    assertTrue(checkHasActiveTask(inst, [node]));
});

test('hasActiveTask: returns false if instance process is dead (dead-process gate)', () => {
    const deadInst = {
        is_running: false,
        config: { id: 'default', is_default: true },
    };
    const node = makeProject({
        instance_id: 'default',
        is_running: true,
        conversations: [makeConversation({ is_running: true })],
    });

    assertFalse(checkHasActiveTask(deadInst, [node]));
});

test('hasActiveTask: returns false if node is owned by a different instance', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const foreignNode = makeProject({
        instance_id: 'default-copy-8159',
        is_running: true,
        conversations: [makeConversation({ is_running: true })],
    });

    assertFalse(checkHasActiveTask(inst, [foreignNode]));
});

test('hasActiveTask: returns false if node itself is marked not running', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const idleNode = makeProject({
        instance_id: 'default',
        is_running: false,
        conversations: [makeConversation({ is_running: true })],
    });

    assertFalse(checkHasActiveTask(inst, [idleNode]));
});

test('hasActiveTask: returns false if running conversation is a ghost (Untitled, 0 words)', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const ghostNode = makeProject({
        instance_id: 'default',
        is_running: true,
        conversations: [
            makeConversation({
                is_running: true,
                title: 'Untitled Conversation',
                prompt_word_count: 0,
                prompt_preview_200w: '',
            }),
        ],
    });

    assertFalse(checkHasActiveTask(inst, [ghostNode]));
});

test('hasActiveTask: returns false if running conversation has 0 words and empty preview even with custom title', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const emptyPromptNode = makeProject({
        instance_id: 'default',
        is_running: true,
        conversations: [
            makeConversation({
                is_running: true,
                title: 'Custom Bug Investigation',
                prompt_word_count: 0,
                prompt_preview_200w: '   ',
            }),
        ],
    });

    assertFalse(checkHasActiveTask(inst, [emptyPromptNode]));
});

test('hasActiveTask: returns false if conversation is idle (is_running: false)', () => {
    const inst = {
        is_running: true,
        config: { id: 'default', is_default: true },
    };
    const idleConvNode = makeProject({
        instance_id: 'default',
        is_running: true,
        conversations: [
            makeConversation({
                is_running: false,
                title: 'Completed analysis',
                prompt_word_count: 50,
                prompt_preview_200w: 'Analysis report completed.',
            }),
        ],
    });

    assertFalse(checkHasActiveTask(inst, [idleConvNode]));
});

test('hasActiveTask: correctly resolves active task for cloned instance with matching seq_num', () => {
    const cloneInst = {
        is_running: true,
        config: { id: 'copy-worker-2', is_default: false, seq_num: 2 },
    };
    const cloneNode = makeProject({
        instance_id: 'other-alias',
        instance_seq_num: 2,
        is_running: true,
        conversations: [
            makeConversation({
                is_running: true,
                title: 'Subagent batch run',
                prompt_word_count: 12,
                prompt_preview_200w: 'Execute worker subtasks',
            }),
        ],
    });

    assertTrue(checkHasActiveTask(cloneInst, [cloneNode]));
});

// ===========================================================================
// Test Group 4: Prompt Status Identification & Ghost Filtering
// ===========================================================================

test('isGhostConversation: identifies untitled empty conversations as ghosts', () => {
    assertTrue(isGhostConversation(makeConversation({ title: 'Untitled', prompt_word_count: 0, prompt_preview_200w: '' })));
    assertTrue(isGhostConversation(makeConversation({ title: 'untitled conversation', prompt_word_count: 0, prompt_preview_200w: '' })));
    assertTrue(isGhostConversation(makeConversation({ title: 'New Conversation', prompt_word_count: 0, prompt_preview_200w: '' })));
    assertTrue(isGhostConversation(makeConversation({ title: 'chat', prompt_word_count: 0, prompt_preview_200w: '' })));
    assertTrue(isGhostConversation(makeConversation({ title: 'conversation-1', prompt_word_count: 0, prompt_preview_200w: '' })));
    assertTrue(isGhostConversation(makeConversation({ title: '', prompt_word_count: 0, prompt_preview_200w: '' })));
});

test('isGhostConversation: short_id matching title with 0 words is identified as ghost', () => {
    const conv = makeConversation({
        short_id: '8a7b6c5d',
        title: '8A7B6C5D',
        prompt_word_count: 0,
        prompt_preview_200w: '',
    });
    assertTrue(isGhostConversation(conv));
});

test('isGhostConversation: conversation with words or preview is NOT a ghost', () => {
    assertFalse(isGhostConversation(makeConversation({
        title: 'Untitled',
        prompt_word_count: 5,
        prompt_preview_200w: 'Some prompt words',
    })));
    assertFalse(isGhostConversation(makeConversation({
        title: 'Fix issue',
        prompt_word_count: 0,
        prompt_preview_200w: '',
    })));
});

test('isStaleOrEmptyConversation: running conversation with content is exempt', () => {
    const runningWithContent = makeConversation({
        is_running: true,
        title: 'New conversation',
        prompt_word_count: 15,
        prompt_preview_200w: 'Working on implementation...',
    });
    assertFalse(isStaleOrEmptyConversation(runningWithContent));
});

test('isStaleOrEmptyConversation: idle conversation with untitled name is marked stale/empty', () => {
    const idleUntitled = makeConversation({
        is_running: false,
        title: 'New conversation',
        prompt_word_count: 15,
        prompt_preview_200w: 'Some past prompt',
    });
    assertTrue(isStaleOrEmptyConversation(idleUntitled));
});

test('isStaleOrEmptyConversation: idle conversation with meaningful title is NOT stale/empty', () => {
    const idleMeaningful = makeConversation({
        is_running: false,
        title: 'Production release checklist',
        prompt_word_count: 20,
        prompt_preview_200w: 'Release checklist item 1',
    });
    assertFalse(isStaleOrEmptyConversation(idleMeaningful));
});

test('isUntitledOrEmpty alias behaves identically to isStaleOrEmptyConversation', () => {
    const ghost = makeConversation({ title: 'Untitled', prompt_word_count: 0, prompt_preview_200w: '' });
    assertEqual(isUntitledOrEmpty(ghost), isStaleOrEmptyConversation(ghost));
});

// ===========================================================================
// Summary
// ===========================================================================

console.log(`\ninstanceRunningPrompts verification: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} instanceRunningPrompts test(s) failed`);
}
