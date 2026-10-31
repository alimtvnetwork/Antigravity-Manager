import {
    formatPromptWithAttachments,
    extractAttachmentsFromPrompt,
    stripAttachmentsFromPrompt,
    normalizeAttachmentPath,
    isFileAttachmentTarget,
} from '../promptAttachment';

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

function assert(condition: boolean, message = 'Assertion failed'): void {
    if (!condition) {
        throw new Error(message);
    }
}

function assertEqual<T>(actual: T, expected: T): void {
    if (actual !== expected) {
        throw new Error(`expected "${expected}", got "${actual}"`);
    }
}

// Test 1: Canonical formatting uses [Attachment XX | Description | filename](file;<path>)
test('formatPromptWithAttachments outputs canonical [Attachment XX | Description | filename](file;<path>) format', () => {
    const prompt = 'Investigate error in billing component';
    const attachments = ['assets/screenshots/billing-error.png', 'assets/screenshots/network-trace.png'];
    const result = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });

    assert(result.includes('Attachments:'), 'Result should include Attachments: header');
    assert(result.includes('1. [Attachment 01 | Billing Error Screenshot | billing-error.png](file;assets/screenshots/billing-error.png)'), 'Should contain 1. [Attachment 01 | ...](file;...)');
    assert(result.includes('2. [Attachment 02 | Network Trace Screenshot | network-trace.png](file;assets/screenshots/network-trace.png)'), 'Should contain 2. [Attachment 02 | ...](file;...)');
});

// Test 2: Extraction handles canonical [Attachment XX | Description | filename](file;<path>)
test('extractAttachmentsFromPrompt extracts clean path from canonical [Attachment XX | Description | filename](file;<path>)', () => {
    const text = 'Check UI:\nAttachments:\n1. [Attachment 01 | Screenshot Attachment | screenshot.png](file;assets/screenshot.png)';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/screenshot.png');
});

// Test 2b: Backward compatibility for previous intermediate syntax @[file_name_sequence](file;<path>)
test('extractAttachmentsFromPrompt extracts clean path from @[file_name_sequence](file;<path>)', () => {
    const text = 'Check UI:\nAttachments:\n1. @[file_name_sequence](file;assets/screenshot.png)';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/screenshot.png');
});

// Test 3: Backward compatibility for legacy @[<path>]
test('extractAttachmentsFromPrompt maintains backward compatibility with legacy @[<path>]', () => {
    const text = 'Check UI:\nAttachments:\n1. @[assets/legacy.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/legacy.png');
});

// Test 4: Tolerates whitespace surrounding semicolon
test('extractAttachmentsFromPrompt tolerates whitespace after file semicolon', () => {
    const text = 'Check UI:\nAttachments:\n1. @[file_name_sequence](file;   assets/spaced.png)';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/spaced.png');
});

// Test 5: Mixed formats deduplicate cleanly
test('extractAttachmentsFromPrompt deduplicates mixed canonical and legacy inputs', () => {
    const text = '1. @[file_name_sequence](file;assets/dup.png)\n2. @[assets/dup.png]';
    const extracted = extractAttachmentsFromPrompt(text, undefined, false);
    assertEqual(extracted.length, 1);
    assertEqual(extracted[0], 'assets/dup.png');
});

// Test 6: stripAttachmentsFromPrompt removes canonical @[file_name_sequence](file;...) blocks
test('stripAttachmentsFromPrompt removes canonical and legacy attachment blocks', () => {
    const text = 'Some prompt\n\nAttachments:\n1. @[file_name_sequence](file;assets/shot.png)\n2. @[file_name_sequence](file;assets/shot2.png)';
    const cleaned = stripAttachmentsFromPrompt(text);
    assertEqual(cleaned, 'Some prompt');
});

// Test 7: stripAttachmentsFromPrompt removes mixed canonical and legacy attachment blocks
test('stripAttachmentsFromPrompt removes mixed canonical and legacy blocks', () => {
    const text = 'Some prompt\n\nAttachments:\n1. @[file_name_sequence](file;assets/shot.png)\n2. @[assets/legacy.png]';
    const cleaned = stripAttachmentsFromPrompt(text);
    assertEqual(cleaned, 'Some prompt');
});

