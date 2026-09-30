#!/usr/bin/env node
// Points git at the tracked .githooks directory. Runs on `npm install` via the `prepare` script.
// Never fails the install: tarball installs, CI checkouts without .git, or missing git are tolerated.

import { execFileSync } from 'node:child_process';
import fs from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

const rootDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), '..');
const hooksDir = path.join(rootDir, '.githooks');

if (!fs.existsSync(path.join(rootDir, '.git')) || !fs.existsSync(hooksDir)) {
    process.exit(0);
}

try {
    execFileSync('git', ['config', 'core.hooksPath', '.githooks'], { cwd: rootDir, stdio: 'ignore' });
    console.log('[hooks] core.hooksPath -> .githooks (pre-commit rustfmt guard enabled)');
} catch {
    console.warn('[hooks] Could not set core.hooksPath; run `git config core.hooksPath .githooks` manually.');
}
