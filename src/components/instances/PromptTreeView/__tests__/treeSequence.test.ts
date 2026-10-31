/**
 * Unit Tests for Prompt Tree View 3-Tier Sequencing & Conversation Status Guards
 *
 * Verifies:
 * 1. isConversationOrTurnQueued in conversationGuards.ts correctly detects top-level
 *    and nested queued turns.
 * 2. getConversationStatus in conversationGuards.ts enforces strict priority:
 *    Running (Tier 1) > Queued (Tier 2) > Completed (Tier 3), with ghost protection.
 * 3. sortConversations in ProjectNode.tsx orders conversations into:
 *    - Tier 1: Running prompts on top
 *    - Tier 2: Queued prompts in the middle (amber badge)
 *    - Tier 3: Completed prompts ordered descending by last_modified timestamp
 * 4. Active filters ('all', 'running', 'queued') in sortConversations.
 *
 * Run: node scripts/run-frontend-tests.mjs treeSequence
 *      npm test -- treeSequence
 */

import { sortConversations } from '../ProjectNode';
import { sortPromptTurns } from '../ConversationNode';
import {
    getConversationStatus,
    isConversationOrTurnQueued,
    isGhostConversation,
    isStaleOrEmptyConversation,
} from '../conversationGuards';
import type { AgmConversationNode, AgmPromptTurnNode } from '../types';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
    if (typeof (globalThis as any).it === 'function') {
        (globalThis as any).it(description, fn);
        return;
    }
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

function assertDeepEqual<T>(actual: T, expected: T, msg?: string): void {
    const a = JSON.stringify(actual);
    const b = JSON.stringify(expected);
    if (a !== b) {
        throw new Error(msg || `expected ${b}, got ${a}`);
    }
}

// ---------------------------------------------------------------------------
// Mock Data Helpers
// ---------------------------------------------------------------------------

function makePromptTurn(overrides: Partial<AgmPromptTurnNode> = {}): AgmPromptTurnNode {
    return {
        turn_number: 1,
        step_index: 0,
        prompt_text: 'Turn prompt content',
        preview_200w: 'Turn prompt content',
        status: 'COMPLETED',
        is_running: false,
        is_queued: false,
        timestamp: '2026-10-18T10:00:00Z',
        created_at: 1729245600,
        media_paths: [],
        media_thumbnails: [],
        ...overrides,
    };
}

function makeConversation(overrides: Partial<AgmConversationNode> = {}): AgmConversationNode {
    return {
        seq_id: 1,
        seq_code: 'C001',
        gitmap_seq_code: 'GM:c1',
        conversation_id: 'conv-1',
        short_id: 'c1',
        title: 'Feature implementation task',
        status: 'COMPLETED',
        is_running: false,
        is_queued: false,
        step_count: 5,
        instance_id: 'default',
        prompt_preview_200w: 'Working on core logic implementation',
        prompt_word_count: 5,
        last_modified: '2026-10-18T12:00:00Z',
        prompts: [],
        ...overrides,
    };
}

// ===========================================================================
// Test Group 1: isConversationOrTurnQueued
// ===========================================================================

test('isConversationOrTurnQueued: returns true when conv.is_queued is true', () => {
    const conv = makeConversation({ is_queued: true });
    assertTrue(isConversationOrTurnQueued(conv));
});

test('isConversationOrTurnQueued: returns true when conv.status includes queue (case-insensitive)', () => {
    assertTrue(isConversationOrTurnQueued(makeConversation({ status: 'QUEUED' })));
    assertTrue(isConversationOrTurnQueued(makeConversation({ status: 'in_queue' })));
    assertTrue(isConversationOrTurnQueued(makeConversation({ status: 'Queued' })));
});

test('isConversationOrTurnQueued: returns true when a child prompt has is_queued = true', () => {
    const conv = makeConversation({
        status: 'IDLE',
        is_queued: false,
        prompts: [
            makePromptTurn({ status: 'COMPLETED', is_queued: false }),
            makePromptTurn({ status: 'QUEUED', is_queued: true }),
        ],
    });
    assertTrue(isConversationOrTurnQueued(conv));
});

