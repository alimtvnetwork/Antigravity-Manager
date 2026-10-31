import { maskEmail } from '../maskEmail';

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

test('maskEmail displays 4 characters first and hides @gmail.com', () => {
    assertEqual(maskEmail('david.smith@gmail.com'), 'davi***');
    assertEqual(maskEmail('kevin@gmail.com'), 'kevi***');
    assertEqual(maskEmail('riseup.asia.team@gmail.com'), 'rise***');
    assertEqual(maskEmail('developer@GMAIL.COM'), 'deve***');
});

test('maskEmail hides @googlemail.com domain', () => {
    assertEqual(maskEmail('samuel@googlemail.com'), 'samu***');
});

test('maskEmail handles short local parts with @gmail.com', () => {
    assertEqual(maskEmail('dan@gmail.com'), 'dan***');
    assertEqual(maskEmail('ab@gmail.com'), 'ab***');
    assertEqual(maskEmail('x@gmail.com'), 'x***');
});

test('maskEmail preserves non-gmail domains while displaying 4 local chars', () => {
    assertEqual(maskEmail('worker.alpha@example.com'), 'work***@example.com');
    assertEqual(maskEmail('support@company.org'), 'supp***@company.org');
    assertEqual(maskEmail('it@mycorp.net'), 'it***@mycorp.net');
});

test('maskEmail handles empty or non-email inputs safely', () => {
    assertEqual(maskEmail(''), '—');
    assertEqual(maskEmail('not-an-email'), 'not-an-email');
});

console.log(`\nmaskEmail tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} maskEmail test(s) failed`);
}
