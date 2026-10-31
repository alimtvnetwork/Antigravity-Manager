/**
 * Unit Tests for Rerun Randomizer & Storage Persistence (Step 288 / Imp-702, Imp-727)
 *
 * Verifies:
 * 1. Default rerun option is 'randomized' and count defaults to 4.
 * 2. RERUN_RANDOM_MESSAGE_SET contains at least 25 distinct actionable variations.
 * 3. resolveRerunMessage correctly picks random items or fixed phrases.
 * 4. resolveRerunMessage avoids previous picks when excludeIndices is provided.
 * 5. loadPersistedRerunMessageOption and savePersistedRerunMessageOption work with both
 *    primary and legacy localStorage keys.
 * 6. loadPersistedRerunCount and savePersistedRerunCount clamp to valid range [1, 8].
 * 7. dispatchRerunMessages auto-picks randomized messages and enqueues all rounds (1..N)
 *    via enqueuePrompt (never calling sendPromptNow).
 * 8. dispatchRerunMessages handles errors gracefully without unhandled crashes.
 *
 * Run: node scripts/run-frontend-tests.mjs rerunActions
 *      npm test -- rerunActions
 */

interface MockInvokeCall {
    cmd: string;
    args: any;
}

const mockInvokeCalls: MockInvokeCall[] = [];
let mockInvokeFailCmd: string | null = null;
let mockInvokeFailAfterCount: number | null = null;

// Mock window, localStorage and Tauri IPC internals if running in Node.js
if (typeof (globalThis as any).window === 'undefined') {
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
    (globalThis as any).window = {
        localStorage: mockStorage,
        location: { pathname: '/prompts', hash: '', search: '', href: 'http://localhost/prompts' },
        __TAURI_INTERNALS__: {
            invoke: async (cmd: string, args: any) => {
                mockInvokeCalls.push({ cmd, args });
                const matchingCalls = mockInvokeCalls.filter((c) => c.cmd === cmd).length;
                if (mockInvokeFailCmd && cmd === mockInvokeFailCmd) {
                    if (mockInvokeFailAfterCount === null || matchingCalls > mockInvokeFailAfterCount) {
                        throw new Error(`Mock invoke failure for ${cmd}`);
                    }
                }
                return 'ok';
            },
        },
    };
    (globalThis as any).localStorage = mockStorage;
} else if (!(globalThis as any).window.__TAURI_INTERNALS__) {
    (globalThis as any).window.__TAURI_INTERNALS__ = {
        invoke: async (cmd: string, args: any) => {
            mockInvokeCalls.push({ cmd, args });
            const matchingCalls = mockInvokeCalls.filter((c) => c.cmd === cmd).length;
            if (mockInvokeFailCmd && cmd === mockInvokeFailCmd) {
                if (mockInvokeFailAfterCount === null || matchingCalls > mockInvokeFailAfterCount) {
                    throw new Error(`Mock invoke failure for ${cmd}`);
                }
            }
            return 'ok';
        },
    };
}

import {
    RERUN_DEFAULT_COUNT,
    RERUN_MIN_COUNT,
    RERUN_MAX_COUNT,
    RERUN_DEFAULT_MESSAGE_OPTION,
    RERUN_MESSAGE_OPTION_STORAGE_KEY,
    RERUN_COUNT_STORAGE_KEY,
    RERUN_RANDOM_MESSAGE_SET,
    resolveRerunMessage,
    pickDiverseRerunNudge,
    getRandomRerunNudge,
    loadPersistedRerunMessageOption,
    savePersistedRerunMessageOption,
    loadPersistedRerunCount,
    savePersistedRerunCount,
    dispatchRerunMessages,
    dispatchMultiPromptResend,
    extractUsedNudgesFromConversation,
    dispatchConversationId,
    rewritePromptImageTargets,
} from '../usePromptTreePromptActions';
import { buildRerunPrompt } from '../verificationTemplates';

let passed = 0;
let failed = 0;

