/**
 * Flywheel helpers: instance targeting and toast copy.
 * Run: npx tsx src/services/__tests__/improvementsFlywheelUtils.test.ts
 */
import {
    flywheelActionToast,
    resolveImprovementsInstanceId,
    workspaceLabel,
} from '../improvementsFlywheelUtils';

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
        throw new Error(`expected "${expected}", got "${actual}"`);
    }
}

test('empty or default instance ids resolve to default', () => {
    assertEqual(resolveImprovementsInstanceId(null), 'default');
    assertEqual(resolveImprovementsInstanceId(''), 'default');
    assertEqual(resolveImprovementsInstanceId('__default__'), 'default');
    assertEqual(resolveImprovementsInstanceId('v2-3127'), 'v2-3127');
});

test('workspace label keeps last two path segments', () => {
    assertEqual(workspaceLabel(''), 'active instance workspace');
    assertEqual(workspaceLabel('D:/work/demo-app'), 'work/demo-app');
    assertEqual(workspaceLabel('C:\\Users\\me\\proj'), 'me/proj');
});

test('toast copy covers plan next detail check', () => {
    assertEqual(
        flywheelActionToast('plan', true, { next_batch: 1, completed_batches: 0 }),
        'Planned 10 batches of 10 (dry-run, no prompt dispatched)',
    );
    assertEqual(
        flywheelActionToast('next', false, { next_batch: 2, completed_batches: 1 }),
        'Dispatched next 10 (batch 1)',
    );
    assertEqual(
        flywheelActionToast('detail', false, { next_batch: 1, completed_batches: 0 }),
        'Queued 100-step breakdown for the active workspace',
    );
    assertEqual(
        flywheelActionToast('check', false, { next_batch: 1, completed_batches: 0 }),
        'Ledger refreshed',
    );
});

console.log(`improvementsFlywheelUtils: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} improvementsFlywheelUtils test(s) failed`);
}