test('isConversationOrTurnQueued: returns true when a child prompt has status containing queue', () => {
    const conv = makeConversation({
        status: 'IDLE',
        is_queued: false,
        prompts: [
            makePromptTurn({ status: 'queued_turn', is_queued: false }),
        ],
    });
    assertTrue(isConversationOrTurnQueued(conv));
});

test('isConversationOrTurnQueued: returns false for normal idle or completed conversation', () => {
    const conv = makeConversation({
        status: 'COMPLETED',
        is_queued: false,
        prompts: [makePromptTurn({ status: 'COMPLETED', is_queued: false })],
    });
    assertFalse(isConversationOrTurnQueued(conv));
});

test('isConversationOrTurnQueued: returns false for active running conversation with no queued turns', () => {
    const conv = makeConversation({
        status: 'RUNNING',
        is_running: true,
        is_queued: false,
        prompts: [makePromptTurn({ status: 'RUNNING', is_running: true })],
    });
    assertFalse(isConversationOrTurnQueued(conv));
});

// ===========================================================================
// Test Group 2: getConversationStatus
// ===========================================================================

test('getConversationStatus: identifies active running conversation as isRunning: true, isQueued: false', () => {
    const conv = makeConversation({
        status: 'RUNNING',
        is_running: true,
        prompt_word_count: 10,
        prompt_preview_200w: 'Active prompt execution',
    });
    const status = getConversationStatus(conv);
    assertTrue(status.isRunning);
    assertFalse(status.isQueued);
});

test('getConversationStatus: identifies conversation with running child prompt as isRunning: true', () => {
    const conv = makeConversation({
        status: 'IDLE',
        is_running: false,
        prompt_word_count: 5,
        prompt_preview_200w: 'Turn running',
        prompts: [
            makePromptTurn({ is_running: true, status: 'RUNNING' }),
        ],
    });
    const status = getConversationStatus(conv);
    assertTrue(status.isRunning);
    assertFalse(status.isQueued);
});

test('getConversationStatus: Running strictly supersedes Queued when both are present', () => {
    const conv = makeConversation({
        status: 'RUNNING',
        is_running: true,
        is_queued: true,
        prompt_word_count: 12,
        prompt_preview_200w: 'Running with queued subturn',
        prompts: [
            makePromptTurn({ is_running: true, status: 'RUNNING' }),
            makePromptTurn({ is_queued: true, status: 'QUEUED' }),
        ],
    });
    const status = getConversationStatus(conv);
    assertTrue(status.isRunning, 'Running must take precedence');
    assertFalse(status.isQueued, 'isQueued must be false when isRunning is true');
});

test('getConversationStatus: identifies purely queued conversation as isRunning: false, isQueued: true', () => {
    const conv = makeConversation({
        status: 'QUEUED',
        is_running: false,
        is_queued: true,
        prompt_word_count: 8,
        prompt_preview_200w: 'Queued turn ready for dispatch',
    });
    const status = getConversationStatus(conv);
    assertFalse(status.isRunning);
    assertTrue(status.isQueued);
});

test('getConversationStatus: identifies conversation with queued prompt turn as isQueued: true', () => {
    const conv = makeConversation({
        status: 'IDLE',
        is_running: false,
        is_queued: false,
        prompt_word_count: 5,
        prompt_preview_200w: 'Parent conversation',
        prompts: [
            makePromptTurn({ status: 'QUEUED', is_queued: true }),
        ],
    });
    const status = getConversationStatus(conv);
    assertFalse(status.isRunning);
    assertTrue(status.isQueued);
});

test('getConversationStatus: completed conversation returns isRunning: false, isQueued: false', () => {
    const conv = makeConversation({
        status: 'COMPLETED',
        is_running: false,
        is_queued: false,
        prompt_word_count: 10,
        prompt_preview_200w: 'Completed yesterday',
    });
    const status = getConversationStatus(conv);
    assertFalse(status.isRunning);
    assertFalse(status.isQueued);
});

