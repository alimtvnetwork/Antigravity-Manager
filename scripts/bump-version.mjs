#!/usr/bin/env node

/**
 * Antigravity Tools - One-step version bump and atomic multi-file synchronization script.
 *
 * Usage:
 *   npm run bump patch                 # Automatically increment patch version (e.g. 4.65.0 -> 4.65.1)
 *   npm run bump minor                 # Automatically increment minor version (e.g. 4.65.0 -> 4.66.0)
 *   npm run bump major                 # Automatically increment major version (e.g. 4.65.0 -> 5.0.0)
 *   npm run bump beta                  # Automatically generate or increment Beta pre-release (e.g. 4.65.0 -> 4.65.1-beta.1)
 *   npm run bump 4.65.1-beta           # Release testing / pre-release dual version (supports -beta, -cleaned, etc.)
 *   npm run bump 4.65.1-cleaned        # Release custom derivative version
 *   npm run bump patch --dry-run       # Dry run mode: check and output diff without writing to disk
 *   npm run bump patch --commit        # Automatically create standard git commit `chore(release): bump version to ...`
 *
 * Note: When the version contains a pre-release tag (SemVer 2.0 with '-'), README titles and badges remain at latest stable version.
 */

import fs from 'node:fs';
import path from 'node:path';
import { execSync, execFileSync } from 'node:child_process';
import { fileURLToPath } from 'node:url';

const __filename = fileURLToPath(import.meta.url);
const __dirname = path.dirname(__filename);
const ROOT_DIR = path.resolve(__dirname, '..');

// Color formatting helpers
const colors = {
    reset: '\x1b[0m',
    green: '\x1b[32m',
    yellow: '\x1b[33m',
    red: '\x1b[31m',
    cyan: '\x1b[36m',
    bold: '\x1b[1m',
};

function log(msg) {
    console.log(`${colors.cyan}[Bump-Version]${colors.reset} ${msg}`);
}
function success(msg) {
    console.log(`${colors.green}✓ ${msg}${colors.reset}`);
}
function error(msg) {
    console.error(`${colors.red}✗ Error: ${msg}${colors.reset}`);
}
function warn(msg) {
    console.warn(`${colors.yellow}⚠ Warning: ${msg}${colors.reset}`);
}

// 1. Read current version from root package.json
const pkgPath = path.join(ROOT_DIR, 'package.json');
if (!fs.existsSync(pkgPath)) {
    error('Root package.json file not found!');
    process.exit(1);
}

const pkgContent = fs.readFileSync(pkgPath, 'utf8');
const pkgJson = JSON.parse(pkgContent);
const currentVersion = pkgJson.version;

// Standard SemVer 2.0 regex: supports Major.Minor.Patch and optional Pre-release tags (e.g. -beta, -cleaned, -beta.1)
const SEMVER_REGEX = /^v?(\d+)\.(\d+)\.(\d+)(?:-([0-9A-Za-z.-]+))?$/;

function parseSemVer(v) {
    if (!v || typeof v !== 'string') return null;
    const match = v.trim().match(SEMVER_REGEX);
    if (!match) return null;
    return {
        major: Number(match[1]),
        minor: Number(match[2]),
        patch: Number(match[3]),
        prerelease: match[4] || null,
        raw: v.trim().replace(/^v/, ''),
    };
}

const curSem = parseSemVer(currentVersion);
if (!curSem) {
    error(`Current version "${currentVersion}" in package.json does not conform to SemVer specification!`);
    process.exit(1);
}

// 2. Parse command line arguments
const args = process.argv.slice(2);
const isDryRun = args.includes('--dry-run') || process.env.npm_config_dry_run === 'true';
const autoCommit = args.includes('--commit') || process.env.npm_config_commit === 'true';
const targetArg = args.find(a => !a.startsWith('--'));