async function test(description: string, fn: () => void | Promise<void>): Promise<void> {
    try {
        await fn();
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
// Tests
// ---------------------------------------------------------------------------

await test('verifies default rerun options and count constants', () => {
    assertEqual(RERUN_DEFAULT_MESSAGE_OPTION, 'randomized');
    assertEqual(RERUN_DEFAULT_COUNT, 4);
    assertEqual(RERUN_MIN_COUNT, 1);
    assertTrue(RERUN_MAX_COUNT >= 8);
    assertEqual(RERUN_MESSAGE_OPTION_STORAGE_KEY, 'agm:prompt-tree:rerun-option');
    assertEqual(RERUN_COUNT_STORAGE_KEY, 'agm:prompt-tree:rerun-count');
});

await test('verifies randomized message set contains at least 25 distinct actionable variations', () => {
    assertTrue(RERUN_RANDOM_MESSAGE_SET.length >= 25);

    const requiredPhrases = [
        'Is it completed?',
        'Is it finished?',
        'Is it released?',
        'Is it done?',
        'Is it published?',
        'Has this been fully implemented and verified?',
        'Can you check the pending tasks and continue?',
        'Is the implementation done and pushed?',
        'Please verify and finish the remaining items.',
        'Did all tests pass and is everything green?',
        'Are all requirements satisfied?',
        'Please double check completion and finalize.',
        'Is there any pending work left?',
        'Please confirm completion status and next steps.',
        'Run comprehensive verification and report status.',
        'Check pending tasks and proceed with the next one.',
        'Verify build cleanliness and finalize.',
        'Confirm all changes are committed and pushed.',
        'Quick status check — is it done yet?',
        'Please continue where you left off and confirm when finished.',
        'Checking in: how is it going? Any blockers or pending items?',
        'Status update please: what is done and what is left to complete?',
        'Are all pending plans completed and verified?',
        'Please confirm when the task is fully finished and ready.',
        'What is the current status of the implementation?',
    ];

    for (const phrase of requiredPhrases) {
        assertTrue(
            RERUN_RANDOM_MESSAGE_SET.includes(phrase),
            `Missing required phrase: "${phrase}"`
        );
    }
});

await test('verifies buildRerunPrompt strips rerun header and retains underlying prompt when markers missing', () => {
    const priorRerunPrompt = [
        '# 🔄 RERUN EXECUTION NUDGE — ROUND 1 OF 4',
        '',
        '> [!NOTE]',
        '> This is automated execution nudge **Round 1 of 4** for the task below.',
        '>',
        '> - **State Preservation:** Review files already modified and steps already completed.',
        '> **Specific Nudge:** Is it completed?',
        '',
        '### Original Task Prompt:',
        '---',
        'Implement authentication tokens and verify API endpoints',
        '---',
        '',
        'Please provide an updated status of your progress in this round and complete any remaining work.',
    ].join('\n');

    const result = buildRerunPrompt(priorRerunPrompt, 2, 4);
    assertTrue(result.includes('Implement authentication tokens and verify API endpoints'));
    assertTrue(result.includes('ROUND 2 OF 4'));
    assertTrue(result.includes('<<<ORIGINAL_TASK_PROMPT_START>>>'));
    assertTrue(result.includes('<<<ORIGINAL_TASK_PROMPT_END>>>'));
});

await test('verifies resolveRerunMessage picks from RERUN_RANDOM_MESSAGE_SET when randomized', () => {
    const pickedMessages = new Set<string>();
    for (let i = 0; i < 50; i++) {
        const { message, pickedIndex } = resolveRerunMessage('randomized');
        assertTrue(pickedIndex !== null);
        assertTrue(pickedIndex! >= 0 && pickedIndex! < RERUN_RANDOM_MESSAGE_SET.length);
        assertTrue(RERUN_RANDOM_MESSAGE_SET.includes(message));
        pickedMessages.add(message);
    }
    assertTrue(pickedMessages.size > 1, 'Random picker did not vary over 50 iterations');
});

await test('verifies resolveRerunMessage avoids previous picks when excludeIndices is provided', () => {
    const used = new Set<number>();
    const pickedPhrases = new Set<string>();
    // Pick 5 consecutive variations
    for (let round = 0; round < 5; round++) {
        const { message, pickedIndex } = resolveRerunMessage('randomized', used);
        assertTrue(pickedIndex !== null);
        assertFalse(used.has(pickedIndex!), `picked index ${pickedIndex} was already in excluded set`);
        used.add(pickedIndex!);
        pickedPhrases.add(message);
    }
    assertEqual(used.size, 5);
    assertEqual(pickedPhrases.size, 5);
});

await test('verifies resolveRerunMessage returns expected fixed messages for non-randomized options', () => {
    assertEqual(resolveRerunMessage('is-it-completed').message, 'Is it completed?');
    assertEqual(resolveRerunMessage('is-it-finished').message, 'Is it finished?');
    assertEqual(resolveRerunMessage('is-it-released').message, 'Is it released?');
    assertEqual(resolveRerunMessage('is-it-done').message, 'Is it done?');
    assertEqual(resolveRerunMessage('is-it-published').message, 'Is it published?');
    assertEqual(
        resolveRerunMessage('verify-and-finish').message,
        'Please verify and finish the remaining items.'
    );
    assertEqual(
        resolveRerunMessage('check-pending-tasks').message,
        'Can you check the pending tasks and continue?'
    );
    assertEqual(
        resolveRerunMessage('run-verification').message,
        'Run comprehensive verification and report status.'
    );
    assertEqual(
        resolveRerunMessage('confirm-pushed').message,
        'Confirm all changes are committed and pushed.'
    );
});

await test('verifies loadPersistedRerunMessageOption defaults to randomized and reads from localStorage', () => {
    window.localStorage.clear();
    assertEqual(loadPersistedRerunMessageOption(), 'randomized');

    savePersistedRerunMessageOption('is-it-finished');
    assertEqual(loadPersistedRerunMessageOption(), 'is-it-finished');

    // Backward compatibility with legacy key
    window.localStorage.removeItem('agm:prompt-tree:rerun-option');
    window.localStorage.setItem('agm_prompt_tree_rerun_message_option', 'is-it-released');
    assertEqual(loadPersistedRerunMessageOption(), 'is-it-released');
});

await test('verifies loadPersistedRerunCount clamps values within bounds [1, RERUN_MAX_COUNT]', () => {
    window.localStorage.clear();
    assertEqual(loadPersistedRerunCount(), 4);

    savePersistedRerunCount(2);
    assertEqual(loadPersistedRerunCount(), 2);

    // Below min clamps to 1
    savePersistedRerunCount(0);
    assertEqual(loadPersistedRerunCount(), 1);

    // Above max clamps to RERUN_MAX_COUNT
    savePersistedRerunCount(999);
    assertEqual(loadPersistedRerunCount(), RERUN_MAX_COUNT);
});

await test('verifies dispatchRerunMessages enqueues all rounds (1..N) via enqueue_prompt and 0 via send_prompt_now', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = null;

    const result = await dispatchRerunMessages({
        targetInstId: 'inst-test-1',
        repoPath: '/work/test-repo',
        convId: 'conv-test-1',
        count: 3,
        messageOption: 'randomized',
        originalPrompt: 'Complete task and push changes',
        source: 'testRunner',
    });

    assertEqual(result.requested, 3);
    assertEqual(result.dispatched, 3);
    assertEqual(result.failed, 0);
    assertTrue(Array.isArray(result.messages));
    assertEqual(result.messages!.length, 3);

    // Verify messages are varied
    const uniqueMessages = new Set(result.messages!);
    assertEqual(uniqueMessages.size, 3, 'All 3 randomized messages should be distinct');

    // Filter invoke calls for prompt dispatches
    const promptCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now' || c.cmd === 'enqueue_prompt');
    assertEqual(promptCalls.length, 3);

    // Invariant: Rerun ALWAYS enqueues into SQLite queue and NEVER sends directly
    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Rerun must never call send_prompt_now directly');

    const enqueueCalls = mockInvokeCalls.filter((c) => c.cmd === 'enqueue_prompt');
    assertEqual(enqueueCalls.length, 3, 'All 3 rerun rounds must be enqueued via enqueue_prompt');

    // All rounds must be enqueue_prompt (FIFO enqueued)
    for (let i = 0; i < 3; i++) {
        assertEqual(promptCalls[i].cmd, 'enqueue_prompt');
        assertEqual(promptCalls[i].args.instanceId, 'inst-test-1');
        assertEqual(promptCalls[i].args.conversationId, 'conv-test-1');
    }
});

await test('verifies dispatchRerunMessages for single count strictly enqueues and never sends directly', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = null;

    const result = await dispatchRerunMessages({
        targetInstId: 'inst-single-1',
        repoPath: '/work/test-repo',
        convId: 'conv-single-1',
        count: 1,
        messageOption: 'randomized',
        originalPrompt: 'Single count rerun test',
        source: 'testRunner',
    });

    assertEqual(result.requested, 1);
    assertEqual(result.dispatched, 1);
    assertEqual(result.failed, 0);

    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Single count rerun must never call send_prompt_now');

    const enqueueCalls = mockInvokeCalls.filter((c) => c.cmd === 'enqueue_prompt');
    assertEqual(enqueueCalls.length, 1, 'Single count rerun must call enqueue_prompt');
    assertEqual(enqueueCalls[0].args.instanceId, 'inst-single-1');
    assertEqual(enqueueCalls[0].args.conversationId, 'conv-single-1');
});

