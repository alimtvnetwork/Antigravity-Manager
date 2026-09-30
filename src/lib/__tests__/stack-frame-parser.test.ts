/**
 * Standalone regression tests for parseStackLine.
 * Run: npm run test -- stack-frame-parser
 */
import { parseStackLine } from '../stack-frame-parser';

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

test('URL-only frame (diagnostic report shape)', () => {
  const frame = parseStackLine(
    '    at http://tauri.localhost/assets/index-Lnket4v2.js:535:10057',
  );
  if (!frame) {
    throw new Error('expected a frame');
  }
  assertEqual(frame.function, '<anonymous>');
  assertEqual(frame.file, 'http://tauri.localhost/assets/index-Lnket4v2.js');
  assertEqual(frame.line, 535);
  assertEqual(frame.column, 10057);
});

test('async URL-only frame', () => {
  const frame = parseStackLine(
    '    at async http://tauri.localhost/assets/index-Lnket4v2.js:535:10057',
  );
  if (!frame) {
    throw new Error('expected a frame');
  }
  assertEqual(frame.function, 'async <anonymous>');
  assertEqual(frame.file, 'http://tauri.localhost/assets/index-Lnket4v2.js');
});

test('named frame with URL in parentheses', () => {
  const frame = parseStackLine(
    '    at dP (http://tauri.localhost/assets/index-Lnket4v2.js:14:243)',
  );
  if (!frame) {
    throw new Error('expected a frame');
  }
  assertEqual(frame.function, 'dP');
  assertEqual(frame.file, 'http://tauri.localhost/assets/index-Lnket4v2.js');
  assertEqual(frame.line, 14);
  assertEqual(frame.column, 243);
});

test('async named frame', () => {
  const frame = parseStackLine('    at async foo (http://localhost:1420/src/x.tsx:10:5)');
  if (!frame) {
    throw new Error('expected a frame');
  }
  assertEqual(frame.function, 'foo');
  assertEqual(frame.file, 'http://localhost:1420/src/x.tsx');
});

test('Windows drive path', () => {
  const frame = parseStackLine('    at run (D:\\work\\app\\main.ts:99:1)');
  if (!frame) {
    throw new Error('expected a frame');
  }
  assertEqual(frame.file, 'D:\\work\\app\\main.ts');
  assertEqual(frame.line, 99);
});

test('malformed line returns null', () => {
  assertEqual(parseStackLine('Error: something'), null);
  assertEqual(parseStackLine('not a stack line'), null);
});

if (failed > 0) {
  throw new Error(`${failed} test(s) failed`);
}

console.log(`${passed} stack-frame-parser test(s) passed`);