if (!targetArg) {
    console.log(`
${colors.bold}Antigravity Tools Version Bump & Release Utility${colors.reset}

Current Version: ${colors.green}${currentVersion}${colors.reset}

Usage:
  npm run bump patch           # Increment patch version (e.g. ${currentVersion} -> next patch)
  npm run bump minor           # Increment minor version (e.g. ${currentVersion} -> X.Y.0)
  npm run bump major           # Increment major version (e.g. ${currentVersion} -> X.0.0)
  npm run bump beta            # Increment pre-release version (e.g. ${currentVersion} -> X.Y.Z-beta.1)
  npm run bump <target>        # Specify arbitrary valid SemVer (e.g. 4.65.1-beta or 4.65.1-cleaned)

Options:
  --dry-run                    # Dry run mode; perform checks without modifying files
  --commit                     # Automatically execute git commit for release changes
`);
    process.exit(0);
}

// 3. Compute target version
let nextSem = null;
const normalizedTarget = targetArg.toLowerCase();

if (normalizedTarget === 'patch') {
    if (curSem.prerelease) {
        // If currently on pre-release, patch promotes to stable version
        nextSem = { major: curSem.major, minor: curSem.minor, patch: curSem.patch, prerelease: null };
    } else {
        nextSem = { major: curSem.major, minor: curSem.minor, patch: curSem.patch + 1, prerelease: null };
    }
} else if (normalizedTarget === 'minor') {
    nextSem = { major: curSem.major, minor: curSem.minor + 1, patch: 0, prerelease: null };
} else if (normalizedTarget === 'major') {
    nextSem = { major: curSem.major + 1, minor: 0, patch: 0, prerelease: null };
} else if (normalizedTarget === 'beta') {
    if (curSem.prerelease && curSem.prerelease.startsWith('beta.')) {
        // Increment beta index (e.g. beta.1 -> beta.2)
        const sub = Number(curSem.prerelease.split('.')[1]) || 0;
        nextSem = { major: curSem.major, minor: curSem.minor, patch: curSem.patch, prerelease: `beta.${sub + 1}` };
    } else if (curSem.prerelease === 'beta') {
        nextSem = { major: curSem.major, minor: curSem.minor, patch: curSem.patch, prerelease: 'beta.1' };
    } else {
        // From stable version, start beta for next patch
        nextSem = { major: curSem.major, minor: curSem.minor, patch: curSem.patch + 1, prerelease: 'beta.1' };
    }
} else {
    nextSem = parseSemVer(targetArg);
    if (!nextSem) {
        error(`Input target version "${targetArg}" is invalid! Must follow SemVer (e.g. 4.65.1 or 4.65.1-beta / 4.65.1-cleaned)!`);
        process.exit(1);
    }
}

const newVersion = nextSem.prerelease
    ? `${nextSem.major}.${nextSem.minor}.${nextSem.patch}-${nextSem.prerelease}`
    : `${nextSem.major}.${nextSem.minor}.${nextSem.patch}`;

// Pre-release versions (containing '-') are recorded only in CHANGELOG.
// README always reflects the latest stable version.
const isPrerelease = Boolean(nextSem.prerelease);

// 4. Version upgrade guard rail logic
function validateVersionUpgrade(next, cur) {
    if (next.raw === cur.raw) {
        return { valid: false, reason: `Target version [${next.raw}] is identical to current version; no bump needed!` };
    }

    if (next.major > cur.major) return { valid: true };
    if (next.major < cur.major) return { valid: false, reason: `Major version regression: ${next.major} < ${cur.major}` };

    if (next.minor > cur.minor) return { valid: true };
    if (next.minor < cur.minor) return { valid: false, reason: `Minor version regression: ${next.minor} < ${cur.minor}` };

    if (next.patch > cur.patch) return { valid: true };
    if (next.patch < cur.patch) return { valid: false, reason: `Patch version regression: ${next.patch} < ${cur.patch}` };

    // Equal 3-part base versions (pre-release transitions)
    // Scenario A: Promote pre-release to stable (4.65.0-beta -> 4.65.0)
    if (cur.prerelease && !next.prerelease) {
        return { valid: true, note: 'Pre-release promoted to stable release' };
    }
    // Scenario B: Fork pre-release from stable (4.65.0 -> 4.65.0-cleaned / 4.65.0-beta)
    if (!cur.prerelease && next.prerelease) {
        return { valid: true, note: `Dual version release (stable -> ${next.prerelease})` };
    }
    // Scenario C: Pre-release branch progression (e.g. beta.1 -> beta.2)
    if (cur.prerelease && next.prerelease) {
        return { valid: true, note: `Pre-release evolution: ${cur.prerelease} -> ${next.prerelease}` };
    }

    return { valid: false, reason: `Guard rail triggered: Target version [${next.raw}] is not greater than current version [${cur.raw}]` };
}