await test('verifies dispatchRerunMessages handles errors gracefully without crashing', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = 'enqueue_prompt'; // Round 2..N will fail
    mockInvokeFailAfterCount = 1;         // Round 1 succeeds, Round 2 fails

    const result = await dispatchRerunMessages({
        targetInstId: 'inst-err-1',
        repoPath: '/work/test-repo',
        convId: 'conv-err-1',
        count: 2,
        messageOption: 'is-it-completed',
        originalPrompt: 'Check test status',
        source: 'testRunner',
    });

    assertEqual(result.requested, 2);
    assertEqual(result.dispatched, 1); // Round 1 succeeded
    assertEqual(result.failed, 1);     // Round 2 failed
    assertEqual(result.messages!.length, 1);

    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Rerun must never call send_prompt_now even during errors');

    mockInvokeFailCmd = null;
    mockInvokeFailAfterCount = null;
});

await test('verifies all 10 diverse actionable questions are present in RERUN_RANDOM_MESSAGE_SET', () => {
    const requiredActionableQuestions = [
        'Is it finished?',
        'Is it released?',
        'Is it completed?',
        'Are you confident?',
        'Did all tests pass?',
        'Are all requirements satisfied?',
        'Is the verification green?',
        'Are edge cases covered?',
        'Is documentation updated?',
        'Is the build passing cleanly?',
    ];
    for (const q of requiredActionableQuestions) {
        assertTrue(
            RERUN_RANDOM_MESSAGE_SET.includes(q),
            `Expected RERUN_RANDOM_MESSAGE_SET to include "${q}"`
        );
    }
});