test('getConversationStatus: ghost conversation (0 words, empty preview, untitled) is never treated as running', () => {
    const ghost = makeConversation({
        title: 'Untitled',
        status: 'RUNNING',
        is_running: true,
        prompt_word_count: 0,
        prompt_preview_200w: '',
    });
    const status = getConversationStatus(ghost);
    assertFalse(status.isRunning, 'Ghost conversation must never be marked isRunning');
    assertFalse(status.isQueued);
});

test('isGhostConversation: detects empty untitled conversations and short_id title matches', () => {
    const ghost = makeConversation({
        title: 'Untitled',
        status: 'IDLE',
        is_running: false,
        prompt_word_count: 0,
        prompt_preview_200w: '',
    });
    assertTrue(isGhostConversation(ghost));

    const meaningful = makeConversation({
        title: 'Fix issue',
        status: 'IDLE',
        is_running: false,
        prompt_word_count: 10,
        prompt_preview_200w: 'Fix issue details',
    });
    assertFalse(isGhostConversation(meaningful));
});

test('isStaleOrEmptyConversation: identifies stale empty conversations and exempts running/queued', () => {
    const stale = makeConversation({
        title: 'New conversation',
        status: 'IDLE',
        is_running: false,
        prompt_word_count: 15,
        prompt_preview_200w: 'Past idle content',
    });
    assertTrue(isStaleOrEmptyConversation(stale));

    const running = makeConversation({
        title: 'New conversation',
        status: 'RUNNING',
        is_running: true,
        prompt_word_count: 15,
        prompt_preview_200w: 'Active running prompt',
    });
    assertFalse(isStaleOrEmptyConversation(running));
});

// ===========================================================================
// Test Group 3: sortConversations 3-Tier Sequence Hierarchy
// ===========================================================================

test('sortConversations: strict 3-tier sequence: Running (Tier 1) -> Queued (Tier 2) -> Completed (Tier 3)', () => {
    const cCompletedOld = makeConversation({
        conversation_id: 'conv-completed-old',
        title: 'Old completed task',
        status: 'COMPLETED',
        is_running: false,
        is_queued: false,
        last_modified: '2026-10-15T12:00:00Z',
    });

    const cQueued = makeConversation({
        conversation_id: 'conv-queued',
        title: 'Pending queued task',
        status: 'QUEUED',
        is_running: false,
        is_queued: true,
        last_modified: '2026-10-16T12:00:00Z',
    });

    const cRunning = makeConversation({
        conversation_id: 'conv-running',
        title: 'Active running task',
        status: 'RUNNING',
        is_running: true,
        is_queued: false,
        last_modified: '2026-10-14T12:00:00Z', // Even with older timestamp, must be Tier 1
    });

    const cCompletedNew = makeConversation({
        conversation_id: 'conv-completed-new',
        title: 'Recent completed task',
        status: 'COMPLETED',
        is_running: false,
        is_queued: false,
        last_modified: '2026-10-19T12:00:00Z',
    });

    // Provide shuffled input
    const input = [cCompletedOld, cQueued, cRunning, cCompletedNew];
    const sorted = sortConversations(input, 'all');

    assertEqual(sorted.length, 4);
    // Tier 1: Running
    assertEqual(sorted[0].conversation_id, 'conv-running', 'Tier 1 must be Running prompt on top');
    // Tier 2: Queued
    assertEqual(sorted[1].conversation_id, 'conv-queued', 'Tier 2 must be Queued prompt in middle');
    // Tier 3: Completed ordered by timestamp descending
    assertEqual(sorted[2].conversation_id, 'conv-completed-new', 'Tier 3 top must be newest completed');
    assertEqual(sorted[3].conversation_id, 'conv-completed-old', 'Tier 3 bottom must be older completed');
});