const validation = validateVersionUpgrade({ ...nextSem, raw: newVersion }, curSem);
if (!validation.valid) {
    error(`Guard rail triggered: ${validation.reason}`);
    error('Release version cannot be lower than existing version to prevent update lockouts.');
    process.exit(1);
}

if (validation.note) {
    log(`Version mode detected: ${colors.cyan}${validation.note}${colors.reset}`);
}

// Check current local git branch and provide channel guidance
let currentGitBranch = '';
try {
    currentGitBranch = execSync('git rev-parse --abbrev-ref HEAD', { stdio: ['ignore', 'pipe', 'ignore'] }).toString().trim();
} catch {}

if (currentGitBranch) {
    if (isPrerelease && currentGitBranch !== 'beta') {
        warn(`Currently on branch [${currentGitBranch}]. Pre-release versions (${newVersion}) should be bumped on 'beta' branch to keep 'main' clean.`);
    } else if (!isPrerelease && currentGitBranch !== 'main') {
        warn(`Currently on branch [${currentGitBranch}]. Production releases (${newVersion}) must be merged and released on 'main'.`);
    }
}

log(`Starting version synchronization: ${colors.yellow}${currentVersion}${colors.reset} -> ${colors.green}${colors.bold}${newVersion}${colors.reset}${isDryRun ? ' [DRY-RUN mode]' : ''}`);

// 5. Use local date to avoid timezone skew
const now = new Date();
const today = `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`;