await test('verifies pickDiverseRerunNudge and getRandomRerunNudge never repeat across 5 successive calls with history', () => {
    const history: string[] = [];
    for (let i = 0; i < 5; i++) {
        const nudge = pickDiverseRerunNudge(history);
        assertTrue(typeof nudge === 'string' && nudge.length > 0);
        assertTrue(RERUN_RANDOM_MESSAGE_SET.includes(nudge));
        assertFalse(
            history.includes(nudge),
            `Turn ${i + 1} repeated nudge "${nudge}" already present in recent history: [${history.join(', ')}]`
        );
        history.push(nudge);
    }
    assertEqual(history.length, 5);
    assertEqual(new Set(history).size, 5);

    // Verify getRandomRerunNudge behaves identically and respects history
    const aliasNudge = getRandomRerunNudge(history);
    assertTrue(typeof aliasNudge === 'string' && aliasNudge.length > 0);
    assertFalse(
        history.includes(aliasNudge),
        `getRandomRerunNudge repeated recent nudge "${aliasNudge}"`
    );
});

await test('verifies pickDiverseRerunNudge fallback correctly samples from remaining options when pool is saturated', () => {
    // Saturated history containing all messages
    const fullHistory = [...RERUN_RANDOM_MESSAGE_SET];
    const fallbackPick = pickDiverseRerunNudge(fullHistory);
    assertTrue(RERUN_RANDOM_MESSAGE_SET.includes(fallbackPick));
    // The fallback should avoid the very last item in fullHistory if possible
    const lastItem = fullHistory[fullHistory.length - 1];
    assertTrue(
        fallbackPick !== lastItem || RERUN_RANDOM_MESSAGE_SET.length === 1,
        'Fallback should avoid the immediate last used question'
    );
});

