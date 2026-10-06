/**
 * Standalone regression tests for resolveFocusTarget.
 * Run: npx tsx src/lib/__tests__/resolve-focus-target.test.ts
 */
import { resolveFocusTarget } from '../resolve-focus-target';

let passed = 0;
let failed = 0;

function test(description: string, fn: () => void): void {
  try {
    fn();
    passed++;
  } catch (e: unknown) {
    failed++;
    const msg = e instanceof Error ? e.message : String(e);
    console.error(`  FAIL: ${description} - ${msg}`);
  }
}

function assertEqual<T>(actual: T, expected: T): void {
  if (actual !== expected) {
    throw new Error(`expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
}

test('returns selectedInstanceAccountId when both are provided', () => {
  assertEqual(resolveFocusTarget('acc-inst-1', 'acc-curr-2'), 'acc-inst-1');
});

test('falls back to currentAccountId when selectedInstanceAccountId is null or empty', () => {
  assertEqual(resolveFocusTarget(null, 'acc-curr-2'), 'acc-curr-2');
  assertEqual(resolveFocusTarget(undefined, 'acc-curr-2'), 'acc-curr-2');
  assertEqual(resolveFocusTarget('   ', 'acc-curr-2'), 'acc-curr-2');
});

test('returns null when neither is provided', () => {
  assertEqual(resolveFocusTarget(null, null), null);
  assertEqual(resolveFocusTarget(undefined, undefined), null);
  assertEqual(resolveFocusTarget('', ''), null);
  assertEqual(resolveFocusTarget('   ', '   '), null);
});

test('trims whitespace around valid IDs', () => {
  assertEqual(resolveFocusTarget('  acc-inst-1  ', null), 'acc-inst-1');
  assertEqual(resolveFocusTarget(null, '  acc-curr-2  '), 'acc-curr-2');
});

if (failed > 0) {
  throw new Error(`${failed} test(s) failed`);
}

console.log(`${passed} resolve-focus-target test(s) passed`);