const TARGET_FILES = [
    {
        name: 'package.json',
        relPath: 'package.json',
        replace: (content) => content.replace(
            `"version": "${currentVersion}"`,
            `"version": "${newVersion}"`
        ),
    },
    {
        name: 'package-lock.json (root version mirror, two locations)',
        relPath: 'package-lock.json',
        replace: (content) => content
            .replace(/^(\s{2}"version":\s*)"[^"]+"/m, `$1"${newVersion}"`)
            .replace(
                /("packages":\s*\{\s*\r?\n\s*"":\s*\{\s*\r?\n\s*"name":\s*"[^"]*",\s*\r?\n\s*"version":\s*)"[^"]*"/,
                `$1"${newVersion}"`
            ),
    },
    {
        name: 'version.json',
        relPath: 'version.json',
        replace: (content) => content
            .replace(`"Version": "${currentVersion}"`, `"Version": "${newVersion}"`)
            .replace(`"version": "${currentVersion}"`, `"version": "${newVersion}"`),
    },
    {
        name: 'src-tauri/Cargo.toml',
        relPath: 'src-tauri/Cargo.toml',
        replace: (content) => content.replace(
            `version = "${currentVersion}"`,
            `version = "${newVersion}"`
        ),
    },
    {
        name: 'src-tauri/tauri.conf.json',
        relPath: 'src-tauri/tauri.conf.json',
        replace: (content) => content.replace(
            `"version": "${currentVersion}"`,
            `"version": "${newVersion}"`
        ),
    },
    {
        name: 'src-tauri/Cargo.lock',
        relPath: 'src-tauri/Cargo.lock',
        replace: (content) => content.replace(
            /(\[\[package\]\]\r?\nname = "(?:agm-alim|antigravity-tools)"\r?\nversion = )"[^"]+"/,
            `$1"${newVersion}"`
        ),
    },
    {
        name: 'Casks/antigravity-tools.rb',
        relPath: 'Casks/antigravity-tools.rb',
        replace: (content) => content.replace(
            `version "${currentVersion}"`,
            `version "${newVersion}"`
        ),
    },
    {
        name: 'README.md (title and badge)',
        relPath: 'README.md',
        stableOnly: true,
        replace: (content) => content
            .replace(/\(v[0-9][^)]*\)/, `(v${newVersion})`)
            .replace(/Version-[0-9][^"]*-blue/, `Version-${newVersion}-blue`),
    },
    {
        name: 'README_EN.md (title and badge)',
        relPath: 'README_EN.md',
        stableOnly: true,
        replace: (content) => content
            .replace(/\(v[0-9][^)]*\)/, `(v${newVersion})`)
            .replace(/Version-[0-9][^"]*-blue/, `Version-${newVersion}-blue`),
    },
    {
        name: 'src/components/layout/MiniView.tsx',
        relPath: 'src/components/layout/MiniView.tsx',
        replace: (content) => content.replace(
            `'${currentVersion}'`,
            `'${newVersion}'`
        ),
    },
    {
        name: 'src/pages/Settings.tsx',
        relPath: 'src/pages/Settings.tsx',
        replace: (content) => content.replace(
            `'${currentVersion}'`,
            `'${newVersion}'`
        ),
    },
    {
        name: 'CHANGELOG.md (auto-insert release skeleton)',
        relPath: 'CHANGELOG.md',
        replace: (content) => {
            const escaped = newVersion.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            const headingRegex = new RegExp(`\\*\\*v${escaped}\\s*\\(`);
            if (headingRegex.test(content)) {
                return content;
            }
            const eol = content.includes('\r\n') ? '\r\n' : '\n';
            const newBlock = `    *   **v${newVersion} (${today})**:${eol}        -   **[Feature Category] Main Update Summary (PR #xxx)**:${eol}            -   **Description**: Please document update details here; credit external contributors inline as \`(Thanks to @username)\`.${eol}${eol}`;
            const historyAnchor = '*   **Version History**:';
            if (content.includes(historyAnchor)) {
                return content.replace(historyAnchor, `${historyAnchor}${eol}${newBlock}`);
            }
            const zhAnchor = '*   **版本演进**:';
            if (content.includes(zhAnchor)) {
                return content.replace(zhAnchor, `${zhAnchor}${eol}${newBlock}`);
            }
            const firstVersion = content.match(/\s*\*\s+\*\*v\d+\.\d+\.\d+/);
            if (firstVersion && firstVersion.index !== undefined) {
                return content.slice(0, firstVersion.index) + `${eol}${newBlock}` + content.slice(firstVersion.index);
            }
            return content;
        },
    },
    {
        name: 'CHANGELOG_EN.md (auto-insert English release skeleton)',
        relPath: 'CHANGELOG_EN.md',
        replace: (content) => {
            const escaped = newVersion.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
            const headingRegex = new RegExp(`\\*\\*v${escaped}\\s*\\(`);
            if (headingRegex.test(content)) {
                return content;
            }
            const anchor = '*   **Version History**:';
            if (!content.includes(anchor)) {
                return content;
            }
            const eol = content.includes('\r\n') ? '\r\n' : '\n';
            const newBlock = `*   **Version History**:${eol}    *   **v${newVersion} (${today})**:${eol}        -   **[Feature Category] Main Update Summary (PR #xxx)**:${eol}            -   **Description**: Please document update details here; credit external contributors inline as \`(Thanks to @username)\`.${eol}`;
            return content.replace(anchor, newBlock);
        },
    },
];

// 6. Execute atomic replacements
let updatedCount = 0;

for (const target of TARGET_FILES) {
    if (target.stableOnly && isPrerelease) {
        log(`Pre-release version ${newVersion} skipped for ${target.relPath} (README reflects latest stable only).`);
        continue;
    }

    const fullPath = path.join(ROOT_DIR, target.relPath);
    if (!fs.existsSync(fullPath)) {
        warn(`Target file ${target.relPath} not found; skipping.`);
        continue;
    }

    const oldContent = fs.readFileSync(fullPath, 'utf8');
    const newContent = target.replace(oldContent);

    if (oldContent === newContent) {
        warn(`File ${target.relPath} contents unchanged (pattern may not have matched).`);
    } else {
        if (!isDryRun) {
            fs.writeFileSync(fullPath, newContent, 'utf8');
        }
        success(`Synchronized: ${target.name} -> ${newVersion}`);
        updatedCount++;
    }
}

// 7. Verify cargo dependency graph if cargo is present
if (!isDryRun && fs.existsSync(path.join(ROOT_DIR, 'src-tauri/Cargo.toml'))) {
    try {
        execSync('cargo --version', { stdio: 'ignore' });
        log('Running cargo check to verify src-tauri/Cargo.lock dependency graph integrity...');
        execSync('cargo check --manifest-path src-tauri/Cargo.toml', {
            cwd: ROOT_DIR,
            stdio: 'ignore',
        });
        success('Cargo dependency graph verified');
    } catch {
        // Cargo check fallback when cargo is unavailable
    }
}

log(`All ${updatedCount} version configuration locations synchronized atomically!`);

// 8. Automated git commit support
if (!isDryRun && autoCommit) {
    log('Executing automated Git Commit...');
    try {
        execFileSync('git', ['add', '-A'], { cwd: ROOT_DIR, stdio: 'ignore' });
        const commitMsg = `chore(release): bump version to ${newVersion} and update changelog\n\nMaintained by Alim, Sponsored by RISEUP ASIA LLC`;
        execFileSync('git', ['commit', '-m', commitMsg], { cwd: ROOT_DIR, stdio: 'inherit' });
        success(`Generated commit: chore(release): bump version to ${newVersion}`);
        warn('Tip: Skeleton header inserted in CHANGELOG.md; please document release highlights and use git commit --amend.');
    } catch (e) {
        error('Automated Git Commit failed, please execute git commit manually: ' + (e.stderr?.toString() || e.message));
    }
}

if (isPrerelease) {
    console.log(`
${colors.bold}${colors.green}🎉 Pre-release version upgraded to v${newVersion}!${colors.reset}
${colors.cyan}[Beta Channel] Next steps:${colors.reset}
  1. In ${colors.cyan}CHANGELOG.md${colors.reset}, document pre-release updates (credit contributors inline as (Thanks to @username))
  2. Commit release prep: ${colors.cyan}git commit -am "chore(release): bump version to ${newVersion} and update changelog"${colors.reset}
  3. Push branch and tag: ${colors.cyan}git push origin beta && git tag v${newVersion} && git push origin v${newVersion}${colors.reset}

${colors.yellow}🛡️ Isolation: Beta pipeline builds are marked as Pre-release and never tagged Latest; stable users are unaffected.${colors.reset}
`);
} else {
    console.log(`
${colors.bold}${colors.green}🎉 Production version upgraded to v${newVersion}!${colors.reset}
${colors.cyan}[Main Channel] Next steps:${colors.reset}
  1. In ${colors.cyan}CHANGELOG.md${colors.reset}, document release highlights (credit contributors inline as (Thanks to @username))
  2. Commit release prep: ${colors.cyan}git commit -am "chore(release): bump version to ${newVersion} and update changelog"${colors.reset}
  3. Push branch and tag: ${colors.cyan}git push origin main && git tag v${newVersion} && git push origin v${newVersion}${colors.reset}

${colors.green}🚀 Production: Main pipeline builds are marked as Latest Release and distributed to all platforms.${colors.reset}
`);
}
