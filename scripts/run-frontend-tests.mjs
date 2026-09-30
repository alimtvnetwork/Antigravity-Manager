#!/usr/bin/env node
// Runs every standalone frontend test script (src/**/__tests__/*.test.ts) through tsx.
// Each test file throws when an assertion fails, so a non-zero exit code means failure.
//
// Usage:
//   npm run test                      run all frontend tests
//   npm run test -- stack-frame       run only files whose path contains "stack-frame"

import { spawnSync } from 'node:child_process';
import { readdirSync, statSync } from 'node:fs';
import { join, relative, sep } from 'node:path';
import { fileURLToPath } from 'node:url';

const repoRoot = join(fileURLToPath(new URL('.', import.meta.url)), '..');
const srcRoot = join(repoRoot, 'src');
const filter = process.argv[2] ?? '';

function collectTestFiles(dir) {
    const files = [];
    for (const name of readdirSync(dir)) {
        const fullPath = join(dir, name);
        if (statSync(fullPath).isDirectory()) {
            files.push(...collectTestFiles(fullPath));
        } else if (fullPath.includes(`${sep}__tests__${sep}`) && name.endsWith('.test.ts')) {
            files.push(fullPath);
        }
    }
    return files;
}

const testFiles = collectTestFiles(srcRoot)
    .map((file) => relative(repoRoot, file))
    .filter((file) => file.includes(filter))
    .sort();

if (testFiles.length === 0) {
    console.error(`No frontend test files matched "${filter}".`);
    process.exit(1);
}

const failedFiles = [];
for (const file of testFiles) {
    const result = spawnSync('npx', ['--yes', 'tsx', file], {
        cwd: repoRoot,
        shell: true,
        encoding: 'utf8',
    });
    const isPassed = result.status === 0;
    console.log(`${isPassed ? 'PASS' : 'FAIL'}  ${file}`);
    if (!isPassed) {
        failedFiles.push(file);
        console.error(`${result.stdout ?? ''}${result.stderr ?? ''}`);
    }
}

console.log(`\n${testFiles.length - failedFiles.length}/${testFiles.length} test files passed`);
process.exit(failedFiles.length === 0 ? 0 : 1);