await test('verifies dispatchRerunMessages carries history and enforces anti-repeat across consecutive runs', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = null;

    const run1 = await dispatchRerunMessages({
        targetInstId: 'inst-multi-1',
        repoPath: '/work/test-repo',
        convId: 'conv-multi-1',
        count: 3,
        messageOption: 'randomized',
        originalPrompt: 'Multi-turn task execution',
        source: 'testRunner',
        history: [],
    });

    assertEqual(run1.requested, 3);
    assertEqual(run1.dispatched, 3);
    assertEqual(run1.messages!.length, 3);

    // Run 2 passes run1 messages as history
    const run2 = await dispatchRerunMessages({
        targetInstId: 'inst-multi-1',
        repoPath: '/work/test-repo',
        convId: 'conv-multi-1',
        count: 2,
        messageOption: 'randomized',
        originalPrompt: 'Multi-turn task execution',
        source: 'testRunner',
        history: run1.messages,
    });

    assertEqual(run2.requested, 2);
    assertEqual(run2.dispatched, 2);
    assertEqual(run2.messages!.length, 2);

    // Verify no overlaps between run1 (3 messages) and run2 (2 messages) since total = 5 <= RERUN_HISTORY_WINDOW (5)
    for (const m of run2.messages!) {
        assertFalse(run1.messages!.includes(m), `run2 message "${m}" was repeated from run1 history`);
    }

    // Combined 5 messages should be all distinct
    const combined = [...run1.messages!, ...run2.messages!];
    assertEqual(new Set(combined).size, 5);

    // Verify ALL rounds across both runs strictly enqueue via enqueue_prompt, zero send_prompt_now
    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Rerun must never call send_prompt_now directly across any run');

    const enqueueCalls = mockInvokeCalls.filter((c) => c.cmd === 'enqueue_prompt');
    assertEqual(enqueueCalls.length, 5, 'All 5 rerun dispatches across both runs must call enqueue_prompt');

    const run1Calls = enqueueCalls.slice(0, 3);
    assertEqual(run1Calls[0].cmd, 'enqueue_prompt');
    assertEqual(run1Calls[1].cmd, 'enqueue_prompt');
    assertEqual(run1Calls[2].cmd, 'enqueue_prompt');

    const run2Calls = enqueueCalls.slice(3, 5);
    assertEqual(run2Calls[0].cmd, 'enqueue_prompt');
    assertEqual(run2Calls[1].cmd, 'enqueue_prompt');
});

test('current conversation keeps its id while the conversation is running', () => {
    const runningId = 'bc221407-c445-415a-9034-b1179a550a79';
    assertEqual(dispatchConversationId('current', runningId, true), runningId);
    assertEqual(dispatchConversationId('current', `prompt-${runningId}`, true), `prompt-${runningId}`);
    assertEqual(dispatchConversationId('new', runningId, true), 'new');
    assertEqual(dispatchConversationId('current', '   ', false), undefined);
});

test('dispatched image markdown is a workspace-relative file the IDE can open', () => {
    const prompt = 'Look ![Shot](file:///D:/work/gitmap/assets/screenshots/shot.png)';
    const rewritten = rewritePromptImageTargets(prompt, 'D:/work/gitmap');
    assertEqual(rewritten, 'Look ![Shot](assets/screenshots/shot.png)');
    assertFalse(rewritten.includes('file:///'), 'absolute file URI must not be sent to the IDE');
});

test('extractUsedNudgesFromConversation extracts used variations from conversation turns and history', () => {
    const mockConv = {
        conversation_id: 'conv-test-1',
        prompts: [
            { turn_number: 1, prompt_text: 'Initial implementation task' },
            { turn_number: 2, prompt_text: 'Did all tests pass and is everything green?' },
            { turn_number: 3, prompt_text: 'Is it completed?' },
        ],
    } as any;

    const used = extractUsedNudgesFromConversation(mockConv, ['Is it finished?']);
    assertTrue(used.includes('Did all tests pass and is everything green?'));
    assertTrue(used.includes('Is it completed?'));
    assertTrue(used.includes('Is it finished?'));
    assertEqual(used.length, 3);
});