test('sortConversations: conversation with running child turn sorts into Tier 1 above queued and completed', () => {
    const cCompleted = makeConversation({
        conversation_id: 'conv-comp',
        status: 'COMPLETED',
        last_modified: '2026-10-19T10:00:00Z',
    });

    const cQueued = makeConversation({
        conversation_id: 'conv-q',
        status: 'QUEUED',
        is_queued: true,
        last_modified: '2026-10-19T09:00:00Z',
    });

    const cTurnRunning = makeConversation({
        conversation_id: 'conv-turn-run',
        status: 'IDLE',
        is_running: false,
        last_modified: '2026-10-10T10:00:00Z',
        prompts: [
            makePromptTurn({ status: 'RUNNING', is_running: true }),
        ],
    });

    const sorted = sortConversations([cCompleted, cQueued, cTurnRunning], 'all');
    assertEqual(sorted[0].conversation_id, 'conv-turn-run', 'Child running turn must promote conversation to Tier 1');
    assertEqual(sorted[1].conversation_id, 'conv-q', 'Queued must remain Tier 2');
    assertEqual(sorted[2].conversation_id, 'conv-comp', 'Completed must remain Tier 3');
});

test('sortConversations: conversation with queued child turn sorts into Tier 2 between running and completed', () => {
    const cRunning = makeConversation({
        conversation_id: 'conv-run',
        status: 'RUNNING',
        is_running: true,
        last_modified: '2026-10-12T10:00:00Z',
    });

    const cTurnQueued = makeConversation({
        conversation_id: 'conv-turn-q',
        status: 'IDLE',
        is_running: false,
        last_modified: '2026-10-10T10:00:00Z',
        prompts: [
            makePromptTurn({ status: 'QUEUED', is_queued: true }),
        ],
    });

    const cCompleted = makeConversation({
        conversation_id: 'conv-comp',
        status: 'COMPLETED',
        last_modified: '2026-10-18T10:00:00Z',
    });

    const sorted = sortConversations([cCompleted, cRunning, cTurnQueued], 'all');
    assertEqual(sorted[0].conversation_id, 'conv-run', 'Tier 1 Running');
    assertEqual(sorted[1].conversation_id, 'conv-turn-q', 'Tier 2 Queued via turn');
    assertEqual(sorted[2].conversation_id, 'conv-comp', 'Tier 3 Completed');
});

test('sortConversations: Tier 3 completed conversations sort with strict timestamp descending fidelity', () => {
    const c1 = makeConversation({ conversation_id: 't-2026-10-01', last_modified: '2026-10-01T00:00:00Z' });
    const c2 = makeConversation({ conversation_id: 't-2026-10-15', last_modified: '2026-10-15T12:00:00Z' });
    const c3 = makeConversation({ conversation_id: 't-2026-10-10', last_modified: '2026-10-10T08:00:00Z' });
    const c4 = makeConversation({ conversation_id: 't-2026-10-18', last_modified: '2026-10-18T19:30:00Z' });

    const sorted = sortConversations([c1, c2, c3, c4], 'all');
    const order = sorted.map((c) => c.conversation_id);
    assertDeepEqual(order, ['t-2026-10-18', 't-2026-10-15', 't-2026-10-10', 't-2026-10-01']);
});

// ===========================================================================
// Test Group 4: activeFilter Options in sortConversations
// ===========================================================================

test('sortConversations: activeFilter="running" filters strictly Tier 1 conversations', () => {
    const cRun1 = makeConversation({ conversation_id: 'r1', is_running: true, status: 'RUNNING' });
    const cRun2 = makeConversation({
        conversation_id: 'r2',
        status: 'IDLE',
        prompts: [makePromptTurn({ is_running: true, status: 'RUNNING' })],
    });
    const cQueued = makeConversation({ conversation_id: 'q1', is_queued: true, status: 'QUEUED' });
    const cCompleted = makeConversation({ conversation_id: 'c1', status: 'COMPLETED' });

    const sorted = sortConversations([cCompleted, cRun1, cQueued, cRun2], 'running');
    assertEqual(sorted.length, 2);
    assertTrue(sorted.every((c) => c.conversation_id === 'r1' || c.conversation_id === 'r2'));
});

