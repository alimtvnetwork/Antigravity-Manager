/**
 * Unit Tests for Rerun Randomizer & Storage Persistence (Step 288 / Imp-702)
 *
 * Verifies:
 * 1. Default rerun option is 'randomized' and count defaults to 4.
 * 2. RERUN_RANDOM_MESSAGE_SET contains at least 12 distinct actionable variations.
 * 3. resolveRerunMessage correctly picks random items or fixed phrases.
 * 4. loadPersistedRerunMessageOption and savePersistedRerunMessageOption work with both
 *    primary and legacy localStorage keys.
 * 5. loadPersistedRerunCount and savePersistedRerunCount clamp to valid range [1, 8].
 *
 * Run: node scripts/run-frontend-tests.mjs rerunActions
 *      npm test -- rerunActions
 */

// Mock window and localStorage if running in Node.js
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
    (globalThis as any).window = { localStorage: mockStorage };
    (globalThis as any).localStorage = mockStorage;
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
    loadPersistedRerunMessageOption,
    savePersistedRerunMessageOption,
    loadPersistedRerunCount,
    savePersistedRerunCount,
} from '../usePromptTreePromptActions';

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

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

test('verifies default rerun options and count constants', () => {
    assertEqual(RERUN_DEFAULT_MESSAGE_OPTION, 'randomized');
    assertEqual(RERUN_DEFAULT_COUNT, 4);
    assertEqual(RERUN_MIN_COUNT, 1);
    assertTrue(RERUN_MAX_COUNT >= 8);
    assertEqual(RERUN_MESSAGE_OPTION_STORAGE_KEY, 'agm:prompt-tree:rerun-option');
    assertEqual(RERUN_COUNT_STORAGE_KEY, 'agm:prompt-tree:rerun-count');
});

test('verifies randomized message set contains at least 12 distinct actionable variations', () => {
    assertTrue(RERUN_RANDOM_MESSAGE_SET.length >= 12);

    const requiredPhrases = [
        'Is it completed?',
        'Is it finished?',
        'Is it released?',
        'Has this been fully implemented and verified?',
        'Can you check the pending tasks and continue?',
        'Is the implementation done and pushed?',
        'Please verify and finish the remaining items.',
        'Did all tests pass and is everything green?',
        'Are all requirements satisfied?',
        'Please double check completion and finalize.',
        'Is there any pending work left?',
        'Please confirm completion status and next steps.',
    ];

    for (const phrase of requiredPhrases) {
        assertTrue(
            RERUN_RANDOM_MESSAGE_SET.includes(phrase),
            `Missing required phrase: "${phrase}"`
        );
    }
});

test('verifies resolveRerunMessage picks from RERUN_RANDOM_MESSAGE_SET when randomized', () => {
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

test('verifies resolveRerunMessage returns expected fixed messages for non-randomized options', () => {
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
});

test('verifies loadPersistedRerunMessageOption defaults to randomized and reads from localStorage', () => {
    window.localStorage.clear();
    assertEqual(loadPersistedRerunMessageOption(), 'randomized');

    savePersistedRerunMessageOption('is-it-finished');
    assertEqual(loadPersistedRerunMessageOption(), 'is-it-finished');

    // Backward compatibility with legacy key
    window.localStorage.removeItem('agm:prompt-tree:rerun-option');
    window.localStorage.setItem('agm_prompt_tree_rerun_message_option', 'is-it-released');
    assertEqual(loadPersistedRerunMessageOption(), 'is-it-released');
});

test('verifies loadPersistedRerunCount clamps values within bounds [1, RERUN_MAX_COUNT]', () => {
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

// ===========================================================================
// Summary
// ===========================================================================

console.log(`\nrerunActions verification: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} rerunActions test(s) failed`);
}