await test('verifies dispatchMultiPromptResend enqueues all selected prompts via enqueue_prompt and 0 via send_prompt_now', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = null;

    const result = await dispatchMultiPromptResend({
        instanceId: 'inst-multi-resend-1',
        repoPath: '/work/test-repo',
        conversationId: 'conv-multi-resend-1',
        count: 3,
        delayMs: 0,
        prompts: [
            'Prompt turn 1: setup architecture',
            'Prompt turn 2: implement components',
            'Prompt turn 3: verify test suite',
        ],
        mediaPaths: ['assets/diagram.png'],
    });

    assertEqual(result.sent, 3, 'All 3 prompts should be marked as sent');
    assertEqual(result.failed, 0, 'No prompt should have failed');

    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Multi-prompt resend must NEVER call send_prompt_now directly');

    const enqueueCalls = mockInvokeCalls.filter((c) => c.cmd === 'enqueue_prompt');
    assertEqual(enqueueCalls.length, 3, 'All 3 prompts must be routed to enqueue_prompt in FIFO order');

    for (let i = 0; i < 3; i++) {
        const call = enqueueCalls[i];
        assertEqual(call.args.instanceId, 'inst-multi-resend-1');
        assertEqual(call.args.repoPath, '/work/test-repo');
        assertEqual(call.args.conversationId, 'conv-multi-resend-1');
        assertTrue(Array.isArray(call.args.mediaPaths), 'mediaPaths must be an array');
        assertTrue(call.args.mediaPaths.includes('assets/diagram.png'), 'mediaPaths must preserve attachment');
    }

    assertTrue(enqueueCalls[0].args.promptContent.includes('Prompt turn 1'));
    assertTrue(enqueueCalls[1].args.promptContent.includes('Prompt turn 2'));
    assertTrue(enqueueCalls[2].args.promptContent.includes('Prompt turn 3'));
});

await test('verifies dispatchMultiPromptResend isolates individual failures without aborting remaining batch items', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = 'enqueue_prompt';
    mockInvokeFailAfterCount = 1;

    const result = await dispatchMultiPromptResend({
        instanceId: 'inst-multi-fail-1',
        repoPath: '/work/test-repo',
        conversationId: 'conv-multi-fail-1',
        count: 3,
        delayMs: 0,
        prompts: ['Step 1 ok', 'Step 2 boom', 'Step 3 boom'],
    });

    assertEqual(result.sent, 1, 'Only the first prompt should be marked as sent');
    assertEqual(result.failed, 2, 'Two subsequent prompts should be recorded as failed');

    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'send_prompt_now must remain 0 even when enqueue fails');

    mockInvokeFailCmd = null;
    mockInvokeFailAfterCount = null;
});

await test('verifies prompt rerun attachment formatting with mediaPaths produces canonical @[file;...] bracket syntax', async () => {
    mockInvokeCalls.length = 0;
    mockInvokeFailCmd = null;

    const result = await dispatchRerunMessages({
        targetInstId: 'inst-rerun-media-1',
        repoPath: '/work/test-repo',
        convId: 'conv-rerun-media-1',
        count: 1,
        messageOption: 'is-it-completed',
        originalPrompt: 'Verify rerun with media attachment',
        source: 'testRunner',
        mediaPaths: ['assets/test.png'],
    });

    assertEqual(result.dispatched, 1);
    assertEqual(result.failed, 0);

    const sendCalls = mockInvokeCalls.filter((c) => c.cmd === 'send_prompt_now');
    assertEqual(sendCalls.length, 0, 'Must never call send_prompt_now');

    const enqueueCalls = mockInvokeCalls.filter((c) => c.cmd === 'enqueue_prompt');
    assertEqual(enqueueCalls.length, 1, 'Must invoke enqueue_prompt once');

    const promptCall = enqueueCalls[0];
    assertEqual(promptCall.args.instanceId, 'inst-rerun-media-1');
    assertEqual(promptCall.args.conversationId, 'conv-rerun-media-1');

    const content = String(promptCall.args.promptContent);
    assertTrue(
        content.includes('@[file_name_sequence](file;'),
        `Enqueued prompt content must contain canonical "@[file_name_sequence](file;" prefix. Got:\n${content}`
    );
    assertTrue(
        content.includes('1. @[file_name_sequence](file;assets/test.png)') ||
        content.includes('@[file_name_sequence](file;/work/test-repo/assets/test.png)') ||
        content.includes('@[file_name_sequence](file;\\work\\test-repo\\assets\\test.png)'),
        `Enqueued prompt content must format attachment canonically with @[file_name_sequence](file;...). Got:\n${content}`
    );
    assertFalse(
        /1\.\s*@\[(?!file;).*?assets\/test\.png\]/.test(content),
        'Enqueued prompt must not contain legacy @[path] attachment format without file; prefix'
    );
});

// ===========================================================================
// Summary
// ===========================================================================

console.log(`\nrerunActions verification: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} rerunActions test(s) failed`);
}