test('sortConversations: activeFilter="queued" filters strictly Tier 2 conversations', () => {
    const cRun = makeConversation({ conversation_id: 'r1', is_running: true, status: 'RUNNING' });
    const cQueued1 = makeConversation({ conversation_id: 'q1', is_queued: true, status: 'QUEUED' });
    const cQueued2 = makeConversation({
        conversation_id: 'q2',
        status: 'IDLE',
        prompts: [makePromptTurn({ is_queued: true, status: 'QUEUED' })],
    });
    const cCompleted = makeConversation({ conversation_id: 'c1', status: 'COMPLETED' });

    const sorted = sortConversations([cRun, cQueued1, cCompleted, cQueued2], 'queued');
    assertEqual(sorted.length, 2);
    assertTrue(sorted.every((c) => c.conversation_id === 'q1' || c.conversation_id === 'q2'));
});

test('sortConversations: handles empty list and single item gracefully', () => {
    const emptyResult = sortConversations([], 'all');
    assertEqual(emptyResult.length, 0);

    const singleConv = makeConversation({ conversation_id: 'single' });
    const singleResult = sortConversations([singleConv], 'all');
    assertEqual(singleResult.length, 1);
    assertEqual(singleResult[0].conversation_id, 'single');
});

// ===========================================================================
// Test Group 5: sortPromptTurns Recent-on-Top & 3-Tier Priority
// ===========================================================================

test('sortPromptTurns: running prompt turn sorts to Tier 1 on top', () => {
    const t1 = makePromptTurn({ turn_number: 1, created_at: 1000, status: 'COMPLETED' });
    const t2 = makePromptTurn({ turn_number: 2, created_at: 2000, status: 'COMPLETED' });
    const t3 = makePromptTurn({ turn_number: 3, created_at: 3000, is_running: true, status: 'RUNNING' });

    const sorted = sortPromptTurns([t1, t2, t3]);
    assertEqual(sorted[0].turn_number, 3, 'Running turn 3 must be on top');
    assertEqual(sorted[1].turn_number, 2, 'Newer completed turn 2 must be second');
    assertEqual(sorted[2].turn_number, 1, 'Oldest completed turn 1 must be at the bottom');
});

test('sortPromptTurns: queued turn sorts below running but above completed', () => {
    const t1 = makePromptTurn({ turn_number: 1, created_at: 1000, status: 'COMPLETED' });
    const t2 = makePromptTurn({ turn_number: 2, created_at: 2000, is_queued: true, status: 'QUEUED' });
    const t3 = makePromptTurn({ turn_number: 3, created_at: 3000, is_running: true, status: 'RUNNING' });

    const sorted = sortPromptTurns([t1, t2, t3]);
    assertEqual(sorted[0].turn_number, 3, 'Tier 1 Running turn must be first');
    assertEqual(sorted[1].turn_number, 2, 'Tier 2 Queued turn must be second');
    assertEqual(sorted[2].turn_number, 1, 'Tier 3 Completed turn must be third');
});

test('sortPromptTurns: completed turns sort in descending order (recent on top)', () => {
    const t1 = makePromptTurn({ turn_number: 1, created_at: 1729200000, status: 'COMPLETED' });
    const t2 = makePromptTurn({ turn_number: 2, created_at: 1729203600, status: 'COMPLETED' });
    const t3 = makePromptTurn({ turn_number: 3, created_at: 1729207200, status: 'COMPLETED' });
    const t4 = makePromptTurn({ turn_number: 4, created_at: 1729210800, status: 'COMPLETED' });

    const sorted = sortPromptTurns([t1, t2, t3, t4]);
    const numbers = sorted.map((t) => t.turn_number);
    assertDeepEqual(numbers, [4, 3, 2, 1], 'Completed prompt turns must display recent on top');
});

test('sortPromptTurns: handles empty and single turn gracefully', () => {
    const emptyResult = sortPromptTurns([]);
    assertEqual(emptyResult.length, 0);

    const singleTurn = makePromptTurn({ turn_number: 1 });
    const singleResult = sortPromptTurns([singleTurn]);
    assertEqual(singleResult.length, 1);
    assertEqual(singleResult[0].turn_number, 1);
});

// ===========================================================================
// Summary
// ===========================================================================

console.log(`\ntreeSequence verification: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} treeSequence test(s) failed`);
}
