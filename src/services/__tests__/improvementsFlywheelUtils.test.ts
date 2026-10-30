/**
 * Flywheel helpers: instance targeting and toast copy.
 * Run: npx tsx src/services/__tests__/improvementsFlywheelUtils.test.ts
 */
import { emptyFlywheelState } from '../improvementsService';
import {
    collectWorkspaceTaskHints,
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
        flywheelActionToast('next', true, {
            next_batch: 3,
            completed_batches: 2,
            task_hints: ['01-fix-login'],
        }),
        'Next batch 3 previewed (1 pending task)',
    );
    assertEqual(
        flywheelActionToast('next', false, { next_batch: 2, completed_batches: 1 }),
        'Dispatched next 10 (batch 1)',
    );
    assertEqual(
        flywheelActionToast('detail', true, {
            next_batch: 1,
            completed_batches: 0,
            task_hints: ['a', 'b'],
        }),
        'Detail-out previewed (2 pending tasks in this workspace)',
    );
    assertEqual(
        flywheelActionToast('detail', false, {
            next_batch: 1,
            completed_batches: 0,
            task_hints: ['a', 'b'],
        }),
        'Queued 100-step breakdown for 2 pending tasks in the active workspace',
    );
    assertEqual(
        flywheelActionToast('check', false, { next_batch: 1, completed_batches: 0 }),
        'Ledger refreshed',
    );
});

test('empty flywheel state never throws and keeps 10 pending batches', () => {
    const empty = emptyFlywheelState();
    assertEqual(empty.last_action, 'unplanned');
    assertEqual(empty.batches.length, 10);
    assertEqual(empty.batches[0]?.status, 'pending');
    assertEqual(empty.task_hints?.length ?? -1, 0);
});

test('next breakdown lists pending tasks and skips completed or readme', () => {
    const hints = collectWorkspaceTaskHints([
        'proj/.ai-memory/plans/readme.md',
        'proj/.ai-memory/plans/completed/99-old-done.md',
        'proj/.ai-memory/plans/pending/01-fix-login.md',
        'proj/.ai-memory/issues/66-open-switch.md',
        'proj/.ai-memory/plans/active-roadmap.md',
        'proj/.ai-memory/ambiguous-questions/01-new-ambiguity/q-session.md',
        'proj/.ai-memory/plans/subtasks/01-fix-login/01-step.md',
        'other/README.md',
    ]);
    assertEqual(hints[0], '01-fix-login');
    assertEqual(hints.includes('66-open-switch'), true);
    assertEqual(hints.includes('active-roadmap'), true);
    assertEqual(hints.includes('q-session'), true);
    assertEqual(hints.includes('99-old-done'), false);
    assertEqual(hints.includes('readme'), false);
    assertEqual(hints.includes('README'), false);
});

test('next breakdown finds real AGM tasks without requiring 01-prompts', () => {
    const hints = collectWorkspaceTaskHints([
        '.ai-memory/issues/66-instances-switch-json-fake-success.md',
        '.ai-memory/issues/67-prompt-tree-enqueue-zero-after-click.md',
        '.ai-memory/issues/readme.md',
        '.ai-memory/plans/completed/237-prompt-tree-enqueue-visibility.md',
        '.ai-memory/plans/gitmap-pe-execution-and-csv-issue-resolution.md',
    ]);
    assertEqual(hints.includes('66-instances-switch-json-fake-success'), true);
    assertEqual(hints.includes('67-prompt-tree-enqueue-zero-after-click'), true);
    assertEqual(hints.includes('gitmap-pe-execution-and-csv-issue-resolution'), true);
    assertEqual(hints.includes('237-prompt-tree-enqueue-visibility'), false);
    assertEqual(hints.includes('readme'), false);
});

console.log(`improvementsFlywheelUtils: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} improvementsFlywheelUtils test(s) failed`);
}