// Test 8: Preserves </USER_REQUEST> tag position
test('formatPromptWithAttachments inserts attachments before </USER_REQUEST>', () => {
    const prompt = 'Please fix this bug\n</USER_REQUEST>';
    const attachments = ['assets/bug.png'];
    const result = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });

    assert(result.indexOf('Attachments:') < result.indexOf('</USER_REQUEST>'), 'Attachments should come before tag');
    assert(result.includes('1. [Attachment 01 | Bug Screenshot | bug.png](file;assets/bug.png)'), 'Should contain canonical attachment');
});

// Test 9: normalizeAttachmentPath strips file; prefix and normalizes correctly
test('normalizeAttachmentPath strips file; prefix', () => {
    const normalized = normalizeAttachmentPath('file;assets/test.png', undefined, false);
    assertEqual(normalized, 'assets/test.png');

    const normalizedSpaced = normalizeAttachmentPath('file;   assets/test.png', undefined, false);
    assertEqual(normalizedSpaced, 'assets/test.png');
});

// Test 10: extractRawTarget handles object inputs with file; prefix
test('formatPromptWithAttachments handles object inputs with file; prefix', () => {
    const prompt = 'Check object input';
    const attachments = [{ path: 'file;assets/from-path.png' }, { src: 'file;assets/from-src.png' }];
    const result = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });

    assert(result.includes('1. [Attachment 01 | From Path Screenshot | from-path.png](file;assets/from-path.png)'), 'Should format object path item canonically');
    assert(result.includes('2. [Attachment 02 | From Src Screenshot | from-src.png](file;assets/from-src.png)'), 'Should format object src item canonically');
});

// Test 11: Mixed markdown and bracketed images extraction
test('extractAttachmentsFromPrompt extracts both markdown and canonical bracketed images', () => {
    const prompt = 'Here is an image: ![Alt](assets/md-img.png)\n\nAttachments:\n1. @[file_name_sequence](file;assets/bracket-img.png)';
    const extracted = extractAttachmentsFromPrompt(prompt, undefined, false);

    assertEqual(extracted.length, 2);
    assertEqual(extracted[0], 'assets/md-img.png');
    assertEqual(extracted[1], 'assets/bracket-img.png');
});

// Test 12: Idempotent reformatting produces identical canonical output
test('formatPromptWithAttachments is idempotent on re-formatting', () => {
    const prompt = 'Initial prompt';
    const attachments = ['assets/image1.png'];
    const firstPass = formatPromptWithAttachments(prompt, attachments, { isWindowsPlatformOverride: false });
    const secondPass = formatPromptWithAttachments(firstPass, [], { isWindowsPlatformOverride: false });

    assertEqual(firstPass, secondPass);
});

const SKILL_COPY_PROMPT = `instead of @[.agents/skills/execute-parent-task-with-n-steps-v7]

Write as

@[execute-parent-task-with-n-steps-v7](file;.agents/skills/execute-parent-task-with-n-steps-v7)`;

test('isFileAttachmentTarget rejects skill refs and accepts image files', () => {
    assert(!isFileAttachmentTarget('.agents/skills/execute-parent-task-with-n-steps-v7'), 'skill dir is not a file attachment');
    assert(!isFileAttachmentTarget('execute-parent-task-with-n-steps-v7'), 'skill name is not a file attachment');
    assert(isFileAttachmentTarget('assets/screenshot.png'), 'png is a file attachment');
    assert(isFileAttachmentTarget('file;assets/screenshot.png'), 'canonical file; png is a file attachment');
});

test('extractAttachmentsFromPrompt ignores inline skill refs', () => {
    const extracted = extractAttachmentsFromPrompt(SKILL_COPY_PROMPT, undefined, false);
    assertEqual(extracted.length, 0);
});

test('formatPromptWithAttachments does not rewrite skill refs into an Attachments block', () => {
    const result = formatPromptWithAttachments(SKILL_COPY_PROMPT, [], { isWindowsPlatformOverride: false });
    assert(!result.includes('Attachments:'), 'Skill refs must not become an Attachments block');
    assert(result.includes('@[.agents/skills/execute-parent-task-with-n-steps-v7]'), 'Legacy skill ref must be preserved');
    assert(
        result.includes('@[execute-parent-task-with-n-steps-v7](file;.agents/skills/execute-parent-task-with-n-steps-v7)'),
        'Canonical skill ref must be preserved'
    );
});

console.log(`\npromptAttachment tests: ${passed} passed, ${failed} failed`);
if (failed > 0) {
    throw new Error(`${failed} promptAttachment test(s) failed`);
}
